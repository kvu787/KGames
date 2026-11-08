// How the modeler edits the mesh, as Blender's edit mode does: vertices,
// edges or faces are selected, by a click, a drag of a box, or a loop; and the
// selection is moved, turned and scaled by the pointer, along an axis or
// freely, or extruded, inset, cut, merged, filled, deleted, creased or
// painted. Each edit is a step of the document's history.
//
// Moving, turning, scaling and insetting are modal, as in Blender: they
// follow the pointer from where they began until a click or Enter keeps them,
// or a right-click or Esc takes them back.

use math::{Vector3, add, dot, length, normalize, scale, subtract};
use polygon_mesh::{Edge, Hit, Mesh, SelectMode, Selection, SubdivisionSurface, Triangles, primitives};
use user_interface::{Point, Rect};

use crate::camera::Camera;
use crate::document::{Document, Snapshot};

// How near the pointer must be to a vertex or an edge to pick it.
pub const PICK_RADIUS: f32 = 12.0;
// How far the pointer moves before a press on nothing becomes a box.
pub const DRAG_THRESHOLD: f32 = 4.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Mode {
    Vertices,
    Edges,
    #[default]
    Faces,
}

impl Mode {
    pub const ALL: [Mode; 3] = [Mode::Vertices, Mode::Edges, Mode::Faces];

    pub fn select_mode(self) -> SelectMode {
        match self {
            Mode::Vertices => SelectMode::Vertices,
            Mode::Edges => SelectMode::Edges,
            Mode::Faces => SelectMode::Faces,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Mode::Vertices => "Vertex",
            Mode::Edges => "Edge",
            Mode::Faces => "Face",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Tool {
    #[default]
    Select,
    // Paints the faces that the pointer presses or drags over.
    Paint,
    // Cuts a loop across the ring of quads under the pointer.
    LoopCut,
}

impl Tool {
    pub const ALL: [Tool; 3] = [Tool::Select, Tool::Paint, Tool::LoopCut];

    pub fn name(self) -> &'static str {
        match self {
            Tool::Select => "Select",
            Tool::Paint => "Paint",
            Tool::LoopCut => "Loop cut",
        }
    }
}

// What a modal edit is limited to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Axis {
    #[default]
    Free,
    X,
    Y,
    Z,
    // Along the selection's normal, as an extrusion moves.
    Normal,
}

impl Axis {
    pub fn name(self) -> &'static str {
        match self {
            Axis::Free => "Free",
            Axis::X => "X",
            Axis::Y => "Y",
            Axis::Z => "Z",
            Axis::Normal => "Normal",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OperationKind {
    Move,
    Rotate,
    Scale,
    Inset,
}

impl OperationKind {
    pub fn name(self) -> &'static str {
        match self {
            OperationKind::Move => "Move",
            OperationKind::Rotate => "Rotate",
            OperationKind::Scale => "Scale",
            OperationKind::Inset => "Inset",
        }
    }
}

// A modal edit under way.
#[derive(Clone, Debug)]
pub struct Operation {
    pub kind: OperationKind,
    pub axis: Axis,
    // What its history step is called, like "Extrude" for a move that
    // follows an extrusion.
    pub name: String,
    // Where the pointer was as it began, and where the pivot is on screen.
    start: Point,
    pivot_screen: Point,
    pivot: Vector3,
    normal: Vector3,
    // What a cancel goes back to, and what the edit applies to.
    restore: Snapshot,
    base: Mesh,
    vertices: Vec<u32>,
    faces: Vec<usize>,
    // What it's done, for the status bar: like "0.25" or "45°".
    pub amount: String,
}

// What the view draws, which a ray from the pointer goes through: the
// surface, when the mesh is subdivided, and the triangles, of it or of the
// mesh.
#[derive(Clone, Copy)]
pub struct Drawn<'a> {
    pub surface: Option<&'a SubdivisionSurface>,
    pub triangles: &'a Triangles,
}

impl Drawn<'_> {
    // The nearest face that the ray hits, as one of the mesh's, through the
    // surface's when the mesh is subdivided.
    pub fn hit(&self, origin: Vector3, direction: Vector3) -> Option<Hit> {
        self.triangles
            .ray_cast(origin, direction)
            .map(|hit| Hit { face: self.surface.map_or(hit.face, |surface| surface.base_faces[hit.face]), ..hit })
    }
}

