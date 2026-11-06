// What input does to MeshModeler's model, through the cards' and the view's
// real layout, which user_interface::Headless builds without a window or a
// GPU: picking and box selection, modal moves and extrusions, loop cuts,
// painting, creases, subdivision, undo, and where the cards go at both
// supported window heights.

use std::path::{Path, PathBuf};

use json::Value;
use math::Vector3;
use mesh_modeler::modeler::Modeler;
use polygon_mesh::Edge;
use user_interface::tokens::{FULLSCREEN_HEIGHT, MAXIMIZED_HEIGHT, PAGE_PADDING, STATUS_BAR_HEIGHT, WINDOW_WIDTH};
use user_interface::{Headless, Key, Modifiers, MouseButton, Point};

fn folder(test: &str) -> PathBuf {
    let folder = Path::new(env!("CARGO_TARGET_TMPDIR")).join("MeshModeler").join(test);
    let _ = std::fs::remove_dir_all(&folder);
    std::fs::create_dir_all(&folder).unwrap();
    folder
}

fn modeler(test: &str) -> Headless<Modeler> {
    let mut app = Headless::new(Modeler::new(folder(test)));
    app.frames(2);
    app
}

// Where a point of the world is in the window.
fn screen(app: &Headless<Modeler>, point: Vector3) -> Point {
    app.program.camera.to_screen(point, app.program.view).0
}

// The vertex at a position of the starting cube, whose corners are at 1 and
// -1.
fn vertex_at(app: &Headless<Modeler>, position: Vector3) -> u32 {
    let mesh = &app.program.document.mesh;
    (0..mesh.positions.len() as u32).find(|vertex| mesh.position(*vertex) == position).unwrap()
}

fn state_at(app: &Headless<Modeler>, path: &[&str]) -> Value {
    let mut value = app.state();
    for part in path {
        value = value.get(part).unwrap_or_else(|| panic!("no {part} in the state")).clone();
    }
    value
}

fn number(app: &Headless<Modeler>, path: &[&str]) -> f64 {
    match state_at(app, path) {
        Value::Number(number) => number,
        other => panic!("{path:?} is {}", other.to_json()),
    }
}

fn text(app: &Headless<Modeler>, name: &str) -> Option<String> {
    match state_at(app, &[name]) {
        Value::String(text) => Some(text),
        _ => None,
    }
}

fn flag(app: &Headless<Modeler>, name: &str) -> bool {
    state_at(app, &[name]) == Value::Bool(true)
}

// What the modal edit under way says of itself, like its Kind or Axis.
fn operation(app: &Headless<Modeler>, name: &str) -> Option<String> {
    match state_at(app, &["Operation"]).get(name) {
        Some(Value::String(text)) => Some(text.clone()),
        _ => None,
    }
}

#[test]
fn a_click_in_the_view_selects_the_face_under_it() {
    let mut app = modeler("ClickFace");
    let center = app.program.view.center();
    app.click_point(center);
    let selection = &app.program.document.selection;
    assert_eq!(selection.faces.len(), 1);
    // The camera looks at the origin from +Z's side, so the face in the
    // middle of the view is the cube's +Z face.
    let face = *selection.faces.iter().next().unwrap();
    let normal = app.program.document.mesh.face_normal(face);
    assert!(normal[2] > 0.99, "{normal:?}");
    assert_eq!(selection.vertices.len(), 4);
    assert_eq!(app.program.document.undo_name(), Some("Select"));

    // A click on nothing selects nothing.
    let corner = app.program.view.origin() + Point::new(20.0, 20.0);
    app.click_point(corner);
    assert!(app.program.document.selection.is_empty());
}

