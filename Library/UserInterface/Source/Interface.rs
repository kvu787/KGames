// The interface's state from frame to frame, and each frame of it.
//
// The interface is immediate mode: each frame, the program calls a function
// for each widget, with where it goes and the value it shows, like
// `frame.button("Save", area, "Save")`, and the function draws it and says
// what the input did to it. Nothing about a widget is kept but what the
// program keeps, and a little that only the interface needs, like where a
// list is scrolled to.
//
// A widget is known by its name, which is unique among the frame's widgets,
// and the same from frame to frame. Each frame, a widget leaves its area, its
// name and what input it takes, which are the frame's targets. Input goes to
// the targets of the frame before it, which is what was on the screen when it
// came: clicks hit what the frame before them drew. A press goes to the
// topmost target under it, which holds the mouse until the release, so that
// it gets the drag even outside its area. A turn of the wheel goes to the
// topmost target under it that scrolls. Keys go to the widget with keyboard
// focus, if it takes them, and to the program otherwise, for its shortcuts.
//
// Widgets draw on three layers, each over the ones before: the base, popups
// like a select box's list, and tooltips. A popup's targets are over all of
// the base's. While a popup is open, a press anywhere but on it or the widget
// that opened it closes it, and does nothing else.
//
// Tab and Shift+Tab move keyboard focus to the next or previous widget that
// takes it, and Esc closes a popup, or else takes focus away. Without the
// window's focus, a frame has no cursor and no events but the release of a
// press, and popups close.

use std::collections::HashMap;

use application::Event;
use canvas::{Canvas, Font, Rect, Vertex};
use json::Value;
use windows_api::*;

// A widget's identity, from its name.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Id(u64);

impl Id {
    // FNV-1a, which is quick for short names.
    pub fn new(name: &str) -> Self {
        let mut hash: u64 = 0xCBF2_9CE4_8422_2325;
        for byte in name.bytes() {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(0x0000_0100_0000_01B3);
        }
        Id(hash)
    }
}

// What a widget is drawn on. Each layer is over the ones before it, and its
// targets are over theirs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Layer {
    Base,
    Popup,
    Tooltip,
}

// What input a target takes.
#[derive(Clone, Copy, Debug)]
pub struct Sense {
    // Presses, and the drag and release that follow.
    pub click: bool,
    // Turns of the wheel.
    pub wheel: bool,
    // Keyboard focus, from a press or Tab.
    pub focus: bool,
    // The keys it takes while it has focus, as virtual-key codes, without
    // Control.
    pub keys: &'static [i32],
    // Typed characters while it has focus, and every key without Control
    // but Tab, Esc and the function keys, so that typing doesn't set off
    // the program's shortcuts.
    pub text: bool,
}

impl Sense {
    // Nothing but the cursor being over it, for tooltips, and to keep the
    // targets under it from the cursor.
    pub const HOVER: Sense = Sense {
        click: false,
        wheel: false,
        focus: false,
        keys: &[],
        text: false,
    };
    // Presses, keyboard focus, and Space and Enter, which press it too.
    pub const BUTTON: Sense = Sense {
        click: true,
        focus: true,
        keys: &[VK_SPACE, VK_RETURN],
        ..Sense::HOVER
    };
}

// A widget's area in a frame, and what input it takes there.
struct Target {
    id: Id,
    name: String,
    // Like "Button", for scripts.
    kind: &'static str,
    // What the widget shows, if its name doesn't say, like a select box's
    // choice, for scripts.
    label: String,
    area: Rect,
    layer: Layer,
    sense: Sense,
    // The widget that a part of another one belongs to, like a select box
    // for each choice in its list, which keeps focus while the part is
    // pressed.
    owner: Option<Id>,
}

// Where an event goes.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Route {
    Widget(Id),
    // No widget takes it, so it's for the program's shortcuts.
    Program,
    // The interface used it, like a Tab, or a press that closed a popup.
    Taken,
}

// What a text field keeps while it's edited.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Caret {
    // In characters from the start.
    pub position: usize,
    // The first character that the field shows.
    pub scroll: usize,
    // When the caret last moved, so that it shows steadily while it moves,
    // and blinks while it doesn't.
    pub moved_at: f64,
}

