// Reads models from binary glTF files, .glb, like the ones that Blender
// exports with File > Export > glTF 2.0: every mesh of the default scene, as
// one list of triangles, with each node's transform applied, and the
// materials that its triangles use.
//
// glTF is Y up, right-handed, with a model's front toward +Z, as Blender
// exports it. Only what KGames uses is read: positions, normals, triangle
// indices, materials' names, base colors and extras, and the nodes' trees
// and transforms. Texture coordinates, textures, animations, skins, cameras
// and lights are ignored. A primitive that isn't triangles, or data in
// another file, is an error.

use json::Value;

// A material, as glTF says it: its name, its base color as linear RGBA, and
// its extras, which Blender fills with the material's custom properties.
#[derive(Clone, Debug, PartialEq)]
pub struct Material {
    pub name: String,
    pub base_color: [f32; 4],
    pub extras: Option<Value>,
}

// A model's triangles: each vertex's position, normal, and the index of its
// triangle's material. Triangles are wound counterclockwise when seen from
// their front, as glTF's are.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Model {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub materials: Vec<u32>,
    pub indices: Vec<u32>,
    // Each material's index is its place here. A primitive without one gets
    // one called "Default", white.
    pub material_list: Vec<Material>,
}

impl Model {
    // The box around its positions: the smallest and the largest of each
    // coordinate.
    pub fn bounds(&self) -> ([f32; 3], [f32; 3]) {
        let mut min = [f32::INFINITY; 3];
        let mut max = [f32::NEG_INFINITY; 3];
        for position in &self.positions {
            for axis in 0..3 {
                min[axis] = min[axis].min(position[axis]);
                max[axis] = max[axis].max(position[axis]);
            }
        }
        (min, max)
    }
}

const MAGIC: u32 = 0x4654_6C67;
const JSON_CHUNK: u32 = 0x4E4F_534A;
const BINARY_CHUNK: u32 = 0x004E_4942;

pub fn load(path: &std::path::Path) -> Result<Model, String> {
    let bytes = std::fs::read(path).map_err(|error| format!("Couldn't read {}: {error}", path.display()))?;
    parse(&bytes).map_err(|problem| format!("{}: {problem}", path.display()))
}

// Reads a .glb file's bytes.
pub fn parse(bytes: &[u8]) -> Result<Model, String> {
    let word = |offset: usize| -> Result<u32, String> {
        bytes
            .get(offset..offset + 4)
            .map(|word| u32::from_le_bytes(word.try_into().unwrap()))
            .ok_or_else(|| "is cut short".to_string())
    };
    if word(0)? != MAGIC {
        return Err("isn't a binary glTF file".to_string());
    }
    if word(4)? != 2 {
        return Err(format!("is glTF version {}, not 2", word(4)?));
    }
    let length = (word(8)? as usize).min(bytes.len());
    let mut offset = 12;
    let mut text = None;
    let mut binary: &[u8] = &[];
    while offset + 8 <= length {
        let chunk_length = word(offset)? as usize;
        let kind = word(offset + 4)?;
        let data = bytes
            .get(offset + 8..offset + 8 + chunk_length)
            .ok_or_else(|| "has a chunk that's cut short".to_string())?;
        match kind {
            JSON_CHUNK if text.is_none() => {
                text = Some(std::str::from_utf8(data).map_err(|_| "has JSON that isn't UTF-8".to_string())?)
            }
            BINARY_CHUNK if binary.is_empty() => binary = data,
            _ => {}
        }
        offset += 8 + chunk_length.next_multiple_of(4);
    }
    let text = text.ok_or_else(|| "has no JSON chunk".to_string())?;
    let document = json::parse(text).map_err(|error| {
        format!("has JSON with a mistake at line {}, column {}: {}", error.line, error.column, error.message)
    })?;
    read(&document, binary)
}

// The member `name` of `value`, as a list, or an empty one.
fn list<'a>(value: &'a Value, name: &str) -> &'a [Value] {
    match value.get(name) {
        Some(Value::Array(items)) => items,
        _ => &[],
    }
}

fn index(value: &Value, name: &str) -> Option<usize> {
    match value.get(name) {
        Some(Value::Number(number)) if *number >= 0.0 && number.fract() == 0.0 => Some(*number as usize),
        _ => None,
    }
}

fn numbers<const N: usize>(value: Option<&Value>, default: [f64; N]) -> Result<[f64; N], String> {
    match value {
        None => Ok(default),
        Some(Value::Array(items)) if items.len() == N => {
            let mut result = default;
            for (slot, item) in result.iter_mut().zip(items) {
                match item {
                    Value::Number(number) if number.is_finite() => *slot = *number,
                    _ => return Err(format!("has a list of {N} numbers with {}", item.describe())),
                }
            }
            Ok(result)
        }
        Some(other) => Err(format!("has {} where a list of {N} numbers should be", other.describe())),
    }
}

