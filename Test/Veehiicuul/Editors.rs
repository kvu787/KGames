// The editors through their frames, as the main loop runs them, with
// presses and drags that the tests make up, in their cards' and views' real
// layout, which Library/UserInterface's Headless lays out without a window
// or a GPU: the tabs, what fits in the cards, which controls keep keyboard
// focus, color fields, painting the terrain, panning and zooming the view,
// building the track, the vehicles' footprints, arranging the scene, undo,
// and saving.

use std::path::PathBuf;

use user_interface::json::Value;
use user_interface::tokens::{APP_BAR_HEIGHT, CARD_PADDING, FULLSCREEN_HEIGHT, MAXIMIZED_HEIGHT, PAGE_PADDING, STATS_BAR_HEIGHT, STATUS_BAR_HEIGHT, WINDOW_WIDTH};
use user_interface::{Headless, Key, Modifiers, Point, Startup};
use veehiicuul::curve;
use veehiicuul::program::{PLAY, SCENE, TERRAIN, TRACK, VEHICLES, Veehiicuul};
use veehiicuul::scene_editor::Picked;
use veehiicuul::vehicle_editor::SHOWN_HEADING;

// The window's size, maximized, which the views fill.
const SIZE: [f32; 2] = [WINDOW_WIDTH, MAXIMIZED_HEIGHT];
// The middle of the window, which is in the editors' view, right of the
// card and under the app bar.
const MIDDLE: Point = Point::new(SIZE[0] / 2.0, SIZE[1] / 2.0);

// The game in Play, after its first frames, saving to a folder of the
// test's own.
fn start(test: &str) -> Headless<Veehiicuul> {
    let saves = save_folder(test);
    let _ = std::fs::remove_dir_all(&saves);
    Headless::start(Startup::for_tests("{}", &saves), Veehiicuul::start)
}

fn save_folder(test: &str) -> PathBuf {
    std::env::temp_dir().join("VeehiicuulEditorTests").join(test)
}

// Where a point on the ground is on the screen, in the shared view.
fn screen(app: &Headless<Veehiicuul>, point: [f64; 2]) -> Point {
    let [x, y] = app.program.viewport.eye.ground_to_screen(point, SIZE);
    Point::new(x, y)
}

fn ground(app: &Headless<Veehiicuul>, point: Point) -> [f64; 2] {
    app.program.viewport.eye.to_ground([point.x, point.y], SIZE)
}

// Has the shared view look at a point on the ground, which goes to the
// middle of the window.
fn look_at(app: &mut Headless<Veehiicuul>, point: [f64; 2]) {
    app.program.viewport.eye.center = point;
    app.frame();
}

fn near(a: [f64; 2], b: [f64; 2]) -> bool {
    curve::distance(a, b) < 1e-3
}

fn offset(point: Point, dx: f32, dy: f32) -> Point {
    Point::new(point.x + dx, point.y + dy)
}

#[test]
fn the_tabs_go_between_play_and_the_editors() {
    let mut app = start("Tabs");
    assert_eq!(app.program.mode, PLAY);
    for (label, mode) in [("Scene", SCENE), ("Terrain", TERRAIN), ("Track", TRACK), ("Vehicles", VEHICLES), ("Play", PLAY)] {
        app.click(&format!("Mode/{label}"));
        assert_eq!(app.program.mode, mode, "{label}");
    }
    // The tabs never take keyboard focus, so the arrow keys drive in Play
    // instead of going to them. Back in Play, the car starts over, and
    // waits a moment first.
    assert_eq!(app.ui.state().get("Focused"), Some(&Value::Null));
    app.hold(Key::UP).hold(Key::LEFT).frames(26);
    assert_eq!(app.program.mode, PLAY);
    assert!(app.program.game.car.speed() > 0.0);
}

// Every editor's controls are in its cards, above the status bar, at either
// height, even the scene editor's with a decoration picked, which shows its
// model's paint.
#[test]
fn the_editors_controls_fit_in_their_cards() {
    for fullscreen in [false, true] {
        let mut app = start("Fit");
        app.set_fullscreen(fullscreen);
        let height = if fullscreen { FULLSCREEN_HEIGHT } else { MAXIMIZED_HEIGHT };
        let bottom = height - STATUS_BAR_HEIGHT - PAGE_PADDING - CARD_PADDING;
        for label in ["Scene", "Terrain", "Track", "Vehicles"] {
            app.click(&format!("Mode/{label}"));
            if label == "Scene" {
                app.click("Scene/Objects/0");
                assert_eq!(app.program.scene_editor.picked, Some(Picked::Decoration(0)));
            }
            for target in app.ui.targets().iter().filter(|target| target.listed) {
                let area = target.rect;
                if target.name.ends_with("/View") || area.y < STATS_BAR_HEIGHT + APP_BAR_HEIGHT {
                    continue;
                }
                assert!(
                    area.x >= PAGE_PADDING + CARD_PADDING
                        && area.right() <= WINDOW_WIDTH - PAGE_PADDING - CARD_PADDING
                        && area.bottom() <= bottom,
                    "{label}: {} is at {area:?}",
                    target.name
                );
            }
        }
    }
}