// A vertex, an edge or a face under the pointer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Element {
    Vertex(u32),
    Edge(Edge),
    Face(usize),
}

pub struct Editor {
    pub mode: Mode,
    pub tool: Tool,
    pub operation: Option<Operation>,
    pub hover: Option<Element>,
    // A press on the view that may become a box, and the box's corners.
    press: Option<Point>,
    pub selecting_box: Option<(Point, Point)>,
    // The loop that the loop cut tool would cut: the edge and the fraction.
    pub loop_preview: Option<(Edge, f32)>,
    // A paint stroke under way, from the document as it was.
    painting: Option<Snapshot>,
    pub active_paint: usize,
    // What an edit said last, like why a face couldn't be made.
    pub message: Option<String>,
}

impl Default for Editor {
    fn default() -> Self {
        Self {
            mode: Mode::Faces,
            tool: Tool::Select,
            operation: None,
            hover: None,
            press: None,
            selecting_box: None,
            loop_preview: None,
            painting: None,
            active_paint: 0,
            message: None,
        }
    }
}

fn axis_vector(axis: Axis, normal: Vector3) -> Option<Vector3> {
    match axis {
        Axis::Free => None,
        Axis::X => Some([1.0, 0.0, 0.0]),
        Axis::Y => Some([0.0, 1.0, 0.0]),
        Axis::Z => Some([0.0, 0.0, 1.0]),
        Axis::Normal => Some(normal),
    }
}

fn angle_of(point: Point, center: Point) -> f32 {
    (center.y - point.y).atan2(point.x - center.x)
}

// The distance from a point to a segment, on screen.
fn segment_distance(point: Point, a: Point, b: Point) -> (f32, f32) {
    let along = b - a;
    let length_squared = along.x * along.x + along.y * along.y;
    let t = if length_squared > 0.0 { (((point.x - a.x) * along.x + (point.y - a.y) * along.y) / length_squared).clamp(0.0, 1.0) } else { 0.0 };
    (point.distance(a.lerp(b, t)), t)
}

impl Editor {
    // Forgets what's under way and the paint picked, for another document,
    // keeping the selection mode.
    pub fn reset(&mut self) {
        *self = Editor { mode: self.mode, ..Editor::default() };
    }

    // Switches the selection mode, converting the selection as Blender does.
    pub fn set_mode(&mut self, document: &mut Document, mode: Mode) {
        if self.mode == mode {
            return;
        }
        self.mode = mode;
        document.selection = document.selection.in_mode(&document.mesh, mode.select_mode());
        document.touch();
    }

    // The element under the pointer, in the selection mode: the nearest
    // vertex or edge within PICK_RADIUS pixels, or the face hit, that the
    // mesh doesn't hide, unless in X-ray or over a subdivided surface.
    pub fn pick(&self, document: &Document, camera: &Camera, view: Rect, pointer: Point, see_through: bool, drawn: Drawn) -> Option<Element> {
        let mesh = &document.mesh;
        let (origin, direction) = camera.ray(pointer, view);
        let hit = drawn.hit(origin, direction);
        let surface = drawn.surface;
        let hidden = |point: Vector3| -> bool {
            if see_through || surface.is_some() {
                return false;
            }
            let distance = dot(subtract(point, origin), direction);
            hit.as_ref().is_some_and(|hit| hit.distance < distance - camera.pixel_size(view) * 3.0)
        };
        match self.mode {
            Mode::Vertices => {
                let mut best: Option<(f32, u32)> = None;
                for vertex in 0..mesh.positions.len() as u32 {
                    let (screen, _) = camera.to_screen(mesh.position(vertex), view);
                    let distance = screen.distance(pointer);
                    if distance <= PICK_RADIUS && best.is_none_or(|(nearest, _)| distance < nearest) && !hidden(mesh.position(vertex)) {
                        best = Some((distance, vertex));
                    }
                }
                best.map(|(_, vertex)| Element::Vertex(vertex))
            }
            Mode::Edges => {
                let mut best: Option<(f32, Edge)> = None;
                for edge in document.edges().iter().copied() {
                    let [a, b] = edge.vertices();
                    let (sa, _) = camera.to_screen(mesh.position(a), view);
                    let (sb, _) = camera.to_screen(mesh.position(b), view);
                    let (distance, t) = segment_distance(pointer, sa, sb);
                    let point = add(mesh.position(a), scale(subtract(mesh.position(b), mesh.position(a)), t));
                    if distance <= PICK_RADIUS && best.is_none_or(|(nearest, _)| distance < nearest) && !hidden(point) {
                        best = Some((distance, edge));
                    }
                }
                best.map(|(_, edge)| Element::Edge(edge))
            }
            Mode::Faces => hit.map(|hit| Element::Face(hit.face)),
        }
    }

