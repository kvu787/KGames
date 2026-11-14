// The interface's frame machinery. A frame takes its input in steps, in the
// order it happened, one for each event that does something, like a press,
// a key or the wheel (see `Input::steps`), so that each event sees the state
// that the events before it left, as a Win32 window's messages do. A step:
// its input is found on the tree that the step before laid out, which for a
// frame's first step is the tree the user saw, and each widget there says
// what the input did to it, as messages; the program's own messages are
// added; the messages are applied, and those they send in turn, until none
// are left; the program describes the step's tree; and the tree is laid
// out, its targets found, and the pointer placed on the topmost of them.
// Then the last step's tree is drawn. So a second click in a frame lands on
// the menu that the first one opened, and every widget shows all that the
// input did, whatever order the tree lists them in, on the frame of the
// input. Every step sees the frame's time, and the frame's delta comes in its
// last step, after its events.
//
// Widgets are named, like "Volume" or "Devices/3". A name is what a widget is
// known by from one frame to the next, for what it keeps, like a caret, and
// it's what scripts and tests point at. Every widget that a frame shows is a
// target with its area, and so is each part of one that's a target of its
// own, like a list's row. The pointer is on the topmost target under it, so
// a popup over a card takes the pointer from the card's widgets, and a
// dialog takes it from everything under it.

use std::any::{Any, TypeId};
use std::collections::VecDeque;
use std::ops::Deref;

use crate::application::program::{Notice, Program};
use crate::core::interaction::{Cursor, Hit, Id, Interaction, Part, Sense, Target, Wheel, id};
use crate::core::tree::{Dispatch, Draw, Element, Look, Parts, Tooltip, Tree};
use crate::design::theme::{Theme, ThemeKind};
use crate::design::tokens::TOOLTIP_DELAY;
use crate::drawing::geometry::{Point, Rect, Size};
use crate::drawing::hashing::{QuickMap, QuickSet};
use crate::drawing::painter::{DrawList, Layer, Painter};
use crate::drawing::text::{Fonts, TextStyle};
use crate::input::devices::{Gamepad, Key, Modifiers, MouseButton, Shortcut};
use crate::input::events::{Event, Input};

// What the program runs on, for its bars and its title.
#[derive(Clone, Debug, Default)]
pub struct Environment {
    // Like "NVIDIA GeForce RTX 5090 Laptop GPU".
    pub gpu: String,
    // The display's identifier, like "CVVNY2", and its model.
    pub display: String,
    pub display_name: String,
    // Whether frames wait for the screen, and never tear, or present at
    // once, and may tear, which F10 switches.
    pub vertical_sync: bool,
    // Whether input comes from a script instead of the user.
    pub scripted: bool,
}

// What a program asks of its window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Request {
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

// What the parts of the last frame cost the CPU, in milliseconds, over all
// of its steps: the input on the tree it came on, the messages, the view,
// and its layout, and the drawing of the last step's tree. For benchmarks.
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

// The most messages a step of a frame's input applies, counting those that
// messages send in turn, before it gives up on a chain that never ends. Per
// step, as a frame can have any number of steps, like a burst of wheel
// readings after a long stall, each with its own messages, which all settle.
const MOST_MESSAGES: usize = 10_000;

// The targets that decide how elements look this frame: the one the pointer
// is on and the one with a press, each as its owner and which part of the
// owner it is, and the text field with focus.
struct LookSources {
    hovered: Option<(Id, Part)>,
    active: Option<(Id, Part, MouseButton)>,
    // Whether the keys reach the focused field: no dialog over it takes them.
    focused: Option<(Id, bool)>,
}

impl LookSources {
    // How an element looks this frame, from what the pointer and the
    // keyboard are doing to its targets.
    fn look(&self, element: Id, enabled: bool) -> Look {
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
        if let Some((owner, keys_here)) = &self.focused
            && *owner == element
        {
            look.focused = *keys_here;
        }
        look
    }
}

