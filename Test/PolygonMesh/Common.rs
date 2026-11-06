// What several of PolygonMesh's tests check meshes with.

use math::{cross, dot, subtract};
use polygon_mesh::Mesh;

// The volume that the faces enclose, positive when they face out, from
// tetrahedra between the origin and a fan of each face's triangles.
pub fn signed_volume(mesh: &Mesh) -> f32 {
    let mut volume = 0.0f64;
    for face in &mesh.faces {
        let first = mesh.position(face.vertices[0]);
        for pair in face.vertices[1..].windows(2) {
            let [b, c] = [pair[0], pair[1]].map(|vertex| mesh.position(vertex));
            volume += dot(first, cross(b, c)) as f64 / 6.0;
        }
    }
    volume as f32
}

pub fn assert_close(actual: f32, expected: f32, tolerance: f32, what: &str) {
    assert!((actual - expected).abs() <= tolerance, "{what}: {actual} isn't {expected}");
}

pub fn assert_near(actual: [f32; 3], expected: [f32; 3], tolerance: f32, what: &str) {
    let difference = subtract(actual, expected);
    assert!(dot(difference, difference).sqrt() <= tolerance, "{what}: {actual:?} isn't {expected:?}");
}

// Checks the mesh's invariants and that its faces agree on which way is out.
pub fn assert_valid(mesh: &Mesh, what: &str) {
    if let Err(problem) = mesh.validate() {
        panic!("{what}: {problem}");
    }
    assert!(mesh.topology().is_consistently_wound(), "{what} isn't consistently wound");
}

// How many faces have each number of sides, from 3.
pub fn sides_histogram(mesh: &Mesh) -> Vec<usize> {
    let most = mesh.faces.iter().map(|face| face.vertices.len()).max().unwrap_or(3);
    let mut counts = vec![0; most - 2];
    for face in &mesh.faces {
        counts[face.vertices.len() - 3] += 1;
    }
    counts
}
