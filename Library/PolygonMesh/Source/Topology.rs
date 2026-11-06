// How a mesh's vertices, edges and faces connect, computed from its faces on
// demand, since a modeler's edits change it all the time. It owns its data,
// so a program can keep it beside the mesh while the mesh doesn't change.
// Edges are numbered in Edge's order, which doesn't depend on the faces'.
// Nothing assumes a manifold: an edge can have any number of faces, and a
// vertex any fan of them.

use crate::{Edge, Mesh};
use std::collections::{BTreeSet, HashMap};

#[derive(Clone, Debug, Default)]
pub struct Topology {
    edges: Vec<Edge>,
    indices: HashMap<Edge, usize>,
    edge_faces: Vec<Vec<usize>>,
    // Whether no two of an edge's faces go along it the same way, which two
    // faces of a consistently wound surface never do.
    edge_consistent: Vec<bool>,
    // For each face, the edge of each side, from vertex i to i + 1.
    face_edges: Vec<Vec<usize>>,
    vertex_edges: Vec<Vec<usize>>,
    vertex_faces: Vec<Vec<usize>>,
}

// An edge ring in order, and the quads that it crosses: faces[i] is between
// edges[i] and edges[i + 1], and for a closed ring, the last is between the
// last edge and the first.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Ring {
    pub edges: Vec<usize>,
    pub faces: Vec<usize>,
    pub closed: bool,
}

impl Topology {
    pub fn new(mesh: &Mesh) -> Topology {
        let edges: Vec<Edge> = mesh.edges().into_iter().collect();
        let indices: HashMap<Edge, usize> = edges.iter().enumerate().map(|(index, &edge)| (edge, index)).collect();
        let mut edge_faces = vec![Vec::new(); edges.len()];
        let mut forward = vec![0u32; edges.len()];
        let mut backward = vec![0u32; edges.len()];
        let mut vertex_faces = vec![Vec::new(); mesh.positions.len()];
        let mut face_edges = Vec::with_capacity(mesh.faces.len());
        for (face_index, face) in mesh.faces.iter().enumerate() {
            let mut sides = Vec::with_capacity(face.vertices.len());
            for [a, b] in face.sides() {
                let edge = indices[&Edge::new(a, b)];
                edge_faces[edge].push(face_index);
                if a < b {
                    forward[edge] += 1;
                } else {
                    backward[edge] += 1;
                }
                sides.push(edge);
            }
            face_edges.push(sides);
            for &vertex in &face.vertices {
                vertex_faces[vertex as usize].push(face_index);
            }
        }
        let mut vertex_edges = vec![Vec::new(); mesh.positions.len()];
        for (index, edge) in edges.iter().enumerate() {
            for vertex in edge.vertices() {
                vertex_edges[vertex as usize].push(index);
            }
        }
        let edge_consistent = (0..edges.len()).map(|edge| forward[edge] <= 1 && backward[edge] <= 1).collect();
        Topology {
            edges,
            indices,
            edge_faces,
            edge_consistent,
            face_edges,
            vertex_edges,
            vertex_faces,
        }
    }

    pub fn edges(&self) -> &[Edge] {
        &self.edges
    }

    pub fn edge(&self, index: usize) -> Edge {
        self.edges[index]
    }

    // The edge's number, or None if no face has it.
    pub fn edge_index(&self, edge: Edge) -> Option<usize> {
        self.indices.get(&edge).copied()
    }

    // The faces that have the edge, by number, in the mesh's order.
    pub fn edge_faces(&self, edge: usize) -> &[usize] {
        &self.edge_faces[edge]
    }

    // The faces that have the edge, or none if it isn't one of the mesh's.
    pub fn faces_of_edge(&self, edge: Edge) -> &[usize] {
        self.edge_index(edge).map_or(&[], |index| &self.edge_faces[index])
    }

    // The edge of each of the face's sides, by number: side i goes from vertex
    // i to vertex i + 1.
    pub fn face_edges(&self, face: usize) -> &[usize] {
        &self.face_edges[face]
    }

    // The edges at the vertex, by number.
    pub fn vertex_edges(&self, vertex: u32) -> &[usize] {
        &self.vertex_edges[vertex as usize]
    }

    // The faces around the vertex, in the mesh's order.
    pub fn vertex_faces(&self, vertex: u32) -> &[usize] {
        &self.vertex_faces[vertex as usize]
    }

    // How many edges meet at the vertex.
    pub fn valence(&self, vertex: u32) -> usize {
        self.vertex_edges[vertex as usize].len()
    }

    // An edge with one face.
    pub fn is_boundary(&self, edge: usize) -> bool {
        self.edge_faces[edge].len() == 1
    }

    // An edge with more than two faces, or with two that go along it the same
    // way.
    pub fn is_non_manifold(&self, edge: usize) -> bool {
        self.edge_faces[edge].len() > 2 || !self.edge_consistent[edge]
    }

    pub fn is_boundary_vertex(&self, vertex: u32) -> bool {
        self.vertex_edges(vertex).iter().any(|&edge| self.is_boundary(edge))
    }

    pub fn boundary_edges(&self) -> Vec<Edge> {
        (0..self.edges.len()).filter(|&edge| self.is_boundary(edge)).map(|edge| self.edges[edge]).collect()
    }

    pub fn non_manifold_edges(&self) -> Vec<Edge> {
        (0..self.edges.len()).filter(|&edge| self.is_non_manifold(edge)).map(|edge| self.edges[edge]).collect()
    }

    // Whether every edge has two faces: a surface without holes.
    pub fn is_closed(&self) -> bool {
        self.edge_faces.iter().all(|faces| faces.len() == 2)
    }