pub struct Interface {
    pub(crate) theme: Theme,
    // The theme that toggle_theme asked for in this step, which the step's
    // view is built in: what `theme` says until then.
    next_theme: Option<Theme>,
    pub(crate) fonts: Fonts,
    pub(crate) draw: DrawList,
    pub(crate) input: Input,
    size: Size,
    // The targets of the tree that's laid out, which the next step's input
    // is on, until that step's layout: the tree the user saw, between
    // frames.
    pub(crate) targets: Vec<Target>,
    pub(crate) hovered: Option<Id>,
    // The target with a press, and the button.
    active: Option<(Id, MouseButton)>,
    // The text field with keyboard focus.
    pub(crate) focused: Option<Id>,
    // Whether a text field with focus takes the keys typed in this step.
    typing: bool,
    pub(crate) popups: Vec<Popup>,
    // Whether the tree that the step's input is on had a dialog, which then
    // takes all input, and whether the tree laid out has one.
    pub(crate) modal_shown: bool,
    modal: bool,
    // Presses that only closed a popup, which no widget may take.
    press_consumed: [bool; 3],
    tooltip: TooltipState,
    tooltips_enabled: bool,
    // The tooltip that this frame showed: its widget's name, and its area.
    pub(crate) shown_tooltip: Option<(String, Rect)>,
    memory: QuickMap<(Id, TypeId), Box<dyn Any>>,
    pub(crate) cursor: Cursor,
    frame: u64,
    pub(crate) environment: Environment,
    pub(crate) requests: Vec<Request>,
    // The notices under the app bar.
    pub(crate) notices: Vec<Notice>,
    // What the PerfBar along the top of the window shows, which every
    // program's window has, and a test's window of a size of its own
    // doesn't.
    pub(crate) performance: Option<crate::application::performance::Performance>,
    // How many messages the last frame applied, over all its steps.
    pub(crate) messages_applied: usize,
    // Whether the frame draws the pointer, while the window has focus, as a
    // window's frames do while it keeps Windows' cursor, hidden, and not a
    // script's or a test's, whose captures never showed Windows' cursor.
    pub(crate) draws_pointer: bool,
    // Whether the step running is its frame's last, which has the frame's
    // delta.
    last_step: bool,
    phases: FramePhases,
}

impl Interface {
    pub fn new(fonts: Fonts, theme: ThemeKind) -> Self {
        Self {
            theme: Theme::new(theme),
            next_theme: None,
            fonts,
            draw: DrawList::default(),
            input: Input::default(),
            size: Size::default(),
            targets: Vec::new(),
            hovered: None,
            active: None,
            focused: None,
            typing: false,
            popups: Vec::new(),
            modal_shown: false,
            modal: false,
            press_consumed: [false; 3],
            tooltip: TooltipState::default(),
            tooltips_enabled: true,
            shown_tooltip: None,
            memory: QuickMap::default(),
            cursor: Cursor::Arrow,
            frame: 0,
            environment: Environment::default(),
            requests: Vec::new(),
            notices: Vec::new(),
            performance: None,
            messages_applied: 0,
            draws_pointer: false,
            last_step: true,
            phases: FramePhases::default(),
        }
    }

    // Runs a frame of the program: its input in steps, each on the tree that
    // the step before left, then the drawing of the last step's tree.
    // `shown` is the tree the user saw, which this leaves as the new one.
    // `sent` are messages from outside, like a test's, applied in the first
    // step, before the input's. Every step sees the frame's time, and the
    // frame's delta comes in its last step.
    pub(crate) fn frame<P: Program>(&mut self, program: &mut P, shown: &mut Option<Tree<P::Message>>, events: &[Event], sent: Vec<P::Message>, time: f64, size: Size) {
        self.frame += 1;
        self.size = size;
        self.requests.clear();
        self.fonts.begin_frame();
        if let Some(performance) = &mut self.performance {
            performance.frame_started(time);
            performance.update(time);
        }
        let mut phases = FramePhases::default();
        let mut applied = 0;
        let mut sent = Some(sent);
        let delta = (time - self.input.time).max(0.0) as f32;
        let steps = self.input.steps(events);
        let last = steps.len() - 1;
        for (index, step) in steps.into_iter().enumerate() {
            self.last_step = index == last;
            let step_delta = if self.last_step { delta } else { 0.0 };
            let tree = self.step(program, shown.as_ref(), step, sent.take().unwrap_or_default(), time, step_delta, &mut applied, &mut phases);
            *shown = Some(tree);
        }
        self.messages_applied = applied;

        // The last step's tree, drawn.
        let started = std::time::Instant::now();
        if let Some(tree) = shown.as_ref() {
            self.draw_tree(tree);
        }
        phases.draw = (std::time::Instant::now() - started).as_secs_f64() * 1000.0;
        self.phases = phases;
    }

