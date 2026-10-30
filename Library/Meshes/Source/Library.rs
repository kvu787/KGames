// Simple meshes for SimplePaint, made in code: a UV sphere, a box and a
// plane. Each has one material, smooth normals on the sphere and flat ones
// on the others, and triangles wound counterclockwise when seen from
// outside, the right-handed convention.

use simple_paint::Vertex;

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

// A sphere of radius 1 around the origin, with its poles on the Y axis:
// `longitudes` vertices around each ring, and `latitudes` bands from pole to
// pole. A port of UVSphere.h from Veehiicuul's 3dTestScene_CppDx12.
pub fn uv_sphere(longitudes: u32, latitudes: u32, material_index: u32) -> Mesh {
    assert!(
        (3..=512).contains(&longitudes) && (2..=512).contains(&latitudes),
        "a UV sphere needs 3 to 512 longitudes and 2 to 512 latitudes"
    );
    let point = |position: [f32; 3]| Vertex {
        position,
        normal: position,
        material_index,
    };
    let mut vertices = Vec::with_capacity((2 + longitudes * (latitudes - 1)) as usize);
    let mut indices = Vec::with_capacity((6 * longitudes * (latitudes - 1)) as usize);
    vertices.push(point([0.0, 1.0, 0.0]));
    for v in 1..latitudes {
        let latitude = std::f32::consts::PI * v as f32 / latitudes as f32;
        for u in 0..longitudes {
            let longitude = std::f32::consts::TAU * u as f32 / longitudes as f32;
            vertices.push(point([
                latitude.sin() * longitude.cos(),
                latitude.cos(),
                latitude.sin() * longitude.sin(),
            ]));
        }
    }
    let bottom = vertices.len() as u32;
    vertices.push(point([0.0, -1.0, 0.0]));

    // Wrap longitude without duplicating the seam. Separate pole fans avoid
    // zero-area triangles, including at the minimum latitude count.
    for u in 0..longitudes {
        let next = (u + 1) % longitudes;
        indices.extend([0, 1 + next, 1 + u]);
        for v in 0..latitudes - 2 {
            let a = 1 + v * longitudes + u;
            let b = 1 + v * longitudes + next;
            let c = a + longitudes;
            let d = b + longitudes;
            indices.extend([a, b, c, b, d, c]);
        }
        let last_ring = 1 + (latitudes - 2) * longitudes;
        indices.extend([bottom, last_ring + u, last_ring + next]);
    }
    Mesh { vertices, indices }
}

// A box centered on the origin, `size` along X, Y and Z.
pub fn cuboid(size: [f32; 3], material_index: u32) -> Mesh {
    let half = size.map(|length| length * 0.5);
    let mut mesh = Mesh {
        vertices: Vec::with_capacity(24),
        indices: Vec::with_capacity(36),
    };
    // Each face's normal, and two axes along it whose cross product is the
    // normal, so the corners go counterclockwise seen from outside.
    for axis in 0..3 {
        for sign in [1.0, -1.0] {
            let (u_axis, v_axis) = ((axis + 1) % 3, (axis + 2) % 3);
            let mut normal = [0.0; 3];
            normal[axis] = sign;
            let mut u = [0.0; 3];
            u[u_axis] = half[u_axis] * sign;
            let mut v = [0.0; 3];
            v[v_axis] = half[v_axis];
            let center = std::array::from_fn(|i| normal[i] * half[i]);
            add_quad(&mut mesh, center, u, v, normal, material_index);
        }
    }
    mesh
}

// A plane through the origin, facing +Y, `size` along X and Z.
pub fn plane(size: [f32; 2], material_index: u32) -> Mesh {
    let mut mesh = Mesh {
        vertices: Vec::with_capacity(4),
        indices: Vec::with_capacity(6),
    };
    // Z cross X is +Y.
    let u = [0.0, 0.0, size[1] * 0.5];
    let v = [size[0] * 0.5, 0.0, 0.0];
    add_quad(&mut mesh, [0.0; 3], u, v, [0.0, 1.0, 0.0], material_index);
    mesh
}

// A rectangle from center - u - v to center + u + v, facing u x v.
fn add_quad(mesh: &mut Mesh, center: [f32; 3], u: [f32; 3], v: [f32; 3], normal: [f32; 3], material_index: u32) {
    let first = mesh.vertices.len() as u32;
    for [a, b] in [[-1.0, -1.0], [1.0, -1.0], [1.0, 1.0], [-1.0, 1.0]] {
        mesh.vertices.push(Vertex {
            position: std::array::from_fn(|i| center[i] + a * u[i] + b * v[i]),
            normal,
            material_index,
        });
    }
    mesh.indices
        .extend([first, first + 1, first + 2, first, first + 2, first + 3]);
}
