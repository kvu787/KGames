// What's selected, in all three of a modeler's modes at once, as Blender
// keeps it: selecting vertices selects the edges whose ends are both
// selected and the faces whose vertices all are, selecting edges selects
// their vertices and the faces whose edges all are, and selecting faces
// selects their edges and vertices. The constructors make selections that
// agree that way, and switching modes keeps the selection of the new mode's
// elements, like Blender.

use crate::{Edge, Mesh, Topology, mesh::average};
use math::Vector3;
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SelectMode {
    Vertices,
    Edges,
    Faces,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Selection {
    pub vertices: BTreeSet<u32>,
    pub edges: BTreeSet<Edge>,
    pub faces: BTreeSet<usize>,
}

impl Selection {
    pub fn new() -> Selection {
        Selection::default()
    }

    // The vertices, the edges between two of them, and the faces with all of
    // their vertices among them. Vertices that aren't in the mesh are left
    // out.
    pub fn from_vertices(mesh: &Mesh, vertices: impl IntoIterator<Item = u32>) -> Selection {
        let count = mesh.positions.len();
        let vertices: BTreeSet<u32> = vertices.into_iter().filter(|&vertex| (vertex as usize) < count).collect();
        let mut selection = Selection { vertices, ..Selection::default() };
        for (index, face) in mesh.faces.iter().enumerate() {
            for edge in face.edges() {
                let [a, b] = edge.vertices();
                if selection.vertices.contains(&a) && selection.vertices.contains(&b) {
                    selection.edges.insert(edge);
                }
            }
            if face.vertices.iter().all(|vertex| selection.vertices.contains(vertex)) {
                selection.faces.insert(index);
            }
        }
        selection
    }

    // The edges, their vertices, and the faces with all of their edges among
    // them. Edges that aren't in the mesh are left out.
    pub fn from_edges(mesh: &Mesh, edges: impl IntoIterator<Item = Edge>) -> Selection {
        let existing = mesh.edges();
        let edges: BTreeSet<Edge> = edges.into_iter().filter(|edge| existing.contains(edge)).collect();
        let vertices = edges.iter().flat_map(|edge| edge.vertices()).collect();
        let faces = (0..mesh.faces.len())
            .filter(|&face| mesh.faces[face].edges().all(|edge| edges.contains(&edge)))
            .collect();
        Selection { vertices, edges, faces }
    }

    // The faces, and their edges and vertices. Faces that aren't in the mesh
    // are left out.
    pub fn from_faces(mesh: &Mesh, faces: impl IntoIterator<Item = usize>) -> Selection {
        let faces: BTreeSet<usize> = faces.into_iter().filter(|&face| face < mesh.faces.len()).collect();
        let mut selection = Selection { faces, ..Selection::default() };
        for &face in &selection.faces {
            let face = &mesh.faces[face];
            selection.vertices.extend(&face.vertices);
            selection.edges.extend(face.edges());
        }
        selection
    }

    // Everything, loose vertices too.
    pub fn all(mesh: &Mesh) -> Selection {
        Selection::from_vertices(mesh, 0..mesh.positions.len() as u32)
    }

    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty() && self.edges.is_empty() && self.faces.is_empty()
    }

    // The selection that switching to `mode` leaves, like Blender: what's
    // selected of the mode's own elements, and what that selects.
    pub fn in_mode(&self, mesh: &Mesh, mode: SelectMode) -> Selection {
        match mode {
            SelectMode::Vertices => Selection::from_vertices(mesh, self.vertices.iter().copied()),
            SelectMode::Edges => Selection::from_edges(mesh, self.edges.iter().copied()),
            SelectMode::Faces => Selection::from_faces(mesh, self.faces.iter().copied()),
        }
    }

    // The mode's elements that aren't selected, and what they select.
    pub fn inverted(&self, mesh: &Mesh, mode: SelectMode) -> Selection {
        match mode {
            SelectMode::Vertices => Selection::from_vertices(
                mesh,
                (0..mesh.positions.len() as u32).filter(|vertex| !self.vertices.contains(vertex)),
            ),
            SelectMode::Edges => {
                Selection::from_edges(mesh, mesh.edges().into_iter().filter(|edge| !self.edges.contains(edge)))
            }
            SelectMode::Faces => {
                Selection::from_faces(mesh, (0..mesh.faces.len()).filter(|face| !self.faces.contains(face)))
            }
        }
    }

    // Both selections' elements, each selection in `mode`, as when adding to
    // a selection with Shift.
    pub fn union(&self, other: &Selection, mesh: &Mesh, mode: SelectMode) -> Selection {
        let mut both = self.clone();
        both.vertices.extend(&other.vertices);
        both.edges.extend(&other.edges);
        both.faces.extend(&other.faces);
        both.in_mode(mesh, mode)
    }

    // The edge loop through an edge, as Alt+click selects it in Blender's
    // vertex and edge modes.
    pub fn edge_loop(mesh: &Mesh, edge: Edge) -> Selection {
        Selection::from_edges(mesh, Topology::new(mesh).edge_loop(edge))
    }

    // The edge ring through an edge, as Ctrl+Alt+click selects it.
    pub fn edge_ring(mesh: &Mesh, edge: Edge) -> Selection {
        Selection::from_edges(mesh, Topology::new(mesh).edge_ring(edge))
    }

    // The quads across the edge ring through an edge, as Alt+click selects
    // them in face mode.
    pub fn face_loop(mesh: &Mesh, edge: Edge) -> Selection {
        Selection::from_faces(mesh, Topology::new(mesh).face_loop(edge))
    }

    // The average of the selected vertices' positions, the median point that
    // Blender rotates and scales about, or None if none are selected.
    pub fn center(&self, mesh: &Mesh) -> Option<Vector3> {
        average(self.vertices.iter().filter_map(|&vertex| mesh.positions.get(vertex as usize).copied()))
    }
}
