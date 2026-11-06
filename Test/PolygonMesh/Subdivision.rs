// Library/PolygonMesh's Catmull-Clark subdivision against known results: the
// counts at each level, a cube's first level against the textbook values,
// infinitely sharp and semi-sharp creases, fixed boundary corners, n-gons and
// triangles becoming quads, and meshes that aren't manifolds.

#[path = "Common.rs"]
#[allow(dead_code)]
mod common;

use common::{assert_close, assert_near, assert_valid, signed_volume, sides_histogram};
use polygon_mesh::{Edge, INFINITELY_SHARP, Mesh, primitives};

fn counts(mesh: &Mesh) -> (usize, usize) {
    (mesh.vertex_count(), mesh.face_count())
}

fn all_quads(mesh: &Mesh) -> bool {
    mesh.faces.iter().all(|face| face.vertices.len() == 4)
}

fn creased_cube(sharpness: f32) -> Mesh {
    let mut cube = primitives::cube(2.0);
    let edges = cube.edges();
    cube.crease_edges(edges, sharpness);
    cube
}

#[test]
fn a_cube_s_levels_have_the_known_counts() {
    let cube = primitives::cube(2.0);
    assert_eq!(cube.catmull_clark(0).mesh, cube);
    for (level, faces) in [(1, 24), (2, 96), (3, 384)] {
        let surface = cube.catmull_clark(level);
        assert_eq!(counts(&surface.mesh), (faces + 2, faces), "level {level}");
        assert!(all_quads(&surface.mesh));
        assert_valid(&surface.mesh, "a subdivided cube");
        let topology = surface.mesh.topology();
        assert!(topology.is_closed());
        assert_eq!(topology.euler_characteristic(), 2);
        assert!(signed_volume(&surface.mesh) > 0.0);
        // Each base face's children are together, in order.
        let per_face = faces / 6;
        assert!(surface.base_faces.iter().enumerate().all(|(face, &base)| base == face / per_face));
    }
}

#[test]
fn a_cube_s_first_level_has_the_textbook_points() {
    let cube = primitives::cube(2.0);
    let surface = cube.catmull_clark(1).mesh;
    let topology = cube.topology();
    // Vertices: the average of the 3 face points, twice the average of the 3
    // edges' midpoints, and none of the vertex, over 3: 5/9 of the way out.
    for vertex in 0..8 {
        let expected = cube.position(vertex).map(|p| p * 5.0 / 9.0);
        assert_near(surface.position(vertex), expected, 1.0e-6, "a vertex point");
    }
    // Edges: the average of the ends and the two faces' points, 3/4 out on
    // the two axes that the edge is at the side of.
    for (index, edge) in topology.edges().iter().enumerate() {
        let [a, b] = edge.vertices().map(|vertex| cube.position(vertex));
        let expected = std::array::from_fn(|axis| if a[axis] == b[axis] { 0.75 * a[axis] } else { 0.0 });
        assert_near(surface.position(8 + index as u32), expected, 1.0e-6, "an edge point");
    }
    // Faces: their centers.
    for face in 0..6 {
        assert_near(surface.position(20 + face as u32), cube.face_center(face), 1.0e-6, "a face point");
    }
}

#[test]
fn an_infinitely_sharp_cube_stays_a_cube() {
    let cube = creased_cube(INFINITELY_SHARP);
    for level in 1..=3 {
        let surface = cube.catmull_clark(level).mesh;
        for vertex in 0..8 {
            assert_eq!(surface.position(vertex), cube.position(vertex), "a corner at level {level}");
        }
        for position in &surface.positions {
            let outermost = position.iter().fold(0.0f32, |most, p| most.max(p.abs()));
            assert_close(outermost, 1.0, 1.0e-6, "a vertex on the cube's surface");
        }
        assert_close(signed_volume(&surface), 8.0, 1.0e-4, "the sharp cube's volume");
        // The creases stay infinitely sharp, on both halves of each edge.
        assert_eq!(surface.creases.len(), 12 << level);
        assert!(surface.creases.values().all(|&sharpness| sharpness == INFINITELY_SHARP));
    }
}

