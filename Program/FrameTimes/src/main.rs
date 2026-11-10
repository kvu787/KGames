// Measures another process's frame times, like a game's while a build runs:
// reads the process's presents with Library/PresentTiming for a while, and
// prints the statistics of the times between its frames on the screen.
//
//     FrameTimes <process name or ID> [--seconds N]
//
// It has no window. See README.md.

use std::process::ExitCode;
use std::time::{Duration, Instant};

use frame_times::{USAGE, find_process, parse_arguments, processes, report, summarize};
use present_timing::{PresentTiming, StartError};

const ACCESS_DENIED: &str = "\
Couldn't start the ETW session that reads the presents: it needs the Performance Log Users group,
or an elevated administrator. To join the group, once, from an elevated PowerShell, then sign out
and back in:

    Add-LocalGroupMember -Group 'Performance Log Users' -Member $env:USERNAME

See Library/PresentTiming/README.md#privileges.";

// How often the presents whose fate became known are taken from the session.
const POLL: Duration = Duration::from_millis(250);

fn run() -> Result<(), String> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let options = parse_arguments(&arguments).map_err(|message| format!("{message}\nUsage: {USAGE}"))?;
    let process = find_process(&options.process, &processes()?)?;
    let mut timing = PresentTiming::start("FrameTimes", process.id).map_err(|error| match error {
        StartError::AccessDenied => ACCESS_DENIED.to_string(),
        StartError::Failed(message) => format!("Couldn't start the ETW session that reads the presents: {message}"),
    })?;
    println!("Measuring {}, process {}, for {} seconds.", process.name, process.id, options.seconds);

    let duration = Duration::from_secs_f64(options.seconds);
    let start = Instant::now();
    let mut presents = Vec::new();
    loop {
        let left = duration.saturating_sub(start.elapsed());
        if left.is_zero() {
            break;
        }
        std::thread::sleep(left.min(POLL));
        presents.extend(timing.take());
    }
    drop(timing);

    match summarize(&presents, present_timing::frequency()) {
        Some(summary) => {
            println!();
            print!("{}", report(&summary));
            Ok(())
        }
        None => Err(format!(
            "Fewer than two of its {} presents reached the screen, which leaves no frame time. Is it drawing, \
             and is its window on a screen?",
            presents.len()
        )),
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("FrameTimes: {message}");
            ExitCode::FAILURE
        }
    }
}
