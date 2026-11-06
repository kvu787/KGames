// MeshModeler's files: a document's JSON keeps its mesh and paints; Save as,
// Open and New go through their dialogs, which ask before changes are lost;
// and files that are wrong say what's wrong with them.

use std::path::{Path, PathBuf};

use mesh_modeler::document::{self, Document};
use mesh_modeler::modeler::Modeler;
use mesh_modeler::paint::{MAXIMUM_PAINTS, Paint};
use polygon_mesh::{Edge, primitives};
use user_interface::{Headless, Key, Modifiers};

fn folder(test: &str) -> PathBuf {
    let folder = Path::new(env!("CARGO_TARGET_TMPDIR")).join("MeshModelerFiles").join(test);
    let _ = std::fs::remove_dir_all(&folder);
    std::fs::create_dir_all(&folder).unwrap();
    folder
}

#[test]
fn a_document_keeps_its_mesh_creases_and_paints_through_json() {
    let mut document = Document::new();
    document.mesh = primitives::torus(1.0, 0.25, 12, 8);
    document.mesh.assign_material([3, 4], 2);
    document.mesh.set_crease(Edge::new(0, 1), 2.5);
    document.paints[1].set_color(user_interface::Color::hex(0x336699));
    document.paints[1].parameters.shift = 0.25;
    document.paints[1].name = "Steel".to_string();
    let text = document.to_json();
    let read = Document::from_json(&text, "Torus").unwrap();
    assert_eq!(read.mesh, document.mesh);
    assert_eq!(read.paints.len(), document.paints.len());
    assert_eq!(read.paints[1].name, "Steel");
    assert!((read.paints[1].parameters.shift - 0.25).abs() < 1e-9);
    assert_eq!(read.paints[1].color().to_hex(), document.paints[1].color().to_hex());
    assert_eq!(read.name, "Torus");
    assert!(!read.dirty);
}

#[test]
fn wrong_files_say_what_s_wrong() {
    let error = |text: &str| Document::from_json(text, "Wrong").err().unwrap();
    assert!(error("{").contains("isn't valid JSON"));
    assert!(error("{}").contains("no Mesh"));
    let cube = Document::new().to_json();
    let no_paints = cube.replace("\"Paints\"", "\"Unused\"");
    assert_eq!(Document::from_json(&no_paints, "Cube").unwrap().paints.len(), 4);
    let mut many = Document::new();
    many.paints = (0..=MAXIMUM_PAINTS).map(|index| Paint::new(&format!("P{index}"), 0x808080)).collect();
    assert!(error(&many.to_json()).contains("from 1 to 16 paints"));
    let mut painted = Document::new();
    painted.mesh.faces[2].material = 7;
    assert!(error(&painted.to_json()).contains("Face 2 has paint 7"));
}

#[test]
fn names_are_checked_for_what_windows_forbids() {
    for good in ["Cube", "My model 2", "a.b"] {
        assert!(document::valid_name(good), "{good}");
    }
    for bad in ["", "   ", "a/b", "a:b", "what?", "dot.", "a\tb"] {
        assert!(!document::valid_name(bad), "{bad:?}");
    }
}

#[test]
fn save_as_open_and_new_go_through_their_dialogs() {
    let folder = folder("Dialogs");
    let mut app = Headless::new(Modeler::new(folder.clone()));
    app.frames(2);
    // An untitled model asks for a name.
    app.key(Key::A);
    app.key_with(Modifiers::CONTROL, Key::S);
    assert_eq!(app.program.prompt.map(|prompt| prompt.name()), Some("SaveAs"));
    // A name that can't be a file's says so, and stays open.
    app.program.name_draft = "a/b".to_string();
    app.click("SaveAs/Primary");
    assert!(app.program.prompt_error.as_deref().unwrap().contains("can't be empty"));
    assert_eq!(app.program.prompt.map(|prompt| prompt.name()), Some("SaveAs"));
    app.program.name_draft = "Box".to_string();
    app.click("SaveAs/Primary");
    assert!(app.program.prompt.is_none());
    assert!(folder.join("Box.json").is_file());
    assert_eq!(app.program.document.name, "Box");
    assert!(!app.program.document.dirty);

    // Ctrl+S saves under its name, without asking.
    app.key_with(Modifiers::ALT, Key::A);
    let center = app.program.view.center();
    app.click_point(center);
    app.key(Key::E);
    app.key(Key::ENTER);
    assert!(app.program.document.dirty);
    app.key_with(Modifiers::CONTROL, Key::S);
    assert!(app.program.prompt.is_none());
    assert!(!app.program.document.dirty);
    let saved = Document::open(&folder, "Box").unwrap();
    assert_eq!(saved.mesh.faces.len(), 10);

    // New with changes asks first; Cancel keeps them.
    app.key_with(Modifiers::ALT, Key::A);
    app.key_with(Modifiers::CONTROL, Key::N);
    assert_eq!(app.program.prompt.map(|prompt| prompt.name()), Some("Unsaved"));
    app.click("Unsaved/Close");
    assert_eq!(app.program.document.name, "Box");
    app.key_with(Modifiers::CONTROL, Key::N);
    app.click("Unsaved/Secondary");
    assert_eq!(app.program.document.name, document::UNTITLED);
    assert_eq!(app.program.document.mesh.faces.len(), 6);

    // Open lists the saved models, and opens the one picked.
    app.key_with(Modifiers::CONTROL, Key::O);
    assert_eq!(app.program.prompt.map(|prompt| prompt.name()), Some("OpenModel"));
    assert_eq!(app.program.saved_names, vec!["Box".to_string()]);
    app.click("OpenModel/Primary");
    assert_eq!(app.program.document.name, "Box");
    assert_eq!(app.program.document.mesh.faces.len(), 10);
    assert!(app.program.prompt.is_none());

    // Export writes an OBJ file beside the model.
    app.key_with(Modifiers::CONTROL, Key::E);
    let obj = std::fs::read_to_string(folder.join("Box.obj")).unwrap();
    assert_eq!(obj.lines().filter(|line| line.starts_with("f ")).count(), 10);
}

#[test]
fn saving_an_untitled_model_from_the_unsaved_dialog_asks_for_its_name_then_goes_on() {
    let folder = folder("SaveThenNew");
    let mut app = Headless::new(Modeler::new(folder.clone()));
    app.frames(2);
    app.key(Key::A);
    app.key_with(Modifiers::CONTROL, Key::N);
    app.click("Unsaved/Primary");
    assert_eq!(app.program.prompt.map(|prompt| prompt.name()), Some("SaveAs"));
    app.program.name_draft = "First".to_string();
    app.click("SaveAs/Primary");
    // Saved, then new, as asked.
    assert!(folder.join("First.json").is_file());
    assert_eq!(app.program.document.name, document::UNTITLED);
    assert!(app.program.prompt.is_none());
}
