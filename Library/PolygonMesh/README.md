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
| Editing       | `translate`, `rotate`, `scale`, `transform`, `extrude_faces`, `extrude_edges`,       |
|               | `inset_region`, `inset_individual`, `delete_vertices`, `delete_edges`,               |
|               | `delete_faces`, `delete_only_faces`, `remove_unused_vertices`, `make_face`, `fill`,  |
|               | `merge_at_center`, `merge_by_distance`, `subdivide_faces`, `loop_cut`,               |
|               | `flip_faces`, `recalculate_normals_outside`, `crease_edges`, `assign_material`,      |
|               | all methods of `Mesh`                                                                |
| Subdivision   | `Mesh::catmull_clark`, `SubdivisionSurface`'s `mesh` and `base_faces`                |

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

## Editing

The edits are Blender's, in edit mode, as methods of `Mesh`. Each takes its vertices, edges or
faces as any iterator of them, like a `Selection`'s sets or an array, and leaves out the ones that
aren't in the mesh. One that changes which faces there are returns the selection that results, as
Blender leaves it, and the others leave the selection as it was.

| Edit                           | What it does                                                                           | What it selects |
| ------------------------------ | -------------------------------------------------------------------------------------- | --------------- |
| `translate`, `rotate`, `scale` | Moves the vertices by an offset, about an axis through a pivot, or by factors about it | As before       |
| `transform`                    | Moves the vertices by a `Matrix`, about a pivot                                        | As before       |
| `extrude_faces`                | Moves the faces onto new vertices as a region, with quads along its boundary           | The region      |
| `extrude_edges`                | Makes a quad from each edge to a copy of it                                            | The copies      |
| `inset_region`                 | Shrinks the region onto copies of its boundary, with quads between                     | The region      |
| `inset_individual`             | Shrinks each face on its own onto a copy of it, with quads between                     | The copies      |
| `delete_vertices`              | Deletes the vertices, and the faces that use them                                      | Nothing         |
| `delete_edges`                 | Deletes the edges' faces, and the edges' vertices that are left unused                 | Nothing         |
| `delete_faces`                 | Deletes the faces, and their vertices that are left unused                             | Nothing         |
| `delete_only_faces`            | Deletes the faces, and leaves their vertices, maybe loose                              | Nothing         |
| `remove_unused_vertices`       | Removes loose vertices, and returns each old vertex's new number                       | -               |
| `make_face`                    | Makes a face of the vertices, along their boundary or around their middle              | The face        |
| `fill`                         | Fills each hole that the boundary edges go around with a face                          | The faces       |
| `merge_at_center`              | Merges the vertices into one, at their average                                         | The vertex      |
| `merge_by_distance`            | Merges each vertex into the first within the distance of it                            | What's left     |
| `subdivide_faces`              | Cuts each face into 4 triangles, or a quad at each corner, without smoothing           | The pieces      |
| `loop_cut`                     | Cuts a loop across the edge ring through an edge                                       | The new loop    |
| `flip_faces`                   | Reverses the faces' winding                                                            | As before       |
| `recalculate_normals_outside`  | Winds the faces consistently, and each surface facing out                              | As before       |
| `crease_edges`                 | Sets the edges' sharpness                                                              | As before       |
| `assign_material`              | Sets the faces' material                                                               | As before       |

- **Numbers.** Edits keep the numbers of the faces that they don't change, and add new vertices
  and faces at the end, so extruded and inset regions keep their faces' numbers. Edits that
  remove vertices renumber the rest in order, without gaps, and `remove_unused_vertices` returns
  each old vertex's new number.
- **Where new vertices start.** Extruded vertices start where the ones that they're copied from
  are, so a modeler moves the selection that the edit returns, as Blender does after E, for
  example along the region's `face_normal`. Inset copies are `thickness` inside each edge of the
  boundary that they're on, in the planes of its faces, and farther at corners, up to 4 times as
  far.
- **Regions.** An extruded region's walls go along each edge that only one of its faces has.
  Vertices in its middle that nothing else uses move with it rather than being copied, and a
  region that shares no edge with another face leaves a flipped copy of itself behind, so that
  a lone quad becomes a closed box.
- **Materials.** Walls, inset quads and pieces take the material of the face that they come from,
  and made and filled faces take a neighbor's.
- **Creases.** A crease stays with its edge as vertices are renumbered, and goes when its edge
  does. Both halves of a cut edge keep its sharpness, extruded edges copy it to their copies, and
  edges that merge keep the sharper.
