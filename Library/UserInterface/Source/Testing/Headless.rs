// Runs a program's frames without a window, for tests: input that the test
// makes up, on a fixed clock, through the same interface as a real run, so
// that presses land on the cards' real layout. DirectWrite lays text out
// without a GPU. A test that wants to see a frame draws it offscreen with
// `render`.
//
// Each input method runs one frame with the input, as if the user did it at
// 60 frames a second, and returns the harness, so that a test reads like
// what it does:
//
//     let mut app = Headless::new(program);
//     app.click("Save").key(Key::F11);
//     assert!(app.fullscreen());
//
// The frame shows all that the input did: a click's press opens a popup
// whose items the next input can point at, and a message's every effect is
// in the frame that applies it.

use graphics::{Frame, Gpu, Image, Offscreen};

use crate::application::performance::Performance;
use crate::application::program::{Program, Startup};
use crate::core::interface::{Interface, Request};
use crate::core::tree::Tree;
use crate::design::theme::ThemeKind;
use crate::design::tokens::{FULLSCREEN_HEIGHT, MAXIMIZED_HEIGHT, WINDOW_WIDTH};
use crate::drawing::geometry::{Point, Rect, Size};
use crate::drawing::renderer::Renderer;
use crate::drawing::text::Fonts;
use crate::input::devices::{Gamepad, Key, Modifiers, MouseButton};
use crate::input::events::Event;
use crate::platform::clipboard::Clipboard;
use crate::testing::script::DEFAULT_FRAME_TIME;

pub struct Headless<P: Program> {
    pub program: P,
    pub ui: Interface,
    // The tree the last frame showed, which the next frame's input is on.
    shown: Option<Tree<P::Message>>,
    frame: u64,
    fullscreen: bool,
    pending: Vec<Event>,
    // Messages a test sends, applied in the next frame before the input's.
    sent: Vec<P::Message>,
    pointer: Point,
    gamepad: Gamepad,
    renderer: Option<Renderer>,
    // A window of a size other than the supported setups', like an overlay.
    custom_size: Option<Size>,
    closed: bool,
}

impl<P: Program> Headless<P> {
    // The program in a maximized window that has focus, in the light theme.
    pub fn new(program: P) -> Self {
        Self::with_theme(program, ThemeKind::Light)
    }

    pub fn with_theme(program: P, theme: ThemeKind) -> Self {
        let fonts = Fonts::new().expect("DirectWrite couldn't start");
        let mut headless = Self {
            program,
            ui: Interface::new(fonts, theme, Clipboard::Own(String::new())),
            shown: None,
            frame: 0,
            fullscreen: false,
            pending: vec![Event::FocusGained(Vec::new())],
            sent: Vec::new(),
            pointer: Point::ZERO,
            gamepad: Gamepad::default(),
            renderer: None,
            custom_size: None,
            closed: false,
        };
        headless.ui.environment.scripted = true;
        // Frames would wait for vertical sync, as a real run's start out.
        headless.ui.environment.vertical_sync = true;
        headless.ui.performance = Some(Performance::without_latency("none in a test"));
        headless.run_frame();
        headless
    }

    // The program that `create` makes from the start, whose notices show
    // under its app bar.
    pub fn start(mut startup: Startup, create: impl FnOnce(&mut Startup) -> Result<P, String>) -> Self {
        let program = create(&mut startup).expect("the program couldn't start");
        let mut headless = Self::new(program);
        headless.ui.notices = startup.take_notices();
        headless.run_frame();
        headless
    }

    // A program in a window of the size, like an overlay, which has no
    // PerfBar.
    pub fn with_size(program: P, theme: ThemeKind, size: Size) -> Self {
        let mut headless = Self::with_theme(program, theme);
        headless.custom_size = Some(size);
        headless.ui.performance = None;
        headless.run_frame();
        headless
    }

    pub fn size(&self) -> Size {
        self.custom_size
            .unwrap_or_else(|| Size::new(WINDOW_WIDTH, if self.fullscreen { FULLSCREEN_HEIGHT } else { MAXIMIZED_HEIGHT }))
    }

    pub fn fullscreen(&self) -> bool {
        self.fullscreen
    }

    pub fn set_fullscreen(&mut self, fullscreen: bool) -> &mut Self {
        self.fullscreen = fullscreen;
        self.run_frame()
    }

    // Whether frames would wait for vertical sync, which a test's frames
    // start with, as a real run's do, and F10 switches.
    pub fn vertical_sync(&self) -> bool {
        self.ui.vertical_sync()
    }

    // Whether the program closed its window with Context::close.
    pub fn closed(&self) -> bool {
        self.closed
    }

    pub fn frame_number(&self) -> u64 {
        self.frame
    }

    pub fn time(&self) -> f64 {
        self.frame as f64 * DEFAULT_FRAME_TIME
    }