// A 4 by 4 matrix, as glTF stores them: column by column.
type Matrix = [f64; 16];

const IDENTITY: Matrix = [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0];

fn multiply(a: &Matrix, b: &Matrix) -> Matrix {
    let mut result = [0.0; 16];
    for column in 0..4 {
        for row in 0..4 {
            result[column * 4 + row] = (0..4).map(|k| a[k * 4 + row] * b[column * 4 + k]).sum();
        }
    }
    result
}

// A node's own transform, from its matrix, or its translation, rotation and
// scale.
fn local_transform(node: &Value) -> Result<Matrix, String> {
    if node.get("matrix").is_some() {
        return numbers::<16>(node.get("matrix"), IDENTITY);
    }
    let [tx, ty, tz] = numbers(node.get("translation"), [0.0; 3])?;
    let [x, y, z, w] = numbers(node.get("rotation"), [0.0, 0.0, 0.0, 1.0])?;
    let [sx, sy, sz] = numbers(node.get("scale"), [1.0; 3])?;
    // The rotation's columns, from its quaternion.
    let rotation = [
        [1.0 - 2.0 * (y * y + z * z), 2.0 * (x * y + z * w), 2.0 * (x * z - y * w)],
        [2.0 * (x * y - z * w), 1.0 - 2.0 * (x * x + z * z), 2.0 * (y * z + x * w)],
        [2.0 * (x * z + y * w), 2.0 * (y * z - x * w), 1.0 - 2.0 * (x * x + y * y)],
    ];
    let scale = [sx, sy, sz];
    let mut matrix = IDENTITY;
    for column in 0..3 {
        for row in 0..3 {
            matrix[column * 4 + row] = rotation[column][row] * scale[column];
        }
    }
    matrix[12] = tx;
    matrix[13] = ty;
    matrix[14] = tz;
    Ok(matrix)
}

// An accessor's elements, each `N` numbers, as f64.
fn accessor<const N: usize>(document: &Value, binary: &[u8], which: usize) -> Result<Vec<[f64; N]>, String> {
    let accessor = list(document, "accessors")
        .get(which)
        .ok_or_else(|| format!("has no accessor {which}"))?;
    let components = match accessor.get("type") {
        Some(Value::String(kind)) => match kind.as_str() {
            "SCALAR" => 1,
            "VEC2" => 2,
            "VEC3" => 3,
            "VEC4" => 4,
            "MAT4" => 16,
            _ => 0,
        },
        _ => 0,
    };
    if components != N {
        return Err(format!("has accessor {which} with {components} numbers an element, not {N}"));
    }
    if accessor.get("sparse").is_some() {
        return Err(format!("has accessor {which}, which is sparse, which isn't supported"));
    }
    let count = index_of(accessor, "count", which)?;
    let component_type = index_of(accessor, "componentType", which)?;
    let normalized = matches!(accessor.get("normalized"), Some(Value::Bool(true)));
    let size = match component_type {
        5120 | 5121 => 1,
        5122 | 5123 => 2,
        5125 | 5126 => 4,
        other => return Err(format!("has accessor {which} of component type {other}")),
    };
    let Some(view_index) = index(accessor, "bufferView") else {
        // Without a buffer view, every element is zero.
        return Ok(vec![[0.0; N]; count]);
    };
    let view = list(document, "bufferViews")
        .get(view_index)
        .ok_or_else(|| format!("has no buffer view {view_index}"))?;
    if index(view, "buffer") != Some(0) {
        return Err(format!("has buffer view {view_index} outside the file's binary chunk"));
    }
    let start = index(view, "byteOffset").unwrap_or(0) + index(accessor, "byteOffset").unwrap_or(0);
    let stride = index(view, "byteStride").unwrap_or(size * N);
    let mut elements = Vec::with_capacity(count);
    for element in 0..count {
        let mut values = [0.0; N];
        for (component, value) in values.iter_mut().enumerate() {
            let at = start + element * stride + component * size;
            let bytes = binary
                .get(at..at + size)
                .ok_or_else(|| format!("has accessor {which} past the end of the binary chunk"))?;
            *value = match component_type {
                5120 => {
                    let raw = bytes[0] as i8 as f64;
                    if normalized { (raw / 127.0).max(-1.0) } else { raw }
                }
                5121 => {
                    let raw = bytes[0] as f64;
                    if normalized { raw / 255.0 } else { raw }
                }
                5122 => {
                    let raw = i16::from_le_bytes([bytes[0], bytes[1]]) as f64;
                    if normalized { (raw / 32767.0).max(-1.0) } else { raw }
                }
                5123 => {
                    let raw = u16::from_le_bytes([bytes[0], bytes[1]]) as f64;
                    if normalized { raw / 65535.0 } else { raw }
                }
                5125 => u32::from_le_bytes(bytes.try_into().unwrap()) as f64,
                _ => f32::from_le_bytes(bytes.try_into().unwrap()) as f64,
            };
        }
        elements.push(values);
    }
    Ok(elements)
}

