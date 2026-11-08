// The interface's frame machinery. A frame: the input is found on the tree
// the user saw, as the frame before this one drew it, and each widget there
// says what the input did to it, as messages; the program's own messages
// are added; the messages are applied, and those they send in turn, until
// none are left; the program describes the frame's tree; the tree is laid
// out, its targets found, the pointer placed on the topmost of them, and
// the tree drawn. So every widget shows all that the input did, whatever
// order the tree lists them in, on the frame of the input.
//
// Widgets are named, like "Volume" or "Devices/3". A name is what a widget is
// known by from one frame to the next, for what it keeps, like a caret, and
// it's what scripts and tests point at. Every widget that a frame shows is a
// target with its area, and so is each part of one that's a target of its
// own, like a list's row. The pointer is on the topmost target under it, so
// a popup over a card takes the pointer from the card's widgets, and a
// dialog takes it from everything under it.

use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet, VecDeque};

use crate::application::program::{Notice, Program};
use crate::core::interaction::{Cursor, Hit, Id, Interaction, Keys, Part, Sense, Target, Wheel, id};
use crate::core::tree::{Dispatch, Draw, Element, Look, Parts, Tooltip, Tree};
use crate::design::theme::{Theme, ThemeKind};
use crate::design::tokens::TOOLTIP_DELAY;
use crate::drawing::geometry::{Point, Rect, Size};
use crate::drawing::painter::{DrawList, Layer, Painter};
use crate::drawing::text::{Fonts, TextStyle};
use crate::input::devices::{Gamepad, Key, Modifiers, MouseButton, Shortcut};
use crate::input::events::{Event, Input};
use crate::platform::clipboard::Clipboard;

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

// What the parts of the last frame cost the CPU, in milliseconds: the input
// on the tree the user saw, the messages, the view, its layout, and its
// drawing. For benchmarks.
#[derive(Clone, Copy, Debug, Default)]
pub struct FramePhases {
    pub input: f64,
    pub update: f64,
    pub view: f64,
    pub layout: f64,
    pub draw: f64,
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
pub(crate) struct Popup {
    pub owner: Id,
    pub name: String,
}

// The most messages a frame applies, counting those that messages send in
// turn, before it gives up on a chain that never ends.
const MOST_MESSAGES: usize = 10_000;

// The targets that decide how elements look this frame: the one the pointer
// is on, the one with a press, and the one with focus, each as its owner and
// which part of the owner it is.
struct LookSources {
    hovered: Option<(Id, Part)>,
    active: Option<(Id, Part, MouseButton)>,
    // Whether the keys reach the focused target: no popup or dialog over it
    // takes them.
    focused: Option<(Id, Part, bool)>,
}

impl LookSources {
    // How an element looks this frame, from what the pointer and the
    // keyboard are doing to its targets.
    fn look(&self, element: Id, enabled: bool, focus_visible: bool) -> Look {
        let mut look = Look {
            enabled,
            ..Look::default()
        };
        if let Some((owner, part)) = &self.hovered
            && *owner == element
            && enabled
        {
            look.hovered_part = Some(part.clone());
            if *part == Part::Whole {
                look.hit.hovered = true;
            }
        }
        if let Some((owner, part, button)) = &self.active
            && *owner == element
        {
            look.held_part = Some(part.clone());
            look.held_button = Some(*button);
            if *part == Part::Whole {
                look.hit.held = true;
                look.hit.held_button = Some(*button);
            }
        }
        if let Some((owner, part, keys_here)) = &self.focused
            && *owner == element
        {
            look.focused = *keys_here;
            look.focused_part = Some(part.clone());
            look.focus_visible = focus_visible;
        }
        look
    }
}

pub struct Interface {
    pub(crate) theme: Theme,
    // The theme that toggle_theme asked for this frame, which the frame's
    // view is drawn in: what `theme` says until then.
    next_theme: Option<Theme>,
    pub(crate) fonts: Fonts,
    pub(crate) draw: DrawList,
    pub(crate) input: Input,
    size: Size,
    fullscreen: bool,
    // The targets of the frame that's laid out, which are the frame the
    // user saw, that the next frame's input is on, until that frame's
    // layout.
    pub(crate) targets: Vec<Target>,
    pub(crate) hovered: Option<Id>,
    // The target with a press, and the button.
    active: Option<(Id, MouseButton)>,
    pub(crate) focused: Option<Id>,
    pub(crate) focus_visible: bool,
    // The keys that the target with focus takes.
    captured: Keys,
    pub(crate) popups: Vec<Popup>,
    // Whether the frame the user saw had a dialog, which then takes all
    // input, and whether the frame laid out has one.
    modal_shown: bool,
    modal: bool,
    // Presses that only closed a popup, which no widget may take.
    press_consumed: [bool; 3],
    pub(crate) escape_consumed: bool,
    tooltip: TooltipState,
    tooltips_enabled: bool,
    // The tooltip that this frame showed: its widget's name, and its area.
    pub(crate) shown_tooltip: Option<(String, Rect)>,
    memory: HashMap<(Id, TypeId), Box<dyn Any>>,
    pub(crate) clipboard: Clipboard,
    pub(crate) cursor: Cursor,
    frame: u64,
    pub(crate) environment: Environment,
    pub(crate) requests: Vec<Request>,
    // The notices under the app bar.
    pub(crate) notices: Vec<Notice>,
    // What the PerfBar along the top of the window shows, which every
    // program's window has, and overlays don't.
    pub(crate) performance: Option<crate::application::performance::Performance>,
    // How many messages the last frame applied, and whether it hit the cap.
    pub(crate) messages_applied: usize,
    pub(crate) messages_capped: bool,
    phases: FramePhases,
}

impl Interface {
    pub fn new(fonts: Fonts, theme: ThemeKind, clipboard: Clipboard) -> Self {
        Self {
            theme: Theme::new(theme),
            next_theme: None,
            fonts,
            draw: DrawList::default(),
            input: Input::default(),
            size: Size::default(),
            fullscreen: false,
            targets: Vec::new(),
            hovered: None,
            active: None,
            focused: None,
            focus_visible: false,
            captured: Keys::None,
            popups: Vec::new(),
            modal_shown: false,
            modal: false,
            press_consumed: [false; 3],
            escape_consumed: false,
            tooltip: TooltipState::default(),
            tooltips_enabled: true,
            shown_tooltip: None,
            memory: HashMap::new(),
            clipboard,
            cursor: Cursor::Arrow,
            frame: 0,
            environment: Environment::default(),
            requests: Vec::new(),
            notices: Vec::new(),
            performance: None,
            messages_applied: 0,
            messages_capped: false,
            phases: FramePhases::default(),
        }
    }

