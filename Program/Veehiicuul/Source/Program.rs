// The game as a Program, which Library/UserInterface's main loop runs. Tabs
// at the top go between playing and the editors, which make the game's
// scenes and vehicles:
//
// | Tab      | What it does                                                         |
// | -------- | -------------------------------------------------------------------- |
// | Play     | Drives the current scene's track, with a box of text over the bottom |
// |          | left corner that says what's going on and what the controls do       |
// | Scene    | Arranges decorations and parked vehicles, and sets the background,   |
// |          | the camera, and models' paint                                        |
// | Terrain  | Paints the hexel terrain                                             |
// | Track    | Builds the track from curves, which make the road and its barriers   |
// | Vehicles | Edits the vehicles, their footprints, and their paint                |
//
// Beside the tabs: the scene to play and edit, a new scene, undo, redo, and
// save, which writes the files that changed. Drawing.rs draws the 3D view
// under all of it.

use std::path::PathBuf;

use json::Value;
use user_interface::theme::{BACKGROUND, BORDER, DIM, PANEL, SUCCESS, TEXT, TITLE, WARNING};
use user_interface::{Choice, Color, Frame, MARGIN, PADDING, Program, Rect, Setup};
use windows_api::*;
use windows_core::{Error, Result};

use crate::NAME;
use crate::camera::Eye;
use crate::controls::Controls;
use crate::document::Document;
use crate::drawing::{Drawing, Ground, Instance, View};
use crate::editing::{TOP, Viewport};
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

