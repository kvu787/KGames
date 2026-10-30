// The demo's settings, from Settings.json in the GameInputDemo folder, next to
// Cargo.toml. The folder's path is built into the exe, so the exe finds the
// file wherever it's run from. The file is in Git.
//
//     {
//         "Display": "Primary",
//         "WindowMode": "Maximized",
//         "AllowTearing": true,
//         "RumbleTestInnerDeadZone": 0.1
//     }
//
// The demo writes that file, with the defaults, if it doesn't exist. A
// setting that's missing or wrong keeps its default, and the demo's header
// says what's wrong with it. The demo never changes the file once it exists.

use std::path::{Path, PathBuf};

use json::Value;

// The only two window setups the demo supports. See WindowCheck.rs.
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

pub struct Settings {
    // The identifier of the display to open on, as IdentifyDisplays shows
    // it, or None for the primary display, which the file calls "Primary".
    pub display: Option<String>,
    pub window_mode: WindowMode,
    // Whether frames may tear, which lets them reach the screen mid-refresh,
    // if the GPU's driver allows it.
    pub allow_tearing: bool,
    // How far a stick must be from its center, as a fraction of full
    // deflection, before it runs its motor in the rumble test.
    pub rumble_dead_zone: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            display: None,
            window_mode: WindowMode::Maximized,
            allow_tearing: true,
            rumble_dead_zone: 0.1,
        }
    }
}

impl Settings {
    // Reads the settings file, or writes one with the defaults if there's
    // none. Problems go in `notices`.
    pub fn load(notices: &mut Vec<String>) -> Self {
        let mut settings = Self::default();
        let path = path();
        let text = match std::fs::read_to_string(&path) {
            Ok(text) => text,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                if let Err(error) = write_defaults(&path) {
                    notices.push(format!("Couldn't write {}: {error}", path.display()));
                }
                return settings;
            }
            Err(error) => {
                notices.push(format!(
                    "Couldn't read {}, so the demo uses its default settings: {error}",
                    path.display()
                ));
                return settings;
            }
        };
        // Notepad can save UTF-8 with a byte order mark.
        let text = text.strip_prefix('\u{FEFF}').unwrap_or(&text);
        match json::parse(text) {
            Ok(value) => {
                for problem in settings.apply(&value) {
                    notices.push(format!("{}: {problem}", path.display()));
                }
            }
            Err(error) => notices.push(format!(
                "{}, line {}, column {}: {}. The demo uses its default settings.",
                path.display(),
                error.line,
                error.column,
                error.message
            )),
        }
        settings
    }

    // Takes the settings that are right from the file's contents, and says
    // what's wrong with the others.
    fn apply(&mut self, value: &Value) -> Vec<String> {
        let Value::Object(members) = value else {
            return vec![format!(
                "The settings must be an object, in {{ }}, not {}. The demo uses its defaults.",
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
                // Less than 1, because the strength grows from the dead
                // zone's edge to the stick's.
                "RumbleTestInnerDeadZone" => match value {
                    Value::Number(number) if (0.0..1.0).contains(number) => {
                        self.rumble_dead_zone = *number as f32;
                        Ok(())
                    }
                    _ => Err(format!(
                        "must be a number from 0 to less than 1, not {}",
                        value.describe()
                    )),
                },
                _ => Err(
                    "isn't a setting. The settings are Display, WindowMode, AllowTearing \
                          and RumbleTestInnerDeadZone."
                        .to_string(),
                ),
            };
            if let Err(problem) = result {
                problems.push(format!("{name} {problem}"));
            }
        }
        problems
    }
}

// "Primary", in any case, means the primary display. Anything else is a
// display's identifier.
pub fn parse_display(text: &str) -> Option<String> {
    let text = text.trim();
    (!text.eq_ignore_ascii_case("Primary")).then(|| text.to_string())
}

fn path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("Settings.json")
}

fn write_defaults(path: &Path) -> std::io::Result<()> {
    let defaults = Settings::default();
    // The default display is the primary one.
    let text = format!(
        r#"{{
    "Display": "Primary",
    "WindowMode": "{}",
    "AllowTearing": {},
    "RumbleTestInnerDeadZone": {}
}}
"#,
        defaults.window_mode.name(),
        defaults.allow_tearing,
        defaults.rumble_dead_zone
    );
    std::fs::write(path, text)
}

fn string(value: &Value) -> Result<&str, String> {
    match value {
        Value::String(text) => Ok(text),
        _ => Err(format!("must be a string, not {}", value.describe())),
    }
}
