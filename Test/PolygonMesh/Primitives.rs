// Library/PolygonMesh's primitives: each is valid, closed where it should be,
// wound so that its faces point out, has the Euler characteristic of its
// shape, and is quads where it can be.

#[path = "Common.rs"]
#[allow(dead_code)]
mod common;

use common::{assert_close, assert_valid, signed_volume, sides_histogram};
use math::{dot, subtract};
use polygon_mesh::{Mesh, primitives};
use std::f32::consts::PI;

// What every closed primitive must be: valid, without boundary or non-manifold
// edges, of Euler characteristic `euler`, enclosing a positive volume.
fn check_closed(mesh: &Mesh, euler: i64, what: &str) {
    assert_valid(mesh, what);
    let topology = mesh.topology();
    assert!(topology.is_closed(), "{what} has boundary edges: {:?}", topology.boundary_edges());
    assert!(topology.non_manifold_edges().is_empty(), "{what} has non-manifold edges");
    assert_eq!(topology.euler_characteristic(), euler, "{what}'s Euler characteristic");
    assert!(signed_volume(mesh) > 0.0, "{what} faces in");
    assert!((0..mesh.vertex_count() as u32).all(|vertex| topology.valence(vertex) > 0), "{what} has loose vertices");
}

// Every face of a convex primitive faces away from its center.
fn check_faces_out_of(mesh: &Mesh, center: [f32; 3], what: &str) {
    for face in 0..mesh.faces.len() {
        let out = subtract(mesh.face_center(face), center);
        assert!(dot(mesh.face_normal(face), out) > 0.0, "{what}'s face {face} faces in");
    }
}

#[test]
fn the_cube_is_6_quads_around_8_vertices() {
    let cube = primitives::cube(2.0);
    check_closed(&cube, 2, "the cube");
    check_faces_out_of(&cube, [0.0; 3], "the cube");
    assert_eq!((cube.vertex_count(), cube.face_count(), cube.edges().len()), (8, 6, 12));
    assert_eq!(sides_histogram(&cube), [0, 6]);
    assert_close(signed_volume(&cube), 8.0, 1.0e-5, "the cube's volume");
    assert!(cube.positions.iter().all(|position| position.iter().all(|p| p.abs() == 1.0)));
    assert_eq!(cube.material_count(), 1);
}

#[test]
fn the_plane_is_a_grid_of_quads_facing_up() {
    let plane = primitives::plane([4.0, 2.0], [2, 3]);
    assert_valid(&plane, "the plane");
    assert_eq!((plane.vertex_count(), plane.face_count()), (12, 6));
    assert_eq!(sides_histogram(&plane), [0, 6]);
    let topology = plane.topology();
    assert_eq!(topology.euler_characteristic(), 1);
    assert_eq!(topology.boundary_edges().len(), 2 * (2 + 3));
    for face in 0..plane.faces.len() {
        assert_eq!(plane.face_normal(face), [0.0, 1.0, 0.0]);
    }
    let area: f32 = (0..plane.faces.len()).map(|face| plane.face_area(face)).sum();
    assert_close(area, 8.0, 1.0e-5, "the plane's area");
    let bounds = plane.positions.iter().fold([f32::MAX, f32::MIN], |[low, high], position| {
        [low.min(position[0]), high.max(position[0])]
    });
    assert_eq!(bounds, [-2.0, 2.0]);
}

#[test]
fn cylinders_are_quads_between_two_caps() {
    for (sides, segments) in [(3, 1), (8, 2), (32, 5)] {
        let what = format!("a cylinder of {sides} sides and {segments} segments");
        let cylinder = primitives::cylinder(1.5, 3.0, sides, segments);
        check_closed(&cylinder, 2, &what);
        check_faces_out_of(&cylinder, [0.0; 3], &what);
        assert_eq!(cylinder.vertex_count() as u32, sides * (segments + 1), "{what}'s vertices");
        assert_eq!(cylinder.face_count() as u32, sides * segments + 2, "{what}'s faces");
        let quads = cylinder.faces.iter().filter(|face| face.vertices.len() == 4).count() as u32;
        assert_eq!(quads, sides * segments + if sides == 4 { 2 } else { 0 }, "{what}'s quads");
        let polygon_area = 0.5 * sides as f32 * 1.5 * 1.5 * (2.0 * PI / sides as f32).sin();
        assert_close(signed_volume(&cylinder), polygon_area * 3.0, 1.0e-3, &what);
    }
}

