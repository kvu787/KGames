// What a track looks like, as one flat mesh on the ground that's drawn in
// order, each part over the ones before, without a depth buffer: the road,
// the islands' grass on it, the curbs on the grass beside every edge, and
// the start line across the road. The grass around the road is the frame's
// background. Each part is a flat color, which Ground.hlsl draws.

use meshes::Mesh;
use simple_paint::Vertex;

use crate::track::Track;

// The mesh's materials, which are Ground.hlsl's colors.
pub const ROAD: u32 = 0;
pub const GRASS: u32 = 1;
// The curbs alternate between these.
pub const CURBS: [u32; 2] = [2, 3];
// So do the start line's squares.
pub const START_LINE: [u32; 2] = [4, 5];
pub const MATERIAL_COUNT: usize = 6;

// The start line's rows of squares, along the road.
const START_LINE_ROWS: usize = 3;
const BLACK: [f32; 3] = [0.0; 3];
const WHITE: [f32; 3] = [1.0; 3];

// The materials' colors, as sRGB.
pub fn colors(track: &Track) -> [[f32; 3]; MATERIAL_COUNT] {
    let [first, second] = track.colors.curbs;
    [track.colors.road, track.colors.grass, first, second, BLACK, WHITE]
}

pub fn mesh(track: &Track) -> Mesh {
    let mut ground = Mesh {
        vertices: Vec::new(),
        indices: Vec::new(),
    };
    polygon(&mut ground, &track.road, ROAD);
    for island in &track.islands {
        polygon(&mut ground, island, GRASS);
    }
    curbs(&mut ground, &track.road, true, track.curb_width, track.curb_length);
    for island in &track.islands {
        curbs(&mut ground, island, false, track.curb_width, track.curb_length);
    }
    start_line(&mut ground, track);
    ground
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
fn area(outline: &[[f64; 2]]) -> f64 {
    (0..outline.len())
        .map(|index| {
            let [a, b] = [outline[index], outline[(index + 1) % outline.len()]];
            a[0] * b[1] - b[0] * a[1]
        })
        .sum()
}

// A quadrilateral, as two triangles wound counterclockwise seen from above,
// like every triangle of the mesh, the right-handed convention for faces
// that face up.
fn quad(ground: &mut Mesh, corners: [[f64; 2]; 4], material: u32) {
    let first = ground.vertices.len() as u32;
    ground.vertices.extend(corners.map(|corner| vertex(corner, material)));
    for [a, b, c] in [[0, 1, 2], [0, 2, 3]] {
        let clockwise = turn(corners[a], corners[b], corners[c]) > 0.0;
        let [b, c] = if clockwise { [c, b] } else { [b, c] };
        ground.indices.extend([a, b, c].map(|corner| first + corner as u32));
    }
}

// A polygon filled with a material.
fn polygon(ground: &mut Mesh, outline: &[[f64; 2]], material: u32) {
    let first = ground.vertices.len() as u32;
    ground.vertices.extend(outline.iter().map(|&point| vertex(point, material)));
    for triangle in triangulate(outline) {
        ground.indices.extend(triangle.map(|corner| first + corner as u32));
    }
}

// Splits a simple polygon, one whose edges don't cross, into triangles, by
// cutting off its ears one at a time: corners that turn toward its inside,
// whose triangle has none of its other corners in it. Each triangle is its
// corners' indices, wound counterclockwise seen from above.
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

// Curbs along an outline, `width` wide, on the grass beside it: outside the
// road's outline, and inside an island's. They alternate between two
// colors, about `length` long each, an even number of them, so that they
// alternate all the way around.
fn curbs(ground: &mut Mesh, outline: &[[f64; 2]], outside: bool, width: f64, length: f64) {
    let count = outline.len();
    let point = |index: usize| outline[index % count];
    // Each edge's normal, of length 1, toward the grass.
    let sign = if (area(outline) > 0.0) == outside { 1.0 } else { -1.0 };
    let normals: Vec<[f64; 2]> = (0..count)
        .map(|index| {
            let [a, b] = [point(index), point(index + 1)];
            let [x, z] = [b[0] - a[0], b[1] - a[1]];
            let length = x.hypot(z);
            [sign * z / length, -sign * x / length]
        })
        .collect();
    // Where each point's curb ends on the grass: along the mitered normal
    // of its two edges, so that the curb is as wide all the way around.
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
        // The curb that the middle of the edge is in.
        let piece = ((along + lengths[index] / 2.0) / perimeter * pieces) as usize;
        along += lengths[index];
        let [a, b] = [point(index), point(index + 1)];
        let [from, to] = [offsets[index], offsets[(index + 1) % count]];
        let corners = [a, [a[0] + from[0], a[1] + from[1]], [b[0] + to[0], b[1] + to[1]], b];
        quad(ground, corners, CURBS[piece % 2]);
    }
}

// The start line: squares of black and white across the road, at the start
// line's place, at a right angle to the start's heading.
fn start_line(ground: &mut Mesh, track: &Track) {
    let (sin, cos) = track.start.heading.sin_cos();
    let ahead = [sin, cos];
    let across = [cos, -sin];
    let [width, depth] = track.start_line_size;
    let square = depth / START_LINE_ROWS as f64;
    let columns = ((width / square).round() as usize).max(1);
    let size = [width / columns as f64, square];
    let center = track.start_line;
    let place = |column: f64, row: f64| {
        let [x, z] = [column * size[0] - width / 2.0, row * size[1] - depth / 2.0];
        [
            center[0] + across[0] * x + ahead[0] * z,
            center[1] + across[1] * x + ahead[1] * z,
        ]
    };
    for row in 0..START_LINE_ROWS {
        for column in 0..columns {
            let [x, z] = [column as f64, row as f64];
            let corners = [place(x, z), place(x + 1.0, z), place(x + 1.0, z + 1.0), place(x, z + 1.0)];
            quad(ground, corners, START_LINE[(row + column) % 2]);
        }
    }
}
