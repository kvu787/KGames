// The main loop of every GUI program, built for the lowest input latency:
//
// 1. Handle the window's messages, without waiting.
// 2. Go on handling them until the presenter is ready: the GPU has finished
//    the last frame, and DXGI's queue has room.
// 3. Only then read the input, from GameInput, build the frame's interface
//    with the newest of it, draw the frame, and present it.
//
// So the input that a frame shows is as fresh as it can be when the frame
// reaches the screen. While the window is minimized, the loop waits for
// messages instead, which stops it drawing.
//
// Programs start in the theme, window mode and vertical sync that their
// Settings.json says, unless the command line says otherwise.
//
// With vertical sync on, frames present at the screen's refreshes, and
// DXGI's queue has room again only once the last frame has gone to the
// screen, so the loop waits for that, and for messages, instead of spinning:
// a frame couldn't begin any sooner, and the input is still read as soon as
// one can. F10 switches it on and off. Without it, frames present as soon as
// they're drawn, and may tear, and the loop spins, for the lowest latency.

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::rc::Rc;
use std::time::Instant;

use displays::Display;
use graphics::{AdapterChoice, Gpu, Image, Presenter, Readback, write_png};

use crate::application::performance::{Frame, Performance};
use crate::application::program::{Arguments, Notice, Options, Program, Startup};
use crate::application::settings::{Settings, WindowMode};
use crate::core::interface::{Environment, Interface, Request};
use crate::core::tree::Tree;
use crate::design::theme::ThemeKind;
use crate::drawing::geometry::Size;
use crate::drawing::renderer::Renderer;
use crate::drawing::text::Fonts;
use crate::input::events::Event;
use crate::input::reader::InputReader;
use crate::platform::clock;
use crate::platform::input_devices::Devices;
use crate::platform::window::Window;
use crate::testing::script::{Script, ScriptFrame, ScriptRunner};
use crate::widgets::info_bar::Severity;
use crate::widgets::shell::unsupported_setup;

// Runs the program that `create` makes until its window closes or its script
// ends. If it can't start, or a script's expectation isn't met, it says why
// on standard error and exits with a failure.
pub fn run<P: Program>(options: Options, create: impl FnOnce(&mut Startup) -> Result<P, String>) -> ExitCode {
    match run_program(&options, create) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{}: {message}", options.name);
            ExitCode::FAILURE
        }
    }
}

// The program's state with the interface's, as scripts save and check it.
pub fn state<P: Program>(program: &P, ui: &Interface) -> json::Value {
    let mut members = match program.state() {
        json::Value::Object(members) => members,
        json::Value::Null => Vec::new(),
        other => vec![("Program".to_string(), other)],
    };
    members.push(("Interface".to_string(), ui.state()));
    json::Value::Object(members)
}

