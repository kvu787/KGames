// The game as a Program, which Library/UserInterface's main loop runs: each
// frame, it reads the controls and moves the game to the frame's time, and
// a box of text over the bottom left corner says what's going on and what
// the controls do. Drawing.rs draws the scene under it: the ground, the
// decorations, the parked vehicles, and the car.

use json::Value;
use user_interface::theme::{DIM, SUCCESS, TEXT, TITLE, WARNING};
use user_interface::{Color, Frame, MARGIN, PADDING, Program, Setup};
use windows_api::*;
use windows_core::{Error, Result};

use crate::NAME;
use crate::camera::Eye;
use crate::controls::Controls;
use crate::drawing::{Drawing, Ground, Instance, View};
use crate::files::srgb;
use crate::game::Game;
use crate::settings::GameSettings;

// A line of the text box, as pieces of text, each in its color.
pub type Line = Vec<(String, Color)>;

pub struct Veehiicuul {
    pub game: Game,
    // What draws it, which tests do without.
    drawing: Option<Drawing>,
    // What the frame's 3D view shows.
    eye: Eye,
    instances: Vec<Instance>,
}

impl Veehiicuul {
    // The game without anything to draw it, for tests.
    pub fn without_drawing(game: Game) -> Veehiicuul {
        Veehiicuul {
            eye: Eye::game(game.camera_center(), game.camera.height),
            game,
            drawing: None,
            instances: Vec::new(),
        }
    }

    // What the 3D view shows: the scene's decorations and parked vehicles,
    // and the car.
    fn build_view(&mut self) {
        let game = &self.game;
        let document = &game.document;
        let models = &game.models;
        let mut instances = Vec::new();
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
        if let Some((model, materials)) = document.vehicle_materials(models, game.vehicle) {
            instances.push(Instance {
                model,
                position: game.car.position,
                heading: game.car.heading,
                scale: 1.0,
                materials,
            });
        }
        self.eye = Eye::game(game.camera_center(), game.camera.height);
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
        let mut program = Veehiicuul::without_drawing(game);
        program.drawing = Some(drawing);
        Ok(program)
    }

    fn frame(&mut self, frame: &mut Frame) -> Result<()> {
        let controls = Controls::read(frame);
        self.game.update(&controls, frame.seconds());
        if self.game.show_text {
            let lines = text(&self.game, frame);
            let height = frame.line_height() * lines.len() as f32 + PADDING * 2.0;
            let y = frame.height - MARGIN - height;
            frame.text_box(MARGIN, y, &lines);
        }
        self.game.shape();
        self.build_view();
        Ok(())
    }

    unsafe fn draw_under(&mut self, renderer: &graphics::Renderer) -> Result<()> {
        let Veehiicuul {
            game,
            drawing,
            eye,
            instances,
        } = self;
        let Some(drawing) = drawing else {
            return Ok(());
        };
        let scene = game.current_scene();
        let view = View {
            eye: *eye,
            background: srgb(scene.background),
            ground: Some(Ground {
                terrain: &scene.terrain,
                track: &scene.track,
                shape: game.last_shape(),
            }),
            instances,
        };
        unsafe { drawing.draw(renderer, &view) }
    }

    fn state(&self) -> Value {
        Value::Object(self.game.state().into_iter().map(|(name, value)| (name.to_string(), value)).collect())
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
