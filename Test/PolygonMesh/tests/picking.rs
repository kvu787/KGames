// Library/PolygonMesh's picking: ray casts against faces, from either side and
// through concave faces only where they are, the vertex and edge nearest a
// ray, and the bounds and centers of meshes and selections.

#[allow(dead_code)]
mod common;

use common::{assert_close, assert_near};
use polygon_mesh::{Edge, Hit, Mesh, primitives};

fn assert_hit(hit: Option<Hit>, face: usize, distance: f32, point: [f32; 3]) {
    let hit = hit.expect("the ray missed");
    assert_eq!(hit.face, face);
    assert_close(hit.distance, distance, 1.0e-5, "the distance to the hit");
    assert_near(hit.point, point, 1.0e-5, "the point hit");
}

#[test]
fn rays_hit_the_nearest_face() {
    let cube = primitives::cube(2.0);
    // The front, +Z, before the back, even along the edge between its
    // triangles.
    assert_hit(cube.ray_cast([0.0, 0.0, 5.0], [0.0, 0.0, -1.0]), 1, 4.0, [0.0, 0.0, 1.0]);
    assert_hit(cube.ray_cast([0.5, -0.5, 5.0], [0.0, 0.0, -1.0]), 1, 4.0, [0.5, -0.5, 1.0]);
    // The direction's length doesn't matter.
    assert_hit(cube.ray_cast([0.25, -0.5, 5.0], [0.0, 0.0, -10.0]), 1, 4.0, [0.25, -0.5, 1.0]);
    // From inside, a face's back.
    assert_hit(cube.ray_cast([0.0; 3], [1.0, 0.0, 0.0]), 5, 1.0, [1.0, 0.0, 0.0]);
    assert_hit(cube.ray_cast([0.5, 3.0, 0.5], [0.0, -1.0, 0.0]), 3, 2.0, [0.5, 1.0, 0.5]);
    // At an angle.
    assert_hit(cube.ray_cast([-3.0, 0.0, 0.0], [1.0, 0.0, 0.25]), 4, 4.25f32.sqrt(), [-1.0, 0.0, 0.5]);

    assert_eq!(cube.ray_cast([5.0, 5.0, 5.0], [1.0, 0.0, 0.0]), None);
    assert_eq!(cube.ray_cast([0.0, 0.0, 5.0], [0.0, 0.0, 1.0]), None);
    assert_eq!(cube.ray_cast([0.0, 0.0, 5.0], [0.0; 3]), None);
    assert_eq!(Mesh::new().ray_cast([0.0; 3], [1.0, 0.0, 0.0]), None);

    // Which quad of a grid, from below.
    let grid = primitives::plane([3.0, 3.0], [3, 3]);
    assert_hit(grid.ray_cast([0.4, -2.0, 0.6], [0.0, 1.0, 0.0]), 7, 2.0, [0.4, 0.0, 0.6]);
}

#[test]
fn rays_through_a_concave_face_hit_only_where_it_is() {
    // An L, missing its corner at x and z from 1 to 2.
    let mut mesh = Mesh::new();
    for [x, z] in [[0.0, 0.0], [0.0, 2.0], [1.0, 2.0], [1.0, 1.0], [2.0, 1.0], [2.0, 0.0]] {
        mesh.add_vertex([x, 0.0, z]);
    }
    mesh.add_face((0..6).collect(), 0);
    assert_eq!(mesh.face_normal(0), [0.0, 1.0, 0.0]);
    let down = [0.0, -1.0, 0.0];
    assert_eq!(mesh.ray_cast([1.5, 1.0, 1.5], down), None);
    for [x, z] in [[0.5, 0.5], [0.5, 1.5], [1.5, 0.5], [0.9, 1.9]] {
        assert_hit(mesh.ray_cast([x, 1.0, z], down), 0, 1.0, [x, 0.0, z]);
    }
}

#[test]
fn the_nearest_vertex_and_edge_to_a_ray() {
    let cube = primitives::cube(2.0);
    let along = [0.0, 0.0, -1.0];
    // Corners 6, at 1, 1, 1, and 2, at 1, 1, -1, are as near the ray, and 6
    // is nearer its start.
    assert_eq!(cube.nearest_vertex([1.05, 1.0, 5.0], along, 0.1), Some(6));
    assert_eq!(cube.nearest_vertex([0.95, 0.98, 5.0], along, 0.1), Some(6));
    assert_eq!(cube.nearest_vertex([1.05, 1.0, 5.0], along, 0.01), None);
    assert_eq!(cube.nearest_vertex([1.0, 1.0, -5.0], along, 0.1), None);
    // The front top edge, 6 to 7, before the back one, 2 to 3.
    assert_eq!(cube.nearest_edge([0.0, 1.02, 5.0], along, 0.1), Some(Edge::new(6, 7)));
    assert_eq!(cube.nearest_edge([0.3, 1.02, 5.0], [0.0, 0.0, 1.0], 0.1), None);
    // Across the middle of the right top edge, 2 to 6, before the left one.
    assert_eq!(cube.nearest_edge([5.0, 1.02, 0.2], [-1.0, 0.0, 0.0], 0.05), Some(Edge::new(2, 6)));
    assert_eq!(cube.nearest_edge([0.0, 0.0, 5.0], along, 0.5), None);
}

#[test]
fn bounds_and_centers() {
    let cube = primitives::cube(2.0);
    assert_eq!(cube.bounds(), Some([[-1.0; 3], [1.0; 3]]));
    let plane = primitives::plane([4.0, 2.0], [1, 1]);
    assert_eq!(plane.bounds(), Some([[-2.0, 0.0, -1.0], [2.0, 0.0, 1.0]]));
    assert_eq!(Mesh::new().bounds(), None);
    assert_eq!(cube.center_of([3, 7, 6, 2]), Some([0.0, 1.0, 0.0]));
    assert_eq!(cube.center_of([6, 6, 99]), Some([1.0; 3]));
    assert_eq!(cube.center_of([]), None);
}

#[test]
fn rays_hit_a_subdivision_surface_s_base_faces() {
    let cube = primitives::cube(2.0);
    let surface = cube.catmull_clark(2);
    let hit = surface.ray_cast([0.3, 0.2, 5.0], [0.0, 0.0, -1.0]).unwrap();
    assert_eq!(hit.face, 1);
    // The smooth surface is inside the cube.
    assert!(hit.distance > 4.0 && hit.distance < 4.5);
    let side = surface.ray_cast([5.0, 0.1, -0.2], [-1.0, 0.0, 0.0]).unwrap();
    assert_eq!(side.face, 5);
}
