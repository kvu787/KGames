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
    AppBar, Button, Card, CheckBox, ComboBox, Icon, InfoBar, Insets, KEYCAP_HEIGHT, Key, Modifiers, MouseButton, Point, Program,
    Rect, SelectorBar, Severity, Startup, Status, Ui, View,
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

    // The app bar: the title and the tabs, in the editors whether their view
    // is at the game's angle, and at its right end, the scene and the
    // commands.
    fn app_bar(&mut self, ui: &mut Ui) {
        let mut bar = ui.app_bar(AppBar::new(Icon::Car, NAME));
        let seconds = ui.time();
        // The tabs take the bar's whole height, so that their text is in
        // its middle, with their pills under it.
        let tabs = bar.cut_left(SelectorBar::width(ui, &MODES), SPACE_XXL);
        let mut mode = self.mode;
        ui.add("Mode", Rect::new(tabs.x, ui.content_top(), tabs.width, APP_BAR_HEIGHT), SelectorBar::new(&mut mode, &MODES));
        if mode != self.mode {
            self.go_to(mode, seconds);
        }

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

        let response = ui.add("Save", save_area, save);
        ui.tooltip(&response, "Writes the files that changed (Ctrl+S)");
        if response.clicked {
            self.save();
        }
        let response = ui.add_enabled(self.can_redo(), "Redo", redo_area, redo);
        ui.tooltip(&response, "Does again what undo undid (Ctrl+Y)");
        if response.clicked {
            self.redo();
        }
        let response = ui.add_enabled(self.can_undo(), "Undo", undo_area, undo);
        ui.tooltip(&response, "Takes back the last change (Ctrl+Z)");
        if response.clicked {
            self.undo();
        }
        ui.divider(divider.pad(Insets::symmetric(0.0, SPACE_XS)));
        let response = ui.add("NewScene", new_scene_area, new_scene);
        ui.tooltip(&response, "Makes an empty scene, and goes to it");
        if response.clicked {
            self.new_scene(seconds);
        }
        let names: Vec<&str> = self.game.document.scenes.iter().map(|scene| scene.name.as_str()).collect();
        let mut scene = self.game.scene;
        ui.add("Scene", scene_area, ComboBox::new(&mut scene, &names));
        if scene != self.game.scene {
            self.game.go_to_scene(scene, seconds);
            let current = self.game.current_scene();
            self.viewport = Viewport::new(current.camera_center, current.camera_height);
        }
        let color = ui.theme().text_secondary;
        ui.text(label_area, "Scene", BODY, color);

        if self.mode != PLAY {
            let label = "The game's angle";
            let area = bar.left_strip(CheckBox::width(ui, label));
            let viewport = if self.mode == VEHICLES { &mut self.vehicle_editor.viewport } else { &mut self.viewport };
            let mut game_angle = viewport.game_angle();
            let response = ui.add("GameAngle", area, CheckBox::new(&mut game_angle, label));
            ui.tooltip(&response, "Shows the view as the game does, instead of straight down");
            if response.changed {
                viewport.set_game_angle(game_angle);
            }
        }
    }

    // The status bar: the shortcuts, what the last save said, and whether
    // there are changes to save.
    fn status_bar(&self, ui: &mut Ui) {
        let mut items = vec![
            Status::Shortcut("Ctrl+S", "Save"),
            Status::Shortcut("Ctrl+Z", "Undo"),
            Status::Shortcut("Ctrl+Y", "Redo"),
        ];
        if self.mode == PLAY {
            items.push(Status::Shortcut("F1", if self.game.show_card { "Hide the card" } else { "Show the card" }));
        } else {
            items.push(Status::Shortcut("Space", "Pan, with a drag"));
            items.push(Status::Shortcut("Wheel", "Zoom"));
        }
        items.push(Status::Shortcut("F10", "VSync"));
        items.push(Status::Shortcut("F11", "Fullscreen"));
        items.push(Status::Spacer);
        if !self.status.is_empty() {
            items.push(Status::Text(&self.status));
        }
        items.push(if self.unsaved_now {
            Status::Indicator(Severity::Warning, "Unsaved changes")
        } else {
            Status::Indicator(Severity::Success, "No unsaved changes")
        });
        ui.status_bar(&items);
    }

    // Play: the controls drive the car, and the wheel over the view zooms.
    fn play(&mut self, ui: &mut Ui, view: Rect) {
        let response = ui.add("Play/View", view, View::new());
        let controls = Controls::read(ui, f64::from(response.wheel.y));
        self.game.update(&controls, ui.time());
        let page = ui.page();
        if self.game.show_card {
            play_card(&self.game, ui, page);
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

    fn update(&mut self, ui: &mut Ui) {
        // Only text fields take keyboard focus, so that Space, the arrows
        // and the other keys that drive and pan never go to a control.
        ui.set_focusable(false);
        self.unsaved_now = self.unsaved();
        self.shortcuts(ui);
        self.app_bar(ui);
        self.status_bar(ui);
        let view = editing::view_area(ui);
        let scene = self.game.scene;
        match self.mode {
            PLAY => self.play(ui, view),
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

// The card over the bottom left corner of the page in Play: the scene, the
// vehicle, how it's going, and what the controls do, as a table of the
// gamepad's and the keyboard's. It's as big as what it says. Returns its
// area, which scripts and tests see as "Play/Card".
pub fn play_card(game: &Game, ui: &mut Ui, page: Rect) -> Rect {
    let theme = ui.theme().clone();
    let vehicle = game.current_vehicle();
    let rates = vehicle.acceleration;
    let camera = if game.camera.follow { "Following the car" } else { "Fixed" };
    let gamepad = if ui.gamepad().connected { "Connected" } else { "None, so drive with the keys" };
    let rows = [
        ("Vehicle", format!("{}, {} of {}", vehicle.name, game.vehicle + 1, game.document.vehicles.list.len())),
        (
            "Accelerates",
            format!("{} ahead, {} in reverse, {} left, {} right", rates.forward, rates.reverse, rates.left, rates.right),
        ),
        ("Speed", format!("{:.1} units a second", game.car.speed())),
        ("Camera", format!("{camera}, {:.0} units tall", game.camera.height)),
        ("Crashes", game.crashes.to_string()),
        ("Gamepad", gamepad.to_string()),
    ];
    let problem = game.last_shape().problems.first().cloned();
    let scene = &game.current_scene().name;
    let scenes = format!("Scene {} of {}", game.scene + 1, game.document.scenes.len());

    // As wide as its widest line.
    let title_width = ui.measure(scene, BODY_STRONG).width + SPACE_S + ui.measure(&scenes, CAPTION).width;
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
    let problem_height = problem_bar.map_or(0.0, |bar| SPACE_M + ui.info_bar_height(content_width, &bar));
    let rows_height = BODY.line_height * rows.len() as f32 + LINE_GAP * (rows.len() - 1) as f32;
    let heading_height = BODY_STRONG.line_height + SPACE_XS + 1.0;
    let controls_height = SPACE_L + heading_height + SPACE_S + KEYCAP_ROW * CONTROLS.len() as f32;
    let content_height = rows_height + problem_height + controls_height;
    let card = Card::new(scene).subtitle(&scenes).floating().named("Play/Card");
    let height = card.height_around(content_height);
    let area = Rect::new(page.x, page.bottom() - height, content_width + CARD_PADDING * 2.0, height);
    let card = ui.card(area, card);

    let mut content = card.content;
    for (index, (label, value)) in rows.iter().enumerate() {
        let gap = if index + 1 < rows.len() { LINE_GAP } else { 0.0 };
        ui.field(content.cut_top(BODY.line_height, gap), label, LABEL_WIDTH, value);
    }
    if let Some(bar) = problem_bar {
        content.cut_top(SPACE_M, 0.0);
        let height = ui.info_bar_height(content.width, &bar);
        ui.add("Play/Problem", content.cut_top(height, 0.0), bar);
    }
    // The table's heading, with its columns' under a divider, as a form's
    // heading has it.
    content.cut_top(SPACE_L, 0.0);
    let heading = content.cut_top(heading_height, SPACE_S);
    let line = heading.top_strip(BODY_STRONG.line_height);
    let keyboard_x = line.x + LABEL_WIDTH + gamepad_width + COLUMN_GAP;
    ui.heading(line, "Controls");
    ui.text(Rect::new(line.x + LABEL_WIDTH, line.y, gamepad_width, line.height), "Gamepad", CAPTION, theme.text_secondary);
    ui.text(Rect::new(keyboard_x, line.y, line.right() - keyboard_x, line.height), "Keyboard", CAPTION, theme.text_secondary);
    ui.divider(heading.bottom_strip(1.0));
    for (does, gamepad, keyboard) in CONTROLS {
        let row = content.cut_top(KEYCAP_ROW, 0.0);
        let top = (row.center().y - KEYCAP_HEIGHT / 2.0).round();
        ui.text(row.left_strip(LABEL_WIDTH - SPACE_S), does, BODY, theme.text_secondary);
        keycaps(ui, Some(Point::new(row.x + LABEL_WIDTH, top)), gamepad);
        keycaps(ui, Some(Point::new(keyboard_x, top)), keyboard);
    }
    area
}