#[test]
fn shift_adds_to_the_selection_and_takes_away_from_it() {
    let mut app = modeler("ShiftClick");
    app.key(Key::DIGIT_1);
    assert_eq!(text(&app, "Mode").as_deref(), Some("Vertex"));
    let a = screen(&app, [1.0, 1.0, 1.0]);
    let b = screen(&app, [-1.0, 1.0, 1.0]);
    app.click_point(a);
    app.hold(Key::SHIFT).click_point(b).release_key(Key::SHIFT);
    assert_eq!(app.program.document.selection.vertices.len(), 2);
    app.hold(Key::SHIFT).click_point(a).release_key(Key::SHIFT);
    let selection: Vec<u32> = app.program.document.selection.vertices.iter().copied().collect();
    assert_eq!(selection, vec![vertex_at(&app, [-1.0, 1.0, 1.0])]);
}

#[test]
fn a_drag_selects_a_box_and_hidden_vertices_are_not_picked() {
    let mut app = modeler("Box");
    app.key(Key::DIGIT_1);
    let view = app.program.view;
    app.drag(view.origin() + Point::new(10.0, 10.0), Point::new(view.right() - 10.0, view.bottom() - 10.0), 4);
    assert_eq!(app.program.document.selection.vertices.len(), 8);
    assert_eq!(app.program.document.undo_name(), Some("Select box"));
    // Ctrl takes a box away.
    let corner = screen(&app, [1.0, 1.0, 1.0]);
    app.hold(Key::CONTROL);
    app.drag(corner - Point::new(15.0, 15.0), corner + Point::new(15.0, 15.0), 2);
    app.release_key(Key::CONTROL);
    assert_eq!(app.program.document.selection.vertices.len(), 7);

    // The corner at the back, which the cube hides, isn't picked by a click
    // on it, until X-ray shows it.
    app.key_with(Modifiers::ALT, Key::A);
    let back = screen(&app, [-1.0, -1.0, -1.0]);
    app.click_point(back);
    assert!(!app.program.document.selection.vertices.contains(&vertex_at(&app, [-1.0, -1.0, -1.0])));
    app.key_with(Modifiers::ALT, Key::Z);
    assert!(flag(&app, "XRay"));
    app.click_point(back);
    let picked: Vec<u32> = app.program.document.selection.vertices.iter().copied().collect();
    assert_eq!(picked, vec![vertex_at(&app, [-1.0, -1.0, -1.0])]);
}

#[test]
fn keys_select_all_none_and_the_inverse_in_each_mode() {
    let mut app = modeler("SelectKeys");
    app.key(Key::A);
    assert_eq!(app.program.document.selection.faces.len(), 6);
    app.key_with(Modifiers::ALT, Key::A);
    assert!(app.program.document.selection.is_empty());
    app.click_point(app.program.view.center());
    app.key_with(Modifiers::CONTROL, Key::I);
    assert_eq!(app.program.document.selection.faces.len(), 5);
    // Edge mode keeps the faces' edges, which are all of them, so all the
    // faces are selected, as in Blender.
    app.key(Key::DIGIT_2);
    assert_eq!(app.program.document.selection.edges.len(), 12);
    assert_eq!(app.program.document.selection.faces.len(), 6);
}