- **Merging.** Faces lose the sides that collapse, split where they come to touch themselves, and
  go when they're left with fewer than 3 vertices or the same vertices as another face, so an
  extrusion that hasn't moved merges back by distance into what it was.
- **Cuts.** Faces next to subdivided ones, and at the ends of a loop cut, get the new vertices on
  the edges that they share, so no edge ends partway along another, like Blender's Create N-Gons.
  A loop cut's `fraction` is from the edge's first vertex, the smaller, and the other edges of
  the ring are cut the same way along, from their ends beside it.
- **Wire edges.** A mesh has no edges without faces, so where Blender would leave one, like after
  `delete_only_faces`, the vertices stay but the edge doesn't.

## Subdivision

`Mesh::catmull_clark(levels)` subdivides the mesh into a `SubdivisionSurface`: the subdivided
`mesh`, and for each of its faces, the base face that it comes from, in `base_faces`, for
picking and drawing what's selected. It's Catmull-Clark subdivision, the Catmark scheme as
[OpenSubdiv](https://graphics.pixar.com/opensubdiv/docs/subdivision_surfaces.html) defines it,
with boundary interpolation `VTX_BOUNDARY_EDGE_AND_CORNER` and uniform creasing.

Each level turns a face of n vertices into n quads, one at each corner, from the corner to the
point of its edge after it, the face's point, and the point of its edge before it, so after one
level every face is a quad, triangles and n-gons too. The quads keep their face's material, and
come in their face's order. The base mesh's vertices keep their numbers at every level, then
come the points of the edges, in `Topology`'s order, then the points of the faces. Loose
vertices stay where they are.

### Points

A face's point is the average of its n vertices:

$$
F = \frac{1}{n} \sum_{i=1}^{n} v_i
$$

An edge from a to b, between faces whose points are $F_1$ and $F_2$, has a smooth point and a
sharp one:

$$
E_{\text{smooth}} = \frac{a + b + F_1 + F_2}{4} \qquad E_{\text{sharp}} = \frac{a + b}{2}
$$

A vertex S with n edges, to $e_1$ to $e_n$, and n faces, whose points are $F_1$ to $F_n$, moves
by one of three masks: smooth, crease, along its two sharp edges, to $e_a$ and $e_b$, or corner:

$$
V_{\text{smooth}} = \frac{n - 2}{n} S + \frac{1}{n^2} \sum_{j=1}^{n} e_j + \frac{1}{n^2} \sum_{j=1}^{n} F_j
$$

$$
V_{\text{crease}} = \frac{e_a + 6 S + e_b}{8} \qquad V_{\text{corner}} = S
$$

Which mask is the vertex's rule, from how many of its edges are sharp:

| Sharp edges at the vertex                 | Rule   | Mask   |
| ----------------------------------------- | ------ | ------ |
| None                                      | Smooth | Smooth |
| 1                                         | Dart   | Smooth |
| 2                                         | Crease | Crease |
| 3 or more, or the vertex is sharp itself  | Corner | Corner |

### Sharpness

An edge is sharp when its sharpness is more than 0, and smooth at 0. Boundary edges, with one
face, are infinitely sharp, and so are non-manifold ones, whatever their creases say. A vertex is
infinitely sharp, a corner, when it's on the boundary with only one face, which on a manifold is
a boundary vertex of 2 edges, like a plane's corners, so they stay put. Vertices where the
surface isn't a manifold are corners too: on a non-manifold edge, on two stretches of boundary,
or with faces that don't make one fan around them.

Each level, both halves of an edge get its sharpness less 1, and infinitely sharp edges, of
sharpness `INFINITELY_SHARP` or more, stay infinitely sharp:

$$
s' = \max(s - 1, 0)
$$

So a crease of sharpness s is sharp for about s levels, and then smooth. Between, the rules
blend, as OpenSubdiv's do:

- An edge that's sharp now and smooth after this level, of sharpness s up to 1, blends its
  points by its sharpness:

$$
E = s E_{\text{sharp}} + (1 - s) E_{\text{smooth}}
$$

- A vertex whose rule is a crease or a corner now, and different after this level, by the
  sharpness that its edges will have, blends the two masks by the fractional weight w: the
  average sharpness of the k edges at the vertex that are sharp now and smooth after, at most 1:

$$
V = w V_{\text{now}} + (1 - w) V_{\text{after}} \qquad w = \min\left(1, \frac{1}{k} \sum_{i=1}^{k} s_i\right)
$$

So a cube whose edges have sharpness 1 subdivides like an infinitely sharp one for its first
level, and like a smooth one after, and sharpness 0.5 is halfway between the two for the first.
