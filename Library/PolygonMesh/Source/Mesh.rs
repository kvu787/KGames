// The mesh: positions, faces of any number of sides with a material each,
// and the sharpness of creased edges.

use crate::Topology;
use math::{Vector3, add, cross, length, scale, subtract};
use std::collections::{BTreeMap, BTreeSet};

// Any sharpness of at least this is infinitely sharp, as in OpenSubdiv, where
// a crease loses 1 of sharpness each level, so 10 outlasts any level that a
// modeler would subdivide to.
pub const INFINITELY_SHARP: f32 = 10.0;

// An undirected edge, by its two vertices, the smaller first, so that both
// faces next to an edge name it the same way.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Edge([u32; 2]);

impl Edge {
    pub fn new(a: u32, b: u32) -> Edge {
        if a <= b { Edge([a, b]) } else { Edge([b, a]) }
    }

    // The smaller vertex, then the larger.
    pub fn vertices(self) -> [u32; 2] {
        self.0
    }

    pub fn contains(self, vertex: u32) -> bool {
        self.0.contains(&vertex)
    }

    // The end that isn't `vertex`.
    pub fn other(self, vertex: u32) -> u32 {
        if self.0[0] == vertex { self.0[1] } else { self.0[0] }
    }
}

// A polygon, by its vertices, counterclockwise when seen from its front, the
// right-handed convention, and its material, which SimplePaint paints it with.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Face {
    pub vertices: Vec<u32>,
    pub material: u32,
}

impl Face {
    pub fn new(vertices: Vec<u32>, material: u32) -> Face {
        Face { vertices, material }
    }

    // Each side, from a vertex to the next, in the face's winding.
    pub fn sides(&self) -> impl Iterator<Item = [u32; 2]> + '_ {
        let count = self.vertices.len();
        (0..count).map(move |index| [self.vertices[index], self.vertices[(index + 1) % count]])
    }

    pub fn edges(&self) -> impl Iterator<Item = Edge> + '_ {
        self.sides().map(|[a, b]| Edge::new(a, b))
    }

    // Whether the face goes from a to b along one of its sides.
    pub fn has_side(&self, a: u32, b: u32) -> bool {
        self.sides().any(|side| side == [a, b])
    }

    // The same polygon wound the other way, from the same first vertex.
    pub fn flip(&mut self) {
        self.vertices[1..].reverse();
    }
}

// A polygon mesh. Every face has at least 3 vertices, each in range and none
// twice, which `validate` checks and every operation keeps. A vertex that no
// face uses is a loose vertex. Edges are the faces' sides, so there are no
// edges without faces. `creases` has the sharpness of each creased edge,
// more than 0 and at most INFINITELY_SHARP; an edge that isn't there is
// smooth. It's Clone so that a modeler can undo by keeping copies.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Mesh {
    pub positions: Vec<Vector3>,
    pub faces: Vec<Face>,
    pub creases: BTreeMap<Edge, f32>,
}

impl Mesh {
    pub fn new() -> Mesh {
        Mesh::default()
    }

    pub fn add_vertex(&mut self, position: Vector3) -> u32 {
        self.positions.push(position);
        (self.positions.len() - 1) as u32
    }

    pub fn add_face(&mut self, vertices: Vec<u32>, material: u32) -> usize {
        self.faces.push(Face::new(vertices, material));
        self.faces.len() - 1
    }

    pub fn vertex_count(&self) -> usize {
        self.positions.len()
    }

    pub fn face_count(&self) -> usize {
        self.faces.len()
    }

    pub fn position(&self, vertex: u32) -> Vector3 {
        self.positions[vertex as usize]
    }

    pub fn topology(&self) -> Topology {
        Topology::new(self)
    }

    // Each edge once, in order.
    pub fn edges(&self) -> BTreeSet<Edge> {
        self.faces.iter().flat_map(Face::edges).collect()
    }

    // An edge's sharpness, 0 if it isn't creased.
    pub fn crease(&self, edge: Edge) -> f32 {
        self.creases.get(&edge).copied().unwrap_or(0.0)
    }

    // Sets an edge's sharpness, limited to 0 to INFINITELY_SHARP, and NaN to
    // 0. Setting 0 removes the crease.
    pub fn set_crease(&mut self, edge: Edge, sharpness: f32) {
        let sharpness = if sharpness.is_nan() { 0.0 } else { sharpness.clamp(0.0, INFINITELY_SHARP) };
        if sharpness > 0.0 {
            self.creases.insert(edge, sharpness);
        } else {
            self.creases.remove(&edge);
        }
    }

    // How many materials SimplePaint needs to paint the mesh: one more than
    // the largest index, and at least 1.
    pub fn material_count(&self) -> u32 {
        self.faces.iter().map(|face| face.material + 1).max().unwrap_or(1)
    }