#[test]
fn g_moves_the_selection_along_an_axis_and_esc_takes_it_back() {
    let mut app = modeler("Move");
    app.key(Key::A);
    let before = app.program.document.mesh.positions.clone();
    let start = app.program.view.center() + Point::new(-200.0, 100.0);
    app.move_point(start);
    app.key(Key::G);
    app.key(Key::X);
    app.move_point(start + Point::new(120.0, 40.0));
    assert_eq!(operation(&app, "Kind").as_deref(), Some("Move"));
    assert_eq!(operation(&app, "Axis").as_deref(), Some("X"));
    let moved = app.program.document.mesh.positions.clone();
    for (old, new) in before.iter().zip(&moved) {
        assert!(new[0] > old[0] + 0.1, "{old:?} {new:?}");
        assert_eq!((new[1], new[2]), (old[1], old[2]));
    }
    // Esc takes it back, without a step of history.
    let steps = app.program.document.undo_steps();
    app.key(Key::ESCAPE);
    assert_eq!(app.program.document.mesh.positions, before);
    assert_eq!(app.program.document.undo_steps(), steps);
    assert!(app.program.editor.operation.is_none());

    // A click keeps it, as a step that Ctrl+Z takes back and Ctrl+Y redoes.
    app.key(Key::G);
    app.key(Key::Y);
    app.move_point(start + Point::new(0.0, -80.0));
    app.click_point(start + Point::new(0.0, -80.0));
    let raised = app.program.document.mesh.positions.clone();
    assert!(raised.iter().zip(&before).all(|(new, old)| new[1] > old[1] && new[0] == old[0]));
    assert_eq!(app.program.document.undo_name(), Some("Move"));
    app.key_with(Modifiers::CONTROL, Key::Z);
    assert_eq!(app.program.document.mesh.positions, before);
    app.key_with(Modifiers::CONTROL, Key::Y);
    assert_eq!(app.program.document.mesh.positions, raised);
    // The click that kept it didn't select anything.
    assert_eq!(app.program.document.selection.faces.len(), 6);
}

#[test]
fn ctrl_snaps_a_rotation_to_15_degrees() {
    let mut app = modeler("Rotate");
    app.key(Key::A);
    let center = app.program.view.center();
    app.move_point(center + Point::new(200.0, 0.0));
    app.key(Key::R);
    app.key(Key::Y);
    app.hold(Key::CONTROL);
    // About 40 degrees counterclockwise on screen snaps to 45.
    let angle = 40f32.to_radians();
    app.move_point(center + Point::new(200.0 * angle.cos(), -200.0 * angle.sin()));
    assert_eq!(operation(&app, "Amount").as_deref(), Some("45.0°"));
    app.key(Key::ENTER);
    app.release_key(Key::CONTROL);
    assert_eq!(app.program.document.undo_name(), Some("Rotate"));
    // Turned about Y, the corners stay as far from the axis and as high.
    for position in &app.program.document.mesh.positions {
        let radius = (position[0] * position[0] + position[2] * position[2]).sqrt();
        assert!((radius - 2f32.sqrt()).abs() < 1e-4, "{position:?}");
        assert!((position[1].abs() - 1.0).abs() < 1e-5);
    }
}

#[test]
fn e_extrudes_the_selected_face_along_its_normal() {
    let mut app = modeler("Extrude");
    let center = app.program.view.center();
    app.click_point(center);
    let face = *app.program.document.selection.faces.iter().next().unwrap();
    assert!(app.program.document.mesh.face_normal(face)[2] > 0.99);
    app.key(Key::E);
    assert_eq!(operation(&app, "Axis").as_deref(), Some("Normal"));
    // Out along +Z, which is toward the pointer's lower left on screen.
    let tip = screen(&app, [0.0, 0.0, 2.0]);
    let base = screen(&app, [0.0, 0.0, 1.0]);
    app.move_point(center + (tip - base));
    app.click_point(center + (tip - base));
    let mesh = &app.program.document.mesh;
    assert_eq!((mesh.positions.len(), mesh.faces.len()), (12, 10));
    assert!(mesh.validate().is_ok());
    let selected: Vec<usize> = app.program.document.selection.faces.iter().copied().collect();
    assert_eq!(selected.len(), 1);
    let moved = mesh.face_center(selected[0]);
    assert!((moved[2] - 2.0).abs() < 0.05, "{moved:?}");
    assert_eq!(app.program.document.undo_name(), Some("Extrude"));
    // Undo takes back the extrusion with its move, in one step.
    app.key_with(Modifiers::CONTROL, Key::Z);
    assert_eq!(app.program.document.mesh.faces.len(), 6);
}

