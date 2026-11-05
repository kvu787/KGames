// The game as a Program, which Library/UserInterface's main loop runs: each
// frame, it reads the controls and moves the game to the frame's time, and
// a box of text over the bottom left corner says what's going on and what
// the controls do. Scene.rs draws the game under it.

use json::Value;
use user_interface::theme::{DIM, SUCCESS, TEXT, TITLE, WARNING};
use user_interface::{Color, Frame, MARGIN, PADDING, Program, Setup};
use windows_api::*;
use windows_core::{Error, Result};

use crate::NAME;
use crate::controls::Controls;
use crate::game::Game;
use crate::scene::Scene;
use crate::settings::GameSettings;

// A line of the text box, as pieces of text, each in its color.
pub type Line = Vec<(String, Color)>;

pub struct Veehiicuul {
    pub game: Game,
    // What draws it, which tests do without.
    scene: Option<Scene>,
}

impl Veehiicuul {
    // The game without anything to draw it, for tests.
    pub fn without_scene(game: Game) -> Veehiicuul {
        Veehiicuul { game, scene: None }
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
        let scene = unsafe { Scene::new(setup.renderer, &game)? };
        Ok(Veehiicuul {
            game,
            scene: Some(scene),
        })
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
        Ok(())
    }

    unsafe fn draw_under(&mut self, renderer: &graphics::Renderer) -> Result<()> {
        match &self.scene {
            Some(scene) => unsafe { scene.draw(renderer, &self.game) },
            None => Ok(()),
        }
    }

    fn state(&self) -> Value {
        self.game.state()
    }
}

// The text box's lines.
pub fn text(game: &Game, frame: &Frame) -> Vec<Line> {
    let piece = |text: &str, color| (text.to_string(), color);
    let vehicle = game.current_vehicle();
    let rates = vehicle.acceleration;
    let rate = |rate: f64| format!("{rate}");
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
    let mut lines = vec![
        vec![piece(NAME, TITLE), piece(&format!("  {}", game.current_track().name), DIM)],
        vec![
            piece("Vehicle: ", DIM),
            piece(&vehicle.name, TEXT),
            piece(
                &format!(
                    "  {} of {}, accelerates {} ahead, {} in reverse, {} left, {} right",
                    game.vehicle + 1,
                    game.vehicles.list.len(),
                    rate(rates.forward),
                    rate(rates.reverse),
                    rate(rates.left),
                    rate(rates.right)
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
            ("PageUp PageDown", " track   "),
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
