// Prints each display's identifier, with its model, mode, place on the
// desktop and device path. Set "Display" in a program's settings file to an
// identifier, or run the program with --display and the identifier, to open
// it on that display.
//
//     IdentifyDisplays
//
// It has no window. See README.md.

use std::process::ExitCode;

use identify_displays::{USAGE, check_arguments, report};

fn run() -> Result<(), String> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    check_arguments(&arguments).map_err(|message| format!("{message}\nUsage: {USAGE}"))?;
    let displays = displays::enumerate().map_err(|error| format!("Couldn't find the displays: {error}"))?;
    print!("{}", report(&displays));
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("IdentifyDisplays: {message}");
            ExitCode::FAILURE
        }
    }
}
