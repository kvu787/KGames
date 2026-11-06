// Extruding faces and edges, and insetting faces, like Blender's. The new
// vertices start where the ones that they come from are, so a modeler moves
// the selection that these return, as Blender does after E.

use crate::{Edge, Face, Mesh, Selection};
use math::{Vector3, add, cross, dot, normalize, scale, subtract};
use std::collections::{BTreeMap, BTreeSet, HashMap};

impl Mesh {
    // Extrudes the faces as one region, like Blender's Extrude Region: the
    // region moves onto new vertices, with a quad along each edge of its
    // boundary between the old vertices and the new, of the material of the
    // face that it comes from. Vertices in the middle of the region, which
    // nothing else uses, move with it instead of being copied. A region that
    // shares no edge with other faces leaves a copy of itself behind, flipped,
    // so that a lone face becomes a closed box. Returns the region, at the
    // same face numbers, with its new vertices.
    pub fn extrude_faces(&mut self, faces: impl IntoIterator<Item = usize>) -> Selection {
        let region = self.face_set(faces);
        if region.is_empty() {
            return Selection::new();
        }
        let topology = self.topology();
        let uses = edge_uses(self, &region);
        let outside = |face: &usize| !region.contains(face);
        let leaves_copy = !uses.keys().any(|&edge| topology.faces_of_edge(edge).iter().any(outside));
        let on_boundary: BTreeSet<u32> =
            uses.iter().filter(|&(_, &count)| count == 1).flat_map(|(edge, _)| edge.vertices()).collect();
        let vertices: BTreeSet<u32> =
            region.iter().flat_map(|&face| self.faces[face].vertices.iter().copied()).collect();
        let mut copies = BTreeMap::new();
        for vertex in vertices {
            if leaves_copy || on_boundary.contains(&vertex) || topology.vertex_faces(vertex).iter().any(outside) {
                copies.insert(vertex, self.add_vertex(self.position(vertex)));
            }
        }
        let moved = |vertex: u32| copies.get(&vertex).copied().unwrap_or(vertex);
        // Creases go with the region's edges, and stay on the boundary's old
        // edges too, which the walls start from.
        for &edge in uses.keys() {
            let [a, b] = edge.vertices();
            if self.crease(edge) > 0.0 {
                self.set_crease(Edge::new(moved(a), moved(b)), self.crease(edge));
            }
        }
        let originals: Vec<Face> = region.iter().map(|&face| self.faces[face].clone()).collect();
        for &face in &region {
            for vertex in &mut self.faces[face].vertices {
                *vertex = moved(*vertex);
            }
        }
        for face in &originals {
            for [a, b] in face.sides() {
                if uses[&Edge::new(a, b)] == 1 {
                    self.add_face(vec![a, b, moved(b), moved(a)], face.material);
                }
            }
        }
        if leaves_copy {
            for mut face in originals {
                face.flip();
                self.faces.push(face);
            }
        }
        self.prune_creases();
        Selection::from_faces(self, region)
    }

    // Extrudes the edges, like Blender does in edge mode: each edge gets a quad
    // from it to a copy of it, wound to agree with the edge's first face, and
    // of that face's material, so that extruding a boundary continues its
    // surface. Edges that share a vertex share its copy. Returns the copies.
    pub fn extrude_edges(&mut self, edges: impl IntoIterator<Item = Edge>) -> Selection {
        let edges = self.edge_set(edges);
        let topology = self.topology();
        let mut copies = BTreeMap::new();
        for edge in &edges {
            for vertex in edge.vertices() {
                copies.entry(vertex).or_insert_with(|| self.add_vertex(self.position(vertex)));
            }
        }
        let mut extruded = Vec::with_capacity(edges.len());
        for edge in edges {
            let face = &self.faces[topology.faces_of_edge(edge)[0]];
            let [a, b] = edge.vertices();
            let [a, b] = if face.has_side(a, b) { [a, b] } else { [b, a] };
            let material = face.material;
            let [copy_a, copy_b] = [copies[&a], copies[&b]];
            self.add_face(vec![b, a, copy_a, copy_b], material);
            self.set_crease(Edge::new(copy_a, copy_b), self.crease(edge));
            extruded.push(Edge::new(copy_a, copy_b));
        }
        Selection::from_edges(self, extruded)
    }

