// What a GUI program is to the library: a Program, which builds each frame's
// interface and handles its input, may draw 3D under it, and tells scripts
// its state. `run` runs one with the Options that say how.

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use graphics::{Frame, Gpu};

use crate::application::settings::{Settings, WindowMode};
use crate::core::ui::Ui;
use crate::platform::input_devices::Devices;
use crate::widgets::info_bar::Severity;

pub trait Program {
    // The window's title.
    fn title(&self) -> String;

    // Builds the frame's interface and does what its input says. Called once
    // a frame, right before the frame is drawn, with the newest input.
    fn update(&mut self, ui: &mut Ui);

    // Creates what the program draws with on the GPU, once, before the first
    // frame.
    fn prepare(&mut self, _gpu: &mut Gpu) -> windows_core::Result<()> {
        Ok(())
    }

    // Draws the program's 3D into the frame, after it's cleared to the
    // theme's background and before the interface draws over it.
    fn render(&mut self, _gpu: &mut Gpu, _frame: &Frame) -> windows_core::Result<()> {
        Ok(())
    }

    // Whether the window may close now that it's asked to, like by its close
    // button. A program with unsaved work can say no, ask with a dialog, and
    // close with Ui::close once the user says so.
    fn may_close(&mut self) -> bool {
        true
    }

    // What scripts save and check, besides the interface's, as a JSON object.
    fn state(&self) -> json::Value {
        json::Value::Object(Vec::new())
    }

    // Does a script's Program step, like connecting a made-up device.
    fn script(&mut self, step: &json::Value) -> Result<(), String> {
        Err(format!("This program has no Program steps, but got {}.", step.to_json()))
    }
}

// How the library runs a program. Every program starts in the light theme,
// which F9 switches.
#[derive(Clone, Debug)]
pub struct Options {
    // The program's name, like "SimplePaintDemo".
    pub name: &'static str,
    // The program's folder, which has its Settings.json: env!("CARGO_MANIFEST_DIR").
    pub folder: &'static str,
    // The window mode when Settings.json doesn't say.
    pub window_mode: WindowMode,
    // Whether the program reads the gamepad: its newest reading goes to
    // Ui::gamepad, and its readings count as input for input latency.
    pub gamepad: bool,
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
            arguments: &[],
        }
    }

    pub fn gamepad(mut self) -> Self {
        self.gamepad = true;
        self
    }

    pub fn window_mode(mut self, mode: WindowMode) -> Self {
        self.window_mode = mode;
        self
    }

    pub fn arguments(mut self, arguments: &'static [&'static str]) -> Self {
        self.arguments = arguments;
        self
    }
}

// Something that a notice under the app bar tells of, like a file that's
// wrong.
#[derive(Clone, Debug, PartialEq)]
pub struct Notice {
    pub severity: Severity,
    pub title: String,
    pub message: String,
}

// What a program starts with.
pub struct Startup {
    // Settings.json, which the program reads its own settings from.
    pub settings: Settings,
    // Where the program reads and writes its files: its folder, or a
    // script's output folder, so that scripts never change its files.
    pub data_folder: PathBuf,
    // Whether input comes from a script.
    pub scripted: bool,
    // The program's own arguments, of those that Options allows.
    pub arguments: Vec<String>,
    // The input devices that the interface reads, for programs that show
    // them or run their motors, like GameInputDemo. Each frame's readings are
    // there when the program's update runs. None in a scripted run, and when
    // GameInput isn't available.
    pub devices: Option<Rc<RefCell<Devices>>>,
    pub(crate) notices: Vec<Notice>,
}

impl Startup {
    pub fn new(settings: Settings, data_folder: &Path, scripted: bool) -> Startup {
        Startup {
            settings,
            data_folder: data_folder.to_path_buf(),
            scripted,
            arguments: Vec::new(),
            devices: None,
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

    // The notices, with one for each problem with the settings first.
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
    // Whether frames wait for vertical sync, with --vsync or --no-vsync,
    // whatever the VerticalSync setting says, like for test runs.
    pub vsync: Option<bool>,
    // Doesn't start GameInput, for test runs, which never have focus, so
    // that nothing of theirs reads the devices of someone using the computer.
    pub no_input: bool,
    // A script to run instead of the user's input, and the folder for what
    // it saves.
    pub script: Option<PathBuf>,
    pub output: Option<PathBuf>,
    // The program's own, of those it allows.
    pub own: Vec<String>,
}

impl Arguments {
    pub fn parse(arguments: impl IntoIterator<Item = String>, own: &[&str]) -> Result<Arguments, String> {
        let mut parsed = Arguments::default();
        let mut arguments = arguments.into_iter();
        while let Some(argument) = arguments.next() {
            let mut value = |name: &str| arguments.next().ok_or_else(|| format!("{name} needs a value after it."));
            match argument.as_str() {
                "--display" => parsed.display = Some(value("--display")?),
                "--no-activate" => parsed.no_activate = true,
                "--vsync" => parsed.vsync = Some(true),
                "--no-vsync" => parsed.vsync = Some(false),
                "--no-input" => parsed.no_input = true,
                "--script" => parsed.script = Some(PathBuf::from(value("--script")?)),
                "--output" => parsed.output = Some(PathBuf::from(value("--output")?)),
                other if own.contains(&other) => parsed.own.push(other.to_string()),
                other => return Err(format!("Unknown argument {other}. Every program takes --display <identifier>, --no-activate, --no-input, --vsync or --no-vsync, and --script <path> with --output <folder>.")),
            }
        }
        if parsed.script.is_some() != parsed.output.is_some() {
            return Err("--script and --output go together.".to_string());
        }
        Ok(parsed)
    }
}
