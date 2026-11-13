// The tree that a program's view describes a frame with: elements, each a
// named widget in a rectangle, with the message that the input on it makes,
// and children for the containers, like cards. The program builds a tree
// from its state each frame, and never changes state while it does; the
// interface lays it out, finds what the input did to the tree the user saw,
// and draws the new one.
//
//     let mut tree = Tree::new();
//     tree.add("Save", rect, Button::new("Save").accent().on_press(Message::Save));
//     tree.add("Volume", rect, Slider::new(self.volume, 0.0, 1.0).on_change(Message::Volume));
//
// A widget is a value that implements Widget: what it is to scripts, what
// its targets are, what the input on them means, and how it draws. The
// interface's own widgets are in Widgets/; a program may add its own.

use crate::core::interaction::{Hit, Id, Interaction, Part, Sense, id};
use crate::core::interface::Interface;
use crate::design::theme::Theme;
use crate::drawing::geometry::{Point, Rect, Size};
use crate::drawing::painter::{Layer, Painter};
use crate::drawing::text::{Fonts, TextStyle};
use crate::input::devices::{Key, Modifiers, MouseButton};
use crate::input::events::Input;
use crate::platform::clipboard::Clipboard;
use crate::widgets::menu::MenuItem;

// A line of a tooltip with more than a sentence in it.
#[derive(Clone, Debug, PartialEq)]
pub enum TooltipLine {
    Heading(String),
    Text(String),
    // A label and its value, in two columns.
    Pair(String, String),
    Mono(String),
    Separator,
}

impl TooltipLine {
    pub fn heading(text: impl Into<String>) -> Self {
        TooltipLine::Heading(text.into())
    }

    pub fn text(text: impl Into<String>) -> Self {
        TooltipLine::Text(text.into())
    }

    pub fn pair(label: impl Into<String>, value: impl Into<String>) -> Self {
        TooltipLine::Pair(label.into(), value.into())
    }

    pub fn mono(text: impl Into<String>) -> Self {
        TooltipLine::Mono(text.into())
    }
}

// What a widget says after the pointer rests on it.
#[derive(Clone, Debug, PartialEq)]
pub struct Tooltip {
    pub lines: Vec<TooltipLine>,
    pub max_width: f32,
}

// The widgets of a frame, in the order they're drawn.
pub struct Tree<M> {
    pub(crate) elements: Vec<Element<M>>,
    // Whether tooltips show this frame.
    pub(crate) tooltips: bool,
}

impl<M: 'static> Default for Tree<M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<M: 'static> Tree<M> {
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
            tooltips: true,
        }
    }

    // Adds a widget under the name, in the rectangle, and returns it to
    // say more about, like its tooltip.
    pub fn add(&mut self, name: &str, rect: Rect, widget: impl Widget<M>) -> &mut Element<M> {
        self.push(Element::new(name, rect, widget))
    }

    // Adds a widget that's no target, like text or a drawing, in the
    // rectangle.
    pub fn draw(&mut self, rect: Rect, widget: impl Widget<M>) -> &mut Element<M> {
        self.push(Element::unnamed(rect, widget))
    }

    pub fn push(&mut self, element: Element<M>) -> &mut Element<M> {
        self.elements.push(element);
        self.elements.last_mut().unwrap()
    }

    // Adds a container, like a card, with the children that `build` adds to
    // the tree it's given, which inherit its enabling.
    pub fn container(&mut self, name: Option<&str>, rect: Rect, widget: impl Widget<M>, build: impl FnOnce(&mut Tree<M>)) -> &mut Element<M> {
        let mut children = Tree::new();
        children.tooltips = self.tooltips;
        build(&mut children);
        let mut element = match name {
            Some(name) => Element::new(name, rect, widget),
            None => Element::unnamed(rect, widget),
        };
        element.children = children.elements;
        self.push(element)
    }

    // Whether tooltips show this frame.
    pub fn set_tooltips(&mut self, shown: bool) {
        self.tooltips = shown;
    }

    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }

    pub fn append(&mut self, mut other: Tree<M>) {
        self.elements.append(&mut other.elements);
    }

    // Every element, with its children after it, as they're drawn.
    pub fn walk(&self, mut visit: impl FnMut(&Element<M>)) {
        fn go<M>(elements: &[Element<M>], visit: &mut impl FnMut(&Element<M>)) {
            for element in elements {
                visit(element);
                go(&element.children, visit);
            }
        }
        go(&self.elements, &mut visit);
    }

    // The element with the name, if the tree has one.
    pub fn find(&self, name: &str) -> Option<&Element<M>> {
        let wanted = id(name);
        let mut found = None;
        self.walk(|element| {
            if found.is_none() && element.id == Some(wanted) {
                found = Some(element as *const Element<M>);
            }
        });
        // The pointer is to an element of self, which lives as long as self.
        found.map(|pointer| unsafe { &*pointer })
    }
}

