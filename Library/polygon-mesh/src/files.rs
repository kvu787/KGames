// Meshes in files: JSON, which keeps all of a mesh and reads well, with each
// position, face and crease on its own line, and Wavefront OBJ, for other
// programs, which keeps the faces and their materials:
//
//     {
//       "Positions": [
//         [-1, -1, -1],
//         [1, -1, -1],
//         ...
//       ],
//       "Faces": [
//         [0, 3, 2, 1],
//         ...
//       ],
//       "Materials": [0, 0, 0, 0, 0, 0],
//       "Creases": [
//         [2, 3, 2.5]
//       ]
//     }
//
// Faces are lists of positions' numbers, from 0, counterclockwise when seen
// from the front. Materials has one material for each face, and Creases an
// edge's two vertices and its sharpness for each creased edge. Materials and
// Creases can be left out, for faces of material 0 and no creases.

use crate::{Edge, Face, INFINITELY_SHARP, Mesh};
use json::Value;
use std::fmt::Write;
use std::path::Path;

const NAMES: [&str; 4] = ["Positions", "Faces", "Materials", "Creases"];
const MEMBERS: &str = "Positions, Faces, Materials and Creases";

// The longest line that the materials take before they wrap.
const LINE_LENGTH: usize = 100;

impl Mesh {
    pub fn to_json(&self) -> String {
        let positions = self.positions.iter().map(|position| format!("[{}]", position.map(number).join(", ")));
        let faces = self.faces.iter().map(|face| format!("[{}]", join(&face.vertices, ", ")));
        let creases = self.creases.iter().map(|(edge, &sharpness)| {
            let [a, b] = edge.vertices();
            format!("[{a}, {b}, {}]", number(sharpness))
        });
        let materials: Vec<u32> = self.faces.iter().map(|face| face.material).collect();
        let mut text = String::from("{\n");
        text.push_str(&nested("Positions", positions.collect()));
        text.push_str(",\n");
        text.push_str(&nested("Faces", faces.collect()));
        text.push_str(",\n");
        text.push_str(&wrapped("Materials", &materials));
        text.push_str(",\n");
        text.push_str(&nested("Creases", creases.collect()));
        text.push_str("\n}\n");
        text
    }

    // Reads a mesh from to_json's format, or says what's wrong with it.
    pub fn from_json(text: &str) -> Result<Mesh, String> {
        let value = json::parse(text).map_err(|error| {
            format!("It isn't valid JSON, at line {}, column {}: {}.", error.line, error.column, error.message)
        })?;
        let Value::Object(members) = &value else {
            return Err(format!("A mesh must be an object with {MEMBERS}, not {}.", value.describe()));
        };
        if let Some((name, _)) = members.iter().find(|(name, _)| !NAMES.contains(&name.as_str())) {
            return Err(format!("{name} isn't part of a mesh, which has {MEMBERS}."));
        }
        let list = |name: &str, required: bool| match value.get(name) {
            Some(Value::Array(items)) => Ok(items.as_slice()),
            None if !required => Ok(&[] as &[Value]),
            None => Err(format!("The mesh has no {name}.")),
            Some(other) => Err(format!("{name} must be a list, not {}.", other.describe())),
        };

        let mut mesh = Mesh::new();
        for (index, item) in list("Positions", true)?.iter().enumerate() {
            let position: Option<Vec<f32>> = match item {
                Value::Array(coordinates) if coordinates.len() == 3 => coordinates.iter().map(coordinate).collect(),
                _ => None,
            };
            let Some(&[x, y, z]) = position.as_deref() else {
                return Err(format!(
                    "Positions[{index}] must be a list of 3 numbers that a float can hold, X, Y and Z, not {}.",
                    item.describe()
                ));
            };
            mesh.add_vertex([x, y, z]);
        }
        let count = mesh.positions.len();
        for (index, item) in list("Faces", true)?.iter().enumerate() {
            let Value::Array(items) = item else {
                return Err(format!("Faces[{index}] must be a list of its vertices' numbers, not {}.", item.describe()));
            };
            if items.len() < 3 {
                return Err(format!("Faces[{index}] has {} vertices, but a face needs at least 3.", items.len()));
            }
            let mut vertices = Vec::with_capacity(items.len());
            for item in items {
                let vertex = vertex_number(item, count).map_err(|problem| format!("Faces[{index}] {problem}"))?;
                if vertices.contains(&vertex) {
                    return Err(format!("Faces[{index}] uses vertex {vertex} twice."));
                }
                vertices.push(vertex);
            }
            mesh.add_face(vertices, 0);
        }
        let materials = list("Materials", false)?;
        if value.get("Materials").is_some() && materials.len() != mesh.faces.len() {
            return Err(format!(
                "Materials has {}, but the mesh has {}, and each needs one.",
                plural(materials.len(), "material"),
                plural(mesh.faces.len(), "face")
            ));
        }
        for (index, item) in materials.iter().enumerate() {
            let Some(material) = whole_number(item) else {
                return Err(format!(
                    "Materials[{index}] must be a whole number from 0 to {}, not {}.",
                    u32::MAX,
                    item.describe()
                ));
            };
            mesh.faces[index].material = material;
        }
        let edges = mesh.edges();
        for (index, item) in list("Creases", false)?.iter().enumerate() {
            let shape = || {
                format!(
                    "Creases[{index}] must be a list of an edge's two vertices and its sharpness, like [0, 1, 2.5], \
                     not {}.",
                    item.describe()
                )
            };
            let Value::Array(items) = item else {
                return Err(shape());
            };
            let [a, b, Value::Number(sharpness)] = items.as_slice() else {
                return Err(shape());
            };
            let [a, b] =
                [a, b].map(|item| vertex_number(item, count).map_err(|problem| format!("Creases[{index}] {problem}")));
            let edge = Edge::new(a?, b?);
            let [a, b] = edge.vertices();
            if a == b || !edges.contains(&edge) {
                return Err(format!("Creases[{index}] is from vertex {a} to {b}, but no face has that edge."));
            }
            let fine = *sharpness as f32;
            if !(fine > 0.0 && fine <= INFINITELY_SHARP) {
                return Err(format!(
                    "Creases[{index}]'s sharpness must be more than 0 and at most {INFINITELY_SHARP}, not {sharpness}."
                ));
            }
            if mesh.creases.insert(edge, fine).is_some() {
                return Err(format!("Creases[{index}] creases the edge from {a} to {b} again."));
            }
        }
        mesh.validate()?;
        Ok(mesh)
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        std::fs::write(path, self.to_json()).map_err(|error| format!("Couldn't save {}: {error}", path.display()))
    }

