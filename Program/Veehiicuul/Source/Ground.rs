// What's flat on the ground, as meshes that are drawn in order, each part
// over the ones before, without a depth buffer: the terrain's hexels, then
// the road, with its islands cut out of it, so that the terrain shows in
// them, then the barriers beside every edge, away from the road, and the
// start line across the road. The scene's background shows where nothing
// else is. Each part is a flat color, which Ground.hlsl draws.

use meshes::Mesh;
use simple_paint::Vertex;

use crate::files::{Color, srgb};
use crate::terrain::{PALETTE_SIZE, Terrain};
use crate::track::{Shape, Track};

// The meshes' materials, which are Ground.hlsl's colors: the terrain's
// palette, then the track's.
pub const ROAD: u32 = 64;
// The barriers alternate between these.
pub const BARRIERS: [u32; 2] = [65, 66];
// So do the start line's squares.
pub const START_LINE: [u32; 2] = [67, 68];
// As Ground.hlsl's MaterialCount says.
pub const COLOR_COUNT: usize = 72;

// The start line's rows of squares, along the road.
const START_LINE_ROWS: usize = 3;
const BLACK: Color = [0, 0, 0];
const WHITE: Color = [255, 255, 255];

// Every material's color, as sRGB.
pub fn colors(terrain: &Terrain, track: &Track) -> [[f32; 3]; COLOR_COUNT] {
    let mut colors = [[0.0; 3]; COLOR_COUNT];
    for (index, &color) in terrain.palette.iter().take(PALETTE_SIZE).enumerate() {
        colors[index] = srgb(color);
    }
    colors[ROAD as usize] = srgb(track.road_color);
    colors[BARRIERS[0] as usize] = srgb(track.barrier_colors[0]);
    colors[BARRIERS[1] as usize] = srgb(track.barrier_colors[1]);
    colors[START_LINE[0] as usize] = srgb(BLACK);
    colors[START_LINE[1] as usize] = srgb(WHITE);
    colors
}

fn empty() -> Mesh {
    Mesh {
        vertices: Vec::new(),
        indices: Vec::new(),
    }
}

// The terrain's painted hexels, each in its palette color.
pub fn terrain_mesh(terrain: &Terrain) -> Mesh {
    let mut mesh = empty();
    for (&cell, &color) in &terrain.cells {
        let corners = terrain.corners(cell);
        let first = mesh.vertices.len() as u32;
        mesh.vertices.extend(corners.map(|corner| vertex(corner, color as u32)));
        // A fan from the first corner, wound counterclockwise seen from
        // above.
        for [b, c] in [[1, 2], [2, 3], [3, 4], [4, 5]] {
            let clockwise = turn(corners[0], corners[b], corners[c]) > 0.0;
            let [b, c] = if clockwise { [c, b] } else { [b, c] };
            mesh.indices.extend([first, first + b as u32, first + c as u32]);
        }
    }
    mesh
}

// The track: the road, the barriers and the start line.
pub fn track_mesh(track: &Track, shape: &Shape) -> Mesh {
    let mut mesh = empty();
    road(&mut mesh, &shape.road, &shape.islands);
    barriers(&mut mesh, &shape.road, true, track.barrier_width, track.barrier_length);
    for island in &shape.islands {
        barriers(&mut mesh, island, false, track.barrier_width, track.barrier_length);
    }
    start_line(&mut mesh, track, shape);
    mesh
}

fn vertex([x, z]: [f64; 2], material_index: u32) -> Vertex {
    Vertex {
        position: [x as f32, 0.0, z as f32],
        normal: [0.0, 1.0, 0.0],
        material_index,
    }
}

