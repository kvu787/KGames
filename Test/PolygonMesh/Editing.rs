// Library/PolygonMesh's editing operations, on small meshes whose results are
// known exactly: what each makes, what it selects, and that it keeps the mesh
// valid, consistently wound, and closed where it was.

#[path = "Common.rs"]
#[allow(dead_code)]
mod common;

use common::{assert_close, assert_near, assert_valid, signed_volume, sides_histogram};
use math::{Matrix, length, subtract};
use polygon_mesh::{Edge, Mesh, Selection, primitives};
use std::collections::BTreeSet;
use std::f32::consts::FRAC_PI_2;

// The cube's faces are -Z, +Z, -Y, +Y, -X and +X, and its top, +Y, is 3, 7,
// 6, 2. Vertex 0 is at -1, -1, -1, and 6 at 1, 1, 1.
const FRONT: usize = 1;
const TOP: usize = 3;

fn set<T: Ord>(items: impl IntoIterator<Item = T>) -> BTreeSet<T> {
    items.into_iter().collect()
}

fn counts(mesh: &Mesh) -> (usize, usize) {
    (mesh.vertex_count(), mesh.face_count())
}

// Valid, consistently wound and without holes.
fn assert_closed(mesh: &Mesh, what: &str) {
    assert_valid(mesh, what);
    let topology = mesh.topology();
    assert!(topology.is_closed(), "{what} has boundary edges: {:?}", topology.boundary_edges());
    assert_eq!(topology.euler_characteristic(), 2, "{what}'s Euler characteristic");
}

fn area(mesh: &Mesh, faces: impl IntoIterator<Item = usize>) -> f32 {
    faces.into_iter().map(|face| mesh.face_area(face)).sum()
}

#[test]
fn moving_vertices() {
    let mut cube = primitives::cube(2.0);
    let top = Selection::from_faces(&cube, [TOP]);
    cube.translate(top.vertices.iter().copied(), [0.0, 1.0, 0.0]);
    assert_eq!(cube.position(6), [1.0, 2.0, 1.0]);
    assert_eq!(cube.position(0), [-1.0, -1.0, -1.0]);

    let mut cube = primitives::cube(2.0);
    cube.scale(top.vertices.iter().copied(), [0.0, 1.0, 0.0], [2.0, 1.0, 0.5]);
    assert_eq!(cube.position(6), [2.0, 1.0, 0.5]);
    assert_eq!(cube.position(5), [1.0, -1.0, 1.0]);

    // Like Math's rotations, and like the same rotation as a matrix.
    let original = primitives::cube(2.0);
    let mut rotated = original.clone();
    rotated.rotate(0..8, [0.0; 3], [0.0, 2.0, 0.0], FRAC_PI_2);
    let mut transformed = original.clone();
    transformed.transform(0..8, [0.0; 3], &Matrix::rotation_y(FRAC_PI_2));
    for vertex in 0..8 {
        let expected = Matrix::rotation_y(FRAC_PI_2).transform_point(original.position(vertex));
        assert_near(rotated.position(vertex), expected, 1.0e-6, "a rotated vertex");
        assert_near(transformed.position(vertex), expected, 1.0e-6, "a transformed vertex");
    }
    assert_near(rotated.position(6), [1.0, 1.0, -1.0], 1.0e-6, "a corner turned a quarter about Y");
    // About a pivot: the pivot stays.
    let mut turned = original.clone();
    turned.rotate([6], [1.0, 1.0, 0.0], [0.0, 0.0, 1.0], FRAC_PI_2);
    assert_near(turned.position(6), [1.0, 1.0, 1.0], 1.0e-6, "a vertex on the axis");
    turned.rotate([5], [1.0, 1.0, 1.0], [0.0, 0.0, 1.0], FRAC_PI_2);
    assert_near(turned.position(5), [3.0, 1.0, 1.0], 1.0e-6, "a vertex turned about a pivot");
    let mut unmoved = original.clone();
    unmoved.rotate(0..8, [0.0; 3], [0.0; 3], 1.0);
    unmoved.translate([42], [1.0, 0.0, 0.0]);
    assert_eq!(unmoved, original);
}