#[test]
fn semi_sharp_creases_lose_1_each_level_and_blend() {
    // Sharpness 1 is a crease for the first level, and smooth after, so its
    // first level is exactly the infinitely sharp one's, and its second
    // isn't.
    let smooth = primitives::cube(2.0);
    let semi = creased_cube(1.0);
    let sharp = creased_cube(INFINITELY_SHARP);
    let [semi_first, sharp_first] = [&semi, &sharp].map(|mesh| mesh.catmull_clark(1).mesh.positions);
    for (a, b) in semi_first.into_iter().zip(sharp_first) {
        assert_near(a, b, 1.0e-6, "a point of the first level");
    }
    let corner = |mesh: &Mesh, level: u32| mesh.catmull_clark(level).mesh.position(6);
    let [smooth, semi, sharp] = [&smooth, &semi, &sharp].map(|mesh| corner(mesh, 2));
    assert_eq!(sharp, [1.0, 1.0, 1.0]);
    for [a, b] in [[smooth, semi], [semi, sharp], [smooth, sharp]] {
        assert!(math::length(math::subtract(a, b)) > 0.05, "{a:?} and {b:?} are too close");
    }
    assert!(semi[0] > smooth[0] && semi[0] < sharp[0]);

    // Sharpness 0.5 is halfway between a crease and smooth: the corner is
    // halfway between where it is and 5/9, and the edges between their
    // midpoints and 3/4.
    let half = creased_cube(0.5).catmull_clark(1).mesh;
    assert_near(half.position(6), [7.0 / 9.0; 3], 1.0e-6, "a half-sharp corner");
    let edge = primitives::cube(2.0).topology().edge_index(Edge::new(2, 6)).unwrap() as u32;
    assert_near(half.position(8 + edge), [0.875, 0.875, 0.0], 1.0e-6, "a half-sharp edge");

    // The crease's sharpness goes down a level at a time, on both halves.
    let mut cube = primitives::cube(2.0);
    cube.set_crease(Edge::new(2, 6), 2.5);
    let levels: Vec<Vec<f32>> =
        (1..=3).map(|level| cube.catmull_clark(level).mesh.creases.values().copied().collect()).collect();
    assert_eq!(levels, [vec![1.5; 2], vec![0.5; 4], vec![]]);
}

#[test]
fn darts_creases_and_corners() {
    // One sharp edge at a vertex is a dart, which moves like a smooth vertex,
    // though the edge's point is its midpoint.
    let mut cube = primitives::cube(2.0);
    cube.set_crease(Edge::new(2, 6), INFINITELY_SHARP);
    let edge = cube.topology().edge_index(Edge::new(2, 6)).unwrap() as u32;
    let surface = cube.catmull_clark(1).mesh;
    assert_near(surface.position(6), [5.0 / 9.0; 3], 1.0e-6, "a dart");
    assert_near(surface.position(8 + edge), [1.0, 1.0, 0.0], 1.0e-6, "a sharp edge's point");
    // Two make a crease, which moves along them: 6/8 of the vertex and 1/8 of
    // each of their other ends.
    cube.set_crease(Edge::new(6, 7), INFINITELY_SHARP);
    let surface = cube.catmull_clark(1).mesh;
    assert_near(surface.position(6), [0.75, 1.0, 0.75], 1.0e-6, "a crease");
    // Three make a corner, which stays.
    cube.set_crease(Edge::new(5, 6), INFINITELY_SHARP);
    assert_eq!(cube.catmull_clark(1).mesh.position(6), [1.0, 1.0, 1.0]);

    // Sharpness 2 is a crease for two levels, the second fading with weight
    // 1, so it matches the infinitely sharp cube for two levels and not the
    // third.
    let twice = creased_cube(2.0);
    let sharp = creased_cube(INFINITELY_SHARP);
    for level in 1..=2 {
        let [a, b] = [&twice, &sharp].map(|mesh| mesh.catmull_clark(level).mesh.positions);
        for (a, b) in a.into_iter().zip(b) {
            assert_near(a, b, 1.0e-6, "a point of sharpness 2");
        }
    }
    let corner = twice.catmull_clark(3).mesh.position(6);
    assert!(corner[0] < 1.0 - 1.0e-3, "sharpness 2 is smooth by the third level");
}

#[test]
fn a_plane_s_boundary_corners_stay_put() {
    let plane = primitives::plane([2.0, 2.0], [2, 2]);
    for level in 1..=3 {
        let surface = plane.catmull_clark(level).mesh;
        assert_valid(&surface, "a subdivided plane");
        for corner in [0, 2, 6, 8] {
            assert_eq!(surface.position(corner), plane.position(corner), "a corner at level {level}");
        }
        assert_eq!(surface.position(1), [0.0, 0.0, -1.0], "the middle of an edge at level {level}");
        assert!(surface.positions.iter().all(|position| position[1] == 0.0));
        // The boundary stays on the square's edges.
        let topology = surface.topology();
        for edge in topology.boundary_edges() {
            for vertex in edge.vertices() {
                let [x, _, z] = surface.position(vertex);
                assert_close(x.abs().max(z.abs()), 1.0, 1.0e-6, "a boundary vertex");
            }
        }
        assert_eq!(topology.boundary_edges().len(), 8 << level);
    }
    // A lone quad becomes a grid of the same square.
    let quad = primitives::plane([2.0, 2.0], [1, 1]).catmull_clark(1).mesh;
    assert_eq!(counts(&quad), (9, 4));
    let mut corners: Vec<[f32; 3]> = quad.positions.clone();
    corners.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mut grid = primitives::plane([2.0, 2.0], [2, 2]).positions;
    grid.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(corners, grid);
}

