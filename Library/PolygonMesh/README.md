# PolygonMesh

The geometry of a polygon mesh modeler, like Blender's edit mode: a mesh of polygons with a
material each and creased edges, its topology, selections of vertices, edges and faces, and
primitives to start from. It has no interface or GPU, so a modeler and its tests use it alike.
`Test/PolygonMesh` tests it.

```rust
use polygon_mesh::{Selection, primitives};

let mut mesh = primitives::cube(2.0);
let top = Selection::from_faces(&mesh, [3]);
let loop_around = Selection::edge_loop(&mesh, polygon_mesh::Edge::new(2, 3));
```

A `Mesh` has `positions`, `faces` and `creases`, all public. A `Face` is its `vertices`,
counterclockwise when seen from its front, the right-handed convention, and its `material`, an
index that SimplePaint paints the face with. Every face has at least 3 vertices, each in range
and none twice, which `Mesh::validate` checks and every operation keeps. Edges are the faces'
sides, so there are no edges without faces, and a vertex that no face uses is loose. An `Edge`
is undirected, its smaller vertex first, so both faces next to it name it the same way.
`creases` maps an edge to its sharpness, more than 0 and at most `INFINITELY_SHARP`, 10, which
counts as infinitely sharp; an edge that isn't there is smooth. `Mesh` is `Clone`, so that a
modeler can undo by keeping copies.

Nothing assumes a manifold. An edge can have any number of faces, and the faces around a vertex
needn't make one fan, so meshes with boundaries, and non-manifold ones, work without panicking.
Element numbers aren't stable across edits that change topology: each one says what it does to
them, and returns the selection that results.

## The API

| Area          | What's there                                                                         |
| ------------- | ------------------------------------------------------------------------------------ |
| `Mesh`        | `add_vertex`, `add_face`, `position`, `edges`, `crease`, `set_crease`,               |
|               | `material_count`, `face_center`, `face_normal`, `face_area`, `face_area_vector`,     |
|               | `validate`, `topology`                                                               |
| `Face`        | `sides` (directed), `edges`, `has_side`, `flip`                                      |
| `Edge`        | `new`, `vertices`, `contains`, `other`                                               |
| `Topology`    | `edges`, `edge`, `edge_index`, `edge_faces`, `faces_of_edge`, `face_edges`,          |
|               | `vertex_edges`, `vertex_faces`, `valence`, `is_boundary`, `is_non_manifold`,         |
|               | `is_boundary_vertex`, `boundary_edges`, `non_manifold_edges`, `is_closed`,           |
|               | `is_consistently_wound`, `euler_characteristic`, `edge_loop`, `edge_ring`,           |
|               | `face_loop`, `ring`                                                                  |
| `Selection`   | `vertices`, `edges`, `faces`; `from_vertices`, `from_edges`, `from_faces`, `all`,    |
|               | `new` (none), `inverted`, `in_mode`, `union`, `edge_loop`, `edge_ring`, `face_loop`, |
|               | `center`, `is_empty`                                                                 |
| `primitives`  | `cube`, `plane`, `cylinder`, `cone`, `uv_sphere`, `torus`                            |

## Topology

`Mesh::topology` computes a `Topology` from the faces, on demand, since edits change it all the
time. It owns its data, so a program can keep it beside the mesh while the mesh doesn't change.
Edges are numbered in `Edge`'s order, and side `i` of a face, in `face_edges`, goes from its
vertex `i` to vertex `i + 1`. A boundary edge has one face, and a non-manifold edge more than
two, or two that go along it the same way, which two faces of a consistently wound surface never
do.

The edge loop through an edge goes on across each vertex where four edges meet, each with two
faces, to the edge that shares no face with the one before, and along the boundary from a
boundary edge, around its corners, as Blender selects it. The edge ring goes from each edge to
the opposite side of each quad next to it. Both go both ways from the edge until they can't go
on, or until they close. `Topology::ring` also gives the quads that a ring crosses, in order,
which `face_loop` selects.

## Selections

A `Selection` keeps what's selected in all three modes at once, as Blender does:

- Selecting vertices selects the edges whose ends are both selected, and the faces whose
  vertices all are.
- Selecting edges selects their vertices, and the faces whose edges all are.
- Selecting faces selects their edges and vertices.

`in_mode` is what switching to a mode leaves: what's selected of that mode's own elements, and
what they select. So two faces that share an edge stay selected from vertex mode to face mode,
and a face's vertices select it again. `inverted` selects the mode's elements that weren't
selected. Elements that aren't in the mesh are left out. `center` is the average of the selected
vertices, the median point that Blender rotates and scales about.

## Primitives

Each is centered on the origin with Y up, has material 0 and no creases, and faces out. They're
quads where they can be, the way subdivision surfaces like them, with caps as single polygons
and triangles only around a sphere's poles and a cone's tip.

| Function                                                  | What it makes                                                            |
| --------------------------------------------------------- | ------------------------------------------------------------------------ |
| `cube(size)`                                              | 8 vertices and 6 quads, `size` along each axis                           |
| `plane(size, segments)`                                   | A grid on Y = 0, facing +Y, of `segments[0]` by `segments[1]` quads      |
| `cylinder(radius, height, sides, segments)`               | `sides` around, `segments` quads tall, and two caps                      |
| `cone(bottom_radius, top_radius, height, sides)`          | A tip of triangles where a radius is 0, and a cap where it isn't         |
| `uv_sphere(radius, segments, rings)`                      | `segments` around, `rings` bands from pole to pole                       |
| `torus(major_radius, minor_radius, major_segments, ...)`  | `major_segments` around the Y axis, `minor_segments` around the tube     |