    // Whether the faces next to each edge agree on which side is their front,
    // so that no two go along an edge the same way.
    pub fn is_consistently_wound(&self) -> bool {
        self.edge_consistent.iter().all(|&consistent| consistent)
    }

    // Vertices minus edges plus faces, counting every vertex: 2 for a closed
    // surface like a sphere, 0 for a torus, and 1 for a disk.
    pub fn euler_characteristic(&self) -> i64 {
        self.vertex_edges.len() as i64 - self.edges.len() as i64 + self.face_edges.len() as i64
    }

    // The edge loop through an edge, in order, as Blender selects it: across
    // each vertex where four edges meet, to the edge that shares no face with
    // the one before, and along the boundary from a boundary edge. It stops
    // where it can't go on, or where it closes.
    pub fn edge_loop(&self, edge: Edge) -> Vec<Edge> {
        let Some(start) = self.edge_index(edge) else {
            return Vec::new();
        };
        let boundary = self.is_boundary(start);
        let [a, b] = self.edges[start].vertices();
        let (forward, closed) = self.walk_loop(start, b, boundary);
        let mut path = Vec::new();
        if !closed {
            let (backward, _) = self.walk_loop(start, a, boundary);
            path.extend(backward.into_iter().rev().filter(|edge| !forward.contains(edge)));
        }
        path.push(start);
        path.extend(forward);
        path.into_iter().map(|edge| self.edges[edge]).collect()
    }

    // The edges after `start` toward `vertex`, and whether the walk came back
    // to `start`.
    fn walk_loop(&self, start: usize, mut vertex: u32, boundary: bool) -> (Vec<usize>, bool) {
        let mut path = Vec::new();
        let mut visited = BTreeSet::from([start]);
        let mut edge = start;
        while let Some(next) = self.next_in_loop(edge, vertex, boundary) {
            if next == start {
                return (path, true);
            }
            if !visited.insert(next) {
                break;
            }
            path.push(next);
            vertex = self.edges[next].other(vertex);
            edge = next;
        }
        (path, false)
    }

    fn next_in_loop(&self, edge: usize, vertex: u32, boundary: bool) -> Option<usize> {
        let around = self.vertex_edges(vertex);
        let candidates: Vec<usize> = if boundary {
            around.iter().copied().filter(|&other| other != edge && self.is_boundary(other)).collect()
        } else {
            if around.len() != 4 || around.iter().any(|&other| self.edge_faces[other].len() != 2) {
                return None;
            }
            let faces = &self.edge_faces[edge];
            around
                .iter()
                .copied()
                .filter(|&other| other != edge && self.edge_faces[other].iter().all(|face| !faces.contains(face)))
                .collect()
        };
        (candidates.len() == 1).then(|| candidates[0])
    }

    // The edge ring through an edge, in order: from each edge to the opposite
    // side of each quad next to it, as far as the quads go, or until it closes.
    pub fn edge_ring(&self, edge: Edge) -> Vec<Edge> {
        self.ring(edge).edges.into_iter().map(|edge| self.edges[edge]).collect()
    }

    // The quads that the edge ring through an edge crosses, which Blender
    // selects as a face loop.
    pub fn face_loop(&self, edge: Edge) -> Vec<usize> {
        self.ring(edge).faces
    }

    // The edge ring through an edge, with the quads between its edges. An edge
    // with more than two faces has no ring but itself.
    pub fn ring(&self, edge: Edge) -> Ring {
        let Some(start) = self.edge_index(edge) else {
            return Ring::default();
        };
        let faces = &self.edge_faces[start];
        if faces.len() > 2 {
            return Ring { edges: vec![start], faces: Vec::new(), closed: false };
        }
        let (forward, forward_faces, closed) = self.walk_ring(start, faces[0]);
        if closed {
            let mut edges = vec![start];
            edges.extend(forward);
            return Ring { edges, faces: forward_faces, closed };
        }
        let (backward, backward_faces) = match faces.get(1) {
            Some(&face) => {
                let (edges, faces, _) = self.walk_ring(start, face);
                (edges, faces)
            }
            None => (Vec::new(), Vec::new()),
        };
        let mut edges: Vec<usize> = backward.into_iter().rev().collect();
        edges.push(start);
        edges.extend(forward);
        let mut faces: Vec<usize> = backward_faces.into_iter().rev().collect();
        faces.extend(forward_faces);
        Ring { edges, faces, closed: false }
    }

    // The edges after `start` through `face`, the quads crossed on the way,
    // and whether the walk came back to `start`, when the last quad is the
    // one between the last edge and `start`.
    fn walk_ring(&self, start: usize, mut face: usize) -> (Vec<usize>, Vec<usize>, bool) {
        let mut edges = Vec::new();
        let mut faces = Vec::new();
        let mut visited = BTreeSet::from([start]);
        let mut edge = start;
        loop {
            let sides = &self.face_edges[face];
            if sides.len() != 4 {
                break;
            }
            let Some(position) = sides.iter().position(|&side| side == edge) else {
                break;
            };
            let opposite = sides[(position + 2) % 4];
            if opposite == start {
                faces.push(face);
                return (edges, faces, true);
            }
            if !visited.insert(opposite) {
                break;
            }
            faces.push(face);
            edges.push(opposite);
            let next_faces = &self.edge_faces[opposite];
            if next_faces.len() != 2 {
                break;
            }
            face = if next_faces[0] == face { next_faces[1] } else { next_faces[0] };
            edge = opposite;
        }
        (edges, faces, false)
    }
}