    // The average of a face's vertices.
    pub fn face_center(&self, face: usize) -> Vector3 {
        let vertices = &self.faces[face].vertices;
        let sum = vertices.iter().fold([0.0; 3], |sum, &vertex| add(sum, self.position(vertex)));
        scale(sum, 1.0 / vertices.len() as f32)
    }

    // A face's area times its unit normal, by Newell's method, which works for
    // any polygon, even one that isn't flat or convex. Measuring from the
    // center keeps it precise far from the origin.
    pub fn face_area_vector(&self, face: usize) -> Vector3 {
        let center = self.face_center(face);
        let face = &self.faces[face];
        let sum = face.sides().fold([0.0; 3], |sum, [a, b]| {
            add(sum, cross(subtract(self.position(a), center), subtract(self.position(b), center)))
        });
        scale(sum, 0.5)
    }

    // The unit normal that the face's winding points to, or zero for a face
    // without area.
    pub fn face_normal(&self, face: usize) -> Vector3 {
        normalize_or_zero(self.face_area_vector(face))
    }

    pub fn face_area(&self, face: usize) -> f32 {
        length(self.face_area_vector(face))
    }

    // Checks what every operation expects: finite positions, faces of at
    // least 3 vertices in range with none twice, and creases between two
    // vertices in range with sharpness in (0, INFINITELY_SHARP].
    pub fn validate(&self) -> Result<(), String> {
        let count = self.positions.len();
        if let Some(vertex) = self.positions.iter().position(|position| position.iter().any(|p| !p.is_finite())) {
            return Err(format!("Vertex {vertex}'s position isn't finite."));
        }
        for (index, face) in self.faces.iter().enumerate() {
            if face.vertices.len() < 3 {
                return Err(format!("Face {index} has {} vertices, but a face needs at least 3.", face.vertices.len()));
            }
            if let Some(vertex) = face.vertices.iter().find(|&&vertex| vertex as usize >= count) {
                return Err(format!("Face {index} uses vertex {vertex}, but there are only {count} vertices."));
            }
            let unique: BTreeSet<u32> = face.vertices.iter().copied().collect();
            if unique.len() != face.vertices.len() {
                return Err(format!("Face {index} uses a vertex more than once."));
            }
        }
        for (edge, &sharpness) in &self.creases {
            let [a, b] = edge.vertices();
            if a == b || b as usize >= count {
                return Err(format!("The crease of {a} to {b} isn't between two vertices of the mesh."));
            }
            if !(sharpness > 0.0 && sharpness <= INFINITELY_SHARP) {
                return Err(format!(
                    "The crease of {a} to {b} has sharpness {sharpness}, but it must be more than 0 and at most \
                     {INFINITELY_SHARP}."
                ));
            }
        }
        Ok(())
    }

    // Drops the creases of edges that no face has any more.
    pub(crate) fn prune_creases(&mut self) {
        if self.creases.is_empty() {
            return;
        }
        let edges = self.edges();
        self.creases.retain(|edge, _| edges.contains(edge));
    }

    // The faces that are in the mesh, each once.
    pub(crate) fn face_set(&self, faces: impl IntoIterator<Item = usize>) -> BTreeSet<usize> {
        faces.into_iter().filter(|&face| face < self.faces.len()).collect()
    }

    // The vertices that are in the mesh, each once.
    pub(crate) fn vertex_set(&self, vertices: impl IntoIterator<Item = u32>) -> BTreeSet<u32> {
        vertices.into_iter().filter(|&vertex| (vertex as usize) < self.positions.len()).collect()
    }

    // The edges that some face has, each once.
    pub(crate) fn edge_set(&self, edges: impl IntoIterator<Item = Edge>) -> BTreeSet<Edge> {
        let existing = self.edges();
        edges.into_iter().filter(|edge| existing.contains(edge)).collect()
    }
}

// v at length 1, or zero if it has no length, or none that a float can
// divide by.
pub(crate) fn normalize_or_zero(v: Vector3) -> Vector3 {
    let length = length(v);
    if length > 1.0e-30 && length.is_finite() { scale(v, 1.0 / length) } else { [0.0; 3] }
}

// The average of the positions, or None if there are none.
pub(crate) fn average(positions: impl IntoIterator<Item = Vector3>) -> Option<Vector3> {
    let mut sum = [0.0f64; 3];
    let mut count = 0usize;
    for position in positions {
        for axis in 0..3 {
            sum[axis] += position[axis] as f64;
        }
        count += 1;
    }
    (count > 0).then(|| sum.map(|s| (s / count as f64) as f32))
}

// From a at t = 0 to b at t = 1.
pub(crate) fn lerp(a: Vector3, b: Vector3, t: f32) -> Vector3 {
    add(a, scale(subtract(b, a), t))
}
