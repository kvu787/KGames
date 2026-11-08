// A scene's track: the curves of its edges, where the car starts, and its
// colors. The road's curve is its outer edge, and each island's curve is
// the edge of a patch of ground inside the road, which isn't road. Their
// shape, from the curves, is what the car mustn't touch, and Ground.rs makes
// the road, its barriers and the start line from it.
//
//     "Track": {
//         "Start": { "Position": [94.5, -48.5], "Heading": -104.3 },
//         "Colors": { "Road": "#81858B", "Barriers": ["#F61E22", "#F6FCFF"] },
//         "BarrierWidth": 1.5,
//         "BarrierLength": 15,
//         "Road": [[-94.6, 36.1], ...],
//         "Islands": [[[-79.7, 21.4], ...], ...]
//     }

use json::Value;

use crate::collision::{Edges, meet};
use crate::curve;
use crate::files::{self, Color};

// Where a car starts, or a thing is put: its place on the ground, and which
// way it faces, in degrees from +Z toward +X.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose {
    pub position: [f64; 2],
    pub heading: f64,
}

impl Pose {
    pub fn radians(&self) -> f64 {
        self.heading.to_radians()
    }

    // The way it faces, on the ground.
    pub fn ahead(&self) -> [f64; 2] {
        let (sin, cos) = self.radians().sin_cos();
        [sin, cos]
    }

    pub fn parse(value: &Value, name: &str) -> Result<Pose, String> {
        files::only(value, &["Position", "Heading"], name)?;
        Ok(Pose {
            position: files::point(files::member(value, "Position", &format!("{name}."))?, &format!("{name}.Position"))?,
            heading: files::number(files::member(value, "Heading", &format!("{name}."))?, &format!("{name}.Heading"))?,
        })
    }

