// Meshes to start modeling from, like Blender's Add > Mesh menu: a cube, a
// plane, a cylinder, a cone, a UV sphere and a torus. Each is centered on the
// origin with Y up, has material 0 and no creases, and is wound
// counterclockwise when seen from outside, so its normals point out. They're
// quads where they can be, with caps as single polygons and triangles only at
// the poles of the sphere and the tip of a cone, the way subdivision
// surfaces like them.

use crate::Mesh;
use std::f32::consts::TAU;

// A cube with edges of `size`, with 8 vertices and 6 quads.
pub fn cube(size: f32) -> Mesh {
    let h = size * 0.5;
    let mut mesh = Mesh::new();
    // Around the -Z side, then around the +Z side.
    for position in [[-h, -h, -h], [h, -h, -h], [h, h, -h], [-h, h, -h], [-h, -h, h], [h, -h, h], [h, h, h], [-h, h, h]] {
        mesh.add_vertex(position);
    }
    // -Z, +Z, -Y, +Y, -X and +X.
    for face in [[0, 3, 2, 1], [4, 5, 6, 7], [0, 1, 5, 4], [3, 7, 6, 2], [0, 4, 7, 3], [1, 2, 6, 5]] {
        mesh.add_face(face.to_vec(), 0);
    }
    mesh
}

// A plane on Y = 0, facing +Y, `size` along X and Z, of `segments` quads
// along each.
pub fn plane(size: [f32; 2], segments: [u32; 2]) -> Mesh {
    assert!(segments[0] >= 1 && segments[1] >= 1, "a plane needs at least 1 segment along X and Z");
    let [columns, rows] = segments;
    let mut mesh = Mesh::new();
    for row in 0..=rows {
        for column in 0..=columns {
            mesh.add_vertex([
                size[0] * (column as f32 / columns as f32 - 0.5),
                0.0,
                size[1] * (row as f32 / rows as f32 - 0.5),
            ]);
        }
    }
    let vertex = |column: u32, row: u32| row * (columns + 1) + column;
    for row in 0..rows {
        for column in 0..columns {
            // From -X -Z toward +Z first, which is counterclockwise from +Y.
            mesh.add_face(
                vec![
                    vertex(column, row),
                    vertex(column, row + 1),
                    vertex(column + 1, row + 1),
                    vertex(column + 1, row),
                ],
                0,
            );
        }
    }
    mesh
}

// A cylinder around the Y axis, `height` tall, with `sides` around and
// `segments` quads from its bottom to its top, and caps of `sides` vertices.
pub fn cylinder(radius: f32, height: f32, sides: u32, segments: u32) -> Mesh {
    assert!(sides >= 3 && segments >= 1, "a cylinder needs at least 3 sides and 1 segment");
    frustum(&ring_radii(radius, radius, segments), height, sides)
}

// A cone around the Y axis, `height` tall, from a base of `bottom_radius` to
// a top of `top_radius`, with `sides` around. Where a radius is 0, the sides
// meet at a tip, in triangles, and elsewhere they end in a cap.
pub fn cone(bottom_radius: f32, top_radius: f32, height: f32, sides: u32) -> Mesh {
    assert!(sides >= 3, "a cone needs at least 3 sides");
    assert!(
        bottom_radius >= 0.0 && top_radius >= 0.0 && (bottom_radius > 0.0 || top_radius > 0.0),
        "a cone needs radii of at least 0, and one more than 0"
    );
    frustum(&ring_radii(bottom_radius, top_radius, 1), height, sides)
}

// The radius of each ring from the bottom to the top.
fn ring_radii(bottom: f32, top: f32, segments: u32) -> Vec<f32> {
    (0..=segments).map(|ring| bottom + (top - bottom) * ring as f32 / segments as f32).collect()
}