    // How many messages the last frame applied, counting those that
    // messages sent in turn.
    pub fn messages_applied(&self) -> usize {
        self.ui.messages_applied
    }

    fn run_frame(&mut self) -> &mut Self {
        self.frame += 1;
        let mut events = std::mem::take(&mut self.pending);
        events.push(Event::Gamepad(self.gamepad));
        let sent = std::mem::take(&mut self.sent);
        let size = self.size();
        let time = self.time();
        self.ui.frame(&mut self.program, &mut self.shown, &events, sent, time, size, self.fullscreen);
        for request in std::mem::take(&mut self.ui.requests) {
            match request {
                Request::ToggleFullscreen => self.fullscreen = !self.fullscreen,
                Request::ToggleVerticalSync => self.ui.environment.vertical_sync = !self.ui.environment.vertical_sync,
                Request::Close => self.closed = true,
            }
        }
        self
    }

    // Runs a frame with whatever input is pending.
    pub fn frame(&mut self) -> &mut Self {
        self.run_frame()
    }

    pub fn frames(&mut self, count: usize) -> &mut Self {
        for _ in 0..count {
            self.run_frame();
        }
        self
    }

    // Runs frames for the seconds.
    pub fn wait(&mut self, seconds: f64) -> &mut Self {
        self.frames((seconds / DEFAULT_FRAME_TIME).ceil() as usize)
    }

    // Sends a message to the program, as a widget would, and runs the frame
    // that applies it.
    pub fn send_message(&mut self, message: P::Message) -> &mut Self {
        self.sent.push(message);
        self.run_frame()
    }

    // Queues a message for the next frame without running it, so that
    // several go in one frame.
    pub fn queue_message(&mut self, message: P::Message) -> &mut Self {
        self.sent.push(message);
        self
    }

    // The area of the widget with the name, in the last frame.
    pub fn target(&self, name: &str) -> Rect {
        match self.ui.target(name) {
            Some(target) => target.rect,
            None => {
                let names: Vec<&str> = self.ui.targets().iter().filter(|target| target.listed).map(|target| target.name.as_str()).collect();
                panic!("The last frame didn't show {name}. It showed: {}", names.join(", "));
            }
        }
    }

    pub fn has_target(&self, name: &str) -> bool {
        self.ui.target(name).is_some()
    }

    pub fn label(&self, name: &str) -> Option<String> {
        self.ui.target(name).and_then(|target| target.label.clone())
    }

    pub fn is_enabled(&self, name: &str) -> bool {
        self.ui.target(name).is_some_and(|target| target.enabled)
    }

    // What scripts save: the program's state and the interface's.
    pub fn state(&self) -> json::Value {
        crate::application::run::state(&self.program, &self.ui)
    }

    pub fn pointer(&self) -> Point {
        self.pointer
    }

    // Moves the pointer to the middle of the widget.
    pub fn move_to(&mut self, name: &str) -> &mut Self {
        let point = self.target(name).center();
        self.move_point(point)
    }

    pub fn move_point(&mut self, point: Point) -> &mut Self {
        self.pointer = point;
        self.pending.push(Event::PointerMoved(point));
        self.run_frame()
    }

    // Presses the left button on the widget, and holds it.
    pub fn press(&mut self, name: &str) -> &mut Self {
        let point = self.target(name).center();
        self.press_point(point)
    }

    pub fn press_point(&mut self, point: Point) -> &mut Self {
        self.press_button(point, MouseButton::Left)
    }

    pub fn press_button(&mut self, point: Point, button: MouseButton) -> &mut Self {
        self.pointer = point;
        self.pending.push(Event::PointerMoved(point));
        self.pending.push(Event::ButtonPressed(button));
        self.run_frame()
    }

    pub fn release(&mut self) -> &mut Self {
        self.release_button(MouseButton::Left)
    }

    pub fn release_button(&mut self, button: MouseButton) -> &mut Self {
        self.pending.push(Event::ButtonReleased(button));
        self.run_frame()
    }

    // A press on the widget, and its release the next frame.
    pub fn click(&mut self, name: &str) -> &mut Self {
        self.press(name).release()
    }

    pub fn click_point(&mut self, point: Point) -> &mut Self {
        self.press_point(point).release()
    }

    // A right press and its release, which asks for a context menu.
    pub fn right_click(&mut self, name: &str) -> &mut Self {
        let point = self.target(name).center();
        self.press_button(point, MouseButton::Right).release_button(MouseButton::Right)
    }

    pub fn double_click(&mut self, name: &str) -> &mut Self {
        let point = self.target(name).center();
        self.pointer = point;
        self.pending.push(Event::PointerMoved(point));
        self.pending.push(Event::ButtonPressed(MouseButton::Left));
        self.pending.push(Event::ButtonReleased(MouseButton::Left));
        self.run_frame();
        self.pending.push(Event::ButtonPressed(MouseButton::Left));
        self.pending.push(Event::ButtonReleased(MouseButton::Left));
        self.run_frame()
    }

