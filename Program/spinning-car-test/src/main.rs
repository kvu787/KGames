// No console window in any build.
#![windows_subsystem = "windows"]

use std::process::ExitCode;

use spinning_car_test::options;
use spinning_car_test::spinning_car::SpinningCar;

fn main() -> ExitCode {
    user_interface::run(options(), |startup| Ok(SpinningCar::start(startup)))
}