#[test]
fn extruding_a_cube_s_face() {
    let mut cube = primitives::cube(2.0);
    cube.assign_material([TOP], 2);
    cube.set_crease(Edge::new(2, 3), 5.0);
    let selection = cube.extrude_faces([TOP]);
    assert_eq!(counts(&cube), (12, 10));
    assert_closed(&cube, "a cube with a face extruded");
    assert_eq!(selection.faces, set([TOP]));
    assert_eq!(selection.vertices, set(8..12));
    assert_eq!(selection.edges.len(), 4);
    // The walls are quads of the face's material, and the crease is on both
    // the old edge and the new one.
    assert_eq!(sides_histogram(&cube), [0, 10]);
    assert!(cube.faces[6..].iter().all(|face| face.material == 2));
    assert_eq!(cube.crease(Edge::new(2, 3)), 5.0);
    assert_eq!(cube.creases.len(), 2);
    cube.translate(selection.vertices.iter().copied(), [0.0, 1.0, 0.0]);
    assert_close(signed_volume(&cube), 12.0, 1.0e-5, "the extruded cube's volume");
    assert_eq!(cube.face_normal(TOP), [0.0, 1.0, 0.0]);
}

#[test]
fn extruding_regions() {
    // Two faces that share an edge: one region, with walls along its 6
    // boundary edges, and none along the edge between them.
    let mut cube = primitives::cube(2.0);
    let selection = cube.extrude_faces([TOP, FRONT]);
    assert_eq!(counts(&cube), (14, 12));
    assert_closed(&cube, "a cube with two faces extruded");
    assert_eq!(selection.faces, set([FRONT, TOP]));
    assert_eq!(selection.vertices.len(), 6);
    cube.translate(selection.vertices.iter().copied(), [0.0, 1.0, 1.0]);
    assert_valid(&cube, "a cube with two faces extruded and moved");
    assert!(signed_volume(&cube) > 8.0);

    // A lone quad becomes a closed box, with the quad left behind, flipped.
    let mut quad = primitives::plane([2.0, 2.0], [1, 1]);
    let selection = quad.extrude_faces([0]);
    assert_eq!(counts(&quad), (8, 6));
    assert_closed(&quad, "an extruded quad");
    quad.translate(selection.vertices.iter().copied(), [0.0, 1.0, 0.0]);
    assert_close(signed_volume(&quad), 4.0, 1.0e-5, "the extruded quad's volume");

    // A whole open grid does too.
    let mut grid = primitives::plane([2.0, 2.0], [2, 2]);
    let selection = grid.extrude_faces(0..4);
    assert_eq!(counts(&grid), (18, 16));
    assert_closed(&grid, "an extruded grid");
    grid.translate(selection.vertices.iter().copied(), [0.0, 0.5, 0.0]);
    assert_close(signed_volume(&grid), 2.0, 1.0e-5, "the extruded grid's volume");

    // A face in the middle of a grid: the walls join the grid, and nothing is
    // left behind.
    let mut grid = primitives::plane([3.0, 3.0], [3, 3]);
    grid.extrude_faces([4]);
    assert_eq!(counts(&grid), (20, 13));
    assert_valid(&grid, "a grid with its middle extruded");
    assert_eq!(grid.topology().boundary_edges().len(), 12);

    // A vertex inside the region, which nothing else uses, moves with it.
    let mut grid = primitives::plane([3.0, 3.0], [3, 3]);
    let selection = grid.extrude_faces([0, 1, 3, 4]);
    assert_eq!(counts(&grid), (24, 17));
    assert_valid(&grid, "a grid with a corner extruded");
    assert!(selection.vertices.contains(&5) && selection.vertices.len() == 9);
}

