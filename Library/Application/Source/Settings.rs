// A program's settings, from a Settings.json in its folder, next to its
// Cargo.toml. The program builds the folder's path into its exe, so the exe
// finds the file wherever it's run from. The file is in Git.
//
//     {
//         "Display": "Primary",
//         "WindowMode": "Maximized",
//         "AllowTearing": true
//     }
//
// Every program has those three, and can add its own after them. The program
// writes the file, with its defaults, if it doesn't exist. A setting that's
// missing or wrong keeps its default, and the program's window says what's
// wrong with it. Programs never change the file once it exists.

use std::path::Path;

use json::Value;

// The only two window setups that programs support. See WindowCheck.rs.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum WindowMode {
    Maximized,
    // A window without a frame that covers its display, for the lowest
    // latency. F11 switches between it and Maximized.
    BorderlessFullscreen,
}

impl WindowMode {
    const ALL: [WindowMode; 2] = [WindowMode::Maximized, WindowMode::BorderlessFullscreen];

    // As the settings file names it.
    fn name(self) -> &'static str {
        match self {
            WindowMode::Maximized => "Maximized",
            WindowMode::BorderlessFullscreen => "BorderlessFullscreen",
        }
    }
}

// The settings that every program has.
pub struct Settings {
    // The identifier of the display to open on, as IdentifyDisplays shows
    // it, or None for the primary display, which the file calls "Primary".
    pub display: Option<String>,
    pub window_mode: WindowMode,
    // Whether frames may tear, which lets them reach the screen mid-refresh,
    // if the GPU's driver allows it.
    pub allow_tearing: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            display: None,
            window_mode: WindowMode::Maximized,
            allow_tearing: true,
        }
    }
}

// A program's own settings, which follow the ones every program has. `()`
// is for a program without any.
pub trait ProgramSettings: Sized {
    // As the settings file names them, in its order.
    const NAMES: &'static [&'static str];

    // The defaults, of the settings every program has too, which a program
    // can change, and of its own.
    fn defaults() -> (Settings, Self);

    // Takes one of the program's own settings, named by one of NAMES, or
    // says what's wrong with its value, after "<name> ".
    fn apply(&mut self, name: &str, value: &Value) -> Result<(), String>;

    // The program's own settings, for the default file, one line each,
    // like `"RumbleTestInnerDeadZone": 0.1`.
    fn default_lines(&self) -> Vec<String>;
}

impl ProgramSettings for () {
    const NAMES: &'static [&'static str] = &[];

    fn defaults() -> (Settings, Self) {
        (Settings::default(), ())
    }

    fn apply(&mut self, _name: &str, _value: &Value) -> Result<(), String> {
        unreachable!("a program without settings of its own has no names to apply")
    }

    fn default_lines(&self) -> Vec<String> {
        Vec::new()
    }
}

impl Settings {
    // Reads the settings file at `path`, or writes one with the defaults if
    // there's none. Problems go in `notices`.
    pub fn load<T: ProgramSettings>(path: &Path, notices: &mut Vec<String>) -> (Self, T) {
        let (mut settings, mut program) = T::defaults();
        let text = match std::fs::read_to_string(path) {
            Ok(text) => text,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                if let Err(error) = write_defaults(path, &settings, &program) {
                    notices.push(format!("Couldn't write {}: {error}", path.display()));
                }
                return (settings, program);
            }
            Err(error) => {
                notices.push(format!(
                    "Couldn't read {}, so the default settings are used: {error}",
                    path.display()
                ));
                return (settings, program);
            }
        };
        // Notepad can save UTF-8 with a byte order mark.
        let text = text.strip_prefix('\u{FEFF}').unwrap_or(&text);
        match json::parse(text) {
            Ok(value) => {
                for problem in settings.apply(&mut program, &value) {
                    notices.push(format!("{}: {problem}", path.display()));
                }
            }
            Err(error) => notices.push(format!(
                "{}, line {}, column {}: {}. The default settings are used.",
                path.display(),
                error.line,
                error.column,
                error.message
            )),
        }
        (settings, program)
    }

    // Takes the settings that are right from the file's contents, and says
    // what's wrong with the others.
    fn apply<T: ProgramSettings>(&mut self, program: &mut T, value: &Value) -> Vec<String> {
        let Value::Object(members) = value else {
            return vec![format!(
                "The settings must be an object, in {{ }}, not {}. The defaults are used.",
                value.describe()
            )];
        };
        let mut problems = Vec::new();
        for (name, value) in members {
            let result = match name.as_str() {
                "Display" => string(value).map(|display| self.display = parse_display(display)),
                "WindowMode" => string(value).and_then(|text| {
                    let mode = WindowMode::ALL
                        .into_iter()
                        .find(|mode| mode.name().eq_ignore_ascii_case(text));
                    self.window_mode = mode.ok_or_else(|| {
                        format!(
                            "must be \"Maximized\" or \"BorderlessFullscreen\", not {}",
                            value.describe()
                        )
                    })?;
                    Ok(())
                }),
                "AllowTearing" => match value {
                    Value::Bool(allow) => {
                        self.allow_tearing = *allow;
                        Ok(())
                    }
                    _ => Err(format!("must be true or false, not {}", value.describe())),
                },
                name if T::NAMES.contains(&name) => program.apply(name, value),
                _ => Err(format!("isn't a setting. The settings are {}.", names::<T>())),
            };
            if let Err(problem) = result {
                problems.push(format!("{name} {problem}"));
            }
        }
        problems
    }
}

// Every setting's name, like "Display, WindowMode and AllowTearing".
fn names<T: ProgramSettings>() -> String {
    let names: Vec<&str> = ["Display", "WindowMode", "AllowTearing"]
        .into_iter()
        .chain(T::NAMES.iter().copied())
        .collect();
    let (last, rest) = names.split_last().unwrap();
    format!("{} and {last}", rest.join(", "))
}

// "Primary", in any case, means the primary display. Anything else is a
// display's identifier.
pub fn parse_display(text: &str) -> Option<String> {
    let text = text.trim();
    (!text.eq_ignore_ascii_case("Primary")).then(|| text.to_string())
}

fn write_defaults<T: ProgramSettings>(
    path: &Path,
    settings: &Settings,
    program: &T,
) -> std::io::Result<()> {
    let display = settings.display.as_deref().unwrap_or("Primary");
    let lines: Vec<String> = [
        format!("\"Display\": \"{display}\""),
        format!("\"WindowMode\": \"{}\"", settings.window_mode.name()),
        format!("\"AllowTearing\": {}", settings.allow_tearing),
    ]
    .into_iter()
    .chain(program.default_lines())
    .map(|line| format!("    {line}"))
    .collect();
    std::fs::write(path, format!("{{\n{}\n}}\n", lines.join(",\n")))
}

fn string(value: &Value) -> Result<&str, String> {
    match value {
        Value::String(text) => Ok(text),
        _ => Err(format!("must be a string, not {}", value.describe())),
    }
}
