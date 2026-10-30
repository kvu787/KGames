// Host-independent mesh validation, before GPU upload. A conservative
// common-cone test guarantees that every barycentrically interpolated normal
// stays nonzero. Bad meshes must be corrected by their author, not in the
// pixel shader.

use crate::Error;

// What validate_mesh reads from a host's vertex.
pub trait PaintVertex {
    fn position(&self) -> [f32; 3];
    fn normal(&self) -> [f32; 3];
    fn material_index(&self) -> u32;
}

// The vertex that SimplePaint.hlsl's adapter reads: POSITION at byte 0,
// NORMAL at 12, and MATERIAL at 24, 28 bytes in all.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub material_index: u32,
}

const _: () = assert!(size_of::<Vertex>() == 28);

impl PaintVertex for Vertex {
    fn position(&self) -> [f32; 3] {
        self.position
    }

    fn normal(&self) -> [f32; 3] {
        self.normal
    }

    fn material_index(&self) -> u32 {
        self.material_index
    }
}

// Checks a nonempty indexed triangle list: finite positions with components
// of magnitude at most a million, normal lengths in [0.5, 2], valid material
// indices, one material per triangle, and normals that share a cone per
// triangle. See Specification.md.
pub fn validate_mesh<V: PaintVertex>(
    vertices: &[V],
    indices: &[u32],
    material_count: u32,
) -> Result<(), Error> {
    let error = |message: &str| Err(Error(message.to_string()));
    if vertices.is_empty() || indices.is_empty() || !indices.len().is_multiple_of(3) || material_count == 0 {
        return error("SimplePaint requires a nonempty indexed triangle mesh and materials.");
    }
    for vertex in vertices {
        let position = vertex.position();
        let normal = vertex.normal();
        let length_squared: f64 = normal.iter().map(|&n| n as f64 * n as f64).sum();
        let bounded = position
            .iter()
            .all(|&p| p.is_finite() && p.abs() <= 1.0e6);
        if !bounded
            || !length_squared.is_finite()
            || !(0.25..=4.0).contains(&length_squared)
            || vertex.material_index() >= material_count
        {
            return error(
                "SimplePaint vertex requires finite bounded position, normal length [0.5,2], \
                 and valid material index.",
            );
        }
    }
    for triangle in indices.as_chunks::<3>().0 {
        if triangle.iter().any(|&index| index as usize >= vertices.len()) {
            return error("SimplePaint mesh index is out of bounds.");
        }
        let corners = triangle.map(|index| &vertices[index as usize]);
        let material = corners[0].material_index();
        if corners.iter().any(|corner| corner.material_index() != material) {
            return error("SimplePaint triangles must have one material.");
        }
        let sum: [f64; 3] =
            std::array::from_fn(|axis| corners.iter().map(|corner| corner.normal()[axis] as f64).sum());
        let length = sum.iter().map(|s| s * s).sum::<f64>().sqrt();
        for corner in corners {
            let normal = corner.normal();
            let projection: f64 = (0..3).map(|axis| normal[axis] as f64 * sum[axis]).sum();
            if length == 0.0 || projection < 0.125 * length {
                return error(
                    "SimplePaint triangle normals must share a cone with projection at least 0.125.",
                );
            }
        }
    }
    Ok(())
}