#[test]
fn extruding_edges() {
    // The quad is 0, 2, 3, 1, so it goes from 1 to 0, and the new quad from 0
    // to 1, then back along the copies, 5 of 1 and 4 of 0.
    let mut quad = primitives::plane([2.0, 2.0], [1, 1]);
    quad.set_crease(Edge::new(0, 1), 1.5);
    let selection = quad.extrude_edges([Edge::new(0, 1), Edge::new(0, 3)]);
    assert_eq!(counts(&quad), (6, 2));
    assert_valid(&quad, "a quad with an edge extruded");
    assert_eq!(quad.faces[1].vertices, [0, 1, 5, 4]);
    assert_eq!(selection.edges, set([Edge::new(4, 5)]));
    assert_eq!(selection.vertices, set([4, 5]));
    assert!(selection.faces.is_empty());
    assert_eq!(quad.crease(Edge::new(4, 5)), 1.5);

    let mut quad = primitives::plane([2.0, 2.0], [1, 1]);
    let boundary = quad.topology().boundary_edges();
    let selection = quad.extrude_edges(boundary);
    assert_eq!(counts(&quad), (8, 5));
    assert_valid(&quad, "a quad with its boundary extruded");
    assert_eq!(selection.edges.len(), 4);
    assert_eq!(selection.faces, set([]));
    quad.scale(selection.vertices.iter().copied(), [0.0; 3], [2.0, 1.0, 2.0]);
    assert_close(area(&quad, 0..5), 16.0, 1.0e-5, "the widened quad's area");
}

#[test]
fn insetting_a_region() {
    let mut cube = primitives::cube(2.0);
    let selection = cube.inset_region([TOP], 0.25);
    assert_eq!(counts(&cube), (12, 10));
    assert_closed(&cube, "a cube with a face inset");
    assert_eq!(selection.faces, set([TOP]));
    for &vertex in &selection.vertices {
        let [x, y, z] = cube.position(vertex);
        assert!((x.abs() - 0.75).abs() < 1.0e-6 && y == 1.0 && (z.abs() - 0.75).abs() < 1.0e-6);
    }
    assert_close(cube.face_area(TOP), 2.25, 1.0e-5, "the inset face's area");

    // Two faces that share an edge: the copies of its ends stay on it.
    let mut cube = primitives::cube(2.0);
    let selection = cube.inset_region([TOP, FRONT], 0.25);
    assert_eq!(counts(&cube), (14, 12));
    assert_closed(&cube, "a cube with two faces inset");
    assert_eq!(selection.faces, set([FRONT, TOP]));
    let positions: Vec<[f32; 3]> = selection.vertices.iter().map(|&vertex| cube.position(vertex)).collect();
    assert!(positions.contains(&[0.75, 1.0, 1.0]) && positions.contains(&[-0.75, 1.0, 1.0]));

    // Flat: the copies are the thickness from each edge, farther at corners.
    let mut strip = primitives::plane([2.0, 1.0], [2, 1]);
    let selection = strip.inset_region([0, 1], 0.25);
    assert_eq!(counts(&strip), (12, 8));
    assert_valid(&strip, "an inset strip");
    assert_close(area(&strip, selection.faces.iter().copied()), 0.75, 1.0e-5, "the inset strip's area");
    assert_close(area(&strip, 0..8), 2.0, 1.0e-5, "the whole strip's area");
    for expected in [[0.0, 0.0, -0.25], [-0.75, 0.0, -0.25], [0.75, 0.0, 0.25]] {
        let found =
            selection.vertices.iter().any(|&vertex| length(subtract(strip.position(vertex), expected)) < 1.0e-6);
        assert!(found, "no copy at {expected:?}");
    }
}

#[test]
fn insetting_faces_on_their_own() {
    let mut cube = primitives::cube(2.0);
    let selection = cube.inset_individual([TOP, FRONT], 0.25);
    assert_eq!(counts(&cube), (16, 14));
    assert_closed(&cube, "a cube with two faces inset on their own");
    assert_eq!(selection.faces, set([FRONT, TOP]));
    assert_eq!(selection.vertices.len(), 8);

    let mut quad = primitives::plane([2.0, 2.0], [1, 1]);
    quad.inset_individual([0], 0.5);
    assert_eq!(counts(&quad), (8, 5));
    assert_valid(&quad, "an inset quad");
    assert_close(quad.face_area(0), 1.0, 1.0e-5, "the inset quad's area");
}

