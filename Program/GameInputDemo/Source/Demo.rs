// The demo, as a Program that Library/UserInterface's main loop runs: what
// it does each frame, before Cards.rs draws what it read.
//
// The demo reads its input from the devices that Library/UserInterface reads
// with GameInput, or from a script's Program steps in a scripted run, its
// shortcuts F1 and F2 too. F10 and F11 are the library's, from the same
// readings. Without GameInput the window still opens, and the library's
// notice under the app bar says why there's no input.

use json::Value;
use user_interface::testing::script::DEFAULT_FRAME_TIME;
use user_interface::{Program, Startup, Ui};

use crate::NAME;
use crate::cards;
use crate::devices::InputSource;
use crate::game_input_source::GameInputSource;
use crate::input::Input;
use crate::scripted_source::ScriptedSource;
use crate::settings::DemoSettings;

pub struct Demo {
    // The input, or why GameInput isn't available.
    pub input: Result<Input, String>,
    // The window's title, with the GPU that draws, and whether frames may
    // tear, once a frame knows them. F10 changes whether they may, so the
    // title follows it: `title_tearing` is what the title says.
    title: String,
    title_tearing: Option<bool>,
    // The last frame's time, and the time between it and the one before, in
    // seconds, which give a script's readings the time of the frame that
    // they're for.
    last_time: f64,
    last_delta: f64,
}

impl Demo {
    // The demo as the main loop starts it: on GameInput, or on a script's
    // devices and readings in a scripted run.
    pub fn new(startup: &mut Startup) -> Demo {
        let settings = DemoSettings::read(&mut startup.settings);
        let source: Result<Box<dyn InputSource>, String> = if startup.scripted {
            Ok(Box::new(ScriptedSource::default()))
        } else {
            match &startup.devices {
                Some(devices) => Ok(Box::new(GameInputSource::new(devices.clone()))),
                None => Err("GameInput isn't available".to_string()),
            }
        };
        Demo::with_input(source.map(|source| Input::new(source, settings.rumble_dead_zone)))
    }

    // The demo with this input, like a test's.
    pub fn with_input(input: Result<Input, String>) -> Demo {
        Demo {
            input,
            title: NAME.to_string(),
            title_tearing: None,
            last_time: 0.0,
            last_delta: 0.0,
        }
    }

    // Input from a test's devices and readings, with the rumble test's dead
    // zone.
    pub fn scripted(rumble_dead_zone: f32) -> Demo {
        Demo::with_input(Ok(Input::new(Box::new(ScriptedSource::default()), rumble_dead_zone)))
    }
}

impl Program for Demo {
    fn title(&self) -> String {
        self.title.clone()
    }

    fn update(&mut self, ui: &mut Ui) {
        // No widget takes keyboard focus, so that the keys stay the user's,
        // to try the Keyboard card with. The combo box's list still takes the
        // arrow keys while it's open.
        ui.set_focusable(false);
        self.last_time = ui.time();
        self.last_delta = ui.delta() as f64;
        let environment = ui.environment();
        if !environment.gpu.is_empty() && self.title_tearing != Some(environment.tearing) {
            self.title_tearing = Some(environment.tearing);
            let tearing = if environment.tearing { "frames may tear" } else { "frames don't tear" };
            self.title = format!("{NAME} - {} - {tearing}", environment.gpu);
        }

        if let Ok(input) = &mut self.input {
            if let Some(scripted) = input.scripted() {
                scripted.now = scripted.now.max(microseconds(ui.time()));
            }
            let focused = ui.window_focused();
            // A frame can both lose focus and get it back, like when focus
            // goes to another window and back between two frames. Losing
            // focus first drops what came meanwhile, stops the motors, and
            // makes getting it back count as getting focus.
            if focused && ui.input().focus_lost {
                input.update(false);
            }
            input.update(focused);
        }
        cards::draw(ui, &mut self.input);
    }

    // What the cards show, for scripts to save and check. The main loop adds
    // the interface's, like whether the combo box's list is open, and which
    // device's tooltip shows.
    fn state(&self) -> Value {
        match &self.input {
            Ok(input) => input.state(),
            Err(_) => Value::Object(Vec::new()),
        }
    }

    // Connects and disconnects a script's devices, and gives their readings.
    fn script(&mut self, step: &Value) -> Result<(), String> {
        let input = self.input.as_mut().map_err(|message| format!("GameInput isn't available: {message}"))?;
        let scripted = input.scripted().ok_or("the input doesn't come from a script")?;
        // The step comes before its frame's update, so its readings are at
        // the time of that frame: the last one's, and the time between
        // frames, which a script keeps the same.
        let frame_time = if self.last_delta > 0.0 { self.last_delta } else { DEFAULT_FRAME_TIME };
        scripted.now = scripted.now.max(microseconds(self.last_time + frame_time));
        scripted.apply(step)
    }
}

fn microseconds(seconds: f64) -> u64 {
    (seconds * 1_000_000.0).round() as u64
}