// Twice the signed area of the triangle a, b, c, along X and Z: below 0 if
// they go counterclockwise seen from above, where X is right and -Z up.
fn turn(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> f64 {
    (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
}

// Twice an outline's signed area, like turn's.
pub fn area(outline: &[[f64; 2]]) -> f64 {
    (0..outline.len())
        .map(|index| {
            let [a, b] = [outline[index], outline[(index + 1) % outline.len()]];
            a[0] * b[1] - b[0] * a[1]
        })
        .sum()
}

// A quadrilateral, as two triangles wound counterclockwise seen from above,
// like every triangle of the meshes, the right-handed convention for faces
// that face up.
fn quad(mesh: &mut Mesh, corners: [[f64; 2]; 4], material: u32) {
    let first = mesh.vertices.len() as u32;
    mesh.vertices.extend(corners.map(|corner| vertex(corner, material)));
    for [a, b, c] in [[0, 1, 2], [0, 2, 3]] {
        let clockwise = turn(corners[a], corners[b], corners[c]) > 0.0;
        let [b, c] = if clockwise { [c, b] } else { [b, c] };
        mesh.indices.extend([a, b, c].map(|corner| first + corner as u32));
    }
}

// The road, with its islands cut out.
fn road(mesh: &mut Mesh, outline: &[[f64; 2]], islands: &[Vec<[f64; 2]>]) {
    let polygon = with_holes(outline, islands);
    let first = mesh.vertices.len() as u32;
    mesh.vertices.extend(polygon.iter().map(|&point| vertex(point, ROAD)));
    for triangle in triangulate(&polygon) {
        mesh.indices.extend(triangle.map(|corner| first + corner as u32));
    }
}

// One outline that goes around `outline` and every hole in it, joined to it
// by a cut from each hole's rightmost point to a point of the outline that
// it can see, there and back, so that ear clipping fills all but the holes.
pub fn with_holes(outline: &[[f64; 2]], holes: &[Vec<[f64; 2]>]) -> Vec<[f64; 2]> {
    // The outline counterclockwise seen from above, and the holes the other
    // way, so that the cuts join them into one outline that goes one way.
    let mut polygon: Vec<[f64; 2]> = outline.to_vec();
    if area(&polygon) > 0.0 {
        polygon.reverse();
    }
    let mut holes: Vec<Vec<[f64; 2]>> = holes
        .iter()
        .filter(|hole| hole.len() >= 3)
        .map(|hole| {
            let mut hole = hole.clone();
            if area(&hole) < 0.0 {
                hole.reverse();
            }
            hole
        })
        .collect();
    let rightmost = |hole: &Vec<[f64; 2]>| {
        (0..hole.len()).max_by(|&a, &b| hole[a][0].total_cmp(&hole[b][0])).unwrap()
    };
    // Rightmost first, so that each cut only crosses the outline.
    holes.sort_by(|a, b| b[rightmost(b)][0].total_cmp(&a[rightmost(a)][0]));
    for hole in holes {
        let m = rightmost(&hole);
        let point = hole[m];
        let Some(visible) = visible_point(&polygon, point) else {
            continue;
        };
        let mut joined = polygon[..=visible].to_vec();
        joined.extend(hole[m..].iter().chain(&hole[..=m]));
        joined.push(polygon[visible]);
        joined.extend(&polygon[visible + 1..]);
        polygon = joined;
    }
    polygon
}

// A point of the polygon that `point`, inside it, can see: along a line
// toward +X to the nearest edge, then the end of that edge farther along X,
// or a point inside the triangle between them nearest that line.
fn visible_point(polygon: &[[f64; 2]], point: [f64; 2]) -> Option<usize> {
    let [x, z] = point;
    let mut nearest: Option<(f64, usize)> = None;
    for (index, &a) in polygon.iter().enumerate() {
        let b = polygon[(index + 1) % polygon.len()];
        if (a[1] > z) == (b[1] > z) {
            continue;
        }
        let crossing = a[0] + (z - a[1]) / (b[1] - a[1]) * (b[0] - a[0]);
        if crossing >= x && nearest.is_none_or(|(nearest, _)| crossing < nearest) {
            nearest = Some((crossing, index));
        }
    }
    let (crossing, edge) = nearest?;
    let next = (edge + 1) % polygon.len();
    let candidate = if polygon[edge][0] > polygon[next][0] { edge } else { next };
    let hit = [crossing, z];
    let triangle = [point, hit, polygon[candidate]];
    // A flat triangle, when the line meets the edge at its end, has nothing
    // in it.
    let doubled_area = turn(triangle[0], triangle[1], triangle[2]);
    let sign = doubled_area.signum();
    let inside = |p: [f64; 2]| {
        doubled_area.abs() > 1e-12
            && (0..3).all(|corner| sign * turn(triangle[corner], triangle[(corner + 1) % 3], p) >= 0.0)
    };
    // The point in the triangle at the smallest angle from the line.
    let mut best = candidate;
    let mut best_angle = f64::INFINITY;
    for (index, &p) in polygon.iter().enumerate() {
        if index == candidate || p == polygon[candidate] || !inside(p) {
            continue;
        }
        let angle = (p[1] - z).atan2(p[0] - x).abs();
        if angle < best_angle {
            best = index;
            best_angle = angle;
        }
    }
    // A point that an earlier cut made twice is the one whose corner the
    // cut goes into, so that the outline doesn't cross itself.
    let count = polygon.len();
    let into = (0..count).filter(|&index| polygon[index] == polygon[best]).find(|&index| {
        let [before, corner, after] = [polygon[(index + count - 1) % count], polygon[index], polygon[(index + 1) % count]];
        // The outline goes counterclockwise seen from above, so its inside
        // is where turns are below 0.
        if turn(before, corner, after) < 0.0 {
            turn(before, corner, point) < 0.0 && turn(corner, after, point) < 0.0
        } else {
            turn(before, corner, point) < 0.0 || turn(corner, after, point) < 0.0
        }
    });
    Some(into.unwrap_or(best))
}

// Splits a simple polygon, one whose edges don't cross, into triangles, by
// cutting off its ears one at a time: corners that turn toward its inside,
// whose triangle has none of its other corners in it. Each triangle is its
// corners' indices, wound counterclockwise seen from above. Corners in the
// same place, like the two sides of a cut to a hole, don't count as in a
// triangle.
pub fn triangulate(outline: &[[f64; 2]]) -> Vec<[usize; 3]> {
    let mut corners: Vec<usize> = (0..outline.len()).collect();
    // Counterclockwise, so that corners that turn toward the inside turn
    // below 0.
    if area(outline) > 0.0 {
        corners.reverse();
    }
    let mut triangles = Vec::with_capacity(outline.len().saturating_sub(2));
    let mut index = 0;
    // How many corners in a row weren't ears, to stop at a polygon that has
    // none, which only one whose edges cross can be.
    let mut misses = 0;
    while corners.len() > 3 {
        let count = corners.len();
        let [previous, corner, next] =
            [(index + count - 1) % count, index, (index + 1) % count].map(|position| corners[position]);
        let [a, b, c] = [outline[previous], outline[corner], outline[next]];
        let ear = turn(a, b, c) < 0.0
            && corners.iter().all(|&other| {
                let point = outline[other];
                [previous, corner, next].contains(&other)
                    || point == a
                    || point == b
                    || point == c
                    || !(turn(a, b, point) <= 0.0 && turn(b, c, point) <= 0.0 && turn(c, a, point) <= 0.0)
            });
        if ear || misses > count {
            triangles.push([previous, corner, next]);
            corners.remove(index);
            index = (index + count - 2) % (count - 1);
            misses = 0;
        } else {
            index = (index + 1) % count;
            misses += 1;
        }
    }
    if let [a, b, c] = corners[..] {
        triangles.push([a, b, c]);
    }
    triangles
}

// Barriers along an outline, `width` wide, beside it on the side away from
// the road: outside the road's outline, and inside an island's. They
// alternate between two colors, about `length` long each, an even number of
// them, so that they alternate all the way around.
fn barriers(mesh: &mut Mesh, outline: &[[f64; 2]], outside: bool, width: f64, length: f64) {
    let count = outline.len();
    let point = |index: usize| outline[index % count];
    // Each edge's normal, of length 1, away from the road.
    let sign = if (area(outline) > 0.0) == outside { 1.0 } else { -1.0 };
    let normals: Vec<[f64; 2]> = (0..count)
        .map(|index| {
            let [a, b] = [point(index), point(index + 1)];
            let [x, z] = [b[0] - a[0], b[1] - a[1]];
            let length = x.hypot(z).max(1e-12);
            [sign * z / length, -sign * x / length]
        })
        .collect();
    // Where each point's barrier ends: along the mitered normal of its two
    // edges, so that the barrier is as wide all the way around.
    let offsets: Vec<[f64; 2]> = (0..count)
        .map(|index| {
            let [before, after] = [normals[(index + count - 1) % count], normals[index]];
            let sum = [before[0] + after[0], before[1] + after[1]];
            let length = sum[0].hypot(sum[1]);
            if length < 1e-9 {
                return [after[0] * width, after[1] * width];
            }
            let miter = [sum[0] / length, sum[1] / length];
            let scale = width / (miter[0] * after[0] + miter[1] * after[1]).max(0.25);
            [miter[0] * scale, miter[1] * scale]
        })
        .collect();
    let lengths: Vec<f64> = (0..count)
        .map(|index| {
            let [a, b] = [point(index), point(index + 1)];
            (b[0] - a[0]).hypot(b[1] - a[1])
        })
        .collect();
    let perimeter: f64 = lengths.iter().sum();
    let pieces = (2.0 * (perimeter / (2.0 * length)).round()).max(2.0);
    let mut along = 0.0;
    for index in 0..count {
        // The barrier that the middle of the edge is in.
        let piece = ((along + lengths[index] / 2.0) / perimeter * pieces) as usize;
        along += lengths[index];
        let [a, b] = [point(index), point(index + 1)];
        let [from, to] = [offsets[index], offsets[(index + 1) % count]];
        let corners = [a, [a[0] + from[0], a[1] + from[1]], [b[0] + to[0], b[1] + to[1]], b];
        quad(mesh, corners, BARRIERS[piece % 2]);
    }
}

// The start line: squares of black and white across the road, at a right
// angle to the start's heading.
fn start_line(mesh: &mut Mesh, track: &Track, shape: &Shape) {
    let [sin, cos] = track.start.ahead();
    let ahead = [sin, cos];
    let across = [cos, -sin];
    let line = shape.start_line;
    let square = line.depth / START_LINE_ROWS as f64;
    let columns = ((line.width / square).round() as usize).max(1);
    let size = [line.width / columns as f64, square];
    let place = |column: f64, row: f64| {
        let [x, z] = [column * size[0] - line.width / 2.0, row * size[1] - line.depth / 2.0];
        [
            line.center[0] + across[0] * x + ahead[0] * z,
            line.center[1] + across[1] * x + ahead[1] * z,
        ]
    };
    for row in 0..START_LINE_ROWS {
        for column in 0..columns {
            let [x, z] = [column as f64, row as f64];
            let corners = [place(x, z), place(x + 1.0, z), place(x + 1.0, z + 1.0), place(x, z + 1.0)];
            quad(mesh, corners, START_LINE[(row + column) % 2]);
        }
    }
}
