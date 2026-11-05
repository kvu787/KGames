// The demo, as a Program that Library/UserInterface's main loop runs: what
// it does each frame, before Panels.rs draws what it read.
//
// The demo reads its input with GameInput, or from a script in a scripted
// run, its own shortcuts too, F11 among them. Without GameInput the window
// still opens, and says why there's no input.

use json::Value;
use user_interface::{Frame, Program, Setup};
use windows_api::*;
use windows_core::{Error, Result};

use crate::NAME;
use crate::devices::InputSource;
use crate::game_input_source::GameInputSource;
use crate::input::Input;
use crate::panels;
use crate::scripted_source::ScriptedSource;
use crate::settings::DemoSettings;

pub struct Demo {
    // The input, or why GameInput isn't available.
    pub input: std::result::Result<Input, String>,
}

impl Program for Demo {
    type Settings = DemoSettings;
    const NAME: &'static str = NAME;
    // F11 comes through GameInput, like the demo's other shortcuts.
    const FULLSCREEN_KEY: bool = false;

    fn new(setup: Setup<'_, DemoSettings>) -> Result<Self> {
        // A scripted run reads the script's devices and readings instead of
        // GameInput's.
        let source: std::result::Result<Box<dyn InputSource>, String> = if setup.scripted {
            Ok(Box::new(ScriptedSource::default()))
        } else {
            GameInputSource::new()
                .map(|source| Box::new(source) as Box<dyn InputSource>)
                .map_err(|error| format!("{} ({})", error.message(), error.code()))
        };
        let dead_zone = setup.program_settings.rumble_dead_zone;
        Ok(Demo {
            input: source.map(|source| Input::new(source, dead_zone)),
        })
    }

    fn frame(&mut self, frame: &mut Frame) -> Result<()> {
        if let Ok(input) = &mut self.input {
            if let Some(scripted) = input.scripted() {
                scripted.now = (frame.seconds() * 1_000_000.0) as u64;
                for step in &frame.input.program_steps {
                    scripted.apply(step).map_err(|problem| {
                        Error::new(E_INVALIDARG, format!("Frame {}: {problem}", frame.input.number))
                    })?;
                }
            }
            input.update(frame.focused());
            if std::mem::take(&mut input.fullscreen_toggled) {
                frame.toggle_fullscreen();
            }
        }
        panels::draw(frame, &mut self.input);
        Ok(())
    }

    // What the panels show, for scripted runs to save and check. The main
    // loop adds the interface's state, like whether the select box's list is
    // open, and which device's tooltip shows.
    fn state(&self) -> Value {
        match &self.input {
            Ok(input) => input.state(),
            Err(_) => Value::Object(Vec::new()),
        }
    }

    // A minimized window doesn't have focus, but minimizing it takes its
    // focus in the same messages, so no frame sees it without focus. Without
    // this, its input would still count as focused when it's restored, and
    // what was typed into other windows meanwhile would be handled then. This
    // drops that, stops the motors, and makes restoring it count as getting
    // focus. The main loop closes the gamepad list, like losing focus does.
    fn minimized(&mut self) {
        if let Ok(input) = &mut self.input {
            input.update(false);
        }
    }
}
