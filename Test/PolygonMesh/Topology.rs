// Library/PolygonMesh's topology: edges and their faces, boundaries,
// valences, and edge loops and rings, on closed, open and non-manifold
// meshes.

use polygon_mesh::{Edge, Mesh, primitives};

fn edges(pairs: &[[u32; 2]]) -> Vec<Edge> {
    pairs.iter().map(|&[a, b]| Edge::new(a, b)).collect()
}

fn sorted(mut edges: Vec<Edge>) -> Vec<Edge> {
    edges.sort();
    edges
}

#[test]
fn edges_are_undirected_with_the_smaller_vertex_first() {
    let edge = Edge::new(7, 3);
    assert_eq!(edge, Edge::new(3, 7));
    assert_eq!(edge.vertices(), [3, 7]);
    assert_eq!((edge.other(3), edge.other(7)), (7, 3));
    assert!(edge.contains(7) && !edge.contains(5));
}

#[test]
fn a_cube_s_edges_each_have_two_faces_and_its_vertices_three() {
    let cube = primitives::cube(2.0);
    let topology = cube.topology();
    assert_eq!(topology.edges().len(), 12);
    for edge in 0..12 {
        assert_eq!(topology.edge_faces(edge).len(), 2);
        assert!(!topology.is_boundary(edge) && !topology.is_non_manifold(edge));
    }
    for vertex in 0..8 {
        assert_eq!(topology.valence(vertex), 3);
        assert_eq!(topology.vertex_faces(vertex).len(), 3);
        assert!(!topology.is_boundary_vertex(vertex));
    }
    // Side i of a face goes from its vertex i to vertex i + 1.
    for (face, sides) in cube.faces.iter().enumerate() {
        for (side, [a, b]) in sides.sides().enumerate() {
            assert_eq!(topology.edge(topology.face_edges(face)[side]), Edge::new(a, b));
        }
    }
    let top = Edge::new(2, 3);
    assert_eq!(topology.faces_of_edge(top), [0, 3]);
    assert_eq!(topology.faces_of_edge(Edge::new(0, 6)), [] as [usize; 0]);
    assert!(topology.is_closed() && topology.is_consistently_wound());
}

#[test]
fn a_plane_s_boundary_and_valences() {
    // Vertex (column, row) is row * 4 + column.
    let plane = primitives::plane([3.0, 3.0], [3, 3]);
    let topology = plane.topology();
    assert_eq!(topology.edges().len(), 24);
    assert_eq!(topology.boundary_edges().len(), 12);
    assert_eq!([0, 1, 5, 15].map(|vertex| topology.valence(vertex)), [2, 3, 4, 2]);
    assert_eq!([0, 1, 5, 15].map(|vertex| topology.is_boundary_vertex(vertex)), [true, true, false, true]);
    assert_eq!(topology.vertex_faces(5), [0, 1, 3, 4]);
    assert!(!topology.is_closed());
}

#[test]
fn edge_loops_cross_vertices_of_four_edges() {
    let plane = primitives::plane([3.0, 3.0], [3, 3]);
    let topology = plane.topology();
    assert_eq!(topology.edge_loop(Edge::new(5, 9)), edges(&[[1, 5], [5, 9], [9, 13]]));
    // Along the boundary, around the corners, back to the start.
    let boundary = topology.edge_loop(Edge::new(0, 1));
    assert_eq!(boundary.len(), 12);
    assert_eq!(sorted(boundary), sorted(topology.boundary_edges()));
    assert_eq!(topology.edge_loop(Edge::new(0, 5)), []);

    // Vertex (ring, side) is ring * 8 + side.
    let cylinder = primitives::cylinder(1.0, 2.0, 8, 3);
    let topology = cylinder.topology();
    let around = topology.edge_loop(Edge::new(8, 9));
    assert_eq!(sorted(around), sorted((0..8).map(|side| Edge::new(8 + side, 8 + (side + 1) % 8)).collect()));
    // Up the side, stopping at the caps, where three edges meet.
    assert_eq!(topology.edge_loop(Edge::new(8, 16)), edges(&[[0, 8], [8, 16], [16, 24]]));

    // Vertex (major, minor) is major * 6 + minor.
    let torus = primitives::torus(1.0, 0.25, 12, 6);
    let topology = torus.topology();
    assert_eq!(topology.edge_loop(Edge::new(0, 1)).len(), 6);
    assert_eq!(topology.edge_loop(Edge::new(0, 6)).len(), 12);
}

