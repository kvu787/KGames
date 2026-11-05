// The interface of a frame. A program builds it each frame, in its update:
// it lays out its cards in fixed pixels, and calls a widget for each control,
// which draws it and says what the input did to it.
//
// Widgets are named, like "Volume" or "Devices/3". The name is what a widget
// is known by from one frame to the next, for its state, like whether it has
// focus, and what scripts and tests point at. Every widget that a frame
// shows is a target, with its area. The pointer is on the topmost target
// under it, from the last frame's targets, so a popup over a card takes the
// pointer from the card's widgets under it.

use std::any::{Any, TypeId};
use std::collections::HashMap;

use crate::clipboard::Clipboard;
use crate::geometry::{Color, Point, Rect, Size};
use crate::input::{Button, Gamepad, Input, Key, Modifiers};
use crate::painter::{DrawList, Layer, Painter};
use crate::text::Fonts;
use crate::theme::{Theme, ThemeKind};
use crate::tokens::{CURVE_LINEAR, Curve};

pub type Id = u64;

// A widget's identity, from its name: FNV-1a, which is fast for short names.
pub fn id(name: &str) -> Id {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in name.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash
}

// Which keys a widget takes while it has keyboard focus, so that the
// program's shortcuts don't also see them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Keys {
    #[default]
    None,
    // Space and Enter, like a button's.
    Activate,
    // The arrows, Home, End, Page Up and Page Down, and Space and Enter,
    // like a slider's or a list's.
    Navigate,
    // Everything typed, and the keys that edit text.
    Text,
}

// What a widget wants from the pointer and the keyboard.
#[derive(Clone, Copy, Debug, Default)]
pub struct Sense {
    // Presses on it, and clicks.
    pub click: bool,
    // Whether it takes keyboard focus when pressed, and with Tab.
    pub focusable: bool,
    pub keys: Keys,
}

impl Sense {
    // Takes the pointer's presses, but never keyboard focus.
    pub const CLICK: Sense = Sense {
        click: true,
        focusable: false,
        keys: Keys::None,
    };
    // Like a button: presses, focus, and Space and Enter.
    pub const BUTTON: Sense = Sense {
        click: true,
        focusable: true,
        keys: Keys::Activate,
    };
    pub const NAVIGATE: Sense = Sense {
        click: true,
        focusable: true,
        keys: Keys::Navigate,
    };
    pub const TEXT: Sense = Sense {
        click: true,
        focusable: true,
        keys: Keys::Text,
    };
    // Only covers what's under it, like a card.
    pub const NONE: Sense = Sense {
        click: false,
        focusable: false,
        keys: Keys::None,
    };
}

#[derive(Clone, Debug)]
pub struct Target {
    pub id: Id,
    pub name: String,
    pub label: Option<String>,
    // Where it is, inside the clip that it was in.
    pub rect: Rect,
    pub layer: Layer,
    pub enabled: bool,
    pub focusable: bool,
    pub keys: Keys,
    // Like "Button" or "Slider".
    pub kind: &'static str,
    // Whether scripts and tests see it. Cards and other surfaces that only
    // cover what's under them aren't listed.
    pub listed: bool,
}

// What the input did to a widget this frame.
#[derive(Clone, Copy, Debug, Default)]
pub struct Response {
    pub id: Id,
    pub rect: Rect,
    // The pointer is on it, and it's enabled.
    pub hovered: bool,
    // The left button went down on it this frame.
    pub pressed: bool,
    // It has the press: the button that went down on it is still down.
    pub held: bool,
    // Its press ended this frame, wherever the pointer was.
    pub released: bool,
    // Its press ended on it, or Space or Enter pressed it.
    pub clicked: bool,
    // Its press was the second of a double-click.
    pub double_clicked: bool,
    // The right button went down on it this frame.
    pub secondary_pressed: bool,
    pub focused: bool,
    // Its value changed, for widgets that have one.
    pub changed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Cursor {
    #[default]
    Arrow,
    Text,
    Hand,
    Move,
    ResizeHorizontal,
    ResizeVertical,
    Crosshair,
}

// What the program runs in, for showing in its bars.
#[derive(Clone, Debug, Default)]
pub struct Environment {
    // The GPU that draws, like "NVIDIA GeForce RTX 5090 Laptop GPU".
    pub gpu: String,
    // The display's identifier, like "CVVNY2", and its model.
    pub display: String,
    pub display_name: String,
    pub tearing: bool,
    // Whether input comes from a script instead of the user.
    pub scripted: bool,
}

// Things a program asks of its window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Request {
    ToggleFullscreen,
    Close,
}

