// Cutting faces into more: subdividing them where they are, and loop cuts,
// like Blender's Subdivide and Loop Cut. A face next to the cut faces gets
// the new vertices on the edges that it shares with them, so that no edge
// ends partway along another, as Blender does with Create N-Gons.

use crate::mesh::lerp;
use crate::{Edge, Face, Mesh, Selection};
use std::collections::{BTreeMap, BTreeSet};

impl Mesh {
    // Splits each face in place, without smoothing: a triangle into 4 at its
    // edges' midpoints, and any other face into a quad at each corner, from
    // the corner to its edges' midpoints and the face's center. Both halves
    // of a creased edge keep its sharpness, and the pieces their face's
    // material. Returns the pieces, the first of each at its face's number.
    pub fn subdivide_faces(&mut self, faces: impl IntoIterator<Item = usize>) -> Selection {
        let faces = self.face_set(faces);
        let edges: BTreeSet<Edge> = faces.iter().flat_map(|&face| self.faces[face].edges()).collect();
        let mut midpoints = BTreeMap::new();
        for edge in edges {
            let [a, b] = edge.vertices().map(|vertex| self.position(vertex));
            midpoints.insert(edge, self.add_vertex(lerp(a, b, 0.5)));
        }
        self.split_creases(&midpoints);
        let mut pieces = Vec::new();
        let mut extra = Vec::new();
        for index in 0..self.faces.len() {
            if !faces.contains(&index) {
                self.faces[index].vertices = with_cuts(&self.faces[index], &midpoints);
                continue;
            }
            let Face { vertices, material } = self.faces[index].clone();
            let count = vertices.len();
            let middle = |place: usize| midpoints[&Edge::new(vertices[place % count], vertices[(place + 1) % count])];
            let mut split = if count == 3 {
                let [ab, bc, ca] = [middle(0), middle(1), middle(2)];
                vec![vec![vertices[0], ab, ca], vec![ab, vertices[1], bc], vec![ca, bc, vertices[2]], vec![ab, bc, ca]]
            } else {
                let center = self.add_vertex(self.face_center(index));
                (0..count)
                    .map(|place| vec![vertices[place], middle(place), center, middle(place + count - 1)])
                    .collect()
            }
            .into_iter();
            self.faces[index].vertices = split.next().unwrap();
            pieces.push(index);
            extra.extend(split.map(|piece| Face::new(piece, material)));
        }
        for face in extra {
            pieces.push(self.faces.len());
            self.faces.push(face);
        }
        self.prune_creases();
        Selection::from_faces(self, pieces)
    }

    // Cuts a loop through the edge ring of `edge`, like Blender's Loop Cut:
    // a new vertex on each edge of the ring, `fraction` of the way along it
    // from `edge`'s first vertex, and the same way along the others, which
    // run beside it. Each quad that the ring crosses splits in two, and the
    // faces at its ends get the new vertices. Returns the new loop of edges.
    pub fn loop_cut(&mut self, edge: Edge, fraction: f32) -> Result<Selection, String> {
        let topology = self.topology();
        let [a, b] = edge.vertices();
        if topology.edge_index(edge).is_none() {
            return Err(format!("The mesh has no edge from vertex {a} to {b}."));
        }
        let ring = topology.ring(edge);
        if ring.faces.is_empty() {
            return Err("A loop cut goes across quads, and this edge has none next to it.".to_string());
        }
        if ring.faces.iter().collect::<BTreeSet<_>>().len() != ring.faces.len() {
            return Err("This edge's ring crosses a quad twice, so it can't be cut.".to_string());
        }
        let fraction = if fraction.is_nan() { 0.5 } else { fraction.clamp(0.0, 1.0) };
        // Each edge of the ring from its end beside the start of the one
        // before, from `edge` both ways.
        let count = ring.edges.len();
        let start = ring.edges.iter().position(|&index| topology.edge(index) == edge).unwrap();
        let mut directions = vec![[0, 0]; count];
        directions[start] = [a, b];
        let forward = if ring.closed { count - 1 } else { count - 1 - start };
        for step in 0..forward {
            let [from, to] = [(start + step) % count, (start + step + 1) % count];
            directions[to] = across(&self.faces[ring.faces[from]], directions[from]);
        }
        if !ring.closed {
            for to in (0..start).rev() {
                directions[to] = across(&self.faces[ring.faces[to]], directions[to + 1]);
            }
        }
        let mut cuts = BTreeMap::new();
        for (index, [from, to]) in ring.edges.iter().zip(directions) {
            let position = lerp(self.position(from), self.position(to), fraction);
            cuts.insert(topology.edge(*index), self.add_vertex(position));
        }
        self.split_creases(&cuts);
        let crossed: BTreeSet<usize> = ring.faces.iter().copied().collect();
        for index in 0..self.faces.len() {
            if !crossed.contains(&index) {
                self.faces[index].vertices = with_cuts(&self.faces[index], &cuts);
            }
        }
        let mut loop_edges = Vec::with_capacity(ring.faces.len());
        for &face in &ring.faces {
            let Face { vertices: quad, material } = self.faces[face].clone();
            let cut = |place: usize| cuts.get(&Edge::new(quad[place % 4], quad[(place + 1) % 4])).copied();
            let place = (0..2).find(|&place| cut(place).is_some() && cut(place + 2).is_some()).unwrap();
            let [first, second] = [cut(place).unwrap(), cut(place + 2).unwrap()];
            let corner = |offset: usize| quad[(place + offset) % 4];
            self.faces[face].vertices = vec![corner(0), first, second, corner(3)];
            self.add_face(vec![first, corner(1), corner(2), second], material);
            loop_edges.push(Edge::new(first, second));
        }
        self.prune_creases();
        Ok(Selection::from_edges(self, loop_edges))
    }

    // Gives both halves of each creased edge that's cut its sharpness.
    fn split_creases(&mut self, cuts: &BTreeMap<Edge, u32>) {
        for (&edge, &middle) in cuts {
            let sharpness = self.crease(edge);
            if sharpness > 0.0 {
                let [a, b] = edge.vertices();
                self.set_crease(Edge::new(a, middle), sharpness);
                self.set_crease(Edge::new(middle, b), sharpness);
            }
        }
    }
}

// The face's vertices with each cut edge's new vertex between its ends.
fn with_cuts(face: &Face, cuts: &BTreeMap<Edge, u32>) -> Vec<u32> {
    let mut vertices = Vec::with_capacity(face.vertices.len());
    for [a, b] in face.sides() {
        vertices.push(a);
        if let Some(&cut) = cuts.get(&Edge::new(a, b)) {
            vertices.push(cut);
        }
    }
    vertices
}

// The side of a quad across from the side from `from` to `to`, from its end
// beside `from`.
fn across(quad: &Face, [from, to]: [u32; 2]) -> [u32; 2] {
    let quad = &quad.vertices;
    let place = quad.iter().position(|&vertex| vertex == from).unwrap();
    if quad[(place + 1) % 4] == to {
        [quad[(place + 3) % 4], quad[(place + 2) % 4]]
    } else {
        [quad[(place + 1) % 4], quad[(place + 2) % 4]]
    }
}