    // A click on the element, or on nothing: picks it, adds it to the
    // selection or takes it away with Shift, or selects nothing.
    pub fn click(&mut self, document: &mut Document, element: Option<Element>, extend: bool) {
        let before = document.snapshot();
        let mesh = &document.mesh;
        let picked = match element {
            Some(Element::Vertex(vertex)) => Selection::from_vertices(mesh, [vertex]),
            Some(Element::Edge(edge)) => Selection::from_edges(mesh, [edge]),
            Some(Element::Face(face)) => Selection::from_faces(mesh, [face]),
            None => Selection::new(),
        };
        document.selection = if extend {
            let already = match element {
                Some(Element::Vertex(vertex)) => document.selection.vertices.contains(&vertex),
                Some(Element::Edge(edge)) => document.selection.edges.contains(&edge),
                Some(Element::Face(face)) => document.selection.faces.contains(&face),
                None => false,
            };
            if already {
                self.without(document, element.unwrap())
            } else {
                document.selection.union(&picked, mesh, self.mode.select_mode())
            }
        } else {
            picked
        };
        document.commit("Select", before);
    }

    // The selection less an element.
    fn without(&self, document: &Document, element: Element) -> Selection {
        let mesh = &document.mesh;
        let selection = &document.selection;
        match element {
            Element::Vertex(vertex) => Selection::from_vertices(mesh, selection.vertices.iter().copied().filter(|other| *other != vertex)),
            Element::Edge(edge) => Selection::from_edges(mesh, selection.edges.iter().copied().filter(|other| *other != edge)),
            Element::Face(face) => Selection::from_faces(mesh, selection.faces.iter().copied().filter(|other| *other != face)),
        }
    }

    // Selects the loop through an edge: an edge loop, or in face mode, the
    // loop of faces across it.
    pub fn select_loop(&mut self, document: &mut Document, edge: Edge, extend: bool) {
        let before = document.snapshot();
        let mesh = &document.mesh;
        let picked = if self.mode == Mode::Faces { Selection::face_loop(mesh, edge) } else { Selection::edge_loop(mesh, edge) };
        document.selection = if extend { document.selection.union(&picked, mesh, self.mode.select_mode()) } else { picked };
        document.commit("Select loop", before);
    }

    // The edge nearest the pointer, whatever the mode, for loops.
    pub fn nearest_edge(&self, document: &Document, camera: &Camera, view: Rect, pointer: Point) -> Option<(Edge, f32)> {
        let mesh = &document.mesh;
        let mut best: Option<(f32, Edge, f32)> = None;
        for edge in document.edges().iter().copied() {
            let [a, b] = edge.vertices();
            let (sa, _) = camera.to_screen(mesh.position(a), view);
            let (sb, _) = camera.to_screen(mesh.position(b), view);
            let (distance, t) = segment_distance(pointer, sa, sb);
            if distance <= PICK_RADIUS * 2.0 && best.is_none_or(|(nearest, _, _)| distance < nearest) {
                best = Some((distance, edge, t));
            }
        }
        best.map(|(_, edge, t)| (edge, t))
    }