// A widget in a rectangle, with its name, and what it says about itself.
pub struct Element<M> {
    pub(crate) id: Option<Id>,
    pub(crate) name: Option<String>,
    pub(crate) rect: Rect,
    pub(crate) enabled: bool,
    // What it shows, for scripts, if the widget doesn't say.
    pub(crate) label: Option<String>,
    pub(crate) tooltip: Option<Tooltip>,
    // A menu that a right-click on it opens.
    pub(crate) context_menu: Option<Vec<MenuItem<M>>>,
    pub(crate) widget: Box<dyn Widget<M>>,
    pub(crate) children: Vec<Element<M>>,
}

impl<M: 'static> Element<M> {
    pub fn new(name: &str, rect: Rect, widget: impl Widget<M>) -> Self {
        Self {
            id: Some(id(name)),
            name: Some(name.to_string()),
            rect,
            enabled: true,
            label: None,
            tooltip: None,
            context_menu: None,
            widget: Box::new(widget),
            children: Vec::new(),
        }
    }

    // An element that's no target: text, a drawing, or a container that only
    // covers what's under it.
    pub fn unnamed(rect: Rect, widget: impl Widget<M>) -> Self {
        Self {
            id: None,
            name: None,
            ..Self::new("", rect, widget)
        }
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn tooltip(mut self, text: impl Into<String>) -> Self {
        self.tooltip = Some(Tooltip {
            lines: vec![TooltipLine::Text(text.into())],
            max_width: crate::design::controls::TOOLTIP_MAX_WIDTH,
        });
        self
    }

    pub fn tooltip_lines(mut self, lines: Vec<TooltipLine>) -> Self {
        self.tooltip = Some(Tooltip {
            lines,
            max_width: crate::design::controls::TOOLTIP_MAX_WIDTH,
        });
        self
    }

    // A tooltip with more in it than WinUI's 320 pixels hold well, up to
    // `max_width` wide.
    pub fn tooltip_wide(mut self, lines: Vec<TooltipLine>, max_width: f32) -> Self {
        self.tooltip = Some(Tooltip { lines, max_width });
        self
    }

    // A menu that opens at the pointer when it's right-clicked.
    pub fn context_menu(mut self, items: Vec<MenuItem<M>>) -> Self {
        self.context_menu = Some(items);
        self
    }

    pub fn children(mut self, children: Tree<M>) -> Self {
        self.children = children.elements;
        self
    }

    // The same, on an element that's in a tree.
    pub fn set_enabled(&mut self, enabled: bool) -> &mut Self {
        self.enabled = enabled;
        self
    }

    pub fn set_label(&mut self, label: impl Into<String>) -> &mut Self {
        self.label = Some(label.into());
        self
    }

    pub fn set_tooltip(&mut self, text: impl Into<String>) -> &mut Self {
        self.tooltip = Some(Tooltip {
            lines: vec![TooltipLine::Text(text.into())],
            max_width: crate::design::controls::TOOLTIP_MAX_WIDTH,
        });
        self
    }

    pub fn set_tooltip_lines(&mut self, lines: Vec<TooltipLine>) -> &mut Self {
        self.tooltip = Some(Tooltip {
            lines,
            max_width: crate::design::controls::TOOLTIP_MAX_WIDTH,
        });
        self
    }

    pub fn set_tooltip_wide(&mut self, lines: Vec<TooltipLine>, max_width: f32) -> &mut Self {
        self.tooltip = Some(Tooltip { lines, max_width });
        self
    }

    pub fn set_context_menu(&mut self, items: Vec<MenuItem<M>>) -> &mut Self {
        self.context_menu = Some(items);
        self
    }

    pub fn rect(&self) -> Rect {
        self.rect
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

// What a widget is: what it is to scripts, what its targets are, what the
// input on them means, and how it draws. Each is called once a frame, for
// each element that has the widget: `parts` and `draw` for the frame's
// tree, and `interact` for the tree the user saw as the input came, with
// what the input did to it.
pub trait Widget<M>: 'static {
    // Like "Button" or "Slider", as WinUI names its controls.
    fn kind(&self) -> &'static str;

    // What it shows, like a button's text, for scripts and tests.
    fn label(&self) -> Option<String> {
        None
    }

    // What its whole area senses, or None for a widget that the pointer
    // goes through, like text.
    fn sense(&self) -> Option<Sense> {
        None
    }

    // Whether an unnamed element of it still stops the pointer, like a card
    // over a 3D view.
    fn covers(&self) -> bool {
        false
    }

    // Whether `interact` is called every frame, even when nothing happened
    // to its targets, like a 3D view that clears what was under the pointer
    // once it leaves.
    fn always_interacts(&self) -> bool {
        false
    }

    // The layer it and its children draw on: a dialog's, or the base.
    fn layer(&self) -> Layer {
        Layer::Base
    }

    // Registers its parts that are targets of their own, like a list's rows,
    // with `ctx.part`.
    fn parts(&self, _ctx: &mut Parts) {}

    // Whether the wheel that nothing under the pointer takes scrolls this
    // widget, like a scroll area or a list. The deepest such widget under
    // the pointer gets it.
    fn takes_free_wheel(&self) -> bool {
        false
    }

    // Does what the input on it means: sends messages, and changes what it
    // keeps between frames.
    fn interact(&self, _interaction: &Interaction, _ctx: &mut Dispatch<M>) {}

    // The tooltip of one of its parts, like a list's row.
    fn part_tooltip(&self, _part: &Part) -> Option<Tooltip> {
        None
    }

    // Draws it.
    fn draw(&self, _ctx: &mut Draw) {}

    // Where its children are clipped to, if they are, and moved by, like a
    // scroll area's content.
    fn children_clip(&self, _ctx: &mut Parts) -> Option<Rect> {
        None
    }
}

// How an element looks this frame: what the input is doing to it.
#[derive(Clone, Debug, Default)]
pub struct Look {
    pub enabled: bool,
    // The whole element's hit.
    pub hit: Hit,
    // The part the pointer is on, and the part with a press.
    pub hovered_part: Option<Part>,
    pub held_part: Option<Part>,
    pub held_button: Option<MouseButton>,
    // It's a text field with keyboard focus.
    pub focused: bool,
}

impl Look {
    pub fn state(&self) -> crate::core::interaction::ControlState {
        self.hit.state(self.enabled)
    }

    // The state of one of its parts.
    pub fn part_state(&self, part: &Part) -> crate::core::interaction::ControlState {
        let hit = Hit {
            hovered: self.hovered_part.as_ref() == Some(part) && self.enabled,
            held: self.held_part.as_ref() == Some(part),
            held_button: self.held_button,
            ..Hit::default()
        };
        hit.state(self.enabled)
    }

    pub fn hovered(&self, part: &Part) -> bool {
        self.hovered_part.as_ref() == Some(part)
    }

    pub fn held(&self, part: &Part) -> bool {
        self.held_part.as_ref() == Some(part)
    }
}

// What a widget's `parts` works with: its element, the fonts to measure
// with, what it keeps between frames, and the interface's state.
pub struct Parts<'a> {
    pub(crate) ui: &'a mut Interface,
    pub(crate) id: Id,
    pub(crate) name: &'a str,
    pub(crate) rect: Rect,
    pub(crate) enabled: bool,
    pub(crate) layer: Layer,
    pub(crate) clip: Rect,
}

