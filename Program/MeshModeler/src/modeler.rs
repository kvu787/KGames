// The modeler, as a program that Library/UserInterface runs: the document,
// the editor, the camera and how the view shades the mesh, and the commands
// that the menus, the cards' buttons and the keys all go through. cards.rs
// lays out and draws the interface; render draws the mesh into the view's
// card before the interface draws over the rest of the frame.

use std::path::PathBuf;

use graphics::{Frame, Gpu};
use json::Value;
use math::Vector3;
use polygon_mesh::{Selection, Shading, SubdivisionSurface};
use simple_paint::orthographic::build_object_transforms;
use user_interface::{Color, Context, DialogResult, Program, Rect, Severity, Startup, Theme, Tree, Ui, ViewInput};
use windows_core::Result;

use crate::camera::{Camera, Viewpoint};
use crate::cards;
use crate::document::{self, Document};
use crate::editing::{Axis, Editor, Mode, OperationKind, Primitive, Tool};
use crate::interaction;
use crate::overlay::{self, Look};
use crate::paint::{MAXIMUM_PAINTS, Paint, next_color};
use crate::viewport::{Scene, Shader, Viewport};

pub const TITLE: &str = "Mesh Modeler";
// The most levels of subdivision that the view shows.
pub const MAXIMUM_LEVELS: u32 = 4;
// Smooth shading splits normals at edges sharper than this, as Blender's auto
// smooth does, so that a cube stays flat and a sphere is smooth.
pub const SMOOTH_ANGLE_DEGREES: f32 = 30.0;
// The light of the flat shader, toward the camera's upper left, and how much
// of a paint's color is ambient.
const LIGHT: [f32; 4] = [-0.37, 0.56, 0.74, 0.35];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    New,
    Open,
    Save,
    SaveAs,
    ExportObj,
    Undo,
    Redo,
    SelectAll,
    SelectNone,
    InvertSelection,
    VertexMode,
    EdgeMode,
    FaceMode,
    SelectTool,
    PaintTool,
    LoopCutTool,
    Move,
    Rotate,
    Scale,
    Extrude,
    Inset,
    Fill,
    Merge,
    Subdivide,
    Delete,
    DeleteFaces,
    FlipNormals,
    RecalculateNormals,
    Crease,
    ClearCrease,
    AssignPaint,
    AddPaint,
    RemovePaint,
    Add(Primitive),
    FrameAll,
    FrameSelected,
    View(Viewpoint),
    Level(u32),
    ToggleShader,
    ToggleSmooth,
    ToggleXray,
    ToggleGrid,
    ToggleTheme,
    VerticalSync,
    Fullscreen,
}

