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

mod cutting;
mod editing;
mod extrusion;
mod files;
mod mesh;
mod picking;
pub mod primitives;
mod selection;
mod subdivision;
mod topology;
mod triangulation;

pub use mesh::{Edge, Face, INFINITELY_SHARP, Mesh};
pub use picking::Hit;
pub use selection::{SelectMode, Selection};
pub use subdivision::SubdivisionSurface;
pub use topology::{Ring, Topology};
pub use triangulation::{Shading, Triangles};
