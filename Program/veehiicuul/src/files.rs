// What the game's JSON files are made of: numbers, points on the ground,
// colors and headings, read with what's wrong with them, and written back
// the same way, so that a file that's read and written again is unchanged.
//
// Points are [X, Z] on the ground, in units, headings are in degrees from +Z
// toward +X, and colors are sRGB, like "#1CDA61".

use std::path::Path;

use json::Value;

// An sRGB color, a byte a channel, as files write it.
pub type Color = [u8; 3];

// Reads a file's text as JSON, or says what's wrong with it.
pub fn parse(text: &str) -> Result<Value, String> {
    let text = text.strip_prefix('\u{FEFF}').unwrap_or(text);
    json::parse(text).map_err(|error| format!("line {}, column {}: {}", error.line, error.column, error.message))
}

pub fn read(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|error| format!("Couldn't read {}: {error}", path.display()))
}

// Writes a file, and the folders it's in.
pub fn write(path: &Path, text: &str) -> Result<(), String> {
    if let Some(folder) = path.parent() {
        std::fs::create_dir_all(folder).map_err(|error| format!("Couldn't make {}: {error}", folder.display()))?;
    }
    std::fs::write(path, text).map_err(|error| format!("Couldn't write {}: {error}", path.display()))
}

// Fails if `value` isn't an object, or has a member that isn't one of
// `names`, which says what it's part of, like "a track".
pub fn only(value: &Value, names: &[&str], what: &str) -> Result<(), String> {
    let Value::Object(members) = value else {
        return Err(format!("{what} must be an object, not {}", value.describe()));
    };
    match members.iter().find(|(member, _)| !names.contains(&member.as_str())) {
        Some((member, _)) => Err(format!("{member} isn't part of {what}. They're {}", names.join(", "))),
        None => Ok(()),
    }
}

pub fn member<'a>(value: &'a Value, name: &str, path: &str) -> Result<&'a Value, String> {
    value.get(name).ok_or_else(|| format!("needs {path}{name}"))
}

pub fn number(value: &Value, name: &str) -> Result<f64, String> {
    match value {
        Value::Number(number) if number.is_finite() => Ok(*number),
        other => Err(format!("{name} must be a number, not {}", other.describe())),
    }
}

pub fn positive(value: &Value, name: &str) -> Result<f64, String> {
    match number(value, name)? {
        number if number > 0.0 => Ok(number),
        number => Err(format!("{name} must be above 0, not {number}")),
    }
}

pub fn text(value: &Value, name: &str) -> Result<String, String> {
    match value {
        Value::String(text) if !text.trim().is_empty() => Ok(text.clone()),
        other => Err(format!("{name} must be a name, not {}", other.describe())),
    }
}

pub fn point(value: &Value, name: &str) -> Result<[f64; 2], String> {
    match value {
        Value::Array(items) => match items.as_slice() {
            [Value::Number(x), Value::Number(z)] if x.is_finite() && z.is_finite() => Ok([*x, *z]),
            _ => Err(format!("{name} must be a point, [X, Z], not {}", value.describe())),
        },
        other => Err(format!("{name} must be a point, [X, Z], not {}", other.describe())),
    }
}

// A list of at least `fewest` points.
pub fn points(value: &Value, name: &str, fewest: usize) -> Result<Vec<[f64; 2]>, String> {
    let Value::Array(items) = value else {
        return Err(format!("{name} must be a list of points, not {}", value.describe()));
    };
    let points = items
        .iter()
        .enumerate()
        .map(|(index, item)| point(item, &format!("{name}[{index}]")))
        .collect::<Result<Vec<_>, _>>()?;
    if points.len() < fewest {
        return Err(format!("{name} needs at least {fewest} points, not {}", points.len()));
    }
    Ok(points)
}

pub fn color(value: &Value, name: &str) -> Result<Color, String> {
    let parsed = match value {
        Value::String(text) => parse_color(text),
        _ => None,
    };
    parsed.ok_or_else(|| format!("{name} must be a color like \"#1CDA61\", not {}", value.describe()))
}

// A color from text like "#1CDA61".
pub fn parse_color(text: &str) -> Option<Color> {
    if text.len() != 7 || !text.starts_with('#') || !text[1..].chars().all(|character| character.is_ascii_hexdigit()) {
        return None;
    }
    let channel = |index: usize| u8::from_str_radix(&text[index..index + 2], 16).ok();
    Some([channel(1)?, channel(3)?, channel(5)?])
}

pub fn color_text([red, green, blue]: Color) -> String {
    format!("#{red:02X}{green:02X}{blue:02X}")
}

// As sRGB from 0 to 1.
pub fn srgb(color: Color) -> [f32; 3] {
    color.map(|channel| channel as f32 / 255.0)
}

// A number as files write it: to 4 decimal places, a ten-thousandth of a
// unit, so that dragging doesn't write noise.
pub fn rounded(value: f64) -> Value {
    let rounded = (value * 10_000.0).round() / 10_000.0;
    // Not -0.
    (rounded + 0.0).into()
}

pub fn point_value([x, z]: [f64; 2]) -> Value {
    Value::Array(vec![rounded(x), rounded(z)])
}

pub fn points_value(points: &[[f64; 2]]) -> Value {
    Value::Array(points.iter().map(|&point| point_value(point)).collect())
}