// Each command's label, which menus show and return, and its shortcuts, of
// which menus and tooltips show the first.
pub const COMMANDS: &[(Command, &str, &[&str])] = &[
    (Command::New, "New", &["Ctrl+N"]),
    (Command::Open, "Open", &["Ctrl+O"]),
    (Command::Save, "Save", &["Ctrl+S"]),
    (Command::SaveAs, "Save as", &["Ctrl+Shift+S"]),
    (Command::ExportObj, "Export OBJ", &["Ctrl+E"]),
    (Command::Undo, "Undo", &["Ctrl+Z"]),
    (Command::Redo, "Redo", &["Ctrl+Y", "Ctrl+Shift+Z"]),
    (Command::SelectAll, "Select all", &["A", "Ctrl+A"]),
    (Command::SelectNone, "Select none", &["Alt+A"]),
    (Command::InvertSelection, "Invert selection", &["Ctrl+I"]),
    (Command::VertexMode, "Vertices", &["1"]),
    (Command::EdgeMode, "Edges", &["2"]),
    (Command::FaceMode, "Faces", &["3"]),
    (Command::SelectTool, "Select tool", &["Q"]),
    (Command::PaintTool, "Paint tool", &["P"]),
    (Command::LoopCutTool, "Loop cut tool", &["Ctrl+R"]),
    (Command::Move, "Move", &["G"]),
    (Command::Rotate, "Rotate", &["R"]),
    (Command::Scale, "Scale", &["S"]),
    (Command::Extrude, "Extrude", &["E"]),
    (Command::Inset, "Inset", &["I"]),
    (Command::Fill, "Fill", &["F"]),
    (Command::Merge, "Merge at center", &["M"]),
    (Command::Subdivide, "Subdivide", &["Ctrl+D"]),
    (Command::Delete, "Delete", &["X", "Delete"]),
    (Command::DeleteFaces, "Delete only faces", &["Shift+X"]),
    (Command::FlipNormals, "Flip normals", &["Alt+N"]),
    (Command::RecalculateNormals, "Recalculate normals", &["Shift+N"]),
    (Command::Crease, "Crease edges", &["Shift+E"]),
    (Command::ClearCrease, "Clear crease", &["Alt+E"]),
    (Command::AssignPaint, "Paint selected faces", &["Ctrl+P"]),
    (Command::AddPaint, "Add paint", &[]),
    (Command::RemovePaint, "Remove paint", &[]),
    (Command::Add(Primitive::Cube), "Cube", &[]),
    (Command::Add(Primitive::Plane), "Plane", &[]),
    (Command::Add(Primitive::Cylinder), "Cylinder", &[]),
    (Command::Add(Primitive::Cone), "Cone", &[]),
    (Command::Add(Primitive::Sphere), "UV sphere", &[]),
    (Command::Add(Primitive::Torus), "Torus", &[]),
    (Command::FrameAll, "Frame all", &["Home"]),
    (Command::FrameSelected, "Frame selected", &["Period"]),
    (Command::View(Viewpoint::Front), "Front", &["Numpad1"]),
    (Command::View(Viewpoint::Back), "Back", &["Ctrl+Numpad1"]),
    (Command::View(Viewpoint::Right), "Right", &["Numpad3"]),
    (Command::View(Viewpoint::Left), "Left", &["Ctrl+Numpad3"]),
    (Command::View(Viewpoint::Top), "Top", &["Numpad7"]),
    (Command::View(Viewpoint::Bottom), "Bottom", &["Ctrl+Numpad7"]),
    (Command::Level(0), "Off", &["Ctrl+0"]),
    (Command::Level(1), "1 level", &["Ctrl+1"]),
    (Command::Level(2), "2 levels", &["Ctrl+2"]),
    (Command::Level(3), "3 levels", &["Ctrl+3"]),
    (Command::Level(4), "4 levels", &["Ctrl+4"]),
    (Command::ToggleShader, "Flat shader", &["Shift+Z"]),
    (Command::ToggleSmooth, "Smooth shading", &["Shift+S"]),
    (Command::ToggleXray, "X-ray", &["Alt+Z"]),
    (Command::ToggleGrid, "Grid", &["Shift+G"]),
    (Command::ToggleTheme, "Dark theme", &["F9"]),
    (Command::VerticalSync, "VSync", &["F10"]),
    (Command::Fullscreen, "Fullscreen", &["F11"]),
];

impl Command {
    pub fn label(self) -> &'static str {
        COMMANDS.iter().find(|(command, _, _)| *command == self).map_or("", |(_, label, _)| label)
    }

    // The shortcut that menus and tooltips show, if it has one.
    pub fn shortcut(self) -> Option<&'static str> {
        COMMANDS.iter().find(|(command, _, _)| *command == self).and_then(|(_, _, keys)| keys.first().copied())
    }

    pub fn from_label(label: &str) -> Option<Command> {
        COMMANDS.iter().find(|(_, known, _)| *known == label).map(|(command, _, _)| *command)
    }
}

// What a dialog's done with, after the changes are saved or not.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pending {
    New,
    Open,
    Close,
}

// The dialog that's open.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Prompt {
    // Asks for the name to save as.
    SaveAs,
    // Lists the saved models to open.
    Open,
    // Asks whether to save the changes first.
    Unsaved(Pending),
}

impl Prompt {
    pub fn name(self) -> &'static str {
        match self {
            Prompt::SaveAs => "SaveAs",
            Prompt::Open => "OpenModel",
            Prompt::Unsaved(_) => "Unsaved",
        }
    }
}

// What the input on the modeler's interface means.
#[derive(Clone, Debug, PartialEq)]
pub enum Message {
    // A command, from a menu, a button or a key.
    Command(Command),
    // The crease slider, and the Add card's primitive.
    Crease(f64),
    Primitive(usize),
    // The pointer in the view, and on the modal edit's cover.
    View(ViewInput),
    Operation(ViewInput),
    // The modal edit's keys.
    Constrain(Axis),
    ConfirmOperation,
    CancelOperation,
    // The paints: the one picked, an edit to it, and the edit's end, which
    // commits it as one step of the history.
    PickPaint(usize),
    Paint(PaintEdit),
    EndPaintEdit,
    // The dialogs: their fields, and their buttons.
    NameDraft(String),
    OpenChoice(usize),
    OpenActivated,
    Unsaved(DialogResult),
    SaveAsResult(DialogResult),
    OpenResult(DialogResult),
    // Once a frame, after everything else: remakes what the view draws.
    Prepare,
}

