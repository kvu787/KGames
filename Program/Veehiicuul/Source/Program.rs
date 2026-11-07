// The game as a Program, which Library/UserInterface's main loop runs. Tabs
// in the app bar go between playing and the editors, which make the game's
// scenes and vehicles:
//
// | Tab      | What it does                                                         |
// | -------- | -------------------------------------------------------------------- |
// | Play     | Drives the current scene's track, with a card over the bottom left   |
// |          | corner that says what's going on and what the controls do            |
// | Scene    | Arranges decorations and parked vehicles, and sets the background,   |
// |          | the camera, and models' paint                                        |
// | Terrain  | Paints the hexel terrain                                             |
// | Track    | Builds the track from curves, which make the road and its barriers   |
// | Vehicles | Edits the vehicles, their footprints, and their paint                |
//
// After the tabs: in the editors, whether their view is at the game's
// angle, and at the right, the scene to play and edit, a new scene, undo,
// redo, and save, which writes the files that changed. The status bar says
// whether there are changes to save. Drawing.rs draws the 3D view under all
// of it, over the whole window.

use std::path::{Path, PathBuf};

use json::Value;
use user_interface::graphics::{Frame, Gpu};
use user_interface::tokens::{
    APP_BAR_HEIGHT, BODY, BODY_STRONG, CAPTION, CARD_PADDING, SPACE_L, SPACE_M, SPACE_S, SPACE_XL, SPACE_XS, SPACE_XXL,
};
use user_interface::{
    AppBar, Button, ButtonStyle, Canvas, Card, CheckBox, ComboBox, Divider, Field, Icon, InfoBar, Insets, KEYCAP_HEIGHT, Key, Modifiers,
    MouseButton, Point, Program, Rect, SelectorBar, Severity, Startup, Status, Text, Ui, UiEventKind, View,
};

use crate::camera::Eye;
use crate::controls::Controls;
use crate::document::Document;
use crate::drawing::{Drawing, Ground, Instance, View as Scene3d};
use crate::editing::{self, Viewport};
use crate::files::{self, srgb};
use crate::game::Game;
use crate::scene::Scene;
use crate::scene_editor::SceneEditor;
use crate::settings::GameSettings;
use crate::terrain_editor::TerrainEditor;
use crate::track_editor::TrackEditor;
use crate::vehicle_editor::{SHOWN_HEADING, VehicleEditor};
use crate::{FOLDER, NAME};

pub const MODES: [&str; 5] = ["Play", "Scene", "Terrain", "Track", "Vehicles"];
pub const PLAY: usize = 0;
pub const SCENE: usize = 1;
pub const TERRAIN: usize = 2;
pub const TRACK: usize = 3;
pub const VEHICLES: usize = 4;
// How many changes undo goes back.
const HISTORY: usize = 200;
// How wide the app bar's combo box of scenes is.
const SCENE_WIDTH: f32 = 200.0;

pub struct Veehiicuul {
    pub game: Game,
    pub mode: usize,
    // The view that the scene, terrain and track editors share.
    pub viewport: Viewport,
    pub scene_editor: SceneEditor,
    pub terrain_editor: TerrainEditor,
    pub track_editor: TrackEditor,
    pub vehicle_editor: VehicleEditor,
    // The document before each change, and after each undone one, and the
    // document as of the last change.
    undo: Vec<Document>,
    redo: Vec<Document>,
    committed: Document,
    // The document as of the last save, or as it was loaded.
    pub saved: Document,
    // Whether the document differed from `saved` as the frame began, for
    // the bars, which ask twice a frame.
    unsaved_now: bool,
    // Where Save writes: the game's folder, or a script's output folder.
    save_folder: PathBuf,
    // What the last save, or anything else, said.
    pub status: String,
    // What draws the 3D view, once prepare made it. Tests do without.
    drawing: Option<Drawing>,
    // What the frame's 3D view shows.
    eye: Eye,
    instances: Vec<Instance>,
    // What the tree was built from, which a change re-lays out, and the tab
    // it was built for, whose nodes this frame has.
    built_fingerprint: String,
    built_mode: usize,
}

