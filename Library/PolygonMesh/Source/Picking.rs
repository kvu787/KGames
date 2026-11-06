// Picking: the face that a ray from the camera hits, the vertex and edge
// nearest it, and where a mesh or some of its vertices are, to frame them or
// to rotate and scale about.

use crate::mesh::average;
use crate::{Edge, Mesh, SubdivisionSurface, Triangles};
use math::{Vector3, add, cross, dot, length, normalize, scale, subtract};

// How far past a triangle's edges, as a fraction of the triangle, a ray still
// hits it.
const EDGE_TOLERANCE: f32 = 1.0e-5;

// Where a ray hits a face: `distance` along the ray, in the mesh's units, to
// `point`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hit {
    pub face: usize,
    pub distance: f32,
    pub point: Vector3,
}

impl Mesh {
    // The nearest face that the ray from `origin` along `direction` hits,
    // from the front or the back, through the face's triangles, so that a
    // concave face is hit only where it is. None if it hits none, or if
    // `direction` is zero.
    pub fn ray_cast(&self, origin: Vector3, direction: Vector3) -> Option<Hit> {
        let direction = normalize(direction);
        if direction == [0.0; 3] {
            return None;
        }
        let mut nearest: Option<Hit> = None;
        for (index, face) in self.faces.iter().enumerate() {
            for triangle in self.face_triangles(index) {
                let corners = triangle.map(|place| self.position(face.vertices[place]));
                let Some(distance) = ray_triangle(origin, direction, corners) else {
                    continue;
                };
                if nearest.is_none_or(|hit| distance < hit.distance) {
                    nearest = Some(Hit { face: index, distance, point: add(origin, scale(direction, distance)) });
                }
            }
        }
        nearest
    }

    // The vertex nearest the ray, of those within `radius` of it and in front
    // of `origin`, the nearer to `origin` of two as near. A modeler checks
    // whether a face hides it.
    pub fn nearest_vertex(&self, origin: Vector3, direction: Vector3, radius: f32) -> Option<u32> {
        let direction = normalize(direction);
        let mut nearest: Option<(f32, f32, u32)> = None;
        for (vertex, &position) in self.positions.iter().enumerate() {
            let along = dot(subtract(position, origin), direction);
            if along < 0.0 {
                continue;
            }
            let away = length(subtract(position, add(origin, scale(direction, along))));
            if away <= radius && nearest.is_none_or(|(near, depth, _)| (away, along) < (near, depth)) {
                nearest = Some((away, along, vertex as u32));
            }
        }
        nearest.map(|(_, _, vertex)| vertex)
    }

    // The edge nearest the ray, of those within `radius` of it, in front of
    // `origin`, the nearer to `origin` of two as near.
    pub fn nearest_edge(&self, origin: Vector3, direction: Vector3, radius: f32) -> Option<Edge> {
        let direction = normalize(direction);
        let mut nearest: Option<(f32, f32, Edge)> = None;
        for edge in self.edges() {
            let [a, b] = edge.vertices().map(|vertex| self.position(vertex));
            let (away, along) = ray_segment_nearest(origin, direction, a, b);
            if away <= radius && nearest.is_none_or(|(near, depth, _)| (away, along) < (near, depth)) {
                nearest = Some((away, along, edge));
            }
        }
        nearest.map(|(_, _, edge)| edge)
    }

    // The smallest box around every vertex, as its least and greatest
    // corners, or None for a mesh without vertices.
    pub fn bounds(&self) -> Option<[Vector3; 2]> {
        let first = *self.positions.first()?;
        Some(self.positions.iter().fold([first, first], |[low, high], position| {
            [
                std::array::from_fn(|axis| low[axis].min(position[axis])),
                std::array::from_fn(|axis| high[axis].max(position[axis])),
            ]
        }))
    }

    // The average of the vertices' positions, or None for none.
    pub fn center_of(&self, vertices: impl IntoIterator<Item = u32>) -> Option<Vector3> {
        average(self.vertex_set(vertices).into_iter().map(|vertex| self.position(vertex)))
    }
}