fn index_of(value: &Value, name: &str, accessor: usize) -> Result<usize, String> {
    index(value, name).ok_or_else(|| format!("has accessor {accessor} without a {name}"))
}

fn read(document: &Value, binary: &[u8]) -> Result<Model, String> {
    let mut model = Model::default();
    for material in list(document, "materials") {
        let name = match material.get("name") {
            Some(Value::String(name)) => name.clone(),
            _ => format!("Material {}", model.material_list.len()),
        };
        let pbr = material.get("pbrMetallicRoughness");
        let color = numbers(pbr.and_then(|pbr| pbr.get("baseColorFactor")), [1.0; 4])
            .map_err(|problem| format!("material {name} {problem}"))?;
        model.material_list.push(Material {
            name,
            base_color: color.map(|channel| channel as f32),
            extras: material.get("extras").cloned(),
        });
    }
    let scenes = list(document, "scenes");
    let scene = index(document, "scene").unwrap_or(0);
    let roots: Vec<usize> = match scenes.get(scene) {
        Some(scene) => list(scene, "nodes").iter().filter_map(|node| match node {
            Value::Number(number) => Some(*number as usize),
            _ => None,
        })
        .collect(),
        // Without scenes, every node that's no other's child is a root.
        None => {
            let nodes = list(document, "nodes");
            let children: Vec<usize> = nodes
                .iter()
                .flat_map(|node| list(node, "children").iter().filter_map(|child| match child {
                    Value::Number(number) => Some(*number as usize),
                    _ => None,
                }))
                .collect();
            (0..nodes.len()).filter(|node| !children.contains(node)).collect()
        }
    };
    let mut visited = Vec::new();
    for root in roots {
        add_node(document, binary, root, &IDENTITY, &mut model, &mut visited)?;
    }
    if model.indices.is_empty() {
        return Err("has no triangles".to_string());
    }
    Ok(model)
}

fn add_node(
    document: &Value,
    binary: &[u8],
    index_in_file: usize,
    parent: &Matrix,
    model: &mut Model,
    visited: &mut Vec<usize>,
) -> Result<(), String> {
    if visited.contains(&index_in_file) {
        return Err(format!("has node {index_in_file} in more than one place, or in a loop"));
    }
    visited.push(index_in_file);
    let node = list(document, "nodes")
        .get(index_in_file)
        .ok_or_else(|| format!("has no node {index_in_file}"))?;
    let transform = multiply(parent, &local_transform(node).map_err(|problem| format!("node {index_in_file} {problem}"))?);
    if let Some(mesh) = index(node, "mesh") {
        let mesh = list(document, "meshes").get(mesh).ok_or_else(|| format!("has no mesh {mesh}"))?;
        for primitive in list(mesh, "primitives") {
            add_primitive(document, binary, primitive, &transform, model)?;
        }
    }
    for child in list(node, "children") {
        if let Value::Number(child) = child {
            add_node(document, binary, *child as usize, &transform, model, visited)?;
        }
    }
    Ok(())
}