// What the interface keeps from frame to frame, besides what it draws.
#[derive(Default)]
pub(crate) struct State {
    // The last frame's targets, which this frame's input goes to, in the
    // order the widgets were drawn.
    targets: Vec<Target>,
    // This frame's, for the next frame's input.
    next_targets: Vec<Target>,
    pub hovered: Option<Id>,
    // The widget that has the press, until its release.
    pub held: Option<Id>,
    pub focus: Option<Id>,
    // Whether the focus moved by keyboard, so that it shows a ring.
    pub focus_visible: bool,
    // The widget whose popup is open.
    pub popup: Option<Id>,
    // This frame's events, with where each goes.
    events: Vec<(Route, Event)>,
    // The widget that the wheel last turned over, and how much it turned
    // there that didn't make a whole step yet. Touchpads and smooth-scrolling
    // mice turn by parts of a notch.
    pub wheel: (Option<Id>, f32),
    // Where each list is scrolled to, as its first row.
    pub scroll: HashMap<Id, usize>,
    // Where a scroll bar's thumb was grabbed, from its top.
    pub grab: f32,
    pub carets: HashMap<Id, Caret>,
    // The tooltip that the frame shows: its name, and its area.
    pub tooltip: Option<(String, Rect)>,
}

impl State {
    // The topmost of the last frame's targets at a point.
    fn top(&self, point: [f32; 2]) -> Option<&Target> {
        self.targets
            .iter()
            .enumerate()
            .filter(|(_, target)| target.area.contains(point))
            .max_by_key(|(index, target)| (target.layer, *index))
            .map(|(_, target)| target)
    }

    fn target(&self, id: Id) -> Option<&Target> {
        self.targets.iter().find(|target| target.id == id)
    }

    fn name(&self, id: Option<Id>) -> Value {
        id.and_then(|id| self.target(id)).map(|target| target.name.clone()).into()
    }

    // Whether a widget takes a key while it has focus.
    fn takes_key(&self, id: Id, key: i32, control: bool) -> bool {
        let Some(target) = self.target(id) else {
            return false;
        };
        if control {
            return false;
        }
        let function_key = (VK_F1..VK_F1 + 24).contains(&key);
        target.sense.keys.contains(&key) || (target.sense.text && !function_key)
    }

    // Starts a frame: works out what the cursor is over, and where each of
    // the frame's events goes.
    fn begin(&mut self, input: &application::Frame) {
        self.events.clear();
        self.next_targets.clear();
        self.tooltip = None;
        if !input.focused {
            self.popup = None;
        }
        self.hovered = input
            .cursor
            .and_then(|point| self.top(point))
            .map(|target| target.id);
        for event in &input.events {
            let route = self.route(event);
            self.events.push((route, *event));
        }
    }

    fn route(&mut self, event: &Event) -> Route {
        match *event {
            Event::Press(point) => {
                let target = self.top(point).map(|target| (target.id, target.sense, target.owner));
                if let Some(popup) = self.popup {
                    let on_popup = target.is_some_and(|(id, _, owner)| id == popup || owner == Some(popup));
                    if !on_popup {
                        self.popup = None;
                        self.held = None;
                        return Route::Taken;
                    }
                }
                self.focus_visible = false;
                match target {
                    Some((id, sense, owner)) if sense.click => {
                        self.held = Some(id);
                        // A part, like a select box's choice, gives focus
                        // to its widget, if that takes focus.
                        let owner = owner.filter(|owner| self.target(*owner).is_some_and(|target| target.sense.focus));
                        self.focus = if sense.focus { Some(id) } else { owner };
                        Route::Widget(id)
                    }
                    _ => {
                        self.held = None;
                        self.focus = None;
                        Route::Program
                    }
                }
            }
            Event::Release(_) => match self.held.take() {
                Some(id) => Route::Widget(id),
                None => Route::Program,
            },
            Event::Wheel(point, _) => {
                let layer = self.top(point).map(|target| target.layer);
                if self.popup.is_some() && layer != Some(Layer::Popup) {
                    return Route::Taken;
                }
                let target = self.targets.iter().rev().find(|target| {
                    Some(target.layer) == layer && target.sense.wheel && target.area.contains(point)
                });
                match target {
                    Some(target) => Route::Widget(target.id),
                    None => Route::Program,
                }
            }
            Event::Key { key, control, shift, .. } => {
                let key = key as i32;
                if key == VK_TAB && !control {
                    self.popup = None;
                    self.move_focus(shift);
                    return Route::Taken;
                }
                if key == VK_ESCAPE {
                    if self.popup.take().is_some() || self.focus.take().is_some() {
                        return Route::Taken;
                    }
                    return Route::Program;
                }
                let owner = self.popup.or(self.focus);
                match owner {
                    Some(id) if self.takes_key(id, key, control) => Route::Widget(id),
                    _ => Route::Program,
                }
            }
            Event::Character(_) => match self.focus {
                Some(id) if self.target(id).is_some_and(|target| target.sense.text) => Route::Widget(id),
                _ => Route::Program,
            },
        }
    }