impl Veehiicuul {
    // The game, saving to `save_folder`.
    pub fn new(game: Game, save_folder: PathBuf) -> Veehiicuul {
        let scene = game.current_scene();
        let viewport = Viewport::new(scene.camera_center, scene.camera_height);
        Veehiicuul {
            committed: game.document.clone(),
            saved: game.document.clone(),
            unsaved_now: false,
            eye: Eye::game(game.camera_center(), game.camera.height),
            game,
            mode: PLAY,
            viewport,
            scene_editor: SceneEditor::default(),
            terrain_editor: TerrainEditor::default(),
            track_editor: TrackEditor::default(),
            vehicle_editor: VehicleEditor::default(),
            undo: Vec::new(),
            redo: Vec::new(),
            save_folder,
            status: String::new(),
            drawing: None,
            instances: Vec::new(),
            built_fingerprint: String::new(),
            built_mode: PLAY,
        }
    }

    // The game as it starts: with its settings, from the files in its
    // folder, saving to the data folder, which is its folder, or a script's
    // output folder, so that a scripted run leaves the game's files alone.
    // Settings that name what isn't there make notices.
    pub fn start(startup: &mut Startup) -> Result<Veehiicuul, String> {
        let settings = GameSettings::read(&mut startup.settings);
        let mut notices = Vec::new();
        let game = Game::load(Path::new(FOLDER), &settings, &mut notices)?;
        for notice in notices {
            startup.notify(Severity::Warning, "Settings.json", &notice);
        }
        Ok(Veehiicuul::new(game, startup.data_folder.clone()))
    }

    pub fn unsaved(&self) -> bool {
        self.game.document != self.saved
    }

