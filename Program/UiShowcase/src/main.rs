// No console window in any build.
#![windows_subsystem = "windows"]

use std::process::ExitCode;

use ui_showcase::showcase::Showcase;
use user_interface::Options;

fn main() -> ExitCode {
    let options = Options::new("UiShowcase", env!("CARGO_MANIFEST_DIR")).gamepad();
    user_interface::run(options, |startup| Ok(Showcase::new(startup)))
}
