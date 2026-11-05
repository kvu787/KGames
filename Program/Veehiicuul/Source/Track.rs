// A track, from its file in Tracks: the road's outline and its islands',
// where the car starts, the start line, the camera, and the colors. The
// ground under the road is grass, and Ground.rs makes the rest of what the
// track looks like from the outlines: the road, the islands, the curbs along
// their edges, and the start line. Points are on the ground, [X, Z], in
// units, and headings are in degrees from +Z toward +X.
//
//     {
//         "Start": { "Position": [94.5, -48.5], "Heading": -104.3 },
//         "StartLine": { "Position": [90.5, -49.8], "Width": 12.7, "Depth": 1.5 },
//         "Camera": { "Center": [-3.5, -15.3], "Height": 150 },
//         "FollowHeight": 45,
//         "Colors": { "Grass": "#1CDA61", "Road": "#81858B", "Curbs": ["#F61E22", "#F6FCFF"] },
//         "CurbWidth": 1.5,
//         "CurbLength": 15,
//         "Road": [[-94.6, 36.1], [-94.6, 38.7], ...],
//         "Islands": [[[-79.7, 21.4], ...], ...]
//     }

use std::path::Path;

use json::Value;

use crate::collision::Edges;

// Where a car starts, and which way it faces, in radians from +Z toward +X.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose {
    pub position: [f64; 2],
    pub heading: f64,
}

// Colors, as sRGB, from 0 to 1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Colors {
    pub grass: [f32; 3],
    pub road: [f32; 3],
    // The curbs alternate between these.
    pub curbs: [[f32; 3]; 2],
}

pub struct Track {
    // Its file's name, without ".json".
    pub name: String,
    // The road's outer edge, and its islands' edges, which are grass. Each
    // outline closes from its last point back to its first.
    pub road: Vec<[f64; 2]>,
    pub islands: Vec<Vec<[f64; 2]>>,
    pub start: Pose,
    // The start line goes across the road, at a right angle to the start's
    // heading. It's `width` across and `depth` along the road.
    pub start_line: [f64; 2],
    pub start_line_size: [f64; 2],
    // The fixed camera looks at `camera_center`, at a view `camera_height`
    // units tall, and the camera that follows the car at one
    // `follow_height` tall.
    pub camera_center: [f64; 2],
    pub camera_height: f64,
    pub follow_height: f64,
    pub colors: Colors,
    // How wide the curbs are, on the grass beside each edge, and about how
    // long each one is.
    pub curb_width: f64,
    pub curb_length: f64,
    // Every outline's segments, which the car mustn't touch.
    pub edges: Edges,
}

const MEMBERS: [&str; 9] = [
    "Start",
    "StartLine",
    "Camera",
    "FollowHeight",
    "Colors",
    "CurbWidth",
    "CurbLength",
    "Road",
    "Islands",
];

impl Track {
    // Reads the track called `name` from its file's text, or says what's
    // wrong with it.
    pub fn parse(name: &str, text: &str) -> Result<Track, String> {
        let text = text.strip_prefix('\u{FEFF}').unwrap_or(text);
        let value = json::parse(text).map_err(|error| {
            format!("line {}, column {}: {}", error.line, error.column, error.message)
        })?;
        let Value::Object(members) = &value else {
            return Err(format!("must be an object, not {}", value.describe()));
        };
        if let Some((member, _)) = members.iter().find(|(member, _)| !MEMBERS.contains(&member.as_str())) {
            return Err(format!(
                "{member} isn't part of a track. They're {}",
                MEMBERS.join(", ")
            ));
        }
        let member = |name: &str| value.get(name).ok_or_else(|| format!("needs {name}"));
        let start = member("Start")?;
        let start_line = member("StartLine")?;
        let camera = member("Camera")?;
        let colors = member("Colors")?;
        let curbs = match colors.get("Curbs") {
            Some(Value::Array(items)) if items.len() == 2 => {
                [color(&items[0], "Colors.Curbs")?, color(&items[1], "Colors.Curbs")?]
            }
            _ => return Err("Colors.Curbs must be a list of 2 colors, which the curbs alternate between".to_string()),
        };
        let road = outline(member("Road")?, "Road")?;
        let islands = match member("Islands")? {
            Value::Array(items) => items
                .iter()
                .enumerate()
                .map(|(index, item)| outline(item, &format!("Islands[{index}]")))
                .collect::<Result<Vec<_>, _>>()?,
            other => return Err(format!("Islands must be a list of outlines, not {}", other.describe())),
        };
        let mut outlines: Vec<&[[f64; 2]]> = vec![&road];
        outlines.extend(islands.iter().map(Vec::as_slice));
        let edges = Edges::new(&outlines);
        Ok(Track {
            name: name.to_string(),
            start: Pose {
                position: point(start.get("Position"), "Start.Position")?,
                heading: number(start.get("Heading"), "Start.Heading")?.to_radians(),
            },
            start_line: point(start_line.get("Position"), "StartLine.Position")?,
            start_line_size: [
                positive(start_line.get("Width"), "StartLine.Width")?,
                positive(start_line.get("Depth"), "StartLine.Depth")?,
            ],
            camera_center: point(camera.get("Center"), "Camera.Center")?,
            camera_height: positive(camera.get("Height"), "Camera.Height")?,
            follow_height: positive(value.get("FollowHeight"), "FollowHeight")?,
            colors: Colors {
                grass: color(colors.get("Grass").unwrap_or(&Value::Null), "Colors.Grass")?,
                road: color(colors.get("Road").unwrap_or(&Value::Null), "Colors.Road")?,
                curbs,
            },
            curb_width: positive(value.get("CurbWidth"), "CurbWidth")?,
            curb_length: positive(value.get("CurbLength"), "CurbLength")?,
            road,
            islands,
            edges,
        })
    }

