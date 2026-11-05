// Whether a car touches the track's edges, on the ground: the car is a
// rectangle seen from above, its footprint, and an edge is one of the
// straight segments of the road's outline and its islands'. A car touches an
// edge when a side of its rectangle meets one, even at a point, or along
// it.
//
// Only where the car is in each frame counts, so a car that's fast enough,
// or a frame that's slow enough, can carry it past an edge between frames.

// The rectangle around a car seen from above, along its own directions: X
// to its left, and Z ahead. Its vehicle says how far it goes each way from
// the car's middle.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Footprint {
    pub min: [f64; 2],
    pub max: [f64; 2],
}

impl Footprint {
    // How far it goes to the car's left, right, front and back.
    pub fn from_sides(left: f64, right: f64, front: f64, back: f64) -> Footprint {
        Footprint {
            min: [-right, -back],
            max: [left, front],
        }
    }

    // The rectangle around a model seen from above.
    pub fn of_model(positions: &[[f32; 3]]) -> Footprint {
        let mut min = [f64::INFINITY; 2];
        let mut max = [f64::NEG_INFINITY; 2];
        for &[x, _, z] in positions {
            for (axis, value) in [x, z].into_iter().enumerate() {
                min[axis] = min[axis].min(value as f64);
                max[axis] = max[axis].max(value as f64);
            }
        }
        Footprint { min, max }
    }

    // Its corners on the ground, for a car at `position` facing `heading`,
    // in order around it.
    pub fn corners(&self, position: [f64; 2], heading: f64) -> [[f64; 2]; 4] {
        let (sin, cos) = heading.sin_cos();
        let place = |[left, ahead]: [f64; 2]| {
            [
                position[0] + left * cos + ahead * sin,
                position[1] + ahead * cos - left * sin,
            ]
        };
        let [min, max] = [self.min, self.max];
        [
            place(min),
            place([max[0], min[1]]),
            place(max),
            place([min[0], max[1]]),
        ]
    }
}

// The track's edges: every segment of each outline, which closes from its
// last point back to its first.
pub struct Edges {
    segments: Vec<Segment>,
}

struct Segment {
    ends: [[f64; 2]; 2],
    // The box around it, to skip it quickly.
    min: [f64; 2],
    max: [f64; 2],
}

impl Edges {
    pub fn new(outlines: &[&[[f64; 2]]]) -> Edges {
        let mut segments = Vec::new();
        for outline in outlines {
            for (index, &start) in outline.iter().enumerate() {
                let end = outline[(index + 1) % outline.len()];
                segments.push(Segment {
                    ends: [start, end],
                    min: [start[0].min(end[0]), start[1].min(end[1])],
                    max: [start[0].max(end[0]), start[1].max(end[1])],
                });
            }
        }
        Edges { segments }
    }

    pub fn len(&self) -> usize {
        self.segments.len()
    }

    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    // Whether a car with this footprint, at `position` facing `heading`,
    // touches an edge.
    pub fn touch(&self, footprint: &Footprint, position: [f64; 2], heading: f64) -> bool {
        let corners = footprint.corners(position, heading);
        let mut min = corners[0];
        let mut max = corners[0];
        for corner in &corners[1..] {
            for axis in 0..2 {
                min[axis] = min[axis].min(corner[axis]);
                max[axis] = max[axis].max(corner[axis]);
            }
        }
        self.segments
            .iter()
            .filter(|segment| {
                segment.max[0] >= min[0]
                    && segment.min[0] <= max[0]
                    && segment.max[1] >= min[1]
                    && segment.min[1] <= max[1]
            })
            .any(|segment| {
                (0..4).any(|side| meet(segment.ends, [corners[side], corners[(side + 1) % 4]]))
            })
    }
}

// Twice the signed area of the triangle a, b, c: positive if they turn one
// way, negative the other, and 0 if they're on a line.
fn turn(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> f64 {
    (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
}

// Whether p, on the line through a and b, is between them.
fn within([a, b]: [[f64; 2]; 2], p: [f64; 2]) -> bool {
    (0..2).all(|axis| a[axis].min(b[axis]) <= p[axis] && p[axis] <= a[axis].max(b[axis]))
}

// Whether two segments meet: cross, touch at a point, or overlap along a
// line.
pub fn meet(first: [[f64; 2]; 2], second: [[f64; 2]; 2]) -> bool {
    let [a, b] = first;
    let [c, d] = second;
    let turns = [turn(c, d, a), turn(c, d, b), turn(a, b, c), turn(a, b, d)];
    let opposite = |x: f64, y: f64| (x > 0.0 && y < 0.0) || (x < 0.0 && y > 0.0);
    if opposite(turns[0], turns[1]) && opposite(turns[2], turns[3]) {
        return true;
    }
    (turns[0] == 0.0 && within(second, a))
        || (turns[1] == 0.0 && within(second, b))
        || (turns[2] == 0.0 && within(first, c))
        || (turns[3] == 0.0 && within(first, d))
}
