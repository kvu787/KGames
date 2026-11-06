// What the view draws over the mesh: the floor's grid and axes, the cage's
// edges and vertices, the selected faces' tint, the element under the
// pointer, and the loop that the loop cut tool would cut. Each is an
// OverlayItem for Viewport, in the theme's colors: the accent for what's
// selected, as Fluent marks selection, and its data colors for the axes and
// creases.

use math::Vector3;
use polygon_mesh::{Edge, INFINITELY_SHARP, SubdivisionSurface, Triangles};
use user_interface::{Color, Theme};

use crate::document::Document;
use crate::editing::{Editor, Element, Mode, Tool};
use crate::viewport::{KIND_LINE, KIND_POINT, KIND_TRIANGLE, OverlayItem};

// The grid: lines a unit apart, out to this many units from the origin.
pub const GRID_REACH: i32 = 10;

// Widths of lines and sizes of dots, in pixels.
const GRID_WIDTH: f32 = 1.0;
const AXIS_WIDTH: f32 = 2.0;
const EDGE_WIDTH: f32 = 1.5;
const SELECTED_EDGE_WIDTH: f32 = 2.5;
const HOVER_WIDTH: f32 = 3.0;
const VERTEX_SIZE: f32 = 5.0;
const SELECTED_VERTEX_SIZE: f32 = 7.0;
const HOVER_VERTEX_SIZE: f32 = 9.0;

pub fn line(a: Vector3, b: Vector3, width: f32, color: Color) -> OverlayItem {
    OverlayItem {
        a,
        width,
        b,
        color: color.pack_premultiplied(),
        c: [0.0; 3],
        kind: KIND_LINE,
    }
}

pub fn point(at: Vector3, size: f32, color: Color) -> OverlayItem {
    OverlayItem {
        a: at,
        width: size,
        b: at,
        color: color.pack_premultiplied(),
        c: at,
        kind: KIND_POINT,
    }
}

pub fn triangle(a: Vector3, b: Vector3, c: Vector3, color: Color) -> OverlayItem {
    OverlayItem {
        a,
        width: 0.0,
        b,
        color: color.pack_premultiplied(),
        c,
        kind: KIND_TRIANGLE,
    }
}

// What the overlay shows, besides the document and the editor.
pub struct Look<'a> {
    pub theme: &'a Theme,
    // The surface that's drawn, when the mesh is subdivided, and the
    // triangles drawn, of it or of the mesh.
    pub surface: Option<&'a SubdivisionSurface>,
    pub triangles: &'a Triangles,
    pub grid: bool,
    // The loop cut's preview's edges.
    pub preview: &'a [[Vector3; 2]],
}

// The floor's grid on the XZ plane, with the X axis in red and Z in blue.
pub fn grid(theme: &Theme, items: &mut Vec<OverlayItem>) {
    let reach = GRID_REACH as f32;
    let minor = theme.divider;
    let major = theme.control_strong_stroke.fade(0.35);
    for step in -GRID_REACH..=GRID_REACH {
        let at = step as f32;
        let color = if step % 5 == 0 { major } else { minor };
        if step != 0 {
            items.push(line([at, 0.0, -reach], [at, 0.0, reach], GRID_WIDTH, color));
            items.push(line([-reach, 0.0, at], [reach, 0.0, at], GRID_WIDTH, color));
        }
    }
    items.push(line([-reach, 0.0, 0.0], [reach, 0.0, 0.0], AXIS_WIDTH, theme.data.red));
    items.push(line([0.0, 0.0, -reach], [0.0, 0.0, reach], AXIS_WIDTH, theme.data.blue));
}

