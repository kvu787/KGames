// Library/PolygonMesh's files: JSON that reads back exactly, its text, what
// it says about files it can't read, saving and loading, and Wavefront OBJ.

use polygon_mesh::{Edge, Mesh, primitives};
use std::path::Path;

// A triangle of material 2 with one creased edge.
fn triangle() -> Mesh {
    let mut mesh = Mesh::new();
    for position in [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.5]] {
        mesh.add_vertex(position);
    }
    mesh.add_face(vec![0, 1, 2], 2);
    mesh.set_crease(Edge::new(0, 1), 2.5);
    mesh
}

#[test]
fn json_reads_back_exactly() {
    let mut cube = primitives::cube(2.0);
    cube.assign_material([1, 4], 3);
    cube.crease_edges([Edge::new(0, 1), Edge::new(2, 6)], 1.25);
    cube.set_crease(Edge::new(4, 5), 10.0);
    assert_eq!(Mesh::from_json(&cube.to_json()).unwrap(), cube);
    // Positions with every digit that a float has.
    let mut torus = primitives::torus(1.3, 0.37, 12, 7).catmull_clark(1).mesh;
    torus.rotate(0..torus.vertex_count() as u32, [0.1, 0.2, 0.3], [1.0, 2.0, 3.0], 0.7);
    let edge = *torus.edges().first().unwrap();
    torus.set_crease(edge, 0.3);
    assert_eq!(Mesh::from_json(&torus.to_json()).unwrap(), torus);
    assert_eq!(Mesh::from_json(&Mesh::new().to_json()).unwrap(), Mesh::new());
}

#[test]
fn json_has_a_line_for_each_position_face_and_crease() {
    let expected = r#"{
    "Positions": [
        [0, 0, 0],
        [1, 0, 0],
        [0, 1, 0.5]
    ],
    "Faces": [
        [0, 1, 2]
    ],
    "Materials": [2],
    "Creases": [
        [0, 1, 2.5]
    ]
}
"#;
    assert_eq!(triangle().to_json(), expected);
    assert_eq!(
        Mesh::new().to_json(),
        "{\n    \"Positions\": [],\n    \"Faces\": [],\n    \"Materials\": [],\n    \"Creases\": []\n}\n"
    );
    // Materials that don't fit on a line wrap.
    let mut grid = primitives::plane([10.0, 10.0], [10, 10]);
    grid.assign_material(50..100, 7);
    let text = grid.to_json();
    let materials: Vec<&str> = text
        .lines()
        .skip_while(|line| !line.contains("Materials"))
        .take_while(|line| !line.contains("Creases"))
        .collect();
    assert!(materials.len() > 3);
    assert!(materials.iter().all(|line| line.len() <= 100));
    assert_eq!(materials[1], format!("        {},", vec!["0"; 31].join(", ")));
    assert_eq!(Mesh::from_json(&text).unwrap(), grid);
}

#[test]
fn json_can_leave_out_materials_and_creases() {
    let text = r#"{"Positions": [[0, 0, 0], [1, 0, 0], [0, 1, 0]], "Faces": [[0, 1, 2]]}"#;
    let mesh = Mesh::from_json(text).unwrap();
    assert_eq!((mesh.vertex_count(), mesh.face_count(), mesh.faces[0].material), (3, 1, 0));
    assert!(mesh.creases.is_empty());
}

