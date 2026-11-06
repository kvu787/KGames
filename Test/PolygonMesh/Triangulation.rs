// Library/PolygonMesh's triangles for SimplePaint: every primitive at levels 0
// to 3 of subdivision passes SimplePaint's checks with flat and smooth normals,
// smooth normals split where they should, a smooth normal that would fail
// falls back to its triangle's flat one, concave faces are cut inside
// themselves, and faces without area are left out.

#[path = "Common.rs"]
#[allow(dead_code)]
mod common;

use common::{assert_close, assert_near};
use math::{Matrix, cross, dot, length, normalize, subtract};
use polygon_mesh::{Mesh, Shading, Triangles, primitives};
use simple_paint::validate_mesh;
use std::f32::consts::PI;

const SHADINGS: [Shading; 3] = [Shading::Flat, Shading::Smooth { angle: PI / 6.0 }, Shading::Smooth { angle: PI }];

// Triangulates the mesh, and checks that SimplePaint takes the triangles, that
// each face with area is drawn in n - 2 of them, of its material, wound like
// it.
fn check(mesh: &Mesh, shading: Shading, what: &str) -> Triangles {
    let triangles = mesh.triangulate(shading);
    if let Err(error) = validate_mesh(&triangles.vertices, &triangles.indices, mesh.material_count()) {
        panic!("{what}, {shading:?}: {error}");
    }
    assert_eq!(triangles.indices.len(), 3 * triangles.faces.len(), "{what}");
    let expected: usize = mesh.faces.iter().map(|face| face.vertices.len() - 2).sum();
    assert_eq!(triangles.faces.len(), expected, "{what}'s triangles");
    for (triangle, &face) in triangles.indices.chunks(3).zip(&triangles.faces) {
        let [a, b, c] = [0, 1, 2].map(|corner| triangles.vertices[triangle[corner] as usize]);
        let normal = cross(subtract(b.position, a.position), subtract(c.position, a.position));
        assert!(dot(normal, mesh.face_normal(face)) > 0.0, "{what}: a triangle of face {face} faces the other way");
        assert!([a, b, c].iter().all(|vertex| vertex.material_index == mesh.faces[face].material));
    }
    triangles
}

#[test]
fn every_primitive_passes_simple_paint_s_checks_at_levels_0_to_3() {
    let mut cylinder = primitives::cylinder(1.0, 2.0, 8, 2);
    cylinder.assign_material([16], 1);
    let mut creased = primitives::cube(2.0);
    creased.crease_edges(creased.topology().edges()[..6].to_vec(), 1.5);
    let meshes = [
        (primitives::cube(2.0), "a cube"),
        (creased, "a creased cube"),
        (primitives::plane([2.0, 3.0], [3, 2]), "a plane"),
        (cylinder, "a cylinder"),
        (primitives::cone(1.0, 0.0, 2.0, 8), "a cone"),
        (primitives::cone(1.0, 0.5, 2.0, 6), "a truncated cone"),
        (primitives::cone(1.0, 0.0, 0.1, 3), "a flat pyramid"),
        (primitives::uv_sphere(1.0, 8, 4), "a UV sphere"),
        (primitives::uv_sphere(1.0, 3, 4), "a UV sphere of 3 segments"),
        (primitives::torus(1.0, 0.25, 12, 6), "a torus"),
    ];
    for (mesh, what) in meshes {
        for level in 0..=3 {
            let surface = mesh.catmull_clark(level).mesh;
            for shading in SHADINGS {
                check(&surface, shading, &format!("{what} at level {level}"));
            }
        }
    }
}

#[test]
fn flat_normals_are_the_faces() {
    let cube = primitives::cube(2.0);
    let triangles = check(&cube, Shading::Flat, "a flat cube");
    assert_eq!((triangles.vertices.len(), triangles.indices.len()), (24, 36));
    assert_eq!(triangles.faces, [0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5]);
    for (triangle, &face) in triangles.indices.chunks(3).zip(&triangles.faces) {
        for &index in triangle {
            assert_eq!(triangles.vertices[index as usize].normal, cube.face_normal(face));
        }
    }
}

#[test]
fn smooth_normals_split_at_sharp_edges_creases_and_materials() {
    // Every edge of a cube is 90 degrees, more than 30.
    let cube = primitives::cube(2.0);
    let split = check(&cube, Shading::Smooth { angle: PI / 6.0 }, "a cube smooth to 30 degrees");
    assert_eq!(split, cube.triangulate(Shading::Flat));
    // But not more than 180, so each corner has one normal, out of it.
    let round = check(&cube, Shading::Smooth { angle: PI }, "a smooth cube");
    assert_eq!(round.vertices.len(), 8);
    for vertex in &round.vertices {
        assert_near(vertex.normal, normalize(vertex.position), 1.0e-6, "a smooth cube's normal");
    }
    // A crease of any sharpness splits them.
    let mut creased = primitives::cube(2.0);
    let edges = creased.edges();
    creased.crease_edges(edges, 0.25);
    assert_eq!(creased.triangulate(Shading::Smooth { angle: PI }), cube.triangulate(Shading::Flat));
    // So does a material, which only changes which vertices there are.
    let mut painted = primitives::cube(2.0);
    painted.assign_material([3], 1);
    let two = check(&painted, Shading::Smooth { angle: PI }, "a cube of two materials");
    assert_eq!(two.vertices.len(), 12);
    // A cylinder's caps are 90 degrees from its sides, which are 45 from
    // each other.
    let cylinder = primitives::cylinder(1.0, 2.0, 8, 1);
    let triangles = check(&cylinder, Shading::Smooth { angle: PI / 3.0 }, "a cylinder");
    assert_eq!(triangles.vertices.len(), 32);
    // A sphere's normals point out of it.
    let sphere = primitives::uv_sphere(1.0, 16, 8);
    let triangles = check(&sphere, Shading::Smooth { angle: PI / 3.0 }, "a sphere");
    assert_eq!(triangles.vertices.len(), sphere.vertex_count());
    for vertex in &triangles.vertices {
        assert!(dot(vertex.normal, vertex.position) > 0.98, "a sphere's normal at {:?}", vertex.position);
        assert_close(length(vertex.normal), 1.0, 1.0e-6, "a normal's length");
    }
}