    pub fn load(path: &Path) -> Result<Track, String> {
        let name = path.file_stem().map(|stem| stem.to_string_lossy().into_owned()).unwrap_or_default();
        let text = std::fs::read_to_string(path).map_err(|error| format!("Couldn't read {}: {error}", path.display()))?;
        Track::parse(&name, &text).map_err(|problem| format!("{}: {problem}", path.display()))
    }

    // Every track in a folder, from its .json files, by name.
    pub fn load_all(folder: &Path) -> Result<Vec<Track>, String> {
        let entries = std::fs::read_dir(folder).map_err(|error| format!("Couldn't read {}: {error}", folder.display()))?;
        let mut paths: Vec<_> = entries
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .filter(|path| path.extension().is_some_and(|extension| extension.eq_ignore_ascii_case("json")))
            .collect();
        paths.sort();
        if paths.is_empty() {
            return Err(format!("{} has no tracks", folder.display()));
        }
        paths.iter().map(|path| Track::load(path)).collect()
    }
}

fn number(value: Option<&Value>, name: &str) -> Result<f64, String> {
    match value {
        Some(Value::Number(number)) if number.is_finite() => Ok(*number),
        Some(other) => Err(format!("{name} must be a number, not {}", other.describe())),
        None => Err(format!("needs {name}")),
    }
}

fn positive(value: Option<&Value>, name: &str) -> Result<f64, String> {
    match number(value, name)? {
        number if number > 0.0 => Ok(number),
        number => Err(format!("{name} must be above 0, not {number}")),
    }
}

fn point(value: Option<&Value>, name: &str) -> Result<[f64; 2], String> {
    match value {
        Some(Value::Array(items)) => match items.as_slice() {
            [Value::Number(x), Value::Number(z)] if x.is_finite() && z.is_finite() => Ok([*x, *z]),
            _ => Err(format!("{name} must be a point, [X, Z], not {}", value.unwrap().describe())),
        },
        Some(other) => Err(format!("{name} must be a point, [X, Z], not {}", other.describe())),
        None => Err(format!("needs {name}")),
    }
}

// An outline of at least 3 points, which don't repeat.
fn outline(value: &Value, name: &str) -> Result<Vec<[f64; 2]>, String> {
    let Value::Array(items) = value else {
        return Err(format!("{name} must be a list of points, not {}", value.describe()));
    };
    let points = items
        .iter()
        .enumerate()
        .map(|(index, item)| point(Some(item), &format!("{name}[{index}]")))
        .collect::<Result<Vec<_>, _>>()?;
    if points.len() < 3 {
        return Err(format!("{name} needs at least 3 points, not {}", points.len()));
    }
    for (index, point) in points.iter().enumerate() {
        if points[(index + 1) % points.len()] == *point {
            return Err(format!("{name}[{index}] is the same point as the one after it"));
        }
    }
    Ok(points)
}

// A color like "#1CDA61", as sRGB from 0 to 1.
fn color(value: &Value, name: &str) -> Result<[f32; 3], String> {
    let parsed = match value {
        Value::String(text) if text.len() == 7 && text.is_ascii() && text.starts_with('#') => {
            let channel = |index: usize| u8::from_str_radix(&text[index..index + 2], 16).ok();
            match (channel(1), channel(3), channel(5)) {
                (Some(red), Some(green), Some(blue)) => Some([red, green, blue].map(|channel| channel as f32 / 255.0)),
                _ => None,
            }
        }
        _ => None,
    };
    parsed.ok_or_else(|| format!("{name} must be a color like \"#1CDA61\", not {}", value.describe()))
}
