// A program's settings file, Settings.json, in its folder: the settings
// every program has, and the program's own.
//
// Each setting is read with its default and what makes it valid. A setting
// that's missing keeps its default, and so does one that's wrong, which a
// notice under the app bar tells of. If the file is missing, it's written
// with every setting's default, in the order they were read. Once it exists,
// it's never changed. Values that are words ignore case; names don't.

use std::path::{Path, PathBuf};

use json::Value;

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

pub struct Settings {
    path: PathBuf,
    members: Option<Vec<(String, Value)>>,
    existed: bool,
    defaults: Vec<(String, Value)>,
    problems: Vec<String>,
}

impl Settings {
    pub fn load(path: &Path) -> Settings {
        let mut settings = Settings {
            path: path.to_path_buf(),
            members: None,
            existed: false,
            defaults: Vec::new(),
            problems: Vec::new(),
        };
        let Ok(text) = std::fs::read_to_string(path) else {
            return settings;
        };
        settings.existed = true;
        match json::parse(&text) {
            Ok(Value::Object(members)) => settings.members = Some(members),
            Ok(other) => settings.problems.push(format!("It must be an object, not {}.", other.describe())),
            Err(error) => settings.problems.push(format!("It isn't valid JSON, at line {}, column {}: {}. Every setting has its default.", error.line, error.column, error.message)),
        }
        settings
    }

    // Settings from text, without a file, for tests.
    pub fn from_text(text: &str) -> Settings {
        let mut settings = Settings {
            path: PathBuf::new(),
            members: None,
            existed: true,
            defaults: Vec::new(),
            problems: Vec::new(),
        };
        match json::parse(text) {
            Ok(Value::Object(members)) => settings.members = Some(members),
            Ok(other) => settings.problems.push(format!("It must be an object, not {}.", other.describe())),
            Err(error) => settings.problems.push(format!("It isn't valid JSON, at line {}, column {}: {}. Every setting has its default.", error.line, error.column, error.message)),
        }
        settings
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    // What was wrong with the file and its settings.
    pub fn problems(&self) -> &[String] {
        &self.problems
    }

    // A setting, made from its JSON by `parse`, which says what's wrong with
    // one that's invalid. `default` is the JSON of its default.
    pub fn value<T>(&mut self, name: &str, default: Value, parse: impl Fn(&Value) -> Result<T, String>) -> T {
        let fallback = parse(&default).unwrap_or_else(|error| panic!("the default of {name} is invalid: {error}"));
        self.defaults.push((name.to_string(), default));
        let Some(value) = self.members.as_ref().and_then(|members| members.iter().find(|(member, _)| member == name)).map(|(_, value)| value.clone())
        else {
            return fallback;
        };
        match parse(&value) {
            Ok(parsed) => parsed,
            Err(error) => {
                self.problems.push(format!("{name} {error}, so it's the default."));
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

    // One of the words, without regard to case. Returns it as `choices` has it.
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

    // Writes the file with every setting's default, if it didn't exist.
    pub fn write_if_missing(&self) -> std::io::Result<()> {
        if self.existed || self.path.as_os_str().is_empty() {
            return Ok(());
        }
        let text = Value::Object(self.defaults.clone()).to_json();
        std::fs::write(&self.path, text + "\n")
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