    // Runs a step of a frame: the step's input on the tree `shown`, which
    // the user saw for the frame's first step, and which the step before
    // left for the others; the messages; and the step's tree, laid out,
    // which this returns. `applied` counts the frame's messages so far.
    #[allow(clippy::too_many_arguments)]
    fn step<P: Program>(&mut self, program: &mut P, shown: Option<&Tree<P::Message>>, events: &[Event], sent: Vec<P::Message>, time: f64, delta: f32, applied: &mut usize, phases: &mut FramePhases) -> Tree<P::Message> {
        self.input.begin(time, events);
        self.input.delta = delta;

        // The input, on the tree it came on.
        let started = std::time::Instant::now();
        self.begin_input();
        let mut messages = sent;
        if let Some(tree) = shown {
            self.dispatch(tree, &mut messages);
        }
        self.end_input();
        {
            let ui = Ui { inner: self };
            messages.extend(program.input(&ui));
        }
        let mut mark = started;
        let mut lap = |phase: &mut f64| {
            let now = std::time::Instant::now();
            *phase += (now - mark).as_secs_f64() * 1000.0;
            mark = now;
        };
        lap(&mut phases.input);

        // The messages, and those they send, until none are left. A chain
        // that doesn't settle is a bug, and stops the program rather than
        // hiding or dropping what's queued behind it.
        let mut queue: VecDeque<P::Message> = messages.into_iter().collect();
        let mut step_applied = 0;
        while let Some(message) = queue.pop_front() {
            assert!(step_applied < MOST_MESSAGES, "the step stopped after {MOST_MESSAGES} messages: a message sends messages without end");
            step_applied += 1;
            *applied += 1;
            let mut context = Context { inner: self, queue: &mut queue };
            program.update(message, &mut context);
        }
        lap(&mut phases.update);

        // The step's tree, laid out, for the next step's input and for the
        // frame's drawing.
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
        tree
    }

    // What the parts of the last frame cost the CPU, over all its steps.
    pub fn phases(&self) -> FramePhases {
        self.phases
    }