    // Moves focus to the next widget that takes it, or the previous one,
    // among the base layer's, in the order they were drawn.
    fn move_focus(&mut self, backward: bool) {
        let order: Vec<Id> = self
            .targets
            .iter()
            .filter(|target| target.sense.focus && target.layer == Layer::Base)
            .map(|target| target.id)
            .collect();
        if order.is_empty() {
            self.focus = None;
            return;
        }
        let current = self.focus.and_then(|id| order.iter().position(|other| *other == id));
        let next = match (current, backward) {
            (None, false) => 0,
            (None, true) => order.len() - 1,
            (Some(index), false) => (index + 1) % order.len(),
            (Some(index), true) => (index + order.len() - 1) % order.len(),
        };
        self.focus = Some(order[next]);
        self.focus_visible = true;
    }

    // Ends a frame: what wasn't drawn can't be held, have focus, or have an
    // open popup, and this frame's targets are the next frame's.
    fn end(&mut self) {
        let drawn = |targets: &[Target], id: Option<Id>| {
            id.is_none_or(|id| targets.iter().any(|target| target.id == id))
        };
        if !drawn(&self.next_targets, self.popup) {
            self.popup = None;
        }
        if !drawn(&self.next_targets, self.focus) {
            self.focus = None;
        }
        if !drawn(&self.next_targets, self.held) {
            self.held = None;
        }
        std::mem::swap(&mut self.targets, &mut self.next_targets);
    }
}

// What the header, the footer and the program's own panels show about the
// program and its window, besides input.
#[derive(Clone, Copy)]
pub struct Status<'a> {
    pub frames_per_second: Option<f64>,
    // The GPU and how frames are presented, from Renderer::description.
    pub adapter: &'a str,
    // The display that the window is on, like "KXGR16 (PA278QGV)".
    pub display: &'a str,
    // Problems with how the program was started, one per paragraph.
    pub notices: &'a [String],
}

impl Status<'static> {
    // For tests, which draw frames without a window or a GPU.
    pub const HEADLESS: Status<'static> = Status {
        frames_per_second: None,
        adapter: "Test GPU",
        display: "TEST01 (Test display)",
        notices: &[],
    };
}

// What a frame's input did to a widget.
#[derive(Clone, Copy, Debug, Default)]
pub struct Response {
    pub area: Rect,
    // The cursor is over it, and over nothing on top of it, and no other
    // widget has the press.
    pub hovered: bool,
    // It got a press this frame. Buttons act on it, not on the release,
    // as it comes sooner.
    pub pressed: bool,
    // It has the press, until the release, like while a slider is dragged.
    pub held: bool,
    pub released: bool,
    pub focused: bool,
    // It was pressed, or Space or Enter pressed it while it had focus.
    pub clicked: bool,
    // Its value changed this frame.
    pub changed: bool,
    // Enter was pressed in it, for a text field.
    pub submitted: bool,
    // How far the wheel turned over it, in notches, positive away from the
    // user.
    pub wheel: f32,
}

// The interface of a program's window, from frame to frame.
#[derive(Default)]
pub struct Interface {
    state: State,
    // Each layer's vertices, reused from frame to frame, so that their
    // memory is too. The base layer's has every layer's once a frame ends.
    layers: [Vec<Vertex>; 3],
}

