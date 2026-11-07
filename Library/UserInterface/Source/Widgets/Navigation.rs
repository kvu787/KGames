// Choosing one of a few places or modes: WinUI's SelectorBar, whose selected
// item has the accent's pill under it, and the Windows Community Toolkit's
// Segmented, whose selected segment is raised like a button. Their items are
// targets named "<name>/<label>", and the control is one focus stop whose
// arrows move the selection, as WinUI's are.

use crate::core::interaction::{ControlState, Response, Sense};
use crate::core::ui::Ui;
use crate::design::controls::{
    BUTTON_ICON_GAP, SELECTOR_BAR_ITEM_PADDING, SELECTOR_BAR_PADDING, SELECTOR_BAR_PILL_HEIGHT, SELECTOR_BAR_PILL_WIDTH,
};
use crate::design::icons::Icon;
use crate::design::tokens::{BODY, ICON, RADIUS_CONTROL};
use crate::drawing::geometry::{Point, Rect};
use crate::drawing::painter::{Align, Border};
use crate::input::devices::Key;
use crate::widgets::widget::{Elevation, Widget};

// An item: its label, and maybe an icon before it.
#[derive(Clone, Copy, Debug)]
pub struct Choice<'a> {
    pub label: &'a str,
    pub icon: Option<Icon>,
}

impl<'a> From<&'a str> for Choice<'a> {
    fn from(label: &'a str) -> Self {
        Choice { label, icon: None }
    }
}

impl<'a> From<(Icon, &'a str)> for Choice<'a> {
    fn from((icon, label): (Icon, &'a str)) -> Self {
        Choice { label, icon: Some(icon) }
    }
}

fn choice_width(ui: &mut Ui, choice: &Choice) -> f32 {
    let icon = if choice.icon.is_some() { ICON + BUTTON_ICON_GAP } else { 0.0 };
    (icon + ui.measure(choice.label, BODY).width).ceil()
}

// The arrows move the selection of a control with focus.
fn arrows(ui: &Ui, focused: bool, selected: &mut usize, count: usize) -> bool {
    if !focused || count == 0 {
        return false;
    }
    let input = ui.input();
    let before = *selected;
    if input.key_pressed(Key::RIGHT) || input.key_pressed(Key::DOWN) {
        *selected = (*selected + 1).min(count - 1);
    }
    if input.key_pressed(Key::LEFT) || input.key_pressed(Key::UP) {
        *selected = selected.saturating_sub(1);
    }
    if input.key_pressed(Key::HOME) {
        *selected = 0;
    }
    if input.key_pressed(Key::END) {
        *selected = count - 1;
    }
    *selected != before
}

pub struct SelectorBar<'a, 'v> {
    selected: &'v mut usize,
    items: Vec<Choice<'a>>,
}

impl<'a, 'v> SelectorBar<'a, 'v> {
    pub fn new<C: Into<Choice<'a>> + Copy>(selected: &'v mut usize, items: &[C]) -> Self {
        Self {
            selected,
            items: items.iter().map(|item| (*item).into()).collect(),
        }
    }

    // The width that its items take.
    pub fn width<C: Into<Choice<'a>> + Copy>(ui: &mut Ui, items: &[C]) -> f32 {
        items.iter().map(|item| choice_width(ui, &(*item).into()) + SELECTOR_BAR_ITEM_PADDING.horizontal() + 2.0).sum()
    }
}

impl Widget for SelectorBar<'_, '_> {
    type Response = Response;

    fn show(self, ui: &mut Ui, name: &str, rect: Rect) -> Response {
        let mut response = ui.interact(name, rect, Sense::NAVIGATE, "SelectorBar");
        let count = self.items.len();
        response.changed |= arrows(ui, response.focused, self.selected, count);
        // The items' input first, and then their drawing, so that a click
        // moves the pill on the frame it's made, on every item at once.
        let items_area = rect.pad(SELECTOR_BAR_PADDING);
        let mut x = items_area.x;
        let mut items = Vec::with_capacity(count);
        for (index, item) in self.items.iter().enumerate() {
            let width = choice_width(ui, item) + SELECTOR_BAR_ITEM_PADDING.horizontal() + 2.0;
            let item_rect = Rect::new(x, items_area.y, width, items_area.height);
            x += width;
            let item_name = format!("{name}/{}", item.label);
            let item_response = ui.interact(&item_name, item_rect, Sense::CLICK, "SelectorBarItem");
            ui.set_label(&item_name, item.label);
            // A press on an item gives the control focus, as its one stop,
            // unless widgets can't take focus here.
            if item_response.pressed && ui.focusable() {
                ui.focus(name);
                ui.set_focus_visible(false);
            }
            if item_response.clicked && *self.selected != index {
                *self.selected = index;
                response.changed = true;
            }
            items.push((item_rect, item_response));
        }
        for (index, (item, (item_rect, item_response))) in self.items.iter().zip(items).enumerate() {
            let selected = *self.selected == index;
            let state = item_response.state();
            let theme = ui.theme();
            // SelectorBarItem's text: secondary with the pointer over it, and
            // tertiary pressed, unless it's selected.
            let color = match (selected, state) {
                (_, ControlState::Disabled) => theme.text_disabled,
                (false, ControlState::Pressed) => theme.text_tertiary,
                (_, ControlState::PointerOver) | (true, ControlState::Pressed) => theme.text_secondary,
                _ => theme.text_primary,
            };
            let pill_color = if state == ControlState::Disabled { theme.accent_disabled } else { theme.accent };
            let content = item_rect.pad(SELECTOR_BAR_ITEM_PADDING);
            let text_line = Rect::new(content.x, content.y, content.width, BODY.line_height);
            let mut painter = ui.painter();
            let mut text_x = content.x;
            if let Some(icon) = item.icon {
                painter.icon(Point::new(text_x + ICON / 2.0, text_line.center().y), icon, ICON * 0.8, color);
                text_x += ICON + BUTTON_ICON_GAP;
            }
            painter.text_in(Rect::new(text_x, text_line.y, content.right() - text_x, text_line.height), item.label, BODY, color, Align::Start);
            if selected {
                let pill = Rect::new(item_rect.center().x - SELECTOR_BAR_PILL_WIDTH / 2.0, text_line.bottom() + 3.0, SELECTOR_BAR_PILL_WIDTH, SELECTOR_BAR_PILL_HEIGHT);
                painter.rounded(pill, 1.0, pill_color);
                ui.focus_visual_around(response.focused, item_rect, RADIUS_CONTROL, crate::drawing::geometry::Insets::all(2.0));
            }
        }
        if let Some(item) = self.items.get(*self.selected) {
            ui.set_label(name, item.label);
        }
        response
    }
}