fn run_program<P: Program>(options: &Options, create: impl FnOnce(&mut Startup) -> Result<P, String>) -> Result<(), String> {
    let arguments = Arguments::parse(std::env::args().skip(1), options.arguments)?;
    let folder = PathBuf::from(options.folder);
    let data_folder = arguments.output.clone().unwrap_or_else(|| folder.clone());
    if let Some(output) = &arguments.output {
        std::fs::create_dir_all(output).map_err(|error| format!("Couldn't make the folder {}: {error}", output.display()))?;
    }
    let mut source = Source::open(&arguments, options.gamepad, &data_folder)?;
    let scripted = matches!(source, Source::Script { .. });

    // The settings that every program has, as the command line overrides
    // them, then, in `create`, the program's own.
    let mut startup = Startup::new(Settings::load(&folder.join("Settings.json")), &data_folder, scripted);
    arguments.override_settings(&mut startup.common);
    startup.arguments = arguments.own.clone();
    startup.devices = source.devices();
    let common = startup.common.clone();
    let displays = displays::enumerate().map_err(|error| format!("Couldn't find the displays: {error}"))?;
    let display = choose_display(&displays, &arguments, &mut startup)?;

    // The interface, in the theme that the settings say, with the PerfBar
    // along the top of the window, and the input latency to the screen,
    // which a scripted run has no input for, nor a run without input, which
    // then also doesn't start the ETW session that every process's presents
    // would go through.
    let mut ui = Interface::new(Fonts::new().map_err(|error| format!("Couldn't start DirectWrite: {error}"))?, common.theme);
    ui.performance = Some(match &source {
        Source::Script { .. } => Performance::without_latency("none in a scripted run"),
        Source::User { devices: None, .. } => Performance::without_latency("none without input"),
        Source::User { .. } => Performance::measure(options.name),
    });
    let mut program = create(&mut startup)?;
    ui.notices = startup.take_notices();
    if !scripted && let Err(error) = startup.settings.write_if_missing() {
        ui.notices.push(Notice {
            severity: Severity::Warning,
            title: "Settings.json".to_string(),
            message: format!("Couldn't write it: {error}."),
        });
    }

    let mut window = Window::create(&program.title(), display.monitor, common.window_mode, ui.theme().background).map_err(|error| format!("Couldn't open the window: {error}"))?;
    window.set_dark_title_bar(ui.theme().kind == ThemeKind::Dark);
    let mut gpu = Gpu::new(AdapterChoice::Monitor(window.monitor())).map_err(|error| format!("Couldn't start Direct3D 12: {error}"))?;
    let (width, height) = window.client_size();
    // The renderer before the presenter: what's made last goes first, and
    // the presenter waits for the GPU to finish the last frame as it goes,
    // before the renderer's buffers and the program's resources go.
    let mut renderer = Renderer::new(&gpu).map_err(|error| format!("Couldn't make the renderer: {error}"))?;
    let mut presenter = Presenter::new(&gpu, window.hwnd(), width, height).map_err(|error| format!("Couldn't make the swap chain: {error}"))?;
    presenter.set_vsync(common.vertical_sync);
    program.prepare(&mut gpu).map_err(|error| format!("Couldn't prepare to draw: {error}"))?;
    ui.environment = Environment {
        gpu: gpu.name.clone(),
        display: display.identifier.clone(),
        display_name: display.name.clone(),
        vertical_sync: common.vertical_sync,
        scripted,
    };
    let mut title = program.title();
    window.show(!arguments.no_activate);

    // The tree the last frame showed, which the next frame's input is on.
    let mut shown: Option<Tree<P::Message>> = None;
    let mut problems = Vec::new();
    loop {
        if !window.pump() {
            // The window was asked to close. A program with unsaved work may
            // say not yet, and close itself later with Ui::close.
            if program.may_close() {
                break;
            }
            window.cancel_close();
        }
        if window.take_setup_changed() {
            problems = window.setup_problems();
        }
        // A minimized window doesn't draw, and waits for messages, but its
        // program still runs a frame when the window gets or loses focus,
        // like as it minimizes, so that it can stop what it's doing, like a
        // gamepad's motors.
        let (width, height) = window.client_size();
        let minimized = !scripted && (window.minimized() || width == 0 || height == 0);
        if !minimized {
            if (width, height) != presenter.size() {
                presenter.resize(&mut gpu, width, height).map_err(|error| format!("Couldn't resize the swap chain: {error}"))?;
            }
            if !presenter.poll_ready(&gpu) {
                // With vertical sync, the loop waits for the GPU and DXGI
                // instead of spinning, which ends as soon as a frame can
                // begin, or a message comes. Without it, the loop spins, so
                // that the frame begins the moment that it can, without a
                // thread's wake-up.
                if presenter.vsync() {
                    presenter.wait_ready(&gpu, 100).map_err(|error| format!("Couldn't wait for the GPU: {error}"))?;
                }
                continue;
            }
        }

        // The input, read as late as it can be.
        let read = clock::now();
        let input = source.read(&mut window, &ui, &mut program)?;
        if minimized && !input.focus_changed {
            window.wait();
            continue;
        }
        let (width, height) = presenter.size();
        let size = Size::new(width as f32, height as f32);
        let fullscreen = window.mode() == WindowMode::BorderlessFullscreen;
        if problems.is_empty() {
            ui.frame(&mut program, &mut shown, &input.events, Vec::new(), input.time, size, fullscreen);
        } else {
            // The error instead of the program, until the setup is right.
            shown = None;
            ui.frame_of(&unsupported_setup::<P::Message>(&problems), &input.events, input.time, size, fullscreen);
        }
        for request in std::mem::take(&mut ui.requests) {
            match request {
                Request::ToggleFullscreen => window.set_mode(window.mode().other()),
                Request::ToggleVerticalSync => {
                    presenter.set_vsync(!presenter.vsync());
                    ui.environment.vertical_sync = presenter.vsync();
                }
                Request::Close => return finish(&mut gpu, source),
            }
        }
        window.set_cursor(ui.cursor());
        window.set_dark_title_bar(ui.theme().kind == ThemeKind::Dark);
        if program.title() != title {
            title = program.title();
            window.set_title(&title);
        }
        if minimized {
            window.wait();
            continue;
        }

        let capture = input.script.as_ref().is_some_and(|script| !script.captures.is_empty());
        let image = draw(&mut gpu, &mut presenter, &mut renderer, &mut ui, &mut program, problems.is_empty(), capture)?;
        if let Some(performance) = &mut ui.performance {
            performance.presented(Frame {
                input: input.input_time,
                read,
                present: presenter.present_time(),
            });
        }
        gpu.report_messages();
        if let Some(script) = input.script
            && source.check(script, image, || state(&program, &ui))?
        {
            break;
        }
    }
    finish(&mut gpu, source)
}