    // Selects what's in the box on screen: vertices inside it, and edges and
    // faces whose vertices all are. Shift adds to the selection, and Ctrl
    // takes away from it.
    pub fn select_box(&mut self, document: &mut Document, camera: &Camera, view: Rect, corners: (Point, Point), extend: bool, subtract: bool) {
        let before = document.snapshot();
        let area = Rect::from_corners(corners.0, corners.1);
        let mesh = &document.mesh;
        let inside: Vec<u32> = (0..mesh.positions.len() as u32).filter(|vertex| area.contains(camera.to_screen(mesh.position(*vertex), view).0)).collect();
        let boxed = match self.mode {
            Mode::Vertices => Selection::from_vertices(mesh, inside.iter().copied()),
            Mode::Edges => Selection::from_edges(mesh, document.edges().iter().copied().filter(|edge| edge.vertices().iter().all(|vertex| inside.contains(vertex)))),
            Mode::Faces => Selection::from_faces(mesh, (0..mesh.faces.len()).filter(|face| mesh.faces[*face].vertices.iter().all(|vertex| inside.contains(vertex)))),
        };
        let mode = self.mode.select_mode();
        document.selection = if subtract {
            let current = document.selection.in_mode(mesh, mode);
            match self.mode {
                Mode::Vertices => Selection::from_vertices(mesh, current.vertices.difference(&boxed.vertices).copied()),
                Mode::Edges => Selection::from_edges(mesh, current.edges.difference(&boxed.edges).copied()),
                Mode::Faces => Selection::from_faces(mesh, current.faces.difference(&boxed.faces).copied()),
            }
        } else if extend {
            document.selection.union(&boxed, mesh, mode)
        } else {
            boxed
        };
        document.commit("Select box", before);
    }

    // A press, a drag and a release of the left button on the view with the
    // Select tool: a click selects, and a drag selects a box.
    pub fn press(&mut self, point: Point) {
        self.press = Some(point);
        self.selecting_box = None;
    }

    pub fn drag(&mut self, point: Point) {
        if let Some(start) = self.press
            && (self.selecting_box.is_some() || start.distance(point) > DRAG_THRESHOLD)
        {
            self.selecting_box = Some((start, point));
        }
    }

    // Ends a press: returns the box, if it became one.
    pub fn release(&mut self) -> Option<(Point, Point)> {
        self.press = None;
        self.selecting_box.take()
    }

    pub fn pressing(&self) -> bool {
        self.press.is_some()
    }

    // The vertices that the selection moves.
    fn moving_vertices(document: &Document) -> Vec<u32> {
        document.selection.vertices.iter().copied().collect()
    }

    // The selection's average normal: its faces', or else the faces' around
    // its vertices.
    fn selection_normal(document: &Document) -> Vector3 {
        let mesh = &document.mesh;
        let mut faces: Vec<usize> = document.selection.faces.iter().copied().collect();
        if faces.is_empty() {
            faces = (0..mesh.faces.len()).filter(|face| mesh.faces[*face].vertices.iter().any(|vertex| document.selection.vertices.contains(vertex))).collect();
        }
        let sum = faces.iter().fold([0.0; 3], |sum, face| add(sum, mesh.face_area_vector(*face)));
        if length(sum) > 1e-9 { normalize(sum) } else { [0.0, 1.0, 0.0] }
    }

    // Begins a modal edit of the selection from the pointer.
    pub fn begin(&mut self, document: &mut Document, kind: OperationKind, pointer: Point, camera: &Camera, view: Rect) -> bool {
        self.begin_from(document, kind, pointer, camera, view, document.snapshot(), kind.name())
    }

    #[allow(clippy::too_many_arguments)]
    fn begin_from(&mut self, document: &mut Document, kind: OperationKind, pointer: Point, camera: &Camera, view: Rect, restore: Snapshot, name: &str) -> bool {
        let vertices = Self::moving_vertices(document);
        if vertices.is_empty() || (kind == OperationKind::Inset && document.selection.faces.is_empty()) {
            self.message = Some(format!("{} needs a selection{}.", kind.name(), if kind == OperationKind::Inset { " of faces" } else { "" }));
            return false;
        }
        let pivot = document.mesh.center_of(vertices.iter().copied()).unwrap_or([0.0; 3]);
        let (pivot_screen, _) = camera.to_screen(pivot, view);
        // A pointer on the pivot would scale and turn by nothing, so they
        // start a little away from it.
        let mut start = pointer;
        if matches!(kind, OperationKind::Scale | OperationKind::Rotate | OperationKind::Inset) && start.distance(pivot_screen) < 20.0 {
            start = Point::new(pivot_screen.x + 100.0, pivot_screen.y);
        }
        self.operation = Some(Operation {
            kind,
            axis: Axis::Free,
            name: name.to_string(),
            start,
            pivot_screen,
            pivot,
            normal: Self::selection_normal(document),
            restore,
            base: document.mesh.clone(),
            faces: document.selection.faces.iter().copied().collect(),
            vertices,
            amount: String::new(),
        });
        self.message = None;
        true
    }