#[test]
fn a_right_click_cancels_an_extrusion_whole() {
    let mut app = modeler("CancelExtrude");
    app.click_point(app.program.view.center());
    let before = app.program.document.mesh.clone();
    app.key(Key::E);
    app.move_point(app.program.view.center() + Point::new(-60.0, 60.0));
    assert_eq!(app.program.document.mesh.faces.len(), 10);
    let pointer = app.pointer();
    app.press_button(pointer, MouseButton::Right).release_button(MouseButton::Right);
    assert_eq!(app.program.document.mesh, before);
    assert!(app.program.editor.operation.is_none());
    // The right-click that cancelled didn't open the view's menu.
    assert!(text(&app, "Prompt").is_none());
}

#[test]
fn i_insets_the_selected_faces() {
    let mut app = modeler("Inset");
    let center = app.program.view.center();
    app.click_point(center);
    app.move_point(center + Point::new(150.0, 0.0));
    app.key(Key::I);
    app.move_point(center + Point::new(100.0, 0.0));
    app.key(Key::ENTER);
    let mesh = &app.program.document.mesh;
    assert_eq!(mesh.faces.len(), 10);
    assert!(mesh.validate().is_ok());
    assert_eq!(app.program.document.undo_name(), Some("Inset"));
    let face = *app.program.document.selection.faces.iter().next().unwrap();
    assert!(mesh.face_area(face) < 4.0 && mesh.face_area(face) > 0.5, "{}", mesh.face_area(face));
}

#[test]
fn ctrl_r_cuts_the_loop_under_the_pointer() {
    let mut app = modeler("LoopCut");
    app.key_with(Modifiers::CONTROL, Key::R);
    assert_eq!(text(&app, "Tool").as_deref(), Some("Loop cut"));
    // The vertical edge at the front right: its middle.
    let middle = screen(&app, [1.0, 0.0, 1.0]);
    app.move_point(middle);
    assert_eq!(number(&app, &["LoopPreviewEdges"]), 4.0);
    app.click_point(middle);
    let mesh = &app.program.document.mesh;
    assert_eq!((mesh.positions.len(), mesh.faces.len()), (12, 10));
    assert!(mesh.validate().is_ok());
    // The cut goes around the cube's middle, and is selected, as edges.
    assert_eq!(app.program.document.selection.edges.len(), 4);
    assert!(app.program.document.selection.vertices.iter().all(|vertex| mesh.position(*vertex)[1].abs() < 1e-5));
    assert_eq!(text(&app, "Mode").as_deref(), Some("Edge"));
    // Esc goes back to the Select tool.
    app.key(Key::ESCAPE);
    assert_eq!(text(&app, "Tool").as_deref(), Some("Select"));
}

#[test]
fn alt_click_selects_an_edge_loop() {
    let mut app = modeler("AltLoop");
    app.key_with(Modifiers::CONTROL, Key::R);
    app.click_point(screen(&app, [1.0, 0.0, 1.0]));
    app.key(Key::ESCAPE);
    app.key(Key::DIGIT_2);
    app.key_with(Modifiers::ALT, Key::A);
    // Alt and a click on a horizontal edge of the middle loop selects it.
    let point = screen(&app, [1.0, 0.0, 0.0]);
    app.hold(Key::ALT).click_point(point).release_key(Key::ALT);
    let selection = &app.program.document.selection;
    assert_eq!(selection.edges.len(), 4, "{:?}", selection.edges);
    assert_eq!(app.program.document.undo_name(), Some("Select loop"));
}

#[test]
fn the_paint_tool_paints_the_faces_it_drags_over() {
    let mut app = modeler("PaintTool");
    app.click("Paints/1");
    assert_eq!(number(&app, &["ActivePaint"]), 1.0);
    app.key(Key::P);
    let front = screen(&app, [0.0, 0.0, 1.0]);
    let right = screen(&app, [1.0, 0.0, 0.0]);
    app.drag(front, right, 6);
    let painted: Vec<usize> = (0..6).filter(|face| app.program.document.mesh.faces[*face].material == 1).collect();
    assert_eq!(painted.len(), 2, "{painted:?}");
    assert_eq!(app.program.document.undo_name(), Some("Paint"));
    // The stroke is one step.
    app.key_with(Modifiers::CONTROL, Key::Z);
    assert!(app.program.document.mesh.faces.iter().all(|face| face.material == 0));
}