// Only text fields keep keyboard focus, so that the arrows never go to a
// control, like the tools' segments or the list. A color field's color
// changes with Enter, and a press in the view takes its focus.
#[test]
fn only_text_fields_keep_keyboard_focus() {
    let mut app = start("Focus");
    app.click("Mode/Terrain");
    app.click("Terrain/Tool/Fill");
    assert_eq!(app.ui.state().get("Focused"), Some(&Value::Null));
    app.key(Key::RIGHT).key(Key::LEFT);
    assert_eq!(app.program.terrain_editor.tool, 2);
    app.click("Mode/Scene").click("Scene/Objects/1");
    assert_eq!(app.ui.state().get("Focused"), Some(&Value::Null));
    app.key(Key::DOWN);
    assert_eq!(app.program.scene_editor.picked, Some(Picked::Parked(0)));

    app.click("Mode/Terrain").click("Terrain/Color");
    assert_eq!(app.ui.state().get("Focused"), Some(&"Terrain/Color".into()));
    app.key_with(Modifiers::CONTROL, Key::A).type_text("#123456");
    assert_ne!(app.program.game.current_scene().terrain.palette[0], [0x12, 0x34, 0x56], "not before Enter");
    app.key(Key::ENTER);
    assert_eq!(app.program.game.current_scene().terrain.palette[0], [0x12, 0x34, 0x56]);
    app.press_point(MIDDLE);
    assert_eq!(app.ui.state().get("Focused"), Some(&Value::Null));
}

// A press paints the hexels under the brush, a drag paints along it, Erase
// takes the paint off, and undo and redo go back and forth.
#[test]
fn the_terrain_editor_paints_and_erases_hexels() {
    let mut app = start("Terrain");
    app.click("Mode/Terrain");
    look_at(&mut app, [150.0, 100.0]);
    let cell = app.program.game.current_scene().terrain.cell_at(ground(&app, MIDDLE));
    let before = app.program.game.current_scene().terrain.cells.len();
    app.click_point(MIDDLE);
    let terrain = &app.program.game.current_scene().terrain;
    assert_eq!(terrain.get(cell), Some(0));
    let painted = terrain.cells.len();
    assert!(painted > before && painted <= before + 7, "{before} then {painted}");
    // The fourth color, from its swatch.
    app.click("Palette/3");
    assert_eq!(app.program.terrain_editor.color, 3);
    app.drag(Point::new(1400.0, 900.0), Point::new(1700.0, 900.0), 6);
    let terrain = &app.program.game.current_scene().terrain;
    let along = terrain.cell_at(ground(&app, Point::new(1550.0, 900.0)));
    assert_eq!(terrain.get(along), Some(3));
    app.click("Terrain/Tool/Erase");
    assert_eq!(app.program.terrain_editor.tool, 1);
    app.click_point(MIDDLE);
    assert_eq!(app.program.game.current_scene().terrain.get(cell), None);
    app.key_with(Modifiers::CONTROL, Key::Z);
    assert_eq!(app.program.game.current_scene().terrain.get(cell), Some(0), "undo brings the paint back");
    app.key_with(Modifiers::CONTROL, Key::Y);
    assert_eq!(app.program.game.current_scene().terrain.get(cell), None, "redo takes it off again");
    assert!(app.program.unsaved());
    // The buttons undo and redo too, and are off when there's nothing to do.
    assert!(app.is_enabled("Undo") && !app.is_enabled("Redo"));
    app.click("Undo");
    assert_eq!(app.program.game.current_scene().terrain.get(cell), Some(0));
    app.click("Redo");
    assert_eq!(app.program.game.current_scene().terrain.get(cell), None);
}

