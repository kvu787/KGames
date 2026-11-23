// PixelFontTester: type ASCII into a box, and see it in the pixel font that
// Glyphs.aseprite at the root of the repository draws. See README.md.
//
// The main loop is Library/user-interface's, for the lowest input latency:
// it handles the window's messages, and hands their input to the tester,
// until the presenter is ready for a frame, and only then draws the tester as
// that input left it. So everything that a key does is in the next frame.

#![windows_subsystem = "windows"]

use std::process::ExitCode;

use graphics::{AdapterChoice, Gpu, Presenter};
use pixel_font_tester::arguments::{Arguments, USAGE};
use pixel_font_tester::font::{Font, glyphs_path};
use pixel_font_tester::renderer::Renderer;
use pixel_font_tester::scene::scene;
use pixel_font_tester::tester::Tester;
use pixel_font_tester::text::Text;
use pixel_font_tester::window::Window;

fn run() -> Result<(), String> {
    let arguments = Arguments::parse(std::env::args().skip(1)).map_err(|message| format!("{message}\nUsage: {USAGE}"))?;
    let font = Font::load(&glyphs_path())?;
    let mut tester = Tester::new(Text::typed(&arguments.text), arguments.scale, arguments.theme, arguments.vsync, &font);
    if let Some(spacing) = arguments.spacing {
        tester.set_spacing(spacing, &font);
    }

    let displays = displays::enumerate().map_err(|error| format!("Couldn't find the displays: {error}"))?;
    let display = if arguments.display.eq_ignore_ascii_case("Primary") {
        displays::primary(&displays).ok_or("There's no primary display.")?
    } else {
        displays::find(&displays, &arguments.display).ok_or_else(|| {
            let connected: Vec<&str> = displays.iter().map(|display| display.identifier.as_str()).collect();
            format!("The display {} isn't connected. These are: {}.", arguments.display, connected.join(", "))
        })?
    };

    let mut window = Window::create("PixelFontTester", display.monitor, tester.theme.colors().background).map_err(|error| format!("Couldn't open the window: {error}"))?;
    let mut gpu = Gpu::new(AdapterChoice::Monitor(window.monitor())).map_err(|error| format!("Couldn't start Direct3D 12: {error}"))?;
    let (width, height) = window.client_size();
    // The renderer before the presenter: what's made last goes first, and
    // the presenter waits for the GPU to finish the last frame as it goes,
    // before the renderer's buffers go.
    let mut renderer = Renderer::new(&mut gpu, &font).map_err(|error| format!("Couldn't make the renderer: {error}"))?;
    let mut presenter = Presenter::new(&gpu, window.hwnd(), width, height).map_err(|error| format!("Couldn't make the swap chain: {error}"))?;
    window.show(!arguments.no_activate);

    let mut problems = Vec::new();
    loop {
        // WM_CLOSE, like the taskbar's Close, quits at once.
        if !window.pump() {
            break;
        }
        for input in window.take_inputs() {
            if !arguments.no_input {
                tester.handle(input, &font);
            }
        }
        // Ctrl+Shift+Alt+F12 quits at once.
        if tester.quit {
            break;
        }
        presenter.set_vsync(tester.vsync);
        window.set_background(tester.theme.colors().background);
        if !window.minimized() && window.take_setup_changed() {
            problems = window.setup_problems();
        }
        let (width, height) = window.client_size();
        if window.minimized() || width == 0 || height == 0 {
            window.wait();
            continue;
        }
        if (width, height) != presenter.size() {
            presenter.resize(&mut gpu, width, height).map_err(|error| format!("Couldn't resize the swap chain: {error}"))?;
        }
        if !presenter.poll_ready(&gpu) {
            // With vertical sync, the loop waits for the GPU and DXGI, or a
            // message, instead of spinning. Without it, it spins, so that a
            // frame begins the moment that it can.
            if presenter.vsync() {
                presenter.wait_ready(&gpu, 100).map_err(|error| format!("Couldn't wait for the GPU: {error}"))?;
            }
            continue;
        }

        let frame = presenter.begin().map_err(|error| format!("Couldn't begin a frame: {error}"))?;
        renderer.draw(&gpu, &frame, &scene(&tester, &font, &problems)).map_err(|error| format!("Couldn't draw the frame: {error}"))?;
        presenter.present(&mut gpu, frame).map_err(|error| format!("Couldn't present: {error}"))?;
        gpu.report_messages();
    }
    gpu.flush().map_err(|error| format!("Couldn't wait for the GPU: {error}"))
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("PixelFontTester: {message}");
            ExitCode::FAILURE
        }
    }
}