    // Limits the edit to an axis, or frees it if it's that axis already.
    pub fn constrain(&mut self, axis: Axis) {
        if let Some(operation) = &mut self.operation {
            operation.axis = if operation.axis == axis { Axis::Free } else { axis };
        }
    }

    // Applies the edit for where the pointer is now; with `snap`, to whole
    // tenths, or 15 degrees.
    pub fn update(&mut self, document: &mut Document, pointer: Point, snap: bool, camera: &Camera, view: Rect) {
        let Some(operation) = &mut self.operation else {
            return;
        };
        let mut mesh = operation.base.clone();
        let vertices = operation.vertices.iter().copied();
        let snap_to = |value: f32, step: f32| if snap { (value / step).round() * step } else { value };
        match operation.kind {
            OperationKind::Move => {
                let delta = pointer - operation.start;
                let offset = match axis_vector(operation.axis, operation.normal) {
                    None => {
                        let size = camera.pixel_size(view);
                        let world = add(scale(camera.right(), delta.x * size), scale(camera.up(), -delta.y * size));
                        world.map(|value| snap_to(value, 0.1))
                    }
                    Some(axis) => {
                        // How far along the axis, from how far the pointer
                        // moved along the axis's line on screen.
                        let (tip, _) = camera.to_screen(add(operation.pivot, axis), view);
                        let along = tip - operation.pivot_screen;
                        let pixels = along.x * along.x + along.y * along.y;
                        let amount = if pixels > 1.0 { (delta.x * along.x + delta.y * along.y) / pixels } else { 0.0 };
                        scale(axis, snap_to(amount, 0.1))
                    }
                };
                mesh.translate(vertices, offset);
                operation.amount = format!("{:.3}, {:.3}, {:.3}", offset[0], offset[1], offset[2]);
            }
            OperationKind::Rotate => {
                let angle = angle_of(pointer, operation.pivot_screen) - angle_of(operation.start, operation.pivot_screen);
                let angle = snap_to(angle.to_degrees(), 15.0).to_radians();
                let backward = camera.backward();
                let axis = match axis_vector(operation.axis, operation.normal) {
                    // Counterclockwise on screen turns about the axis toward
                    // the camera.
                    None => backward,
                    Some(axis) if dot(axis, backward) < 0.0 => scale(axis, -1.0),
                    Some(axis) => axis,
                };
                mesh.rotate(vertices, operation.pivot, axis, angle);
                operation.amount = format!("{:.1}°", angle.to_degrees());
            }
            OperationKind::Scale => {
                let initial = operation.start.distance(operation.pivot_screen).max(1.0);
                let factor = snap_to(pointer.distance(operation.pivot_screen) / initial, 0.1);
                let factors = match operation.axis {
                    Axis::X => [factor, 1.0, 1.0],
                    Axis::Y => [1.0, factor, 1.0],
                    Axis::Z => [1.0, 1.0, factor],
                    Axis::Free | Axis::Normal => [factor; 3],
                };
                mesh.scale(vertices, operation.pivot, factors);
                operation.amount = format!("{factor:.3}");
            }
            OperationKind::Inset => {
                let initial = operation.start.distance(operation.pivot_screen);
                let thickness = snap_to(((initial - pointer.distance(operation.pivot_screen)) * camera.pixel_size(view)).max(0.0), 0.05);
                let selection = mesh.inset_region(operation.faces.iter().copied(), thickness);
                document.selection = selection;
                operation.amount = format!("{thickness:.3}");
            }
        }
        document.mesh = mesh;
        document.touch();
    }

    // Keeps the edit, as one step of the history.
    pub fn confirm(&mut self, document: &mut Document) {
        if let Some(operation) = self.operation.take() {
            document.commit(&operation.name, operation.restore);
        }
    }

    // Takes the edit back, and whatever began it, like an extrusion.
    pub fn cancel(&mut self, document: &mut Document) {
        if let Some(operation) = self.operation.take() {
            document.restore(operation.restore);
        }
    }

    // Extrudes the selected faces, or else edges, and moves what it made
    // along their normal.
    pub fn extrude(&mut self, document: &mut Document, pointer: Point, camera: &Camera, view: Rect) -> bool {
        let restore = document.snapshot();
        let selection = if !document.selection.faces.is_empty() {
            let faces: Vec<usize> = document.selection.faces.iter().copied().collect();
            document.mesh.extrude_faces(faces)
        } else if !document.selection.edges.is_empty() {
            let edges: Vec<Edge> = document.selection.edges.iter().copied().collect();
            document.mesh.extrude_edges(edges)
        } else {
            self.message = Some("Extrude needs faces or edges selected.".to_string());
            return false;
        };
        document.selection = selection;
        document.touch();
        let faces = !document.selection.faces.is_empty();
        let began = self.begin_from(document, OperationKind::Move, pointer, camera, view, restore, "Extrude");
        if began && faces {
            self.constrain(Axis::Normal);
        }
        began
    }

