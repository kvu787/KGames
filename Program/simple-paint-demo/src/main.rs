// No console window in any build.
#![windows_subsystem = "windows"]

use std::process::ExitCode;

use simple_paint_demo::demo::Demo;
use user_interface::Options;

fn main() -> ExitCode {
    user_interface::run(Options::new("SimplePaintDemo", env!("CARGO_MANIFEST_DIR")), |startup| {
        Ok(Demo::start(startup))
    })
}