impl Parts<'_> {
    pub fn rect(&self) -> Rect {
        self.rect
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub fn name(&self) -> &str {
        self.name
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn fonts(&mut self) -> &mut Fonts {
        &mut self.ui.fonts
    }

    pub fn measure(&mut self, text: &str, style: TextStyle) -> Size {
        self.ui.fonts.measure(text, style)
    }

    pub fn theme(&self) -> &Theme {
        &self.ui.theme
    }

    pub fn time(&self) -> f64 {
        self.ui.input.time
    }

    pub fn bounds(&self) -> Rect {
        self.ui.bounds()
    }

    // What the widget keeps between frames, by its id.
    pub fn memory<T: Default + 'static>(&mut self) -> &mut T {
        let id = self.id;
        self.ui.memory(id)
    }

    pub fn memory_of<T: Default + 'static>(&mut self, id: Id) -> &mut T {
        self.ui.memory(id)
    }

    pub fn popup_open(&self) -> bool {
        self.ui.popup_open(self.id)
    }

    pub fn popup_open_of(&self, id: Id) -> bool {
        self.ui.popup_open(id)
    }

    pub fn is_focused(&self) -> bool {
        self.ui.focused == Some(self.id)
    }

    pub fn input(&self) -> &Input {
        &self.ui.input
    }

    // Registers a part of the widget as a target, named "<widget>/<suffix>".
    pub fn part(&mut self, part: Part, suffix: &str, rect: Rect, sense: Sense, kind: &'static str, label: Option<String>) {
        let name = format!("{}/{suffix}", self.name);
        let layer = self.layer;
        self.part_on(layer, part, &name, rect, sense, kind, label);
    }

    // Registers a part on a layer, like a popup's items, with its full name.
    #[allow(clippy::too_many_arguments)]
    pub fn part_on(&mut self, layer: Layer, part: Part, name: &str, rect: Rect, sense: Sense, kind: &'static str, label: Option<String>) {
        let enabled = self.enabled;
        self.part_on_enabled(layer, part, name, rect, sense, kind, label, enabled);
    }

    // Like part_on, for a part that's disabled on its own, like a menu item.
    #[allow(clippy::too_many_arguments)]
    pub fn part_on_enabled(&mut self, layer: Layer, part: Part, name: &str, rect: Rect, sense: Sense, kind: &'static str, label: Option<String>, enabled: bool) {
        let clip = if layer > self.layer { self.ui.bounds() } else { self.clip };
        let enabled = enabled && self.enabled;
        self.ui.targets.push(crate::core::interaction::Target {
            id: id(name),
            name: name.to_string(),
            label,
            rect: rect.intersect(clip),
            layer,
            enabled,
            sense,
            kind,
            listed: true,
            owner: self.id,
            part,
        });
    }

    // Covers the area, so that the pointer stops at it, without a target
    // that scripts see.
    pub fn cover_on(&mut self, layer: Layer, rect: Rect) {
        self.ui.cover_on(layer, rect, self.id);
    }
}

