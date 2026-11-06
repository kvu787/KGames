// A modeler's edits of a mesh, like Blender's in edit mode: moving vertices,
// deleting, removing loose vertices, making faces, merging, flipping and
// recalculating normals, creasing, and assigning materials. Extrusion.rs and
// Cutting.rs have the rest. An edit that changes which faces there are
// returns the selection that results, as Blender leaves it, and the others
// leave the selection as it was. Elements that aren't in the mesh are left
// out of an edit, and so is an edit with nothing to do.
//
// Edits that add vertices and faces add them at the end, and keep the
// numbers of the faces that they change in place, like an extruded region's.
// Edits that remove vertices or faces, deleting and merging, renumber the
// rest in order, without gaps.

use crate::mesh::{average, normalize_or_zero};
use crate::{Edge, Face, Mesh, Selection, Topology};
use math::{Matrix, Vector3, add, cross, dot, length, scale, subtract};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};

impl Mesh {
    pub fn translate(&mut self, vertices: impl IntoIterator<Item = u32>, offset: Vector3) {
        for vertex in self.vertex_set(vertices) {
            let position = &mut self.positions[vertex as usize];
            *position = add(*position, offset);
        }
    }

    // Rotates by `angle` radians about `axis` through `pivot`, counterclockwise
    // when seen from where the axis points, like Math's rotations.
    pub fn rotate(&mut self, vertices: impl IntoIterator<Item = u32>, pivot: Vector3, axis: Vector3, angle: f32) {
        let axis = normalize_or_zero(axis);
        if axis == [0.0; 3] {
            return;
        }
        let (sin, cos) = angle.sin_cos();
        for vertex in self.vertex_set(vertices) {
            let position = &mut self.positions[vertex as usize];
            // Rodrigues' rotation formula.
            let p = subtract(*position, pivot);
            let rotated = add(
                add(scale(p, cos), scale(cross(axis, p), sin)),
                scale(axis, dot(axis, p) * (1.0 - cos)),
            );
            *position = add(pivot, rotated);
        }
    }

    // Scales by each of `factors` along X, Y and Z, about `pivot`. A negative
    // factor mirrors the vertices, which turns their faces inside out until
    // they're flipped.
    pub fn scale(&mut self, vertices: impl IntoIterator<Item = u32>, pivot: Vector3, factors: Vector3) {
        for vertex in self.vertex_set(vertices) {
            let position = &mut self.positions[vertex as usize];
            let p = subtract(*position, pivot);
            *position = add(pivot, [p[0] * factors[0], p[1] * factors[1], p[2] * factors[2]]);
        }
    }

    // Transforms by `matrix` about `pivot`: the pivot goes where the matrix
    // takes the origin, and the rest goes with it.
    pub fn transform(&mut self, vertices: impl IntoIterator<Item = u32>, pivot: Vector3, matrix: &Matrix) {
        for vertex in self.vertex_set(vertices) {
            let position = &mut self.positions[vertex as usize];
            *position = add(pivot, matrix.transform_point(subtract(*position, pivot)));
        }
    }

    // Deletes the vertices and every face that uses one, like Blender's
    // Delete Vertices. The faces' other vertices stay, loose if no face is
    // left that uses them.
    pub fn delete_vertices(&mut self, vertices: impl IntoIterator<Item = u32>) -> Selection {
        let removed = self.vertex_set(vertices);
        let faces = (0..self.faces.len())
            .filter(|&face| self.faces[face].vertices.iter().any(|vertex| removed.contains(vertex)))
            .collect();
        self.remove_faces(&faces);
        self.remove_vertices(&removed);
        Selection::new()
    }

    // Deletes the faces that have one of the edges, and the edges' vertices
    // that no face is left using, like Blender's Delete Edges.
    pub fn delete_edges(&mut self, edges: impl IntoIterator<Item = Edge>) -> Selection {
        let edges = self.edge_set(edges);
        let faces = (0..self.faces.len())
            .filter(|&face| self.faces[face].edges().any(|edge| edges.contains(&edge)))
            .collect();
        let candidates = edges.iter().flat_map(|edge| edge.vertices()).collect();
        self.remove_faces(&faces);
        self.remove_unused(&candidates);
        Selection::new()
    }

    // Deletes the faces, and their vertices that no face is left using, like
    // Blender's Delete Faces.
    pub fn delete_faces(&mut self, faces: impl IntoIterator<Item = usize>) -> Selection {
        let faces = self.face_set(faces);
        let candidates = faces.iter().flat_map(|&face| self.faces[face].vertices.iter().copied()).collect();
        self.remove_faces(&faces);
        self.remove_unused(&candidates);
        Selection::new()
    }