impl Triangles {
    // The nearest triangle that the ray hits, as a hit on the face it's of:
    // what Mesh::ray_cast gives on the mesh these are of, without
    // triangulating every face again, for a modeler that asks every frame
    // what's under the pointer, with the triangles it draws.
    pub fn ray_cast(&self, origin: Vector3, direction: Vector3) -> Option<Hit> {
        let direction = normalize(direction);
        if direction == [0.0; 3] {
            return None;
        }
        let mut nearest: Option<Hit> = None;
        for (index, corners) in self.indices.as_chunks::<3>().0.iter().enumerate() {
            let corners = corners.map(|corner| self.vertices[corner as usize].position);
            let Some(distance) = ray_triangle(origin, direction, corners) else {
                continue;
            };
            if nearest.is_none_or(|hit| distance < hit.distance) {
                nearest = Some(Hit { face: self.faces[index], distance, point: add(origin, scale(direction, distance)) });
            }
        }
        nearest
    }
}

impl SubdivisionSurface {
    // Where the ray hits the subdivided mesh, with `face` the base face that
    // the face it hits comes from.
    pub fn ray_cast(&self, origin: Vector3, direction: Vector3) -> Option<Hit> {
        self.mesh.ray_cast(origin, direction).map(|hit| Hit { face: self.base_faces[hit.face], ..hit })
    }
}

// How far along a ray of unit `direction` it hits the triangle, from either
// side, by Moller and Trumbore's method. A little past the triangle's edges
// counts, so that a ray along the edge between two triangles hits one of them
// whatever the rounding.
fn ray_triangle(origin: Vector3, direction: Vector3, [a, b, c]: [Vector3; 3]) -> Option<f32> {
    let first = subtract(b, a);
    let second = subtract(c, a);
    let p = cross(direction, second);
    let determinant = dot(first, p);
    // Along the triangle's plane, or a triangle without area.
    if determinant.abs() <= 1.0e-9 * length(first) * length(second) {
        return None;
    }
    let inverse = 1.0 / determinant;
    let from_a = subtract(origin, a);
    let u = dot(from_a, p) * inverse;
    if !(-EDGE_TOLERANCE..=1.0 + EDGE_TOLERANCE).contains(&u) {
        return None;
    }
    let q = cross(from_a, first);
    let v = dot(direction, q) * inverse;
    if v < -EDGE_TOLERANCE || u + v > 1.0 + EDGE_TOLERANCE {
        return None;
    }
    let distance = dot(second, q) * inverse;
    (distance >= 0.0).then_some(distance)
}

// How near the ray from `origin` along unit `direction` comes to the segment
// from a to b, and how far along the ray, from the nearest points of their
// lines, kept on the ray and the segment.
fn ray_segment_nearest(origin: Vector3, direction: Vector3, a: Vector3, b: Vector3) -> (f32, f32) {
    let along = subtract(b, a);
    let length_squared = dot(along, along);
    let on_segment = |point: Vector3| {
        if length_squared > 0.0 { (dot(subtract(point, a), along) / length_squared).clamp(0.0, 1.0) } else { 0.0 }
    };
    let on_ray = |point: Vector3| dot(subtract(point, origin), direction).max(0.0);
    let from_a = subtract(origin, a);
    let (d, e) = (dot(direction, along), dot(along, from_a));
    let denominator = length_squared - d * d;
    // Parallel lines are nearest everywhere, so from a.
    let s = if denominator > 1.0e-12 * length_squared {
        ((e - d * dot(direction, from_a)) / denominator).clamp(0.0, 1.0)
    } else {
        0.0
    };
    // Kept on the ray, then back on the segment, then on the ray.
    let t = on_ray(add(a, scale(along, s)));
    let s = on_segment(add(origin, scale(direction, t)));
    let t = on_ray(add(a, scale(along, s)));
    (length(subtract(add(origin, scale(direction, t)), add(a, scale(along, s)))), t)
}
