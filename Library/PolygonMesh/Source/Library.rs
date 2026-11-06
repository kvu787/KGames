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
// - subdivision: Catmull-Clark subdivision surfaces, as OpenSubdiv makes them.
// - triangulation: triangles for SimplePaint, with flat or smooth normals.
// - picking: ray casts, the nearest vertex and edge, bounds and centers.
// - files: JSON, and Wavefront OBJ.

#[path = "Cutting.rs"]
mod cutting;
#[path = "Editing.rs"]
mod editing;
#[path = "Extrusion.rs"]
mod extrusion;
#[path = "Files.rs"]
mod files;
#[path = "Mesh.rs"]
mod mesh;
#[path = "Picking.rs"]
mod picking;
#[path = "Primitives.rs"]
pub mod primitives;
#[path = "Selection.rs"]
mod selection;
#[path = "Subdivision.rs"]
mod subdivision;
#[path = "Topology.rs"]
mod topology;
#[path = "Triangulation.rs"]
mod triangulation;

pub use mesh::{Edge, Face, INFINITELY_SHARP, Mesh};
pub use picking::Hit;
pub use selection::{SelectMode, Selection};
pub use subdivision::SubdivisionSurface;
pub use topology::{Ring, Topology};
pub use triangulation::{Shading, Triangles};