// An edit to the paint that's picked, from its card.
#[derive(Clone, Debug, PartialEq)]
pub enum PaintEdit {
    Color(Color),
    Name(String),
    Brightness(f64),
    Shift(f64),
    Rotation(f64),
    Dark(f64),
    Light(f64),
}

pub struct Modeler {
    pub document: Document,
    pub editor: Editor,
    pub camera: Camera,
    pub shader: Shader,
    // Levels of Catmull-Clark subdivision, which the view shows the mesh
    // with, around its cage.
    pub levels: u32,
    pub smooth: bool,
    pub xray: bool,
    pub grid: bool,
    // The sharpness that Crease gives edges.
    pub crease: f32,
    // The primitive that the Add card adds.
    pub primitive: usize,
    // Where models are saved: Models in the data folder.
    pub folder: PathBuf,
    pub prompt: Option<Prompt>,
    // What a dialog edits: the name to save as, and the model to open.
    pub name_draft: String,
    pub open_choice: Option<usize>,
    pub saved_names: Vec<String>,
    // What's pending until a Save as dialog saves.
    after_save: Option<Pending>,
    // What the last command said, like where a file was saved, for the
    // status bar.
    pub message: Option<(Severity, String)>,
    // Where the last frame's view is in the window.
    pub view: Rect,
    pub scene: Scene,
    pub(crate) surface: Option<SubdivisionSurface>,
    // What the scene's triangles are of: the document's version, the levels,
    // and whether they're smooth.
    built: Option<(u64, u32, bool)>,
    // The loop cut's preview's edges, and what they're of.
    pub preview: Vec<[Vector3; 2]>,
    preview_key: Option<(u64, polygon_mesh::Edge, i32)>,
    viewport: Option<Viewport>,
    // The paints' names and colors as their fields have them, for an edit
    // that's one step of history while a slider is dragged.
    pub paint_edit: Option<document::Snapshot>,
    // What's wrong with the name in the Save as dialog.
    pub prompt_error: Option<String>,
}

impl Modeler {
    pub fn new(folder: PathBuf) -> Self {
        Self {
            document: Document::new(),
            editor: Editor::default(),
            camera: Camera::default(),
            shader: Shader::SimplePaint,
            levels: 0,
            smooth: true,
            xray: false,
            grid: true,
            crease: 1.0,
            primitive: 0,
            folder,
            prompt: None,
            name_draft: String::new(),
            open_choice: None,
            saved_names: Vec::new(),
            after_save: None,
            message: None,
            view: Rect::ZERO,
            scene: Scene::default(),
            surface: None,
            built: None,
            preview: Vec::new(),
            preview_key: None,
            viewport: None,
            paint_edit: None,
            prompt_error: None,
        }
    }

    // The modeler, which saves models in Models in its data folder: its own
    // folder, or a script's output folder.
    pub fn start(startup: &mut Startup) -> Self {
        Modeler::new(startup.data_folder.join("Models"))
    }

    // The surface that the view shows, when the mesh is subdivided.
    pub fn surface(&self) -> Option<&SubdivisionSurface> {
        self.surface.as_ref()
    }

    fn say(&mut self, severity: Severity, message: impl Into<String>) {
        self.message = Some((severity, message.into()));
    }