    // Begins insetting the selected faces, as a region.
    pub fn inset(&mut self, document: &mut Document, pointer: Point, camera: &Camera, view: Rect) -> bool {
        self.begin(document, OperationKind::Inset, pointer, camera, view)
    }

    // An instant edit of the selection, as one step of the history, which
    // says why if it can't be done.
    pub fn apply(&mut self, document: &mut Document, name: &str, edit: impl FnOnce(&mut Mesh, &Selection) -> Result<Selection, String>) {
        let before = document.snapshot();
        let selection = document.selection.clone();
        match edit(&mut document.mesh, &selection) {
            Ok(selection) => {
                document.selection = selection;
                document.commit(name, before);
                self.message = None;
            }
            Err(message) => {
                document.restore(before);
                self.message = Some(message);
            }
        }
    }

    pub fn delete(&mut self, document: &mut Document) {
        let mode = self.mode;
        self.apply(document, "Delete", |mesh, selection| {
            if selection.is_empty() {
                return Err("Delete needs a selection.".to_string());
            }
            Ok(match mode {
                Mode::Vertices => mesh.delete_vertices(selection.vertices.iter().copied()),
                Mode::Edges => mesh.delete_edges(selection.edges.iter().copied()),
                Mode::Faces => mesh.delete_faces(selection.faces.iter().copied()),
            })
        });
    }

    pub fn delete_only_faces(&mut self, document: &mut Document) {
        self.apply(document, "Delete faces", |mesh, selection| {
            if selection.faces.is_empty() {
                return Err("Delete faces needs faces selected.".to_string());
            }
            Ok(mesh.delete_only_faces(selection.faces.iter().copied()))
        });
    }

    // Makes a face of the selected vertices, or fills the loop of the
    // selected boundary edges.
    pub fn fill(&mut self, document: &mut Document) {
        let mode = self.mode;
        self.apply(document, "Fill", |mesh, selection| {
            if mode == Mode::Edges && !selection.edges.is_empty() {
                mesh.fill(selection.edges.iter().copied())
            } else {
                mesh.make_face(selection.vertices.iter().copied())
            }
        });
    }

    pub fn merge(&mut self, document: &mut Document) {
        self.apply(document, "Merge at center", |mesh, selection| {
            if selection.vertices.len() < 2 {
                return Err("Merge needs two vertices or more selected.".to_string());
            }
            Ok(mesh.merge_at_center(selection.vertices.iter().copied()))
        });
    }

    pub fn subdivide(&mut self, document: &mut Document) {
        self.apply(document, "Subdivide", |mesh, selection| {
            if selection.faces.is_empty() {
                return Err("Subdivide needs faces selected.".to_string());
            }
            Ok(mesh.subdivide_faces(selection.faces.iter().copied()))
        });
    }

    pub fn flip(&mut self, document: &mut Document) {
        self.apply(document, "Flip normals", |mesh, selection| {
            if selection.faces.is_empty() {
                return Err("Flip needs faces selected.".to_string());
            }
            mesh.flip_faces(selection.faces.iter().copied());
            Ok(selection.clone())
        });
    }

    pub fn recalculate_normals(&mut self, document: &mut Document) {
        self.apply(document, "Recalculate normals", |mesh, selection| {
            let faces: Vec<usize> = if selection.faces.is_empty() { (0..mesh.faces.len()).collect() } else { selection.faces.iter().copied().collect() };
            mesh.recalculate_normals_outside(faces);
            Ok(selection.clone())
        });
    }

    // Sets the selected edges' crease sharpness.
    pub fn crease(&mut self, document: &mut Document, sharpness: f32) {
        self.apply(document, "Crease", |mesh, selection| {
            if selection.edges.is_empty() {
                return Err("Crease needs edges selected.".to_string());
            }
            mesh.crease_edges(selection.edges.iter().copied(), sharpness);
            Ok(selection.clone())
        });
    }

