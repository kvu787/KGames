// MeshModeler's parts, as a library, so that Test/MeshModeler can test what
// input does to the model, and where the cards go, without a window. Main.rs
// runs the program, which Modeler.rs is.

#[path = "Camera.rs"]
pub mod camera;
#[path = "Cards.rs"]
pub mod cards;
#[path = "Document.rs"]
pub mod document;
#[path = "Editing.rs"]
pub mod editing;
#[path = "Interaction.rs"]
pub mod interaction;
#[path = "Modeler.rs"]
pub mod modeler;
#[path = "Overlay.rs"]
pub mod overlay;
#[path = "Paint.rs"]
pub mod paint;
#[path = "Viewport.rs"]
pub mod viewport;