// The display to open on: --display's, which must be connected, or else the
// setting's, or else the primary one, with a notice that the setting's
// display isn't connected.
fn choose_display<'a>(displays: &'a [Display], arguments: &Arguments, startup: &mut Startup) -> Result<&'a Display, String> {
    let primary = displays::primary(displays).ok_or("There's no primary display.")?;
    let setting = startup.common.display.clone();
    match &arguments.display {
        Some(name) if name.eq_ignore_ascii_case("Primary") => Ok(primary),
        Some(name) => displays::find(displays, name).ok_or_else(|| {
            let connected: Vec<&str> = displays.iter().map(|display| display.identifier.as_str()).collect();
            format!("The display {name} isn't connected. These are: {}.", connected.join(", "))
        }),
        None if setting.eq_ignore_ascii_case("Primary") => Ok(primary),
        None => Ok(displays::find(displays, &setting).unwrap_or_else(|| {
            let message = format!("The display {setting} isn't connected, so the window is on the primary display.");
            startup.notify(Severity::Warning, "Settings.json", &message);
            primary
        })),
    }
}

// Draws a frame: the theme's background, the program's 3D, unless it's
// hidden, and the interface over them, and presents it. Returns what the
// frame shows, if it's captured.
fn draw<P: Program>(gpu: &mut Gpu, presenter: &mut Presenter, renderer: &mut Renderer, ui: &mut Interface, program: &mut P, with_program: bool, capture: bool) -> Result<Option<Image>, String> {
    let target = presenter.begin().map_err(|error| format!("Couldn't begin a frame: {error}"))?;
    let background = ui.theme().background;
    unsafe { target.clear([background.r, background.g, background.b, 1.0]) };
    if with_program {
        program.render(gpu, &target).map_err(|error| format!("Couldn't draw the frame: {error}"))?;
    }
    renderer.draw(gpu, &target, &ui.draw, &mut ui.fonts).map_err(|error| format!("Couldn't draw the interface: {error}"))?;
    let readback = if capture { Some(unsafe { Readback::record(&gpu.device, &target) }.map_err(|error| format!("Couldn't capture the frame: {error}"))?) } else { None };
    let fence = presenter.present(gpu, target).map_err(|error| format!("Couldn't present: {error}"))?;
    readback.map(|readback| readback.read(gpu, fence).map_err(|error| format!("Couldn't read the frame back: {error}"))).transpose()
}

// Waits for the GPU to finish the last frame, as the presenter does as it
// goes, on every way out of the loop, but says so when the wait fails, and
// says whether a script's expectations were met.
fn finish(gpu: &mut Gpu, source: Source) -> Result<(), String> {
    gpu.flush().map_err(|error| format!("Couldn't wait for the GPU: {error}"))?;
    match source {
        Source::Script { runner, .. } if !runner.failures.is_empty() => Err(format!("The script's expectations weren't met:\n{}", runner.failures.join("\n"))),
        Source::Script { frame, .. } => {
            println!("The script passed, in {frame} frames.");
            Ok(())
        }
        Source::User { .. } => Ok(()),
    }
}