#[test]
fn deleting() {
    let mut cube = primitives::cube(2.0);
    cube.set_crease(Edge::new(4, 5), 2.0);
    cube.set_crease(Edge::new(0, 1), 2.0);
    let selection = cube.delete_vertices([0]);
    assert!(selection.is_empty());
    assert_eq!(counts(&cube), (7, 3));
    assert_valid(&cube, "a cube without a corner");
    assert_eq!(cube.topology().boundary_edges().len(), 6);
    assert_eq!(cube.position(0), [1.0, -1.0, -1.0]);
    assert_eq!(cube.faces[0].vertices, [3, 4, 5, 6]);
    // The crease follows its edge's renumbering, and goes with its edge.
    assert_eq!(cube.creases.iter().collect::<Vec<_>>(), [(&Edge::new(3, 4), &2.0)]);

    // An edge takes its faces, and its vertices if nothing else uses them.
    let mut cube = primitives::cube(2.0);
    cube.delete_edges([Edge::new(2, 3)]);
    assert_eq!(counts(&cube), (8, 4));
    let mut quad = primitives::plane([2.0, 2.0], [1, 1]);
    quad.delete_edges([Edge::new(0, 1)]);
    assert_eq!(counts(&quad), (2, 0));

    // Faces take the vertices that nothing else uses.
    let mut cube = primitives::cube(2.0);
    cube.delete_faces([TOP]);
    assert_eq!(counts(&cube), (8, 5));
    assert_eq!(cube.topology().boundary_edges().len(), 4);
    cube.delete_faces(0..5);
    assert_eq!(counts(&cube), (0, 0));
    let mut grid = primitives::plane([3.0, 3.0], [3, 3]);
    grid.delete_faces([4]);
    assert_eq!(counts(&grid), (16, 8));
    grid.delete_faces([0]);
    assert_eq!(counts(&grid), (15, 7));

    // Only faces leaves the vertices, loose.
    let mut cube = primitives::cube(2.0);
    cube.delete_only_faces(0..6);
    assert_eq!(counts(&cube), (8, 0));
    assert_eq!(cube.remove_unused_vertices(), vec![None; 8]);
    assert_eq!(counts(&cube), (0, 0));
}

#[test]
fn removing_unused_vertices_renumbers_the_rest() {
    let mut grid = primitives::plane([3.0, 3.0], [3, 3]);
    let second = grid.faces[1].clone();
    grid.delete_only_faces([0]);
    assert_eq!(counts(&grid), (16, 8));
    let renumbered = grid.remove_unused_vertices();
    assert_eq!(renumbered[0], None);
    assert_eq!(renumbered[1..], (0..15).map(Some).collect::<Vec<_>>());
    assert_eq!(grid.faces[0].vertices, second.vertices.iter().map(|vertex| vertex - 1).collect::<Vec<_>>());
    assert_eq!(grid.remove_unused_vertices(), (0..15).map(Some).collect::<Vec<_>>());
}

#[test]
fn making_faces() {
    // Filling the cube's top back in follows the hole's boundary.
    let mut cube = primitives::cube(2.0);
    cube.assign_material(0..6, 4);
    cube.delete_only_faces([TOP]);
    let selection = cube.make_face([2, 3, 6, 7]).unwrap();
    assert_closed(&cube, "a cube with its top made again");
    assert_eq!(selection.faces, set([5]));
    assert_eq!(cube.faces[5].material, 4);
    assert_eq!(cube.face_normal(5), [0.0, 1.0, 0.0]);
    assert_close(signed_volume(&cube), 8.0, 1.0e-5, "the cube's volume");

    // Three of the hole's four vertices: around their middle, wound like the
    // faces around them.
    let mut cube = primitives::cube(2.0);
    cube.delete_only_faces([TOP]);
    cube.make_face([2, 3, 6]).unwrap();
    assert_valid(&cube, "a cube with a triangle in its top");
    assert_eq!(cube.topology().boundary_edges().len(), 3);
    assert_eq!(cube.face_normal(5), [0.0, 1.0, 0.0]);

    let mut cube = primitives::cube(2.0);
    assert_eq!(cube.make_face([2, 3]).unwrap_err(), "Making a face takes at least 3 vertices.");
    assert_eq!(cube.make_face([2, 3, 6, 7]).unwrap_err(), "There's already a face with these vertices.");

    // Loose vertices, with nothing to agree with.
    let mut quad = primitives::plane([2.0, 2.0], [1, 1]);
    quad.delete_only_faces([0]);
    quad.make_face([0, 1, 2, 3]).unwrap();
    assert_valid(&quad, "a quad made from loose vertices");
    assert_eq!(quad.face_normal(0)[1].abs(), 1.0);
    assert_close(quad.face_area(0), 4.0, 1.0e-5, "the made quad's area");
}