// A line of the text box, as pieces of text, each in its color.
pub type Line = Vec<(String, Color)>;

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

    fn top_bar(&mut self, frame: &mut Frame) {
        // A strip under the tabs, so that the 3D view doesn't show through.
        let strip = TOP - MARGIN / 2.0;
        frame.canvas.rect(0.0, 0.0, frame.width, strip, PANEL);
        frame.canvas.rect(0.0, strip, frame.width, 1.0, BORDER);
        let focusable = frame.set_focusable(false);
        let height = frame.control_height();
        let mut x = MARGIN;
        let mut next = |width: f32| {
            let area = Rect::new(x, MARGIN, width, height);
            x += width + 8.0;
            area
        };
        let mut mode = self.mode;
        frame.tabs("Mode", next(520.0), &MODES, &mut mode);
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
        frame.select("Scene", next(220.0), &choices, &mut scene);
        if scene != self.game.scene {
            self.game.go_to_scene(scene, seconds);
            let current = self.game.current_scene();
            self.viewport = Viewport::new(current.camera_center, current.camera_height);
        }
        if frame.button("NewScene", next(120.0), "New scene").clicked {
            self.new_scene(seconds);
        }
        if frame.button("Undo", next(70.0), "Undo").clicked {
            self.undo();
        }
        if frame.button("Redo", next(70.0), "Redo").clicked {
            self.redo();
        }
        if frame.button("Save", next(70.0), "Save").clicked {
            self.save();
        }
        if self.mode != PLAY {
            let viewport = if self.mode == VEHICLES { &mut self.vehicle_editor.viewport } else { &mut self.viewport };
            let mut game_angle = viewport.game_angle();
            if frame.checkbox("GameAngle", next(190.0), "The game's angle", &mut game_angle).clicked {
                viewport.set_game_angle(game_angle);
            }
        }
        frame.set_focusable(focusable);
        let y = MARGIN + (height - frame.line_height()) / 2.0;
        let (text, color) = if !self.status.is_empty() {
            (self.status.as_str(), TEXT)
        } else if self.unsaved() {
            ("Unsaved changes", WARNING)
        } else {
            ("Saved", SUCCESS)
        };
        let right = frame.width - MARGIN;
        let unsaved = if self.unsaved() && !self.status.is_empty() { "  (unsaved changes)" } else { "" };
        frame.canvas.text_clipped(x + 8.0, y, &format!("{text}{unsaved}"), right - x - 8.0, color);
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
        self.top_bar(frame);
        let scene = self.game.scene;
        match self.mode {
            PLAY => {
                let controls = Controls::read(frame);
                self.game.update(&controls, frame.seconds());
                if self.game.show_text {
                    let lines = text(&self.game, frame);
                    let height = frame.line_height() * lines.len() as f32 + PADDING * 2.0;
                    let y = frame.height - MARGIN - height;
                    frame.text_box(MARGIN, y, &lines);
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
            ..
        } = self;
        let Some(drawing) = drawing else {
            return Ok(());
        };
        let scene = game.current_scene();
        let (background, ground) = if *mode == VEHICLES {
            ([BACKGROUND[0], BACKGROUND[1], BACKGROUND[2]], None)
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

// The text box's lines, in Play.
pub fn text(game: &Game, frame: &Frame) -> Vec<Line> {
    let piece = |text: &str, color| (text.to_string(), color);
    let vehicle = game.current_vehicle();
    let rates = vehicle.acceleration;
    let camera = if game.camera.follow {
        "following the car"
    } else {
        "fixed"
    };
    let status = frame.status;
    let frames = match status.frames_per_second {
        Some(rate) => format!("{rate:.0}  ({:.2} ms a frame)", 1000.0 / rate),
        None => "measuring".to_string(),
    };
    let gamepad = if frame.gamepad().connected {
        piece("connected", TEXT)
    } else {
        piece("none, so drive with the keys", DIM)
    };
    let focus = if frame.focused() {
        piece("Window has focus", SUCCESS)
    } else {
        piece("Window doesn't have focus: input is ignored", WARNING)
    };
    let problems = &game.last_shape().problems;
    let mut lines = vec![
        vec![piece(NAME, TITLE), piece(&format!("  {}", game.current_scene().name), DIM)],
        vec![
            piece("Vehicle: ", DIM),
            piece(&vehicle.name, TEXT),
            piece(
                &format!(
                    "  {} of {}, accelerates {} ahead, {} in reverse, {} left, {} right",
                    game.vehicle + 1,
                    game.document.vehicles.list.len(),
                    rates.forward,
                    rates.reverse,
                    rates.left,
                    rates.right
                ),
                DIM,
            ),
        ],
        vec![
            piece("Speed: ", DIM),
            piece(&format!("{:.1}", game.car.speed()), TEXT),
            piece(" units a second", DIM),
        ],
        vec![
            piece("Camera: ", DIM),
            piece(camera, TEXT),
            piece(&format!(", {:.0} units tall", game.camera.height), DIM),
        ],
        vec![piece("Crashes: ", DIM), piece(&game.crashes.to_string(), TEXT)],
        vec![piece("FPS: ", DIM), piece(&frames, TEXT)],
        vec![piece("Gamepad: ", DIM), gamepad],
        vec![focus],
    ];
    if let Some(problem) = problems.first() {
        lines.push(vec![piece(&format!("The track has a problem: {problem}"), WARNING)]);
    }
    let controls: [&[(&str, &str)]; 3] = [
        &[
            ("Right stick", " or "),
            ("arrows", " drive   "),
            ("LT", " or "),
            ("Space", " brakes   "),
            ("X", " or "),
            ("R", " back to the start"),
        ],
        &[
            ("D-pad", " or "),
            ("Q E", " vehicle   "),
            ("D-pad", " or "),
            ("PageUp PageDown", " scene   "),
            ("Menu", " or "),
            ("C", " camera"),
        ],
        &[
            ("RB+left stick", " or "),
            ("wheel", " zoom   "),
            ("Y", " or "),
            ("Home", " unzoom   "),
            ("View", " or "),
            ("F1", " hide this   "),
            ("F11", " fullscreen"),
        ],
    ];
    for line in controls {
        lines.push(line.iter().flat_map(|(key, does)| [piece(key, TEXT), piece(does, DIM)]).collect());
    }
    lines.extend(status.notices.iter().map(|notice| vec![piece(notice, WARNING)]));
    lines
}