#[test]
fn smooth_normals_too_far_apart_fall_back_to_flat() {
    // Around a sphere of 3 segments, the normals at a quad's corners near the
    // equator are 120 degrees apart, too far for SimplePaint.
    let sphere = primitives::uv_sphere(1.0, 3, 4);
    let triangles = check(&sphere, Shading::Smooth { angle: PI }, "a sphere of 3 segments");
    assert!(triangles.vertices.len() > sphere.vertex_count(), "no triangle fell back to flat");
    let mut flat = 0;
    for triangle in triangles.indices.chunks(3) {
        let [a, b, c] = [0, 1, 2].map(|corner| triangles.vertices[triangle[corner] as usize]);
        if a.normal == b.normal && b.normal == c.normal {
            let normal = normalize(cross(subtract(b.position, a.position), subtract(c.position, a.position)));
            assert_near(a.normal, normal, 1.0e-6, "a fallen-back normal");
            flat += 1;
        }
    }
    assert!(flat > 0);
}

// The polygon's points as one face, turned out of the XZ plane.
fn polygon(points: &[[f32; 2]]) -> Mesh {
    let turn = Matrix::rotation_x(0.5) * Matrix::rotation_z(0.3);
    let mut mesh = Mesh::new();
    for &[x, z] in points {
        mesh.add_vertex(turn.transform_point([x, 0.0, z]));
    }
    mesh.add_face((0..points.len() as u32).collect(), 0);
    mesh
}

// The polygon's area, by the shoelace formula.
fn shoelace(points: &[[f32; 2]]) -> f32 {
    let count = points.len();
    let twice: f32 = (0..count)
        .map(|index| {
            let ([x0, y0], [x1, y1]) = (points[index], points[(index + 1) % count]);
            x0 * y1 - x1 * y0
        })
        .sum();
    twice.abs() * 0.5
}

#[test]
fn concave_faces_are_cut_inside_themselves() {
    let l_shape = [[0.0, 0.0], [2.0, 0.0], [2.0, 1.0], [1.0, 1.0], [1.0, 2.0], [0.0, 2.0]];
    let star: Vec<[f32; 2]> = (0..10)
        .map(|point| {
            let angle = std::f32::consts::TAU * point as f32 / 10.0;
            let radius = if point % 2 == 0 { 2.0 } else { 0.6 };
            [radius * angle.cos(), radius * angle.sin()]
        })
        .collect();
    let comb = [
        [0.0, 0.0],
        [5.0, 0.0],
        [5.0, 3.0],
        [4.0, 3.0],
        [4.0, 1.0],
        [3.0, 1.0],
        [3.0, 3.0],
        [2.0, 3.0],
        [2.0, 1.0],
        [1.0, 1.0],
        [1.0, 3.0],
        [0.0, 3.0],
    ];
    // A square with a vertex in the middle of each side, like the faces next
    // to subdivided ones.
    let sides = [[0.0, 0.0], [1.0, 0.0], [2.0, 0.0], [2.0, 1.0], [2.0, 2.0], [1.0, 2.0], [0.0, 2.0], [0.0, 1.0]];
    let polygons = [(&l_shape[..], "an L"), (&star, "a star"), (&comb, "a comb"), (&sides, "a square with midpoints")];
    for (points, what) in polygons {
        let mesh = polygon(points);
        let triangles = mesh.face_triangles(0);
        assert_eq!(triangles.len(), points.len() - 2, "{what}");
        let face = &mesh.faces[0].vertices;
        let normal = mesh.face_normal(0);
        let mut area = 0.0;
        for triangle in &triangles {
            let [a, b, c] = triangle.map(|place| mesh.position(face[place]));
            let twice = cross(subtract(b, a), subtract(c, a));
            assert!(dot(twice, normal) > 1.0e-6, "{what} has a triangle that's backward or without area");
            area += length(twice) * 0.5;
        }
        assert_close(area, shoelace(points), 1.0e-4, &format!("{what}'s area"));
        assert_close(mesh.face_area(0), area, 1.0e-4, &format!("{what}'s face area"));
        for shading in SHADINGS {
            check(&mesh, shading, what);
        }
    }
}

#[test]
fn faces_without_area_are_left_out() {
    // An extrusion that hasn't moved has walls without area.
    let mut cube = primitives::cube(2.0);
    cube.extrude_faces([3]);
    for shading in SHADINGS {
        let triangles = cube.triangulate(shading);
        validate_mesh(&triangles.vertices, &triangles.indices, 1).unwrap();
        assert_eq!(triangles.faces.len(), 12);
        assert!(triangles.faces.iter().all(|&face| face < 6));
    }
}