    // Does a command, from a menu, a button or a key.
    pub fn run(&mut self, ui: &mut Context<Message>, command: Command) {
        // A command ends a modal edit, keeping it, as a click would.
        if self.editor.operation.is_some() && !matches!(command, Command::Undo | Command::Redo) {
            self.editor.confirm(&mut self.document);
        }
        if self.editor.operation.is_some() {
            self.editor.cancel(&mut self.document);
        }
        // A held paint stroke commits as its own step before any command,
        // Undo and Redo included, so the history never interleaves.
        self.editor.end_paint(&mut self.document);
        self.end_paint_edit();
        let pointer = ui.input().pointer.unwrap_or(self.view.center());
        match command {
            Command::New => self.ask_first(Pending::New),
            Command::Open => self.ask_first(Pending::Open),
            Command::Save => _ = self.save(),
            Command::SaveAs => self.prompt_save_as(None),
            Command::ExportObj => match self.document.export_obj(&self.folder) {
                Ok(path) => self.say(Severity::Success, format!("Exported {}", path.display())),
                Err(error) => self.say(Severity::Error, error),
            },
            Command::Undo => {
                match self.document.undo() {
                    Some(name) => self.say(Severity::Informational, format!("Undid {name}")),
                    None => self.say(Severity::Informational, "There's nothing to undo"),
                }
                // The paint picked may be gone with the step, like one that
                // was just added. Another is picked here, and not only at
                // the frame's end, in Prepare: the next message of this
                // frame may edit the paint picked, or paint with it.
                self.keep_paint_picked();
            }
            Command::Redo => {
                match self.document.redo() {
                    Some(name) => self.say(Severity::Informational, format!("Redid {name}")),
                    None => self.say(Severity::Informational, "There's nothing to redo"),
                }
                // As after Undo: the step may be a paint's removal.
                self.keep_paint_picked();
            }
            Command::SelectAll => {
                let before = self.document.snapshot();
                self.document.selection = Selection::all(&self.document.mesh).in_mode(&self.document.mesh, self.editor.mode.select_mode());
                self.document.commit("Select all", before);
            }
            Command::SelectNone => {
                let before = self.document.snapshot();
                self.document.selection = Selection::new();
                self.document.commit("Select none", before);
            }
            Command::InvertSelection => {
                let before = self.document.snapshot();
                self.document.selection = self.document.selection.inverted(&self.document.mesh, self.editor.mode.select_mode());
                self.document.commit("Invert selection", before);
            }
            Command::VertexMode => self.editor.set_mode(&mut self.document, Mode::Vertices),
            Command::EdgeMode => self.editor.set_mode(&mut self.document, Mode::Edges),
            Command::FaceMode => self.editor.set_mode(&mut self.document, Mode::Faces),
            Command::SelectTool => self.editor.tool = Tool::Select,
            Command::PaintTool => self.editor.tool = Tool::Paint,
            Command::LoopCutTool => self.editor.tool = Tool::LoopCut,
            Command::Move => _ = self.editor.begin(&mut self.document, OperationKind::Move, pointer, &self.camera, self.view),
            Command::Rotate => _ = self.editor.begin(&mut self.document, OperationKind::Rotate, pointer, &self.camera, self.view),
            Command::Scale => _ = self.editor.begin(&mut self.document, OperationKind::Scale, pointer, &self.camera, self.view),
            Command::Extrude => _ = self.editor.extrude(&mut self.document, pointer, &self.camera, self.view),
            Command::Inset => _ = self.editor.inset(&mut self.document, pointer, &self.camera, self.view),
            Command::Fill => self.editor.fill(&mut self.document),
            Command::Merge => self.editor.merge(&mut self.document),
            Command::Subdivide => self.editor.subdivide(&mut self.document),
            Command::Delete => self.editor.delete(&mut self.document),
            Command::DeleteFaces => self.editor.delete_only_faces(&mut self.document),
            Command::FlipNormals => self.editor.flip(&mut self.document),
            Command::RecalculateNormals => self.editor.recalculate_normals(&mut self.document),
            Command::Crease => self.editor.crease(&mut self.document, self.crease),
            Command::ClearCrease => self.editor.crease(&mut self.document, 0.0),
            Command::AssignPaint => self.editor.assign_paint(&mut self.document),
            Command::AddPaint => {
                let before = self.document.snapshot();
                let count = self.document.paints.len();
                let paint = Paint::new(&format!("Paint {}", count + 1), next_color(count));
                match self.document.add_paint(paint) {
                    Some(index) => {
                        self.editor.active_paint = index;
                        self.document.commit("Add paint", before);
                    }
                    None => self.say(Severity::Warning, format!("A model has at most {MAXIMUM_PAINTS} paints")),
                }
            }
            Command::RemovePaint => {
                let before = self.document.snapshot();
                let index = self.editor.active_paint;
                if self.document.remove_paint(index) {
                    self.editor.active_paint = index.min(self.document.paints.len() - 1);
                    self.document.commit("Remove paint", before);
                } else {
                    self.say(Severity::Warning, "A model needs a paint");
                }
            }
            Command::Add(primitive) => self.editor.add_primitive(&mut self.document, primitive),
            Command::FrameAll => {
                if let Some([minimum, maximum]) = self.document.mesh.bounds() {
                    self.camera.frame(minimum, maximum);
                }
            }
            Command::FrameSelected => {
                let mesh = &self.document.mesh;
                let points: Vec<Vector3> = self.document.selection.vertices.iter().filter(|vertex| (**vertex as usize) < mesh.positions.len()).map(|vertex| mesh.position(*vertex)).collect();
                if let Some(first) = points.first() {
                    let (minimum, maximum) = points.iter().fold((*first, *first), |(low, high), point| {
                        (std::array::from_fn(|axis| low[axis].min(point[axis])), std::array::from_fn(|axis| high[axis].max(point[axis])))
                    });
                    self.camera.frame(minimum, maximum);
                } else {
                    self.say(Severity::Informational, "Select something to frame it");
                }
            }
            Command::View(viewpoint) => self.camera.look_from(viewpoint),
            Command::Level(levels) => self.levels = levels.min(MAXIMUM_LEVELS),
            Command::ToggleShader => {
                self.shader = match self.shader {
                    Shader::SimplePaint => Shader::Flat,
                    Shader::Flat => Shader::SimplePaint,
                }
            }
            Command::ToggleSmooth => self.smooth = !self.smooth,
            Command::ToggleXray => self.xray = !self.xray,
            Command::ToggleGrid => self.grid = !self.grid,
            Command::ToggleTheme => ui.toggle_theme(),
            Command::VerticalSync => ui.toggle_vertical_sync(),
            Command::Fullscreen => ui.toggle_fullscreen(),
        }
    }

