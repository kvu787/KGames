// No console window in any build.
#![windows_subsystem = "windows"]

use std::process::ExitCode;

use game_input_demo::demo::Demo;
use user_interface::Options;

fn main() -> ExitCode {
    // The demo shows every gamepad's readings, from the library's devices,
    // so they're input that its frames take, like its keys and mice.
    let options = Options {
        gamepad: true,
        ..Options::new("GameInputDemo", env!("CARGO_MANIFEST_DIR"))
    };
    user_interface::run(options, |startup| Ok(Demo::new(startup)))
}