// Space and a drag pan the view, without painting, and the wheel zooms it
// around the pointer, which keeps the ground under it where it is.
#[test]
fn the_view_pans_with_space_and_zooms_with_the_wheel() {
    let mut app = start("View");
    app.click("Mode/Terrain");
    let painted = app.program.game.current_scene().terrain.cells.len();
    let center = app.program.viewport.eye.center;
    let grabbed = ground(&app, MIDDLE);
    app.hold(Key::SPACE).drag(MIDDLE, offset(MIDDLE, 200.0, 100.0), 4).release_key(Key::SPACE);
    assert_eq!(app.program.game.current_scene().terrain.cells.len(), painted);
    assert_ne!(app.program.viewport.eye.center, center);
    assert!(near(ground(&app, offset(MIDDLE, 200.0, 100.0)), grabbed));
    let height = app.program.viewport.eye.height;
    let at = Point::new(1800.0, 400.0);
    let under = ground(&app, at);
    app.scroll_point(at, 1.0);
    assert!((app.program.viewport.eye.height - height / 1.25).abs() < 1e-9);
    assert!(near(ground(&app, at), under));
    assert!(!app.program.unsaved());
    // The wheel over the card doesn't zoom.
    app.scroll_point(Point::new(200.0, 600.0), 1.0);
    assert!((app.program.viewport.eye.height - height / 1.25).abs() < 1e-9);
}

// Moving the road's first control point, adding one and deleting it, adding
// an island, and moving the start.
#[test]
fn the_track_builder_moves_adds_and_deletes_points() {
    let mut app = start("Track");
    app.click("Mode/Track");
    let first = app.program.game.current_scene().track.road[0];
    look_at(&mut app, first);
    let from = screen(&app, first);
    let to = offset(from, 40.0, 30.0);
    app.drag(from, to, 3);
    let moved = app.program.game.current_scene().track.road[0];
    assert!(near(moved, ground(&app, to)), "{moved:?}");
    assert_eq!(app.program.track_editor.curve, 0);

    app.click("Track/Tool/Add points");
    let road = curve::sample(&app.program.game.current_scene().track.road);
    look_at(&mut app, road[100]);
    let on_the_road = screen(&app, road[100]);
    app.click_point(on_the_road);
    let track = &app.program.game.current_scene().track;
    assert_eq!(track.road.len(), 65);
    assert!(track.road.iter().any(|&point| near(point, ground(&app, on_the_road))));

    app.click("Track/Tool/Delete points");
    app.click_point(on_the_road);
    assert_eq!(app.program.game.current_scene().track.road.len(), 64);

    app.click("Track/AddIsland");
    assert_eq!(app.program.game.current_scene().track.islands.len(), 3);
    assert_eq!(app.program.track_editor.curve, 3);
    app.click("Track/DeleteIsland");
    assert_eq!(app.program.game.current_scene().track.islands.len(), 2);

    app.click("Track/Tool/Move points");
    let start = app.program.game.current_scene().track.start;
    look_at(&mut app, start.position);
    let from = screen(&app, start.position);
    let to = offset(from, -20.0, 0.0);
    app.drag(from, to, 2);
    let moved = app.program.game.current_scene().track.start;
    assert!(near(moved.position, ground(&app, to)));
    assert_eq!(moved.heading, start.heading);
}

// A track with a problem says so, in the Track tab's card, and in Play's,
// which still fits the page.
#[test]
fn a_track_with_a_problem_says_so() {
    let mut app = start("Problem");
    app.program.game.document.scenes[0].track.start.position = [1000.0, 1000.0];
    app.frame();
    let problems = app.program.game.last_shape().problems.clone();
    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(app.has_target("Play/Problem"));
    let card = app.target("Play/Card");
    assert!(card.y > STATS_BAR_HEIGHT + APP_BAR_HEIGHT && card.bottom() == MAXIMIZED_HEIGHT - STATUS_BAR_HEIGHT - PAGE_PADDING, "{card:?}");
    app.click("Mode/Track");
    assert!(app.has_target("Track/Problem/0") && !app.has_target("Track/Problem/1"));
    let state = app.state();
    let listed = state.get("TrackEditor").and_then(|editor| editor.get("Problems"));
    assert_eq!(listed, Some(&Value::from(problems)));
}