fn add_primitive(
    document: &Value,
    binary: &[u8],
    primitive: &Value,
    transform: &Matrix,
    model: &mut Model,
) -> Result<(), String> {
    if index(primitive, "mode").is_some_and(|mode| mode != 4) {
        return Err("has a primitive that isn't triangles".to_string());
    }
    let attributes = primitive.get("attributes").ok_or_else(|| "has a primitive without attributes".to_string())?;
    let position_accessor = index(attributes, "POSITION").ok_or_else(|| "has a primitive without positions".to_string())?;
    let positions = accessor::<3>(document, binary, position_accessor)?;
    let normals = match index(attributes, "NORMAL") {
        Some(normal_accessor) => Some(accessor::<3>(document, binary, normal_accessor)?),
        None => None,
    };
    let indices: Vec<usize> = match index(primitive, "indices") {
        Some(indices) => accessor::<1>(document, binary, indices)?.into_iter().map(|[index]| index as usize).collect(),
        None => (0..positions.len()).collect(),
    };
    if !indices.len().is_multiple_of(3) || indices.iter().any(|&index| index >= positions.len()) {
        return Err("has a primitive whose indices aren't whole triangles of its vertices".to_string());
    }
    let material = match index(primitive, "material") {
        Some(material) if material < model.material_list.len() => material as u32,
        Some(material) => return Err(format!("has no material {material}")),
        None => default_material(model),
    };

    // Positions go through the transform, and normals through its inverse
    // transpose, which keeps them at right angles to the surface under any
    // scale. A transform that mirrors turns triangles inside out, so their
    // winding is reversed to keep their fronts out.
    let linear = [
        [transform[0], transform[1], transform[2]],
        [transform[4], transform[5], transform[6]],
        [transform[8], transform[9], transform[10]],
    ];
    let place = |[x, y, z]: [f64; 3]| -> [f32; 3] {
        let mut result = [transform[12], transform[13], transform[14]];
        for row in 0..3 {
            result[row] += linear[0][row] * x + linear[1][row] * y + linear[2][row] * z;
        }
        result.map(|value| value as f32)
    };
    let determinant = determinant(&linear);
    if determinant.abs() < 1e-12 {
        return Err("has a node whose transform flattens its mesh".to_string());
    }
    let normal_matrix = inverse_transpose(&linear, determinant);
    let turn = |[x, y, z]: [f64; 3]| -> [f32; 3] {
        let mut result = [0.0; 3];
        for row in 0..3 {
            result[row] = normal_matrix[0][row] * x + normal_matrix[1][row] * y + normal_matrix[2][row] * z;
        }
        let length = (result[0] * result[0] + result[1] * result[1] + result[2] * result[2]).sqrt();
        if length > 0.0 {
            result.map(|value| (value / length) as f32)
        } else {
            [0.0, 1.0, 0.0]
        }
    };
    let mirrored = determinant < 0.0;
    let first = model.positions.len() as u32;
    match normals {
        Some(normals) => {
            if normals.len() != positions.len() {
                return Err("has a primitive with a different number of normals and positions".to_string());
            }
            model.positions.extend(positions.iter().map(|&position| place(position)));
            model.normals.extend(normals.iter().map(|&normal| turn(normal)));
            model.materials.extend(std::iter::repeat_n(material, positions.len()));
            for triangle in indices.chunks(3) {
                let [a, b, c] = [triangle[0], triangle[1], triangle[2]].map(|index| first + index as u32);
                model.indices.extend(if mirrored { [a, c, b] } else { [a, b, c] });
            }
        }
        // Without normals, each triangle is flat, with its own vertices.
        None => {
            for triangle in indices.chunks(3) {
                let mut corners = [triangle[0], triangle[1], triangle[2]].map(|index| place(positions[index]));
                if mirrored {
                    corners.swap(1, 2);
                }
                let normal = face_normal(corners);
                let base = model.positions.len() as u32;
                model.positions.extend(corners);
                model.normals.extend([normal; 3]);
                model.materials.extend([material; 3]);
                model.indices.extend([base, base + 1, base + 2]);
            }
        }
    }
    Ok(())
}

fn default_material(model: &mut Model) -> u32 {
    if let Some(index) = model.material_list.iter().position(|material| material.name == "Default") {
        return index as u32;
    }
    model.material_list.push(Material {
        name: "Default".to_string(),
        base_color: [1.0; 4],
        extras: None,
    });
    (model.material_list.len() - 1) as u32
}

fn determinant(m: &[[f64; 3]; 3]) -> f64 {
    m[0][0] * (m[1][1] * m[2][2] - m[2][1] * m[1][2]) - m[1][0] * (m[0][1] * m[2][2] - m[2][1] * m[0][2])
        + m[2][0] * (m[0][1] * m[1][2] - m[1][1] * m[0][2])
}

// The inverse transpose of a matrix stored column by column, as columns.
fn inverse_transpose(m: &[[f64; 3]; 3], determinant: f64) -> [[f64; 3]; 3] {
    // The cofactor matrix is the inverse's transpose, times the determinant.
    let cofactor = |column: usize, row: usize| {
        let (c1, c2) = ((column + 1) % 3, (column + 2) % 3);
        let (r1, r2) = ((row + 1) % 3, (row + 2) % 3);
        m[c1][r1] * m[c2][r2] - m[c2][r1] * m[c1][r2]
    };
    let mut result = [[0.0; 3]; 3];
    for (column, values) in result.iter_mut().enumerate() {
        for (row, value) in values.iter_mut().enumerate() {
            *value = cofactor(column, row) / determinant;
        }
    }
    result
}

fn face_normal([a, b, c]: [[f32; 3]; 3]) -> [f32; 3] {
    let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let normal = [u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0]];
    let length = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
    if length > 0.0 {
        normal.map(|value| value / length)
    } else {
        [0.0, 1.0, 0.0]
    }
}