    pub fn to_value(&self) -> Value {
        json::object([("Position", files::point_value(self.position)), ("Heading", files::rounded(self.heading))])
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Track {
    // The control points of the road's outer edge, and of each island's.
    pub road: Vec<[f64; 2]>,
    pub islands: Vec<Vec<[f64; 2]>>,
    pub start: Pose,
    pub road_color: Color,
    // The barriers alternate between these.
    pub barrier_colors: [Color; 2],
    // How wide the barriers are, on the ground beside each edge, and about
    // how long each one is.
    pub barrier_width: f64,
    pub barrier_length: f64,
}

// How far ahead of the car's start the start line is, and how deep it is
// along the road.
pub const START_LINE_AHEAD: f64 = 4.25;
pub const START_LINE_DEPTH: f64 = 1.5;
// How wide the start line is where it can't find the road's edges.
const START_LINE_WIDTH: f64 = 10.0;

const MEMBERS: [&str; 7] = ["Start", "Colors", "BarrierWidth", "BarrierLength", "Road", "Islands", ""];

impl Track {
    // A track with a round road and no islands, around the origin.
    pub fn round() -> Track {
        Track {
            road: curve::circle([0.0, 0.0], 40.0, 12),
            islands: vec![curve::circle([0.0, 0.0], 25.0, 10)],
            start: Pose {
                position: [32.5, 0.0],
                heading: 180.0,
            },
            road_color: [0x81, 0x85, 0x8B],
            barrier_colors: [[0xF6, 0x1E, 0x22], [0xF6, 0xFC, 0xFF]],
            barrier_width: 1.5,
            barrier_length: 15.0,
        }
    }

    pub fn parse(value: &Value) -> Result<Track, String> {
        files::only(value, &MEMBERS[..6], "the track")?;
        let member = |name: &str| files::member(value, name, "Track.");
        let colors = member("Colors")?;
        files::only(colors, &["Road", "Barriers"], "Track.Colors")?;
        let barriers = match files::member(colors, "Barriers", "Track.Colors.")? {
            Value::Array(items) if items.len() == 2 => [
                files::color(&items[0], "Track.Colors.Barriers[0]")?,
                files::color(&items[1], "Track.Colors.Barriers[1]")?,
            ],
            _ => return Err("Track.Colors.Barriers must be a list of 2 colors, which the barriers alternate between".to_string()),
        };
        let islands = match member("Islands")? {
            Value::Array(items) => items
                .iter()
                .enumerate()
                .map(|(index, item)| files::points(item, &format!("Track.Islands[{index}]"), curve::FEWEST))
                .collect::<Result<Vec<_>, _>>()?,
            other => return Err(format!("Track.Islands must be a list of curves, not {}", other.describe())),
        };
        Ok(Track {
            road: files::points(member("Road")?, "Track.Road", curve::FEWEST)?,
            islands,
            start: Pose::parse(member("Start")?, "Track.Start")?,
            road_color: files::color(files::member(colors, "Road", "Track.Colors.")?, "Track.Colors.Road")?,
            barrier_colors: barriers,
            barrier_width: files::positive(member("BarrierWidth")?, "Track.BarrierWidth")?,
            barrier_length: files::positive(member("BarrierLength")?, "Track.BarrierLength")?,
        })
    }

    pub fn to_value(&self) -> Value {
        json::object([
            ("Start", self.start.to_value()),
            (
                "Colors",
                json::object([
                    ("Road", files::color_text(self.road_color).into()),
                    (
                        "Barriers",
                        Value::Array(self.barrier_colors.iter().map(|&color| files::color_text(color).into()).collect()),
                    ),
                ]),
            ),
            ("BarrierWidth", files::rounded(self.barrier_width)),
            ("BarrierLength", files::rounded(self.barrier_length)),
            ("Road", files::points_value(&self.road)),
            ("Islands", Value::Array(self.islands.iter().map(|island| files::points_value(island)).collect())),
        ])
    }

    // Every curve, the road's first, then the islands', by index.
    pub fn curves(&self) -> impl Iterator<Item = &Vec<[f64; 2]>> {
        std::iter::once(&self.road).chain(&self.islands)
    }

    pub fn curve_mut(&mut self, index: usize) -> Option<&mut Vec<[f64; 2]>> {
        if index == 0 { Some(&mut self.road) } else { self.islands.get_mut(index - 1) }
    }

    pub fn shape(&self) -> Shape {
        let road = curve::sample(&self.road);
        let islands: Vec<Vec<[f64; 2]>> = self.islands.iter().map(|island| curve::sample(island)).collect();
        let mut outlines: Vec<&[[f64; 2]]> = vec![&road];
        outlines.extend(islands.iter().map(Vec::as_slice));
        let edges = Edges::new(&outlines);
        let start_line = start_line(&outlines, &self.start);
        let problems = problems(&road, &islands, &self.start, &edges);
        Shape {
            road,
            islands,
            edges,
            start_line,
            problems,
        }
    }
}

// The start line, across the road ahead of the start.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StartLine {
    pub center: [f64; 2],
    // Across the road, and along it.
    pub width: f64,
    pub depth: f64,
}

// What a track's curves make: the outlines of its road and its islands,
// their edges, the start line, and what's wrong with them, if anything.
pub struct Shape {
    pub road: Vec<[f64; 2]>,
    pub islands: Vec<Vec<[f64; 2]>>,
    pub edges: Edges,
    pub start_line: StartLine,
    pub problems: Vec<String>,
}

// The start line goes across the road from edge to edge, START_LINE_AHEAD of
// the start: as far as the nearest edge on either side of its middle.
fn start_line(outlines: &[&[[f64; 2]]], start: &Pose) -> StartLine {
    let [sin, cos] = start.ahead();
    let center = [
        start.position[0] + sin * START_LINE_AHEAD,
        start.position[1] + cos * START_LINE_AHEAD,
    ];
    let across = [cos, -sin];
    let reach = |direction: f64| {
        let far = [center[0] + across[0] * direction * 1000.0, center[1] + across[1] * direction * 1000.0];
        let mut nearest: Option<f64> = None;
        for outline in outlines {
            for (index, &a) in outline.iter().enumerate() {
                let b = outline[(index + 1) % outline.len()];
                if let Some(distance) = crossing(center, far, a, b) {
                    nearest = Some(nearest.map_or(distance, |nearest| nearest.min(distance)));
                }
            }
        }
        nearest
    };
    let width = match (reach(1.0), reach(-1.0)) {
        (Some(left), Some(right)) => left + right,
        _ => START_LINE_WIDTH,
    };
    // The middle moves to halfway between the edges, so the line spans the
    // road.
    let shift = match (reach(1.0), reach(-1.0)) {
        (Some(left), Some(right)) => (left - right) / 2.0,
        _ => 0.0,
    };
    StartLine {
        center: [center[0] + across[0] * shift, center[1] + across[1] * shift],
        width,
        depth: START_LINE_DEPTH,
    }
}

// How far along from `from` to `to` the segment a to b crosses it, in units,
// if it does.
fn crossing(from: [f64; 2], to: [f64; 2], a: [f64; 2], b: [f64; 2]) -> Option<f64> {
    let direction = [to[0] - from[0], to[1] - from[1]];
    let edge = [b[0] - a[0], b[1] - a[1]];
    let denominator = direction[0] * edge[1] - direction[1] * edge[0];
    if denominator.abs() < 1e-12 {
        return None;
    }
    let offset = [a[0] - from[0], a[1] - from[1]];
    let t = (offset[0] * edge[1] - offset[1] * edge[0]) / denominator;
    let s = (offset[0] * direction[1] - offset[1] * direction[0]) / denominator;
    ((0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&s)).then(|| t * direction[0].hypot(direction[1]))
}

// Whether a point is inside an outline, by how many of its edges a line
// from it toward +X crosses.
pub fn inside(outline: &[[f64; 2]], [x, z]: [f64; 2]) -> bool {
    let mut inside = false;
    for (index, &a) in outline.iter().enumerate() {
        let b = outline[(index + 1) % outline.len()];
        if (a[1] > z) != (b[1] > z) && x < a[0] + (z - a[1]) / (b[1] - a[1]) * (b[0] - a[0]) {
            inside = !inside;
        }
    }
    inside
}

// What's wrong with a track's shape, for the track builder to say: edges
// that cross, islands outside the road or inside each other, and a start
// that isn't on the road.
fn problems(road: &[[f64; 2]], islands: &[Vec<[f64; 2]>], start: &Pose, edges: &Edges) -> Vec<String> {
    let mut problems = Vec::new();
    let mut outlines: Vec<(&str, usize, &[[f64; 2]])> = vec![("The road", 0, road)];
    outlines.extend(islands.iter().enumerate().map(|(index, island)| ("Island", index + 1, island.as_slice())));
    let name = |(kind, index, _): &(&str, usize, &[[f64; 2]])| {
        if *index == 0 { kind.to_string() } else { format!("{kind} {index}") }
    };
    for (first, a) in outlines.iter().enumerate() {
        for (second, b) in outlines.iter().enumerate().skip(first) {
            if crosses(a.2, b.2, first == second) {
                problems.push(if first == second {
                    format!("{}'s edge crosses itself", name(a))
                } else {
                    format!("{}'s edge crosses {}'s", name(a), name(b).to_lowercase())
                });
            }
        }
    }
    for island in &outlines[1..] {
        if !inside(road, island.2[0]) {
            problems.push(format!("{} is outside the road", name(island)));
        }
    }
    // An island is inside another if its first point is, whichever of the
    // two comes first in the list. One point only decides it where their
    // edges don't cross, which is a problem of its own, above.
    for outer in &outlines[1..] {
        for inner in &outlines[1..] {
            if inner.1 != outer.1 && inside(outer.2, inner.2[0]) {
                problems.push(format!("{} is inside {}", name(inner), name(outer).to_lowercase()));
            }
        }
    }
    let on_road = inside(road, start.position) && islands.iter().all(|island| !inside(island, start.position));
    if !on_road {
        problems.push("The start isn't on the road".to_string());
    }
    let _ = edges;
    problems
}

// Whether two outlines' edges cross, or, for one outline, whether edges that
// aren't next to each other do.
fn crosses(a: &[[f64; 2]], b: &[[f64; 2]], same: bool) -> bool {
    let segment = |outline: &[[f64; 2]], index: usize| [outline[index], outline[(index + 1) % outline.len()]];
    for i in 0..a.len() {
        let first = segment(a, i);
        let (min, max) = bounds(first);
        for j in (if same { i + 2 } else { 0 })..b.len() {
            if same && i == 0 && j == b.len() - 1 {
                continue;
            }
            let second = segment(b, j);
            let (other_min, other_max) = bounds(second);
            if other_max[0] < min[0] || other_min[0] > max[0] || other_max[1] < min[1] || other_min[1] > max[1] {
                continue;
            }
            if meet(first, second) {
                return true;
            }
        }
    }
    false
}

fn bounds([a, b]: [[f64; 2]; 2]) -> ([f64; 2], [f64; 2]) {
    ([a[0].min(b[0]), a[1].min(b[1])], [a[0].max(b[0]), a[1].max(b[1])])
}