#[derive(Clone, Copy, Debug)]
struct Animation {
    from: [f32; 4],
    to: [f32; 4],
    start: f64,
    duration: f32,
    curve: Curve,
}

impl Animation {
    fn value(&self, now: f64) -> [f32; 4] {
        let t = if self.duration <= 0.0 {
            1.0
        } else {
            ((now - self.start) as f32 / self.duration).clamp(0.0, 1.0)
        };
        let eased = self.curve.ease(t);
        std::array::from_fn(|index| self.from[index] + (self.to[index] - self.from[index]) * eased)
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct TooltipState {
    candidate: Option<Id>,
    since: f64,
    anchor: Point,
    dismissed: bool,
}

// How long the pointer must rest on a widget before its tooltip shows.
pub const TOOLTIP_DELAY: f64 = 0.5;

// The frame rate, measured over half-second windows.
#[derive(Clone, Copy, Debug, Default)]
pub struct FrameStats {
    pub frames_per_second: f32,
    // Milliseconds.
    pub frame_time: f32,
    window_start: f64,
    window_frames: u32,
}

pub struct Ui {
    pub(crate) theme: Theme,
    pub(crate) fonts: Fonts,
    pub(crate) draw: DrawList,
    pub(crate) input: Input,
    size: Size,
    fullscreen: bool,
    targets: Vec<Target>,
    previous: Vec<Target>,
    hovered: Option<Id>,
    active: Option<Id>,
    focused: Option<Id>,
    focus_visible: bool,
    // The keys that the focused widget takes.
    captured: Keys,
    popup: Option<Id>,
    popup_names: HashMap<Id, String>,
    // A press that only closed a popup, which nothing else may take.
    press_consumed: bool,
    escape_consumed: bool,
    tooltip: TooltipState,
    tooltips_enabled: bool,
    // The tooltip that this frame showed: its widget's name, and its area.
    pub(crate) shown_tooltip: Option<(String, Rect)>,
    enabled: bool,
    focusable: bool,
    animations: HashMap<(Id, u8), Animation>,
    memory: HashMap<(Id, TypeId), Box<dyn Any>>,
    pub(crate) clipboard: Clipboard,
    cursor: Cursor,
    frame: u64,
    stats: FrameStats,
    pub(crate) environment: Environment,
    pub(crate) requests: Vec<Request>,
    // The shell's: notices under the app bar, where the page starts, and
    // whether a status bar is under it.
    pub(crate) notices: Vec<crate::widgets::shell::Notice>,
    pub(crate) page_top: f32,
    pub(crate) status_bar_shown: bool,
}

impl Ui {
    pub fn new(fonts: Fonts, theme: ThemeKind, clipboard: Clipboard) -> Self {
        Self {
            theme: Theme::new(theme),
            fonts,
            draw: DrawList::default(),
            input: Input::default(),
            size: Size::default(),
            fullscreen: false,
            targets: Vec::new(),
            previous: Vec::new(),
            hovered: None,
            active: None,
            focused: None,
            focus_visible: false,
            captured: Keys::None,
            popup: None,
            popup_names: HashMap::new(),
            press_consumed: false,
            escape_consumed: false,
            tooltip: TooltipState::default(),
            tooltips_enabled: true,
            shown_tooltip: None,
            enabled: true,
            focusable: true,
            animations: HashMap::new(),
            memory: HashMap::new(),
            clipboard,
            cursor: Cursor::Arrow,
            frame: 0,
            stats: FrameStats::default(),
            environment: Environment::default(),
            requests: Vec::new(),
            notices: Vec::new(),
            page_top: 0.0,
            status_bar_shown: true,
        }
    }

    // Starts a frame of the size, with the events since the last one.
    pub(crate) fn begin_frame(&mut self, events: &[crate::input::Event], time: f64, size: Size, fullscreen: bool) {
        self.frame += 1;
        self.size = size;
        self.fullscreen = fullscreen;
        self.input.begin(time, events);
        self.fonts.begin_frame();
        self.draw.clear(size);
        std::mem::swap(&mut self.targets, &mut self.previous);
        self.targets.clear();
        self.enabled = true;
        self.focusable = true;
        self.cursor = Cursor::Arrow;
        self.press_consumed = false;
        self.escape_consumed = false;
        self.shown_tooltip = None;
        self.page_top = 0.0;
        self.status_bar_shown = true;
        self.update_stats(time);

        if self.input.focus_lost {
            self.active = None;
            self.popup = None;
        }

        // The pointer is on the topmost target under it, or only on the
        // widget that has the press, while one does.
        let pointer = self.input.pointer;
        let topmost = pointer.and_then(|point| {
            let mut best: Option<&Target> = None;
            for target in &self.previous {
                if target.rect.contains(point) && best.is_none_or(|best| target.layer >= best.layer) {
                    best = Some(target);
                }
            }
            best.map(|target| target.id)
        });
        self.hovered = match self.active {
            Some(active) => (topmost == Some(active)).then_some(active),
            None => topmost,
        };

        // A press outside an open popup closes it, and does nothing else.
        if let Some(popup) = self.popup
            && self.input.button_pressed(Button::Left)
        {
            let on_popup = topmost.is_some_and(|hit| {
                hit == popup || self.previous.iter().any(|target| target.id == hit && target.layer == Layer::Popup)
            });
            if !on_popup {
                self.popup = None;
                self.press_consumed = true;
            }
        }

        // Tooltips show once the pointer rests on a widget, and a press or
        // the wheel puts them away until the pointer moves to another.
        if self.hovered != self.tooltip.candidate {
            self.tooltip = TooltipState {
                candidate: self.hovered,
                since: time,
                anchor: pointer.unwrap_or_default(),
                dismissed: false,
            };
        }
        if self.input.button_pressed(Button::Left)
            || self.input.button_pressed(Button::Right)
            || self.input.wheel != Point::default()
        {
            self.tooltip.dismissed = true;
        }

        self.captured = match self.focused {
            Some(focused) => self
                .previous
                .iter()
                .find(|target| target.id == focused)
                .map(|target| target.keys)
                .unwrap_or_default(),
            None => Keys::None,
        };
        if self.popup.is_some() {
            self.captured = Keys::Navigate;
        }

        // Esc closes a popup, or else takes keyboard focus away.
        if self.input.key_pressed(Key::ESCAPE) {
            if self.popup.is_some() {
                self.popup = None;
                self.escape_consumed = true;
            } else if self.focused.is_some() {
                self.focused = None;
                self.escape_consumed = true;
            }
        }
    }

    pub(crate) fn end_frame(&mut self) {
        // Tab and Shift+Tab move keyboard focus through the widgets that take
        // it, in the order the frame showed them.
        if self.input.key_pressed(Key::TAB) && self.popup.is_none() && !self.input.modifiers().control {
            let order: Vec<Id> = self
                .targets
                .iter()
                .filter(|target| target.focusable && target.enabled && target.layer == Layer::Base)
                .map(|target| target.id)
                .collect();
            if !order.is_empty() {
                let backward = self.input.modifiers().shift;
                let next = match self.focused.and_then(|focused| order.iter().position(|id| *id == focused)) {
                    Some(index) if backward => (index + order.len() - 1) % order.len(),
                    Some(index) => (index + 1) % order.len(),
                    None if backward => order.len() - 1,
                    None => 0,
                };
                self.focused = Some(order[next]);
                self.focus_visible = true;
            }
        }
        // A press that no widget took takes focus away.
        if self.input.button_pressed(Button::Left) && !self.press_consumed && self.active.is_none() {
            self.focused = None;
        }
        // A widget that's gone, or disabled, can't keep focus or the press.
        if let Some(focused) = self.focused
            && !self.targets.iter().any(|target| target.id == focused && target.enabled)
        {
            self.focused = None;
        }
        if !self.input.button_down(Button::Left) {
            self.active = None;
        }
        if let Some(active) = self.active
            && !self.targets.iter().any(|target| target.id == active)
        {
            self.active = None;
        }
        if let Some(popup) = self.popup
            && !self.targets.iter().any(|target| target.id == popup)
        {
            self.popup = None;
        }
        if self.input.key_pressed_once(Key::F11) && self.captured != Keys::Text {
            self.requests.push(Request::ToggleFullscreen);
        }
    }

    fn update_stats(&mut self, time: f64) {
        let stats = &mut self.stats;
        stats.window_frames += 1;
        let elapsed = time - stats.window_start;
        if elapsed >= 0.5 {
            stats.frames_per_second = (stats.window_frames as f64 / elapsed) as f32;
            stats.frame_time = (elapsed * 1000.0 / stats.window_frames as f64) as f32;
            stats.window_start = time;
            stats.window_frames = 0;
        }
    }

    // The theme, whose roles color everything.
    pub fn theme(&self) -> &Theme {
        &self.theme
    }

    pub fn set_theme(&mut self, kind: ThemeKind) {
        if self.theme.kind != kind {
            self.theme = Theme::new(kind);
        }
    }

    // The window's client area.
    pub fn size(&self) -> Size {
        self.size
    }

    pub fn bounds(&self) -> Rect {
        Rect::new(0.0, 0.0, self.size.width, self.size.height)
    }

    pub fn fullscreen(&self) -> bool {
        self.fullscreen
    }

    pub fn toggle_fullscreen(&mut self) {
        self.requests.push(Request::ToggleFullscreen);
    }

    pub fn close(&mut self) {
        self.requests.push(Request::Close);
    }

    // Seconds since the program started, on the frames' clock, which runs
    // at a fixed rate for scripts and tests.
    pub fn time(&self) -> f64 {
        self.input.time
    }

    // Seconds since the last frame.
    pub fn delta(&self) -> f32 {
        self.input.delta
    }

    pub fn frame_number(&self) -> u64 {
        self.frame
    }

    pub fn stats(&self) -> FrameStats {
        self.stats
    }

    pub fn environment(&self) -> &Environment {
        &self.environment
    }

    // All of this frame's input, for programs that read it themselves, like
    // the keys held for driving.
    pub fn input(&self) -> &Input {
        &self.input
    }

    // Whether the window has focus. Without it, there's no input at all.
    pub fn window_focused(&self) -> bool {
        self.input.focused
    }

    pub fn gamepad(&self) -> &Gamepad {
        &self.input.gamepad
    }

    // Whether a key was pressed this frame, counting a held key's repeats,
    // unless the widget with keyboard focus takes it, like a text field takes
    // letters.
    pub fn key_pressed(&self, key: Key) -> bool {
        self.input.key_pressed(key) && !self.is_captured(key)
    }

    // Whether the key was pressed with exactly these modifiers, like Ctrl+S.
    pub fn shortcut(&self, modifiers: Modifiers, key: Key) -> bool {
        self.input.modifiers() == modifiers && self.key_pressed(key)
    }

    pub fn key_down(&self, key: Key) -> bool {
        self.input.key_down(key) && !(self.captured == Keys::Text && !key.is_modifier())
    }

    fn is_captured(&self, key: Key) -> bool {
        if key == Key::ESCAPE && self.escape_consumed {
            return true;
        }
        let navigation = matches!(
            key,
            Key::LEFT | Key::RIGHT | Key::UP | Key::DOWN | Key::HOME | Key::END | Key::PAGE_UP | Key::PAGE_DOWN
        );
        match self.captured {
            Keys::None => false,
            Keys::Activate => key == Key::SPACE || key == Key::ENTER,
            Keys::Navigate => navigation || key == Key::SPACE || key == Key::ENTER,
            Keys::Text => {
                let function = (0x70..=0x87).contains(&key.0);
                let control = self.input.modifiers().control;
                let editing = matches!(
                    key,
                    Key::A | Key::C | Key::X | Key::V | Key::Z | Key::Y | Key::LEFT | Key::RIGHT | Key::BACKSPACE | Key::DELETE | Key::HOME | Key::END
                );
                !function && key != Key::ESCAPE && key != Key::TAB && (!control || editing)
            }
        }
    }

    pub fn painter(&mut self) -> Painter<'_> {
        Painter {
            list: &mut self.draw,
            fonts: &mut self.fonts,
            theme_kind: self.theme.kind,
        }
    }

    pub fn fonts(&mut self) -> &mut Fonts {
        &mut self.fonts
    }

    pub fn measure(&mut self, text: &str, style: crate::text::TextStyle) -> Size {
        self.fonts.measure(text, style)
    }

    pub fn set_cursor(&mut self, cursor: Cursor) {
        self.cursor = cursor;
    }

    pub fn cursor(&self) -> Cursor {
        self.cursor
    }

    // Widgets after this are enabled or disabled, until it's called again.
    // Each frame starts enabled.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    // Widgets after this take keyboard focus or don't, until it's called
    // again. Each frame starts with them taking it.
    pub fn set_focusable(&mut self, focusable: bool) {
        self.focusable = focusable;
    }

    pub fn tooltips_enabled(&self) -> bool {
        self.tooltips_enabled
    }

    pub fn set_tooltips_enabled(&mut self, enabled: bool) {
        self.tooltips_enabled = enabled;
    }

    // Registers a target: a widget's area, which the pointer can be on.
    pub fn register(&mut self, name: &str, rect: Rect, sense: Sense, kind: &'static str) -> Id {
        let id = id(name);
        let clip = self.draw.current_clip();
        self.targets.push(Target {
            id,
            name: name.to_string(),
            label: None,
            rect: rect.intersect(clip),
            layer: self.painter_layer(),
            enabled: self.enabled,
            focusable: sense.focusable && self.focusable,
            keys: sense.keys,
            kind,
            listed: true,
        });
        id
    }

    // Covers what's under the area, like a card covers the view under it,
    // without being a widget that scripts point at.
    pub fn cover(&mut self, rect: Rect) {
        let clip = self.draw.current_clip();
        let id = id(&format!("cover:{}:{}:{}:{}", rect.x, rect.y, rect.width, rect.height));
        self.targets.push(Target {
            id,
            name: String::new(),
            label: None,
            rect: rect.intersect(clip),
            layer: self.painter_layer(),
            enabled: true,
            focusable: false,
            keys: Keys::None,
            kind: "Surface",
            listed: false,
        });
    }

    // Gives the target that this frame registered with the name a label,
    // like the text that its row shows, for scripts and tests.
    pub fn set_label(&mut self, name: &str, label: &str) {
        let id = id(name);
        if let Some(target) = self.targets.iter_mut().rev().find(|target| target.id == id) {
            target.label = Some(label.to_string());
        }
    }

    fn painter_layer(&self) -> Layer {
        self.draw.current_layer()
    }

    // Registers a widget and says what the input did to it.
    pub fn interact(&mut self, name: &str, rect: Rect, sense: Sense, kind: &'static str) -> Response {
        let id = self.register(name, rect, sense, kind);
        let enabled = self.enabled;
        let mut response = Response {
            id,
            rect,
            hovered: enabled && self.hovered == Some(id),
            focused: self.focused == Some(id),
            ..Default::default()
        };
        if !enabled {
            if self.active == Some(id) {
                self.active = None;
            }
            return response;
        }
        if response.hovered && sense.click && self.input.button_pressed(Button::Left) && !self.press_consumed {
            self.active = Some(id);
            self.press_consumed = true;
            response.pressed = true;
            response.double_clicked = self.input.clicks(Button::Left) >= 2;
            // A widget that doesn't take focus leaves it where it was, like
            // a text field's own clear button leaves it in the field.
            if sense.focusable && self.focusable {
                self.focused = Some(id);
                self.focus_visible = false;
            }
            response.focused = self.focused == Some(id);
        }
        if response.hovered && self.input.button_pressed(Button::Right) {
            response.secondary_pressed = true;
        }
        response.held = self.active == Some(id);
        if response.held && (self.input.button_released(Button::Left) || self.input.focus_lost) {
            response.released = true;
            response.clicked = !self.input.focus_lost
                && self
                    .input
                    .pointer
                    .is_some_and(|pointer| rect.intersect(self.draw.current_clip()).contains(pointer));
        }
        if response.focused
            && sense.keys == Keys::Activate
            && (self.input.key_pressed_once(Key::SPACE) || self.input.key_pressed_once(Key::ENTER))
        {
            response.clicked = true;
        }
        response
    }

    // The widget with the name has keyboard focus.
    pub fn is_focused(&self, name: &str) -> bool {
        self.focused == Some(id(name))
    }

    pub fn focus(&mut self, name: &str) {
        self.focused = Some(id(name));
    }

    pub fn clear_focus(&mut self) {
        self.focused = None;
    }

    // Whether the ring that shows keyboard focus is shown, which it is after
    // the keyboard moved focus, and not after a press did.
    pub fn focus_visible(&self) -> bool {
        self.focus_visible
    }

    pub(crate) fn set_focus_visible(&mut self, visible: bool) {
        self.focus_visible = visible;
    }

    pub fn hovered_id(&self) -> Option<Id> {
        self.hovered
    }

    // Whether the pointer is in the area, on this frame's input, whatever
    // covers it.
    pub fn pointer_in(&self, rect: Rect) -> bool {
        self.input.pointer.is_some_and(|pointer| rect.contains(pointer))
    }

    pub fn pointer(&self) -> Option<Point> {
        self.input.pointer
    }

    // Opens the popup that the widget with the id owns, like a combo box's
    // list, and closes any other.
    pub fn open_popup(&mut self, owner: Id, name: &str) {
        self.popup = Some(owner);
        self.popup_names.insert(owner, name.to_string());
    }

    pub fn close_popup(&mut self) {
        self.popup = None;
    }

    pub fn popup_open(&self, owner: Id) -> bool {
        self.popup == Some(owner)
    }

    pub fn any_popup(&self) -> bool {
        self.popup.is_some()
    }

    // Whether the tooltip of the widget with the id should show now.
    pub(crate) fn tooltip_due(&self, id: Id) -> Option<Point> {
        let due = self.tooltips_enabled
            && self.active.is_none()
            && self.popup.is_none()
            && self.hovered == Some(id)
            && self.tooltip.candidate == Some(id)
            && !self.tooltip.dismissed
            && self.input.time - self.tooltip.since >= TOOLTIP_DELAY;
        due.then_some(self.tooltip.anchor)
    }

    // A value that moves to `target` over `duration` seconds along the
    // curve, whenever the target changes. Starts at its first target.
    pub fn animate(&mut self, id: Id, slot: u8, target: f32, duration: f32, curve: Curve) -> f32 {
        self.animate4(id, slot, [target, 0.0, 0.0, 0.0], duration, curve)[0]
    }

    pub fn animate_color(&mut self, id: Id, slot: u8, target: Color, duration: f32) -> Color {
        let [r, g, b, a] = self.animate4(id, slot, [target.r, target.g, target.b, target.a], duration, CURVE_LINEAR);
        Color::rgba(r, g, b, a)
    }

    fn animate4(&mut self, id: Id, slot: u8, target: [f32; 4], duration: f32, curve: Curve) -> [f32; 4] {
        let now = self.input.time;
        let animation = self.animations.entry((id, slot)).or_insert(Animation {
            from: target,
            to: target,
            start: now,
            duration,
            curve,
        });
        if animation.to != target {
            animation.from = animation.value(now);
            animation.to = target;
            animation.start = now;
            animation.duration = duration;
            animation.curve = curve;
        }
        animation.value(now)
    }

    // State that a widget keeps from frame to frame, like a text field's
    // caret, by the widget's id.
    pub fn memory<T: Default + 'static>(&mut self, id: Id) -> &mut T {
        self.memory
            .entry((id, TypeId::of::<T>()))
            .or_insert_with(|| Box::new(T::default()))
            .downcast_mut::<T>()
            .unwrap()
    }