    // Deletes the faces and nothing else, like Blender's Only Faces. Their
    // vertices stay, loose if no face is left that uses them. A mesh has no
    // edges without faces, so the edges that only these faces had go too.
    pub fn delete_only_faces(&mut self, faces: impl IntoIterator<Item = usize>) -> Selection {
        let faces = self.face_set(faces);
        self.remove_faces(&faces);
        Selection::new()
    }

    // Removes the vertices that no face uses, and returns each old vertex's
    // new number, or None for one that's gone.
    pub fn remove_unused_vertices(&mut self) -> Vec<Option<u32>> {
        let used: BTreeSet<u32> = self.faces.iter().flat_map(|face| face.vertices.iter().copied()).collect();
        let unused = (0..self.positions.len() as u32).filter(|vertex| !used.contains(vertex)).collect();
        self.remove_vertices(&unused)
    }

    // Makes a face of the vertices, like Blender's F. If their boundary edges
    // go around them in one loop, the face follows it, wound against the
    // faces next to it, filling the hole. Otherwise it goes around their
    // middle, in the plane that they spread across most, wound the way that
    // agrees with most of the faces that it shares an edge with, or else
    // facing away from the rest of the mesh. Its material is a neighbor's.
    pub fn make_face(&mut self, vertices: impl IntoIterator<Item = u32>) -> Result<Selection, String> {
        let vertices = self.vertex_set(vertices);
        if vertices.len() < 3 {
            return Err("Making a face takes at least 3 vertices.".to_string());
        }
        let topology = self.topology();
        let order = match self.boundary_cycle(&topology, &vertices) {
            Some(cycle) => cycle,
            None => self.around_middle(&topology, &vertices),
        };
        if self.faces.iter().any(|face| face.vertices.iter().copied().collect::<BTreeSet<u32>>() == vertices) {
            return Err("There's already a face with these vertices.".to_string());
        }
        let material = self.neighbor_material(&topology, &order);
        let face = self.add_face(order, material);
        Ok(Selection::from_faces(self, [face]))
    }

    // The loop that the vertices' boundary edges make, if they make one loop
    // through all of them, in the order that fills it: against each edge's
    // face.
    fn boundary_cycle(&self, topology: &Topology, vertices: &BTreeSet<u32>) -> Option<Vec<u32>> {
        let mut next = BTreeMap::new();
        for edge in topology.boundary_edges() {
            let [a, b] = edge.vertices();
            if !vertices.contains(&a) || !vertices.contains(&b) {
                continue;
            }
            let [from, to] = self.filling_side(topology, edge);
            if next.insert(from, to).is_some() {
                return None;
            }
        }
        let start = *vertices.first()?;
        let mut cycle = vec![start];
        let mut vertex = *next.get(&start)?;
        while vertex != start {
            if cycle.len() > vertices.len() {
                return None;
            }
            cycle.push(vertex);
            vertex = *next.get(&vertex)?;
        }
        (cycle.len() == vertices.len()).then_some(cycle)
    }

    // A boundary edge's vertices in the order that a face filling its hole
    // goes along it, against its one face.
    fn filling_side(&self, topology: &Topology, edge: Edge) -> [u32; 2] {
        let [a, b] = edge.vertices();
        let face = topology.faces_of_edge(edge)[0];
        if self.faces[face].has_side(a, b) { [b, a] } else { [a, b] }
    }

    fn around_middle(&self, topology: &Topology, vertices: &BTreeSet<u32>) -> Vec<u32> {
        let center = average(vertices.iter().map(|&vertex| self.position(vertex))).unwrap();
        let offset = |vertex: u32| subtract(self.position(vertex), center);
        // The plane through the farthest vertex and the one farthest from the
        // line through it.
        let far = |measure: &dyn Fn(u32) -> f32| {
            vertices.iter().copied().max_by(|&a, &b| measure(a).total_cmp(&measure(b))).unwrap()
        };
        let first = far(&|vertex| length(offset(vertex)));
        let u = normalize_or_zero(offset(first));
        let second = far(&|vertex| length(cross(u, offset(vertex))));
        let normal = normalize_or_zero(cross(offset(first), offset(second)));
        let w = cross(normal, u);
        let angle = |vertex: u32| dot(offset(vertex), w).atan2(dot(offset(vertex), u));
        let mut order: Vec<u32> = vertices.iter().copied().collect();
        order.sort_by(|&a, &b| angle(a).total_cmp(&angle(b)));
        // Counterclockwise around `normal` now. Each face that shares a side
        // with it should go along that side the other way.
        let mut agree = 0;
        let mut disagree = 0;
        let count = order.len();
        for index in 0..count {
            let [a, b] = [order[index], order[(index + 1) % count]];
            for &face in topology.faces_of_edge(Edge::new(a, b)) {
                if self.faces[face].has_side(a, b) {
                    disagree += 1;
                } else {
                    agree += 1;
                }
            }
        }
        let away = average(self.positions.iter().copied()).map_or(0.0, |middle| dot(normal, subtract(center, middle)));
        if disagree > agree || (agree == disagree && away < 0.0) {
            order[1..].reverse();
        }
        order
    }