    // Paints the selected faces with the active paint.
    pub fn assign_paint(&mut self, document: &mut Document) {
        let paint = self.active_paint as u32;
        self.apply(document, "Paint", |mesh, selection| {
            if selection.faces.is_empty() {
                return Err("Paint needs faces selected.".to_string());
            }
            mesh.assign_material(selection.faces.iter().copied(), paint);
            Ok(selection.clone())
        });
    }

    // A stroke of the Paint tool: each face the pointer presses or drags
    // over takes the active paint; the whole stroke is one step.
    pub fn paint_at(&mut self, document: &mut Document, face: Option<usize>) {
        if self.painting.is_none() {
            self.painting = Some(document.snapshot());
        }
        if let Some(face) = face
            && document.mesh.faces.get(face).is_some_and(|painted| painted.material != self.active_paint as u32)
        {
            document.mesh.assign_material([face], self.active_paint as u32);
            document.touch();
        }
    }

    pub fn end_paint(&mut self, document: &mut Document) {
        if let Some(before) = self.painting.take() {
            document.commit("Paint", before);
        }
    }

    // Cuts the loop through the edge, at the fraction along it from its
    // smaller vertex, and selects the new loop.
    pub fn loop_cut(&mut self, document: &mut Document, edge: Edge, fraction: f32) {
        self.apply(document, "Loop cut", |mesh, _| mesh.loop_cut(edge, fraction));
        if self.message.is_none() {
            // The cut's edges, in edge mode, are what Blender selects.
            let mode = if self.mode == Mode::Faces { Mode::Edges } else { self.mode };
            self.mode = mode;
        }
    }

    // The edges that a loop cut through the edge would make, to preview.
    pub fn preview_loop(document: &Document, edge: Edge, fraction: f32) -> Vec<[Vector3; 2]> {
        let mut mesh = document.mesh.clone();
        match mesh.loop_cut(edge, fraction) {
            Ok(selection) => selection
                .edges
                .iter()
                .map(|edge| {
                    let [a, b] = edge.vertices();
                    [mesh.position(a), mesh.position(b)]
                })
                .collect(),
            Err(_) => Vec::new(),
        }
    }

    // Adds a primitive to the mesh, around the origin, and selects it.
    pub fn add_primitive(&mut self, document: &mut Document, kind: Primitive) {
        let before = document.snapshot();
        let mut added = kind.make();
        let paint = self.active_paint as u32;
        for face in &mut added.faces {
            face.material = paint;
        }
        let offset = document.mesh.positions.len() as u32;
        let first_face = document.mesh.faces.len();
        document.mesh.positions.extend(added.positions);
        for face in added.faces {
            let vertices = face.vertices.iter().map(|vertex| vertex + offset).collect();
            document.mesh.add_face(vertices, face.material);
        }
        for (edge, sharpness) in added.creases {
            let [a, b] = edge.vertices();
            document.mesh.set_crease(Edge::new(a + offset, b + offset), sharpness);
        }
        document.selection = Selection::from_faces(&document.mesh, first_face..document.mesh.faces.len());
        document.commit(&format!("Add {}", kind.name()), before);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Primitive {
    Cube,
    Plane,
    Cylinder,
    Cone,
    Sphere,
    Torus,
}

impl Primitive {
    pub const ALL: [Primitive; 6] = [Primitive::Cube, Primitive::Plane, Primitive::Cylinder, Primitive::Cone, Primitive::Sphere, Primitive::Torus];

    pub fn name(self) -> &'static str {
        match self {
            Primitive::Cube => "Cube",
            Primitive::Plane => "Plane",
            Primitive::Cylinder => "Cylinder",
            Primitive::Cone => "Cone",
            Primitive::Sphere => "UV sphere",
            Primitive::Torus => "Torus",
        }
    }

    pub fn make(self) -> Mesh {
        match self {
            Primitive::Cube => primitives::cube(2.0),
            Primitive::Plane => primitives::plane([2.0, 2.0], [1, 1]),
            Primitive::Cylinder => primitives::cylinder(1.0, 2.0, 16, 1),
            Primitive::Cone => primitives::cone(1.0, 0.0, 2.0, 16),
            Primitive::Sphere => primitives::uv_sphere(1.0, 24, 12),
            Primitive::Torus => primitives::torus(1.0, 0.35, 24, 12),
        }
    }
}