#[test]
fn the_paint_button_paints_the_selected_faces_with_the_paint_picked() {
    let mut app = modeler("PaintButton");
    app.key(Key::A);
    assert!(!app.is_enabled("Remove paint") || app.program.document.paints.len() > 1);
    app.click("Paints/3");
    app.click("Paint selected faces");
    assert!(app.program.document.mesh.faces.iter().all(|face| face.material == 3));
    // Removing the paint repaints its faces with the first.
    app.click("Remove paint");
    assert_eq!(app.program.document.paints.len(), 3);
    assert!(app.program.document.mesh.faces.iter().all(|face| face.material == 0));
    // Adding one picks it.
    app.click("Add paint");
    assert_eq!(app.program.document.paints.len(), 4);
    assert_eq!(app.program.editor.active_paint, 3);
}

#[test]
fn a_paint_s_color_and_settings_change_in_one_step_each() {
    let mut app = modeler("PaintSettings");
    let steps = app.program.document.undo_steps();
    let rail = app.target("Paint/Shift");
    app.drag(Point::new(rail.x + 20.0, rail.bottom() - 10.0), Point::new(rail.center().x, rail.bottom() - 10.0), 4);
    app.frames(2);
    assert!(app.program.document.paints[0].parameters.shift > 0.1);
    assert_eq!(app.program.document.undo_steps(), steps + 1);
    assert_eq!(app.program.document.undo_name(), Some("Edit paint"));
    app.key_with(Modifiers::CONTROL, Key::Z);
    assert_eq!(app.program.document.paints[0].parameters.shift, 0.0);
}

#[test]
fn shift_e_creases_the_selected_edges_and_the_surface_keeps_them() {
    let mut app = modeler("Crease");
    app.key(Key::DIGIT_2);
    app.click_point(screen(&app, [1.0, 0.0, 1.0]));
    assert_eq!(app.program.document.selection.edges.len(), 1);
    let edge = *app.program.document.selection.edges.iter().next().unwrap();
    let expected = Edge::new(vertex_at(&app, [1.0, 1.0, 1.0]), vertex_at(&app, [1.0, -1.0, 1.0]));
    assert_eq!(edge, expected);
    app.key_with(Modifiers::SHIFT, Key::E);
    assert_eq!(app.program.document.mesh.crease(edge), 1.0);
    app.key_with(Modifiers::ALT, Key::E);
    assert_eq!(app.program.document.mesh.crease(edge), 0.0);
}

#[test]
fn ctrl_and_a_digit_subdivide_the_view_s_surface() {
    let mut app = modeler("Subdivision");
    app.key_with(Modifiers::CONTROL, Key::DIGIT_2);
    assert_eq!(number(&app, &["Subdivision"]), 2.0);
    assert_eq!(number(&app, &["SurfaceFaces"]), 96.0);
    // Faces are still picked on the surface, as the cage's.
    app.click_point(app.program.view.center());
    assert_eq!(app.program.document.selection.faces.len(), 1);
    // Subdivision is how the view shows the model, not a change to it.
    assert_eq!(app.program.document.mesh.faces.len(), 6);
    app.key_with(Modifiers::CONTROL, Key::DIGIT_0);
    assert_eq!(number(&app, &["SurfaceFaces"]), 6.0);
}

#[test]
fn delete_and_merge_change_the_mesh_and_say_what_s_missing() {
    let mut app = modeler("Delete");
    // Nothing selected: Delete says why.
    app.key(Key::X);
    assert!(text(&app, "EditorMessage").unwrap().contains("selection"));
    app.click_point(app.program.view.center());
    app.key(Key::X);
    assert_eq!(app.program.document.mesh.faces.len(), 5);
    assert!(text(&app, "EditorMessage").is_none());
    // Merging the open side's corners closes it to a point.
    app.key(Key::DIGIT_1);
    app.key(Key::A);
    app.key(Key::M);
    assert_eq!(app.program.document.mesh.positions.len(), 1);
}