    // Runs a frame of a tree that isn't a program's, like the unsupported
    // setup's: the interface's own keys, in steps, and drawing.
    pub(crate) fn frame_of<M: Clone + 'static>(&mut self, tree: &Tree<M>, events: &[Event], time: f64, size: Size) {
        self.frame += 1;
        self.size = size;
        self.requests.clear();
        self.fonts.begin_frame();
        let delta = (time - self.input.time).max(0.0) as f32;
        let steps = self.input.steps(events);
        let last = steps.len() - 1;
        for (index, step) in steps.into_iter().enumerate() {
            self.last_step = index == last;
            self.input.begin(time, step);
            self.input.delta = if self.last_step { delta } else { 0.0 };
            self.begin_input();
            self.end_input();
            if let Some(theme) = self.next_theme.take() {
                self.theme = theme;
            }
            self.layout(tree);
        }
        self.draw_tree(tree);
    }

    // Starts on a step's input: where the pointer is, by the targets of the
    // tree it came on, and what the keys do before any widget hears them.
    fn begin_input(&mut self) {
        self.press_consumed = [false; 3];
        if self.input.focus_lost {
            self.popups.clear();
        }

        // The pointer is on the topmost target under it, or only on the
        // target with a press while there is one. A dialog leaves nothing
        // under it to be on.
        let topmost = self.topmost(&self.targets, self.modal_shown);
        self.hovered = self.under_pointer(topmost);

        // A press outside the open popups closes them, and does nothing else,
        // even on what a popup belongs to, like a combo box, or an area with
        // a context menu.
        if !self.popups.is_empty() {
            for button in MouseButton::ALL {
                if !self.input.button_pressed(button) {
                    continue;
                }
                let inside = topmost.is_some_and(|hit| self.targets.iter().any(|target| target.id == hit && target.layer == Layer::Popup));
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

        self.typing = self.focused.is_some_and(|focused| self.targets.iter().any(|target| target.id == focused && target.sense.text));

        // F9 switches the theme from this step, whose view is in the new one.
        if self.input.key_pressed(Key::F9) {
            self.toggle_theme();
        }
    }

    // Finds what the step's input did to each target of the tree it came
    // on, and tells each widget of the tree what the input did to it, which
    // it turns into messages.
    fn dispatch<M: Clone + 'static>(&mut self, tree: &Tree<M>, messages: &mut Vec<M>) {
        let shown = std::mem::take(&mut self.targets);
        let mut interactions: QuickMap<Id, Interaction> = QuickMap::default();
        for target in &shown {
            let hit = self.hit(target);
            let anything = hit.hovered || hit.pressed || hit.held || hit.released || hit.secondary_pressed || hit.middle_pressed || hit.wheel != Point::ZERO;
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

        // The text field with keyboard focus hears the keys, if no dialog
        // over it takes them.
        let focused_owner = owner_of_focus(&self.targets, self.modal_shown, self.focused);

        let pointer = self.input.pointer;
        let pointer_delta = self.input.pointer_delta;
        let mut elements: Vec<(&Element<M>, bool, Layer)> = Vec::new();
        collect(&tree.elements, true, Layer::Base, &mut elements);
        // The wheel that nothing under the pointer takes goes to the deepest
        // widget there that scrolls, and only to it, so one notch never
        // scrolls a list and what it's in together.
        let wheel_target: Option<Id> = if wheel_free {
            elements
                .iter()
                .filter(|(element, _, layer)| {
                    element.id.is_some()
                        && element.widget.takes_free_wheel()
                        && pointer.is_some_and(|point| element.rect.contains(point))
                        && hovered_layer.is_none_or(|hovered| hovered <= *layer)
                })
                .next_back()
                .and_then(|(element, _, _)| element.id)
        } else {
            None
        };
        for &(element, enabled, _layer) in &elements {
            let Some(element_id) = element.id else {
                continue;
            };
            let mut interaction = interactions.remove(&element_id).unwrap_or_default();
            let focused = matches!(focused_owner, Some((owner, true)) if owner == element_id);
            let wheel_over = wheel_target == Some(element_id);
            if interaction.hits.is_empty() && !focused && !wheel_over && !element.widget.always_interacts() {
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
            let mut dispatch = Dispatch {
                ui: self,
                messages,
                id: element_id,
                name: element.name.as_deref().unwrap_or(""),
                rect: element.rect,
                enabled,
            };
            element.widget.interact(&interaction, &mut dispatch);
            if let Some(items) = &element.context_menu {
                crate::widgets::menu::interact_context_menu(&interaction, &mut dispatch, items);
            }
        }
        self.take_focus_away();
    }

    // What the step's input did to a target. A press acts on the step that
    // the button goes down: a left press presses the target, and a
    // right press asks for its context menu, whatever the pointer does
    // before the button comes up.
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
                    if sense.text {
                        self.focused = Some(id);
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
            if button == MouseButton::Left && (self.input.button_released(button) || self.input.focus_lost) {
                hit.released = true;
            }
        }
        if hit.hovered {
            hit.wheel = self.input.wheel;
        }
        hit
    }

    // A left press on a text field gives it keyboard focus, as its hit is
    // found, and any press anywhere else takes it away, here, once every
    // widget has heard the step's input. A press on the field itself, or on
    // its own parts, like its clear button, leaves it there.
    fn take_focus_away(&mut self) {
        let pressed = MouseButton::ALL.iter().any(|&button| self.input.button_pressed(button));
        if pressed && self.focused.is_some() {
            let on = self.hovered.and_then(|hovered| self.targets.iter().find(|target| target.id == hovered)).map(|target| target.owner);
            if on != self.focused {
                self.focused = None;
            }
        }
    }

    // Ends the step's input: what no widget took is let go of, and the
    // interface's own keys.
    fn end_input(&mut self) {
        if let Some((_, button)) = self.active
            && !self.input.button_down(button)
        {
            self.active = None;
        }
        if self.input.key_pressed(Key::F10) {
            // The frame presents with the new setting, so the PerfBar says
            // it from this very frame; the loop re-reads the truth after.
            self.environment.vertical_sync = !self.environment.vertical_sync;
            self.requests.push(Request::ToggleVerticalSync);
        }
    }

    // Lays a step's tree out: finds every target, and which layer each
    // is on, and places the pointer on the topmost of them, for the look.
    fn layout<M: Clone + 'static>(&mut self, tree: &Tree<M>) {
        self.targets.clear();
        self.modal = false;
        let bounds = self.bounds();
        // The PerfBar along the top covers what's under it.
        if self.performance.is_some() {
            self.cover_on(Layer::Base, Rect::new(0.0, 0.0, self.size.width, self.content_top()), id("PerfBar"));
        }
        self.layout_elements(&tree.elements, true, Layer::Base, bounds);

        // A target that's gone, disabled, or no longer a text field can't
        // keep focus, a press, or a popup.
        if let Some(focused) = self.focused
            && !self.targets.iter().any(|target| target.id == focused && target.enabled && target.sense.text)
        {
            self.focused = None;
        }
        if let Some((active, _)) = self.active
            && !self.targets.iter().any(|target| target.id == active)
        {
            self.active = None;
        }
        if !self.popups.is_empty() {
            // A popup stays only while its owner still exists AND still lays
            // out popup content, so a widget that changes kind under a kept
            // name can't leave a stale popup capturing the keys. The owner
            // may be a part, like a menu bar's menu, whose content the
            // part's element lays out.
            let popup_owners: QuickSet<Id> = self
                .targets
                .iter()
                .filter(|target| target.layer >= Layer::Popup)
                .map(|target| target.owner)
                .collect();
            self.popups.retain(|popup| {
                self.targets
                    .iter()
                    .any(|target| target.id == popup.owner && popup_owners.contains(&target.owner))
            });
        }

        // The pointer, on the tree as it's laid out, for the next step's
        // input and the look of what's under it.
        let topmost = self.topmost(&self.targets, self.modal);
        self.hovered = self.under_pointer(topmost);
        self.modal_shown = self.modal;

        // The pointer's look, from the widget it's on, or the one that holds
        // its press, in this tree, so that a frame's cursor is its drawn
        // tree's, like a text field's that a press put under the pointer.
        let hovered = self.hovered.filter(|hovered| self.targets.iter().any(|target| target.id == *hovered && target.part == Part::Whole && target.enabled));
        let held = self.active.map(|(active, _)| active);
        let mut cursor = None;
        let ui: &Interface = self;
        tree.walk(|element| {
            if cursor.is_none()
                && let Some(id) = element.id
                && (hovered == Some(id) || held == Some(id))
            {
                cursor = element.widget.cursor(ui, hovered == Some(id), held == Some(id));
            }
        });
        self.cursor = cursor.unwrap_or(Cursor::Arrow);

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

    fn layout_elements<M: Clone + 'static>(&mut self, elements: &[Element<M>], enabled: bool, layer: Layer, clip: Rect) {
        for element in elements {
            let enabled = enabled && element.enabled;
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
                layer,
                clip,
            };
            element.widget.parts(&mut parts);
            if let Some(items) = &element.context_menu {
                crate::widgets::menu::parts_context_menu(&mut parts, items);
            }
            let children_clip = element.widget.children_clip(&mut parts).map_or(clip, |own| own.intersect(clip));
            self.layout_elements(&element.children, enabled, layer, children_clip);
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
        // The pointer, over everything.
        if self.draws_pointer
            && self.input.focused
            && let Some(at) = self.input.pointer
        {
            let cursor = self.cursor;
            self.draw.set_layer(Layer::Tooltip);
            self.draw.push_clip(Rect::new(0.0, 0.0, self.size.width, self.size.height), true);
            crate::widgets::pointer::draw_pointer(&mut self.painter(), at, cursor);
            self.draw.pop_clip();
        }
    }

    #[allow(clippy::type_complexity)]
    fn draw_elements<M: Clone + 'static>(&mut self, elements: &[Element<M>], enabled: bool, layer: Layer, clip: Rect, sources: &LookSources, due: &Option<(Id, Part, String, Point)>, tooltip: &mut Option<(Tooltip, String, Point)>) {
        for element in elements {
            let enabled = enabled && element.enabled;
            let layer = layer.max(element.widget.layer());
            let clip = if element.widget.layer() > Layer::Base { self.bounds() } else { clip };
            let element_id = element.id.unwrap_or_else(|| cover_id(element.rect));
            let look = sources.look(element_id, enabled);
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
        LookSources {
            hovered: find(self.hovered).map(|target| (target.owner, target.part.clone())),
            active: self.active.and_then(|(active, button)| find(Some(active)).map(|target| (target.owner, target.part.clone(), button))),
            focused: find(self.focused).map(|target| (target.owner, !self.modal || target.layer >= Layer::Dialog)),
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
        let target = targets.iter().rev().find(|target| target.id == hovered && target.enabled);
        match target.map(|target| target.sense.wheel) {
            Some(Wheel::Taken) => true,
            Some(Wheel::WhileFocused) => self.hovered == self.focused,
            _ => false,
        }
    }

    // The target that the pointer is on, and the one with a press, for
    // widgets built of others.
    pub fn hovered_target(&self) -> Option<&Target> {
        let hovered = self.hovered?;
        self.targets.iter().rev().find(|target| target.id == hovered)
    }

    pub fn held_target(&self) -> Option<&Target> {
        let (active, _) = self.active?;
        self.targets.iter().rev().find(|target| target.id == active)
    }

    // The theme, whose roles color everything: the one this step's view is
    // built in, as the step's toggles leave it.
    pub fn theme(&self) -> &Theme {
        self.next_theme.as_ref().unwrap_or(&self.theme)
    }

    // Switches between the light and dark themes, as F9 does, for this
    // step's view on.
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
    // PERF_BAR_HEIGHT pixels down, or at the top without one.
    pub fn content_top(&self) -> f32 {
        if self.performance.is_some() { crate::design::tokens::PERF_BAR_HEIGHT } else { 0.0 }
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

    // Seconds since the last frame, in a frame's last step, and 0 in the
    // steps before it, so that a simulation moves once a frame, after its
    // events.
    pub fn delta(&self) -> f32 {
        self.input.delta
    }

    // Whether this is the frame's last step, which has the frame's delta, after
    // all of the frame's input: for what a program does once a frame, like
    // reading the devices' readings itself (see Startup::devices), which
    // come once a frame, with the window's focus as the frame leaves it.
    pub fn last_step(&self) -> bool {
        self.last_step
    }

    // Whether the window has focus. Without it there's no input at all.
    pub fn window_focused(&self) -> bool {
        self.input.focused
    }

    pub fn pointer(&self) -> Option<Point> {
        self.input.pointer
    }

    // Shift, Ctrl and Alt, as they're held.
    pub fn modifiers(&self) -> Modifiers {
        self.input.modifiers()
    }

    pub fn tooltips_enabled(&self) -> bool {
        self.tooltips_enabled
    }

    // Whether the key was pressed in this step,
    // unless a text field with focus takes it, or a dialog is open over the
    // page.
    pub fn key_pressed(&self, key: Key) -> bool {
        self.input.key_pressed(key) && !self.is_typed(key) && !self.modal_shown
    }

    // Whether the key is down, unless a text field with focus takes it, or a
    // dialog is open. Shift, Ctrl and Alt are always the program's.
    pub fn key_down(&self, key: Key) -> bool {
        self.input.key_down(key) && !(self.typing && !key.is_modifier() && !key.is_function()) && !self.modal_shown
    }

    // Whether the key was pressed with exactly these modifiers, like Ctrl+S.
    pub fn shortcut(&self, modifiers: Modifiers, key: Key) -> bool {
        self.input.modifiers() == modifiers && self.key_pressed(key)
    }

    pub fn shortcut_pressed(&self, shortcut: Shortcut) -> bool {
        self.shortcut(shortcut.modifiers, shortcut.key)
    }

    // Whether the text field with the name has keyboard focus.
    pub fn is_focused(&self, name: &str) -> bool {
        self.focused == Some(id(name))
    }

    // Whether a text field with focus takes the key: every key pressed
    // without Ctrl, but the function keys. Ctrl's shortcuts, like Ctrl+S,
    // and F9 and F10, stay the program's and the window's.
    fn is_typed(&self, key: Key) -> bool {
        self.typing && !key.is_function() && !self.input.modifiers().control
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

    // The targets of the last tree that's laid out.
    pub fn targets(&self) -> &[Target] {
        &self.targets
    }

    // The last tree's listed target with the name: the first of them, as
    // the state lists it, if the name is used twice.
    pub fn target(&self, name: &str) -> Option<&Target> {
        let id = id(name);
        self.targets.iter().find(|target| target.id == id && target.listed)
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

// The id of a cover over the area, which is no widget: a hash of where it
// is, which no name hashes to.
fn cover_id(rect: Rect) -> Id {
    let mut hash: u64 = 0x84222325_cbf29ce4;
    for value in [rect.x, rect.y, rect.width, rect.height] {
        hash ^= value.to_bits() as u64;
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash
}

// The focus's owner among the targets, and whether the keys reach it under
// an open dialog.
fn owner_of_focus(targets: &[Target], modal_shown: bool, focused: Option<Id>) -> Option<(Id, bool)> {
    let focused = focused?;
    targets
        .iter()
        .find(|target| target.id == focused)
        .map(|target| (target.owner, !modal_shown || target.layer >= Layer::Dialog))
}

// Every element with its enabling and its layer, in drawing order.
fn collect<'a, M: 'static>(elements: &'a [Element<M>], enabled: bool, layer: Layer, out: &mut Vec<(&'a Element<M>, bool, Layer)>) {
    for element in elements {
        let enabled = enabled && element.enabled;
        let layer = layer.max(element.widget.layer());
        out.push((element, enabled, layer));
        collect(&element.children, enabled, layer, out);
    }
}

// What a program's `view` and `input` see of the interface: the frame as it
// stands, which they read and don't change, through Interface's getters,
// and the fonts to measure text with.
pub struct Ui<'a> {
    pub(crate) inner: &'a mut Interface,
}

impl Deref for Ui<'_> {
    type Target = Interface;

    fn deref(&self) -> &Interface {
        self.inner
    }
}

impl Ui<'_> {
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

    // What a widget keeps between frames, read for laying the frame out,
    // like how far a scroll area is scrolled.
    pub fn memory<T: Default + Clone + 'static>(&mut self, name: &str) -> T {
        self.inner.memory::<T>(id(name)).clone()
    }
}

// What a program's `update` works with: the interface's state, which it
// reads through Interface's getters and changes with the requests here, and
// where the messages it sends go.
pub struct Context<'a, M> {
    pub(crate) inner: &'a mut Interface,
    pub(crate) queue: &'a mut VecDeque<M>,
}