    // Whether a command can be done now, for its menu item and button.
    pub fn can(&self, command: Command) -> bool {
        let selection = &self.document.selection;
        match command {
            Command::Undo => self.document.undo_name().is_some(),
            Command::Redo => self.document.redo_name().is_some(),
            Command::Move | Command::Rotate | Command::Scale | Command::FrameSelected => !selection.vertices.is_empty(),
            Command::Extrude => !selection.faces.is_empty() || !selection.edges.is_empty(),
            Command::Inset | Command::Subdivide | Command::FlipNormals | Command::DeleteFaces | Command::AssignPaint => !selection.faces.is_empty(),
            Command::Fill => selection.vertices.len() >= 2,
            Command::Merge => selection.vertices.len() >= 2,
            Command::Delete | Command::SelectNone => !selection.is_empty(),
            Command::Crease | Command::ClearCrease => !selection.edges.is_empty(),
            Command::AddPaint => self.document.paints.len() < MAXIMUM_PAINTS,
            Command::RemovePaint => self.document.paints.len() > 1,
            _ => true,
        }
    }

    // Whether the command's toggle is on, for its menu item's check.
    pub fn checked(&self, command: Command) -> Option<bool> {
        Some(match command {
            Command::ToggleShader => self.shader == Shader::Flat,
            Command::ToggleSmooth => self.smooth,
            Command::ToggleXray => self.xray,
            Command::ToggleGrid => self.grid,
            Command::Level(levels) => self.levels == levels,
            Command::VertexMode => self.editor.mode == Mode::Vertices,
            Command::EdgeMode => self.editor.mode == Mode::Edges,
            Command::FaceMode => self.editor.mode == Mode::Faces,
            Command::SelectTool => self.editor.tool == Tool::Select,
            Command::PaintTool => self.editor.tool == Tool::Paint,
            Command::LoopCutTool => self.editor.tool == Tool::LoopCut,
            _ => return None,
        })
    }

    // Asks whether to save the changes before what's pending, if there are
    // any, or else does it.
    fn ask_first(&mut self, pending: Pending) {
        if self.document.dirty {
            self.prompt = Some(Prompt::Unsaved(pending));
        } else {
            self.proceed(pending);
        }
    }

    // Does what was pending, now that the changes are saved or not. Closing
    // is done by cards.rs, which has the Ui.
    pub fn proceed(&mut self, pending: Pending) {
        match pending {
            Pending::New => {
                self.replace_document(Document::new());
                self.editor.reset();
                self.camera = Camera::default();
                self.prompt = None;
                self.say(Severity::Informational, "Started a new model");
            }
            Pending::Open => {
                self.saved_names = document::saved_names(&self.folder);
                self.open_choice = self.saved_names.iter().position(|name| *name == self.document.name).or((!self.saved_names.is_empty()).then_some(0));
                self.prompt = Some(Prompt::Open);
            }
            Pending::Close => self.prompt = None,
        }
    }

    pub fn prompt_save_as(&mut self, after: Option<Pending>) {
        self.name_draft = self.document.name.clone();
        self.after_save = after;
        self.prompt = Some(Prompt::SaveAs);
    }

    // Saves under the document's name, or asks for one if it has none yet.
    pub fn save(&mut self) -> bool {
        if self.document.name == document::UNTITLED {
            self.prompt_save_as(None);
            return false;
        }
        match self.document.save(&self.folder) {
            Ok(path) => {
                self.say(Severity::Success, format!("Saved {}", path.display()));
                true
            }
            Err(error) => {
                self.say(Severity::Error, error);
                false
            }
        }
    }

