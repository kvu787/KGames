// Runs the benchmarks and prints their table, and writes them as JSON if
// asked. It has no window. See README.md.
//
//     Benchmarks [--filter <text>] [--quick] [--json <path>] [--list]

use std::process::ExitCode;

use benchmarks::{USAGE, environment, parse_arguments, report, selected, table};

fn run() -> Result<(), String> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let options = parse_arguments(&arguments).map_err(|message| format!("{message}\nUsage: {USAGE}"))?;
    let groups = selected(options.filter.as_deref());
    if groups.is_empty() {
        return Err(format!("No group's name has {:?}. Use --list to see them.", options.filter.unwrap_or_default()));
    }
    if options.list {
        for group in groups {
            println!("{}{}", group.name, if group.gpu { " (GPU)" } else { "" });
        }
        return Ok(());
    }
    let environment = environment();
    println!("Environment: {}", environment.to_json());
    println!("Budget: {:?}", options.budget);
    let mut measurements = Vec::new();
    for group in groups {
        eprintln!("Running {}...", group.name);
        measurements.extend((group.run)(options.budget));
    }
    println!();
    print!("{}", table(&measurements));
    if let Some(path) = &options.json {
        std::fs::write(path, report(&environment, &measurements).to_json() + "\n").map_err(|error| format!("Couldn't write {path}: {error}"))?;
        println!("\nWrote {path}.");
    }
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("Benchmarks: {message}");
            ExitCode::FAILURE
        }
    }
}