    // Keeps the document as of now, for undo, if it changed.
    fn commit(&mut self) {
        if self.game.document != self.committed {
            let before = std::mem::replace(&mut self.committed, self.game.document.clone());
            self.undo.push(before);
            if self.undo.len() > HISTORY {
                self.undo.remove(0);
            }
            self.redo.clear();
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty() || self.game.document != self.committed
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    pub fn undo(&mut self) {
        self.commit();
        if let Some(before) = self.undo.pop() {
            self.redo.push(std::mem::replace(&mut self.game.document, before));
            self.committed = self.game.document.clone();
            self.game.keep_in_document();
        }
    }

    pub fn redo(&mut self) {
        if let Some(after) = self.redo.pop() {
            self.undo.push(std::mem::replace(&mut self.game.document, after));
            self.committed = self.game.document.clone();
            self.game.keep_in_document();
        }
    }

    // Writes the files that changed since the last save.
    pub fn save(&mut self) {
        let changed = self.game.document.changed_files(&self.saved);
        if changed.is_empty() {
            self.status = "Nothing to save".to_string();
            return;
        }
        for (path, text) in &changed {
            if let Err(problem) = files::write(&self.save_folder.join(path), text) {
                self.status = problem;
                return;
            }
        }
        let names: Vec<&str> = changed.iter().map(|(path, _)| path.as_str()).collect();
        self.status = format!("Saved {}", names.join(", "));
        self.saved = self.game.document.clone();
    }

    pub fn new_scene(&mut self, seconds: f64) {
        let scenes = &self.game.document.scenes;
        let name = (scenes.len() + 1..)
            .map(|number| format!("Scene {number}"))
            .find(|name| scenes.iter().all(|scene| scene.name != *name))
            .unwrap();
        let scene = Scene::new(&name);
        self.viewport = Viewport::new(scene.camera_center, scene.camera_height);
        self.game.document.scenes.push(scene);
        self.game.go_to_scene(self.game.document.scenes.len() - 1, seconds);
    }

    // Goes to a tab. Going to Play puts the car at the start.
    pub fn go_to(&mut self, mode: usize, seconds: f64) {
        self.mode = mode;
        self.game.keep_in_document();
        if mode == PLAY {
            self.game.put_at_start(seconds);
        }
    }

    fn shortcuts(&mut self, ui: &mut Ui) {
        if ui.shortcut(Modifiers::CONTROL, Key::S) {
            self.save();
        }
        if ui.shortcut(Modifiers::CONTROL, Key::Z) {
            self.undo();
        }
        if ui.shortcut(Modifiers::CONTROL, Key::Y) {
            self.redo();
        }
    }

    // What shapes the tree: the mode, the scenes, and the active editor's
    // card. A change re-lays it out.
    fn fingerprint(&self) -> String {
        let names: Vec<&str> = self.game.document.scenes.iter().map(|scene| scene.name.as_str()).collect();
        let scene = self.game.scene;
        let editor = match self.mode {
            PLAY => self.play_fingerprint(),
            SCENE => self.scene_editor.fingerprint(&self.game.document, &self.game.models, scene),
            TERRAIN => self.terrain_editor.fingerprint(&self.game.document.scenes[scene].terrain),
            TRACK => self.track_editor.fingerprint(&self.game.document.scenes[scene].track, &self.game.last_shape().problems),
            _ => self.vehicle_editor.fingerprint(&self.game.document, &self.game.models, self.game.vehicle),
        };
        format!("mode {} scenes {names:?} scene {scene} unsaved {} {editor}", self.mode, self.unsaved(),)
    }

    // What shapes the play card: its scene, its vehicle, the first problem,
    // the gamepad, the camera's mode and whether the card shows.
    fn play_fingerprint(&self) -> String {
        let game = &self.game;
        let vehicle = game.current_vehicle();
        format!(
            "card {} scene {} vehicle {} {:?} problem {:?} camera {}",
            game.show_card,
            game.current_scene().name,
            vehicle.name,
            vehicle.acceleration,
            game.last_shape().problems.first(),
            game.camera.follow,
        )
    }

    // The app bar: the title and the tabs, in the editors whether their view
    // is at the game's angle, and at its right end, the scene and the
    // commands.
    fn build_app_bar(&self, ui: &mut Ui) {
        let mut bar = ui.app_bar(AppBar::new(Icon::Car, NAME));
        // The tabs take the bar's whole height, so that their text is in
        // its middle, with their pills under it.
        let tabs = bar.cut_left(SelectorBar::width(ui, &MODES), SPACE_XXL);
        ui.node("Mode", Rect::new(tabs.x, ui.content_top(), tabs.width, APP_BAR_HEIGHT), SelectorBar::new(&MODES, self.mode));

        // From the right end: save, redo and undo, then the scene.
        let save = Button::new("Save").icon(Icon::Save);
        let save = if self.unsaved_now { save.accent() } else { save };
        let undo = Button::new("Undo").icon(Icon::Undo);
        let redo = Button::new("Redo").icon(Icon::Redo);
        let new_scene = Button::new("New scene").icon(Icon::Add);
        let save_area = bar.cut_right(save.width(ui), SPACE_S);
        let redo_area = bar.cut_right(redo.width(ui), SPACE_S);
        let undo_area = bar.cut_right(undo.width(ui), SPACE_L);
        let divider = bar.cut_right(1.0, SPACE_L);
        let new_scene_area = bar.cut_right(new_scene.width(ui), SPACE_S);
        let scene_area = bar.cut_right(SCENE_WIDTH, SPACE_S);
        let label_area = bar.cut_right(ui.measure("Scene", BODY).width.ceil(), SPACE_L);

        ui.node("Save", save_area, save);
        ui.set_tooltip("Save", "Writes the files that changed (Ctrl+S)");
        ui.node("Redo", redo_area, redo);
        ui.set_tooltip("Redo", "Does again what undo undid (Ctrl+Y)");
        ui.node("Undo", undo_area, undo);
        ui.set_tooltip("Undo", "Takes back the last change (Ctrl+Z)");
        ui.node("BarDivider", divider.pad(Insets::symmetric(0.0, SPACE_XS)), Divider);
        ui.node("NewScene", new_scene_area, new_scene);
        ui.set_tooltip("NewScene", "Makes an empty scene, and goes to it");
        let names: Vec<&str> = self.game.document.scenes.iter().map(|scene| scene.name.as_str()).collect();
        ui.node("Scene", scene_area, ComboBox::new(&names, self.game.scene));
        ui.node("SceneLabel", label_area, Text::new("Scene", BODY).secondary());

        if self.mode != PLAY {
            let label = "The game's angle";
            let area = bar.left_strip(CheckBox::width(ui, label));
            let viewport = if self.mode == VEHICLES { &self.vehicle_editor.viewport } else { &self.viewport };
            ui.node("GameAngle", area, CheckBox::new(viewport.game_angle(), label));
            ui.set_tooltip("GameAngle", "Shows the view as the game does, instead of straight down");
        }
    }

    // What the app bar's and the global events did.
    fn events(&mut self, ui: &mut Ui) {
        let seconds = ui.time();
        let events: Vec<user_interface::UiEvent> = ui.events().to_vec();
        for event in events {
            match (event.source.as_str(), &event.kind) {
                ("Mode", UiEventKind::Selected(mode)) => self.go_to(*mode, seconds),
                ("Save", UiEventKind::Clicked) => self.save(),
                ("Redo", UiEventKind::Clicked) => self.redo(),
                ("Undo", UiEventKind::Clicked) => self.undo(),
                ("NewScene", UiEventKind::Clicked) => self.new_scene(seconds),
                ("Scene", UiEventKind::Selected(scene)) => {
                    if *scene != self.game.scene {
                        self.game.go_to_scene(*scene, seconds);
                        let current = self.game.current_scene();
                        self.viewport = Viewport::new(current.camera_center, current.camera_height);
                    }
                }
                ("GameAngle", UiEventKind::Toggled(on)) => {
                    let viewport = if self.mode == VEHICLES { &mut self.vehicle_editor.viewport } else { &mut self.viewport };
                    viewport.set_game_angle(*on);
                }
                _ => {}
            }
        }
    }

    // Keeps the bars true to the game.
    fn sync_bars(&self, ui: &mut Ui) {
        ui.get_mut::<SelectorBar>("Mode").set_selected(self.mode);
        ui.get_mut::<Button>("Save").set_style(if self.unsaved_now { ButtonStyle::Accent } else { ButtonStyle::Standard });
        ui.set_enabled("Redo", self.can_redo());
        ui.set_enabled("Undo", self.can_undo());
        let names: Vec<&str> = self.game.document.scenes.iter().map(|scene| scene.name.as_str()).collect();
        let combo = ui.get_mut::<ComboBox>("Scene");
        combo.set_items(&names);
        combo.set_selected(self.game.scene);
        if self.built_mode != PLAY && ui.has_node("GameAngle") {
            let viewport = if self.built_mode == VEHICLES { &self.vehicle_editor.viewport } else { &self.viewport };
            ui.get_mut::<CheckBox>("GameAngle").set_checked(viewport.game_angle());
        }

        // The status bar: the shortcuts, what the last save said, and
        // whether there are changes to save.
        let mut items = vec![
            Status::shortcut("Ctrl+S", "Save"),
            Status::shortcut("Ctrl+Z", "Undo"),
            Status::shortcut("Ctrl+Y", "Redo"),
        ];
        if self.mode == PLAY {
            items.push(Status::shortcut("F1", if self.game.show_card { "Hide the card" } else { "Show the card" }));
        } else {
            items.push(Status::shortcut("Space", "Pan, with a drag"));
            items.push(Status::shortcut("Wheel", "Zoom"));
        }
        items.push(Status::shortcut("F10", "VSync"));
        items.push(Status::shortcut("F11", "Fullscreen"));
        items.push(Status::Spacer);
        if !self.status.is_empty() {
            items.push(Status::text(&self.status));
        }
        items.push(if self.unsaved_now {
            Status::indicator(Severity::Warning, "Unsaved changes")
        } else {
            Status::indicator(Severity::Success, "No unsaved changes")
        });
        ui.get_mut::<user_interface::StatusBar>("StatusBar").set_items(items);
    }

    // Play: the controls drive the car, and the wheel over the view zooms.
    fn play(&mut self, ui: &mut Ui) {
        let response = ui.response("Play/View");
        let controls = Controls::read(ui, f64::from(response.wheel.y));
        self.game.update(&controls, ui.time());
        if self.game.show_card && ui.has_node("Play/Card") {
            sync_play_card(&self.game, ui);
        }
    }

    // What the 3D view shows: the scene's decorations and parked vehicles,
    // and the car, where it is in Play, and at the start in the editors, or
    // in Vehicles, the vehicle alone.
    fn build_view(&mut self) {
        let game = &self.game;
        let document = &game.document;
        let models = &game.models;
        let mut instances = Vec::new();
        if self.mode == VEHICLES {
            if let Some((model, materials)) = document.vehicle_materials(models, game.vehicle) {
                instances.push(Instance {
                    model,
                    position: [0.0, 0.0],
                    heading: SHOWN_HEADING.to_radians(),
                    scale: 1.0,
                    materials,
                });
            }
            self.eye = self.vehicle_editor.viewport.eye;
            self.instances = instances;
            return;
        }
        let scene = game.current_scene();
        for decoration in &scene.decorations {
            if let Some(model) = models.find(&decoration.model) {
                instances.push(Instance {
                    model,
                    position: decoration.pose.position,
                    heading: decoration.pose.radians(),
                    scale: decoration.scale,
                    materials: document.model_materials(models, model),
                });
            }
        }
        for parked in &scene.parked {
            if let Some(vehicle) = document.vehicles.find(&parked.vehicle)
                && let Some((model, materials)) = document.vehicle_materials(models, vehicle)
            {
                instances.push(Instance {
                    model,
                    position: parked.pose.position,
                    heading: parked.pose.radians(),
                    scale: 1.0,
                    materials,
                });
            }
        }
        let (position, heading) = if self.mode == PLAY {
            (game.car.position, game.car.heading)
        } else {
            (scene.track.start.position, scene.track.start.radians())
        };
        if let Some((model, materials)) = document.vehicle_materials(models, game.vehicle) {
            instances.push(Instance {
                model,
                position,
                heading,
                scale: 1.0,
                materials,
            });
        }
        self.eye = if self.mode == PLAY {
            Eye::game(game.camera_center(), game.camera.height)
        } else {
            self.viewport.eye
        };
        self.instances = instances;
    }
}

impl Program for Veehiicuul {
    fn title(&self) -> String {
        NAME.to_string()
    }

    fn build(&mut self, ui: &mut Ui) {
        // Only text fields take keyboard focus, so that Space, the arrows
        // and the other keys that drive and pan never go to a control; the
        // fields turn it back on for themselves.
        ui.set_build_focusable(false);
        self.unsaved_now = self.unsaved();
        self.build_app_bar(ui);
        ui.status_bar(Vec::new());
        let view = editing::view_area(ui);
        let scene = self.game.scene;
        match self.mode {
            PLAY => {
                ui.node("Play/View", view, View::new());
                let page = ui.page();
                if self.game.show_card {
                    build_play_card(&self.game, ui, page);
                }
            }
            SCENE => self.scene_editor.build(ui, view, &self.game.document, &self.game.models, scene),
            TERRAIN => self.terrain_editor.build(ui, view, &self.game.document.scenes[scene].terrain),
            TRACK => self.track_editor.build(ui, view, &self.game.document.scenes[scene].track, &self.game.last_shape().problems),
            _ => self.vehicle_editor.build(ui, view, &self.game.document, &self.game.models, self.game.vehicle),
        }
        self.built_fingerprint = self.fingerprint();
        self.built_mode = self.mode;
    }

    fn update(&mut self, ui: &mut Ui) {
        self.unsaved_now = self.unsaved();
        self.shortcuts(ui);
        self.events(ui);
        let view = editing::view_area(ui);
        let scene = self.game.scene;
        // The frame has the nodes of the tab it was built for; a switched
        // tab's build comes next frame.
        match self.built_mode {
            PLAY => self.play(ui),
            SCENE => {
                let models = &self.game.models;
                self.scene_editor.update(ui, view, &mut self.viewport, &mut self.game.document, models, scene);
            }
            TERRAIN => {
                let terrain = &mut self.game.document.scenes[scene].terrain;
                self.terrain_editor.update(ui, view, &mut self.viewport, terrain);
            }
            TRACK => {
                let problems = self.game.shape().problems.clone();
                let track = &mut self.game.document.scenes[scene].track;
                self.track_editor.update(ui, view, &mut self.viewport, track, &problems);
            }
            _ => {
                let models = &self.game.models;
                self.vehicle_editor.update(ui, view, &mut self.game.document, models, &mut self.game.vehicle);
            }
        }
        self.game.keep_in_document();
        // A drag is one change, as of when it's let go of.
        if !ui.input().button_down(MouseButton::Left) {
            self.commit();
        }
        self.game.shape();
        self.unsaved_now = self.unsaved();
        self.sync_bars(ui);
        // A change to what shapes the tree re-lays it out.
        let fingerprint = self.fingerprint();
        if self.built_fingerprint != fingerprint {
            self.built_fingerprint = fingerprint;
            ui.request_rebuild();
        }
        self.build_view();
    }

    fn prepare(&mut self, gpu: &mut Gpu) -> windows_core::Result<()> {
        self.drawing = Some(Drawing::new(gpu, &self.game.models)?);
        Ok(())
    }

    fn render(&mut self, gpu: &mut Gpu, frame: &Frame) -> windows_core::Result<()> {
        let Some(drawing) = &mut self.drawing else {
            return Ok(());
        };
        let scene = self.game.current_scene();
        // The Vehicles tab shows its vehicle on the theme's background.
        let (background, ground) = if self.mode == VEHICLES {
            (None, None)
        } else {
            let ground = Ground {
                terrain: &scene.terrain,
                track: &scene.track,
                shape: self.game.last_shape(),
            };
            (Some(srgb(scene.background)), Some(ground))
        };
        let view = Scene3d {
            eye: self.eye,
            background,
            ground,
            instances: &self.instances,
        };
        drawing.draw(gpu, frame, &view)
    }

    fn state(&self) -> Value {
        let mut members: Vec<(String, Value)> = vec![
            ("Mode".to_string(), MODES[self.mode].into()),
            ("Unsaved".to_string(), self.unsaved().into()),
            ("Status".to_string(), self.status.clone().into()),
            ("Undo".to_string(), self.undo.len().into()),
        ];
        members.extend(self.game.state().into_iter().map(|(name, value)| (name.to_string(), value)));
        let document = &self.game.document;
        let scene = self.game.scene;
        let track = &document.scenes[scene].track;
        members.push(("SceneEditor".to_string(), self.scene_editor.state(document, scene)));
        members.push(("TerrainEditor".to_string(), self.terrain_editor.state(&document.scenes[scene].terrain)));
        members.push(("TrackEditor".to_string(), self.track_editor.state(track, &self.game.last_shape().problems)));
        members.push(("VehicleEditor".to_string(), self.vehicle_editor.state(document, self.game.vehicle)));
        Value::Object(members)
    }
}

// What each control does in Play, with the gamepad's buttons, then the
// keyboard's keys, each on a keycap. A "+" joins keys that go together, like
// a shortcut's.
const CONTROLS: [(&str, &[&str], &[&str]); 9] = [
    ("Drive", &["Right stick"], &["Arrows", "WASD"]),
    ("Brake", &["LT"], &["Space"]),
    ("Back to the start", &["X"], &["R"]),
    ("Vehicle", &["D-pad"], &["Q", "E"]),
    ("Scene", &["D-pad"], &["PageUp", "PageDown"]),
    ("Camera", &["Menu"], &["C"]),
    ("Zoom", &["RB+Left stick"], &["Wheel"]),
    ("Zoom back", &["Y"], &["Home"]),
    ("Hide this", &["View"], &["F1"]),
];
// The label column's width in the card, the gap before the keyboard's
// column, and a row of keycaps' height.
const LABEL_WIDTH: f32 = 136.0;
const COLUMN_GAP: f32 = SPACE_XL;
const KEYCAP_ROW: f32 = 28.0;
// The gap between the card's lines of what's going on.
const LINE_GAP: f32 = SPACE_XS;
// Draws the keycaps of the ways to do something from `origin`, or only
// measures them without it, and returns their width. Each way is a key, or
// keys that go together, like "RB+Left stick", which Ui::keys joins.
fn keycaps(ui: &mut Ui, origin: Option<Point>, ways: &[&str]) -> f32 {
    let mut width = 0.0;
    for (index, way) in ways.iter().enumerate() {
        if index > 0 {
            width += SPACE_S;
        }
        width += match origin {
            Some(origin) => ui.keys(Point::new(origin.x + width, origin.y), way),
            None => ui.keys_width(way),
        };
    }
    width
}

// The play card's lines, from the game.
fn play_rows(game: &Game, gamepad_connected: bool) -> [(&'static str, String); 6] {
    let vehicle = game.current_vehicle();
    let rates = vehicle.acceleration;
    let camera = if game.camera.follow { "Following the car" } else { "Fixed" };
    let gamepad = if gamepad_connected { "Connected" } else { "None, so drive with the keys" };
    [
        ("Vehicle", format!("{}, {} of {}", vehicle.name, game.vehicle + 1, game.document.vehicles.list.len())),
        (
            "Accelerates",
            format!("{} ahead, {} in reverse, {} left, {} right", rates.forward, rates.reverse, rates.left, rates.right),
        ),
        ("Speed", format!("{:.1} units a second", game.car.speed())),
        ("Camera", format!("{camera}, {:.0} units tall", game.camera.height)),
        ("Crashes", game.crashes.to_string()),
        ("Gamepad", gamepad.to_string()),
    ]
}

// The card over the bottom left corner of the page in Play: the scene, the
// vehicle, how it's going, and what the controls do, as a table of the
// gamepad's and the keyboard's. It's as big as what it says, and scripts and
// tests see it as "Play/Card". Returns its area.
pub fn build_play_card(game: &Game, ui: &mut Ui, page: Rect) -> Rect {
    let gamepad_connected = ui.gamepad().connected;
    let rows = play_rows(game, gamepad_connected);
    let problem = game.last_shape().problems.first().cloned();
    let scene = game.current_scene().name.clone();
    let scenes = format!("Scene {} of {}", game.scene + 1, game.document.scenes.len());

    // As wide as its widest line.
    let title_width = ui.measure(&scene, BODY_STRONG).width + SPACE_S + ui.measure(&scenes, CAPTION).width;
    let rows_width = rows
        .iter()
        .map(|(_, value)| LABEL_WIDTH + ui.measure(value, BODY).width)
        .fold(0.0, f32::max);
    let (gamepad_header, keyboard_header) = (ui.measure("Gamepad", CAPTION).width, ui.measure("Keyboard", CAPTION).width);
    let gamepad_width = CONTROLS
        .iter()
        .map(|(_, gamepad, _)| keycaps(ui, None, gamepad))
        .fold(gamepad_header, f32::max)
        .ceil();
    let keyboard_width = CONTROLS
        .iter()
        .map(|(_, _, keyboard)| keycaps(ui, None, keyboard))
        .fold(keyboard_header, f32::max);
    let controls_width = LABEL_WIDTH + gamepad_width + COLUMN_GAP + keyboard_width;
    let content_width = title_width
        .max(rows_width)
        .max(controls_width)
        .min(page.width - CARD_PADDING * 2.0)
        .ceil();
    let problem_bar = problem
        .as_deref()
        .map(|problem| InfoBar::new(Severity::Warning, "The track has a problem", problem));
    let problem_height = problem_bar.as_ref().map_or(0.0, |bar| SPACE_M + bar.height(ui, content_width));
    let rows_height = BODY.line_height * rows.len() as f32 + LINE_GAP * (rows.len() - 1) as f32;
    let heading_height = BODY_STRONG.line_height + SPACE_XS + 1.0;
    let controls_height = SPACE_L + heading_height + SPACE_S + KEYCAP_ROW * CONTROLS.len() as f32;
    let content_height = rows_height + problem_height + controls_height;
    let card = Card::new(&scene).subtitle(&scenes).floating().listed().label(&scene);
    let height = card.height_around(content_height);
    let area = Rect::new(page.x, page.bottom() - height, content_width + CARD_PADDING * 2.0, height);
    ui.card("Play/Card", area, card, |ui, card| {
        let mut content = card.content;
        for (index, (label, value)) in rows.iter().enumerate() {
            let gap = if index + 1 < rows.len() { LINE_GAP } else { 0.0 };
            ui.node(&format!("Play/{label}"), content.cut_top(BODY.line_height, gap), Field::new(label, LABEL_WIDTH, value));
        }
        if let Some(bar) = problem_bar {
            content.cut_top(SPACE_M, 0.0);
            let height = bar.height(ui, content.width);
            ui.node("Play/Problem", content.cut_top(height, 0.0), bar);
        }
        // The table's heading, with its columns' under a divider, then the
        // keycaps, which never change.
        content.cut_top(SPACE_L, 0.0);
        ui.node("Play/Controls", content, Canvas::new(gamepad_width, draw_controls_table));
    });
    area
}

// Keeps the play card's lines true to the game.
fn sync_play_card(game: &Game, ui: &mut Ui) {
    let gamepad_connected = ui.gamepad().connected;
    for (label, value) in play_rows(game, gamepad_connected) {
        ui.get_mut::<Field>(&format!("Play/{label}")).set_value(&value);
    }
}

// The controls table: its heading, its columns' names, and each control's
// keycaps.
fn draw_controls_table(ui: &mut Ui, content: Rect, gamepad_width: &f32) {
    let theme = ui.theme().clone();
    let heading_height = BODY_STRONG.line_height + SPACE_XS + 1.0;
    let mut content = content;
    let heading = content.cut_top(heading_height, SPACE_S);
    let line = heading.top_strip(BODY_STRONG.line_height);
    let keyboard_x = line.x + LABEL_WIDTH + gamepad_width + COLUMN_GAP;
    ui.heading(line, "Controls");
    ui.text(Rect::new(line.x + LABEL_WIDTH, line.y, *gamepad_width, line.height), "Gamepad", CAPTION, theme.text_secondary);
    ui.text(Rect::new(keyboard_x, line.y, line.right() - keyboard_x, line.height), "Keyboard", CAPTION, theme.text_secondary);
    ui.divider(heading.bottom_strip(1.0));
    for (does, gamepad, keyboard) in CONTROLS {
        let row = content.cut_top(KEYCAP_ROW, 0.0);
        let top = (row.center().y - KEYCAP_HEIGHT / 2.0).round();
        ui.text(row.left_strip(LABEL_WIDTH - SPACE_S), does, BODY, theme.text_secondary);
        keycaps(ui, Some(Point::new(row.x + LABEL_WIDTH, top)), gamepad);
        keycaps(ui, Some(Point::new(keyboard_x, top)), keyboard);
    }
}