#[test]
fn edge_rings_cross_quads() {
    let plane = primitives::plane([3.0, 3.0], [3, 3]);
    let topology = plane.topology();
    assert_eq!(topology.edge_ring(Edge::new(5, 9)), edges(&[[7, 11], [6, 10], [5, 9], [4, 8]]));
    let ring = topology.ring(Edge::new(5, 9));
    assert_eq!((ring.faces, ring.closed), (vec![5, 4, 3], false));

    let cylinder = primitives::cylinder(1.0, 2.0, 8, 3);
    let topology = cylinder.topology();
    let ring = topology.ring(Edge::new(8, 16));
    assert!(ring.closed);
    assert_eq!((ring.edges.len(), ring.faces.len()), (8, 8));
    assert_eq!(topology.face_loop(Edge::new(8, 16)).len(), 8);
    // From one cap to the other, which aren't quads.
    assert_eq!(topology.edge_ring(Edge::new(8, 9)), edges(&[[24, 25], [16, 17], [8, 9], [0, 1]]));

    let torus = primitives::torus(1.0, 0.25, 12, 6);
    let topology = torus.topology();
    assert_eq!(topology.edge_ring(Edge::new(0, 1)).len(), 12);
    assert_eq!(topology.edge_ring(Edge::new(0, 6)).len(), 6);

    // Between the triangles at the poles.
    let sphere = primitives::uv_sphere(1.0, 8, 4);
    let topology = sphere.topology();
    assert_eq!(topology.edge_ring(Edge::new(1, 2)), edges(&[[17, 18], [9, 10], [1, 2]]));
    assert_eq!(topology.edge_ring(Edge::new(0, 1)), edges(&[[0, 1]]));
}

// Three quads on one edge, like a page between two others.
fn three_pages() -> Mesh {
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
    mesh
}

#[test]
fn non_manifold_meshes_are_described_without_panicking() {
    let mesh = three_pages();
    mesh.validate().unwrap();
    let topology = mesh.topology();
    let spine = topology.edge_index(Edge::new(0, 1)).unwrap();
    assert_eq!(topology.edge_faces(spine), [0, 1, 2]);
    assert!(topology.is_non_manifold(spine) && !topology.is_boundary(spine));
    assert_eq!(topology.non_manifold_edges(), [Edge::new(0, 1)]);
    assert_eq!(topology.boundary_edges().len(), 9);
    assert_eq!(topology.edge_ring(Edge::new(0, 1)), [Edge::new(0, 1)]);
    assert_eq!(topology.edge_loop(Edge::new(0, 1)), [Edge::new(0, 1)]);
    assert_eq!(topology.edge_loop(Edge::new(2, 3)), edges(&[[1, 2], [2, 3], [0, 3]]));
    assert_eq!(topology.edge_ring(Edge::new(1, 2)), edges(&[[1, 2], [0, 3]]));
    assert_eq!(topology.valence(0), 4);
    assert_eq!(topology.euler_characteristic(), 8 - 10 + 3);
}

#[test]
fn faces_that_go_the_same_way_along_an_edge_are_inconsistent() {
    let mut mesh = Mesh::new();
    for position in [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, -1.0, 0.0]] {
        mesh.add_vertex(position);
    }
    mesh.add_face(vec![0, 1, 2], 0);
    mesh.add_face(vec![0, 1, 3], 0);
    let topology = mesh.topology();
    assert!(!topology.is_consistently_wound());
    assert_eq!(topology.non_manifold_edges(), [Edge::new(0, 1)]);
    mesh.faces[1].flip();
    assert_eq!(mesh.faces[1].vertices, [0, 3, 1]);
    assert!(mesh.topology().is_consistently_wound());
}

#[test]
fn validation_says_what_s_wrong() {
    let mut mesh = primitives::cube(1.0);
    mesh.validate().unwrap();
    mesh.faces[2].vertices[1] = 9;
    assert_eq!(mesh.validate().unwrap_err(), "Face 2 uses vertex 9, but there are only 8 vertices.");
    mesh.faces[2].vertices = vec![0, 1, 0, 4];
    assert_eq!(mesh.validate().unwrap_err(), "Face 2 uses a vertex more than once.");
    mesh.faces[2].vertices = vec![0, 1];
    assert_eq!(mesh.validate().unwrap_err(), "Face 2 has 2 vertices, but a face needs at least 3.");
    let mut mesh = primitives::cube(1.0);
    mesh.positions[3][1] = f32::NAN;
    assert_eq!(mesh.validate().unwrap_err(), "Vertex 3's position isn't finite.");
    let mut mesh = primitives::cube(1.0);
    mesh.creases.insert(Edge::new(0, 1), 11.0);
    assert!(mesh.validate().unwrap_err().contains("has sharpness 11"));
    mesh.set_crease(Edge::new(0, 1), 11.0);
    assert_eq!(mesh.crease(Edge::new(0, 1)), 10.0);
    mesh.validate().unwrap();
    mesh.set_crease(Edge::new(0, 1), 0.0);
    assert!(mesh.creases.is_empty());
}
