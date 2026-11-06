// The main loop of every GUI program, built for the lowest input latency:
//
// 1. Handle the window's messages, without waiting.
// 2. Go on handling them until the presenter is ready: the GPU has finished
//    the last frame, and DXGI's queue has room.
// 3. Only then read the input, from GameInput, build the frame's interface
//    with the newest of it, draw the frame, and present it without waiting
//    for vertical sync.
//
// So the input that a frame shows is as fresh as it can be when the frame
// reaches the screen. While the window is minimized, the loop waits for
// messages instead, which stops it drawing.
//
// With vertical sync, frames present at the screen's refreshes, and DXGI's
// queue has room again only once the last frame has gone to the screen, so
// the loop waits for that, and for messages, instead of spinning: a frame
// couldn't begin any sooner, and the input is still read as soon as one can.
// Coding agents' test runs use it, so that they don't take the machine from
// someone, like the user playing a game. F10 switches it on and off.

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::rc::Rc;
use std::time::Instant;

use graphics::{AdapterChoice, Gpu, Presenter, Readback, write_png};

use crate::application::performance::{Frame, Performance};
use crate::application::program::{Arguments, Options, Program, Startup};
use crate::application::settings::{Settings, WindowMode};
use crate::core::ui::{Environment, Request, Ui};
use crate::design::theme::ThemeKind;
use crate::drawing::geometry::Size;
use crate::drawing::renderer::Renderer;
use crate::drawing::text::Fonts;
use crate::input::reader::InputReader;
use crate::platform::clipboard::Clipboard;
use crate::platform::clock;
use crate::platform::input_devices::Devices;
use crate::platform::window::Window;
use crate::testing::script::{Script, ScriptRunner};
use crate::widgets::info_bar::Severity;

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
pub fn state<P: Program>(program: &P, ui: &Ui) -> json::Value {
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
    let scripted = arguments.script.is_some();
    let script = arguments.script.as_deref().map(Script::load).transpose()?;
    let data_folder = arguments.output.clone().unwrap_or_else(|| folder.clone());
    if let Some(output) = &arguments.output {
        std::fs::create_dir_all(output).map_err(|error| format!("Couldn't make the folder {}: {error}", output.display()))?;
    }

    // The settings that every program has, then the program's own.
    let mut settings = Settings::load(&folder.join("Settings.json"));
    let display_setting = settings.string("Display", "Primary");
    let modes = [WindowMode::Maximized.name(), WindowMode::BorderlessFullscreen.name()];
    let mode = settings.choice("WindowMode", options.window_mode.name(), &modes);
    let mode = if mode == WindowMode::Maximized.name() { WindowMode::Maximized } else { WindowMode::BorderlessFullscreen };
    let allow_tearing = settings.boolean("AllowTearing", true);
    let vertical_sync = settings.boolean("VerticalSync", false) || arguments.vsync;
    let mut startup = Startup::new(settings, &data_folder, scripted);
    startup.arguments = arguments.own.clone();

    // All input comes from GameInput, except in a scripted run, which the
    // user's input never reaches, and with --no-input.
    let devices = if scripted || arguments.no_input {
        None
    } else {
        match Devices::new() {
            Ok(devices) => Some(Rc::new(RefCell::new(devices))),
            Err(message) => {
                startup.notify(Severity::Error, "GameInput isn't available", &format!("The program can't read any input: {message}."));
                None
            }
        }
    };
    startup.devices = devices.clone();

    // The display: --display's, which must be connected, or else the
    // setting's, or else the primary one.
    let displays = displays::enumerate().map_err(|error| format!("Couldn't find the displays: {error}"))?;
    let primary = displays::primary(&displays).ok_or("There's no primary display.")?;
    let display = match &arguments.display {
        Some(name) if name.eq_ignore_ascii_case("Primary") => primary,
        Some(name) => displays::find(&displays, name).ok_or_else(|| {
            let connected: Vec<&str> = displays.iter().map(|display| display.identifier.as_str()).collect();
            format!("The display {name} isn't connected. These are: {}.", connected.join(", "))
        })?,
        None if display_setting.eq_ignore_ascii_case("Primary") => primary,
        None => match displays::find(&displays, &display_setting) {
            Some(display) => display,
            None => {
                startup.notify(
                    Severity::Warning,
                    "Settings.json",
                    &format!("The display {display_setting} isn't connected, so the window is on the primary display."),
                );
                primary
            }
        },
    };

    let fonts = Fonts::new().map_err(|error| format!("Couldn't start DirectWrite: {error}"))?;
    let clipboard = if scripted { Clipboard::Own(String::new()) } else { Clipboard::Windows };
    let mut ui = Ui::new(fonts, options.theme, clipboard);
    // The performance stats along the top of the window, with the input
    // latency to the screen, which a scripted run has no input for, nor a
    // run without input, which then also doesn't start the ETW session that
    // every process's presents would go through.
    ui.performance = Some(if scripted {
        Performance::without_latency("none in a scripted run")
    } else if arguments.no_input {
        Performance::without_latency("none without input")
    } else {
        Performance::measure(options.name)
    });
    let mut program = create(&mut startup)?;
    ui.notices = startup.take_notices();
    if !scripted && let Err(error) = startup.settings.write_if_missing() {
        ui.notices.push(crate::application::program::Notice {
            severity: Severity::Warning,
            title: "Settings.json".to_string(),
            message: format!("Couldn't write it: {error}."),
        });
    }

    let mut window = Window::create(&program.title(), display.monitor, mode).map_err(|error| format!("Couldn't open the window: {error}"))?;
    window.set_dark_title_bar(ui.theme().kind == ThemeKind::Dark);
    let mut gpu = Gpu::new(AdapterChoice::Monitor(window.monitor())).map_err(|error| format!("Couldn't start Direct3D 12: {error}"))?;
    let (width, height) = window.client_size();
    // The renderer before the presenter: what's made last goes first, and
    // the presenter waits for the GPU to finish the last frame as it goes,
    // before the renderer's buffers and the program's resources go.
    let mut renderer = Renderer::new(&gpu).map_err(|error| format!("Couldn't make the renderer: {error}"))?;
    let mut presenter = Presenter::new(&gpu, window.hwnd(), width, height, allow_tearing).map_err(|error| format!("Couldn't make the swap chain: {error}"))?;
    presenter.set_vsync(vertical_sync);
    program.prepare(&mut gpu).map_err(|error| format!("Couldn't prepare to draw: {error}"))?;
    ui.environment = Environment {
        gpu: gpu.name.clone(),
        display: display.identifier.clone(),
        display_name: display.name.clone(),
        tearing: presenter.tearing(),
        vertical_sync,
        scripted,
    };
    let mut title = program.title();
    window.show(!arguments.no_activate);

    let mut reader = InputReader::new(options.gamepad);
    let mut runner = script.map(ScriptRunner::new);
    let start = Instant::now();
    let mut frame = 0u64;
    let mut problems = Vec::new();
    'frames: loop {
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
        let (width, height) = window.client_size();
        if (window.minimized() || width == 0 || height == 0) && runner.is_none() {
            // A minimized window doesn't draw, but the program still hears
            // what happened, like the window losing focus as it minimized,
            // so that it can stop what it's doing, like a gamepad's motors.
            let focus_changes = window.take_focus_changes();
            if let Some(devices) = &devices {
                devices.borrow_mut().read();
            }
            let input = reader.frame(devices.as_ref().map(|devices| devices.borrow()).as_deref(), &mut window, &focus_changes);
            if !focus_changes.is_empty() && problems.is_empty() && runner.is_none() {
                let (width, height) = presenter.size();
                let fullscreen = window.mode() == WindowMode::BorderlessFullscreen;
                ui.begin_frame(&input.events, start.elapsed().as_secs_f64(), Size::new(width as f32, height as f32), fullscreen);
                program.update(&mut ui);
                ui.end_frame();
                for request in std::mem::take(&mut ui.requests) {
                    if request == Request::Close {
                        break 'frames;
                    }
                }
            }
            window.wait();
            continue;
        }
        if (width, height) != presenter.size() {
            presenter.resize(&mut gpu, width, height).map_err(|error| format!("Couldn't resize the swap chain: {error}"))?;
        }
        if !presenter.poll_ready(&gpu) {
            // With vertical sync, the loop waits for the GPU and DXGI instead
            // of spinning, which ends as soon as a frame can begin, or a
            // message comes. Without it, the loop spins, so that the frame
            // begins the moment that it can, without a thread's wake-up.
            if presenter.vsync() {
                presenter.wait_ready(&gpu, 100).map_err(|error| format!("Couldn't wait for the GPU: {error}"))?;
            }
            continue;
        }

        // The input, read as late as it can be. A script's frames count from
        // 0, and the user's input never reaches a scripted run.
        let script_number = frame;
        frame += 1;
        let focus_changes = window.take_focus_changes();
        let read = clock::now();
        let mut input_time = None;
        let events;
        let mut script_frame = None;
        let time = match &mut runner {
            Some(runner) => {
                let scripted_frame = runner.frame(script_number, &ui)?;
                events = scripted_frame.events.clone();
                for step in &scripted_frame.program_steps {
                    program.script(step).map_err(|error| format!("Frame {script_number}: Program step: {error}"))?;
                }
                script_frame = Some(scripted_frame);
                script_number as f64 * runner.frame_time()
            }
            None => {
                if let Some(devices) = &devices {
                    devices.borrow_mut().read();
                }
                let input = reader.frame(devices.as_ref().map(|devices| devices.borrow()).as_deref(), &mut window, &focus_changes);
                events = input.events;
                input_time = input.input_time;
                start.elapsed().as_secs_f64()
            }
        };
        let fullscreen = window.mode() == WindowMode::BorderlessFullscreen;
        ui.begin_frame(&events, time, Size::new(width as f32, height as f32), fullscreen);
        if problems.is_empty() {
            program.update(&mut ui);
        } else {
            ui.unsupported_setup(&problems);
        }
        ui.end_frame();
        for request in std::mem::take(&mut ui.requests) {
            match request {
                Request::ToggleFullscreen => window.set_mode(window.mode().other()),
                Request::ToggleVerticalSync => {
                    presenter.set_vsync(!presenter.vsync());
                    ui.environment.vertical_sync = presenter.vsync();
                    ui.environment.tearing = presenter.tearing();
                }
                Request::Close => break 'frames,
            }
        }
        window.set_cursor(ui.cursor());
        window.set_dark_title_bar(ui.theme().kind == ThemeKind::Dark);
        let new_title = program.title();
        if new_title != title {
            window.set_title(&new_title);
            title = new_title;
        }

        // The frame: the theme's background, the program's 3D, and the
        // interface over them.
        let target = presenter.begin().map_err(|error| format!("Couldn't begin a frame: {error}"))?;
        let background = ui.theme().background;
        unsafe { target.clear([background.r, background.g, background.b, 1.0]) };
        if problems.is_empty() {
            program.render(&mut gpu, &target).map_err(|error| format!("Couldn't draw the frame: {error}"))?;
        }
        renderer.draw(&gpu, &target, &ui.draw, &mut ui.fonts).map_err(|error| format!("Couldn't draw the interface: {error}"))?;
        let captures = script_frame.as_ref().map(|frame| frame.captures.clone()).unwrap_or_default();
        let readback = if captures.is_empty() {
            None
        } else {
            Some(unsafe { Readback::record(&gpu.device, &target) }.map_err(|error| format!("Couldn't capture the frame: {error}"))?)
        };
        let fence = presenter.present(&mut gpu, target).map_err(|error| format!("Couldn't present: {error}"))?;
        if let Some(performance) = &mut ui.performance {
            performance.presented(Frame {
                input: input_time,
                read,
                present: presenter.present_time(),
            });
        }
        gpu.report_messages();

        if let (Some(runner), Some(script_frame)) = (&mut runner, script_frame) {
            let state = state(&program, &ui);
            if let Some(readback) = readback {
                let image = readback.read(&gpu, fence).map_err(|error| format!("Couldn't read the frame back: {error}"))?;
                for name in &captures {
                    save_capture(&data_folder, name, &image, &state)?;
                }
            }
            for expectation in &script_frame.expectations {
                runner.check(script_number, expectation, &state);
            }
            if script_number >= runner.last_frame() {
                break;
            }
        }
    }

    // The presenter waits for the GPU to finish the last frame as it goes,
    // on every way out of here. This also says so when the wait fails.
    gpu.flush().map_err(|error| format!("Couldn't wait for the GPU: {error}"))?;
    if let Some(runner) = runner {
        if !runner.failures.is_empty() {
            return Err(format!("The script's expectations weren't met:\n{}", runner.failures.join("\n")));
        }
        println!("The script passed, in {frame} frames.");
    }
    Ok(())
}

fn save_capture(folder: &Path, name: &str, image: &graphics::Image, state: &json::Value) -> Result<(), String> {
    let png = folder.join(format!("{name}.png"));
    write_png(&png, image).map_err(|error| format!("Couldn't write {}: {error}", png.display()))?;
    let path = folder.join(format!("{name}.json"));
    std::fs::write(&path, state.to_json() + "\n").map_err(|error| format!("Couldn't write {}: {error}", path.display()))
}
