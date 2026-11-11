// What a GUI program is to the library: a Program with a message type, which
// says what the input on its widgets means; `view`, which describes a
// frame's interface from the program's state and changes nothing; `update`,
// which applies a message; and `input`, which turns the frame's keys, time
// and devices into messages. It may draw 3D under the interface, and tells
// scripts its state. `run` runs one with the Options that say how.
//
// A frame: the interface finds what the input did to the widgets the user
// saw, as messages; the program's `input` adds its own; `update` applies
// them all, and any they send in turn, until none are left; then `view`
// describes the frame, which is laid out and drawn. So every widget shows
// what every message did, on the frame of the input.

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use graphics::{Frame, Gpu};

use crate::application::settings::{CommonSettings, Settings, WindowMode};
use crate::core::interface::{Context, Ui};
use crate::core::tree::Tree;
use crate::design::theme::ThemeKind;
use crate::platform::input_devices::Devices;
use crate::widgets::info_bar::Severity;

pub trait Program {
    // What the input on its widgets means, and what else it does.
    type Message: Clone + 'static;

    // The window's title.
    fn title(&self) -> String;

    // The frame's messages from what isn't a widget: the keys that no widget
    // took, the time since the last frame, the gamepad, and the devices.
    // Once a frame, before the messages are applied.
    fn input(&self, _ui: &Ui) -> Vec<Self::Message> {
        Vec::new()
    }

    // Applies a message: changes the program's state, and may send more
    // messages, or ask the window to close, or switch the theme.
    fn update(&mut self, message: Self::Message, ui: &mut Context<Self::Message>);

    // Describes the frame's interface from the program's state, which it
    // doesn't change. Once a frame, after the messages are applied.
    fn view(&self, ui: &mut Ui) -> Tree<Self::Message>;

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
    // close with Context::close once the user says so.
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

// How the library runs a program. Every program starts in the theme, window
// mode and vertical sync that its Settings.json says, unless the command
// line says otherwise, and F9, F11 and F10 switch them.
#[derive(Clone, Debug)]
pub struct Options {
    // The program's name, like "SimplePaintDemo".
    pub name: &'static str,
    // The program's folder, which has its Settings.json: env!("CARGO_MANIFEST_DIR").
    pub folder: &'static str,
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
            gamepad: false,
            arguments: &[],
        }
    }

    pub fn gamepad(mut self) -> Self {
        self.gamepad = true;
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
    // The settings that every program has, which were read from `settings`
    // first, as the command line overrides them: the window mode, vertical
    // sync and theme that the program starts in. `display` is the setting's,
    // which --display overrides with a display that must be connected.
    pub common: CommonSettings,
    // Where the program reads and writes its files: its folder, or a
    // script's output folder, so that scripts never change its files.
    pub data_folder: PathBuf,
    // Whether input comes from a script.
    pub scripted: bool,
    // The program's own arguments, of those that Options allows.
    pub arguments: Vec<String>,
    // The input devices that the interface reads, for programs that show
    // them or run their motors, like GameInputDemo. Each frame's readings are
    // there when the program's messages are applied. None in a scripted run,
    // and with --no-input.
    pub devices: Option<Rc<RefCell<Devices>>>,
    pub(crate) notices: Vec<Notice>,
}

impl Startup {
    // Reads the settings that every program has from `settings`, before the
    // program reads its own.
    pub fn new(mut settings: Settings, data_folder: &Path, scripted: bool) -> Startup {
        let common = CommonSettings::read(&mut settings);
        Startup {
            settings,
            common,
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
    // The theme and window mode to start in, with --theme and --window-mode,
    // whatever the Theme and WindowMode settings say, so that test runs start
    // the same way whatever someone set for themselves.
    pub theme: Option<ThemeKind>,
    pub window_mode: Option<WindowMode>,
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
                "--theme" => parsed.theme = Some(choose("--theme", value("--theme")?, &[ThemeKind::Light, ThemeKind::Dark], ThemeKind::name)?),
                "--window-mode" => {
                    let modes = [WindowMode::MaximizedWindowed, WindowMode::BorderlessFullscreen];
                    parsed.window_mode = Some(choose("--window-mode", value("--window-mode")?, &modes, WindowMode::name)?);
                }
                "--no-input" => parsed.no_input = true,
                "--script" => parsed.script = Some(PathBuf::from(value("--script")?)),
                "--output" => parsed.output = Some(PathBuf::from(value("--output")?)),
                other if own.contains(&other) => parsed.own.push(other.to_string()),
                other => {
                    return Err(format!(
                        "Unknown argument {other}. Every program takes --display <identifier>, --no-activate, --no-input, --vsync or --no-vsync, --theme <Light or Dark>, --window-mode <MaximizedWindowed or BorderlessFullscreen>, and --script <path> with --output <folder>."
                    ));
                }
            }
        }
        if parsed.script.is_some() != parsed.output.is_some() {
            return Err("--script and --output go together.".to_string());
        }
        Ok(parsed)
    }

    // Overrides the settings that every program has with what the command
    // line says, of vertical sync, the theme and the window mode.
    pub fn override_settings(&self, common: &mut CommonSettings) {
        if let Some(vsync) = self.vsync {
            common.vertical_sync = vsync;
        }
        if let Some(theme) = self.theme {
            common.theme = theme;
        }
        if let Some(window_mode) = self.window_mode {
            common.window_mode = window_mode;
        }
    }
}

// One of the choices, by its name, whatever the name's case.
fn choose<T: Copy>(argument: &str, value: String, choices: &[T], name: impl Fn(T) -> &'static str) -> Result<T, String> {
    choices.iter().copied().find(|choice| name(*choice).eq_ignore_ascii_case(&value)).ok_or_else(|| {
        let names: Vec<&str> = choices.iter().map(|choice| name(*choice)).collect();
        format!("{argument} takes {}, not {value}.", names.join(" or "))
    })
}