// Where the frames' input comes from: the user's devices, read with
// GameInput, or a script, on a clock of its own, which the user's input
// never reaches.
enum Source {
    User {
        // None with --no-input.
        devices: Option<Rc<RefCell<Devices>>>,
        reader: Box<InputReader>,
        // When the first frame read its input, which time counts from.
        start: Option<Instant>,
    },
    Script {
        runner: ScriptRunner,
        // The next frame's number, from 0.
        frame: u64,
        // Where captures go.
        folder: PathBuf,
    },
}

// A frame's input, from its source.
struct Input {
    events: Vec<Event>,
    // Seconds since the start.
    time: f64,
    // When GameInput got the newest reading that the frame took, for the
    // input latency.
    input_time: Option<i64>,
    // Whether the window got or lost focus since the last frame.
    focus_changed: bool,
    // What a script does around the frame.
    script: Option<ScriptFrame>,
}

impl Source {
    // Every program needs GameInput 4.0.306 installed, and doesn't start
    // without it, even when it doesn't read it (see Library/GameInput).
    fn open(arguments: &Arguments, gamepad: bool, folder: &Path) -> Result<Source, String> {
        if let Some(path) = &arguments.script {
            let script = Script::load(path)?;
            game_input::installation::installed_runtime()?;
            return Ok(Source::Script {
                runner: ScriptRunner::new(script),
                frame: 0,
                folder: folder.to_path_buf(),
            });
        }
        let devices = if arguments.no_input {
            game_input::installation::installed_runtime()?;
            None
        } else {
            Some(Rc::new(RefCell::new(Devices::new()?)))
        };
        Ok(Source::User {
            devices,
            reader: Box::new(InputReader::new(gamepad)),
            start: None,
        })
    }

    // The devices, for programs that read them themselves.
    fn devices(&self) -> Option<Rc<RefCell<Devices>>> {
        match self {
            Source::User { devices, .. } => devices.clone(),
            Source::Script { .. } => None,
        }
    }

    // The frame's input. A script's steps for the program happen here, before
    // the frame.
    fn read<P: Program>(&mut self, window: &mut Window, ui: &Interface, program: &mut P) -> Result<Input, String> {
        let focus_changes = window.take_focus_changes();
        match self {
            Source::User { devices, reader, start } => {
                if let Some(devices) = devices {
                    devices.borrow_mut().read();
                }
                let read = reader.frame(devices.as_ref().map(|devices| devices.borrow()).as_deref(), window, &focus_changes);
                Ok(Input {
                    events: read.events,
                    time: start.get_or_insert_with(Instant::now).elapsed().as_secs_f64(),
                    input_time: read.input_time,
                    focus_changed: !focus_changes.is_empty(),
                    script: None,
                })
            }
            Source::Script { runner, frame, .. } => {
                let number = *frame;
                *frame += 1;
                let script = runner.frame(number, ui)?;
                for step in &script.program_steps {
                    program.script(step).map_err(|error| format!("Frame {number}: Program step: {error}"))?;
                }
                Ok(Input {
                    events: script.events.clone(),
                    time: number as f64 * runner.frame_time(),
                    input_time: None,
                    focus_changed: false,
                    script: Some(script),
                })
            }
        }
    }

    // After a scripted frame is drawn: saves its captures, with the state,
    // and checks its expectations. Returns whether the script is done.
    fn check(&mut self, script: ScriptFrame, image: Option<Image>, state: impl FnOnce() -> json::Value) -> Result<bool, String> {
        let Source::Script { runner, frame, folder } = self else {
            return Ok(false);
        };
        let number = *frame - 1;
        let state = state();
        if let Some(image) = image {
            for name in &script.captures {
                save_capture(folder, name, &image, &state)?;
            }
        }
        for expectation in &script.expectations {
            runner.check(number, expectation, &state);
        }
        Ok(number >= runner.last_frame())
    }
}

fn save_capture(folder: &Path, name: &str, image: &Image, state: &json::Value) -> Result<(), String> {
    let png = folder.join(format!("{name}.png"));
    write_png(&png, image).map_err(|error| format!("Couldn't write {}: {error}", png.display()))?;
    let path = folder.join(format!("{name}.json"));
    std::fs::write(&path, state.to_json() + "\n").map_err(|error| format!("Couldn't write {}: {error}", path.display()))
}
