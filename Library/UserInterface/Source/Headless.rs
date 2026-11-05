// Runs a program's frames without a window, for tests: input that the test
// makes up, on a fixed clock, through the same interface as a real run, so
// that presses land on the cards' real layout. Text is laid out with
// DirectWrite, which needs no GPU. A test that wants to see a frame can draw
// it offscreen with `render`.
//
// Each input method runs a frame, as if the user did it at 60 frames a
// second, and returns the harness, so that a test reads like what it does:
//
//     let mut app = Headless::new(program);
//     app.click("Save").key(Key::F11);
//     assert_eq!(app.target("Volume").width, 300.0);

use graphics::{Gpu, Image, Offscreen};

use crate::clipboard::Clipboard;
use crate::geometry::{Point, Rect, Size};
use crate::input::{Button, Event, Gamepad, Key, Modifiers};
use crate::program::{Program, Startup};
use crate::renderer::Renderer;
use crate::script::DEFAULT_FRAME_TIME;
use crate::text::Fonts;
use crate::theme::ThemeKind;
use crate::tokens::{FULLSCREEN_HEIGHT, MAXIMIZED_HEIGHT, WINDOW_WIDTH};
use crate::ui::{Request, Ui};

pub struct Headless<P: Program> {
    pub program: P,
    pub ui: Ui,
    frame: u64,
    fullscreen: bool,
    pending: Vec<Event>,
    pointer: Point,
    gamepad: Gamepad,
    renderer: Option<Renderer>,
    // A window of another size than the supported setups', like an overlay's.
    custom_size: Option<Size>,
}

impl<P: Program> Headless<P> {
    // A program in a maximized window that has focus, in the light theme.
    pub fn new(program: P) -> Self {
        Self::with_theme(program, ThemeKind::Light)
    }

    pub fn with_theme(program: P, theme: ThemeKind) -> Self {
        let fonts = Fonts::new().expect("DirectWrite couldn't start");
        let mut headless = Self {
            program,
            ui: Ui::new(fonts, theme, Clipboard::Own(String::new())),
            frame: 0,
            fullscreen: false,
            pending: vec![Event::FocusGained(Vec::new())],
            pointer: Point::default(),
            gamepad: Gamepad::default(),
            renderer: None,
            custom_size: None,
        };
        headless.ui.environment.scripted = true;
        headless.run_frame();
        headless
    }

    // A program that `create` makes from the start, whose notices show under
    // its app bar.
    pub fn start(mut startup: Startup, create: impl FnOnce(&mut Startup) -> Result<P, String>) -> Self {
        let program = create(&mut startup).expect("the program couldn't start");
        let mut headless = Self::new(program);
        headless.ui.notices = startup.take_notices();
        headless.run_frame();
        headless
    }