    // Saves under the name in the Save as dialog. Returns what was pending
    // on the save, if it saved.
    pub fn save_as(&mut self) -> std::result::Result<Option<Pending>, String> {
        let name = self.name_draft.trim().to_string();
        if !document::valid_name(&name) {
            return Err("A name can't be empty, end with a period, or have any of < > : \" / \\ | ? *.".to_string());
        }
        let previous = std::mem::replace(&mut self.document.name, name);
        match self.document.save(&self.folder) {
            Ok(path) => {
                self.prompt = None;
                self.say(Severity::Success, format!("Saved {}", path.display()));
                Ok(self.after_save.take())
            }
            Err(error) => {
                self.document.name = previous;
                Err(error)
            }
        }
    }

    // Opens the model chosen in the Open dialog.
    pub fn open_chosen(&mut self) {
        let Some(name) = self.open_choice.and_then(|index| self.saved_names.get(index)).cloned() else {
            return;
        };
        match Document::open(&self.folder, &name) {
            Ok(document) => {
                self.replace_document(document);
                self.editor.reset();
                if let Some([minimum, maximum]) = self.document.mesh.bounds() {
                    self.camera.frame(minimum, maximum);
                }
                self.prompt = None;
                self.say(Severity::Success, format!("Opened {name}"));
            }
            Err(error) => self.say(Severity::Error, error),
        }
    }

    // Makes another document the modeler's, and forgets what the view drew
    // of the last one. A document counts its own versions from 1, so the new
    // one's version may be the last one's, and the triangles made for that
    // version aren't this document's.
    fn replace_document(&mut self, document: Document) {
        self.document = document;
        self.built = None;
        self.preview_key = None;
    }

    // Picks the last paint, if the one picked isn't among the paints any
    // more. A document always has a paint.
    fn keep_paint_picked(&mut self) {
        self.editor.active_paint = self.editor.active_paint.min(self.document.paints.len() - 1);
    }

    // A change to the paints from their card, which is one step of the
    // history until the control is let go of.
    pub fn begin_paint_edit(&mut self) {
        if self.paint_edit.is_none() {
            self.paint_edit = Some(self.document.snapshot());
        }
    }

    pub fn end_paint_edit(&mut self) {
        if let Some(before) = self.paint_edit.take() {
            self.document.commit("Edit paint", before);
        }
    }

    // Remakes what the view draws for this frame: the triangles, when the
    // mesh or how it's shaded changed, the transforms, the paints, and the
    // overlay.
    pub fn prepare_scene(&mut self, theme: &Theme) {
        let view = self.view;
        let key = (self.document.version, self.levels, self.smooth);
        if self.built != Some(key) {
            let shading = if self.smooth { Shading::Smooth { angle: SMOOTH_ANGLE_DEGREES.to_radians() } } else { Shading::Flat };
            self.surface = (self.levels > 0).then(|| self.document.mesh.catmull_clark(self.levels));
            self.scene.triangles = match &self.surface {
                Some(surface) => surface.mesh.triangulate(shading),
                None => self.document.mesh.triangulate(shading),
            };
            self.scene.version += 1;
            self.built = Some(key);
        }
        self.update_preview();
        let background = theme.card_opaque();
        self.scene.view = view;
        self.scene.background = [background.r, background.g, background.b, 1.0];
        self.scene.shader = self.shader;
        if view.width >= 1.0 && view.height >= 1.0 {
            self.scene.world_to_clip = self.camera.world_to_clip(view);
            if let Ok(transforms) = build_object_transforms(&self.camera.view(), &self.camera.projection(view)) {
                self.scene.transforms = transforms;
            }
        }
        let paints = &self.document.paints;
        for index in 0..MAXIMUM_PAINTS {
            let paint = paints.get(index).unwrap_or(&paints[0]);
            self.scene.materials[index] = paint.material();
            self.scene.colors[index] = paint.linear_color();
        }
        self.scene.light = LIGHT;
        let look = Look {
            theme,
            surface: self.surface.as_ref(),
            triangles: &self.scene.triangles,
            grid: self.grid,
            preview: &self.preview,
        };
        let (mut tested, mut through) = (std::mem::take(&mut self.scene.overlay), std::mem::take(&mut self.scene.xray));
        overlay::build(&self.document, &self.editor, &look, self.xray, &mut tested, &mut through);
        self.scene.overlay = tested;
        self.scene.xray = through;
    }

