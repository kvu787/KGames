// The editors through their frames, as the main loop runs them, with
// presses and drags that the tests make up, in their cards' and views' real
// layout, which Library/UserInterface's Headless draws without a window or a
// GPU: the tabs, what fits in the cards, painting the terrain, building the
// track, the vehicles' footprints, arranging the scene, undo, and saving.

use std::path::{Path, PathBuf};

use application::{Event, ProgramSettings};
use user_interface::headless::{FULLSCREEN, Headless, MAXIMIZED, click, control, typed};
use user_interface::{CARD_PADDING, MARGIN, PANELS_TOP, STATUS_BAR_HEIGHT};
use veehiicuul::curve;
use veehiicuul::game::Game;
use veehiicuul::program::{PLAY, SCENE, TERRAIN, TRACK, VEHICLES, Veehiicuul};
use veehiicuul::scene_editor::Picked;
use veehiicuul::settings::GameSettings;
use veehiicuul::vehicle_editor::SHOWN_HEADING;

// The game in Play, drawn once, saving to a folder of the test's own.
fn start(test: &str) -> (Veehiicuul, Headless) {
    let folder = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Program/Veehiicuul");
    let (_, settings) = GameSettings::defaults();
    let game = Game::load(&folder, &settings, &mut Vec::new()).unwrap_or_else(|problem| panic!("{problem}"));
    let saves = save_folder(test);
    let _ = std::fs::remove_dir_all(&saves);
    let mut program = Veehiicuul::without_drawing(game, saves);
    let mut headless = Headless::default();
    headless.program(&mut program, &[]).unwrap();
    (program, headless)
}

fn save_folder(test: &str) -> PathBuf {
    std::env::temp_dir().join("VeehiicuulEditorTests").join(test)
}

fn step(program: &mut Veehiicuul, headless: &mut Headless, events: &[Event]) {
    headless.program(program, events).unwrap();
}

// Presses a widget by its name, then draws a frame.
fn press(program: &mut Veehiicuul, headless: &mut Headless, name: &str) {
    let center = headless.center(name);
    step(program, headless, &click(center));
    step(program, headless, &[]);
}

fn tab(program: &mut Veehiicuul, headless: &mut Headless, label: &str) {
    press(program, headless, &format!("Mode/{label}"));
}

// Where a point on the ground is on the screen, in the shared view.
fn screen(program: &Veehiicuul, point: [f64; 2]) -> [f32; 2] {
    program.viewport.eye.ground_to_screen(point, MAXIMIZED)
}

fn ground(program: &Veehiicuul, point: [f32; 2]) -> [f64; 2] {
    program.viewport.eye.to_ground(point, MAXIMIZED)
}

// A press at `from`, a move halfway, and a release at `to`.
fn drag(program: &mut Veehiicuul, headless: &mut Headless, from: [f32; 2], to: [f32; 2]) {
    step(program, headless, &[Event::Press(from)]);
    headless.move_to([(from[0] + to[0]) / 2.0, (from[1] + to[1]) / 2.0]);
    step(program, headless, &[]);
    headless.move_to(to);
    step(program, headless, &[]);
    step(program, headless, &[Event::Release(to)]);
}

// Where the shared view looks, in the middle of the window, which is in the
// editors' view, right of the card and under the app bar.
const MIDDLE: [f32; 2] = [MAXIMIZED[0] / 2.0, MAXIMIZED[1] / 2.0];

fn look_at(program: &mut Veehiicuul, point: [f64; 2]) {
    program.viewport.eye.center = point;
}

fn near(a: [f64; 2], b: [f64; 2]) -> bool {
    curve::distance(a, b) < 1e-3
}

#[test]
fn the_tabs_go_between_play_and_the_editors() {
    let (mut program, mut headless) = start("Tabs");
    assert_eq!(program.mode, PLAY);
    for (label, mode) in [("Scene", SCENE), ("Terrain", TERRAIN), ("Track", TRACK), ("Vehicles", VEHICLES), ("Play", PLAY)] {
        tab(&mut program, &mut headless, label);
        assert_eq!(program.mode, mode, "{label}");
    }
    // Arrow keys drive in Play, and don't go to the tabs, which never take
    // keyboard focus.
    // Back in Play, the car starts over, and waits a moment first.
    headless.held_keys = vec![0x26, 0x25];
    for _ in 0..30 {
        step(&mut program, &mut headless, &[]);
    }
    assert_eq!(program.mode, PLAY);
    assert!(program.game.car.speed() > 0.0);
}

