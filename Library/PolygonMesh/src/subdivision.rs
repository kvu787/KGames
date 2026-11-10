// Catmull-Clark subdivision, the Catmark scheme as OpenSubdiv defines it,
// with its boundary interpolation VTX_BOUNDARY_EDGE_AND_CORNER and its
// uniform creasing: face points, edge points, and vertex points by the
// smooth, dart, crease and corner rules, with semi-sharp creases that lose 1
// of sharpness each level, and vertices whose rule changes between levels
// blending the two by the fractional weight. README.md has the formulas.

use crate::mesh::lerp;
use crate::{Edge, Face, INFINITELY_SHARP, Mesh, Topology};
use math::{Vector3, add, scale};

// A mesh subdivided some levels, and the face of the base mesh that each of
// its faces comes from, for picking and drawing what's selected.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SubdivisionSurface {
    pub mesh: Mesh,
    pub base_faces: Vec<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Rule {
    Smooth,
    Dart,
    Crease,
    Corner,
}

impl Mesh {
    // The mesh subdivided `levels` times. Every face becomes a quad at each
    // corner, so after one level all faces are quads, of their face's
    // material. The base mesh's vertices keep their numbers at every level,
    // then come a vertex for each edge, in the topology's order, then one for
    // each face. Loose vertices stay where they are.
    pub fn catmull_clark(&self, levels: u32) -> SubdivisionSurface {
        let mut surface = SubdivisionSurface { mesh: self.clone(), base_faces: (0..self.faces.len()).collect() };
        for _ in 0..levels {
            let (mesh, parents) = subdivide(&surface.mesh);
            surface.base_faces = parents.into_iter().map(|parent| surface.base_faces[parent]).collect();
            surface.mesh = mesh;
        }
        surface
    }
}

// An edge's sharpness after a level: 1 less, infinite staying infinite.
fn child_sharpness(sharpness: f32) -> f32 {
    if sharpness >= INFINITELY_SHARP {
        INFINITELY_SHARP
    } else if sharpness > 1.0 {
        sharpness - 1.0
    } else {
        0.0
    }
}

// One level, and the face that each new face comes from.
fn subdivide(mesh: &Mesh) -> (Mesh, Vec<usize>) {
    let topology = mesh.topology();
    let edges = topology.edges();
    let (vertex_count, edge_count) = (mesh.positions.len(), edges.len());
    // Boundary and non-manifold edges are infinitely sharp, whatever their
    // creases say.
    let sharpness: Vec<f32> = (0..edge_count)
        .map(|edge| {
            if topology.edge_faces(edge).len() != 2 || topology.is_non_manifold(edge) {
                INFINITELY_SHARP
            } else {
                mesh.crease(edges[edge])
            }
        })
        .collect();
    let child: Vec<f32> = sharpness.iter().map(|&sharpness| child_sharpness(sharpness)).collect();
    let face_points: Vec<Vector3> = (0..mesh.faces.len()).map(|face| mesh.face_center(face)).collect();

    let mut positions = Vec::with_capacity(vertex_count + edge_count + face_points.len());
    for vertex in 0..vertex_count as u32 {
        positions.push(vertex_point(mesh, &topology, &face_points, &sharpness, &child, vertex));
    }
    for (index, edge) in edges.iter().enumerate() {
        let [a, b] = edge.vertices().map(|vertex| mesh.position(vertex));
        let sharp = lerp(a, b, 0.5);
        let smooth = || {
            let faces = topology.edge_faces(index);
            scale(add(add(a, b), add(face_points[faces[0]], face_points[faces[1]])), 0.25)
        };
        // A crease that's gone after this level blends toward smooth by how
        // much of it was left.
        let point = match (sharpness[index] > 0.0, child[index] > 0.0) {
            (false, _) => smooth(),
            (true, true) => sharp,
            (true, false) => lerp(smooth(), sharp, sharpness[index].min(1.0)),
        };
        positions.push(point);
    }
    positions.extend_from_slice(&face_points);

    let edge_point = |edge: usize| (vertex_count + edge) as u32;
    let mut faces = Vec::with_capacity(mesh.faces.len() * 4);
    let mut parents = Vec::with_capacity(mesh.faces.len() * 4);
    for (index, face) in mesh.faces.iter().enumerate() {
        let sides = topology.face_edges(index);
        let count = sides.len();
        let center = (vertex_count + edge_count + index) as u32;
        for corner in 0..count {
            let before = sides[(corner + count - 1) % count];
            faces.push(Face::new(
                vec![face.vertices[corner], edge_point(sides[corner]), center, edge_point(before)],
                face.material,
            ));
            parents.push(index);
        }
    }
    let mut child_mesh = Mesh { positions, faces, ..Mesh::default() };
    for (&edge, &sharpness) in &mesh.creases {
        let (Some(index), true) = (topology.edge_index(edge), child_sharpness(sharpness) > 0.0) else {
            continue;
        };
        let [a, b] = edge.vertices();
        child_mesh.set_crease(Edge::new(a, edge_point(index)), child_sharpness(sharpness));
        child_mesh.set_crease(Edge::new(edge_point(index), b), child_sharpness(sharpness));
    }
    (child_mesh, parents)
}

