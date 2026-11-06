// What a GUI program is to the interface: a Program, which builds each
// frame's interface, may draw 3D before it, and tells scripts its state.

use std::path::{Path, PathBuf};

use graphics::{Frame, Gpu};

use crate::settings::{Settings, WindowMode};
use crate::theme::ThemeKind;
use crate::ui::Ui;
use crate::widgets::shell::Notice;
use crate::widgets::surfaces::Severity;

pub trait Program {
    // The window's title.
    fn title(&self) -> String;

    // Builds the frame's interface, and does what its input says. Called once
    // a frame, right before the frame is drawn, with the newest input.
    fn update(&mut self, ui: &mut Ui);

    // Creates what the program draws with on the GPU, once, before the
    // first frame.
    fn prepare(&mut self, _gpu: &mut Gpu) -> windows_core::Result<()> {
        Ok(())
    }

    // Draws the program's 3D views into the frame, after it's cleared to the
    // theme's background and before the interface draws over it.
    fn render(&mut self, _gpu: &mut Gpu, _frame: &Frame) -> windows_core::Result<()> {
        Ok(())
    }

    // What scripts save and check, besides the interface's, as a JSON object.
    fn state(&self) -> json::Value {
        json::Value::Object(Vec::new())
    }

    // Does a script's Program step, like connecting a made-up device.
    fn script(&mut self, step: &json::Value) -> Result<(), String> {
        Err(format!("this program has no Program steps, but got {}", step.to_json()))
    }
}

// How the interface runs a program.
#[derive(Clone, Debug)]
pub struct Options {
    // The program's name, like "SimplePaintDemo".
    pub name: &'static str,
    // The program's folder, with its Settings.json: env!("CARGO_MANIFEST_DIR").
    pub folder: &'static str,
    // The window mode when Settings.json doesn't say.
    pub window_mode: WindowMode,
    // Whether the interface reads the gamepad, for Ui::gamepad.
    pub gamepad: bool,
    pub theme: ThemeKind,
    // Arguments of the program's own, which it reads from Startup.
    pub arguments: &'static [&'static str],
}

impl Options {
    pub fn new(name: &'static str, folder: &'static str) -> Self {
        Self {
            name,
            folder,
            window_mode: WindowMode::Maximized,
            gamepad: false,
            theme: ThemeKind::Light,
            arguments: &[],
        }
    }
}

// What a program starts with.
pub struct Startup {
    // Settings.json, to read the program's own settings from.
    pub settings: Settings,
    // Where the program reads and writes its files: its folder, or the
    // output folder of a script, so that scripts never change its files.
    pub data_folder: PathBuf,
    // Whether input comes from a script.
    pub scripted: bool,
    // The program's own arguments, of those that Options allows.
    pub arguments: Vec<String>,
    pub(crate) notices: Vec<Notice>,
}

impl Startup {
    pub fn new(settings: Settings, data_folder: &Path, scripted: bool) -> Startup {
        Startup {
            settings,
            data_folder: data_folder.to_path_buf(),
            scripted,
            arguments: Vec::new(),
            notices: Vec::new(),
        }
    }

    // A start for tests: settings from the text, and files in the folder.
    pub fn for_tests(settings: &str, data_folder: &Path) -> Startup {
        Startup::new(Settings::from_text(settings), data_folder, true)
    }

    // Tells of something under the app bar, like a file that's wrong.
    pub fn notify(&mut self, severity: Severity, title: &str, message: &str) {
        self.notices.push(Notice {
            severity,
            title: title.to_string(),
            message: message.to_string(),
        });
    }

    // The notices, with one for each problem with the settings.
    pub(crate) fn take_notices(&mut self) -> Vec<Notice> {
        let mut notices: Vec<Notice> = self
            .settings
            .problems()
            .iter()
            .map(|problem| Notice {
                severity: Severity::Warning,
                title: "Settings.json".to_string(),
                message: problem.clone(),
            })
            .collect();
        notices.append(&mut self.notices);
        notices
    }
}

// The command line that every program takes.
#[derive(Clone, Debug, Default)]
pub struct Arguments {
    // The display to open on, by its identifier, or "Primary".
    pub display: Option<String>,
    // Opens without taking focus.
    pub no_activate: bool,
    // Frames wait for the screen's refresh, as test runs want.
    pub vertical_sync: bool,
    // At most this many frames a second, as test runs want.
    pub frame_limit: Option<f64>,
    // A script to run instead of the user's input, and the folder for what
    // it saves.
    pub script: Option<PathBuf>,
    pub output: Option<PathBuf>,
    pub own: Vec<String>,
}

impl Arguments {
    pub fn parse(arguments: impl IntoIterator<Item = String>, own: &[&str]) -> Result<Arguments, String> {
        let mut parsed = Arguments::default();
        let mut arguments = arguments.into_iter();
        while let Some(argument) = arguments.next() {
            let mut value = |name: &str| arguments.next().ok_or_else(|| format!("{name} needs a value after it"));
            match argument.as_str() {
                "--display" => parsed.display = Some(value("--display")?),
                "--no-activate" => parsed.no_activate = true,
                "--vsync" => parsed.vertical_sync = true,
                "--frame-limit" => {
                    let text = value("--frame-limit")?;
                    match text.parse::<f64>() {
                        Ok(fps) if fps > 0.0 && fps.is_finite() => parsed.frame_limit = Some(fps),
                        _ => return Err(format!("--frame-limit needs frames a second after it, more than 0, not {text}.")),
                    }
                }
                "--script" => parsed.script = Some(PathBuf::from(value("--script")?)),
                "--output" => parsed.output = Some(PathBuf::from(value("--output")?)),
                other if own.contains(&other) => parsed.own.push(other.to_string()),
                other => return Err(format!("Unknown argument {other}. Every program takes --display <identifier>, --no-activate, and --script <path> with --output <folder>.")),
            }
        }
        if parsed.script.is_some() != parsed.output.is_some() {
            return Err("--script and --output go together".to_string());
        }
        Ok(parsed)
    }
}
