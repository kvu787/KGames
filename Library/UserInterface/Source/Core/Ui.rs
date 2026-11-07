// The interface of a frame. A program builds it each frame, in its update: it
// lays out its cards in fixed pixels and adds a widget for each control,
// which draws it and says what the input did to it.
//
//     if ui.add("Save", rect, Button::new("Save").accent()).clicked { ... }
//
// Widgets are named, like "Volume" or "Devices/3". A name is what a widget is
// known by from one frame to the next, for its state, like whether it has
// focus, and it's what scripts and tests point at. Every widget that a frame
// shows is a target with its area. The pointer is on the topmost target under
// it, by the last frame's targets, so a popup over a card takes the pointer
// from the card's widgets, and a dialog takes it from everything under it.

use std::any::{Any, TypeId};
use std::collections::HashMap;

use crate::application::program::Notice;
use crate::core::control::{Control, UiEvent};
use crate::core::tree::Tree;
use crate::design::theme::{Theme, ThemeKind};
use crate::design::tokens::{Curve, CURVE_LINEAR, TOOLTIP_DELAY};
use crate::drawing::geometry::{Color, Point, Rect, Size};
use crate::drawing::painter::{DrawList, Layer, Painter};
use crate::drawing::text::{Fonts, TextStyle};
use crate::input::devices::{Gamepad, Key, Modifiers, MouseButton, Shortcut};
use crate::input::events::{Event, Input};
use crate::platform::clipboard::Clipboard;

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

// The keys that a widget takes while it has keyboard focus, which the
// program's shortcuts then don't see.
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
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Sense {
    // Presses of each mouse button on it.
    pub left: bool,
    pub right: bool,
    pub middle: bool,
    // Whether it takes keyboard focus when pressed, and with Tab.
    pub focusable: bool,
    pub keys: Keys,
}

impl Sense {
    // Covers what's under it, and nothing else, like a card.
    pub const NONE: Sense = Sense::new(false, false, false, false, Keys::None);
    // Left presses, without taking focus.
    pub const CLICK: Sense = Sense::new(true, false, false, false, Keys::None);
    // Like a button: left presses, focus, and Space and Enter.
    pub const BUTTON: Sense = Sense::new(true, false, false, true, Keys::Activate);
    // Like a slider or a list: focus, and the keys that move.
    pub const NAVIGATE: Sense = Sense::new(true, false, false, true, Keys::Navigate);
    pub const TEXT: Sense = Sense::new(true, false, false, true, Keys::Text);
    // Every button, without focus, like a 3D view's.
    pub const VIEW: Sense = Sense::new(true, true, true, false, Keys::None);

    pub const fn new(left: bool, right: bool, middle: bool, focusable: bool, keys: Keys) -> Self {
        Self { left, right, middle, focusable, keys }
    }

    fn takes(&self, button: MouseButton) -> bool {
        match button {
            MouseButton::Left => self.left,
            MouseButton::Right => self.right,
            MouseButton::Middle => self.middle,
        }
    }
}

// A widget's area, which the pointer can be on.
#[derive(Clone, Debug)]
pub struct Target {
    pub id: Id,
    pub name: String,
    // What it shows, like a row's text, for scripts and tests.
    pub label: Option<String>,
    // Its area, inside the clip it was in.
    pub rect: Rect,
    pub layer: Layer,
    pub enabled: bool,
    pub focusable: bool,
    pub keys: Keys,
    // Like "Button" or "Slider", as WinUI names its controls.
    pub kind: &'static str,
    // Whether scripts and tests see it. Surfaces that only cover what's
    // under them, like cards, aren't listed.
    pub listed: bool,
}

// WinUI's visual states of a control, which pick its brushes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ControlState {
    #[default]
    Normal,
    PointerOver,
    Pressed,
    Disabled,
}

// What the input did to a widget this frame.
#[derive(Clone, Copy, Debug, Default)]
pub struct Response {
    pub id: Id,
    pub rect: Rect,
    pub enabled: bool,
    // The pointer is on it, and it's enabled.
    pub hovered: bool,
    // The left button went down on it this frame.
    pub pressed: bool,
    // It has a press: a button that went down on it is still down.
    pub held: bool,
    // Its left press ended this frame, wherever the pointer was.
    pub released: bool,
    // Its left press ended on it, or Space or Enter pressed it.
    pub clicked: bool,
    // Its press was the second of a double-click.
    pub double_clicked: bool,
    // The right button went down on it, or came up on it after going down
    // on it, which is when WinUI asks for a context menu.
    pub secondary_pressed: bool,
    pub secondary_clicked: bool,
    // The middle button went down on it.
    pub middle_pressed: bool,
    // The button that its press is of, while it's held.
    pub held_button: Option<MouseButton>,
    pub focused: bool,
    // Its value changed, for widgets that have one.
    pub changed: bool,
    // The pointer, while it's on the widget or the widget is held.
    pub pointer: Option<Point>,
    // How far the pointer moved this frame while the widget was held.
    pub drag_delta: Point,
    // Notches of the wheel turned over it.
    pub wheel: Point,
}