impl Interface {
    // Starts a frame of the interface, `size` pixels big, with the window's
    // input. The frame ends when it's dropped, and then `vertices` has what
    // it drew.
    pub fn frame<'a>(
        &'a mut self,
        font: &'a Font,
        input: &'a application::Frame,
        status: Status<'a>,
        [width, height]: [f32; 2],
    ) -> Frame<'a> {
        let Interface { state, layers: [base, popup, tooltip] } = self;
        state.begin(input);
        Frame {
            canvas: Canvas::new(font, base),
            popup: Canvas::new(font, popup),
            tooltips: Canvas::new(font, tooltip),
            state,
            input,
            status,
            width,
            height,
            enabled: true,
            focusable: true,
            fullscreen: false,
        }
    }

    // What the last frame drew, every layer in order, for one draw call.
    pub fn vertices(&self) -> &[Vertex] {
        &self.layers[0]
    }

    // Closes popups, and drops the press and the hover, like while the
    // window is minimized.
    pub fn reset(&mut self) {
        self.state.popup = None;
        self.state.held = None;
        self.state.hovered = None;
    }

    // Where the last frame's widget of this name is, if it drew one.
    pub fn target(&self, name: &str) -> Option<Rect> {
        self.state.targets.iter().find(|target| target.name == name).map(|target| target.area)
    }

    // The names of the last frame's widgets, in the order they were drawn.
    pub fn target_names(&self) -> Vec<String> {
        self.state.targets.iter().map(|target| target.name.clone()).collect()
    }

    // The widget that has keyboard focus, by name.
    pub fn focus(&self) -> Option<String> {
        self.state.focus.and_then(|id| self.state.target(id)).map(|target| target.name.clone())
    }

    // The widget whose popup is open, by name.
    pub fn popup(&self) -> Option<String> {
        self.state.popup.and_then(|id| self.state.target(id)).map(|target| target.name.clone())
    }

    // The last frame's tooltip, by name, and its area.
    pub fn tooltip(&self) -> Option<(String, Rect)> {
        self.state.tooltip.clone()
    }

    // The interface's state, for scripts to save and check: each widget, by
    // name, with its kind and area, and which one the cursor is over, which
    // has the press, which has focus, whose popup is open, and the tooltip.
    pub fn state(&self) -> Value {
        let state = &self.state;
        let targets = state
            .targets
            .iter()
            .map(|target| {
                let mut members = vec![
                    ("Name".to_string(), target.name.clone().into()),
                    ("Kind".to_string(), target.kind.into()),
                ];
                if !target.label.is_empty() {
                    members.push(("Label".to_string(), target.label.clone().into()));
                }
                members.push(("Area".to_string(), area(target.area)));
                Value::Object(members)
            })
            .collect();
        let tooltip = state.tooltip.as_ref().map(|(name, rect)| {
            json::object([("Name", name.clone().into()), ("Area", area(*rect))])
        });
        json::object([
            ("Targets", Value::Array(targets)),
            ("Hovered", state.name(state.hovered)),
            ("Held", state.name(state.held)),
            ("Focus", state.name(state.focus)),
            ("Popup", state.name(state.popup)),
            ("Tooltip", tooltip.into()),
        ])
    }
}

pub(crate) fn area(rect: Rect) -> Value {
    [rect.x, rect.y, rect.width, rect.height].into()
}

// One frame of the interface: the canvases that its layers are drawn on, its
// input, and the widgets, as methods. It ends when it's dropped.
pub struct Frame<'a> {
    // The base layer, for the program's own drawing.
    pub canvas: Canvas<'a>,
    pub(crate) popup: Canvas<'a>,
    pub(crate) tooltips: Canvas<'a>,
    pub(crate) state: &'a mut State,
    pub input: &'a application::Frame,
    pub status: Status<'a>,
    // The client area's size, in pixels.
    pub width: f32,
    pub height: f32,
    // Whether the widgets drawn now take input, and keyboard focus.
    enabled: bool,
    focusable: bool,
    fullscreen: bool,
}

impl Drop for Frame<'_> {
    fn drop(&mut self) {
        self.state.end();
        self.canvas.vertices.extend_from_slice(self.popup.vertices);
        self.canvas.vertices.extend_from_slice(self.tooltips.vertices);
    }
}

impl<'a> Frame<'a> {
    pub fn seconds(&self) -> f64 {
        self.input.seconds
    }