// What a widget's `interact` works with: the frame's input, what it keeps
// between frames, the interface's state, and where its messages go.
pub struct Dispatch<'a, M> {
    pub(crate) ui: &'a mut Interface,
    pub(crate) messages: &'a mut Vec<M>,
    pub(crate) id: Id,
    pub(crate) name: &'a str,
    pub(crate) rect: Rect,
    pub(crate) enabled: bool,
}

impl<M> Dispatch<'_, M> {
    pub fn send(&mut self, message: M) {
        self.messages.push(message);
    }

    pub fn rect(&self) -> Rect {
        self.rect
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub fn name(&self) -> &str {
        self.name
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn input(&self) -> &Input {
        &self.ui.input
    }

    pub fn modifiers(&self) -> Modifiers {
        self.ui.input.modifiers()
    }

    pub fn time(&self) -> f64 {
        self.ui.input.time
    }

    pub fn pointer(&self) -> Option<Point> {
        self.ui.input.pointer
    }

    pub fn fonts(&mut self) -> &mut Fonts {
        &mut self.ui.fonts
    }

    pub fn measure(&mut self, text: &str, style: TextStyle) -> Size {
        self.ui.fonts.measure(text, style)
    }

    pub fn clipboard(&mut self) -> &mut Clipboard {
        &mut self.ui.clipboard
    }

    pub fn memory<T: Default + 'static>(&mut self) -> &mut T {
        let id = self.id;
        self.ui.memory(id)
    }

    pub fn memory_of<T: Default + 'static>(&mut self, id: Id) -> &mut T {
        self.ui.memory(id)
    }

    // Whether the key is down, unless a text field with keyboard focus
    // takes it, or a dialog is open.
    pub fn key_down(&self, key: Key) -> bool {
        self.ui.key_down(key)
    }

    // Opens the widget's popup, named `name`, like a combo box's list. A
    // popup whose owner is in an open popup opens over it, like a submenu;
    // any other closes the open ones first.
    pub fn open_popup(&mut self, name: &str) {
        let id = self.id;
        self.ui.open_popup(id, name);
    }

    pub fn open_popup_of(&mut self, owner: Id, name: &str) {
        self.ui.open_popup(owner, name);
    }

    pub fn close_popup(&mut self) {
        let id = self.id;
        self.ui.close_popup(id);
    }

    pub fn close_popup_of(&mut self, owner: Id) {
        self.ui.close_popup(owner);
    }

    pub fn close_popups(&mut self) {
        self.ui.popups.clear();
    }

    pub fn popup_open(&self) -> bool {
        self.ui.popup_open(self.id)
    }

    pub fn popup_open_of(&self, owner: Id) -> bool {
        self.ui.popup_open(owner)
    }

    pub fn any_popup(&self) -> bool {
        !self.ui.popups.is_empty()
    }

    pub fn is_focused(&self) -> bool {
        self.ui.focused == Some(self.id)
    }

    pub fn set_cursor(&mut self, cursor: crate::core::interaction::Cursor) {
        self.ui.cursor = cursor;
    }

    // The name of the target under the pointer, as of the frame the user
    // saw.
    pub fn hovered_name(&self) -> Option<&str> {
        self.ui.hovered_name()
    }

    pub fn hovered_id(&self) -> Option<Id> {
        self.ui.hovered
    }

    // The name of the target with keyboard focus, as this frame's input
    // leaves it.
    pub fn focused_name(&self) -> Option<&str> {
        self.ui.focused_name()
    }

    pub fn bounds(&self) -> Rect {
        self.ui.bounds()
    }

    // The notices under the app bar, for the bar that shows them.
    pub(crate) fn notices_mut(&mut self) -> &mut Vec<crate::application::program::Notice> {
        &mut self.ui.notices
    }
}