// Builds the overlay: what the mesh hides into `tested`, and what shows
// through it into `through`, which is everything in X-ray.
pub fn build(document: &Document, editor: &Editor, look: &Look, xray: bool, tested: &mut Vec<OverlayItem>, through: &mut Vec<OverlayItem>) {
    tested.clear();
    through.clear();
    let theme = look.theme;
    if look.grid {
        grid(theme, tested);
    }
    let mut cage_items = Vec::new();
    let items = &mut cage_items;
    let mesh = &document.mesh;
    let selection = &document.selection;
    let accent = theme.accent;

    // The selected faces' tint, on the triangles drawn.
    let tint = accent.fade(if xray { 0.18 } else { 0.28 });
    let hover_face = match editor.hover {
        Some(Element::Face(face)) if editor.operation.is_none() => Some(face),
        _ => None,
    };
    let hover_tint = accent.fade(0.12);
    if !selection.faces.is_empty() || hover_face.is_some() {
        let triangles = look.triangles;
        for (index, corners) in triangles.indices.as_chunks::<3>().0.iter().enumerate() {
            let face = triangles.faces[index];
            let base = look.surface.map_or(face, |surface| surface.base_faces[face]);
            let color = if selection.faces.contains(&base) {
                tint
            } else if hover_face == Some(base) {
                hover_tint
            } else {
                continue;
            };
            let [a, b, c] = [0, 1, 2].map(|corner| triangles.vertices[corners[corner] as usize].position);
            items.push(triangle(a, b, c, color));
        }
    }

    // The cage's edges: creased ones in pink, selected ones in the accent.
    let cage = if look.surface.is_some() { theme.text_secondary.fade(0.8) } else { theme.text_primary.fade(0.7) };
    let crease = theme.data.pink;
    let mut selected_edges = Vec::new();
    for edge in document.edges().iter().copied() {
        let [a, b] = edge.vertices();
        let (pa, pb) = (mesh.position(a), mesh.position(b));
        if selection.edges.contains(&edge) {
            selected_edges.push((pa, pb));
            continue;
        }
        let sharpness = mesh.crease(edge);
        if sharpness > 0.0 {
            let width = EDGE_WIDTH + sharpness.min(INFINITELY_SHARP) / INFINITELY_SHARP;
            items.push(line(pa, pb, width, crease));
        } else {
            items.push(line(pa, pb, EDGE_WIDTH, cage));
        }
    }
    for (a, b) in selected_edges {
        items.push(line(a, b, SELECTED_EDGE_WIDTH, accent));
    }

    // Vertices, in vertex mode.
    if editor.mode == Mode::Vertices {
        let dot = theme.text_primary;
        for vertex in 0..mesh.positions.len() as u32 {
            if !selection.vertices.contains(&vertex) {
                items.push(point(mesh.position(vertex), VERTEX_SIZE, dot));
            }
        }
        for vertex in &selection.vertices {
            if (*vertex as usize) < mesh.positions.len() {
                items.push(point(mesh.position(*vertex), SELECTED_VERTEX_SIZE, accent));
            }
        }
    }

    // What the pointer is on, over the rest.
    let hover = accent.fade(0.65);
    match editor.hover {
        Some(Element::Vertex(vertex)) if editor.operation.is_none() && (vertex as usize) < mesh.positions.len() => {
            items.push(point(mesh.position(vertex), HOVER_VERTEX_SIZE, hover));
        }
        Some(Element::Edge(edge)) if editor.operation.is_none() => {
            push_edge(mesh_edge(document, edge), HOVER_WIDTH, hover, items);
        }
        _ => {}
    }

    if xray {
        through.append(&mut cage_items);
    } else {
        tested.append(&mut cage_items);
    }

    // The loop cut's preview shows through the mesh, as it goes around it.
    if editor.tool == Tool::LoopCut {
        let color = theme.data.orange;
        for [a, b] in look.preview {
            through.push(line(*a, *b, SELECTED_EDGE_WIDTH, color));
        }
    }
}

fn mesh_edge(document: &Document, edge: Edge) -> Option<[Vector3; 2]> {
    let [a, b] = edge.vertices();
    let count = document.mesh.positions.len() as u32;
    (a < count && b < count).then(|| [document.mesh.position(a), document.mesh.position(b)])
}

fn push_edge(edge: Option<[Vector3; 2]>, width: f32, color: Color, items: &mut Vec<OverlayItem>) {
    if let Some([a, b]) = edge {
        items.push(line(a, b, width, color));
    }
}
