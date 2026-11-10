// FrameTimes' parts that don't need an ETW session, as a library that
// Test/FrameTimes tests: the statistics of a process's frame times, from the
// presents that Library/PresentTiming gives, the command line, and finding
// the process that it names.

use std::fmt::Write;

use present_timing::{Present, PresentMode};
use windows_api::{
    CloseHandle, CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS,
};

pub const USAGE: &str = "FrameTimes <process name or ID> [--seconds N]";

// How long FrameTimes measures for, without --seconds.
pub const DEFAULT_SECONDS: f64 = 10.0;

// What the command line asks for.
#[derive(Clone, Debug, PartialEq)]
pub struct Options {
    // The process's exe name, like AceCombat8.exe or AceCombat8, or its ID.
    pub process: String,
    pub seconds: f64,
}

pub fn parse_arguments(arguments: &[String]) -> Result<Options, String> {
    let mut process = None;
    let mut seconds = DEFAULT_SECONDS;
    let mut arguments = arguments.iter();
    while let Some(argument) = arguments.next() {
        if argument == "--seconds" {
            let value = arguments.next().ok_or("--seconds needs a number of seconds after it.")?;
            seconds = match value.parse::<f64>() {
                Ok(value) if value.is_finite() && value > 0.0 => value,
                _ => return Err(format!("--seconds needs a number of seconds above 0, not {value}.")),
            };
        } else if argument.starts_with("--") {
            return Err(format!("Unknown option: {argument}. The only one is --seconds."));
        } else if let Some(first) = &process {
            return Err(format!("Name one process, not both {first} and {argument}."));
        } else {
            process = Some(argument.clone());
        }
    }
    let process = process.ok_or("Name the process to measure, by its exe's name or its ID.")?;
    Ok(Options { process, seconds })
}

// A running process.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Process {
    pub id: u32,
    // Its exe's file name, like AceCombat8.exe.
    pub name: String,
}

// Every running process, from a snapshot of the system's.
pub fn processes() -> Result<Vec<Process>, String> {
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS as u32, 0);
        // INVALID_HANDLE_VALUE.
        if snapshot.0 as isize == -1 {
            return Err(format!("Couldn't list the processes: {}", std::io::Error::last_os_error()));
        }
        let mut entry = PROCESSENTRY32W {
            dwSize: size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        let mut list = Vec::new();
        let mut more = Process32FirstW(snapshot, &mut entry).as_bool();
        while more {
            let name = &entry.szExeFile;
            let length = name.iter().position(|&character| character == 0).unwrap_or(name.len());
            list.push(Process {
                id: entry.th32ProcessID,
                name: String::from_utf16_lossy(&name[..length]),
            });
            more = Process32NextW(snapshot, &mut entry).as_bool();
        }
        let _ = CloseHandle(snapshot);
        Ok(list)
    }
}

// The process that the command line names: by its ID if it's a number, or
// else by its exe's name, regardless of case, with or without ".exe".
pub fn find_process(query: &str, processes: &[Process]) -> Result<Process, String> {
    if let Ok(id) = query.parse::<u32>() {
        return processes
            .iter()
            .find(|process| process.id == id)
            .cloned()
            .ok_or_else(|| format!("No process has the ID {id}."));
    }
    let wanted = query.to_lowercase();
    let found: Vec<&Process> = processes
        .iter()
        .filter(|process| {
            let name = process.name.to_lowercase();
            name == wanted || name.strip_suffix(".exe") == Some(wanted.as_str())
        })
        .collect();
    match found.as_slice() {
        [] => Err(format!("No process is named {query}.")),
        [process] => Ok((*process).clone()),
        _ => {
            let ids: Vec<String> = found.iter().map(|process| process.id.to_string()).collect();
            Err(format!(
                "{} processes are named {}: {}. Name the one to measure by its ID.",
                found.len(),
                found[0].name,
                ids.join(", ")
            ))
        }
    }
}

// The time from each frame on the screen to the next, in milliseconds: the
// differences between the screen times of the presents that reached the
// screen, in order, on a performance counter of `frequency` ticks a second.
// Presents that never reached the screen don't count.
pub fn frame_times(presents: &[Present], frequency: i64) -> Vec<f64> {
    let mut screens: Vec<i64> = presents.iter().filter_map(|present| present.screen).collect();
    screens.sort_unstable();
    screens
        .windows(2)
        .map(|pair| (pair[1] - pair[0]) as f64 * 1000.0 / frequency as f64)
        .collect()
}

