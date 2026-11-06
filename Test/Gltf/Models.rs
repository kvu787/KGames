// Library/Gltf: reading .glb files made here, with nodes, transforms,
// indices of each size, and missing normals and materials, and the models
// that Veehiicuul reads, which Blender exported.

use std::path::Path;

use gltf::{Model, parse};

// A .glb file with this JSON, and this binary chunk.
fn glb(json: &str, binary: &[u8]) -> Vec<u8> {
    let mut json = json.as_bytes().to_vec();
    while !json.len().is_multiple_of(4) {
        json.push(b' ');
    }
    let mut binary = binary.to_vec();
    while !binary.len().is_multiple_of(4) {
        binary.push(0);
    }
    let length = 12 + 8 + json.len() + 8 + binary.len();
    let mut bytes = Vec::new();
    for word in [0x4654_6C67u32, 2, length as u32, json.len() as u32, 0x4E4F_534A] {
        bytes.extend(word.to_le_bytes());
    }
    bytes.extend(&json);
    bytes.extend((binary.len() as u32).to_le_bytes());
    bytes.extend(0x004E_4942u32.to_le_bytes());
    bytes.extend(&binary);
    bytes
}

fn floats(values: &[f32]) -> Vec<u8> {
    values.iter().flat_map(|value| value.to_le_bytes()).collect()
}

// One triangle in the XY plane, facing +Z, with its positions, normals and
// 16-bit indices, and the nodes and materials that the test gives.
fn triangle(nodes: &str, materials: &str, primitive: &str) -> Vec<u8> {
    let mut binary = floats(&[0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0]);
    binary.extend(floats(&[0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0]));
    binary.extend([0u16, 1, 2].iter().flat_map(|index| index.to_le_bytes()));
    let json = format!(
        r#"{{
            "asset": {{ "version": "2.0" }},
            "scene": 0,
            "scenes": [{{ "nodes": [0] }}],
            "nodes": {nodes},
            "materials": {materials},
            "meshes": [{{ "primitives": [{primitive}] }}],
            "buffers": [{{ "byteLength": {} }}],
            "bufferViews": [
                {{ "buffer": 0, "byteOffset": 0, "byteLength": 36 }},
                {{ "buffer": 0, "byteOffset": 36, "byteLength": 36 }},
                {{ "buffer": 0, "byteOffset": 72, "byteLength": 6 }}
            ],
            "accessors": [
                {{ "bufferView": 0, "componentType": 5126, "count": 3, "type": "VEC3" }},
                {{ "bufferView": 1, "componentType": 5126, "count": 3, "type": "VEC3" }},
                {{ "bufferView": 2, "componentType": 5123, "count": 3, "type": "SCALAR" }}
            ]
        }}"#,
        binary.len()
    );
    glb(&json, &binary)
}

const PRIMITIVE: &str = r#"{ "attributes": { "POSITION": 0, "NORMAL": 1 }, "indices": 2, "material": 0 }"#;
const MATERIALS: &str = r#"[{ "name": "Red", "pbrMetallicRoughness": { "baseColorFactor": [1, 0, 0, 1] }, "extras": { "Shiny": 1 } }]"#;

fn near(actual: [f32; 3], expected: [f32; 3]) -> bool {
    (0..3).all(|axis| (actual[axis] - expected[axis]).abs() < 1e-5)
}