    // The loop cut's preview, remade only when the edge or where along it
    // changes, to a hundredth.
    fn update_preview(&mut self) {
        let Some((edge, fraction)) = self.editor.loop_preview.filter(|_| self.editor.tool == Tool::LoopCut) else {
            self.preview.clear();
            self.preview_key = None;
            return;
        };
        let key = (self.document.version, edge, (fraction * 100.0).round() as i32);
        if self.preview_key != Some(key) {
            self.preview = Editor::preview_loop(&self.document, edge, fraction);
            self.preview_key = Some(key);
        }
    }

    // The modeler's state, for scripts and tests.
    pub fn state(&self) -> Value {
        let document = &self.document;
        let mesh = &document.mesh;
        let selection = &document.selection;
        let numbers = |values: Vec<Value>| Value::Array(values);
        let operation = self.editor.operation.as_ref().map_or(Value::Null, |operation| {
            object(vec![
                ("Kind", Value::from(operation.kind.name())),
                ("Axis", Value::from(operation.axis.name())),
                ("Amount", Value::from(operation.amount.as_str())),
            ])
        });
        let hover = match self.editor.hover {
            Some(crate::editing::Element::Vertex(vertex)) => Value::from(format!("Vertex {vertex}")),
            Some(crate::editing::Element::Edge(edge)) => {
                let [a, b] = edge.vertices();
                Value::from(format!("Edge {a}-{b}"))
            }
            Some(crate::editing::Element::Face(face)) => Value::from(format!("Face {face}")),
            None => Value::Null,
        };
        let materials: Vec<Value> = mesh.faces.iter().map(|face| Value::from(face.material)).collect();
        object(vec![
            ("Name", Value::from(document.name.as_str())),
            ("Dirty", Value::from(document.dirty)),
            ("Mode", Value::from(self.editor.mode.name())),
            ("Tool", Value::from(self.editor.tool.name())),
            ("Operation", operation),
            ("Hover", hover),
            (
                "Mesh",
                object(vec![
                    ("Vertices", Value::from(mesh.positions.len())),
                    ("Edges", Value::from(self.document.edges().len())),
                    ("Faces", Value::from(mesh.faces.len())),
                    ("Creases", Value::from(mesh.creases.len())),
                    ("Valid", Value::from(mesh.validate().is_ok())),
                    ("Materials", numbers(materials)),
                ]),
            ),
            (
                "Selection",
                object(vec![
                    ("Vertices", Value::from(selection.vertices.len())),
                    ("Edges", Value::from(selection.edges.len())),
                    ("Faces", Value::from(selection.faces.len())),
                    ("FaceList", numbers(selection.faces.iter().map(|face| Value::from(*face)).collect())),
                ]),
            ),
            ("Paints", Value::Array(document.paints.iter().map(|paint| Value::from(paint.name.as_str())).collect())),
            ("ActivePaint", Value::from(self.editor.active_paint)),
            ("Shader", Value::from(self.shader.name())),
            ("Subdivision", Value::from(self.levels)),
            ("SurfaceFaces", Value::from(self.surface.as_ref().map_or(mesh.faces.len(), |surface| surface.mesh.faces.len()))),
            ("Smooth", Value::from(self.smooth)),
            ("XRay", Value::from(self.xray)),
            ("Undo", document.undo_name().map_or(Value::Null, Value::from)),
            ("Redo", document.redo_name().map_or(Value::Null, Value::from)),
            ("UndoSteps", Value::from(document.undo_steps())),
            ("Prompt", self.prompt.map_or(Value::Null, |prompt| Value::from(prompt.name()))),
            ("Message", self.message.as_ref().map_or(Value::Null, |(_, message)| Value::from(message.as_str()))),
            ("EditorMessage", self.editor.message.as_ref().map_or(Value::Null, |message| Value::from(message.as_str()))),
            ("LoopPreviewEdges", Value::from(self.preview.len())),
            (
                "Camera",
                object(vec![
                    ("Yaw", Value::from(self.camera.yaw.to_degrees())),
                    ("Pitch", Value::from(self.camera.pitch.to_degrees())),
                    ("Height", Value::from(self.camera.height)),
                ]),
            ),
        ])
    }
}

fn object(members: Vec<(&str, Value)>) -> Value {
    Value::Object(members.into_iter().map(|(name, value)| (name.to_string(), value)).collect())
}

impl Modeler {
    // Does what was pending once the changes are saved or not.
    fn finish(&mut self, ui: &mut Context<Message>, pending: Pending) {
        if pending == Pending::Close {
            self.prompt = None;
            ui.close();
        } else {
            self.proceed(pending);
        }
    }

