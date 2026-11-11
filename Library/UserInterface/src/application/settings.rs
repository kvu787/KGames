// A program's settings file, Settings.json, in its folder: the settings that
// every program has, then the program's own.
//
// Each setting is read with its default and what makes it valid. A missing
// setting has its default, and so does a wrong one, which a notice under the
// app bar tells of. A missing file is written with every default, in the
// order they were read; once it exists, it's never changed. Values that are
// words ignore case; names don't.

use std::path::{Path, PathBuf};

use json::Value;

use crate::design::theme::ThemeKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum WindowMode {
    #[default]
    Maximized,
    BorderlessFullscreen,
}

impl WindowMode {
    pub fn name(self) -> &'static str {
        match self {
            WindowMode::Maximized => "Maximized",
            WindowMode::BorderlessFullscreen => "BorderlessFullscreen",
        }
    }

    pub fn other(self) -> WindowMode {
        match self {
            WindowMode::Maximized => WindowMode::BorderlessFullscreen,
            WindowMode::BorderlessFullscreen => WindowMode::Maximized,
        }
    }
}

// The settings that every program has, which the library reads first, in
// this order, before the program reads its own.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommonSettings {
    // "Primary", or the identifier of the display to open on.
    pub display: String,
    // How the window opens, which F11 switches.
    pub window_mode: WindowMode,
    // Whether frames start waiting for vertical sync, unless the command
    // line says otherwise. F10 switches it.
    pub vertical_sync: bool,
    // The theme that the program starts in, which F9 switches.
    pub theme: ThemeKind,
}

impl CommonSettings {
    // Reads them, each its default if it's missing or wrong.
    pub fn read(settings: &mut Settings) -> CommonSettings {
        let display = settings.string("Display", "Primary");
        let modes = [WindowMode::Maximized, WindowMode::BorderlessFullscreen];
        let window_mode = settings.choice_of("WindowMode", WindowMode::Maximized, &modes, |mode| mode.name());
        let vertical_sync = settings.boolean("VerticalSync", true);
        let themes = [ThemeKind::Light, ThemeKind::Dark];
        let theme = settings.choice_of("Theme", ThemeKind::Light, &themes, |theme| theme.name());
        CommonSettings {
            display,
            window_mode,
            vertical_sync,
            theme,
        }
    }
}

pub struct Settings {
    path: PathBuf,
    members: Option<Vec<(String, Value)>>,
    existed: bool,
    defaults: Vec<(String, Value)>,
    problems: Vec<String>,
}

impl Settings {
    pub fn load(path: &Path) -> Settings {
        match std::fs::read_to_string(path) {
            Ok(text) => Settings::parse(path, &text),
            Err(_) => Settings {
                path: path.to_path_buf(),
                members: None,
                existed: false,
                defaults: Vec::new(),
                problems: Vec::new(),
            },
        }
    }

    // Settings from text, without a file, for tests.
    pub fn from_text(text: &str) -> Settings {
        Settings::parse(Path::new(""), text)
    }

    fn parse(path: &Path, text: &str) -> Settings {
        let mut settings = Settings {
            path: path.to_path_buf(),
            members: None,
            existed: true,
            defaults: Vec::new(),
            problems: Vec::new(),
        };
        match json::parse(text) {
            Ok(Value::Object(members)) => settings.members = Some(members),
            Ok(other) => settings.problems.push(format!("It must be an object, not {}. Every setting has its default.", other.describe())),
            Err(error) => settings.problems.push(format!(
                "It isn't valid JSON, at line {}, column {}: {}. Every setting has its default.",
                error.line, error.column, error.message
            )),
        }
        settings
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    // What's wrong with the file and its settings, a sentence each.
    pub fn problems(&self) -> &[String] {
        &self.problems
    }

    // A setting, which `parse` makes from its JSON, or says what's wrong
    // with. `default` is the JSON of its default, which must parse.
    pub fn value<T>(&mut self, name: &str, default: Value, parse: impl Fn(&Value) -> Result<T, String>) -> T {
        let fallback = parse(&default).unwrap_or_else(|error| panic!("the default of {name} is invalid: {error}"));
        self.defaults.push((name.to_string(), default));
        let found = self.members.as_ref().and_then(|members| members.iter().find(|(member, _)| member == name));
        let Some((_, value)) = found else {
            return fallback;
        };
        match parse(value) {
            Ok(parsed) => parsed,
            Err(error) => {
                self.problems.push(format!("{name} {}, so it's the default.", error.trim_end_matches('.')));
                fallback
            }
        }
    }

    pub fn boolean(&mut self, name: &str, default: bool) -> bool {
        self.value(name, Value::Bool(default), |value| match value {
            Value::Bool(value) => Ok(*value),
            other => Err(format!("must be true or false, not {}", other.describe())),
        })
    }

    // A number that `check` accepts, or says why not.
    pub fn number(&mut self, name: &str, default: f64, check: impl Fn(f64) -> Result<(), String>) -> f64 {
        self.value(name, Value::Number(default), |value| match value {
            Value::Number(number) => check(*number).map(|()| *number),
            other => Err(format!("must be a number, not {}", other.describe())),
        })
    }

    pub fn string(&mut self, name: &str, default: &str) -> String {
        self.value(name, Value::from(default), |value| match value {
            Value::String(text) => Ok(text.clone()),
            other => Err(format!("must be text, not {}", other.describe())),
        })
    }

    // One of the words, whatever its case, returned as `choices` has it.
    pub fn choice(&mut self, name: &str, default: &str, choices: &[&str]) -> String {
        self.value(name, Value::from(default), |value| match value {
            Value::String(text) => choices
                .iter()
                .find(|choice| choice.eq_ignore_ascii_case(text))
                .map(|choice| choice.to_string())
                .ok_or_else(|| format!("must be {}, not \"{text}\"", list_choices(choices))),
            other => Err(format!("must be {}, not {}", list_choices(choices), other.describe())),
        })
    }

    // One of the choices, by its name, whatever the name's case.
    pub fn choice_of<T: Copy>(&mut self, name: &str, default: T, choices: &[T], choice_name: impl Fn(T) -> &'static str) -> T {
        let names: Vec<&str> = choices.iter().map(|choice| choice_name(*choice)).collect();
        let chosen = self.choice(name, choice_name(default), &names);
        choices.iter().copied().find(|choice| choice_name(*choice) == chosen).unwrap_or(default)
    }

    // Writes the file with every setting's default, if it didn't exist.
    pub fn write_if_missing(&self) -> std::io::Result<()> {
        if self.existed || self.path.as_os_str().is_empty() {
            return Ok(());
        }
        std::fs::write(&self.path, Value::Object(self.defaults.clone()).to_json() + "\n")
    }
}

fn list_choices(choices: &[&str]) -> String {
    match choices {
        [] => String::new(),
        [only] => format!("\"{only}\""),
        [rest @ .., last] => {
            let rest: Vec<String> = rest.iter().map(|choice| format!("\"{choice}\"")).collect();
            format!("{} or \"{last}\"", rest.join(", "))
        }
    }
}
