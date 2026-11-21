// No console window in any build.
#![windows_subsystem = "windows"]

use std::process::ExitCode;

use shadow_testing::shadow_testing::{ShadowTesting, TITLE};
use user_interface::Options;

fn main() -> ExitCode {
    user_interface::run(Options::new(TITLE, env!("CARGO_MANIFEST_DIR")), |_startup| Ok(ShadowTesting::default()))
}