impl<M> Deref for Context<'_, M> {
    type Target = Interface;

    fn deref(&self) -> &Interface {
        self.inner
    }
}

impl<M> Context<'_, M> {
    // Sends a message, which is applied after those before it, in this step.
    pub fn send(&mut self, message: M) {
        self.queue.push_back(message);
    }

    // Switches between the light and dark themes, as F9 does, from this
    // step's view on.
    pub fn toggle_theme(&mut self) {
        self.inner.toggle_theme();
    }

    // Switches vertical sync, as F10 does, from the next frame on.
    pub fn toggle_vertical_sync(&mut self) {
        self.inner.environment.vertical_sync = !self.inner.environment.vertical_sync;
        self.inner.requests.push(Request::ToggleVerticalSync);
    }

    // Closes the window, whatever Program::may_close says.
    pub fn close(&mut self) {
        self.inner.requests.push(Request::Close);
    }

    // Shows a notice under the app bar.
    pub fn notify(&mut self, notice: Notice) {
        self.inner.notices.push(notice);
    }

    pub fn remove_notice(&mut self, index: usize) {
        if index < self.inner.notices.len() {
            self.inner.notices.remove(index);
        }
    }

    // Whether the tree that the step's input was on had a dialog, which
    // took the input.
    #[allow(clippy::misnamed_getters)]
    pub fn modal(&self) -> bool {
        self.inner.modal_shown
    }

    pub fn close_popups(&mut self) {
        self.inner.popups.clear();
    }
}