    // The material of a face that shares a side with the vertices' loop, or
    // else of one that uses one of them, or 0.
    fn neighbor_material(&self, topology: &Topology, order: &[u32]) -> u32 {
        let count = order.len();
        (0..count)
            .flat_map(|index| topology.faces_of_edge(Edge::new(order[index], order[(index + 1) % count])))
            .chain(order.iter().flat_map(|&vertex| topology.vertex_faces(vertex)))
            .next()
            .map_or(0, |&face| self.faces[face].material)
    }

    // Fills each hole that the boundary edges go all the way around with a
    // face, wound against the faces around it, like F on a loop of edges in
    // Blender. Edges that aren't on the boundary are left out.
    pub fn fill(&mut self, edges: impl IntoIterator<Item = Edge>) -> Result<Selection, String> {
        let topology = self.topology();
        let mut next: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
        let mut sides = Vec::new();
        for edge in self.edge_set(edges) {
            if topology.faces_of_edge(edge).len() == 1 {
                let [from, to] = self.filling_side(&topology, edge);
                next.entry(from).or_default().push(to);
                sides.push([from, to]);
            }
        }
        let mut used = BTreeSet::new();
        let mut filled = Vec::new();
        for [from, to] in sides {
            if !used.insert([from, to]) {
                continue;
            }
            let mut cycle = vec![from];
            let mut vertex = to;
            let closed = loop {
                if vertex == from {
                    break true;
                }
                let unused =
                    next.get(&vertex).and_then(|ends| ends.iter().find(|&&end| !used.contains(&[vertex, end])));
                let Some(&after) = unused else {
                    break false;
                };
                used.insert([vertex, after]);
                cycle.push(vertex);
                vertex = after;
            };
            if closed && cycle.len() >= 3 {
                let material = self.neighbor_material(&topology, &cycle);
                filled.push(self.add_face(cycle, material));
            }
        }
        if filled.is_empty() {
            return Err("The selection has no loop of boundary edges around a hole to fill.".to_string());
        }
        Ok(Selection::from_faces(self, filled))
    }

    // Merges the vertices into one at their average, which keeps the
    // smallest of their numbers before the rest are renumbered. Faces lose
    // the sides that collapse, and faces left with fewer than 3 vertices go.
    pub fn merge_at_center(&mut self, vertices: impl IntoIterator<Item = u32>) -> Selection {
        let vertices = self.vertex_set(vertices);
        let Some(&target) = vertices.first() else {
            return Selection::new();
        };
        self.positions[target as usize] = average(vertices.iter().map(|&vertex| self.position(vertex))).unwrap();
        let mut representative: Vec<u32> = (0..self.positions.len() as u32).collect();
        for &vertex in &vertices {
            representative[vertex as usize] = target;
        }
        let renumbered = self.weld(&representative);
        Selection::from_vertices(self, renumbered[target as usize])
    }

