// Triangles for SimplePaint to draw: each face cut into triangles by ear
// clipping in its plane, so that concave faces work, with normals flat per
// face, or smooth across the edges where faces meet at up to an angle, like
// Blender's Auto Smooth. What comes out passes simple_paint::validate_mesh for
// any mesh with positions SimplePaint accepts: triangles without area are left
// out, and a triangle whose smooth normals fail SimplePaint's check that they
// share a cone gets its own flat normal instead.

use crate::Mesh;
use crate::mesh::normalize_or_zero;
use math::{Vector3, cross, dot, subtract};
use simple_paint::Vertex;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Shading {
    // Each face's own normal.
    Flat,
    // Normals smooth across each edge between two faces that meet at up to
    // `angle` radians, and split at the others, at creased edges, of any
    // sharpness, and at boundary and non-manifold edges. An angle of PI
    // smooths every edge that isn't creased.
    Smooth { angle: f32 },
}

// An indexed triangle list for simple_paint::Vertex, wound counterclockwise
// when seen from the front, like the faces, and the face that each triangle
// comes from, for picking and for drawing what's selected.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Triangles {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub faces: Vec<usize>,
}

// What validate_mesh needs of each normal: projecting onto the normalized sum
// of the three at least 0.125. This asks a little more, so that rounding
// can't fail what passes here.
const CONE_PROJECTION: f64 = 0.13;

impl Mesh {
    pub fn triangulate(&self, shading: Shading) -> Triangles {
        let normals: Vec<Vector3> = (0..self.faces.len()).map(|face| self.face_normal(face)).collect();
        let mut first_corner = Vec::with_capacity(self.faces.len());
        let mut corners = 0;
        for face in &self.faces {
            first_corner.push(corners);
            corners += face.vertices.len();
        }
        // Each corner's normal, and the fan of corners that it shares a
        // vertex with: just itself when flat.
        let (corner_normals, fans) = match shading {
            Shading::Flat => {
                let corner_normals = (0..self.faces.len())
                    .flat_map(|face| std::iter::repeat_n(normals[face], self.faces[face].vertices.len()))
                    .collect();
                (corner_normals, (0..corners).collect())
            }
            Shading::Smooth { angle } => {
                let fans = self.smooth_fans(&normals, &first_corner, corners, angle);
                let mut sums = vec![[0.0f64; 3]; corners];
                for (index, face) in self.faces.iter().enumerate() {
                    for place in 0..face.vertices.len() {
                        let weight = self.corner_angle(index, place) as f64;
                        let sum = &mut sums[fans[first_corner[index] + place]];
                        for axis in 0..3 {
                            sum[axis] += normals[index][axis] as f64 * weight;
                        }
                    }
                }
                let corner_normals: Vec<Vector3> =
                    fans.iter().map(|&fan| normalize_or_zero(sums[fan].map(|s| s as f32))).collect();
                (corner_normals, fans)
            }
        };
        // Vertices are made as triangles first use them, so that every one
        // is used, which validate_mesh needs of them, and shared by a fan's
        // corners of the same material.
        let mut shared: HashMap<(usize, u32), u32> = HashMap::new();
        let mut triangles = Triangles::default();
        for (index, face) in self.faces.iter().enumerate() {
            for triangle in self.face_triangles(index) {
                let positions = triangle.map(|place| self.position(face.vertices[place]));
                let [a, b, c] = positions;
                let flat = normalize_or_zero(cross(subtract(b, a), subtract(c, a)));
                if flat == [0.0; 3] {
                    continue;
                }
                let corners = triangle.map(|place| first_corner[index] + place);
                if share_cone(corners.map(|corner| corner_normals[corner])) {
                    for (corner, position) in corners.into_iter().zip(positions) {
                        let vertex = *shared.entry((fans[corner], face.material)).or_insert_with(|| {
                            let normal = corner_normals[corner];
                            triangles.vertices.push(Vertex { position, normal, material_index: face.material });
                            triangles.vertices.len() as u32 - 1
                        });
                        triangles.indices.push(vertex);
                    }
                } else {
                    // A face without area of its own, or smooth normals too far
                    // apart: the triangle's own flat normal.
                    for position in positions {
                        triangles.indices.push(triangles.vertices.len() as u32);
                        triangles.vertices.push(Vertex { position, normal: flat, material_index: face.material });
                    }
                }
                triangles.faces.push(index);
            }
        }
        triangles
    }

