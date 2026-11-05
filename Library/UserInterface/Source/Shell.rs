// The main loop that every program shares: it opens the window with the
// program's settings, draws frames with the lowest latency until the window
// closes or a script ends, and gives each frame to the program, which reads
// its input and draws its cards with the interface's widgets.
//
// Each frame, once the renderer is ready, the loop reads the input, so that
// the frame shows the newest, and the program handles it and lays out and
// draws its interface on a canvas. Then the loop draws the program's 3D
// view, if it has one, the canvas over it in one draw call, and anything
// the program draws over the canvas, like an image in a card. F11 switches
// between maximized and borderless fullscreen, unless the program reads it
// itself. When the window isn't in a supported setup, the frame shows what's
// wrong instead of the program's interface, which takes no input.

use std::path::Path;

use application::{Arguments, FrameCounter, ProgramSettings, Settings, Window};
use canvas::{Fonts, Pipeline};
use graphics::Renderer;
use json::Value;
use windows_api::*;
use windows_core::Result;

use crate::clipboard::Clipboard;
use crate::{Frame, Interface, Status};

// What a program is made with, once its window and renderer are.
pub struct Setup<'a, S> {
    // The program's folder, with its Cargo.toml and Settings.json.
    pub folder: &'a Path,
    pub arguments: &'a Arguments,
    pub settings: &'a Settings,
    // The program's own settings, from its Settings.json.
    pub program_settings: S,
    // Problems with how the program was started, which the app bar shows.
    pub notices: &'a mut Vec<String>,
    pub renderer: &'a mut Renderer,
    // Whether a script is the program's input, instead of the user.
    pub scripted: bool,
}

// A program, which `run` runs.
pub trait Program: Sized {
    // Its own settings, in its Settings.json after the ones that every
    // program has, or () for none.
    type Settings: ProgramSettings;

    // Its name, for its window's title.
    const NAME: &'static str;

    // Whether the renderer has a depth buffer, for a 3D view.
    const DEPTH: bool = false;

    // Whether the loop switches to and from borderless fullscreen on F11.
    // A program that reads its keys some other way, like with GameInput,
    // asks for it with Frame::toggle_fullscreen instead.
    const FULLSCREEN_KEY: bool = true;

    fn new(setup: Setup<'_, Self::Settings>) -> Result<Self>;

    // Handles a frame's input, and draws its interface. Every frame comes
    // here, even when the window isn't in a supported setup, and what it
    // draws doesn't show.
    fn frame(&mut self, frame: &mut Frame) -> Result<()>;

    // Records draws into the frame under the interface, like a 3D view.
    unsafe fn draw_under(&mut self, _renderer: &Renderer) -> Result<()> {
        Ok(())
    }

    // Records draws into the frame over the interface, like an image in a
    // card.
    unsafe fn draw_over(&mut self, _renderer: &Renderer) -> Result<()> {
        Ok(())
    }

    // The program's state, for scripts to save and check. The loop adds the
    // interface's, as "Interface", and the window's, as "Window".
    fn state(&self) -> Value;

    // The window was minimized, and the loop sleeps until it's restored.
    fn minimized(&mut self) {}
}

// Runs a program, whose folder is `folder`, until its window closes, or its
// script ends.
pub fn run<P: Program>(folder: &Path) -> Result<()> {
    unsafe {
        application::prepare_process()?;
        // Problems with the settings and the command line, which the app
        // bar shows.
        let mut notices = Vec::new();
        let arguments = Arguments::parse(&mut notices);
        let (settings, program_settings) =
            Settings::for_program::<P::Settings>(&folder.join("Settings.json"), &arguments, &mut notices);
        let mut window = Window::open(P::NAME, &settings, &arguments, &mut notices)?;

        let fonts = Fonts::new()?;
        let mut renderer = Renderer::new(window.handle, settings.allow_tearing, P::DEPTH)?;
        let pipeline = Pipeline::new(&mut renderer, &fonts)?;
        let mut program = P::new(Setup {
            folder,
            arguments: &arguments,
            settings: &settings,
            program_settings,
            notices: &mut notices,
            renderer: &mut renderer,
            scripted: window.scripted(),
        })?;
        // The title bar says how frames are presented, as it affects latency.
        let adapter = renderer.description();
        window.set_title(&format!("{} - {adapter}", P::NAME))?;
        window.show(arguments.activate);
        let mut frames = FrameCounter::default();
        // A script copies and pastes on the interface's own clipboard, so
        // that it never touches the user's.
        let clipboard = if window.scripted() { Clipboard::memory() } else { Clipboard::System };
        let mut interface = Interface::new(clipboard);

        // Handle all pending messages, then render one frame once the
        // renderer is ready for it.
        while window.handle_messages() {
            if let Some([width, height]) = window.take_resize() {
                renderer.resize(width, height)?;
            }

            if window.minimized() {
                // A minimized window doesn't have focus, but minimizing it
                // takes its focus in the same messages, so no frame sees it
                // without focus. Popups close, as they do when the window
                // loses focus. Then the loop sleeps until something happens
                // instead of spinning.
                program.minimized();
                interface.reset();
                window.wait_for_message()?;
                frames.pause();
                continue;
            }

            // Spin until the GPU and DXGI can take a frame. Blocking on an
            // event would instead wait for Windows to wake the thread, which
            // can take a while if its core went into a power-saving state.
            // The wait is short, because frames don't wait for vertical
            // sync, and the loop keeps handling window messages through it.
            if !renderer.is_ready()? {
                std::hint::spin_loop();
                continue;
            }
            // The frame's setup, before the input is read.
            renderer.begin_frame()?;

            // Read input only now, right before drawing, so the frame shows
            // the newest. Without focus, there's none. A script can name a
            // widget instead of a point, which is where the frame before
            // drew it.
            let input = window.begin_frame(|name| interface.target(name).map(|area| [area.x, area.y, area.width, area.height]));
            frames.frame(input.seconds);
            let supported = window.problems.is_empty();
            let status = Status {
                frames_per_second: frames.per_second,
                adapter: &adapter,
                display: &window.display,
                notices: &notices,
            };
            let size = [renderer.width as f32, renderer.height as f32];
            let toggle = {
                let mut frame = interface.frame(&fonts, &input, status, size);
                if P::FULLSCREEN_KEY && frame.shortcut(VK_F11) {
                    frame.toggle_fullscreen();
                }
                program.frame(&mut frame)?;
                if !supported {
                    frame.show_window_problems(&window.supported, &window.problems);
                }
                frame.fullscreen_requested()
            };
            if toggle {
                window.toggle_fullscreen()?;
            }

            renderer.clear(interface.theme().clear_color());
            if supported {
                program.draw_under(&renderer)?;
            }
            pipeline.draw(&renderer, interface.instances());
            if supported {
                program.draw_over(&renderer)?;
            }
            window.handle_checkpoints(&input, |path| renderer.capture(path), || state(&program, &interface))?;
            renderer.present()?;
        }
        // What the frames used is dropped before the renderer.
        renderer.wait_for_gpu()?;
        window.finish()
    }
}

// The program's state, with the interface's.
pub fn state<P: Program>(program: &P, interface: &Interface) -> Value {
    let mut state = program.state();
    match &mut state {
        Value::Object(members) => members.push(("Interface".to_string(), interface.state())),
        other => *other = json::object([("Program", other.clone()), ("Interface", interface.state())]),
    }
    state
}