    pub(crate) fn clipboard(&mut self) -> &mut Clipboard {
        &mut self.clipboard
    }

    // The targets of the last frame that's done.
    pub fn targets(&self) -> &[Target] {
        &self.targets
    }

    pub fn target(&self, name: &str) -> Option<&Target> {
        let id = id(name);
        self.targets.iter().rev().find(|target| target.id == id && target.listed)
    }

    fn name_of(&self, id: Option<Id>) -> json::Value {
        match id.and_then(|id| self.targets.iter().chain(&self.previous).find(|target| target.id == id)) {
            Some(target) if target.listed => json::Value::String(target.name.clone()),
            _ => json::Value::Null,
        }
    }

    // What scripts save and check of the interface, as JSON: what the input
    // is on, and where each widget is.
    pub fn state(&self) -> json::Value {
        let area = |rect: Rect| json::Value::from(vec![rect.x as f64, rect.y as f64, rect.width as f64, rect.height as f64]);
        let mut targets = Vec::new();
        for target in self.targets.iter().filter(|target| target.listed) {
            if targets.iter().any(|(name, _): &(String, json::Value)| *name == target.name) {
                continue;
            }
            let mut members = vec![
                ("Area".to_string(), area(target.rect)),
                ("Kind".to_string(), json::Value::from(target.kind)),
                ("Enabled".to_string(), json::Value::from(target.enabled)),
            ];
            if let Some(label) = &target.label {
                members.push(("Label".to_string(), json::Value::from(label.as_str())));
            }
            targets.push((target.name.clone(), json::Value::Object(members)));
        }
        let popup = match self.popup {
            Some(owner) => self
                .popup_names
                .get(&owner)
                .map(|name| json::Value::from(name.as_str()))
                .unwrap_or(json::Value::Null),
            None => json::Value::Null,
        };
        let tooltip = match &self.shown_tooltip {
            Some((name, rect)) => json::Value::Object(vec![
                ("Name".to_string(), json::Value::from(name.as_str())),
                ("Area".to_string(), area(*rect)),
            ]),
            None => json::Value::Null,
        };
        let pointer = match self.input.pointer {
            Some(point) => json::Value::from(vec![point.x as f64, point.y as f64]),
            None => json::Value::Null,
        };
        json::Value::Object(vec![
            ("Theme".to_string(), json::Value::from(self.theme.kind.name())),
            ("Size".to_string(), json::Value::from(vec![self.size.width as f64, self.size.height as f64])),
            ("Fullscreen".to_string(), json::Value::from(self.fullscreen)),
            ("WindowFocused".to_string(), json::Value::from(self.input.focused)),
            ("Pointer".to_string(), pointer),
            ("Hovered".to_string(), self.name_of(self.hovered)),
            ("Pressed".to_string(), self.name_of(self.active)),
            ("Focused".to_string(), self.name_of(self.focused)),
            ("Popup".to_string(), popup),
            ("Tooltip".to_string(), tooltip),
            ("Targets".to_string(), json::Value::Object(targets)),
        ])
    }
}
