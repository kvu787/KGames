// Library/Application's command line, settings file and frame counter.

use std::path::{Path, PathBuf};

use application::{Arguments, FrameCounter, ProgramSettings, Settings, WindowMode};
use json::Value;

// A program's own setting, like GameInputDemo's.
#[derive(Debug, PartialEq)]
struct Volume(f64);

impl ProgramSettings for Volume {
    const NAMES: &'static [&'static str] = &["Volume"];

    fn defaults() -> (Settings, Self) {
        (Settings::default(), Volume(0.5))
    }

    fn apply(&mut self, _name: &str, value: &Value) -> Result<(), String> {
        match value {
            Value::Number(number) if (0.0..=1.0).contains(number) => {
                self.0 = *number;
                Ok(())
            }
            _ => Err(format!("must be a number from 0 to 1, not {}", value.describe())),
        }
    }

    fn default_lines(&self) -> Vec<String> {
        vec![format!("\"Volume\": {}", self.0)]
    }
}

// A fresh folder for one test's files.
fn folder(name: &str) -> PathBuf {
    let folder = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&folder);
    std::fs::create_dir_all(&folder).unwrap();
    folder
}

fn arguments(list: &[&str]) -> (Arguments, Vec<String>) {
    let mut notices = Vec::new();
    let arguments = Arguments::parse_from(list.iter().map(|argument| argument.to_string()), &mut notices);
    (arguments, notices)
}

#[test]
fn reads_the_command_line() {
    let (parsed, notices) = arguments(&[
        "--display", "KXGR16", "--no-activate", "--script", "Steps.json", "--output", "Out", "--loud",
        "--display",
    ]);
    assert_eq!(parsed.display.as_deref(), Some("KXGR16"));
    assert!(!parsed.activate);
    assert_eq!(parsed.script, Some(PathBuf::from("Steps.json")));
    assert_eq!(parsed.output, Some(PathBuf::from("Out")));
    assert!(parsed.scripted());
    assert_eq!(
        notices,
        ["Unknown command-line argument: --loud", "--display needs a display identifier after it"]
    );
}

#[test]
fn keeps_a_scripted_run_s_files_in_its_output_folder() {
    let (scripted, _) = arguments(&["--script", "Steps.json", "--output", r"C:\Out"]);
    assert_eq!(scripted.file(Path::new(r"C:\Program"), "Paint.json"), Path::new(r"C:\Out\Paint.json"));
    let (plain, _) = arguments(&["--output", r"C:\Out"]);
    assert_eq!(plain.file(Path::new(r"C:\Program"), "Paint.json"), Path::new(r"C:\Program\Paint.json"));
}

#[test]
fn writes_the_defaults_when_there_is_no_file() {
    let path = folder("Defaults").join("Settings.json");
    let mut notices = Vec::new();
    let (settings, volume) = Settings::load::<Volume>(&path, &mut notices);
    assert!(notices.is_empty(), "{notices:?}");
    assert_eq!((settings.display, settings.window_mode, settings.allow_tearing), (None, WindowMode::Maximized, true));
    assert_eq!(volume, Volume(0.5));
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        "{\n    \"Display\": \"Primary\",\n    \"WindowMode\": \"Maximized\",\n    \"AllowTearing\": true,\n    \"Volume\": 0.5\n}\n"
    );
}

#[test]
fn takes_what_is_right_and_says_what_is_wrong() {
    let path = folder("Mixed").join("Settings.json");
    std::fs::write(
        &path,
        "\u{FEFF}{ \"Display\": \"kxgr16\", \"WindowMode\": \"borderlessfullscreen\", \"AllowTearing\": 1, \"Volume\": 2, \"Colour\": 1 }",
    )
    .unwrap();
    let mut notices = Vec::new();
    let (settings, volume) = Settings::load::<Volume>(&path, &mut notices);
    assert_eq!(settings.display.as_deref(), Some("kxgr16"));
    assert_eq!(settings.window_mode, WindowMode::BorderlessFullscreen);
    assert!(settings.allow_tearing);
    assert_eq!(volume, Volume(0.5));
    let prefix = format!("{}: ", path.display());
    assert_eq!(
        notices,
        [
            format!("{prefix}AllowTearing must be true or false, not 1"),
            format!("{prefix}Volume must be a number from 0 to 1, not 2"),
            format!(
                "{prefix}Colour isn't a setting. The settings are Display, WindowMode, AllowTearing and Volume."
            ),
        ]
    );
}

#[test]
fn says_where_the_file_isn_t_json() {
    let path = folder("Broken").join("Settings.json");
    std::fs::write(&path, "{\n  \"Display\": Primary\n}").unwrap();
    let mut notices = Vec::new();
    let (settings, _) = Settings::load::<()>(&path, &mut notices);
    assert_eq!(settings.display, None);
    assert_eq!(
        notices,
        [format!("{}, line 2, column 14: expected a value. The default settings are used.", path.display())]
    );
}

#[test]
fn uses_the_defaults_in_a_scripted_run_and_the_command_line_s_display() {
    let path = folder("Scripted").join("Settings.json");
    std::fs::write(&path, "{ \"WindowMode\": \"BorderlessFullscreen\", \"Volume\": 1 }").unwrap();
    let mut notices = Vec::new();
    let (scripted, _) = arguments(&["--script", "Steps.json", "--output", "Out", "--display", "Primary"]);
    let (settings, volume) = Settings::for_program::<Volume>(&path, &scripted, &mut notices);
    assert_eq!((settings.window_mode, volume), (WindowMode::Maximized, Volume(0.5)));
    assert_eq!(settings.display, None);
    let (plain, _) = arguments(&["--display", "KXGR16"]);
    let (settings, volume) = Settings::for_program::<Volume>(&path, &plain, &mut notices);
    assert_eq!((settings.window_mode, volume), (WindowMode::BorderlessFullscreen, Volume(1.0)));
    assert_eq!(settings.display.as_deref(), Some("KXGR16"));
    assert!(notices.is_empty(), "{notices:?}");
}

#[test]
fn counts_frames_per_second_on_the_frames_clock() {
    let mut frames = FrameCounter::default();
    for frame in 0..=250 {
        frames.frame(frame as f64 * 0.01);
    }
    assert!((frames.per_second.unwrap() - 100.0).abs() < 1e-9);
    frames.pause();
    frames.frame(10.0);
    frames.frame(10.5);
    assert!((frames.per_second.unwrap() - 100.0).abs() < 1e-9, "a pause keeps the last rate until a new second ends");
}