    pub fn load(path: &Path) -> Result<Mesh, String> {
        let text =
            std::fs::read_to_string(path).map_err(|error| format!("Couldn't read {}: {error}", path.display()))?;
        Mesh::from_json(&text).map_err(|problem| format!("{}: {problem}", path.display()))
    }

    // The mesh as Wavefront OBJ: its positions, and its faces, each after a
    // usemtl of its material, Material0 for material 0, wherever the material
    // changes. OBJ has no creases, and counts vertices from 1.
    pub fn to_obj(&self) -> String {
        let mut text = format!(
            "# {} vertices and {} faces, from KGames' PolygonMesh\n",
            self.positions.len(),
            self.faces.len()
        );
        for position in &self.positions {
            let _ = writeln!(text, "v {}", position.map(number).join(" "));
        }
        let mut material = None;
        for Face { vertices, material: face_material } in &self.faces {
            if material != Some(*face_material) {
                let _ = writeln!(text, "usemtl Material{face_material}");
                material = Some(*face_material);
            }
            let vertices: Vec<u32> = vertices.iter().map(|vertex| vertex + 1).collect();
            let _ = writeln!(text, "f {}", join(&vertices, " "));
        }
        text
    }
}

// The float's shortest decimal that reads back as the same float, or null,
// which JSON has instead of infinities and NaN.
fn number(value: f32) -> String {
    if value.is_finite() { value.to_string() } else { "null".to_string() }
}

// Like "1 face" or "2 faces".
fn plural(count: usize, noun: &str) -> String {
    if count == 1 { format!("1 {noun}") } else { format!("{count} {noun}s") }
}

fn join(numbers: &[u32], separator: &str) -> String {
    numbers.iter().map(u32::to_string).collect::<Vec<_>>().join(separator)
}

// A member that's a list with an item on each line.
fn nested(name: &str, items: Vec<String>) -> String {
    if items.is_empty() {
        return format!("  \"{name}\": []");
    }
    format!("  \"{name}\": [\n    {}\n  ]", items.join(",\n    "))
}

// A member that's a list of numbers, on one line if it fits, and wrapped
// otherwise.
fn wrapped(name: &str, numbers: &[u32]) -> String {
    let line = format!("  \"{name}\": [{}]", join(numbers, ", "));
    if line.len() <= LINE_LENGTH {
        return line;
    }
    let mut lines: Vec<String> = Vec::new();
    for number in numbers {
        let item = number.to_string();
        match lines.last_mut() {
            // The indent, the line so far, a comma and a space, and the number,
            // with room for the comma after it.
            Some(last) if 4 + last.len() + 2 + item.len() < LINE_LENGTH => {
                last.push_str(", ");
                last.push_str(&item);
            }
            _ => lines.push(item),
        }
    }
    format!("  \"{name}\": [\n    {}\n  ]", lines.join(",\n    "))
}

// A coordinate, if it's a number that a float can hold.
fn coordinate(value: &Value) -> Option<f32> {
    match value {
        Value::Number(number) if number.abs() <= f32::MAX as f64 => Some(*number as f32),
        _ => None,
    }
}

fn whole_number(value: &Value) -> Option<u32> {
    match value {
        Value::Number(number) if number.fract() == 0.0 && (0.0..=u32::MAX as f64).contains(number) => {
            Some(*number as u32)
        }
        _ => None,
    }
}

// A vertex's number, or what's wrong with it, to follow what it's in.
fn vertex_number(value: &Value, count: usize) -> Result<u32, String> {
    let Some(vertex) = whole_number(value) else {
        return Err(format!("has {}, which isn't a vertex's number.", value.describe()));
    };
    if vertex as usize >= count {
        return Err(format!("uses vertex {vertex}, but there are only {count} positions."));
    }
    Ok(vertex)
}