// The arithmetic mean, or None of no values.
pub fn mean(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    Some(values.iter().sum::<f64>() / values.len() as f64)
}

fn sorted(values: &[f64]) -> Vec<f64> {
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    sorted
}

// The middle value, or the mean of the two middle values of an even count.
pub fn median(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let sorted = sorted(values);
    let middle = sorted.len() / 2;
    if sorted.len() % 2 == 1 {
        Some(sorted[middle])
    } else {
        Some((sorted[middle - 1] + sorted[middle]) / 2.0)
    }
}

// The nearest-rank percentile: the smallest value that at least `percent`
// percent of the values are at or under.
pub fn percentile(values: &[f64], percent: f64) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let sorted = sorted(values);
    // Multiplying first keeps whole ranks whole, like 99% of 100 values.
    let rank = (percent * sorted.len() as f64 / 100.0).ceil() as usize;
    Some(sorted[rank.clamp(1, sorted.len()) - 1])
}

pub fn maximum(values: &[f64]) -> Option<f64> {
    values.iter().copied().max_by(f64::total_cmp)
}

// How many frames took more than twice the median frame time: a frame that
// stayed on the screen for two of the usual frames or more, which shows as
// a hitch.
pub fn stutters(frame_times: &[f64]) -> usize {
    let Some(median) = median(frame_times) else {
        return 0;
    };
    frame_times.iter().filter(|&&time| time > 2.0 * median).count()
}

// The mode that most of the presents that reached the screen got there by,
// or None if none did. A tie goes to the mode that reached it first.
pub fn most_common_mode(presents: &[Present]) -> Option<PresentMode> {
    let mut counts: Vec<(PresentMode, usize)> = Vec::new();
    for present in presents.iter().filter(|present| present.screen.is_some()) {
        match counts.iter_mut().find(|(mode, _)| *mode == present.mode) {
            Some((_, count)) => *count += 1,
            None => counts.push((present.mode, 1)),
        }
    }
    let mut most: Option<(PresentMode, usize)> = None;
    for (mode, count) in counts {
        if most.is_none_or(|(_, most_count)| count > most_count) {
            most = Some((mode, count));
        }
    }
    most.map(|(mode, _)| mode)
}

// The statistics of a process's frame times, in milliseconds.
#[derive(Clone, Debug, PartialEq)]
pub struct Summary {
    // How many frame times there are: one fewer than the presents that
    // reached the screen.
    pub frames: usize,
    pub frames_per_second: f64,
    pub mean: f64,
    pub median: f64,
    pub percentile_99: f64,
    pub maximum: f64,
    // How many frames took more than twice the median.
    pub stutters: usize,
    // The mode that most of the presents reached the screen by.
    pub mode: PresentMode,
}

// The statistics of these presents' frame times, or None if fewer than two
// of them reached the screen, which leaves no frame time.
pub fn summarize(presents: &[Present], frequency: i64) -> Option<Summary> {
    let times = frame_times(presents, frequency);
    let mean = mean(&times)?;
    Some(Summary {
        frames: times.len(),
        // The frames over the time from the first on the screen to the last.
        frames_per_second: if mean > 0.0 { 1000.0 / mean } else { 0.0 },
        mean,
        median: median(&times)?,
        percentile_99: percentile(&times, 99.0)?,
        maximum: maximum(&times)?,
        stutters: stutters(&times),
        mode: most_common_mode(presents)?,
    })
}

// The summary as FrameTimes prints it, a line for each statistic.
pub fn report(summary: &Summary) -> String {
    let lines = [
        ("Frames", summary.frames.to_string()),
        ("Frames a second", format!("{:.1}", summary.frames_per_second)),
        ("Mean", format!("{:.2} ms", summary.mean)),
        ("Median", format!("{:.2} ms", summary.median)),
        ("99th percentile", format!("{:.2} ms", summary.percentile_99)),
        ("Maximum", format!("{:.2} ms", summary.maximum)),
        (
            "Stutters",
            format!("{}, frames over twice the median ({:.2} ms)", summary.stutters, 2.0 * summary.median),
        ),
        ("Present mode", summary.mode.name().to_string()),
    ];
    let mut text = String::new();
    for (label, value) in lines {
        let _ = writeln!(text, "  {label:<17}{value}");
    }
    text
}
