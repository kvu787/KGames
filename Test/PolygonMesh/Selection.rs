// Library/PolygonMesh's selections: what selecting vertices, edges and faces
// selects of the others, switching modes, all, none, inverting, loops and
// rings, as in Blender.

use polygon_mesh::{Edge, SelectMode, Selection, primitives};
use std::collections::BTreeSet;

// The cube's faces are -Z, +Z, -Y, +Y, -X and +X. Its top, +Y, is 3, 7, 6, 2.
const TOP: usize = 3;
const FRONT: usize = 1;

fn set<T: Ord>(items: impl IntoIterator<Item = T>) -> BTreeSet<T> {
    items.into_iter().collect()
}

#[test]
fn vertices_select_the_edges_and_faces_between_them() {
    let cube = primitives::cube(2.0);
    let top = Selection::from_vertices(&cube, [3, 7, 6, 2]);
    assert_eq!(top.edges.len(), 4);
    assert_eq!(top.faces, set([TOP]));
    // Two edges, but no face, and no diagonal, which isn't an edge.
    let three = Selection::from_vertices(&cube, [3, 7, 6]);
    assert_eq!(three.edges, set([Edge::new(3, 7), Edge::new(6, 7)]));
    assert!(three.faces.is_empty());
    // Vertices that aren't in the mesh are left out.
    assert_eq!(Selection::from_vertices(&cube, [1, 99]).vertices, set([1]));
}

#[test]
fn edges_select_their_vertices_and_the_faces_they_go_around() {
    let cube = primitives::cube(2.0);
    let around = [Edge::new(3, 7), Edge::new(6, 7), Edge::new(2, 6), Edge::new(2, 3)];
    let top = Selection::from_edges(&cube, around);
    assert_eq!(top.vertices, set([2, 3, 6, 7]));
    assert_eq!(top.faces, set([TOP]));
    let three = Selection::from_edges(&cube, around[..3].iter().copied());
    assert!(three.faces.is_empty());
    assert_eq!(three.vertices, set([2, 3, 6, 7]));
    assert!(Selection::from_edges(&cube, [Edge::new(0, 6)]).is_empty());
}

#[test]
fn faces_select_their_edges_and_vertices() {
    let cube = primitives::cube(2.0);
    let two = Selection::from_faces(&cube, [TOP, FRONT, 17]);
    assert_eq!(two.faces, set([FRONT, TOP]));
    assert_eq!((two.vertices.len(), two.edges.len()), (6, 7));
}

#[test]
fn switching_modes_keeps_what_s_selected_of_the_new_mode_s_elements() {
    let cube = primitives::cube(2.0);
    // The top's vertices and one of the bottom's.
    let vertices = Selection::from_vertices(&cube, [3, 7, 6, 2, 0]);
    assert_eq!(vertices.faces, set([TOP]));
    assert_eq!(vertices.edges.len(), 5);
    let edges = vertices.in_mode(&cube, SelectMode::Edges);
    assert_eq!(edges.edges.len(), 5);
    assert_eq!(edges.faces, set([TOP]));
    let faces = edges.in_mode(&cube, SelectMode::Faces);
    assert_eq!(faces, Selection::from_faces(&cube, [TOP]));
    // Back to vertices from two faces: their six vertices, and what they
    // select, which is the same two faces.
    let two = Selection::from_faces(&cube, [TOP, FRONT]).in_mode(&cube, SelectMode::Vertices);
    assert_eq!(two.vertices.len(), 6);
    assert_eq!(two.faces, set([FRONT, TOP]));
    assert_eq!(two.in_mode(&cube, SelectMode::Vertices), two);
}

#[test]
fn all_none_and_inverting() {
    let cube = primitives::cube(2.0);
    let all = Selection::all(&cube);
    assert_eq!((all.vertices.len(), all.edges.len(), all.faces.len()), (8, 12, 6));
    assert!(Selection::new().is_empty());
    assert_eq!(all.inverted(&cube, SelectMode::Faces), Selection::new());
    assert_eq!(Selection::new().inverted(&cube, SelectMode::Edges), all);

    let top = Selection::from_faces(&cube, [TOP]);
    assert_eq!(top.inverted(&cube, SelectMode::Faces).faces, set([0, 1, 2, 4, 5]));
    // The other vertices are the bottom's, which select the bottom face.
    let bottom = top.inverted(&cube, SelectMode::Vertices);
    assert_eq!(bottom.vertices, set([0, 1, 4, 5]));
    assert_eq!(bottom.faces, set([2]));
    // The other edges are the 8 not around the top, which go all the way
    // around the bottom.
    let others = top.inverted(&cube, SelectMode::Edges);
    assert_eq!(others.edges.len(), 8);
    assert_eq!(others.faces, set([2]));

    let both = top.union(&bottom, &cube, SelectMode::Faces);
    assert_eq!(both.faces, set([2, TOP]));
}

#[test]
fn loops_and_rings() {
    // Vertex (ring, side) is ring * 8 + side.
    let cylinder = primitives::cylinder(1.0, 2.0, 8, 3);
    let around = Selection::edge_loop(&cylinder, Edge::new(8, 9));
    assert_eq!((around.vertices.len(), around.edges.len(), around.faces.len()), (8, 8, 0));
    let ring = Selection::edge_ring(&cylinder, Edge::new(8, 16));
    assert_eq!((ring.vertices.len(), ring.edges.len(), ring.faces.len()), (16, 8, 0));
    let faces = Selection::face_loop(&cylinder, Edge::new(8, 16));
    assert_eq!(faces.faces, set(8..16));
    assert_eq!(faces.vertices, set(8..24));
}

#[test]
fn a_selection_s_center_is_its_vertices_average() {
    let cube = primitives::cube(2.0);
    assert_eq!(Selection::from_faces(&cube, [TOP]).center(&cube), Some([0.0, 1.0, 0.0]));
    assert_eq!(Selection::all(&cube).center(&cube), Some([0.0, 0.0, 0.0]));
    assert_eq!(Selection::new().center(&cube), None);
}