#[test]
fn filling_holes() {
    let mut cube = primitives::cube(2.0);
    cube.delete_only_faces([TOP]);
    let boundary = cube.topology().boundary_edges();
    let selection = cube.fill(boundary).unwrap();
    assert_closed(&cube, "a cube with its top filled");
    assert_eq!(selection.faces, set([5]));
    assert_close(signed_volume(&cube), 8.0, 1.0e-5, "the filled cube's volume");

    // Both ends of a tube.
    let mut tube = primitives::cylinder(1.0, 2.0, 8, 1);
    tube.delete_only_faces([8, 9]);
    let boundary = tube.topology().boundary_edges();
    let selection = tube.fill(boundary).unwrap();
    assert_closed(&tube, "a tube with its ends filled");
    assert_eq!(selection.faces, set([8, 9]));
    assert_eq!(sides_histogram(&tube), [0, 8, 0, 0, 0, 2]);
    assert!(signed_volume(&tube) > 0.0);

    let mut cube = primitives::cube(2.0);
    let edges = cube.edges();
    assert_eq!(cube.fill(edges).unwrap_err(), "The selection has no loop of boundary edges around a hole to fill.");
}

#[test]
fn merging_at_the_center() {
    // The top collapses into a point: a pyramid.
    let mut cube = primitives::cube(2.0);
    let selection = cube.merge_at_center([3, 7, 6, 2]);
    assert_eq!(counts(&cube), (5, 5));
    assert_closed(&cube, "a pyramid");
    assert_eq!(selection.vertices, set([2]));
    assert_eq!(cube.position(2), [0.0, 1.0, 0.0]);
    assert_eq!(sides_histogram(&cube), [4, 1]);

    // An edge collapses: its two faces become triangles.
    let mut cube = primitives::cube(2.0);
    cube.set_crease(Edge::new(3, 7), 3.0);
    cube.merge_at_center([2, 3]);
    assert_eq!(counts(&cube), (7, 6));
    assert_closed(&cube, "a cube with an edge merged");
    assert_eq!(sides_histogram(&cube), [2, 4]);
    assert_eq!(cube.position(2), [0.0, 1.0, -1.0]);
    // Vertex 7 is 6 now, and the crease went with it.
    assert_eq!(cube.crease(Edge::new(2, 6)), 3.0);
}

#[test]
fn merging_by_distance() {
    // Two quads side by side, each with its own copy of the edge between them.
    let mut mesh = Mesh::new();
    for position in [
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [1.0, 0.0, 0.00001],
        [2.0, 0.0, 0.0],
        [2.0, 0.0, 1.0],
        [1.0, 0.0, 1.0],
    ] {
        mesh.add_vertex(position);
    }
    mesh.add_face(vec![0, 3, 2, 1], 0);
    mesh.add_face(vec![4, 7, 6, 5], 0);
    let selection = mesh.merge_by_distance(0..8, 0.001);
    assert_eq!(counts(&mesh), (6, 2));
    assert_valid(&mesh, "two quads merged");
    assert_eq!(mesh.topology().boundary_edges().len(), 6);
    assert_eq!(selection.vertices, set(0..6));
    // The first of each pair stays where it was.
    assert_eq!(mesh.position(1), [1.0, 0.0, 0.0]);

    // Nothing within the distance: nothing changes.
    let mut cube = primitives::cube(2.0);
    cube.merge_by_distance(0..8, 0.5);
    assert_eq!(cube, primitives::cube(2.0));

    // An extrusion that hasn't moved merges back into the cube.
    let mut cube = primitives::cube(2.0);
    cube.extrude_faces([TOP]);
    cube.merge_by_distance(0..12, 0.0001);
    assert_eq!(counts(&cube), (8, 6));
    assert_closed(&cube, "a cube merged back");
    assert_close(signed_volume(&cube), 8.0, 1.0e-5, "the merged cube's volume");
}

