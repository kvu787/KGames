// FrameTimes' command line, and finding the process that it names, by its
// exe's name or its ID.

use frame_times::{DEFAULT_SECONDS, Options, Process, find_process, parse_arguments, processes};

fn parse(arguments: &[&str]) -> Result<Options, String> {
    let arguments: Vec<String> = arguments.iter().map(|argument| argument.to_string()).collect();
    parse_arguments(&arguments)
}

fn process(id: u32, name: &str) -> Process {
    Process {
        id,
        name: name.to_string(),
    }
}

#[test]
fn the_process_and_how_long_to_measure() {
    assert_eq!(
        parse(&["AceCombat8"]),
        Ok(Options {
            process: "AceCombat8".to_string(),
            seconds: DEFAULT_SECONDS,
        })
    );
    assert_eq!(DEFAULT_SECONDS, 10.0);
    assert_eq!(
        parse(&["--seconds", "2.5", "1234"]),
        Ok(Options {
            process: "1234".to_string(),
            seconds: 2.5,
        })
    );
    assert_eq!(parse(&["AceCombat8.exe", "--seconds", "30"]).unwrap().seconds, 30.0);
}

#[test]
fn wrong_command_lines_say_whats_wrong() {
    assert!(parse(&[]).unwrap_err().contains("Name the process"));
    assert!(parse(&["--seconds", "5"]).unwrap_err().contains("Name the process"));
    assert!(parse(&["Game", "--seconds"]).unwrap_err().contains("needs a number"));
    for seconds in ["0", "-1", "ten", "inf", "NaN"] {
        assert!(parse(&["Game", "--seconds", seconds]).unwrap_err().contains("above 0"), "{seconds}");
    }
    assert!(parse(&["Game", "--frames", "5"]).unwrap_err().contains("Unknown option: --frames"));
    assert!(parse(&["Game", "Other"]).unwrap_err().contains("not both Game and Other"));
}

#[test]
fn a_process_is_found_by_its_name_without_regard_to_case_or_exe() {
    let processes = [process(4, "System"), process(1200, "AceCombat8.exe"), process(1300, "explorer.exe")];
    for query in ["AceCombat8.exe", "acecombat8.EXE", "AceCombat8", "ACECOMBAT8"] {
        assert_eq!(find_process(query, &processes), Ok(process(1200, "AceCombat8.exe")), "{query}");
    }
    assert_eq!(find_process("system", &processes), Ok(process(4, "System")));
    assert!(find_process("AceCombat", &processes).unwrap_err().contains("No process is named AceCombat"));
    assert!(find_process("AceCombat8.ex", &processes).is_err());
}

#[test]
fn a_process_is_found_by_its_id() {
    let processes = [process(1200, "AceCombat8.exe"), process(1300, "explorer.exe")];
    assert_eq!(find_process("1300", &processes), Ok(process(1300, "explorer.exe")));
    assert!(find_process("1400", &processes).unwrap_err().contains("No process has the ID 1400"));
}

#[test]
fn processes_of_the_same_name_need_an_id() {
    let processes = [process(1200, "Game.exe"), process(1300, "explorer.exe"), process(1400, "game.exe")];
    let error = find_process("Game", &processes).unwrap_err();
    assert!(error.contains("2 processes are named Game.exe: 1200, 1400"), "{error}");
    assert_eq!(find_process("1400", &processes), Ok(process(1400, "game.exe")));
}

#[test]
fn the_running_processes_include_this_one() {
    let processes = processes().unwrap();
    let this = processes.iter().find(|process| process.id == std::process::id()).unwrap();
    let exe = std::env::current_exe().unwrap();
    assert!(this.name.eq_ignore_ascii_case(exe.file_name().unwrap().to_str().unwrap()), "{}", this.name);
    assert_eq!(find_process(&std::process::id().to_string(), &processes), Ok(this.clone()));
}
