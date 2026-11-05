// No console window in any build.
#![windows_subsystem = "windows"]

use std::process::ExitCode;

use game_input_demo::demo::Demo;
use user_interface::Options;

fn main() -> ExitCode {
    // The demo reads gamepads itself, with GameInput, so the library doesn't.
    let options = Options {
        gamepad: false,
        ..Options::new("GameInputDemo", env!("CARGO_MANIFEST_DIR"))
    };
    user_interface::run(options, |startup| Ok(Demo::new(startup)))
}
