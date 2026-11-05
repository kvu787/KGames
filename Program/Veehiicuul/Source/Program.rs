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
// After the tabs: the scene to play and edit, a new scene, undo, redo, and
// save, which writes the files that changed. The status bar says whether
// there are changes to save. Drawing.rs draws the 3D view under all of it.

use std::path::PathBuf;

use json::Value;
use user_interface::{
    Button, CARD_PADDING, CONTROL_HEIGHT, Choice, Color, Frame, Icon, MARGIN, PANELS_TOP, Program, Rect, SPACE_M,
    SPACE_S, SPACE_XS, SPACE_XXL, SPACE_XXS, STATUS_BAR_HEIGHT, Setup, TAB_HEIGHT, TextStyle, keycap_width, tab_widths,
    theme, toggle_width,
};
use windows_api::*;
use windows_core::{Error, Result};

use crate::NAME;
use crate::camera::Eye;
use crate::controls::Controls;
use crate::document::Document;
use crate::drawing::{Drawing, Ground, Instance, View};
use crate::editing::{self, Viewport};
use crate::files::{self, srgb};
use crate::game::Game;
use crate::scene::Scene;
use crate::scene_editor::SceneEditor;
use crate::settings::GameSettings;
use crate::terrain_editor::TerrainEditor;
use crate::track_editor::TrackEditor;
use crate::vehicle_editor::{SHOWN_HEADING, VehicleEditor};

pub const MODES: [&str; 5] = ["Play", "Scene", "Terrain", "Track", "Vehicles"];
pub const PLAY: usize = 0;
pub const SCENE: usize = 1;
pub const TERRAIN: usize = 2;
pub const TRACK: usize = 3;
pub const VEHICLES: usize = 4;
// How many changes undo goes back.
const HISTORY: usize = 200;
// The middle of the app bar's line, where its controls are.
const APP_BAR_MIDDLE: f32 = PANELS_TOP / 2.0;
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
    // Where Save writes, the game's folder, or a script's output folder.
    save_folder: PathBuf,
    // What the last save, or anything else, said.
    pub status: String,
    // What draws it, which tests do without.
    drawing: Option<Drawing>,
    // What the frame's 3D view shows.
    eye: Eye,
    instances: Vec<Instance>,
    // The theme's background, which the Vehicles tab shows its vehicle on.
    backdrop: Color,
}

impl Veehiicuul {
    pub fn with(game: Game, save_folder: PathBuf, drawing: Option<Drawing>) -> Veehiicuul {
        let scene = game.current_scene();
        let viewport = Viewport::new(scene.camera_center, scene.camera_height);
        Veehiicuul {
            committed: game.document.clone(),
            saved: game.document.clone(),
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
            drawing,
            instances: Vec::new(),
            backdrop: theme::LIGHT.background,
        }
    }

