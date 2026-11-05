// Library/Meshes: every mesh passes SimplePaint's checks, which Mesh::new
// makes before it uploads one.

use meshes::{Mesh, car, cuboid, plane, uv_sphere};
use simple_paint::validate_mesh;

fn check(mesh: &Mesh, what: &str) {
    if let Err(error) = validate_mesh(&mesh.vertices, &mesh.indices, 1) {
        panic!("{what}: {error}");
    }
}

// The fewest longitudes, where the normals of a triangle are furthest apart,
// with every number of latitudes, and the others with fewer.
#[test]
fn spheres_pass_simple_paint_s_checks() {
    for latitudes in 2..=512 {
        check(&uv_sphere(4, latitudes, 0), &format!("a sphere of 4 by {latitudes}"));
    }
    for longitudes in 5..=16 {
        for latitudes in (2..=64).chain([512]) {
            check(&uv_sphere(longitudes, latitudes, 0), &format!("a sphere of {longitudes} by {latitudes}"));
        }
    }
    for [longitudes, latitudes] in [[64, 32], [512, 2], [512, 512]] {
        check(&uv_sphere(longitudes, latitudes, 0), &format!("a sphere of {longitudes} by {latitudes}"));
    }
}

#[test]
#[should_panic(expected = "a UV sphere needs 4 to 512 longitudes")]
fn a_sphere_needs_4_longitudes() {
    uv_sphere(3, 4, 0);
}

#[test]
fn the_car_passes_simple_paint_s_checks_with_each_part_painted() {
    assert_eq!((car::VERTICES.len(), car::INDICES.len(), car::MATERIAL_COUNT), (1938, 5652, 5));
    validate_mesh(&car::VERTICES, &car::INDICES, car::MATERIAL_COUNT).unwrap();
    for material in [car::AXLES, car::BODY, car::CABIN, car::HEADLIGHTS, car::WHEELS] {
        let painted = car::VERTICES.iter().any(|vertex| vertex.material_index == material);
        assert!(painted, "no vertex of material {material}");
    }
}

#[test]
fn boxes_and_planes_pass_simple_paint_s_checks() {
    check(&cuboid([1.0, 2.0, 3.0], 0), "a box");
    check(&plane([10.0, 4.0], 0), "a plane");
}