#[test]
fn the_middle_button_orbits_and_the_wheel_zooms() {
    let mut app = modeler("Camera");
    let center = app.program.view.center();
    let yaw = app.program.camera.yaw;
    app.drag_button(center, center + Point::new(100.0, 0.0), 4, MouseButton::Middle);
    assert!(app.program.camera.yaw < yaw);
    // The middle button doesn't select.
    assert!(app.program.document.selection.is_empty());
    let height = app.program.camera.height;
    app.scroll_point(center, 2.0);
    assert!(app.program.camera.height < height);
    app.key(Key::HOME);
    assert!(app.program.camera.height > 3.0);
}

#[test]
fn menus_run_commands() {
    let mut app = modeler("Menus");
    app.click("Menu/Mesh");
    app.move_to("Menu/Mesh/Menu/Add").frames(30);
    app.click("Menu/Mesh/Menu/Add/Menu/Torus");
    assert_eq!(app.program.document.mesh.faces.len(), 6 + 24 * 12);
    assert_eq!(app.program.document.selection.faces.len(), 24 * 12);
    assert_eq!(app.program.document.undo_name(), Some("Add Torus"));
    app.click("Menu/View");
    app.click("Menu/View/Menu/Flat shader");
    assert_eq!(text(&app, "Shader").as_deref(), Some("Flat"));
}

#[test]
fn the_cards_fit_at_both_window_heights() {
    for (height, fullscreen) in [(MAXIMIZED_HEIGHT, false), (FULLSCREEN_HEIGHT, true)] {
        let mut app = modeler(if fullscreen { "Fullscreen" } else { "Maximized" });
        app.set_fullscreen(fullscreen).frames(2);
        assert_eq!(app.size().height, height);
        let bottom = height - STATUS_BAR_HEIGHT - PAGE_PADDING;
        for name in ["Paint/Light", "Clear crease", "AddPrimitive", "Grid"] {
            assert!(app.target(name).bottom() <= bottom, "{name} at {:?} is below {bottom}", app.target(name));
        }
        let view = app.target("View");
        assert!(view.bottom() <= bottom && view.bottom() > bottom - 40.0, "{view:?}");
        assert!(view.right() < WINDOW_WIDTH - 400.0);
    }
}

#[test]
fn closing_with_changes_asks_first() {
    let mut app = modeler("Close");
    use user_interface::Program;
    assert!(app.program.may_close());
    app.key(Key::A);
    assert!(!app.program.may_close());
    app.frames(2);
    assert_eq!(text(&app, "Prompt").as_deref(), Some("Unsaved"));
    app.click("Unsaved/Close");
    assert!(!app.closed());
    assert!(!app.program.may_close());
    app.frames(2);
    app.click("Unsaved/Secondary");
    assert!(app.closed());
}

// How long a frame takes with the pointer over a subdivided mesh, where the
// view picks what's under it: a benchmark, which only reports.
#[test]
fn benchmark_hover_over_a_subdivided_mesh() {
    let mut app = modeler("HoverBenchmark");
    app.program.levels = 4;
    app.frames(2);
    assert!(app.program.scene.triangles.indices.len() >= 6 * 256 * 2 * 3, "{} indices", app.program.scene.triangles.indices.len());
    app.move_to("View");
    let start = std::time::Instant::now();
    app.frames(30);
    let per_frame = start.elapsed().as_secs_f64() * 1000.0 / 30.0;
    eprintln!("A frame with the pointer over the subdivided cube takes {per_frame:.3} ms to build");
    assert!(matches!(app.program.editor.hover, Some(mesh_modeler::editing::Element::Face(_))), "{:?}", app.program.editor.hover);
}