// Where a vertex moves: by its rule this level, blended with its rule at the
// next where that's different.
fn vertex_point(
    mesh: &Mesh,
    topology: &Topology,
    face_points: &[Vector3],
    sharpness: &[f32],
    child: &[f32],
    vertex: u32,
) -> Vector3 {
    let position = mesh.position(vertex);
    if topology.vertex_faces(vertex).is_empty() {
        return position;
    }
    let corner = is_corner(topology, vertex);
    let around = topology.vertex_edges(vertex);
    let sharp: Vec<usize> = around.iter().copied().filter(|&edge| sharpness[edge] > 0.0).collect();
    let sharp_after: Vec<usize> = around.iter().copied().filter(|&edge| child[edge] > 0.0).collect();
    let rule = rule_of(corner, sharp.len());
    let rule_after = rule_of(corner, sharp_after.len());
    let point_by = |rule: Rule, sharp: &[usize]| match rule {
        Rule::Smooth | Rule::Dart => smooth_point(mesh, topology, face_points, vertex),
        Rule::Crease => crease_point(mesh, topology, vertex, sharp),
        Rule::Corner => position,
    };
    let point = point_by(rule, &sharp);
    if matches!(rule, Rule::Smooth | Rule::Dart) || rule_after == rule {
        return point;
    }
    // The fractional weight: the average sharpness of the edges that stop
    // being sharp after this level.
    let fading: Vec<f32> = sharp.iter().filter(|&&edge| child[edge] == 0.0).map(|&edge| sharpness[edge]).collect();
    let weight = if fading.is_empty() { 0.0 } else { (fading.iter().sum::<f32>() / fading.len() as f32).min(1.0) };
    lerp(point_by(rule_after, &sharp_after), point, weight)
}

fn rule_of(corner: bool, sharp_edges: usize) -> Rule {
    match (corner, sharp_edges) {
        (true, _) => Rule::Corner,
        (false, 0) => Rule::Smooth,
        (false, 1) => Rule::Dart,
        (false, 2) => Rule::Crease,
        (false, _) => Rule::Corner,
    }
}

// The smooth rule: the average of the vertex's face points, twice the average
// of its edges' midpoints, and n - 3 times the vertex, over its valence n.
fn smooth_point(mesh: &Mesh, topology: &Topology, face_points: &[Vector3], vertex: u32) -> Vector3 {
    let position = mesh.position(vertex);
    let around = topology.vertex_edges(vertex);
    let faces = topology.vertex_faces(vertex);
    let valence = around.len() as f32;
    let faces_average =
        scale(faces.iter().fold([0.0; 3], |sum, &face| add(sum, face_points[face])), 1.0 / faces.len() as f32);
    let neighbors_average = scale(
        around.iter().fold([0.0; 3], |sum, &edge| add(sum, mesh.position(topology.edge(edge).other(vertex)))),
        1.0 / valence,
    );
    // Q + 2R + (n - 3) S, where 2R is S plus the neighbors' average.
    scale(add(add(faces_average, neighbors_average), scale(position, valence - 2.0)), 1.0 / valence)
}

// The crease rule: along the curve of the two sharp edges, 6/8 of the vertex
// and 1/8 of each edge's other end.
fn crease_point(mesh: &Mesh, topology: &Topology, vertex: u32, sharp: &[usize]) -> Vector3 {
    let [a, b] = [sharp[0], sharp[1]].map(|edge| mesh.position(topology.edge(edge).other(vertex)));
    scale(add(add(a, b), scale(mesh.position(vertex), 6.0)), 0.125)
}

// Whether the vertex is infinitely sharp: on the boundary with only one face,
// a corner by VTX_BOUNDARY_EDGE_AND_CORNER, or where the surface isn't a
// manifold: an edge of it with more than two faces or two that disagree, more
// than one stretch of boundary through it, or faces around it that don't make
// one fan.
fn is_corner(topology: &Topology, vertex: u32) -> bool {
    let around = topology.vertex_edges(vertex);
    let faces = topology.vertex_faces(vertex);
    let boundary = around.iter().filter(|&&edge| topology.is_boundary(edge)).count();
    if (boundary > 0 && faces.len() == 1) || around.iter().any(|&edge| topology.is_non_manifold(edge)) {
        return true;
    }
    if boundary != 0 && boundary != 2 {
        return true;
    }
    // One fan: the faces joined through the edges between them are one group.
    let mut group: Vec<usize> = (0..faces.len()).collect();
    for &edge in around {
        if let [first, second] = topology.edge_faces(edge) {
            let place = |face: &usize| faces.iter().position(|other| other == face).unwrap();
            let (old, new) = (group[place(second)], group[place(first)]);
            group.iter_mut().filter(|member| **member == old).for_each(|member| *member = new);
        }
    }
    group.iter().any(|&member| member != group[0])
}