// Every editor's controls are in its cards, above the status bar, at either
// height, even the scene editor's with a decoration picked, which shows its
// model's paint.
#[test]
fn the_editors_controls_fit_in_their_cards() {
    for size in [MAXIMIZED, FULLSCREEN] {
        let (mut program, mut headless) = start("Fit");
        headless.size = size;
        for label in ["Scene", "Terrain", "Track", "Vehicles"] {
            tab(&mut program, &mut headless, label);
            if label == "Scene" {
                press(&mut program, &mut headless, "Scene/Objects/0");
                assert_eq!(program.scene_editor.picked, Some(Picked::Decoration(0)));
            }
            let bottom = size[1] - STATUS_BAR_HEIGHT - MARGIN - CARD_PADDING;
            for name in headless.interface.target_names() {
                let area = headless.target(&name);
                if name.ends_with("/View") || area.y < PANELS_TOP {
                    continue;
                }
                assert!(area.bottom() <= bottom && area.right() <= size[0], "{label}: {name} is at {area:?}");
            }
        }
    }
}

// A press paints the hexels under the brush, a drag paints along it, Erase
// takes the paint off, and undo and redo go back and forth.
#[test]
fn the_terrain_editor_paints_and_erases_hexels() {
    let (mut program, mut headless) = start("Terrain");
    tab(&mut program, &mut headless, "Terrain");
    look_at(&mut program, [150.0, 100.0]);
    let point = MIDDLE;
    let cell = program.game.current_scene().terrain.cell_at(ground(&program, point));
    let before = program.game.current_scene().terrain.cells.len();
    step(&mut program, &mut headless, &click(point));
    step(&mut program, &mut headless, &[]);
    let terrain = &program.game.current_scene().terrain;
    assert_eq!(terrain.get(cell), Some(0));
    let painted = terrain.cells.len();
    assert!(painted > before && painted <= before + 7, "{before} then {painted}");
    // The fourth color, from its swatch.
    press(&mut program, &mut headless, "Palette/3");
    assert_eq!(program.terrain_editor.color, 3);
    drag(&mut program, &mut headless, [1400.0, 900.0], [1700.0, 900.0]);
    let terrain = &program.game.current_scene().terrain;
    let along = terrain.cell_at(ground(&program, [1550.0, 900.0]));
    assert_eq!(terrain.get(along), Some(3));
    press(&mut program, &mut headless, "Terrain/Tool/Erase");
    step(&mut program, &mut headless, &click(point));
    step(&mut program, &mut headless, &[]);
    assert_eq!(program.game.current_scene().terrain.get(cell), None);
    step(&mut program, &mut headless, &[control('Z' as i32)]);
    assert_eq!(program.game.current_scene().terrain.get(cell), Some(0), "undo brings the paint back");
    step(&mut program, &mut headless, &[control('Y' as i32)]);
    assert_eq!(program.game.current_scene().terrain.get(cell), None, "redo takes it off again");
    assert!(program.unsaved());
}

// Moving the road's first control point, adding one and deleting it, adding
// an island, and moving and turning the start.
#[test]
fn the_track_builder_moves_adds_and_deletes_points() {
    let (mut program, mut headless) = start("Track");
    tab(&mut program, &mut headless, "Track");
    let first = program.game.current_scene().track.road[0];
    look_at(&mut program, first);
    step(&mut program, &mut headless, &[]);
    let from = screen(&program, first);
    let to = [from[0] + 40.0, from[1] + 30.0];
    drag(&mut program, &mut headless, from, to);
    let moved = program.game.current_scene().track.road[0];
    assert!(near(moved, ground(&program, to)), "{moved:?}");
    assert_eq!(program.track_editor.curve, 0);

    press(&mut program, &mut headless, "Track/Tool/Add points");
    let road = curve::sample(&program.game.current_scene().track.road);
    look_at(&mut program, road[100]);
    step(&mut program, &mut headless, &[]);
    let on_the_road = screen(&program, road[100]);
    step(&mut program, &mut headless, &click(on_the_road));
    step(&mut program, &mut headless, &[]);
    let track = &program.game.current_scene().track;
    assert_eq!(track.road.len(), 65);
    assert!(track.road.iter().any(|&point| near(point, ground(&program, on_the_road))));

    press(&mut program, &mut headless, "Track/Tool/Delete points");
    step(&mut program, &mut headless, &click(on_the_road));
    step(&mut program, &mut headless, &[]);
    assert_eq!(program.game.current_scene().track.road.len(), 64);

    press(&mut program, &mut headless, "Track/AddIsland");
    assert_eq!(program.game.current_scene().track.islands.len(), 3);
    assert_eq!(program.track_editor.curve, 3);
    press(&mut program, &mut headless, "Track/DeleteIsland");
    assert_eq!(program.game.current_scene().track.islands.len(), 2);

    press(&mut program, &mut headless, "Track/Tool/Move points");
    let start = program.game.current_scene().track.start;
    look_at(&mut program, start.position);
    step(&mut program, &mut headless, &[]);
    let from = screen(&program, start.position);
    drag(&mut program, &mut headless, from, [from[0] - 20.0, from[1]]);
    let moved = program.game.current_scene().track.start;
    assert!(near(moved.position, ground(&program, [from[0] - 20.0, from[1]])));
    assert_eq!(moved.heading, start.heading);
}