impl Response {
    pub fn state(&self) -> ControlState {
        if !self.enabled {
            ControlState::Disabled
        } else if self.held && self.held_button == Some(MouseButton::Left) && self.hovered {
            ControlState::Pressed
        } else if self.hovered || (self.held && self.held_button == Some(MouseButton::Left)) {
            ControlState::PointerOver
        } else {
            ControlState::Normal
        }
    }
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
    ResizeDiagonal,
    ResizeAntidiagonal,
    Crosshair,
    NotAllowed,
}

// What the program runs on, for its bars and its title.
#[derive(Clone, Debug, Default)]
pub struct Environment {
    // Like "NVIDIA GeForce RTX 5090 Laptop GPU".
    pub gpu: String,
    // The display's identifier, like "CVVNY2", and its model.
    pub display: String,
    pub display_name: String,
    // Whether frames may tear, and whether they wait for the screen, which
    // F10 switches.
    pub tearing: bool,
    pub vertical_sync: bool,
    // Whether input comes from a script instead of the user.
    pub scripted: bool,
}

// What a program asks of its window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Request {
    ToggleFullscreen,
    ToggleVerticalSync,
    Close,
}

// The frame rate, as the PerfBar along the top of the window measures it:
// over the last second, refreshed 10 times a second.
#[derive(Clone, Copy, Debug, Default)]
pub struct FrameStats {
    pub frames_per_second: f32,
    // Milliseconds.
    pub frame_time: f32,
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
        let t = if self.duration <= 0.0 { 1.0 } else { ((now - self.start) as f32 / self.duration).clamp(0.0, 1.0) };
        let eased = self.curve.ease(t);
        std::array::from_fn(|index| self.from[index] + (self.to[index] - self.from[index]) * eased)
    }

    fn done(&self, now: f64) -> bool {
        (now - self.start) as f32 >= self.duration
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct TooltipState {
    candidate: Option<Id>,
    since: f64,
    anchor: Point,
    dismissed: bool,
}

// An open popup, like a combo box's list or a menu, by the widget that owns
// it. Popups stack: a submenu opens over its menu.
#[derive(Clone, Debug)]
struct Popup {
    owner: Id,
    name: String,
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
    // The widget with a press, and the button.
    active: Option<(Id, MouseButton)>,
    focused: Option<Id>,
    focus_visible: bool,
    // The keys that the widget with focus takes.
    captured: Keys,
    popups: Vec<Popup>,
    // Whether the last frame showed a dialog, which then takes all input.
    modal: bool,
    modal_now: bool,
    // Presses that only closed a popup, which no widget may take.
    press_consumed: [bool; 3],
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
    pub(crate) environment: Environment,
    pub(crate) requests: Vec<Request>,
    // The page's: notices under the app bar, and where the page is.
    pub(crate) notices: Vec<Notice>,
    pub(crate) page_top: f32,
    pub(crate) page_bottom: f32,
    // What the PerfBar along the top of the window shows, which every
    // program's window has, and overlays don't.
    pub(crate) performance: Option<crate::application::performance::Performance>,
    // The retained tree that Program::build makes, and the frame's events
    // from its controls.
    pub(crate) tree: Tree,
    pub(crate) events: Vec<UiEvent>,
    pub(crate) rebuild_requested: bool,
    // The old nodes' controls during a rebuild, for adoption by name.
    pub(crate) previous_nodes: HashMap<Id, Option<Box<dyn Control>>>,
    // What's attached to nodes: tooltips with their widths, and context
    // menus, by node id.
    pub(crate) tooltips: HashMap<Id, (Vec<crate::widgets::tool_tip::Tip>, f32)>,
    pub(crate) context_menus: HashMap<Id, (String, Vec<crate::widgets::menu::MenuItem>)>,
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
            popups: Vec::new(),
            modal: false,
            modal_now: false,
            press_consumed: [false; 3],
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
            environment: Environment::default(),
            requests: Vec::new(),
            notices: Vec::new(),
            page_top: 0.0,
            page_bottom: 0.0,
            performance: None,
            tree: Tree::default(),
            events: Vec::new(),
            rebuild_requested: false,
            previous_nodes: HashMap::new(),
            tooltips: HashMap::new(),
            context_menus: HashMap::new(),
        }
    }

    // Starts a frame of the size, with the events since the last one.
    pub(crate) fn begin_frame(&mut self, events: &[Event], time: f64, size: Size, fullscreen: bool) {
        self.frame += 1;
        // A new client area, like F11's, re-lays the tree out.
        if size != self.size {
            self.rebuild_requested = true;
        }
        self.size = size;
        self.fullscreen = fullscreen;
        self.events.clear();
        self.input.begin(time, events);
        self.fonts.begin_frame();
        self.draw.clear(size);
        std::mem::swap(&mut self.targets, &mut self.previous);
        self.targets.clear();
        self.enabled = true;
        self.focusable = true;
        self.cursor = Cursor::Arrow;
        self.press_consumed = [false; 3];
        self.escape_consumed = false;
        self.shown_tooltip = None;
        self.page_top = self.content_top();
        self.page_bottom = size.height;
        self.modal = std::mem::replace(&mut self.modal_now, false);
        if let Some(performance) = &mut self.performance {
            performance.frame_started(time);
            performance.update(time);
        }
        // Now and then, the animations of widgets that are gone go. A done
        // animation of a widget that's shown stays, so that its next change
        // starts from where it is. On a frame of its own, apart from the
        // fonts' purge of old layouts, so that the two don't make one long
        // frame.
        if self.frame % 600 == 300 {
            let shown: std::collections::HashSet<Id> = self.previous.iter().map(|target| target.id).collect();
            self.animations.retain(|(id, _), animation| !animation.done(time) || shown.contains(id));
        }

        if self.input.focus_lost {
            self.popups.clear();
        }

        // The pointer is on the topmost target under it, or only on the
        // widget with a press while there is one. A dialog leaves nothing
        // under it to be on.
        let pointer = self.input.pointer;
        let lowest = if self.modal { Layer::Dialog } else { Layer::Base };
        let topmost = pointer.and_then(|point| {
            self.previous
                .iter()
                .filter(|target| target.layer >= lowest && target.rect.contains(point))
                .fold(None::<&Target>, |best, target| match best {
                    Some(best) if best.layer > target.layer => Some(best),
                    _ => Some(target),
                })
                .map(|target| target.id)
        });
        self.hovered = match self.active {
            Some((active, _)) => (topmost == Some(active)).then_some(active),
            None => topmost,
        };

        // A press outside the open popups closes them, and does nothing else.
        if !self.popups.is_empty() {
            for button in MouseButton::ALL {
                if !self.input.button_pressed(button) {
                    continue;
                }
                let inside = topmost.is_some_and(|hit| {
                    self.popups.iter().any(|popup| popup.owner == hit)
                        || self.previous.iter().any(|target| target.id == hit && target.layer == Layer::Popup)
                });
                if !inside {
                    self.popups.clear();
                    self.press_consumed[button.index()] = true;
                }
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
        if MouseButton::ALL.iter().any(|button| self.input.button_pressed(*button)) || self.input.wheel != Point::ZERO {
            self.tooltip.dismissed = true;
        }

        self.captured = match self.focused {
            Some(focused) => self.previous.iter().find(|target| target.id == focused).map(|target| target.keys).unwrap_or_default(),
            None => Keys::None,
        };
        if !self.popups.is_empty() && self.captured != Keys::Text {
            self.captured = Keys::Navigate;
        }

        // Esc closes the top popup, or else takes keyboard focus away. A
        // dialog hears it with dialog_escape.
        if self.input.key_pressed(Key::ESCAPE) {
            if self.popups.pop().is_some() {
                self.escape_consumed = true;
            } else if self.focused.is_some() && !self.modal {
                self.focused = None;
                self.escape_consumed = true;
            }
        }
    }

    pub(crate) fn end_frame(&mut self) {
        // Tab and Shift+Tab move keyboard focus through the widgets that take
        // it, in the order the frame showed them, in a dialog if one is open.
        if self.input.key_pressed(Key::TAB) && self.popups.is_empty() && !self.input.modifiers().control {
            let layer = if self.modal_now { Layer::Dialog } else { Layer::Base };
            let order: Vec<Id> = self
                .targets
                .iter()
                .filter(|target| target.focusable && target.enabled && target.layer == layer)
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
        // A left press that no widget took takes focus away.
        if self.input.button_pressed(MouseButton::Left) && !self.press_consumed[0] && self.active.is_none() {
            self.focused = None;
        }
        // A widget that's gone or disabled can't keep focus, a press, or a
        // popup.
        if let Some(focused) = self.focused
            && !self.targets.iter().any(|target| target.id == focused && target.enabled)
        {
            self.focused = None;
        }
        if let Some((active, button)) = self.active
            && (!self.input.button_down(button) || !self.targets.iter().any(|target| target.id == active))
        {
            self.active = None;
        }
        let shown: Vec<Id> = self.targets.iter().map(|target| target.id).collect();
        self.popups.retain(|popup| shown.contains(&popup.owner));
        if self.input.key_pressed_once(Key::F10) && !self.is_captured(Key::F10) {
            self.requests.push(Request::ToggleVerticalSync);
        }
        if self.input.key_pressed_once(Key::F11) && !self.is_captured(Key::F11) {
            self.requests.push(Request::ToggleFullscreen);
        }
        if self.performance.is_some() {
            self.perf_bar();
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

    // The part of the client area that the program has: all of it, under the
    // PerfBar along its top.
    pub fn bounds(&self) -> Rect {
        let top = self.content_top();
        Rect::new(0.0, top, self.size.width, (self.size.height - top).max(0.0))
    }

    // Where the program's part of the window starts: under the PerfBar,
    // PERF_BAR_HEIGHT pixels down, or at the top in an overlay.
    pub fn content_top(&self) -> f32 {
        if self.performance.is_some() { crate::design::tokens::PERF_BAR_HEIGHT } else { 0.0 }
    }

    pub fn fullscreen(&self) -> bool {
        self.fullscreen
    }

    pub fn toggle_fullscreen(&mut self) {
        self.requests.push(Request::ToggleFullscreen);
    }

    // Whether frames wait for the screen's refresh.
    pub fn vertical_sync(&self) -> bool {
        self.environment.vertical_sync
    }

    // Switches vertical sync, as F10 does, from the next frame on.
    pub fn toggle_vertical_sync(&mut self) {
        self.requests.push(Request::ToggleVerticalSync);
    }

    // Closes the window, whatever Program::may_close says.
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

    // The frame rate, as the PerfBar measures it, so that a program's own
    // card agrees with the PerfBar. Zero without it.
    pub fn stats(&self) -> FrameStats {
        match &self.performance {
            Some(performance) => {
                let summary = performance.summary();
                FrameStats {
                    frames_per_second: summary.frames_per_second as f32,
                    frame_time: summary.frame_time as f32,
                }
            }
            None => FrameStats::default(),
        }
    }

    pub fn environment(&self) -> &Environment {
        &self.environment
    }

    // All of the frame's input, for programs that read it themselves, like
    // the keys held for driving.
    pub fn input(&self) -> &Input {
        &self.input
    }

    // Whether the window has focus. Without it there's no input at all.
    pub fn window_focused(&self) -> bool {
        self.input.focused
    }

    pub fn gamepad(&self) -> &Gamepad {
        &self.input.gamepad
    }

    pub fn pointer(&self) -> Option<Point> {
        self.input.pointer
    }

    // Whether the key was pressed this frame, counting a held key's repeats,
    // unless the widget with keyboard focus takes it, as a text field takes
    // letters, or a dialog is open over the page.
    pub fn key_pressed(&self, key: Key) -> bool {
        self.input.key_pressed(key) && !self.is_captured(key) && !self.modal
    }

    // Like key_pressed, without repeats.
    pub fn key_pressed_once(&self, key: Key) -> bool {
        self.input.key_pressed_once(key) && !self.is_captured(key) && !self.modal
    }

    // Whether the key was pressed with exactly these modifiers, like Ctrl+S.
    pub fn shortcut(&self, modifiers: Modifiers, key: Key) -> bool {
        self.input.modifiers() == modifiers && self.key_pressed(key)
    }

    pub fn shortcut_pressed(&self, shortcut: Shortcut) -> bool {
        self.shortcut(shortcut.modifiers, shortcut.key)
    }

    // Whether the key is held, unless a text field with focus takes it.
    pub fn key_down(&self, key: Key) -> bool {
        self.input.key_down(key) && !(self.captured == Keys::Text && !key.is_modifier()) && !self.modal
    }

    pub(crate) fn is_captured(&self, key: Key) -> bool {
        if key == Key::ESCAPE && self.escape_consumed {
            return true;
        }
        match self.captured {
            Keys::None => false,
            Keys::Activate => key == Key::SPACE || key == Key::ENTER,
            Keys::Navigate => key.is_navigation() || key == Key::SPACE || key == Key::ENTER,
            Keys::Text => {
                let control = self.input.modifiers().control;
                let editing = matches!(
                    key,
                    Key::A | Key::C | Key::X | Key::V | Key::Z | Key::Y | Key::LEFT | Key::RIGHT | Key::BACKSPACE | Key::DELETE | Key::HOME | Key::END
                );
                !key.is_function() && key != Key::ESCAPE && key != Key::TAB && (!control || editing)
            }
        }
    }

    pub fn painter(&mut self) -> Painter<'_> {
        Painter {
            list: &mut self.draw,
            fonts: &mut self.fonts,
            theme: &self.theme,
        }
    }

    pub fn fonts(&mut self) -> &mut Fonts {
        &mut self.fonts
    }

    pub fn measure(&mut self, text: &str, style: TextStyle) -> Size {
        self.fonts.measure(text, style)
    }

    pub fn set_cursor(&mut self, cursor: Cursor) {
        self.cursor = cursor;
    }

    pub fn cursor(&self) -> Cursor {
        self.cursor
    }

    // Targets registered after this are enabled or not, until it's called
    // again, for a control's internal parts. Each frame starts enabled.
    pub fn set_scope_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    // Targets registered after this take keyboard focus or don't, until it's
    // called again, for a control's internal parts. Each frame starts with
    // them taking it.
    pub fn set_scope_focusable(&mut self, focusable: bool) {
        self.focusable = focusable;
    }

    pub fn focusable(&self) -> bool {
        self.focusable
    }

    pub fn tooltips_enabled(&self) -> bool {
        self.tooltips_enabled
    }

    pub fn set_tooltips_enabled(&mut self, enabled: bool) {
        self.tooltips_enabled = enabled;
    }

    // The layer that widgets go on now: the base, a dialog, or a popup.
    pub fn layer(&self) -> Layer {
        self.draw.layer()
    }

    pub(crate) fn set_layer(&mut self, layer: Layer) {
        self.draw.set_layer(layer);
        if layer == Layer::Dialog {
            self.modal_now = true;
        }
    }

    // Registers a target: a widget's area, which the pointer can be on.
    pub fn register(&mut self, name: &str, rect: Rect, sense: Sense, kind: &'static str) -> Id {
        self.register_as(name, rect, sense, kind, self.enabled, true)
    }

    pub(crate) fn register_as(&mut self, name: &str, rect: Rect, sense: Sense, kind: &'static str, enabled: bool, listed: bool) -> Id {
        let id = id(name);
        self.targets.push(Target {
            id,
            name: name.to_string(),
            label: None,
            rect: rect.intersect(self.draw.clip()),
            layer: self.draw.layer(),
            enabled,
            focusable: sense.focusable && self.focusable,
            keys: sense.keys,
            kind,
            listed,
        });
        id
    }

    // How many instances the frame drew: each shape, glyph, line, arc,
    // polygon and shadow is one, which the renderer draws in one call. For
    // benchmarks.
    pub fn instance_count(&self) -> usize {
        self.draw.instance_count()
    }

    // Covers what's under the area, like a card covers the 3D view under
    // it, without being a widget that scripts see.
    pub fn cover(&mut self, rect: Rect) {
        let id = id(&format!("Cover:{}:{}:{}:{}", rect.x, rect.y, rect.width, rect.height));
        self.targets.push(Target {
            id,
            name: String::new(),
            label: None,
            rect: rect.intersect(self.draw.clip()),
            layer: self.draw.layer(),
            enabled: true,
            focusable: false,
            keys: Keys::None,
            kind: "Surface",
            listed: false,
        });
    }

    // Takes back this frame's targets of the kind whose names start with the
    // prefix, for a list whose rows changed after dispatch.
    pub(crate) fn remove_targets(&mut self, prefix: &str, kind: &'static str) {
        self.targets.retain(|target| !(target.kind == kind && target.name.starts_with(prefix)));
    }

    // Gives the target that this frame registered with the name another kind,
    // for a widget built of another, like a number box of a text box.
    pub(crate) fn retarget(&mut self, name: &str, kind: &'static str) {
        let id = id(name);
        if let Some(target) = self.targets.iter_mut().rev().find(|target| target.id == id) {
            target.kind = kind;
        }
    }

    // Gives the target that this frame registered with the name a label, like
    // the text its row shows, for scripts and tests.
    pub fn set_label(&mut self, name: &str, label: &str) {
        let id = id(name);
        if let Some(target) = self.targets.iter_mut().rev().find(|target| target.id == id) {
            target.label = Some(label.to_string());
        }
    }

    // Registers a widget and says what the input did to it.
    pub fn interact(&mut self, name: &str, rect: Rect, sense: Sense, kind: &'static str) -> Response {
        self.interact_as(name, rect, sense, kind, self.enabled, true)
    }

    // Like interact, saying whether the widget is enabled and whether scripts
    // see it, for the tree's nodes.
    pub(crate) fn interact_as(&mut self, name: &str, rect: Rect, sense: Sense, kind: &'static str, enabled: bool, listed: bool) -> Response {
        let id = self.register_as(name, rect, sense, kind, enabled, listed);
        let visible = rect.intersect(self.draw.clip());
        // While a popup or a dialog is open, the keys are theirs, as WinUI
        // moves focus into them: a widget under one has focus, but not for
        // keys.
        let layer = self.draw.layer();
        let keys_here = (self.popups.is_empty() || layer >= Layer::Popup) && (!self.modal || layer >= Layer::Dialog);
        let mut response = Response {
            id,
            rect,
            enabled,
            hovered: enabled && self.hovered == Some(id),
            focused: self.focused == Some(id) && keys_here,
            ..Default::default()
        };
        if !enabled {
            if self.active.is_some_and(|(active, _)| active == id) {
                self.active = None;
            }
            return response;
        }
        for button in MouseButton::ALL {
            if !(response.hovered && sense.takes(button) && self.input.button_pressed(button) && !self.press_consumed[button.index()]) {
                continue;
            }
            self.active = Some((id, button));
            self.press_consumed[button.index()] = true;
            match button {
                MouseButton::Left => {
                    response.pressed = true;
                    response.double_clicked = self.input.clicks(MouseButton::Left) >= 2;
                    // A widget that doesn't take focus leaves it where it was,
                    // like a text field's clear button leaves it in the field.
                    if sense.focusable && self.focusable {
                        self.focused = Some(id);
                        self.focus_visible = false;
                    }
                    response.focused = self.focused == Some(id) && keys_here;
                }
                MouseButton::Right => response.secondary_pressed = true,
                MouseButton::Middle => response.middle_pressed = true,
            }
        }
        // A widget that doesn't take the right button still hears a right
        // press, for a context menu, unless the press closed the popups.
        if response.hovered && !sense.right && self.input.button_pressed(MouseButton::Right) && !self.press_consumed[MouseButton::Right.index()] {
            response.secondary_pressed = true;
        }
        if let Some((active, button)) = self.active
            && active == id
        {
            response.held = true;
            response.held_button = Some(button);
            response.drag_delta = self.input.pointer_delta;
            if self.input.button_released(button) || self.input.focus_lost {
                let on_it = !self.input.focus_lost && self.input.pointer.is_some_and(|pointer| visible.contains(pointer));
                match button {
                    MouseButton::Left => {
                        response.released = true;
                        response.clicked = on_it;
                    }
                    MouseButton::Right => response.secondary_clicked = on_it,
                    MouseButton::Middle => {}
                }
            }
        }
        if response.hovered || response.held {
            response.pointer = self.input.pointer;
        }
        if response.hovered {
            response.wheel = self.input.wheel;
        }
        if response.focused && sense.keys == Keys::Activate && (self.input.key_pressed_once(Key::SPACE) || self.input.key_pressed_once(Key::ENTER)) {
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

    // Whether the focus visual shows, which it does after the keyboard moved
    // focus, and not after a press did, as in Windows.
    pub fn focus_visible(&self) -> bool {
        self.focus_visible
    }

    pub(crate) fn set_focus_visible(&mut self, visible: bool) {
        self.focus_visible = visible;
    }

    pub fn hovered_id(&self) -> Option<Id> {
        self.hovered
    }

    // The kind of the widget under the pointer, like "Slider".
    pub(crate) fn hovered_kind(&self) -> Option<&'static str> {
        let hovered = self.hovered?;
        self.previous.iter().rev().find(|target| target.id == hovered).map(|target| target.kind)
    }

    // Whether the pointer is in the area, whatever covers it.
    pub fn pointer_in(&self, rect: Rect) -> bool {
        self.input.pointer.is_some_and(|pointer| rect.contains(pointer))
    }

    // Whether the wheel turned over the area this frame, for what scrolls:
    // the pointer is in it, nothing of a higher layer, like a menu or a
    // dialog, is under the pointer, and the widget under the pointer doesn't
    // take the wheel itself, like a slider.
    pub fn wheel_over(&self, rect: Rect) -> bool {
        self.pointer_in(rect)
            && self.input.wheel.y != 0.0
            && !self.hovered_takes_wheel()
            && self.hovered_layer().is_none_or(|layer| layer <= self.layer())
    }

    // Whether the widget under the pointer is the one with keyboard focus.
    pub(crate) fn hovered_has_focus(&self) -> bool {
        self.hovered.is_some() && self.hovered == self.focused
    }

    // Whether the widget under the pointer takes the wheel, so that what
    // it's in shouldn't scroll.
    pub(crate) fn hovered_takes_wheel(&self) -> bool {
        match self.hovered_kind() {
            Some("Slider" | "View" | "ComboBoxPopup" | "ComboBoxItem") => true,
            // A number box takes the wheel only while it has focus, as
            // WinUI's does.
            Some("NumberBox") => self.hovered_has_focus(),
            _ => false,
        }
    }

    // The layer of the widget under the pointer.
    fn hovered_layer(&self) -> Option<Layer> {
        let hovered = self.hovered?;
        self.previous.iter().rev().find(|target| target.id == hovered).map(|target| target.layer)
    }

    // Whether a press of the button that nothing took happened this frame.
    pub fn press_unclaimed(&self, button: MouseButton) -> bool {
        self.input.button_pressed(button) && !self.press_consumed[button.index()]
    }

    // Opens the popup that the widget with the id owns, like a combo box's
    // list. A popup whose owner is in an open popup opens over it, like a
    // submenu; any other closes the open ones first.
    pub fn open_popup(&mut self, owner: Id, name: &str) {
        let in_popup = self.targets.iter().chain(&self.previous).any(|target| target.id == owner && target.layer == Layer::Popup);
        if !in_popup {
            self.popups.clear();
        } else if let Some(index) = self.popups.iter().position(|popup| popup.owner == owner) {
            self.popups.truncate(index);
        }
        self.popups.push(Popup {
            owner,
            name: name.to_string(),
        });
    }

    // Closes the popup that the widget owns, and those over it.
    pub fn close_popup(&mut self, owner: Id) {
        if let Some(index) = self.popups.iter().position(|popup| popup.owner == owner) {
            self.popups.truncate(index);
        }
    }

    pub fn close_popups(&mut self) {
        self.popups.clear();
    }

    pub fn popup_open(&self, owner: Id) -> bool {
        self.popups.iter().any(|popup| popup.owner == owner)
    }

    // Whether the widget's popup is the top one, which takes the keys.
    pub fn popup_on_top(&self, owner: Id) -> bool {
        self.popups.last().is_some_and(|popup| popup.owner == owner)
    }

    pub fn any_popup(&self) -> bool {
        !self.popups.is_empty()
    }

    // Whether a dialog is open, which takes the input from the page.
    pub fn modal(&self) -> bool {
        self.modal
    }

    // Esc for a dialog, which closes it like its close button.
    pub(crate) fn dialog_escape(&mut self) -> bool {
        if self.input.key_pressed(Key::ESCAPE) && !self.escape_consumed && self.popups.is_empty() {
            self.escape_consumed = true;
            return true;
        }
        false
    }

    // Where the pointer rested on the widget, if its tooltip should show now.
    pub(crate) fn tooltip_due(&self, id: Id) -> Option<Point> {
        let due = self.tooltips_enabled
            && self.active.is_none()
            && self.popups.is_empty()
            && self.hovered == Some(id)
            && self.tooltip.candidate == Some(id)
            && !self.tooltip.dismissed
            && self.input.time - self.tooltip.since >= TOOLTIP_DELAY;
        due.then_some(self.tooltip.anchor)
    }

    // A value that moves to `target` over `duration` seconds along the curve
    // whenever the target changes. It starts at its first target.
    pub fn animate(&mut self, id: Id, slot: u8, target: f32, duration: f32, curve: Curve) -> f32 {
        self.animate_values(id, slot, [target, 0.0, 0.0, 0.0], duration, curve)[0]
    }

    pub fn animate_color(&mut self, id: Id, slot: u8, target: Color, duration: f32) -> Color {
        let [r, g, b, a] = self.animate_values(id, slot, [target.r, target.g, target.b, target.a], duration, CURVE_LINEAR);
        Color::rgba(r, g, b, a)
    }

    pub fn animate_rect(&mut self, id: Id, slot: u8, target: Rect, duration: f32, curve: Curve) -> Rect {
        let [x, y, width, height] = self.animate_values(id, slot, [target.x, target.y, target.width, target.height], duration, curve);
        Rect::new(x, y, width, height)
    }

    fn animate_values(&mut self, id: Id, slot: u8, target: [f32; 4], duration: f32, curve: Curve) -> [f32; 4] {
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
            .expect("memory is keyed by its type")
    }

    pub(crate) fn clipboard(&mut self) -> &mut Clipboard {
        &mut self.clipboard
    }

    // The notices under the app bar, which a program without one can show
    // itself.
    pub fn notices(&self) -> &[Notice] {
        &self.notices
    }

    pub fn notify(&mut self, notice: Notice) {
        self.notices.push(notice);
        self.rebuild_requested = true;
    }

    pub(crate) fn remove_notice(&mut self, index: usize) {
        if index < self.notices.len() {
            self.notices.remove(index);
        }
    }

    // The targets of the last frame that's done.
    pub fn targets(&self) -> &[Target] {
        &self.targets
    }

    // The last frame's listed target with the name.
    pub fn target(&self, name: &str) -> Option<&Target> {
        let id = id(name);
        self.targets.iter().rev().find(|target| target.id == id && target.listed)
    }

    fn listed_name(&self, id: Option<Id>) -> Option<&str> {
        let id = id?;
        self.targets.iter().chain(&self.previous).find(|target| target.id == id && target.listed).map(|target| target.name.as_str())
    }

    // The names of the widgets that the input is on, as the interface's
    // state has them.
    pub fn hovered_name(&self) -> Option<&str> {
        self.listed_name(self.hovered)
    }

    pub fn pressed_name(&self) -> Option<&str> {
        self.listed_name(self.active.map(|(id, _)| id))
    }

    pub fn focused_name(&self) -> Option<&str> {
        self.listed_name(self.focused)
    }

    pub fn popup_name(&self) -> Option<&str> {
        self.popups.last().map(|popup| popup.name.as_str())
    }

    pub fn tooltip_name(&self) -> Option<&str> {
        self.shown_tooltip.as_ref().map(|(name, _)| name.as_str())
    }

    // What scripts save and check of the interface: what the input is on,
    // and where each widget is.
    pub fn state(&self) -> json::Value {
        use json::Value;
        let area = |rect: Rect| Value::from(vec![rect.x as f64, rect.y as f64, rect.width as f64, rect.height as f64]);
        let name = |name: Option<&str>| name.map_or(Value::Null, Value::from);
        let mut targets: Vec<(String, Value)> = Vec::new();
        for target in self.targets.iter().filter(|target| target.listed) {
            if targets.iter().any(|(name, _)| *name == target.name) {
                continue;
            }
            let mut members = vec![
                ("Area".to_string(), area(target.rect)),
                ("Kind".to_string(), Value::from(target.kind)),
                ("Enabled".to_string(), Value::from(target.enabled)),
            ];
            if let Some(label) = &target.label {
                members.push(("Label".to_string(), Value::from(label.as_str())));
            }
            targets.push((target.name.clone(), Value::Object(members)));
        }
        let tooltip = match &self.shown_tooltip {
            Some((name, rect)) => Value::Object(vec![("Name".to_string(), Value::from(name.as_str())), ("Area".to_string(), area(*rect))]),
            None => Value::Null,
        };
        let pointer = self.input.pointer.map_or(Value::Null, |point| Value::from(vec![point.x as f64, point.y as f64]));
        Value::Object(vec![
            ("Theme".to_string(), Value::from(self.theme.kind.name())),
            ("Size".to_string(), Value::from(vec![self.size.width as f64, self.size.height as f64])),
            ("Fullscreen".to_string(), Value::from(self.fullscreen)),
            ("VerticalSync".to_string(), Value::from(self.environment.vertical_sync)),
            ("WindowFocused".to_string(), Value::from(self.input.focused)),
            ("Pointer".to_string(), pointer),
            ("Hovered".to_string(), name(self.hovered_name())),
            ("Pressed".to_string(), name(self.pressed_name())),
            ("Focused".to_string(), name(self.focused_name())),
            ("Popup".to_string(), name(self.popup_name())),
            ("Dialog".to_string(), Value::from(self.modal_now)),
            ("Tooltip".to_string(), tooltip),
            ("Targets".to_string(), Value::Object(targets)),
        ])
    }
}