    // A program in a window of the size, like an overlay.
    pub fn with_size(program: P, theme: ThemeKind, size: Size) -> Self {
        let mut headless = Self::with_theme(program, theme);
        headless.custom_size = Some(size);
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

    pub fn frame_number(&self) -> u64 {
        self.frame
    }

    pub fn time(&self) -> f64 {
        self.frame as f64 * DEFAULT_FRAME_TIME
    }

    fn run_frame(&mut self) -> &mut Self {
        self.frame += 1;
        let mut events = std::mem::take(&mut self.pending);
        events.push(Event::Gamepad(self.gamepad));
        let size = self.size();
        self.ui.begin_frame(&events, self.time(), size, self.fullscreen);
        self.program.update(&mut self.ui);
        self.ui.end_frame();
        for request in std::mem::take(&mut self.ui.requests) {
            match request {
                Request::ToggleFullscreen => self.fullscreen = !self.fullscreen,
                Request::Close => {}
            }
        }
        self
    }

    // Runs a frame with whatever input is pending.
    pub fn frame(&mut self) -> &mut Self {
        self.run_frame()
    }

    // Runs a frame with the pending input, and another after it, so that
    // whatever the input made appear, like a popup, is in the targets.
    fn act(&mut self) -> &mut Self {
        self.run_frame();
        self.run_frame()
    }

    pub fn frames(&mut self, count: usize) -> &mut Self {
        for _ in 0..count {
            self.run_frame();
        }
        self
    }

    // Runs frames for the seconds of time.
    pub fn wait(&mut self, seconds: f64) -> &mut Self {
        let count = (seconds / DEFAULT_FRAME_TIME).ceil() as usize;
        self.frames(count)
    }

    // The area of the widget with the name, from the last frame.
    pub fn target(&self, name: &str) -> Rect {
        match self.ui.target(name) {
            Some(target) => target.rect,
            None => {
                let names: Vec<&str> = self.ui.targets().iter().filter(|target| target.listed).map(|target| target.name.as_str()).collect();
                panic!("the last frame didn't show {name}. It showed: {}", names.join(", "));
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
        crate::run::state(&self.program, &self.ui)
    }

    fn at(&self, name: &str) -> Point {
        self.target(name).center()
    }

    // Moves the pointer to the middle of the widget.
    pub fn move_to(&mut self, name: &str) -> &mut Self {
        let point = self.at(name);
        self.move_point(point)
    }

    pub fn move_point(&mut self, point: Point) -> &mut Self {
        self.pointer = point;
        self.pending.push(Event::PointerMoved(point));
        self.act()
    }

    // Presses the left button on the widget, and holds it.
    pub fn press(&mut self, name: &str) -> &mut Self {
        let point = self.at(name);
        self.press_point(point)
    }

    pub fn press_point(&mut self, point: Point) -> &mut Self {
        self.pointer = point;
        self.pending.push(Event::PointerMoved(point));
        self.pending.push(Event::ButtonPressed(Button::Left));
        self.act()
    }

    pub fn press_button(&mut self, point: Point, button: Button) -> &mut Self {
        self.pointer = point;
        self.pending.push(Event::PointerMoved(point));
        self.pending.push(Event::ButtonPressed(button));
        self.act()
    }

    pub fn release(&mut self) -> &mut Self {
        self.release_button(Button::Left)
    }

    pub fn release_button(&mut self, button: Button) -> &mut Self {
        self.pending.push(Event::ButtonReleased(button));
        self.act()
    }

    // A press on the widget, and its release the next frame.
    pub fn click(&mut self, name: &str) -> &mut Self {
        self.press(name).release()
    }

    pub fn click_point(&mut self, point: Point) -> &mut Self {
        self.press_point(point).release()
    }

    pub fn double_click(&mut self, name: &str) -> &mut Self {
        let point = self.at(name);
        self.pending.push(Event::PointerMoved(point));
        self.pending.push(Event::ButtonPressed(Button::Left));
        self.pending.push(Event::ButtonReleased(Button::Left));
        self.act();
        self.pending.push(Event::ButtonPressed(Button::Left));
        self.pending.push(Event::ButtonReleased(Button::Left));
        self.act()
    }

    // Presses at one point, moves to the other over `steps` frames, and
    // releases there.
    pub fn drag(&mut self, from: Point, to: Point, steps: usize) -> &mut Self {
        self.press_point(from);
        for step in 1..=steps.max(1) {
            self.move_point(from.lerp(to, step as f32 / steps.max(1) as f32));
        }
        self.release()
    }

    // Turns the wheel over the widget, by notches, up for more than 0.
    pub fn scroll(&mut self, name: &str, notches: f32) -> &mut Self {
        let point = self.at(name);
        self.scroll_point(point, notches)
    }

    pub fn scroll_point(&mut self, point: Point, notches: f32) -> &mut Self {
        self.pointer = point;
        self.pending.push(Event::PointerMoved(point));
        self.pending.push(Event::Wheel(Point::new(0.0, notches)));
        self.act()
    }

    // Presses the key and lets go of it the next frame.
    pub fn key(&mut self, key: Key) -> &mut Self {
        self.key_with(Modifiers::NONE, key)
    }

    // Presses the key with modifiers, like Ctrl+S, and lets go of them.
    pub fn key_with(&mut self, modifiers: Modifiers, key: Key) -> &mut Self {
        let mut keys = Vec::new();
        if modifiers.control {
            keys.push(Key::CONTROL);
        }
        if modifiers.shift {
            keys.push(Key::SHIFT);
        }
        if modifiers.alt {
            keys.push(Key::ALT);
        }
        keys.push(key);
        for key in &keys {
            self.pending.push(Event::KeyPressed { key: *key, repeat: false });
        }
        self.act();
        for key in keys.iter().rev() {
            self.pending.push(Event::KeyReleased(*key));
        }
        self.act()
    }

    // Presses the key, and holds it until release_key.
    pub fn hold(&mut self, key: Key) -> &mut Self {
        self.pending.push(Event::KeyPressed { key, repeat: false });
        self.act()
    }

    pub fn release_key(&mut self, key: Key) -> &mut Self {
        self.pending.push(Event::KeyReleased(key));
        self.act()
    }

    // Sends events as they are, all in one frame, like several presses
    // between two frames.
    pub fn send(&mut self, events: &[Event]) -> &mut Self {
        self.pending.extend_from_slice(events);
        self.act()
    }

    // A held key's repeat, as Windows sends while a key stays down.
    pub fn repeat(&mut self, key: Key) -> &mut Self {
        self.pending.push(Event::KeyPressed { key, repeat: true });
        self.act()
    }

    // Types the text, all in one frame.
    pub fn type_text(&mut self, text: &str) -> &mut Self {
        self.pending.extend(text.chars().map(Event::Text));
        self.act()
    }

    pub fn set_gamepad(&mut self, gamepad: Gamepad) -> &mut Self {
        self.gamepad = gamepad;
        self.act()
    }

    // Gives the window focus, or takes it away.
    pub fn focus(&mut self, focused: bool) -> &mut Self {
        self.pending.push(if focused { Event::FocusGained(Vec::new()) } else { Event::FocusLost });
        self.act()
    }

    pub fn pointer(&self) -> Point {
        self.pointer
    }

    // Draws the last frame offscreen, as the window would, with the
    // program's 3D. The first call prepares the program.
    pub fn render(&mut self, gpu: &mut Gpu) -> windows_core::Result<Image> {
        if self.renderer.is_none() {
            self.program.prepare(gpu)?;
            self.renderer = Some(Renderer::new(gpu)?);
        }
        let size = self.size();
        let background = self.ui.theme().background;
        let clear = [background.r, background.g, background.b, 1.0];
        let mut target = Offscreen::new(gpu, size.width as u32, size.height as u32, clear)?;
        let frame = target.begin()?;
        unsafe { frame.clear(clear) };
        self.program.render(gpu, &frame)?;
        self.renderer.as_mut().unwrap().draw(gpu, &frame, &self.ui.draw, &mut self.ui.fonts)?;
        target.finish(gpu, frame)
    }
}