    // Merges each of the vertices into the first, by number, within
    // `distance` of it, like Blender's Merge by Distance: the first stays
    // where it is. Returns the selected vertices that are left.
    pub fn merge_by_distance(&mut self, vertices: impl IntoIterator<Item = u32>, distance: f32) -> Selection {
        let vertices = self.vertex_set(vertices);
        let distance = if distance > 0.0 { distance } else { 0.0 };
        // Vertices by the cube of `distance` that they're in, so each looks
        // only at its own and the 26 around it.
        let cell_size = distance.max(1.0e-6) as f64;
        let cell = |position: Vector3| position.map(|p| (p as f64 / cell_size).floor() as i64);
        let mut cells: HashMap<[i64; 3], Vec<u32>> = HashMap::new();
        for &vertex in &vertices {
            cells.entry(cell(self.position(vertex))).or_default().push(vertex);
        }
        let mut representative: Vec<u32> = (0..self.positions.len() as u32).collect();
        for &vertex in &vertices {
            if representative[vertex as usize] != vertex {
                continue;
            }
            let position = self.position(vertex);
            let [x, y, z] = cell(position);
            let mut near = Vec::new();
            for dx in -1..=1 {
                for dy in -1..=1 {
                    for dz in -1..=1 {
                        near.extend(cells.get(&[x + dx, y + dy, z + dz]).into_iter().flatten().copied());
                    }
                }
            }
            for other in near {
                if other > vertex
                    && representative[other as usize] == other
                    && length(subtract(self.position(other), position)) <= distance
                {
                    representative[other as usize] = vertex;
                }
            }
        }
        let survivors: Vec<u32> =
            vertices.iter().copied().filter(|&vertex| representative[vertex as usize] == vertex).collect();
        let renumbered = self.weld(&representative);
        Selection::from_vertices(self, survivors.into_iter().filter_map(|vertex| renumbered[vertex as usize]))
    }

    // Replaces each vertex with its representative in faces and creases,
    // removes the vertices that were replaced, and returns each old vertex's
    // new number. Faces lose the sides that collapse, split where they come
    // to touch themselves, and go when they're left with fewer than 3
    // vertices, or with the same vertices as a face before them.
    pub(crate) fn weld(&mut self, representative: &[u32]) -> Vec<Option<u32>> {
        let mut seen = HashSet::new();
        let mut faces = Vec::with_capacity(self.faces.len());
        for face in std::mem::take(&mut self.faces) {
            let vertices = face.vertices.iter().map(|&vertex| representative[vertex as usize]).collect();
            for piece in simple_polygons(vertices) {
                let mut key = piece.clone();
                key.sort_unstable();
                if seen.insert(key) {
                    faces.push(Face::new(piece, face.material));
                }
            }
        }
        self.faces = faces;
        let mut creases = BTreeMap::new();
        for (edge, sharpness) in std::mem::take(&mut self.creases) {
            let [a, b] = edge.vertices().map(|vertex| representative[vertex as usize]);
            if a != b {
                let merged: &mut f32 = creases.entry(Edge::new(a, b)).or_default();
                *merged = merged.max(sharpness);
            }
        }
        self.creases = creases;
        let replaced =
            (0..self.positions.len() as u32).filter(|&vertex| representative[vertex as usize] != vertex).collect();
        let renumbered = self.remove_vertices(&replaced);
        self.prune_creases();
        renumbered
    }

    // Removes the faces, and the creases of edges that no face has any more.
    pub(crate) fn remove_faces(&mut self, faces: &BTreeSet<usize>) {
        if faces.is_empty() {
            return;
        }
        let mut index = 0;
        self.faces.retain(|_| {
            index += 1;
            !faces.contains(&(index - 1))
        });
        self.prune_creases();
    }

    // Removes those of the vertices that no face uses.
    fn remove_unused(&mut self, candidates: &BTreeSet<u32>) {
        let used: BTreeSet<u32> = self.faces.iter().flat_map(|face| face.vertices.iter().copied()).collect();
        let unused = candidates.difference(&used).copied().collect();
        self.remove_vertices(&unused);
    }

    // Removes the vertices, and any face or crease that uses one, renumbers
    // the rest in order, and returns each old vertex's new number.
    pub(crate) fn remove_vertices(&mut self, removed: &BTreeSet<u32>) -> Vec<Option<u32>> {
        let mut renumbered = Vec::with_capacity(self.positions.len());
        let mut kept = 0;
        for vertex in 0..self.positions.len() {
            if removed.contains(&(vertex as u32)) {
                renumbered.push(None);
            } else {
                self.positions[kept] = self.positions[vertex];
                renumbered.push(Some(kept as u32));
                kept += 1;
            }
        }
        if removed.is_empty() {
            return renumbered;
        }
        self.positions.truncate(kept);
        let new = |vertex: &u32| renumbered[*vertex as usize];
        self.faces.retain(|face| face.vertices.iter().all(|vertex| new(vertex).is_some()));
        for face in &mut self.faces {
            for vertex in &mut face.vertices {
                *vertex = new(vertex).unwrap();
            }
        }
        self.creases = std::mem::take(&mut self.creases)
            .into_iter()
            .filter_map(|(edge, sharpness)| {
                let [a, b] = edge.vertices();
                Some((Edge::new(new(&a)?, new(&b)?), sharpness))
            })
            .collect();
        renumbered
    }