#[test]
fn json_that_isn_t_a_mesh_says_what_s_wrong() {
    let positions = r#""Positions": [[0, 0, 0], [1, 0, 0], [0, 1, 0], [5, 5, 5]]"#;
    let with = |rest: &str| format!("{{{positions}, {rest}}}");
    let cases = [
        ("nonsense".to_string(), "It isn't valid JSON, at line 1, column 1: expected a value."),
        ("[]".to_string(), "A mesh must be an object with Positions, Faces, Materials and Creases, not a list of 0."),
        (format!("{{{positions}}}"), "The mesh has no Faces."),
        (r#"{"Faces": []}"#.to_string(), "The mesh has no Positions."),
        (
            with(r#""Faces": [], "Normals": []"#),
            "Normals isn't part of a mesh, which has Positions, Faces, Materials and Creases.",
        ),
        (r#"{"Positions": {}, "Faces": []}"#.to_string(), "Positions must be a list, not an object."),
        (
            r#"{"Positions": [[0, 0]], "Faces": []}"#.to_string(),
            "Positions[0] must be a list of 3 numbers that a float can hold, X, Y and Z, not a list of 2.",
        ),
        (
            r#"{"Positions": [[0, 0, 1e39]], "Faces": []}"#.to_string(),
            "Positions[0] must be a list of 3 numbers that a float can hold, X, Y and Z, not a list of 3.",
        ),
        (with(r#""Faces": [3]"#), "Faces[0] must be a list of its vertices' numbers, not 3."),
        (with(r#""Faces": [[0, 1]]"#), "Faces[0] has 2 vertices, but a face needs at least 3."),
        (with(r#""Faces": [[0, 1, 2], [0, 1, 4]]"#), "Faces[1] uses vertex 4, but there are only 4 positions."),
        (with(r#""Faces": [[0, 1, 1.5]]"#), "Faces[0] has 1.5, which isn't a vertex's number."),
        (with(r#""Faces": [[0, -1, 2]]"#), "Faces[0] has -1, which isn't a vertex's number."),
        (with(r#""Faces": [[0, 1, 0]]"#), "Faces[0] uses vertex 0 twice."),
        (
            with(r#""Faces": [[0, 1, 2]], "Materials": [0, 1]"#),
            "Materials has 2 materials, but the mesh has 1 face, and each needs one.",
        ),
        (
            with(r#""Faces": [[0, 1, 2]], "Materials": [-1]"#),
            "Materials[0] must be a whole number from 0 to 4294967295, not -1.",
        ),
        (
            with(r#""Faces": [[0, 1, 2]], "Creases": [[0, 1]]"#),
            "Creases[0] must be a list of an edge's two vertices and its sharpness, like [0, 1, 2.5], not a list of 2.",
        ),
        (
            with(r#""Faces": [[0, 1, 2]], "Creases": [[0, 9, 1]]"#),
            "Creases[0] uses vertex 9, but there are only 4 positions.",
        ),
        (
            with(r#""Faces": [[0, 1, 2]], "Creases": [[3, 0, 1]]"#),
            "Creases[0] is from vertex 0 to 3, but no face has that edge.",
        ),
        (
            with(r#""Faces": [[0, 1, 2]], "Creases": [[1, 1, 1]]"#),
            "Creases[0] is from vertex 1 to 1, but no face has that edge.",
        ),
        (
            with(r#""Faces": [[0, 1, 2]], "Creases": [[0, 1, 0]]"#),
            "Creases[0]'s sharpness must be more than 0 and at most 10, not 0.",
        ),
        (
            with(r#""Faces": [[0, 1, 2]], "Creases": [[0, 1, 10.5]]"#),
            "Creases[0]'s sharpness must be more than 0 and at most 10, not 10.5.",
        ),
        (
            with(r#""Faces": [[0, 1, 2]], "Creases": [[0, 1, 1], [1, 0, 2]]"#),
            "Creases[1] creases the edge from 0 to 1 again.",
        ),
    ];
    for (text, expected) in cases {
        assert_eq!(Mesh::from_json(&text).unwrap_err(), expected, "for {text}");
    }
}

#[test]
fn saving_and_loading() {
    let folder = Path::new(env!("CARGO_TARGET_TMPDIR")).join("PolygonMesh");
    std::fs::create_dir_all(&folder).unwrap();
    let path = folder.join("Cube.json");
    let mut cube = primitives::cube(1.0);
    cube.set_crease(Edge::new(0, 1), 3.0);
    cube.save(&path).unwrap();
    assert_eq!(Mesh::load(&path).unwrap(), cube);

    std::fs::write(&path, "{}").unwrap();
    assert_eq!(Mesh::load(&path).unwrap_err(), format!("{}: The mesh has no Positions.", path.display()));
    let missing = folder.join("Missing.json");
    assert!(Mesh::load(&missing).unwrap_err().starts_with(&format!("Couldn't read {}: ", missing.display())));
}

#[test]
fn obj_has_the_positions_and_the_faces_by_material() {
    let mut mesh = Mesh::new();
    for position in [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 1.0, -0.25]] {
        mesh.add_vertex(position);
    }
    mesh.add_face(vec![0, 1, 2], 0);
    mesh.add_face(vec![0, 2, 3], 1);
    mesh.add_face(vec![3, 2, 1, 0], 1);
    mesh.add_face(vec![2, 1, 0], 0);
    let expected = "\
# 4 vertices and 4 faces, from KGames' PolygonMesh
v 0 0 0
v 1 0 0
v 1 1 0
v 0 1 -0.25
usemtl Material0
f 1 2 3
usemtl Material1
f 1 3 4
f 4 3 2 1
usemtl Material0
f 3 2 1
";
    assert_eq!(mesh.to_obj(), expected);
    let cube = primitives::cube(2.0).to_obj();
    assert_eq!(cube.lines().filter(|line| line.starts_with("usemtl")).count(), 1);
    assert_eq!(cube.lines().filter(|line| line.starts_with("f ")).count(), 6);
}
