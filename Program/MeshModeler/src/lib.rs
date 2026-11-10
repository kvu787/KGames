// MeshModeler's parts, as a library, so that Test/MeshModeler can test what
// input does to the model, and where the cards go, without a window. main.rs
// runs the program, which modeler.rs is.

pub mod camera;
pub mod cards;
pub mod document;
pub mod editing;
pub mod interaction;
pub mod modeler;
pub mod overlay;
pub mod paint;
pub mod viewport;