    // Reverses the faces' winding, so they face the other way.
    pub fn flip_faces(&mut self, faces: impl IntoIterator<Item = usize>) {
        for face in self.face_set(faces) {
            self.faces[face].flip();
        }
    }

    // Winds the faces consistently, and each surface that they make so that
    // it faces out, like Blender's Recalculate Outside. Faces that share an
    // edge with only each other agree on their winding, and then a closed
    // surface faces out when it encloses a positive volume, and an open one
    // when its face farthest from its middle faces away from it.
    pub fn recalculate_normals_outside(&mut self, faces: impl IntoIterator<Item = usize>) {
        let faces = self.face_set(faces);
        let topology = self.topology();
        let mut visited = BTreeSet::new();
        for &start in &faces {
            if !visited.insert(start) {
                continue;
            }
            let mut surface = vec![start];
            let mut queue = VecDeque::from([start]);
            while let Some(face) = queue.pop_front() {
                for &edge in topology.face_edges(face) {
                    let neighbors = topology.edge_faces(edge);
                    if neighbors.len() != 2 {
                        continue;
                    }
                    let other = if neighbors[0] == face { neighbors[1] } else { neighbors[0] };
                    if !faces.contains(&other) || !visited.insert(other) {
                        continue;
                    }
                    let [a, b] = topology.edge(edge).vertices();
                    let along = if self.faces[face].has_side(a, b) { [a, b] } else { [b, a] };
                    if self.faces[other].has_side(along[0], along[1]) {
                        self.faces[other].flip();
                    }
                    surface.push(other);
                    queue.push_back(other);
                }
            }
            if self.faces_in(&surface) {
                for &face in &surface {
                    self.faces[face].flip();
                }
            }
        }
    }

    // Whether a consistently wound surface faces in.
    fn faces_in(&self, surface: &[usize]) -> bool {
        let vertices: BTreeSet<u32> =
            surface.iter().flat_map(|&face| self.faces[face].vertices.iter().copied()).collect();
        let middle = average(vertices.iter().map(|&vertex| self.position(vertex))).unwrap();
        let mut volume = 0.0f64;
        let mut size = 0.0f32;
        for &face in surface {
            let corners: Vec<Vector3> =
                self.faces[face].vertices.iter().map(|&vertex| subtract(self.position(vertex), middle)).collect();
            for pair in corners[1..].windows(2) {
                volume += dot(corners[0], cross(pair[0], pair[1])) as f64 / 6.0;
            }
            size = corners.iter().fold(size, |size, &corner| size.max(length(corner)));
        }
        if volume.abs() > 1.0e-4 * (size as f64).powi(3) {
            return volume < 0.0;
        }
        let farthest = surface
            .iter()
            .copied()
            .max_by(|&a, &b| {
                let distance = |face: usize| length(subtract(self.face_center(face), middle));
                distance(a).total_cmp(&distance(b))
            })
            .unwrap();
        dot(self.face_normal(farthest), subtract(self.face_center(farthest), middle)) < 0.0
    }

    // Sets the edges' sharpness, from 0, smooth, to INFINITELY_SHARP.
    pub fn crease_edges(&mut self, edges: impl IntoIterator<Item = Edge>, sharpness: f32) {
        for edge in self.edge_set(edges) {
            self.set_crease(edge, sharpness);
        }
    }

    pub fn assign_material(&mut self, faces: impl IntoIterator<Item = usize>, material: u32) {
        for face in self.face_set(faces) {
            self.faces[face].material = material;
        }
    }
}

// A polygon's vertices without the sides that collapsed to a point, split
// into simple polygons where it touches itself, without the ones of fewer
// than 3 vertices.
fn simple_polygons(mut vertices: Vec<u32>) -> Vec<Vec<u32>> {
    vertices.dedup();
    while vertices.len() > 1 && vertices.first() == vertices.last() {
        vertices.pop();
    }
    if vertices.len() < 3 {
        return Vec::new();
    }
    for later in 1..vertices.len() {
        if let Some(earlier) = vertices[..later].iter().position(|&vertex| vertex == vertices[later]) {
            let mut rest = vertices[later..].to_vec();
            rest.extend_from_slice(&vertices[..earlier]);
            let mut pieces = simple_polygons(vertices[earlier..later].to_vec());
            pieces.extend(simple_polygons(rest));
            return pieces;
        }
    }
    vec![vertices]
}
