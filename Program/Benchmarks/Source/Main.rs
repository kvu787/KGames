// Runs the benchmarks and prints their tables, and writes them as JSON if
// asked. It has no window. See README.md.
//
//     Benchmarks [--filter <text>] [--quick] [--json <path>] [--list]

use std::process::ExitCode;

use benchmarks::memory::{table as memory_table, video_table};
use benchmarks::{
    Report, USAGE, environment, parse_arguments, process_memory, report, selected, selected_memory, table,
};

fn run() -> Result<(), String> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let options = parse_arguments(&arguments).map_err(|message| format!("{message}\nUsage: {USAGE}"))?;
    let groups = selected(options.filter.as_deref());
    let memory_groups = selected_memory(options.filter.as_deref());
    if groups.is_empty() && memory_groups.is_empty() {
        return Err(format!("No group's name has {:?}. Use --list to see them.", options.filter.unwrap_or_default()));
    }
    if options.list {
        for group in groups {
            println!("{}{}", group.name, if group.gpu { " (GPU)" } else { "" });
        }
        for group in memory_groups {
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
    let mut memory = Report::default();
    for group in memory_groups {
        eprintln!("Running {}...", group.name);
        memory.add((group.run)(options.budget));
    }
    let process = process_memory();
    println!();
    if !measurements.is_empty() {
        print!("{}", table(&measurements));
    }
    if !memory.heap.is_empty() {
        println!();
        print!("{}", memory_table(&memory.heap));
    }
    if !memory.video.is_empty() {
        println!();
        print!("{}", video_table(&memory.video));
    }
    println!(
        "\nThis process: {:.1} MiB of working set, {:.1} MiB at its peak, {:.1} MiB of private commit.",
        process.working_set as f64 / (1024.0 * 1024.0),
        process.peak_working_set as f64 / (1024.0 * 1024.0),
        process.private_commit as f64 / (1024.0 * 1024.0)
    );
    if let Some(path) = &options.json {
        std::fs::write(path, report(&environment, &measurements, &memory, process).to_json() + "\n")
            .map_err(|error| format!("Couldn't write {path}: {error}"))?;
        println!("Wrote {path}.");
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