// Rings of `sides` vertices from the bottom to the top, evenly up `height`,
// with quads between them, and either a cap or a tip at each end: a tip where
// a ring's radius is 0.
fn frustum(radii: &[f32], height: f32, sides: u32) -> Mesh {
    let mut mesh = Mesh::new();
    let rings = radii.len() as u32;
    // A ring of radius 0 at an end is a single vertex, its tip.
    let mut ring_start = Vec::with_capacity(radii.len());
    for (ring, &radius) in radii.iter().enumerate() {
        let y = height * (ring as f32 / (rings - 1) as f32 - 0.5);
        ring_start.push(mesh.positions.len() as u32);
        if radius == 0.0 {
            mesh.add_vertex([0.0, y, 0.0]);
            continue;
        }
        for side in 0..sides {
            let angle = TAU * side as f32 / sides as f32;
            mesh.add_vertex([radius * angle.cos(), y, radius * angle.sin()]);
        }
    }
    let vertex = |ring: usize, side: u32| {
        if radii[ring] == 0.0 { ring_start[ring] } else { ring_start[ring] + side % sides }
    };
    for ring in 0..radii.len() - 1 {
        for side in 0..sides {
            // Around from +X toward +Z, the bottom, then the top: outward.
            let mut face = vec![vertex(ring, side), vertex(ring + 1, side), vertex(ring + 1, side + 1), vertex(ring, side + 1)];
            face.dedup();
            if face.first() == face.last() {
                face.pop();
            }
            mesh.add_face(face, 0);
        }
    }
    // From +X toward +Z faces -Y, which is out of the bottom cap.
    if radii[0] > 0.0 {
        mesh.add_face((0..sides).map(|side| vertex(0, side)).collect(), 0);
    }
    let top = radii.len() - 1;
    if radii[top] > 0.0 {
        mesh.add_face((0..sides).rev().map(|side| vertex(top, side)).collect(), 0);
    }
    mesh
}

// A sphere with its poles on the Y axis, `segments` vertices around each
// ring, and `rings` bands from pole to pole: quads, with triangles around the
// poles.
pub fn uv_sphere(radius: f32, segments: u32, rings: u32) -> Mesh {
    assert!(segments >= 3 && rings >= 2, "a UV sphere needs at least 3 segments and 2 rings");
    let mut mesh = Mesh::new();
    let top = mesh.add_vertex([0.0, radius, 0.0]);
    for ring in 1..rings {
        let latitude = std::f32::consts::PI * ring as f32 / rings as f32;
        for segment in 0..segments {
            let longitude = TAU * segment as f32 / segments as f32;
            mesh.add_vertex([
                radius * latitude.sin() * longitude.cos(),
                radius * latitude.cos(),
                radius * latitude.sin() * longitude.sin(),
            ]);
        }
    }
    let bottom = mesh.add_vertex([0.0, -radius, 0.0]);
    let vertex = |ring: u32, segment: u32| 1 + (ring - 1) * segments + segment % segments;
    for segment in 0..segments {
        // Around from +X toward +Z, the lower ring, then the upper: outward.
        mesh.add_face(vec![vertex(1, segment), top, vertex(1, segment + 1)], 0);
        for ring in 1..rings - 1 {
            mesh.add_face(
                vec![vertex(ring + 1, segment), vertex(ring, segment), vertex(ring, segment + 1), vertex(ring + 1, segment + 1)],
                0,
            );
        }
        mesh.add_face(vec![bottom, vertex(rings - 1, segment), vertex(rings - 1, segment + 1)], 0);
    }
    mesh
}

// A ring around the Y axis, `major_radius` from the axis to the middle of
// its tube, which has `minor_radius`: `major_segments` around the axis and
// `minor_segments` around the tube, all quads.
pub fn torus(major_radius: f32, minor_radius: f32, major_segments: u32, minor_segments: u32) -> Mesh {
    assert!(major_segments >= 3 && minor_segments >= 3, "a torus needs at least 3 segments each way");
    let mut mesh = Mesh::new();
    for major in 0..major_segments {
        let (sin, cos) = (TAU * major as f32 / major_segments as f32).sin_cos();
        for minor in 0..minor_segments {
            let (tube_sin, tube_cos) = (TAU * minor as f32 / minor_segments as f32).sin_cos();
            let distance = major_radius + minor_radius * tube_cos;
            mesh.add_vertex([distance * cos, minor_radius * tube_sin, distance * sin]);
        }
    }
    let vertex = |major: u32, minor: u32| (major % major_segments) * minor_segments + minor % minor_segments;
    for major in 0..major_segments {
        for minor in 0..minor_segments {
            // Around the tube first, then around the axis: outward.
            mesh.add_face(
                vec![vertex(major, minor), vertex(major, minor + 1), vertex(major + 1, minor + 1), vertex(major + 1, minor)],
                0,
            );
        }
    }
    mesh
}