#[test]
fn subdividing_faces() {
    let mut cube = primitives::cube(2.0);
    cube.set_crease(Edge::new(2, 3), 3.0);
    let selection = cube.subdivide_faces([TOP]);
    assert_eq!(counts(&cube), (13, 9));
    assert_closed(&cube, "a cube with its top subdivided");
    assert_eq!(selection.faces, set([TOP, 6, 7, 8]));
    // The faces next to it get the midpoints of the edges that they share.
    assert_eq!(sides_histogram(&cube), [0, 5, 4]);
    assert_close(area(&cube, selection.faces.iter().copied()), 4.0, 1.0e-5, "the top's area");
    let positions: Vec<[f32; 3]> = selection.vertices.iter().map(|&vertex| cube.position(vertex)).collect();
    assert!(positions.contains(&[0.0, 1.0, 0.0]) && positions.contains(&[0.0, 1.0, -1.0]));
    // Both halves of the creased edge are creased.
    assert_eq!(cube.creases.len(), 2);
    assert!(cube.creases.values().all(|&sharpness| sharpness == 3.0));

    let mut cube = primitives::cube(2.0);
    cube.subdivide_faces(0..6);
    assert_eq!(counts(&cube), (26, 24));
    assert_closed(&cube, "a subdivided cube");
    assert_close(signed_volume(&cube), 8.0, 1.0e-5, "the subdivided cube's volume");

    // A triangle splits into 4, and a pentagon into 5 quads.
    let mut triangle = Mesh::new();
    for position in [[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 2.0, 0.0]] {
        triangle.add_vertex(position);
    }
    triangle.add_face(vec![0, 1, 2], 7);
    triangle.subdivide_faces([0]);
    assert_eq!(counts(&triangle), (6, 4));
    assert_eq!(sides_histogram(&triangle), [4]);
    assert_valid(&triangle, "a subdivided triangle");
    assert_close(area(&triangle, 0..4), 2.0, 1.0e-6, "the triangle's area");
    assert!(triangle.faces.iter().all(|face| face.material == 7));
    let mut pentagon = primitives::cone(1.0, 0.0, 1.0, 5);
    pentagon.subdivide_faces([5]);
    assert_eq!(counts(&pentagon), (6 + 5 + 1, 6 + 4));
    assert_closed(&pentagon, "a pyramid with its base subdivided");
}

#[test]
fn loop_cuts() {
    // Around the cube's sides, a quarter of the way up from vertex 0.
    let mut cube = primitives::cube(2.0);
    let selection = cube.loop_cut(Edge::new(0, 3), 0.25).unwrap();
    assert_eq!(counts(&cube), (12, 10));
    assert_closed(&cube, "a cube with a loop cut");
    assert_eq!(selection.vertices, set(8..12));
    assert_eq!(selection.edges.len(), 4);
    assert!(selection.vertices.iter().all(|&vertex| cube.position(vertex)[1] == -0.5));
    assert_eq!(sides_histogram(&cube), [0, 10]);
    assert_close(signed_volume(&cube), 8.0, 1.0e-5, "the cut cube's volume");

    // Across a grid, from boundary to boundary.
    let mut grid = primitives::plane([3.0, 3.0], [3, 3]);
    let selection = grid.loop_cut(Edge::new(5, 9), 0.5).unwrap();
    assert_eq!(counts(&grid), (20, 12));
    assert_valid(&grid, "a cut grid");
    assert_eq!(grid.topology().boundary_edges().len(), 14);
    assert_eq!(selection.edges.len(), 3);

    // Ending at faces that aren't quads, which get the new vertices.
    let mut cylinder = primitives::cylinder(1.0, 2.0, 8, 1);
    cylinder.loop_cut(Edge::new(0, 1), 0.5).unwrap();
    assert_eq!(counts(&cylinder), (18, 11));
    assert_closed(&cylinder, "a cut cylinder");
    assert_eq!(sides_histogram(&cylinder), [0, 9, 0, 0, 0, 0, 2]);
    let mut sphere = primitives::uv_sphere(1.0, 8, 4);
    sphere.loop_cut(Edge::new(1, 2), 0.5).unwrap();
    assert_eq!(counts(&sphere), (29, 34));
    assert_closed(&sphere, "a cut sphere");

    let mut pyramid = primitives::cone(1.0, 0.0, 1.0, 3);
    assert_eq!(
        pyramid.loop_cut(Edge::new(0, 1), 0.5).unwrap_err(),
        "A loop cut goes across quads, and this edge has none next to it."
    );
    assert_eq!(pyramid.loop_cut(Edge::new(0, 9), 0.5).unwrap_err(), "The mesh has no edge from vertex 0 to 9.");
}