#[test]
fn n_gons_and_triangles_become_quads() {
    let cone = primitives::cone(1.0, 0.0, 2.0, 8);
    let surface = cone.catmull_clark(1);
    assert_eq!(counts(&surface.mesh), (9 + 16 + 9, 8 * 3 + 8));
    assert!(all_quads(&surface.mesh));
    assert_valid(&surface.mesh, "a subdivided cone");
    assert!(surface.mesh.topology().is_closed());
    assert_eq!(surface.base_faces.iter().filter(|&&base| base == 8).count(), 8);

    let tetrahedron = primitives::cone(1.0, 0.0, 1.0, 3);
    let surface = tetrahedron.catmull_clark(2).mesh;
    assert_eq!(counts(&surface), (12 * 4 + 2, 12 * 4));
    assert_eq!(sides_histogram(&surface), [0, 48]);
}

#[test]
fn children_keep_their_face_s_material() {
    let mut cylinder = primitives::cylinder(1.0, 2.0, 6, 2);
    cylinder.assign_material([3], 2);
    cylinder.assign_material([12, 13], 5);
    let surface = cylinder.catmull_clark(2);
    for (face, &base) in surface.mesh.faces.iter().zip(&surface.base_faces) {
        assert_eq!(face.material, cylinder.faces[base].material);
    }
    assert_eq!(surface.mesh.material_count(), 6);
    assert_eq!(surface.mesh.faces.iter().filter(|face| face.material == 5).count(), 2 * 6 * 4);
}

#[test]
fn closed_surfaces_stay_closed_and_wound_out() {
    for (mesh, euler, what) in [
        (primitives::torus(1.0, 0.25, 12, 6), 0, "a torus"),
        (primitives::uv_sphere(1.0, 8, 4), 2, "a sphere"),
        (primitives::cylinder(1.0, 2.0, 5, 1), 2, "a cylinder"),
    ] {
        let surface = mesh.catmull_clark(2).mesh;
        assert_valid(&surface, what);
        let topology = surface.topology();
        assert!(topology.is_closed(), "{what}");
        assert_eq!(topology.euler_characteristic(), euler, "{what}");
        assert!(signed_volume(&surface) > 0.0, "{what}");
        assert!(all_quads(&surface), "{what}");
    }
    // A sphere's subdivision shrinks inside it, toward a smooth limit.
    let sphere = primitives::uv_sphere(1.0, 16, 8).catmull_clark(3).mesh;
    assert!(sphere.positions.iter().all(|&position| math::length(position) < 1.0 + 1.0e-6));
    assert!(sphere.positions.iter().all(|&position| math::length(position) > 0.9));
}

#[test]
fn non_manifold_meshes_subdivide_without_panicking() {
    // Three quads on one edge, whose ends stay put, as corners.
    let mut mesh = Mesh::new();
    for position in [
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [1.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [1.0, -1.0, 0.0],
        [0.0, -1.0, 0.0],
    ] {
        mesh.add_vertex(position);
    }
    mesh.add_face(vec![0, 1, 2, 3], 0);
    mesh.add_face(vec![1, 0, 5, 4], 0);
    mesh.add_face(vec![1, 0, 7, 6], 0);
    // A loose vertex stays where it is.
    mesh.add_vertex([5.0, 5.0, 5.0]);
    let surface = mesh.catmull_clark(3).mesh;
    surface.validate().unwrap();
    assert_eq!(surface.face_count(), 3 * 64);
    assert_eq!(surface.position(0), [0.0, 0.0, 0.0]);
    assert_eq!(surface.position(1), [1.0, 0.0, 0.0]);
    assert_eq!(surface.position(8), [5.0, 5.0, 5.0]);
    // The spine is infinitely sharp, so its points stay on it.
    let topology = surface.topology();
    for edge in topology.non_manifold_edges() {
        for vertex in edge.vertices() {
            let [_, y, z] = surface.position(vertex);
            assert!(y == 0.0 && z == 0.0, "vertex {vertex} left the spine");
        }
    }
}
