// No console window in any build.
#![windows_subsystem = "windows"]

use std::process::ExitCode;

use minimal_scene_test::moving_sphere::MovingSphere;
use minimal_scene_test::options;

fn main() -> ExitCode {
    user_interface::run(options(), |startup| Ok(MovingSphere::start(startup)))
}
