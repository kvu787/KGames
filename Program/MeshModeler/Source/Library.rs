// MeshModeler's parts, as a library, so that Test/MeshModeler can test them
// without a window.

#[path = "Camera.rs"]
pub mod camera;
#[path = "Document.rs"]
pub mod document;
#[path = "Editing.rs"]
pub mod editing;
#[path = "Overlay.rs"]
pub mod overlay;
#[path = "Paint.rs"]
pub mod paint;
#[path = "Viewport.rs"]
pub mod viewport;