// A side of the footprint follows a drag from it, Fit makes it the model's
// rectangle, and a new vehicle is a copy, with a name of its own.
#[test]
fn the_footprint_creator_moves_a_side_and_fits_the_model() {
    let mut app = start("Footprint");
    app.click("Mode/Vehicles");
    let front = app.program.game.current_vehicle().footprint.max[1];
    let heading = SHOWN_HEADING.to_radians();
    let ahead = [heading.sin(), heading.cos()];
    let eye = app.program.vehicle_editor.viewport.eye;
    let [x, y] = eye.ground_to_screen([ahead[0] * front, ahead[1] * front], SIZE);
    let [far_x, far_y] = eye.ground_to_screen([ahead[0] * (front + 1.0), ahead[1] * (front + 1.0)], SIZE);
    app.drag(Point::new(x, y), Point::new(far_x, far_y), 3);
    let moved = app.program.game.current_vehicle().footprint.max[1];
    assert!((moved - (front + 1.0)).abs() < 1e-3, "{front} then {moved}");
    app.click("Footprint/Fit");
    let footprint = app.program.game.current_vehicle().footprint;
    assert!((footprint.max[1] - 3.157522).abs() < 1e-5 && (footprint.min[0] + 1.5).abs() < 1e-5, "{footprint:?}");

    app.click("Vehicles/Add");
    assert_eq!(app.program.game.document.vehicles.list.len(), 7);
    assert_eq!(app.program.game.current_vehicle().name, "Vehicle 7");
    // Renaming Green renames it where it's parked too.
    app.click("Vehicles/List/0");
    assert_eq!(app.program.game.current_vehicle().name, "Green");
    let field = app.target("Vehicles/Name");
    app.click_point(Point::new(field.right() - 4.0, field.center().y));
    app.type_text("ish");
    assert_eq!(app.program.game.current_vehicle().name, "Greenish");
    assert_eq!(app.program.game.current_scene().parked[0].vehicle, "Greenish");
    // A parked vehicle can't be deleted.
    app.click("Vehicles/Delete");
    assert_eq!(app.program.game.document.vehicles.list.len(), 7);
    assert_eq!(app.program.vehicle_editor.message, "Greenish is parked in Ribeye, so it can't go.");
    assert!(app.has_target("Vehicles/Message"));
}

// A press on the cone picks it, and a drag moves it. A new decoration goes
// where the view looks, and Delete takes the picked one away.
#[test]
fn the_scene_editor_picks_and_moves_decorations() {
    let mut app = start("Scene");
    app.click("Mode/Scene");
    let cone = app.program.game.current_scene().decorations[0].pose.position;
    look_at(&mut app, cone);
    let from = screen(&app, cone);
    let to = offset(from, 50.0, 20.0);
    app.drag(from, to, 3);
    assert_eq!(app.program.scene_editor.picked, Some(Picked::Decoration(0)));
    let moved = app.program.game.current_scene().decorations[0].pose.position;
    let (dropped, grabbed) = (ground(&app, to), ground(&app, from));
    assert!(near(moved, [cone[0] + dropped[0] - grabbed[0], cone[1] + dropped[1] - grabbed[1]]), "{moved:?}");
    // A press on nothing picks nothing.
    app.click_point(Point::new(2400.0, 1300.0));
    assert_eq!(app.program.scene_editor.picked, None);
    app.click("Scene/AddDecoration");
    assert_eq!(app.program.game.current_scene().decorations.len(), 2);
    assert_eq!(app.program.scene_editor.picked, Some(Picked::Decoration(1)));
    app.click("Scene/Delete");
    assert_eq!(app.program.game.current_scene().decorations.len(), 1);
    app.click("Scene/Park");
    assert_eq!(app.program.game.current_scene().parked.len(), 7);
    assert_eq!(app.program.scene_editor.picked, Some(Picked::Parked(6)));
}

// Save writes the files that changed, and only those, in the save folder.
#[test]
fn save_writes_only_what_changed() {
    let mut app = start("Save");
    app.key_with(Modifiers::CONTROL, Key::S);
    assert_eq!(app.program.status, "Nothing to save");
    app.program.game.document.vehicles.list[0].acceleration.forward = 12.0;
    app.click("NewScene");
    assert_eq!(app.program.game.current_scene().name, "Scene 2");
    assert!(app.program.unsaved());
    app.click("Save");
    assert_eq!(app.program.status, "Saved Scenes/Scene 2.json, Vehicles.json");
    assert!(!app.program.unsaved());
    let saves = save_folder("Save");
    assert!(saves.join("Scenes/Scene 2.json").is_file());
    assert!(!saves.join("Scenes/Ribeye.json").exists());
    let vehicles = std::fs::read_to_string(saves.join("Vehicles.json")).unwrap();
    assert!(vehicles.contains("\"Forward\": 12.0"), "{vehicles}");
}