    // Runs a frame of the program: the input on the tree the user saw, the
    // messages, the new tree, and its drawing. `shown` is that tree, which
    // this leaves as the new one. `sent` are messages from outside, like a
    // test's, applied before the input's.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn frame<P: Program>(&mut self, program: &mut P, shown: &mut Option<Tree<P::Message>>, events: &[Event], sent: Vec<P::Message>, time: f64, size: Size, fullscreen: bool) {
        self.frame += 1;
        self.size = size;
        self.fullscreen = fullscreen;
        self.input.begin(time, events);
        self.fonts.begin_frame();
        if let Some(performance) = &mut self.performance {
            performance.frame_started(time);
            performance.update(time);
        }

        // The input, on the frame the user saw.
        let started = std::time::Instant::now();
        self.begin_input();
        let mut messages = sent;
        if let Some(tree) = shown.as_ref() {
            self.dispatch(tree, &mut messages);
        }
        self.end_input();
        {
            let ui = Ui { inner: self };
            messages.extend(program.input(&ui));
        }
        let mut phases = FramePhases::default();
        let mut mark = started;
        let mut lap = |phase: &mut f64| {
            let now = std::time::Instant::now();
            *phase = (now - mark).as_secs_f64() * 1000.0;
            mark = now;
        };
        lap(&mut phases.input);

        // The messages, and those they send, until none are left.
        let mut queue: VecDeque<P::Message> = messages.into_iter().collect();
        let mut applied = 0;
        self.messages_capped = false;
        while let Some(message) = queue.pop_front() {
            if applied >= MOST_MESSAGES {
                self.messages_capped = true;
                eprintln!("The interface stopped after {MOST_MESSAGES} messages in one frame: a message sends messages without end.");
                break;
            }
            applied += 1;
            let mut context = Context { inner: self, queue: &mut queue };
            program.update(message, &mut context);
        }
        self.messages_applied = applied;
        lap(&mut phases.update);

        // The frame's tree, laid out and drawn.
        if let Some(theme) = self.next_theme.take() {
            self.theme = theme;
        }
        let tree = {
            let mut ui = Ui { inner: self };
            program.view(&mut ui)
        };
        self.tooltips_enabled = tree.tooltips;
        lap(&mut phases.view);
        self.layout(&tree);
        lap(&mut phases.layout);
        self.draw_tree(&tree);
        lap(&mut phases.draw);
        self.phases = phases;
        *shown = Some(tree);
    }

    // What the parts of the last frame cost the CPU.
    pub fn phases(&self) -> FramePhases {
        self.phases
    }