    // The fan that each corner is in, by the number of one of its corners:
    // corners at a vertex share a fan when they're joined through smooth
    // edges.
    fn smooth_fans(&self, normals: &[Vector3], first_corner: &[usize], corners: usize, angle: f32) -> Vec<usize> {
        let mut fans: Vec<usize> = (0..corners).collect();
        fn root(fans: &mut [usize], mut corner: usize) -> usize {
            while fans[corner] != corner {
                fans[corner] = fans[fans[corner]];
                corner = fans[corner];
            }
            corner
        }
        let limit = angle.cos() - 1.0e-6;
        let topology = self.topology();
        for edge in 0..topology.edges().len() {
            let &[first, second] = topology.edge_faces(edge) else {
                continue;
            };
            let smooth = !topology.is_non_manifold(edge)
                && self.crease(topology.edge(edge)) == 0.0
                && normals[first] != [0.0; 3]
                && normals[second] != [0.0; 3]
                && dot(normals[first], normals[second]) >= limit;
            if !smooth {
                continue;
            }
            for vertex in topology.edge(edge).vertices() {
                let corner = |face: usize| {
                    first_corner[face] + self.faces[face].vertices.iter().position(|&other| other == vertex).unwrap()
                };
                let (a, b) = (root(&mut fans, corner(first)), root(&mut fans, corner(second)));
                fans[a] = b;
            }
        }
        (0..corners).map(|corner| root(&mut fans, corner)).collect()
    }

    // The angle of a face's corner, in radians, which weighs its normal in
    // the smooth normal of the corner's vertex, as Blender weighs it.
    fn corner_angle(&self, face: usize, place: usize) -> f32 {
        let vertices = &self.faces[face].vertices;
        let count = vertices.len();
        let position = self.position(vertices[place]);
        let previous = normalize_or_zero(subtract(self.position(vertices[(place + count - 1) % count]), position));
        let next = normalize_or_zero(subtract(self.position(vertices[(place + 1) % count]), position));
        dot(previous, next).clamp(-1.0, 1.0).acos()
    }

    // A face's triangles, as places in its list of vertices, by ear clipping
    // in the plane of its normal, so that concave faces work, wound like the
    // face: n - 2 of them for a face of n vertices. A face without a normal,
    // or one that crosses itself, is cut as well as it can be.
    pub fn face_triangles(&self, face: usize) -> Vec<[usize; 3]> {
        let vertices = &self.faces[face].vertices;
        let normal = self.face_area_vector(face);
        if vertices.len() == 3 {
            return vec![[0, 1, 2]];
        }
        // Axes in the plane whose cross product is the normal, so that the
        // face is counterclockwise in them.
        let u = normalize_or_zero(if normal[0].abs() > normal[1].abs() {
            [-normal[2], 0.0, normal[0]]
        } else {
            [0.0, normal[2], -normal[1]]
        });
        let v = normalize_or_zero(cross(normal, u));
        let points: Vec<[f64; 2]> = vertices
            .iter()
            .map(|&vertex| {
                let position = self.position(vertex);
                [dot(position, u) as f64, dot(position, v) as f64]
            })
            .collect();
        ear_clip(&points)
    }
}

// Whether SimplePaint's interpolation between the normals stays away from
// zero, as validate_mesh checks it, in the same arithmetic.
fn share_cone(normals: [Vector3; 3]) -> bool {
    let sum: [f64; 3] = std::array::from_fn(|axis| normals.iter().map(|normal| normal[axis] as f64).sum());
    let length = sum.iter().map(|s| s * s).sum::<f64>().sqrt();
    length > 0.0
        && normals.iter().all(|normal| {
            let projection: f64 = (0..3).map(|axis| normal[axis] as f64 * sum[axis]).sum();
            projection >= CONE_PROJECTION * length
        })
}

// Twice the signed area of the triangle a, b, c: positive when it's
// counterclockwise.
fn turn(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> f64 {
    (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
}

// The triangles of a counterclockwise polygon, as indices into its points.
// Each step cuts off an ear: a corner that turns left, with no other point
// in or on its triangle. A polygon with no ear, which only one that's
// degenerate or crosses itself can be, loses the corner that turns left most.
fn ear_clip(points: &[[f64; 2]]) -> Vec<[usize; 3]> {
    let mut remaining: Vec<usize> = (0..points.len()).collect();
    let mut triangles = Vec::with_capacity(points.len().saturating_sub(2));
    while remaining.len() > 3 {
        let count = remaining.len();
        let corner = |place: usize| {
            [remaining[(place + count - 1) % count], remaining[place], remaining[(place + 1) % count]]
        };
        let is_ear = |place: usize| {
            let [a, b, c] = corner(place).map(|index| points[index]);
            turn(a, b, c) > 0.0
                && remaining.iter().all(|&other| {
                    let p = points[other];
                    [a, b, c].contains(&p) || turn(a, b, p) < 0.0 || turn(b, c, p) < 0.0 || turn(c, a, p) < 0.0
                })
        };
        let place = (0..count).find(|&place| is_ear(place)).unwrap_or_else(|| {
            (0..count)
                .max_by(|&first, &second| {
                    let [a, b, c] = corner(first).map(|index| points[index]);
                    let [d, e, f] = corner(second).map(|index| points[index]);
                    turn(a, b, c).total_cmp(&turn(d, e, f))
                })
                .unwrap()
        });
        triangles.push(corner(place));
        remaining.remove(place);
    }
    if remaining.len() == 3 {
        triangles.push([remaining[0], remaining[1], remaining[2]]);
    }
    triangles
}