    // A dialog's answer.
    fn answer(&mut self, ui: &mut Context<Message>, prompt: Prompt, result: DialogResult) {
        if self.prompt != Some(prompt) {
            return;
        }
        match prompt {
            Prompt::Unsaved(pending) => match result {
                DialogResult::Primary => {
                    if self.document.name == document::UNTITLED {
                        self.prompt_save_as(Some(pending));
                    } else if self.save() {
                        self.finish(ui, pending);
                    } else {
                        self.prompt = None;
                    }
                }
                DialogResult::Secondary => self.finish(ui, pending),
                DialogResult::Close => self.prompt = None,
            },
            Prompt::SaveAs => match result {
                DialogResult::Primary => match self.save_as() {
                    Ok(pending) => {
                        self.prompt_error = None;
                        if let Some(pending) = pending {
                            self.finish(ui, pending);
                        }
                    }
                    Err(error) => {
                        self.prompt_error = Some(error);
                        self.prompt = Some(Prompt::SaveAs);
                    }
                },
                _ => {
                    self.prompt = None;
                    self.prompt_error = None;
                }
            },
            Prompt::Open => match result {
                DialogResult::Primary => self.open_chosen(),
                _ => self.prompt = None,
            },
        }
    }

    // An edit to the paint that's picked, which begins one step of the
    // history if none is under way.
    fn edit_paint(&mut self, edit: PaintEdit) {
        self.begin_paint_edit();
        let paint = &mut self.document.paints[self.editor.active_paint];
        match edit {
            PaintEdit::Color(color) => paint.set_color(color),
            PaintEdit::Name(name) => paint.name = name,
            PaintEdit::Brightness(value) => paint.parameters.brightness = value,
            PaintEdit::Shift(value) => paint.parameters.shift = value,
            PaintEdit::Rotation(value) => paint.parameters.rotation_degrees = value,
            PaintEdit::Dark(value) => paint.parameters.dark_point = value,
            PaintEdit::Light(value) => paint.parameters.light_point = value,
        }
        self.document.touch();
    }
}

impl Program for Modeler {
    type Message = Message;

    fn title(&self) -> String {
        let dirty = if self.document.dirty { "*" } else { "" };
        format!("{}{dirty} - {TITLE}", self.document.name)
    }

    fn input(&self, ui: &Ui) -> Vec<Message> {
        interaction::messages(self, ui)
    }

    fn update(&mut self, message: Message, ui: &mut Context<Message>) {
        match message {
            Message::Command(command) => self.run(ui, command),
            Message::Crease(value) => self.crease = value as f32,
            Message::Primitive(index) => self.primitive = index,
            Message::View(input) => interaction::view(self, input),
            Message::Operation(input) => interaction::operation(self, input),
            Message::Constrain(axis) => self.editor.constrain(axis),
            Message::ConfirmOperation => self.editor.confirm(&mut self.document),
            Message::CancelOperation => self.editor.cancel(&mut self.document),
            Message::PickPaint(index) => {
                // The row is of the frame that the user saw. An Undo before
                // it in this frame may have taken its paint away.
                self.editor.active_paint = index;
                self.keep_paint_picked();
            }
            Message::Paint(edit) => self.edit_paint(edit),
            Message::EndPaintEdit => self.end_paint_edit(),
            Message::NameDraft(name) => self.name_draft = name,
            Message::OpenChoice(index) => self.open_choice = Some(index),
            Message::OpenActivated => self.open_chosen(),
            Message::Unsaved(result) => {
                if let Some(prompt @ Prompt::Unsaved(_)) = self.prompt {
                    self.answer(ui, prompt, result);
                }
            }
            Message::SaveAsResult(result) => self.answer(ui, Prompt::SaveAs, result),
            Message::OpenResult(result) => self.answer(ui, Prompt::Open, result),
            Message::Prepare => {
                self.keep_paint_picked();
                self.view = cards::view_content(ui.bounds());
                let theme = ui.theme().clone();
                self.prepare_scene(&theme);
            }
        }
    }

    fn view(&self, ui: &mut Ui) -> Tree<Message> {
        cards::view(self, ui)
    }

    fn prepare(&mut self, gpu: &mut Gpu) -> Result<()> {
        self.viewport = Some(Viewport::new(gpu)?);
        Ok(())
    }

    fn render(&mut self, gpu: &mut Gpu, frame: &Frame) -> Result<()> {
        if let Some(viewport) = &mut self.viewport {
            viewport.draw(gpu, frame, &self.scene)?;
        }
        Ok(())
    }

    fn may_close(&mut self) -> bool {
        if self.document.dirty {
            self.prompt = Some(Prompt::Unsaved(Pending::Close));
            return false;
        }
        true
    }

    fn state(&self) -> Value {
        Modeler::state(self)
    }
}