// A side of the footprint follows a drag from it, Fit makes it the model's
// rectangle, and a new vehicle is a copy, with a name of its own.
#[test]
fn the_footprint_creator_moves_a_side_and_fits_the_model() {
    let (mut program, mut headless) = start("Footprint");
    tab(&mut program, &mut headless, "Vehicles");
    let front = program.game.current_vehicle().footprint.max[1];
    let heading = SHOWN_HEADING.to_radians();
    let ahead = [heading.sin(), heading.cos()];
    let eye = program.vehicle_editor.viewport.eye;
    let middle = eye.ground_to_screen([ahead[0] * front, ahead[1] * front], MAXIMIZED);
    let farther = eye.ground_to_screen([ahead[0] * (front + 1.0), ahead[1] * (front + 1.0)], MAXIMIZED);
    drag(&mut program, &mut headless, middle, farther);
    let moved = program.game.current_vehicle().footprint.max[1];
    assert!((moved - (front + 1.0)).abs() < 1e-3, "{front} then {moved}");
    press(&mut program, &mut headless, "Footprint/Fit");
    let footprint = program.game.current_vehicle().footprint;
    assert!((footprint.max[1] - 3.157522).abs() < 1e-5 && (footprint.min[0] + 1.5).abs() < 1e-5, "{footprint:?}");

    press(&mut program, &mut headless, "Vehicles/Add");
    assert_eq!(program.game.document.vehicles.list.len(), 7);
    assert_eq!(program.game.current_vehicle().name, "Vehicle 7");
    // Renaming Green renames it where it's parked too.
    press(&mut program, &mut headless, "Vehicles/List/0");
    assert_eq!(program.game.current_vehicle().name, "Green");
    let field = headless.target("Vehicles/Name");
    step(&mut program, &mut headless, &click([field.right() - 4.0, field.center()[1]]));
    step(&mut program, &mut headless, &typed("ish"));
    step(&mut program, &mut headless, &[]);
    assert_eq!(program.game.current_vehicle().name, "Greenish");
    assert_eq!(program.game.current_scene().parked[0].vehicle, "Greenish");
    // A parked vehicle can't be deleted.
    press(&mut program, &mut headless, "Vehicles/Delete");
    assert_eq!(program.game.document.vehicles.list.len(), 7);
    assert_eq!(program.vehicle_editor.message, "Greenish is parked in Ribeye, so it can't go.");
}

// A press on the cone picks it, and a drag moves it. A new decoration goes
// where the view looks, and Delete takes the picked one away.
#[test]
fn the_scene_editor_picks_and_moves_decorations() {
    let (mut program, mut headless) = start("Scene");
    tab(&mut program, &mut headless, "Scene");
    let cone = program.game.current_scene().decorations[0].pose.position;
    look_at(&mut program, cone);
    step(&mut program, &mut headless, &[]);
    let from = screen(&program, cone);
    let to = [from[0] + 50.0, from[1] + 20.0];
    drag(&mut program, &mut headless, from, to);
    assert_eq!(program.scene_editor.picked, Some(Picked::Decoration(0)));
    let moved = program.game.current_scene().decorations[0].pose.position;
    assert!(near(moved, [cone[0] + ground(&program, to)[0] - ground(&program, from)[0], cone[1] + ground(&program, to)[1] - ground(&program, from)[1]]));
    // A press on nothing picks nothing.
    step(&mut program, &mut headless, &click([2400.0, 1300.0]));
    step(&mut program, &mut headless, &[]);
    assert_eq!(program.scene_editor.picked, None);
    press(&mut program, &mut headless, "Scene/AddDecoration");
    assert_eq!(program.game.current_scene().decorations.len(), 2);
    assert_eq!(program.scene_editor.picked, Some(Picked::Decoration(1)));
    press(&mut program, &mut headless, "Scene/Delete");
    assert_eq!(program.game.current_scene().decorations.len(), 1);
    press(&mut program, &mut headless, "Scene/Park");
    assert_eq!(program.game.current_scene().parked.len(), 7);
}

// Save writes the files that changed, and only those, in the save folder.
#[test]
fn save_writes_only_what_changed() {
    let (mut program, mut headless) = start("Save");
    step(&mut program, &mut headless, &[control('S' as i32)]);
    assert_eq!(program.status, "Nothing to save");
    program.game.document.vehicles.list[0].acceleration.forward = 12.0;
    press(&mut program, &mut headless, "NewScene");
    assert_eq!(program.game.current_scene().name, "Scene 2");
    assert!(program.unsaved());
    press(&mut program, &mut headless, "Save");
    assert_eq!(program.status, "Saved Scenes/Scene 2.json, Vehicles.json");
    assert!(!program.unsaved());
    let saves = save_folder("Save");
    assert!(saves.join("Scenes/Scene 2.json").is_file());
    assert!(!saves.join("Scenes/Ribeye.json").exists());
    let vehicles = std::fs::read_to_string(saves.join("Vehicles.json")).unwrap();
    assert!(vehicles.contains("\"Forward\": 12.0"), "{vehicles}");
}