// What a widget's `draw` works with: the painter, the theme, how the
// element looks, and what it keeps between frames.
pub struct Draw<'a> {
    pub(crate) ui: &'a mut Interface,
    pub(crate) id: Id,
    pub(crate) name: &'a str,
    pub(crate) rect: Rect,
    pub(crate) look: Look,
    pub(crate) layer: Layer,
}

impl Draw<'_> {
    pub fn painter(&mut self) -> Painter<'_> {
        Painter {
            list: &mut self.ui.draw,
            fonts: &mut self.ui.fonts,
            theme: &self.ui.theme,
        }
    }

    pub fn theme(&self) -> &Theme {
        &self.ui.theme
    }

    pub fn look(&self) -> &Look {
        &self.look
    }

    pub fn state(&self) -> crate::core::interaction::ControlState {
        self.look.state()
    }

    pub fn rect(&self) -> Rect {
        self.rect
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub fn name(&self) -> &str {
        self.name
    }

    pub fn enabled(&self) -> bool {
        self.look.enabled
    }

    // Whether it's a text field with keyboard focus.
    pub fn focused(&self) -> bool {
        self.look.focused
    }

    pub fn layer(&self) -> Layer {
        self.layer
    }

    pub fn time(&self) -> f64 {
        self.ui.input.time
    }

    pub fn bounds(&self) -> Rect {
        self.ui.bounds()
    }

    pub fn input(&self) -> &Input {
        &self.ui.input
    }

    pub fn fonts(&mut self) -> &mut Fonts {
        &mut self.ui.fonts
    }

    pub fn measure(&mut self, text: &str, style: TextStyle) -> Size {
        self.ui.fonts.measure(text, style)
    }

    pub fn memory<T: Default + 'static>(&mut self) -> &mut T {
        let id = self.id;
        self.ui.memory(id)
    }

    pub fn memory_of<T: Default + 'static>(&mut self, id: Id) -> &mut T {
        self.ui.memory(id)
    }

    pub fn popup_open(&self) -> bool {
        self.ui.popup_open(self.id)
    }

    pub fn popup_open_of(&self, id: Id) -> bool {
        self.ui.popup_open(id)
    }

    // The part of another widget that the pointer is on, by that widget's
    // id, for widgets built of others.
    pub fn hovered_target(&self) -> Option<&crate::core::interaction::Target> {
        self.ui.hovered_target()
    }

    pub fn held_target(&self) -> Option<&crate::core::interaction::Target> {
        self.ui.held_target()
    }

    pub fn environment(&self) -> &crate::core::interface::Environment {
        &self.ui.environment
    }

    pub fn window_focused(&self) -> bool {
        self.ui.input.focused
    }

    pub fn stats(&self) -> crate::core::interface::FrameStats {
        self.ui.stats()
    }

    pub fn notices(&self) -> &[crate::application::program::Notice] {
        &self.ui.notices
    }

    // Where the painter's instances are in its layer, to mark a place in
    // it, for drawing something behind what was drawn after.
    pub fn layer_len(&self) -> usize {
        self.ui.draw.layer_len()
    }

    pub fn move_to_back(&mut self, first: usize, middle: usize) {
        self.ui.draw.move_to_back(first, middle);
    }
}