    // Presses at one point, moves to the other over `steps` frames, and lets
    // go there.
    pub fn drag(&mut self, from: Point, to: Point, steps: usize) -> &mut Self {
        self.drag_button(from, to, steps, MouseButton::Left)
    }

    pub fn drag_button(&mut self, from: Point, to: Point, steps: usize, button: MouseButton) -> &mut Self {
        self.press_button(from, button);
        let steps = steps.max(1);
        for step in 1..=steps {
            self.move_point(from.lerp(to, step as f32 / steps as f32));
        }
        self.release_button(button)
    }

    // Turns the wheel over the widget, by notches, up for more than 0.
    pub fn scroll(&mut self, name: &str, notches: f32) -> &mut Self {
        let point = self.target(name).center();
        self.scroll_point(point, notches)
    }

    pub fn scroll_point(&mut self, point: Point, notches: f32) -> &mut Self {
        self.pointer = point;
        self.pending.push(Event::PointerMoved(point));
        self.pending.push(Event::Wheel(Point::new(0.0, notches)));
        self.run_frame()
    }

    // Presses the key, and lets go of it the next frame.
    pub fn key(&mut self, key: Key) -> &mut Self {
        self.key_with(Modifiers::NONE, key)
    }

    // Presses the key with the modifiers, like Ctrl+S, and lets go of them.
    pub fn key_with(&mut self, modifiers: Modifiers, key: Key) -> &mut Self {
        let mut keys = Vec::new();
        for (held, modifier) in [(modifiers.control, Key::CONTROL), (modifiers.shift, Key::SHIFT), (modifiers.alt, Key::ALT)] {
            if held {
                keys.push(modifier);
            }
        }
        keys.push(key);
        self.pending.extend(keys.iter().map(|key| Event::KeyPressed { key: *key, repeat: false }));
        self.run_frame();
        self.pending.extend(keys.iter().rev().map(|key| Event::KeyReleased(*key)));
        self.run_frame()
    }

    // Presses the key and holds it, until release_key.
    pub fn hold(&mut self, key: Key) -> &mut Self {
        self.pending.push(Event::KeyPressed { key, repeat: false });
        self.run_frame()
    }

    pub fn release_key(&mut self, key: Key) -> &mut Self {
        self.pending.push(Event::KeyReleased(key));
        self.run_frame()
    }

    // A held key's repeat, as Windows sends while it stays down.
    pub fn repeat(&mut self, key: Key) -> &mut Self {
        self.pending.push(Event::KeyPressed { key, repeat: true });
        self.run_frame()
    }

    // Sends the events as they are, all in one frame.
    pub fn send(&mut self, events: &[Event]) -> &mut Self {
        self.pending.extend_from_slice(events);
        self.run_frame()
    }

    // Queues the events for the next frame without running it, unlike
    // send, so that a test can look at the very frame that reads them,
    // which frame runs.
    pub fn queue(&mut self, events: &[Event]) -> &mut Self {
        self.pending.extend_from_slice(events);
        self
    }

    // Types the text, all in one frame.
    pub fn type_text(&mut self, text: &str) -> &mut Self {
        self.pending.extend(text.chars().map(Event::Text));
        self.run_frame()
    }

    pub fn set_gamepad(&mut self, gamepad: Gamepad) -> &mut Self {
        self.gamepad = gamepad;
        self.run_frame()
    }

    // Gives the window focus, or takes it away.
    pub fn focus(&mut self, focused: bool) -> &mut Self {
        self.pending.push(if focused { Event::FocusGained(Vec::new()) } else { Event::FocusLost });
        self.run_frame()
    }

    // Draws the last frame offscreen, as the window would, with the
    // program's 3D. The first call prepares the program.
    pub fn render(&mut self, gpu: &mut Gpu) -> windows_core::Result<Image> {
        let size = self.size();
        let background = self.ui.theme().background;
        let clear = [background.r, background.g, background.b, 1.0];
        let mut target = Offscreen::new(gpu, size.width as u32, size.height as u32, clear)?;
        let frame = target.begin()?;
        unsafe { frame.clear(clear) };
        self.record(gpu, &frame)?;
        target.finish(gpu, frame)
    }

    // Records the last frame's drawing into a frame of the caller's, as
    // the window would draw it over the theme's background: the program's
    // 3D, then the interface. For drawing a frame into a target that's kept
    // from frame to frame, like a benchmark's. The first call prepares the
    // program.
    pub fn record(&mut self, gpu: &mut Gpu, frame: &Frame) -> windows_core::Result<()> {
        if self.renderer.is_none() {
            self.program.prepare(gpu)?;
            self.renderer = Some(Renderer::new(gpu)?);
        }
        self.program.render(gpu, frame)?;
        self.renderer.as_mut().unwrap().draw(gpu, frame, &self.ui.draw, &mut self.ui.fonts)
    }
}