#[test]
fn flipping_and_recalculating_normals() {
    let mut cube = primitives::cube(2.0);
    cube.flip_faces(0..6);
    assert_valid(&cube, "an inside-out cube");
    assert_close(signed_volume(&cube), -8.0, 1.0e-5, "the inside-out cube's volume");
    cube.recalculate_normals_outside(0..6);
    assert_close(signed_volume(&cube), 8.0, 1.0e-5, "the cube's volume");

    let mut cube = primitives::cube(2.0);
    cube.flip_faces([FRONT, TOP]);
    assert!(!cube.topology().is_consistently_wound());
    cube.recalculate_normals_outside(0..6);
    assert_eq!(cube, primitives::cube(2.0));

    let mut torus = primitives::torus(1.0, 0.25, 12, 6);
    torus.flip_faces((0..72).filter(|face| face % 3 != 0));
    torus.recalculate_normals_outside(0..72);
    assert_valid(&torus, "a torus with its normals recalculated");
    assert!(signed_volume(&torus) > 0.0);

    // An open grid agrees with its first face.
    let mut grid = primitives::plane([3.0, 3.0], [3, 3]);
    grid.flip_faces([4, 7]);
    grid.recalculate_normals_outside(0..9);
    assert_eq!(grid, primitives::plane([3.0, 3.0], [3, 3]));
}

#[test]
fn creases_and_materials() {
    let mut cube = primitives::cube(2.0);
    let top = Selection::from_faces(&cube, [TOP]);
    cube.crease_edges(top.edges.iter().copied().chain([Edge::new(0, 6)]), 2.5);
    assert_eq!(cube.creases.len(), 4);
    assert_eq!(cube.crease(Edge::new(2, 3)), 2.5);
    cube.crease_edges([Edge::new(2, 3)], 0.0);
    assert_eq!(cube.creases.len(), 3);
    cube.crease_edges([Edge::new(3, 7)], 25.0);
    assert_eq!(cube.crease(Edge::new(3, 7)), 10.0);

    cube.assign_material([1, 2, 99], 3);
    assert_eq!(cube.faces.iter().map(|face| face.material).collect::<Vec<_>>(), [0, 3, 3, 0, 0, 0]);
    assert_eq!(cube.material_count(), 4);
}

// Two holes that touch at a vertex: two triangles that share only vertex
// 0, filled from their boundary. Each gets a face of its own, instead of
// one face that goes through vertex 0 twice.
#[test]
fn filling_holes_that_touch_at_a_vertex() {
    let mut mesh = Mesh::new();
    for position in [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [-1.0, 0.0, 0.0], [1.0, 0.0, 1.0], [-1.0, 0.0, 1.0]] {
        mesh.add_vertex(position);
    }
    mesh.add_face(vec![2, 0, 4], 0);
    mesh.add_face(vec![0, 1, 3], 0);
    let edges = mesh.edges();
    let selection = mesh.fill(edges).unwrap();
    assert_eq!(selection.faces, set([2, 3]));
    assert_valid(&mesh, "two touching triangles with their backs filled");
    assert_eq!(sides_histogram(&mesh), [4]);
    assert_eq!(mesh.topology().boundary_edges().len(), 0);
}