    // Runs a frame of a tree that isn't a program's, like an overlay's or
    // the unsupported setup's: the interface's own keys, and drawing.
    pub(crate) fn frame_of<M: Clone + 'static>(&mut self, tree: &Tree<M>, events: &[Event], time: f64, size: Size, fullscreen: bool) {
        self.frame += 1;
        self.size = size;
        self.fullscreen = fullscreen;
        self.input.begin(time, events);
        self.fonts.begin_frame();
        self.begin_input();
        self.end_input();
        if let Some(theme) = self.next_theme.take() {
            self.theme = theme;
        }
        self.layout(tree);
        self.draw_tree(tree);
    }

    // Starts on the frame's input: where the pointer is, by the targets the
    // user saw, and what the keys do before any widget hears them.
    fn begin_input(&mut self) {
        self.cursor = Cursor::Arrow;
        self.press_consumed = [false; 3];
        self.escape_consumed = false;
        self.requests.clear();
        if self.input.focus_lost {
            self.popups.clear();
        }

        // The pointer is on the topmost target under it, or only on the
        // target with a press while there is one. A dialog leaves nothing
        // under it to be on.
        let topmost = self.topmost(&self.targets, self.modal_shown);
        self.hovered = self.under_pointer(topmost);

        // A press outside the open popups closes them, and does nothing else.
        if !self.popups.is_empty() {
            for button in MouseButton::ALL {
                if !self.input.button_pressed(button) {
                    continue;
                }
                let inside = topmost.is_some_and(|hit| {
                    self.popups.iter().any(|popup| popup.owner == hit)
                        || self.targets.iter().any(|target| target.id == hit && target.layer == Layer::Popup)
                        || self.targets.iter().any(|target| target.id == hit && self.popups.iter().any(|popup| popup.owner == target.owner) && target.layer == Layer::Popup)
                });
                if !inside {
                    self.popups.clear();
                    self.press_consumed[button.index()] = true;
                }
            }
        }

        // A press or the wheel puts the tooltip away until the pointer moves
        // to another widget.
        if MouseButton::ALL.iter().any(|button| self.input.button_pressed(*button)) || self.input.wheel != Point::ZERO {
            self.tooltip.dismissed = true;
        }

        self.captured = match self.focused {
            Some(focused) => self.targets.iter().find(|target| target.id == focused).map(|target| target.sense.keys).unwrap_or_default(),
            None => Keys::None,
        };
        if !self.popups.is_empty() && self.captured != Keys::Text {
            self.captured = Keys::Navigate;
        }

        // F9 switches the theme from this frame, which draws in the new one.
        if self.input.key_pressed_once(Key::F9) && !self.is_captured(Key::F9) {
            self.toggle_theme();
        }

        // Esc closes the top popup, or else takes keyboard focus away. A
        // dialog hears it itself.
        if self.input.key_pressed(Key::ESCAPE) {
            if self.popups.pop().is_some() {
                self.escape_consumed = true;
            } else if self.focused.is_some() && !self.modal_shown {
                self.focused = None;
                self.escape_consumed = true;
            }
        }
    }

    // Finds what the input did to each target the user saw, and tells each
    // widget of the tree what the input did to it, which it turns into
    // messages.
    fn dispatch<M: Clone + 'static>(&mut self, tree: &Tree<M>, messages: &mut Vec<M>) {
        let focused_before = self.focused;
        let shown = std::mem::take(&mut self.targets);
        let mut interactions: HashMap<Id, Interaction> = HashMap::new();
        for target in &shown {
            let hit = self.hit(target);
            let anything = hit.hovered || hit.pressed || hit.held || hit.released || hit.clicked || hit.secondary_pressed || hit.secondary_clicked || hit.middle_pressed || hit.wheel != Point::ZERO;
            if !anything {
                continue;
            }
            let interaction = interactions.entry(target.owner).or_default();
            interaction.hits.push((target.part.clone(), hit));
        }
        // The wheel over a widget whose target under the pointer doesn't
        // take it, for what scrolls.
        let wheel_free = self.input.wheel != Point::ZERO && !self.hovered_takes_wheel(&shown);
        let hovered_layer = self.hovered.and_then(|hovered| shown.iter().rev().find(|target| target.id == hovered).map(|target| target.layer));
        self.targets = shown;
        self.move_focus();

        // The widget with keyboard focus hears the keys, if no popup or
        // dialog over it takes them; and the widget that lost focus this
        // frame hears that, like a number box that takes what was typed.
        let owner_of = |focused: Option<Id>| {
            focused.and_then(|focused| {
                self.targets.iter().find(|target| target.id == focused).map(|target| {
                    let keys_here = (self.popups.is_empty() || target.layer >= Layer::Popup) && (!self.modal_shown || target.layer >= Layer::Dialog);
                    (target.owner, target.part.clone(), keys_here)
                })
            })
        };
        let focused_owner = owner_of(self.focused);
        let lost_focus = if self.focused != focused_before { owner_of(focused_before).map(|(owner, _, _)| owner) } else { None };

        let pointer = self.input.pointer;
        let pointer_delta = self.input.pointer_delta;
        let mut elements: Vec<(&Element<M>, bool, bool, Layer)> = Vec::new();
        collect(&tree.elements, true, true, Layer::Base, &mut elements);
        for (element, enabled, focusable, layer) in elements {
            let Some(element_id) = element.id else {
                continue;
            };
            let mut interaction = interactions.remove(&element_id).unwrap_or_default();
            let (focused, focused_part) = match &focused_owner {
                Some((owner, part, keys_here)) if *owner == element_id => (*keys_here, Some(part.clone())),
                _ => (false, None),
            };
            let wheel_over = wheel_free && pointer.is_some_and(|point| element.rect.contains(point)) && hovered_layer.is_none_or(|hovered| hovered <= layer);
            if interaction.hits.is_empty() && !focused && !wheel_over && lost_focus != Some(element_id) && !element.widget.always_interacts() {
                continue;
            }
            interaction.enabled = enabled;
            interaction.focused = focused;
            let on_it = interaction.hits.iter().any(|(_, hit)| hit.hovered || hit.held);
            interaction.pointer = if on_it { pointer } else { None };
            interaction.drag_delta = if interaction.any_held() { pointer_delta } else { Point::ZERO };
            if wheel_over {
                interaction.wheel_over = self.input.wheel;
            }
            interaction.focused_part = focused_part;
            let mut dispatch = Dispatch {
                ui: self,
                messages,
                id: element_id,
                name: element.name.as_deref().unwrap_or(""),
                rect: element.rect,
                enabled,
                focusable,
            };
            element.widget.interact(&interaction, &mut dispatch);
            if let Some(items) = &element.context_menu {
                crate::widgets::menu::interact_context_menu(&interaction, &mut dispatch, items);
            }
        }
    }

    // What the input did to a target this frame.
    fn hit(&mut self, target: &Target) -> Hit {
        let id = target.id;
        let sense = target.sense;
        let mut hit = Hit {
            hovered: target.enabled && self.hovered == Some(id),
            ..Hit::default()
        };
        if !target.enabled {
            if self.active.is_some_and(|(active, _)| active == id) {
                self.active = None;
            }
            return hit;
        }
        for button in MouseButton::ALL {
            if !(hit.hovered && sense.takes(button) && self.input.button_pressed(button) && !self.press_consumed[button.index()]) {
                continue;
            }
            self.active = Some((id, button));
            self.press_consumed[button.index()] = true;
            match button {
                MouseButton::Left => {
                    hit.pressed = true;
                    hit.double_clicked = self.input.clicks(MouseButton::Left) >= 2;
                    // A target that doesn't take focus leaves it where it
                    // was, like a text field's clear button leaves it in the
                    // field.
                    if sense.focusable && target.focusable {
                        self.focused = Some(id);
                        self.focus_visible = false;
                    }
                }
                MouseButton::Right => hit.secondary_pressed = true,
                MouseButton::Middle => hit.middle_pressed = true,
            }
        }
        // A widget that doesn't take the right button still hears a right
        // press, for a context menu, unless the press closed the popups.
        if hit.hovered && !sense.right && self.input.button_pressed(MouseButton::Right) && !self.press_consumed[MouseButton::Right.index()] {
            hit.secondary_pressed = true;
        }
        if let Some((active, button)) = self.active
            && active == id
        {
            hit.held = true;
            hit.held_button = Some(button);
            if self.input.button_released(button) || self.input.focus_lost {
                let on_it = !self.input.focus_lost && self.input.pointer.is_some_and(|pointer| target.rect.contains(pointer));
                match button {
                    MouseButton::Left => {
                        hit.released = true;
                        hit.clicked = on_it;
                    }
                    MouseButton::Right => hit.secondary_clicked = on_it,
                    MouseButton::Middle => {}
                }
            }
        }
        if hit.hovered {
            hit.wheel = self.input.wheel;
        }
        if self.focused == Some(id) && sense.keys == Keys::Activate && (self.input.key_pressed_once(Key::SPACE) || self.input.key_pressed_once(Key::ENTER)) {
            let keys_here = (self.popups.is_empty() || target.layer >= Layer::Popup) && (!self.modal_shown || target.layer >= Layer::Dialog);
            if keys_here {
                hit.clicked = true;
            }
        }
        hit
    }

    // Moves keyboard focus as the frame's input asks, after every target's
    // hit is found and before any widget hears the input, so that each
    // widget knows whether it has focus as the frame ends.
    fn move_focus(&mut self) {
        // Tab and Shift+Tab move keyboard focus through the targets that take
        // it, in the order the frame showed them, in a dialog if one is open.
        if self.input.key_pressed(Key::TAB) && self.popups.is_empty() && !self.input.modifiers().control {
            let layer = if self.modal_shown { Layer::Dialog } else { Layer::Base };
            let order: Vec<Id> = self
                .targets
                .iter()
                .filter(|target| target.focusable && target.sense.focusable && target.enabled && target.layer == layer)
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
    }

    // Ends the frame's input: what no widget took is let go of, and the
    // interface's own keys.
    fn end_input(&mut self) {
        if let Some((_, button)) = self.active
            && !self.input.button_down(button)
        {
            self.active = None;
        }
        if self.input.key_pressed_once(Key::F10) && !self.is_captured(Key::F10) {
            self.requests.push(Request::ToggleVerticalSync);
        }
        if self.input.key_pressed_once(Key::F11) && !self.is_captured(Key::F11) {
            self.requests.push(Request::ToggleFullscreen);
        }
    }

    // Lays the frame's tree out: finds every target, and which layer each
    // is on, and places the pointer on the topmost of them, for the look.
    fn layout<M: Clone + 'static>(&mut self, tree: &Tree<M>) {
        self.targets.clear();
        self.modal = false;
        let bounds = self.bounds();
        // The PerfBar along the top covers what's under it.
        if self.performance.is_some() {
            self.cover_on(Layer::Base, Rect::new(0.0, 0.0, self.size.width, self.content_top()), id("PerfBar"));
        }
        self.layout_elements(&tree.elements, true, true, Layer::Base, bounds);

        // A target that's gone or disabled can't keep focus, a press, or a
        // popup.
        if let Some(focused) = self.focused
            && !self.targets.iter().any(|target| target.id == focused && target.enabled)
        {
            self.focused = None;
        }
        if let Some((active, _)) = self.active
            && !self.targets.iter().any(|target| target.id == active)
        {
            self.active = None;
        }
        let shown: HashSet<Id> = self.targets.iter().map(|target| target.id).collect();
        self.popups.retain(|popup| shown.contains(&popup.owner));

        // The pointer, on the frame as it's drawn, for the look of what's
        // under it.
        let topmost = self.topmost(&self.targets, self.modal);
        self.hovered = self.under_pointer(topmost);
        self.modal_shown = self.modal;

        // Tooltips show once the pointer rests on a widget.
        if self.hovered != self.tooltip.candidate {
            self.tooltip = TooltipState {
                candidate: self.hovered,
                since: self.input.time,
                anchor: self.input.pointer.unwrap_or_default(),
                dismissed: false,
            };
        }
    }

    fn layout_elements<M: Clone + 'static>(&mut self, elements: &[Element<M>], enabled: bool, focusable: bool, layer: Layer, clip: Rect) {
        for element in elements {
            let enabled = enabled && element.enabled;
            let focusable = focusable && element.focusable;
            let layer = layer.max(element.widget.layer());
            if layer == Layer::Dialog {
                self.modal = true;
            }
            let clip = if element.widget.layer() > Layer::Base { self.bounds() } else { clip };
            let element_id = element.id.unwrap_or_else(|| cover_id(element.rect));
            match (element.id, element.widget.sense()) {
                (Some(id), Some(sense)) => {
                    let label = element.label.clone().or_else(|| element.widget.label());
                    self.targets.push(Target {
                        id,
                        name: element.name.clone().unwrap_or_default(),
                        label,
                        rect: element.rect.intersect(clip),
                        layer,
                        enabled,
                        focusable: focusable && sense.focusable,
                        sense,
                        kind: element.widget.kind(),
                        listed: true,
                        owner: id,
                        part: Part::Whole,
                    });
                }
                _ if element.widget.covers() => self.cover_on(layer, element.rect.intersect(clip), element_id),
                _ => {}
            }
            let mut parts = Parts {
                ui: self,
                id: element_id,
                name: element.name.as_deref().unwrap_or(""),
                rect: element.rect,
                enabled,
                focusable,
                layer,
                clip,
            };
            element.widget.parts(&mut parts);
            if let Some(items) = &element.context_menu {
                crate::widgets::menu::parts_context_menu(&mut parts, items);
            }
            let children_clip = element.widget.children_clip(&mut parts).map_or(clip, |own| own.intersect(clip));
            self.layout_elements(&element.children, enabled, focusable, layer, children_clip);
        }
    }

    // Draws the frame's tree, each element after what it's over, and the
    // tooltip and the PerfBar over all of it.
    fn draw_tree<M: Clone + 'static>(&mut self, tree: &Tree<M>) {
        self.draw.clear(self.size);
        self.shown_tooltip = None;
        let bounds = self.bounds();
        // The tooltip due, if one is: the widget it's of, and which part.
        let due = self.tooltip_due().and_then(|anchor| self.targets.iter().find(|target| Some(target.id) == self.hovered).map(|target| (target.owner, target.part.clone(), target.name.clone(), anchor)));
        let mut tooltip: Option<(Tooltip, String, Point)> = None;
        let sources = self.look_sources();
        self.draw_elements(&tree.elements, true, Layer::Base, bounds, &sources, &due, &mut tooltip);
        if let Some((tooltip, name, anchor)) = tooltip {
            crate::widgets::tool_tip::draw_tooltip(self, &name, &tooltip, anchor);
        }
        if self.performance.is_some() {
            self.perf_bar();
        }
    }

    #[allow(clippy::type_complexity)]
    fn draw_elements<M: Clone + 'static>(&mut self, elements: &[Element<M>], enabled: bool, layer: Layer, clip: Rect, sources: &LookSources, due: &Option<(Id, Part, String, Point)>, tooltip: &mut Option<(Tooltip, String, Point)>) {
        for element in elements {
            let enabled = enabled && element.enabled;
            let layer = layer.max(element.widget.layer());
            let clip = if element.widget.layer() > Layer::Base { self.bounds() } else { clip };
            let element_id = element.id.unwrap_or_else(|| cover_id(element.rect));
            let look = sources.look(element_id, enabled, self.focus_visible);
            self.draw.set_layer(layer);
            self.draw.push_clip(clip, true);
            let mut draw = Draw {
                ui: self,
                id: element_id,
                name: element.name.as_deref().unwrap_or(""),
                rect: element.rect,
                look,
                layer,
            };
            element.widget.draw(&mut draw);
            if let Some(items) = &element.context_menu
                && self.popup_open(element_id)
            {
                let mut draw = Draw {
                    ui: self,
                    id: element_id,
                    name: element.name.as_deref().unwrap_or(""),
                    rect: element.rect,
                    look: Look::default(),
                    layer,
                };
                crate::widgets::menu::draw_context_menu(&mut draw, items);
            }
            if let Some((owner, part, name, anchor)) = due
                && *owner == element_id
                && tooltip.is_none()
            {
                let own = match part {
                    Part::Whole => element.tooltip.clone(),
                    part => element.widget.part_tooltip(part).or_else(|| element.tooltip.clone()),
                };
                if let Some(own) = own {
                    *tooltip = Some((own, name.clone(), *anchor));
                }
            }
            self.draw.pop_clip();
            let children_clip = {
                let mut parts = Parts {
                    ui: self,
                    id: element_id,
                    name: element.name.as_deref().unwrap_or(""),
                    rect: element.rect,
                    enabled,
                    focusable: true,
                    layer,
                    clip,
                };
                element.widget.children_clip(&mut parts).map_or(clip, |own| own.intersect(clip))
            };
            self.draw_elements(&element.children, enabled, layer, children_clip, sources, due, tooltip);
        }
    }

    // What every element's look this frame comes from: the targets that the
    // pointer is on, that have a press, and that have focus, found once.
    fn look_sources(&self) -> LookSources {
        let find = |id: Option<Id>| id.and_then(|id| self.targets.iter().find(|target| target.id == id));
        let keys_here = |target: &Target| (self.popups.is_empty() || target.layer >= Layer::Popup) && (!self.modal || target.layer >= Layer::Dialog);
        LookSources {
            hovered: find(self.hovered).map(|target| (target.owner, target.part.clone())),
            active: self.active.and_then(|(active, button)| find(Some(active)).map(|target| (target.owner, target.part.clone(), button))),
            focused: find(self.focused).map(|target| (target.owner, target.part.clone(), keys_here(target))),
        }
    }

    // The topmost of the targets under the pointer, of the dialog's layer or
    // above while a dialog is open, which leaves nothing under it to be on.
    fn topmost(&self, targets: &[Target], modal: bool) -> Option<Id> {
        let point = self.input.pointer?;
        let lowest = if modal { Layer::Dialog } else { Layer::Base };
        targets
            .iter()
            .filter(|target| target.layer >= lowest && target.rect.contains(point))
            .fold(None::<&Target>, |best, target| match best {
                Some(best) if best.layer > target.layer => Some(best),
                _ => Some(target),
            })
            .map(|target| target.id)
    }

    // The target that the pointer is on, with the topmost target under it:
    // that one, or only the target with a press while there is one.
    fn under_pointer(&self, topmost: Option<Id>) -> Option<Id> {
        match self.active {
            Some((active, _)) => (topmost == Some(active)).then_some(active),
            None => topmost,
        }
    }

    // Whether the target under the pointer takes the wheel itself, so that
    // what it's in shouldn't scroll.
    fn hovered_takes_wheel(&self, targets: &[Target]) -> bool {
        let Some(hovered) = self.hovered else { return false };
        match targets.iter().rev().find(|target| target.id == hovered).map(|target| target.sense.wheel) {
            Some(Wheel::Taken) => true,
            Some(Wheel::WhileFocused) => self.hovered == self.focused,
            _ => false,
        }
    }

    pub(crate) fn hovered_target(&self) -> Option<&Target> {
        let hovered = self.hovered?;
        self.targets.iter().rev().find(|target| target.id == hovered)
    }

    pub(crate) fn held_target(&self) -> Option<&Target> {
        let (active, _) = self.active?;
        self.targets.iter().rev().find(|target| target.id == active)
    }

    // The theme, whose roles color everything: the one this frame draws in,
    // as the frame's toggles leave it.
    pub fn theme(&self) -> &Theme {
        self.next_theme.as_ref().unwrap_or(&self.theme)
    }

    // Switches between the light and dark themes, as F9 does, for this
    // frame's view on.
    pub fn toggle_theme(&mut self) {
        let kind = self.theme().kind.other();
        self.next_theme = Some(Theme::new(kind));
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

    // Whether frames wait for the screen's refresh.
    pub fn vertical_sync(&self) -> bool {
        self.environment.vertical_sync
    }

    // Seconds since the program started, on the frames' clock, which runs
    // at a fixed rate for scripts and tests.
    pub fn time(&self) -> f64 {
        self.input.time
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

    pub fn input(&self) -> &Input {
        &self.input
    }

    pub fn gamepad(&self) -> &Gamepad {
        &self.input.gamepad
    }

    pub fn tooltips_enabled(&self) -> bool {
        self.tooltips_enabled
    }

    // Whether the key was pressed this frame, counting a held key's repeats,
    // unless the widget with keyboard focus takes it, as a text field takes
    // letters, or a dialog is open over the page.
    pub fn key_pressed(&self, key: Key) -> bool {
        self.input.key_pressed(key) && !self.is_captured(key) && !self.modal_shown
    }

    pub fn key_pressed_once(&self, key: Key) -> bool {
        self.input.key_pressed_once(key) && !self.is_captured(key) && !self.modal_shown
    }

    pub fn key_down(&self, key: Key) -> bool {
        self.input.key_down(key) && !(self.captured == Keys::Text && !key.is_modifier()) && !self.modal_shown
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

    pub(crate) fn painter(&mut self) -> Painter<'_> {
        Painter {
            list: &mut self.draw,
            fonts: &mut self.fonts,
            theme: &self.theme,
        }
    }

    pub fn measure(&mut self, text: &str, style: TextStyle) -> Size {
        self.fonts.measure(text, style)
    }

    pub fn fonts(&mut self) -> &mut Fonts {
        &mut self.fonts
    }

    pub fn cursor(&self) -> Cursor {
        self.cursor
    }

    // Covers what's under the area, like a card covers the 3D view under
    // it, without being a target that scripts see.
    pub(crate) fn cover_on(&mut self, layer: Layer, rect: Rect, owner: Id) {
        self.targets.push(Target {
            id: cover_id(rect),
            name: String::new(),
            label: None,
            rect,
            layer,
            enabled: true,
            focusable: false,
            sense: Sense::NONE,
            kind: "Surface",
            listed: false,
            owner,
            part: Part::Named("Cover".to_string()),
        });
    }

    // How many instances the frame drew: each shape, glyph, line, arc,
    // polygon and shadow is one, which the renderer draws in one call. For
    // benchmarks.
    pub fn instance_count(&self) -> usize {
        self.draw.instance_count()
    }

    // Opens the popup that the widget with the id owns, like a combo box's
    // list. A popup whose owner is in an open popup opens over it, like a
    // submenu; any other closes the open ones first.
    pub(crate) fn open_popup(&mut self, owner: Id, name: &str) {
        let in_popup = self.targets.iter().any(|target| target.id == owner && target.layer == Layer::Popup);
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
    pub(crate) fn close_popup(&mut self, owner: Id) {
        if let Some(index) = self.popups.iter().position(|popup| popup.owner == owner) {
            self.popups.truncate(index);
        }
    }

    pub(crate) fn popup_open(&self, owner: Id) -> bool {
        self.popups.iter().any(|popup| popup.owner == owner)
    }

    // Whether the widget's popup is the top one, which takes the keys.
    pub(crate) fn popup_on_top(&self, owner: Id) -> bool {
        self.popups.last().is_some_and(|popup| popup.owner == owner)
    }

    pub fn any_popup(&self) -> bool {
        !self.popups.is_empty()
    }

    // Whether a dialog is open, which takes the input from the page.
    pub fn modal(&self) -> bool {
        self.modal
    }

    // Where the pointer rested on the hovered target, if its tooltip should
    // show now.
    fn tooltip_due(&self) -> Option<Point> {
        let due = self.tooltips_enabled
            && self.active.is_none()
            && self.popups.is_empty()
            && self.hovered.is_some()
            && self.tooltip.candidate == self.hovered
            && !self.tooltip.dismissed
            && self.input.time - self.tooltip.since >= TOOLTIP_DELAY;
        due.then_some(self.tooltip.anchor)
    }

    // State that a widget keeps from frame to frame, like a text field's
    // caret, by the widget's id.
    pub(crate) fn memory<T: Default + 'static>(&mut self, id: Id) -> &mut T {
        self.memory
            .entry((id, TypeId::of::<T>()))
            .or_insert_with(|| Box::new(T::default()))
            .downcast_mut::<T>()
            .expect("memory is keyed by its type")
    }

    // The notices under the app bar.
    pub fn notices(&self) -> &[Notice] {
        &self.notices
    }

    // The targets of the last frame that's laid out.
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
        self.targets.iter().find(|target| target.id == id && target.listed).map(|target| target.name.as_str())
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
            ("Dialog".to_string(), Value::from(self.modal)),
            ("Tooltip".to_string(), tooltip),
            ("Targets".to_string(), Value::Object(targets)),
        ])
    }
}