pub struct Segmented<'a, 'v> {
    selected: &'v mut usize,
    items: Vec<Choice<'a>>,
}

impl<'a, 'v> Segmented<'a, 'v> {
    pub fn new<C: Into<Choice<'a>> + Copy>(selected: &'v mut usize, items: &[C]) -> Self {
        Self {
            selected,
            items: items.iter().map(|item| (*item).into()).collect(),
        }
    }
}

impl Widget for Segmented<'_, '_> {
    type Response = Response;

    fn show(self, ui: &mut Ui, name: &str, rect: Rect) -> Response {
        let mut response = ui.interact(name, rect, Sense::NAVIGATE, "Segmented");
        let count = self.items.len().max(1);
        response.changed |= arrows(ui, response.focused, self.selected, self.items.len());
        let inner = rect.inset(2.0);
        let segment_width = inner.width / count as f32;
        let segment_at = move |index: usize| Rect::new(inner.x + segment_width * index as f32, inner.y, segment_width, inner.height).round();
        // The items' input first, and then the drawing, so that a click
        // raises its segment on the frame it's made.
        let mut responses = Vec::with_capacity(self.items.len());
        for (index, item) in self.items.iter().enumerate() {
            let item_name = format!("{name}/{}", item.label);
            let item_response = ui.interact(&item_name, segment_at(index), Sense::CLICK, "SegmentedItem");
            ui.set_label(&item_name, item.label);
            // A press on an item gives the control focus, as its one stop,
            // unless widgets can't take focus here.
            if item_response.pressed && ui.focusable() {
                ui.focus(name);
                ui.set_focus_visible(false);
            }
            if item_response.clicked && *self.selected != index {
                *self.selected = index;
                response.changed = true;
            }
            responses.push(item_response);
        }
        {
            let mut painter = ui.painter();
            let theme = painter.theme();
            painter.shape(rect, RADIUS_CONTROL, theme.control_alt_fill_secondary, Some(Border::new(theme.control_stroke, 1.0)));
            let raised = segment_at(*self.selected);
            let fill = if response.enabled { theme.control_fill } else { theme.control_fill_disabled };
            painter.shape(raised, RADIUS_CONTROL - 1.0, fill, Some(Elevation::Control.border(theme, raised)));
        }
        for (index, (item, item_response)) in self.items.iter().zip(responses).enumerate() {
            let segment = segment_at(index);
            let selected = *self.selected == index;
            let state = item_response.state();
            let theme = ui.theme();
            let hover = match (selected, state) {
                (false, ControlState::PointerOver) => theme.subtle_fill_secondary,
                (false, ControlState::Pressed) => theme.subtle_fill_tertiary,
                _ => theme.subtle_fill_transparent,
            };
            let color = match state {
                ControlState::Disabled => theme.text_disabled,
                ControlState::Pressed => theme.text_secondary,
                _ => theme.text_primary,
            };
            let accent = theme.accent;
            let mut painter = ui.painter();
            painter.rounded(segment.inset(1.0), RADIUS_CONTROL - 1.0, hover);
            let width = item.icon.map_or(0.0, |_| ICON + if item.label.is_empty() { 0.0 } else { BUTTON_ICON_GAP })
                + if item.label.is_empty() { 0.0 } else { painter.measure(item.label, BODY).width };
            let mut x = (segment.center().x - width / 2.0).max(segment.x + 4.0).round();
            if let Some(icon) = item.icon {
                painter.icon(Point::new(x + ICON / 2.0, segment.center().y), icon, ICON, color);
                x += ICON + BUTTON_ICON_GAP;
            }
            if !item.label.is_empty() {
                painter.text_fit(Rect::new(x, segment.y, segment.right() - x - 4.0, segment.height), item.label, BODY, color, Align::Start);
            }
            if selected {
                let pill = Rect::new(segment.center().x - 8.0, segment.bottom() - 4.0, 16.0, 3.0);
                painter.rounded(pill, 1.5, accent);
                ui.focus_visual_if(response.focused, segment, RADIUS_CONTROL);
            }
        }
        if let Some(item) = self.items.get(*self.selected) {
            ui.set_label(name, item.label);
        }
        response
    }
}