#[test]
fn reads_a_triangle_with_its_material() {
    let model = parse(&triangle(r#"[{ "mesh": 0 }]"#, MATERIALS, PRIMITIVE)).unwrap();
    assert_eq!(model.positions, [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]);
    assert_eq!(model.normals, [[0.0, 0.0, 1.0]; 3]);
    assert_eq!(model.indices, [0, 1, 2]);
    assert_eq!(model.materials, [0, 0, 0]);
    assert_eq!(model.material_list[0].name, "Red");
    assert_eq!(model.material_list[0].base_color, [1.0, 0.0, 0.0, 1.0]);
    assert_eq!(model.material_list[0].extras, Some(json::object([("Shiny", 1.0.into())])));
}

// A child node's transform is its parent's, then its own: here a parent
// moved along X, and a child turned 90 degrees around Y and scaled by 2.
// Its normal turns with it, and keeps its length.
#[test]
fn applies_each_node_s_transform_and_its_parents() {
    let nodes = r#"[
        { "translation": [10, 0, 0], "children": [1] },
        { "mesh": 0, "rotation": [0, 0.70710678, 0, 0.70710678], "scale": [2, 2, 2] }
    ]"#;
    let model = parse(&triangle(nodes, MATERIALS, PRIMITIVE)).unwrap();
    assert!(near(model.positions[1], [10.0, 0.0, -2.0]), "{:?}", model.positions);
    assert!(near(model.positions[2], [10.0, 2.0, 0.0]), "{:?}", model.positions);
    assert!(near(model.normals[0], [1.0, 0.0, 0.0]), "{:?}", model.normals);
    // A scale that mirrors reverses the winding, so the front stays out.
    let mirrored = parse(&triangle(r#"[{ "mesh": 0, "scale": [-1, 1, 1] }]"#, MATERIALS, PRIMITIVE)).unwrap();
    assert_eq!(mirrored.indices, [0, 2, 1]);
    assert!(near(mirrored.normals[0], [0.0, 0.0, 1.0]));
}

// Without normals, each triangle is flat, and without a material, it gets
// one called Default.
#[test]
fn makes_flat_normals_and_a_default_material() {
    let primitive = r#"{ "attributes": { "POSITION": 0 }, "indices": 2 }"#;
    let model = parse(&triangle(r#"[{ "mesh": 0 }]"#, "[]", primitive)).unwrap();
    assert_eq!(model.normals, [[0.0, 0.0, 1.0]; 3]);
    assert_eq!(model.material_list.len(), 1);
    assert_eq!(model.material_list[0].name, "Default");
    assert_eq!(model.material_list[0].base_color, [1.0; 4]);
}

#[test]
fn says_what_is_wrong_with_a_file() {
    let lines = r#"{ "attributes": { "POSITION": 0 }, "mode": 1 }"#;
    for (bytes, problem) in [
        (b"glTF".to_vec(), "is cut short".to_string()),
        (vec![0; 20], "isn't a binary glTF file".to_string()),
        (triangle(r#"[{ "mesh": 0 }]"#, MATERIALS, lines), "has a primitive that isn't triangles".to_string()),
        (
            triangle(r#"[{ "mesh": 0 }]"#, "[]", PRIMITIVE),
            "has no material 0".to_string(),
        ),
        (
            triangle(r#"[{ "mesh": 0, "scale": [0, 1, 1] }]"#, MATERIALS, PRIMITIVE),
            "has a node whose transform flattens its mesh".to_string(),
        ),
        (triangle(r#"[{ "children": [0] }]"#, MATERIALS, PRIMITIVE), "has node 0 in more than one place, or in a loop".to_string()),
    ] {
        assert_eq!(parse(&bytes), Err(problem));
    }
}

fn model(name: &str) -> Model {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../Program/Veehiicuul/Models/{name}.glb"));
    gltf::load(&path).unwrap_or_else(|problem| panic!("{problem}"))
}

// The car, as Blender exported it: one mesh with five materials, 3 units
// wide and about 6 long, facing +Z, with its wheels on the ground.
#[test]
fn reads_the_car_that_blender_exported() {
    let car = model("SlopeCar");
    let names: Vec<&str> = car.material_list.iter().map(|material| material.name.as_str()).collect();
    assert_eq!(names, ["SlopeCarAxles", "SlopeCarBodyBlue", "SlopeCarCabinBlue", "SlopeCarHeadlights", "SlopeCarWheels"]);
    let (min, max) = car.bounds();
    assert!(near(min, [-1.5, 0.0, -3.0]) && near(max, [1.5, 2.3902397, 3.157522]), "{min:?} {max:?}");
    assert_eq!(car.indices.len(), 5652);
    for normal in &car.normals {
        let length = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
        assert!((length - 1.0).abs() < 1e-4);
    }
}

// The cone, whose parts are under one empty node, each moved up from it.
#[test]
fn reads_the_cone_s_parts_where_their_nodes_put_them() {
    let cone = model("BigCone");
    assert_eq!(cone.material_list.len(), 4);
    let (min, max) = cone.bounds();
    assert!(min[1].abs() < 1e-4, "its base is on the ground: {min:?}");
    assert!((5.9..6.1).contains(&max[1]), "{max:?}");
    // The beak is at the front, toward +Z.
    let beak = cone.material_list.iter().position(|material| material.name == "BigConeBeak").unwrap() as u32;
    let front = cone
        .positions
        .iter()
        .zip(&cone.materials)
        .filter(|(_, material)| **material == beak)
        .all(|(position, _)| position[2] > 1.0);
    assert!(front);
}

// A file can say anything about its accessors and nodes, and the reader
// says what's wrong instead of allocating, reading past the chunk, or
// overflowing the stack.
#[test]
fn says_what_is_wrong_with_a_file_s_sizes_and_nodes() {
    let accessors = |accessors: &str| {
        let json = format!(
            r#"{{
                "asset": {{ "version": "2.0" }},
                "scenes": [{{ "nodes": [0] }}],
                "nodes": [{{ "mesh": 0 }}],
                "materials": {MATERIALS},
                "meshes": [{{ "primitives": [{PRIMITIVE}] }}],
                "buffers": [{{ "byteLength": 8 }}],
                "bufferViews": [{{ "buffer": 0, "byteOffset": 0, "byteLength": 8 }}],
                "accessors": {accessors}
            }}"#
        );
        glb(&json, &[0; 8])
    };
    let three = |first: &str| format!(r#"[{first}, {first}, {{ "bufferView": 0, "componentType": 5123, "count": 3, "type": "SCALAR" }}]"#);
    for (accessor, problem) in [
        (r#"{ "componentType": 5126, "count": 4294967295, "type": "VEC3" }"#, "has accessor 0 with 4294967295 elements, more than 16777216"),
        (r#"{ "componentType": 5126, "count": 1e18, "type": "VEC3" }"#, "has accessor 0 without a count"),
        (r#"{ "bufferView": 0, "componentType": 5126, "count": 3, "type": "VEC3" }"#, "has accessor 0 past the end of the binary chunk"),
        (r#"{ "bufferView": 0, "byteOffset": 4294967295, "componentType": 5126, "count": 1, "type": "VEC3" }"#, "has accessor 0 past the end of the binary chunk"),
    ] {
        assert_eq!(parse(&accessors(&three(accessor))).unwrap_err(), problem);
    }
    let strided = r#"[{ "buffer": 0, "byteOffset": 0, "byteLength": 8, "byteStride": 4294967295 }]"#;
    let json = accessors(&three(r#"{ "bufferView": 0, "componentType": 5126, "count": 2, "type": "VEC3" }"#));
    let json = String::from_utf8(json).unwrap().replace(r#"[{ "buffer": 0, "byteOffset": 0, "byteLength": 8 }]"#, strided);
    assert_eq!(parse(&glb(&json[20..json.len() - 16], &[0; 8])).unwrap_err(), "has accessor 0 past the end of the binary chunk");

    // A chain of nodes as long as a stack would be deep.
    let count = 100_000;
    let mut nodes = String::from("[");
    for node in 0..count {
        if node > 0 {
            nodes.push(',');
        }
        if node + 1 < count {
            nodes.push_str(&format!(r#"{{ "children": [{}] }}"#, node + 1));
        } else {
            nodes.push_str(r#"{ "mesh": 0 }"#);
        }
    }
    nodes.push(']');
    let model = parse(&triangle(&nodes, MATERIALS, PRIMITIVE)).unwrap();
    assert_eq!(model.indices.len(), 3);
    let looped = r#"[{ "children": [1] }, { "children": [0], "mesh": 0 }]"#;
    assert_eq!(parse(&triangle(looped, MATERIALS, PRIMITIVE)).unwrap_err(), "has node 0 in more than one place, or in a loop");
}