    // The game without anything to draw it, for tests, saving to a folder.
    pub fn without_drawing(game: Game, save_folder: PathBuf) -> Veehiicuul {
        Veehiicuul::with(game, save_folder, None)
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

    // The bars at the top and the bottom, which are opaque, so that the 3D
    // view doesn't show through them, and in the editors, the window's
    // background left of their view, under their cards.
    fn backgrounds(&self, frame: &mut Frame) {
        let theme = frame.theme();
        let (width, bottom) = (frame.width, frame.content_bottom());
        let view_left = editing::view_area(frame).x;
        let canvas = &mut frame.canvas;
        canvas.fill(Rect::new(0.0, 0.0, width, PANELS_TOP), 0.0, theme.background);
        canvas.fill(Rect::new(0.0, bottom, width, STATUS_BAR_HEIGHT), 0.0, theme.background);
        if self.mode != PLAY {
            canvas.fill(Rect::from_edges(0.0, PANELS_TOP, view_left, bottom), 0.0, theme.background);
        }
    }

    // The app bar: the title, the tabs, the scene, its buttons, and in the
    // editors, whether their view is at the game's angle.
    fn app_bar(&mut self, frame: &mut Frame) {
        frame.header(NAME, "");
        let focusable = frame.set_focusable(false);
        let fonts = frame.fonts();
        let control = |x: f32, width: f32| Rect::new(x, APP_BAR_MIDDLE - CONTROL_HEIGHT / 2.0, width, CONTROL_HEIGHT);
        let mut x = MARGIN + fonts.measure(NAME, TextStyle::Subtitle).ceil() + SPACE_XXL;
        let tabs_width: f32 = tab_widths(fonts, &MODES).iter().sum();
        let mut mode = self.mode;
        frame.tabs("Mode", Rect::new(x, APP_BAR_MIDDLE - TAB_HEIGHT / 2.0, tabs_width, TAB_HEIGHT), &MODES, &mut mode);
        x += tabs_width + SPACE_XXL;
        let seconds = frame.seconds();
        if mode != self.mode {
            self.mode = mode;
            self.game.keep_in_document();
            if mode == PLAY {
                self.game.put_at_start(seconds);
            }
        }
        let names: Vec<&str> = self.game.document.scenes.iter().map(|scene| scene.name.as_str()).collect();
        let choices: Vec<Choice> = names.iter().map(|label| Choice { label, detail: "" }).collect();
        let mut scene = self.game.scene;
        frame.select("Scene", control(x, SCENE_WIDTH), &choices, &mut scene);
        x += SCENE_WIDTH + SPACE_S;
        if scene != self.game.scene {
            self.game.go_to_scene(scene, seconds);
            let current = self.game.current_scene();
            self.viewport = Viewport::new(current.camera_center, current.camera_height);
        }
        let buttons = [
            ("NewScene", Button::new("New scene").icon(Icon::Add)),
            ("Undo", Button::new("Undo")),
            ("Redo", Button::new("Redo")),
            ("Save", Button::new("Save").icon(Icon::Save)),
        ];
        for (index, (name, button)) in buttons.into_iter().enumerate() {
            let width = frame.button_width(&button);
            if frame.button(name, control(x, width), button).clicked {
                match index {
                    0 => self.new_scene(seconds),
                    1 => self.undo(),
                    2 => self.redo(),
                    _ => self.save(),
                }
            }
            x += width + SPACE_S;
        }
        if self.mode != PLAY {
            let label = "The game's angle";
            let area = control(x + SPACE_M, toggle_width(fonts, label, false));
            let viewport = if self.mode == VEHICLES { &mut self.vehicle_editor.viewport } else { &mut self.viewport };
            let mut game_angle = viewport.game_angle();
            if frame.checkbox("GameAngle", area, label, &mut game_angle).clicked {
                viewport.set_game_angle(game_angle);
            }
        }
        frame.set_focusable(focusable);
    }

    // The status bar: the shortcuts, and whether there are changes to save.
    fn status_bar(&self, frame: &mut Frame) {
        let theme = frame.theme();
        let mut shortcuts = vec![("Ctrl+S", "save"), ("Ctrl+Z", "undo"), ("Ctrl+Y", "redo")];
        if self.mode == PLAY {
            shortcuts.push(("F1", "hide the card"));
        } else {
            shortcuts.extend([("Space", "and a drag pan"), ("Wheel", "zooms")]);
        }
        shortcuts.push(("F11", "fullscreen"));
        let unsaved = if self.unsaved() && !self.status.is_empty() { "  Unsaved changes" } else { "" };
        let (text, color) = if !self.status.is_empty() {
            (self.status.as_str(), theme.text_secondary)
        } else if self.unsaved() {
            ("Unsaved changes", theme.caution)
        } else {
            ("Saved", theme.success)
        };
        frame.footer(&shortcuts, &[(text, color), (unsaved, theme.caution)]);
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
    type Settings = GameSettings;
    const NAME: &'static str = NAME;
    const DEPTH: bool = true;
    const GAMEPAD: bool = true;

    fn new(setup: Setup<'_, GameSettings>) -> Result<Self> {
        let game = Game::load(setup.folder, &setup.program_settings, setup.notices)
            .map_err(|problem| Error::new(E_FAIL, problem))?;
        let drawing = unsafe { Drawing::new(setup.renderer, &game.models)? };
        // A scripted run saves in its output folder, and leaves the game's
        // files alone.
        let save_folder = setup.arguments.file(setup.folder, "");
        Ok(Veehiicuul::with(game, save_folder, Some(drawing)))
    }

    fn frame(&mut self, frame: &mut Frame) -> Result<()> {
        if frame.control_shortcut('S' as i32) {
            self.save();
        }
        if frame.control_shortcut('Z' as i32) {
            self.undo();
        }
        if frame.control_shortcut('Y' as i32) {
            self.redo();
        }
        self.backdrop = frame.theme().background;
        self.backgrounds(frame);
        self.app_bar(frame);
        self.status_bar(frame);
        let scene = self.game.scene;
        match self.mode {
            PLAY => {
                let controls = Controls::read(frame);
                self.game.update(&controls, frame.seconds());
                if self.game.show_card {
                    play_card(&self.game, frame);
                }
            }
            SCENE => self.scene_editor.frame(frame, &mut self.viewport, &mut self.game.document, &self.game.models, scene),
            TERRAIN => {
                let terrain = &mut self.game.document.scenes[scene].terrain;
                self.terrain_editor.frame(frame, &mut self.viewport, terrain);
            }
            TRACK => {
                let shape = self.game.shape();
                let problems = shape.problems.clone();
                let track = &mut self.game.document.scenes[scene].track;
                self.track_editor.frame(frame, &mut self.viewport, track, &problems);
            }
            _ => self.vehicle_editor.frame(frame, &mut self.game.document, &self.game.models, &mut self.game.vehicle),
        }
        self.game.keep_in_document();
        if frame.held_name().is_none() {
            self.commit();
        }
        self.game.shape();
        self.build_view();
        Ok(())
    }

    unsafe fn draw_under(&mut self, renderer: &graphics::Renderer) -> Result<()> {
        let Veehiicuul {
            game,
            mode,
            drawing,
            eye,
            instances,
            backdrop,
            ..
        } = self;
        let Some(drawing) = drawing else {
            return Ok(());
        };
        let scene = game.current_scene();
        let (background, ground) = if *mode == VEHICLES {
            let [red, green, blue, _] = backdrop.to_floats();
            ([red, green, blue], None)
        } else {
            let ground = Ground {
                terrain: &scene.terrain,
                track: &scene.track,
                shape: game.last_shape(),
            };
            (srgb(scene.background), Some(ground))
        };
        let view = View {
            eye: *eye,
            background,
            ground,
            instances,
        };
        unsafe { drawing.draw(renderer, &view) }
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
// keyboard's keys, each on a keycap. A "+" joins keycaps that go together.
const CONTROLS: [(&str, &[&str], &[&str]); 9] = [
    ("Drive", &["Right stick"], &["Arrows"]),
    ("Brake", &["LT"], &["Space"]),
    ("Back to the start", &["X"], &["R"]),
    ("Vehicle", &["D-pad"], &["Q", "E"]),
    ("Scene", &["D-pad"], &["PageUp", "PageDown"]),
    ("Camera", &["Menu"], &["C"]),
    ("Zoom", &["RB+Left stick"], &["Wheel"]),
    ("Zoom back", &["Y"], &["Home"]),
    ("Hide this", &["View"], &["F1"]),
];
// The label column's width in the card, and the height of a row of keycaps.
const LABEL_WIDTH: f32 = 136.0;
const KEYCAP_ROW: f32 = 28.0;

// How wide keycaps are, side by side.
fn keycaps_width(frame: &Frame, keys: &[&str]) -> f32 {
    let fonts = frame.fonts();
    let caps: f32 = keys
        .iter()
        .flat_map(|keys| keys.split('+'))
        .map(|key| keycap_width(fonts, key) + SPACE_XXS)
        .sum();
    caps + SPACE_XS * keys.len() as f32
}

// Draws keycaps from `x`, on a row whose middle is `middle`, and returns
// where they end.
fn keycaps(frame: &mut Frame, mut x: f32, middle: f32, keys: &[&str]) -> f32 {
    for (index, joined) in keys.iter().enumerate() {
        if index > 0 {
            x += SPACE_XS;
        }
        for (part, key) in joined.split('+').enumerate() {
            if part > 0 {
                x += SPACE_XXS;
            }
            x = frame.keycap([x, middle], key).right();
        }
    }
    x
}

// The card over the bottom left corner in Play: the scene, the vehicle, how
// it's going, and what the controls do. It's as big as what it says.
// Returns its area.
pub fn play_card(game: &Game, frame: &mut Frame) -> Rect {
    let theme = frame.theme();
    let fonts = frame.fonts();
    let vehicle = game.current_vehicle();
    let rates = vehicle.acceleration;
    let camera = if game.camera.follow { "following the car" } else { "fixed" };
    let gamepad = if frame.gamepad().connected { "connected" } else { "none, so drive with the keys" };
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
    let problem = game.last_shape().problems.first().map(|problem| format!("The track has a problem: {problem}"));
    let scene = &game.current_scene().name;
    let line = frame.line_height(TextStyle::Body);
    let or_width = fonts.measure("or", TextStyle::Caption) + SPACE_S * 2.0;

    // As wide as its widest line.
    let title_width = fonts.measure(NAME, TextStyle::BodyStrong) + SPACE_M + fonts.measure(scene, TextStyle::Body);
    let rows_width = rows
        .iter()
        .map(|(_, value)| LABEL_WIDTH + fonts.measure(value, TextStyle::Body))
        .fold(0.0, f32::max);
    let controls_width = CONTROLS
        .iter()
        .map(|(_, gamepad, keyboard)| LABEL_WIDTH + keycaps_width(frame, gamepad) + or_width + keycaps_width(frame, keyboard))
        .fold(0.0, f32::max);
    let problem_width = problem.as_ref().map_or(0.0, |problem| 24.0 + fonts.measure(problem, TextStyle::Body));
    let content_width = title_width
        .max(rows_width)
        .max(controls_width)
        .max(problem_width)
        .min(frame.width - MARGIN * 2.0 - CARD_PADDING * 2.0)
        .ceil();
    let problem_height = if problem.is_some() { line } else { 0.0 };
    let content_height =
        line + SPACE_S + line * rows.len() as f32 + problem_height + SPACE_M + KEYCAP_ROW * CONTROLS.len() as f32;
    let area = Rect::new(
        MARGIN,
        frame.content_bottom() - MARGIN - content_height - CARD_PADDING * 2.0,
        content_width + CARD_PADDING * 2.0,
        content_height + CARD_PADDING * 2.0,
    );
    frame.overlay_card(area);

    let inner = area.inset(CARD_PADDING);
    let canvas = &mut frame.canvas;
    canvas.push_clip(inner);
    let mut y = inner.y;
    let end = canvas.text([inner.x, y], NAME, TextStyle::BodyStrong, theme.text);
    canvas.text([end + SPACE_M, y], scene, TextStyle::Body, theme.text_secondary);
    y += line + SPACE_S;
    for (label, value) in &rows {
        canvas.text([inner.x, y], label, TextStyle::Body, theme.text_secondary);
        canvas.text([inner.x + LABEL_WIDTH, y], value, TextStyle::Body, theme.text);
        y += line;
    }
    if let Some(problem) = &problem {
        canvas.icon([inner.x + 8.0, y + line * 0.5], Icon::Warning, 16.0, theme.caution);
        canvas.text([inner.x + 24.0, y], problem, TextStyle::Body, theme.text);
        y += line;
    }
    canvas.pop_clip();
    y += SPACE_M;
    let caption = frame.line_height(TextStyle::Caption);
    for (does, gamepad, keyboard) in CONTROLS {
        let middle = y + KEYCAP_ROW * 0.5;
        frame.canvas.text([inner.x, middle - line * 0.5], does, TextStyle::Body, theme.text_secondary);
        let x = keycaps(frame, inner.x + LABEL_WIDTH, middle, gamepad);
        frame.canvas.text([x + SPACE_S, middle - caption * 0.5], "or", TextStyle::Caption, theme.text_secondary);
        keycaps(frame, x + or_width, middle, keyboard);
        y += KEYCAP_ROW;
    }
    area
}
