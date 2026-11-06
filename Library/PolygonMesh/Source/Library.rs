// The geometry of a polygon mesh modeler, with no interface or GPU, so that
// programs and tests use it alike. README.md describes it.
//
// - mesh: positions, faces with materials, and creased edges.
// - topology: edges and their faces, boundaries, valences, loops and rings.
// - selection: selected vertices, edges and faces, and modes.
// - primitives: a cube, a plane, a cylinder, a cone, a UV sphere and a torus.
// - editing: moving, deleting, making faces, merging, normals, creases and
//   materials.
// - extrusion: extruding faces and edges, and insetting faces.
// - cutting: subdividing faces, and loop cuts.

#[path = "Cutting.rs"]
mod cutting;
#[path = "Editing.rs"]
mod editing;
#[path = "Extrusion.rs"]
mod extrusion;
#[path = "Mesh.rs"]
mod mesh;
#[path = "Primitives.rs"]
pub mod primitives;
#[path = "Selection.rs"]
mod selection;
#[path = "Topology.rs"]
mod topology;

pub use mesh::{Edge, Face, INFINITELY_SHARP, Mesh};
pub use selection::{SelectMode, Selection};
pub use topology::{Ring, Topology};