    // Insets the faces as one region, like Blender's Inset Faces: each vertex
    // on the region's boundary gets a copy `thickness` inside it, in the
    // planes of its faces, and the region shrinks onto the copies, with a quad
    // of the face's material along each edge of the boundary, between the
    // old vertices and the new. Where the boundary turns, the copy goes
    // farther, so that it's `thickness` from both edges. Returns the region,
    // at the same face numbers.
    pub fn inset_region(&mut self, faces: impl IntoIterator<Item = usize>, thickness: f32) -> Selection {
        let region = self.face_set(faces);
        if region.is_empty() {
            return Selection::new();
        }
        let uses = edge_uses(self, &region);
        let normals: BTreeMap<usize, Vector3> = region.iter().map(|&face| (face, self.face_normal(face))).collect();
        // Each vertex's corners in the region's faces, as a face and a place
        // in it.
        let mut corners: BTreeMap<u32, Vec<(usize, usize)>> = BTreeMap::new();
        for &face in &region {
            for (place, &vertex) in self.faces[face].vertices.iter().enumerate() {
                corners.entry(vertex).or_default().push((face, place));
            }
        }
        // Each copy to make, with the corners that move onto it.
        let mut moves: Vec<(Vector3, Vec<(usize, usize)>)> = Vec::new();
        for (&vertex, corners) in &corners {
            // Corners that share an edge inside the region move together. A
            // vertex where the region touches itself only at the vertex gets
            // a copy for each part.
            let sides = |&(face, place): &(usize, usize)| {
                let vertices = &self.faces[face].vertices;
                let count = vertices.len();
                [vertices[(place + count - 1) % count], vertices[(place + 1) % count]]
            };
            let mut group: Vec<usize> = (0..corners.len()).collect();
            for first in 0..corners.len() {
                for second in first + 1..corners.len() {
                    let shared = sides(&corners[first])
                        .iter()
                        .any(|&end| sides(&corners[second]).contains(&end) && uses[&Edge::new(vertex, end)] >= 2);
                    if shared {
                        let (old, new) = (group[second], group[first]);
                        group.iter_mut().filter(|member| **member == old).for_each(|member| *member = new);
                    }
                }
            }
            for root in group.iter().copied().collect::<BTreeSet<usize>>() {
                let members: Vec<(usize, usize)> =
                    (0..corners.len()).filter(|&corner| group[corner] == root).map(|corner| corners[corner]).collect();
                let position = self.position(vertex);
                let mut inward = Vec::new();
                for corner in &members {
                    let [previous, next] = sides(corner);
                    let normal = normals[&corner.0];
                    if uses[&Edge::new(previous, vertex)] == 1 {
                        inward.push(normalize(cross(normal, subtract(position, self.position(previous)))));
                    }
                    if uses[&Edge::new(vertex, next)] == 1 {
                        inward.push(normalize(cross(normal, subtract(self.position(next), position))));
                    }
                }
                if inward.is_empty() {
                    continue;
                }
                moves.push((add(position, inset_offset(&inward, thickness)), members));
            }
        }
        let mut copies: HashMap<(usize, usize), u32> = HashMap::new();
        for (position, members) in moves {
            let copy = self.add_vertex(position);
            copies.extend(members.into_iter().map(|corner| (corner, copy)));
        }
        let originals: Vec<(usize, Face)> = region.iter().map(|&face| (face, self.faces[face].clone())).collect();
        for (face, original) in &originals {
            let face = *face;
            let count = original.vertices.len();
            let inner = |place: usize| {
                copies.get(&(face, place % count)).copied().unwrap_or(original.vertices[place % count])
            };
            for place in 0..count {
                let [a, b] = [original.vertices[place], original.vertices[(place + 1) % count]];
                let edge = Edge::new(a, b);
                if uses[&edge] == 1 {
                    self.add_face(vec![a, b, inner(place + 1), inner(place)], original.material);
                } else if self.crease(edge) > 0.0 {
                    self.set_crease(Edge::new(inner(place), inner(place + 1)), self.crease(edge));
                }
            }
            self.faces[face].vertices = (0..count).map(inner).collect();
        }
        self.prune_creases();
        Selection::from_faces(self, region)
    }

    // Insets each face on its own, like Blender's Inset Faces with
    // Individual: a smaller copy of the face, `thickness` inside each of its
    // edges, in its plane, and a quad between each of its edges and the
    // copy's. Returns the copies, at the faces' numbers.
    pub fn inset_individual(&mut self, faces: impl IntoIterator<Item = usize>, thickness: f32) -> Selection {
        let faces = self.face_set(faces);
        for &face in &faces {
            let normal = self.face_normal(face);
            if normal == [0.0; 3] {
                continue;
            }
            let Face { vertices, material } = self.faces[face].clone();
            let count = vertices.len();
            let inner: Vec<u32> = (0..count)
                .map(|place| {
                    let [previous, vertex, next] =
                        [vertices[(place + count - 1) % count], vertices[place], vertices[(place + 1) % count]]
                            .map(|vertex| self.position(vertex));
                    let inward = [
                        normalize(cross(normal, subtract(vertex, previous))),
                        normalize(cross(normal, subtract(next, vertex))),
                    ];
                    self.add_vertex(add(vertex, inset_offset(&inward, thickness)))
                })
                .collect();
            for place in 0..count {
                let next = (place + 1) % count;
                self.add_face(vec![vertices[place], vertices[next], inner[next], inner[place]], material);
            }
            self.faces[face].vertices = inner;
        }
        Selection::from_faces(self, faces)
    }
}

// How many of the faces have each of their edges: 1 on the boundary of the
// region that they make.
fn edge_uses(mesh: &Mesh, faces: &BTreeSet<usize>) -> BTreeMap<Edge, usize> {
    let mut uses = BTreeMap::new();
    for &face in faces {
        for edge in mesh.faces[face].edges() {
            *uses.entry(edge).or_default() += 1;
        }
    }
    uses
}

// Where a vertex moves to be `thickness` inside each of the edges that it's
// on, from the directions into the face away from each edge. Along the
// bisector of two of them, it goes farther the more they turn, up to 4 times
// as far, for a nearly folded-back corner.
fn inset_offset(inward: &[Vector3], thickness: f32) -> Vector3 {
    let sum = inward.iter().fold([0.0; 3], |sum, &direction| add(sum, direction));
    let direction = normalize(sum);
    if direction == [0.0; 3] {
        return scale(inward[0], thickness);
    }
    if inward.len() == 2 {
        scale(direction, thickness / dot(direction, inward[0]).max(0.25))
    } else {
        scale(direction, thickness)
    }
}