    // Whether the window has focus. Without it, there's no input.
    pub fn focused(&self) -> bool {
        self.input.focused
    }

    pub fn cursor(&self) -> Option<[f32; 2]> {
        self.input.cursor
    }

    pub fn line_height(&self) -> f32 {
        self.canvas.line_height()
    }

    pub fn char_width(&self) -> f32 {
        self.canvas.char_width()
    }

    pub fn text_width(&self, text: &str) -> f32 {
        self.canvas.text_width(text)
    }

    // How many lines of text didn't fit where they went, on any layer, so
    // that tests can check that the panels' text fits.
    pub fn dropped_lines(&self) -> usize {
        self.canvas.dropped_lines + self.popup.dropped_lines + self.tooltips.dropped_lines
    }

    // Has the window switch between maximized and borderless fullscreen,
    // once the frame is drawn.
    pub fn toggle_fullscreen(&mut self) {
        self.fullscreen = !self.fullscreen;
    }

    pub fn fullscreen_requested(&self) -> bool {
        self.fullscreen
    }

    // Whether the widgets drawn after this take input. Disabled ones are
    // faint, and only show tooltips. Returns what it was.
    pub fn set_enabled(&mut self, enabled: bool) -> bool {
        std::mem::replace(&mut self.enabled, enabled)
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    // Whether the widgets drawn after this take keyboard focus, from a press
    // or Tab. A program that has its own keys for them turns it off. Returns
    // what it was.
    pub fn set_focusable(&mut self, focusable: bool) -> bool {
        std::mem::replace(&mut self.focusable, focusable)
    }

    // Gives keyboard focus to the widget of this name, if the last frame
    // drew it.
    pub fn focus(&mut self, name: &str) {
        let id = Id::new(name);
        if self.state.target(id).is_some() {
            self.state.focus = Some(id);
        }
    }

    // Takes keyboard focus away from every widget.
    pub fn clear_focus(&mut self) {
        self.state.focus = None;
    }

    // Whether the widget of this name has an open popup.
    pub fn is_open(&self, name: &str) -> bool {
        self.state.popup == Some(Id::new(name))
    }

    // What the input is on, by name, for a program to show it: what the
    // cursor is over, what has the press, what has keyboard focus, and
    // whose popup is open.
    pub fn hovered_name(&self) -> Option<&str> {
        self.name_of(self.state.hovered)
    }

    pub fn held_name(&self) -> Option<&str> {
        self.name_of(self.state.held)
    }

    pub fn focus_name(&self) -> Option<&str> {
        self.name_of(self.state.focus)
    }

    pub fn popup_name(&self) -> Option<&str> {
        self.name_of(self.state.popup)
    }

    // How many targets the frame before drew.
    pub fn target_count(&self) -> usize {
        self.state.targets.len()
    }

    fn name_of(&self, id: Option<Id>) -> Option<&str> {
        id.and_then(|id| self.state.target(id)).map(|target| target.name.as_str())
    }

    // The events that no widget took: keys for the program's shortcuts, and
    // presses and turns of the wheel over nothing.
    pub fn program_events(&self) -> impl Iterator<Item = &Event> {
        self.state
            .events
            .iter()
            .filter(|(route, _)| *route == Route::Program)
            .map(|(_, event)| event)
    }

    // Whether `key` went down this frame without Control, and not as a
    // repeat, with no widget taking it, for the program's shortcuts. Holding
    // the key down does nothing more.
    pub fn shortcut(&self, key: i32) -> bool {
        self.program_events().any(|event| {
            matches!(*event, Event::Key { key: pressed, control: false, repeat: false, .. } if pressed as i32 == key)
        })
    }

    // Like shortcut, with Control.
    pub fn control_shortcut(&self, key: i32) -> bool {
        self.program_events().any(|event| {
            matches!(*event, Event::Key { key: pressed, control: true, repeat: false, .. } if pressed as i32 == key)
        })
    }

    // Makes `area` a target of the widget called `name`, on the base layer,
    // for the next frame's input, and says what this frame's input did to
    // it. For widgets that a program makes itself.
    pub fn interact(&mut self, name: &str, kind: &'static str, area: Rect, sense: Sense) -> Response {
        self.interact_on(name, kind, "", area, sense, Layer::Base, None)
    }

    // Like interact, with what the widget shows, which scripts see as the
    // target's Label, for a widget whose name doesn't say, like a row of a
    // list of devices, which is named by its place.
    pub fn interact_labeled(&mut self, name: &str, kind: &'static str, label: &str, area: Rect, sense: Sense) -> Response {
        self.interact_on(name, kind, label, area, sense, Layer::Base, None)
    }

    // Changes the label of a target that the frame drew, once its input has
    // changed what it shows, like a select box's choice.
    pub(crate) fn relabel(&mut self, id: Id, label: &str) {
        if let Some(target) = self.state.next_targets.iter_mut().rev().find(|target| target.id == id) {
            target.label.clear();
            target.label.push_str(label);
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn interact_on(
        &mut self,
        name: &str,
        kind: &'static str,
        label: &str,
        area: Rect,
        sense: Sense,
        layer: Layer,
        owner: Option<Id>,
    ) -> Response {
        let id = Id::new(name);
        let sense = if self.enabled {
            Sense {
                focus: sense.focus && self.focusable,
                ..sense
            }
        } else {
            Sense::HOVER
        };
        self.state.next_targets.push(Target {
            id,
            name: name.to_string(),
            kind,
            label: label.to_string(),
            area,
            layer,
            sense,
            owner,
        });
        let state = &self.state;
        let mut response = Response {
            area,
            hovered: state.hovered == Some(id) && state.held.is_none_or(|held| held == id),
            held: state.held == Some(id),
            focused: state.focus == Some(id),
            ..Response::default()
        };
        for event in self.events_for(id) {
            match event {
                Event::Press(_) => {
                    response.pressed = true;
                    response.clicked = true;
                }
                Event::Release(_) => response.released = true,
                Event::Wheel(_, notches) => response.wheel += notches,
                Event::Key { key, repeat: false, .. }
                    if !sense.text && (key as i32 == VK_SPACE || key as i32 == VK_RETURN) =>
                {
                    response.clicked = true
                }
                _ => {}
            }
        }
        response
    }

    // This frame's events for a widget, oldest first. A disabled widget
    // gets none, even if it took input in the frame before.
    pub(crate) fn events_for(&self, id: Id) -> Vec<Event> {
        if !self.enabled {
            return Vec::new();
        }
        self.state
            .events
            .iter()
            .filter(|(route, _)| *route == Route::Widget(id))
            .map(|(_, event)| *event)
            .collect()
    }

    // This frame's events for the parts of a widget, like the rows of a
    // list, by the number after the part's name's last "/", oldest first.
    // A part gets them even if it isn't drawn this frame, like a row that the
    // wheel scrolled away.
    pub(crate) fn part_events(&self, owner: Id) -> Vec<(usize, Event)> {
        if !self.enabled {
            return Vec::new();
        }
        self.state
            .events
            .iter()
            .filter_map(|(route, event)| {
                let Route::Widget(id) = *route else {
                    return None;
                };
                let target = self.state.target(id).filter(|target| target.owner == Some(owner))?;
                let index = target.name.rsplit('/').next()?.parse().ok()?;
                Some((index, *event))
            })
            .collect()
    }

    // The canvas of a layer.
    pub(crate) fn layer(&mut self, layer: Layer) -> &mut Canvas<'a> {
        match layer {
            Layer::Base => &mut self.canvas,
            Layer::Popup => &mut self.popup,
            Layer::Tooltip => &mut self.tooltips,
        }
    }

    // Whether a focused widget shows a ring, as the focus moved by keyboard.
    pub(crate) fn shows_focus(&self, response: &Response) -> bool {
        response.focused && self.state.focus_visible
    }

    // Opens or closes a widget's popup.
    pub(crate) fn set_popup(&mut self, id: Id, open: bool) {
        if open {
            self.state.popup = Some(id);
        } else if self.state.popup == Some(id) {
            self.state.popup = None;
        }
    }

    // Draws nothing of the program's, and takes away its targets, for a
    // window that isn't in a supported setup, which shows only what's wrong.
    pub(crate) fn clear(&mut self) {
        self.canvas.vertices.clear();
        self.popup.vertices.clear();
        self.tooltips.vertices.clear();
        self.state.next_targets.clear();
        self.state.tooltip = None;
    }
}
