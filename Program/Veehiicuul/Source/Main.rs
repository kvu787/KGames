// No console window in any build.
#![windows_subsystem = "windows"]

use std::process::ExitCode;

use user_interface::Options;
use veehiicuul::program::Veehiicuul;

fn main() -> ExitCode {
    let options = Options {
        gamepad: true,
        ..Options::new(veehiicuul::NAME, veehiicuul::FOLDER)
    };
    user_interface::run(options, Veehiicuul::start)
}