// The id of a cover over the area, which is no widget.
fn cover_id(rect: Rect) -> Id {
    id(&format!("Cover:{}:{}:{}:{}", rect.x, rect.y, rect.width, rect.height))
}

// Every element with its enabling, whether it takes focus, and its layer,
// in drawing order.
fn collect<'a, M: 'static>(elements: &'a [Element<M>], enabled: bool, focusable: bool, layer: Layer, out: &mut Vec<(&'a Element<M>, bool, bool, Layer)>) {
    for element in elements {
        let enabled = enabled && element.enabled;
        let focusable = focusable && element.focusable;
        let layer = layer.max(element.widget.layer());
        out.push((element, enabled, focusable, layer));
        collect(&element.children, enabled, focusable, layer, out);
    }
}

// What a program's `view` and `input` see of the interface: the frame as it
// stands, which they read and don't change, and the fonts to measure text
// with.
pub struct Ui<'a> {
    pub(crate) inner: &'a mut Interface,
}

impl Ui<'_> {
    pub fn theme(&self) -> &Theme {
        self.inner.theme()
    }

    pub fn size(&self) -> Size {
        self.inner.size
    }

    pub fn bounds(&self) -> Rect {
        self.inner.bounds()
    }

    pub fn content_top(&self) -> f32 {
        self.inner.content_top()
    }

    pub fn fullscreen(&self) -> bool {
        self.inner.fullscreen
    }

    pub fn vertical_sync(&self) -> bool {
        self.inner.environment.vertical_sync
    }

    pub fn time(&self) -> f64 {
        self.inner.input.time
    }

    // Seconds since the last frame.
    pub fn delta(&self) -> f32 {
        self.inner.input.delta
    }

    pub fn frame_number(&self) -> u64 {
        self.inner.frame
    }

    pub fn stats(&self) -> FrameStats {
        self.inner.stats()
    }

    pub fn environment(&self) -> &Environment {
        &self.inner.environment
    }

    // All of the frame's input, for programs that read it themselves, like
    // the keys held for driving.
    pub fn input(&self) -> &Input {
        &self.inner.input
    }

    // Whether the window has focus. Without it there's no input at all.
    pub fn window_focused(&self) -> bool {
        self.inner.input.focused
    }

    pub fn gamepad(&self) -> &Gamepad {
        &self.inner.input.gamepad
    }

    pub fn pointer(&self) -> Option<Point> {
        self.inner.input.pointer
    }

    // Whether the key was pressed this frame, counting a held key's repeats,
    // unless the widget with keyboard focus takes it, as a text field takes
    // letters, or a dialog is open over the page.
    pub fn key_pressed(&self, key: Key) -> bool {
        self.inner.key_pressed(key)
    }

    pub fn key_pressed_once(&self, key: Key) -> bool {
        self.inner.key_pressed_once(key)
    }

    // Whether the key was pressed with exactly these modifiers, like Ctrl+S.
    pub fn shortcut(&self, modifiers: Modifiers, key: Key) -> bool {
        self.inner.input.modifiers() == modifiers && self.key_pressed(key)
    }

    pub fn shortcut_pressed(&self, shortcut: Shortcut) -> bool {
        self.shortcut(shortcut.modifiers, shortcut.key)
    }

    pub fn key_down(&self, key: Key) -> bool {
        self.inner.key_down(key)
    }

    pub fn measure(&mut self, text: &str, style: TextStyle) -> Size {
        self.inner.fonts.measure(text, style)
    }

    pub fn measure_wrapped(&mut self, text: &str, style: TextStyle, width: f32) -> Size {
        self.inner.fonts.measure_wrapped(text, style, width)
    }

    // How tall the text is, wrapped at the width.
    pub fn paragraph_height(&mut self, width: f32, text: &str, style: TextStyle) -> f32 {
        self.inner.fonts.measure_wrapped(text, style, width).height
    }

    pub fn fonts(&mut self) -> &mut Fonts {
        &mut self.inner.fonts
    }

    pub fn notices(&self) -> &[Notice] {
        &self.inner.notices
    }

    // Whether a dialog is open, which takes the input from the page.
    pub fn modal(&self) -> bool {
        self.inner.modal
    }

    pub fn any_popup(&self) -> bool {
        !self.inner.popups.is_empty()
    }

    // The names of the widgets that the input is on.
    pub fn hovered_name(&self) -> Option<&str> {
        self.inner.hovered_name()
    }

    pub fn pressed_name(&self) -> Option<&str> {
        self.inner.pressed_name()
    }

    pub fn focused_name(&self) -> Option<&str> {
        self.inner.focused_name()
    }

    pub fn popup_name(&self) -> Option<&str> {
        self.inner.popup_name()
    }

    pub fn tooltip_name(&self) -> Option<&str> {
        self.inner.tooltip_name()
    }

    pub fn is_focused(&self, name: &str) -> bool {
        self.inner.focused == Some(id(name))
    }

    pub fn tooltips_enabled(&self) -> bool {
        self.inner.tooltips_enabled
    }

    // What a widget keeps between frames, read for laying the frame out,
    // like how far a scroll area is scrolled.
    pub fn memory<T: Default + Clone + 'static>(&mut self, name: &str) -> T {
        self.inner.memory::<T>(id(name)).clone()
    }

}

