// No console window in any build.
#![windows_subsystem = "windows"]

use std::process::ExitCode;

use platformer::program::Platformer;
use user_interface::Options;

fn main() -> ExitCode {
    let options = Options::new(platformer::NAME, platformer::FOLDER).gamepad();
    user_interface::run(options, |startup| Ok(Platformer::start(startup)))
}