#[test]
fn cones_end_in_tips_or_caps() {
    let cone = primitives::cone(1.0, 0.0, 2.0, 8);
    check_closed(&cone, 2, "a cone");
    check_faces_out_of(&cone, [0.0; 3], "a cone");
    assert_eq!((cone.vertex_count(), cone.face_count()), (9, 9));
    assert_eq!(sides_histogram(&cone), [8, 0, 0, 0, 0, 1]);
    let base_area = 0.5 * 8.0 * (2.0 * PI / 8.0).sin();
    assert_close(signed_volume(&cone), base_area * 2.0 / 3.0, 1.0e-4, "a cone's volume");

    let upside_down = primitives::cone(0.0, 1.0, 2.0, 8);
    check_closed(&upside_down, 2, "an upside-down cone");
    check_faces_out_of(&upside_down, [0.0; 3], "an upside-down cone");
    assert_close(signed_volume(&upside_down), base_area * 2.0 / 3.0, 1.0e-4, "an upside-down cone's volume");

    let truncated = primitives::cone(1.0, 0.5, 2.0, 6);
    check_closed(&truncated, 2, "a truncated cone");
    check_faces_out_of(&truncated, [0.0; 3], "a truncated cone");
    assert_eq!((truncated.vertex_count(), truncated.face_count()), (12, 8));
    assert_eq!(sides_histogram(&truncated), [0, 6, 0, 2]);
}

#[test]
fn uv_spheres_are_quads_with_triangles_at_the_poles() {
    for (segments, rings) in [(3, 2), (8, 4), (32, 16)] {
        let what = format!("a UV sphere of {segments} by {rings}");
        let sphere = primitives::uv_sphere(2.0, segments, rings);
        check_closed(&sphere, 2, &what);
        check_faces_out_of(&sphere, [0.0; 3], &what);
        assert_eq!(sphere.vertex_count() as u32, 2 + segments * (rings - 1), "{what}'s vertices");
        assert_eq!(sphere.face_count() as u32, segments * rings, "{what}'s faces");
        let triangles = sphere.faces.iter().filter(|face| face.vertices.len() == 3).count() as u32;
        assert_eq!(triangles, 2 * segments, "{what}'s triangles");
        for position in &sphere.positions {
            assert_close(dot(*position, *position).sqrt(), 2.0, 1.0e-5, &what);
        }
    }
    let fine = primitives::uv_sphere(1.0, 64, 32);
    assert_close(signed_volume(&fine), 4.0 / 3.0 * PI, 0.03, "a fine sphere's volume");
}

#[test]
fn tori_are_all_quads() {
    for (major, minor) in [(3, 3), (12, 6), (48, 12)] {
        let what = format!("a torus of {major} by {minor}");
        let torus = primitives::torus(1.0, 0.25, major, minor);
        check_closed(&torus, 0, &what);
        assert_eq!(torus.vertex_count() as u32, major * minor, "{what}'s vertices");
        assert_eq!(torus.face_count() as u32, major * minor, "{what}'s faces");
        assert_eq!(sides_histogram(&torus), [0, (major * minor) as usize]);
        // Out of the tube: away from the circle through its middle, which a
        // torus of 3 segments around is too coarse to have its faces near.
        for face in (0..torus.faces.len()).filter(|_| major >= 12) {
            let center = torus.face_center(face);
            let around = (center[0] * center[0] + center[2] * center[2]).sqrt();
            let middle = [center[0] / around, 0.0, center[2] / around];
            assert!(dot(torus.face_normal(face), subtract(center, middle)) > 0.0, "{what}'s face {face} faces in");
        }
    }
    let fine = primitives::torus(1.0, 0.25, 96, 48);
    assert_close(signed_volume(&fine), 2.0 * PI * PI * 0.25 * 0.25, 0.01, "a fine torus's volume");
}

#[test]
#[should_panic(expected = "a cylinder needs at least 3 sides")]
fn a_cylinder_needs_3_sides() {
    primitives::cylinder(1.0, 1.0, 2, 1);
}