// What a program's `update` works with: the interface's state, and where
// the messages it sends go.
pub struct Context<'a, M> {
    pub(crate) inner: &'a mut Interface,
    pub(crate) queue: &'a mut VecDeque<M>,
}

impl<M> Context<'_, M> {
    // Sends a message, which is applied after those before it, this frame.
    pub fn send(&mut self, message: M) {
        self.queue.push_back(message);
    }

    pub fn theme(&self) -> &Theme {
        self.inner.theme()
    }

    // Switches between the light and dark themes, as F9 does, from this
    // frame's view on.
    pub fn toggle_theme(&mut self) {
        self.inner.toggle_theme();
    }

    pub fn toggle_fullscreen(&mut self) {
        self.inner.requests.push(Request::ToggleFullscreen);
    }

    // Switches vertical sync, as F10 does, from the next frame on.
    pub fn toggle_vertical_sync(&mut self) {
        self.inner.requests.push(Request::ToggleVerticalSync);
    }

    // Closes the window, whatever Program::may_close says.
    pub fn close(&mut self) {
        self.inner.requests.push(Request::Close);
    }

    pub fn time(&self) -> f64 {
        self.inner.input.time
    }

    pub fn delta(&self) -> f32 {
        self.inner.input.delta
    }

    pub fn fullscreen(&self) -> bool {
        self.inner.fullscreen
    }

    pub fn vertical_sync(&self) -> bool {
        self.inner.environment.vertical_sync
    }

    pub fn environment(&self) -> &Environment {
        &self.inner.environment
    }

    pub fn input(&self) -> &Input {
        &self.inner.input
    }

    pub fn window_focused(&self) -> bool {
        self.inner.input.focused
    }

    pub fn size(&self) -> Size {
        self.inner.size
    }

    pub fn bounds(&self) -> Rect {
        self.inner.bounds()
    }

    // Shows a notice under the app bar.
    pub fn notify(&mut self, notice: Notice) {
        self.inner.notices.push(notice);
    }

    pub fn notices(&self) -> &[Notice] {
        &self.inner.notices
    }

    pub fn remove_notice(&mut self, index: usize) {
        if index < self.inner.notices.len() {
            self.inner.notices.remove(index);
        }
    }

    // Gives keyboard focus to the widget with the name.
    pub fn focus(&mut self, name: &str) {
        self.inner.focused = Some(id(name));
    }

    pub fn clear_focus(&mut self) {
        self.inner.focused = None;
    }

    pub fn is_focused(&self, name: &str) -> bool {
        self.inner.focused == Some(id(name))
    }

    pub fn hovered_name(&self) -> Option<&str> {
        self.inner.hovered_name()
    }

    pub fn pressed_name(&self) -> Option<&str> {
        self.inner.pressed_name()
    }

    pub fn focused_name(&self) -> Option<&str> {
        self.inner.focused_name()
    }

    pub fn popup_name(&self) -> Option<&str> {
        self.inner.popup_name()
    }

    pub fn tooltip_name(&self) -> Option<&str> {
        self.inner.tooltip_name()
    }

    // Whether the frame the user saw had a dialog, which took its input.
    #[allow(clippy::misnamed_getters)]
    pub fn modal(&self) -> bool {
        self.inner.modal_shown
    }

    pub fn close_popups(&mut self) {
        self.inner.popups.clear();
    }
}
