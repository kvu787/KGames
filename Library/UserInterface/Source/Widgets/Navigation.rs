// Choosing one of a few places or modes: WinUI's SelectorBar, whose selected
// item has the accent's pill under it, and the Windows Community Toolkit's
// Segmented, whose selected segment is raised like a button. Their items are
// targets named "<name>/<label>", and the control is one focus stop whose
// arrows move the selection, as WinUI's are.

use crate::core::control::{Control, NodeContext, UiEventKind, adopted};
use crate::core::ui::{ControlState, Response, Sense, Ui};
use crate::design::controls::{
    BUTTON_ICON_GAP, SELECTOR_BAR_ITEM_PADDING, SELECTOR_BAR_PADDING, SELECTOR_BAR_PILL_HEIGHT, SELECTOR_BAR_PILL_WIDTH,
};
use crate::design::icons::Icon;
use crate::design::tokens::{BODY, CURVE_FAST_OUT_SLOW_IN, DURATION_FAST, DURATION_FASTER, ICON, RADIUS_CONTROL};
use crate::drawing::geometry::{Point, Rect};
use crate::drawing::painter::{Align, Border};
use crate::input::devices::Key;
use crate::widgets::widget::Elevation;

// An item: its label, and maybe an icon before it.
#[derive(Clone, Debug)]
pub struct Choice {
    pub label: String,
    pub icon: Option<Icon>,
}

impl From<&str> for Choice {
    fn from(label: &str) -> Self {
        Choice {
            label: label.to_string(),
            icon: None,
        }
    }
}

impl From<(Icon, &str)> for Choice {
    fn from((icon, label): (Icon, &str)) -> Self {
        Choice {
            label: label.to_string(),
            icon: Some(icon),
        }
    }
}

fn choice_width(ui: &mut Ui, choice: &Choice) -> f32 {
    let icon = if choice.icon.is_some() { ICON + BUTTON_ICON_GAP } else { 0.0 };
    (icon + ui.measure(&choice.label, BODY).width).ceil()
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

pub struct SelectorBar {
    selected: usize,
    items: Vec<Choice>,
    responses: Vec<Response>,
}

impl SelectorBar {
    pub fn new<C: Into<Choice> + Clone>(items: &[C], selected: usize) -> Self {
        Self {
            selected,
            items: items.iter().map(|item| item.clone().into()).collect(),
            responses: Vec::new(),
        }
    }

    pub fn selected(&self) -> usize {
        self.selected
    }

    pub fn set_selected(&mut self, selected: usize) {
        self.selected = selected.min(self.items.len().saturating_sub(1));
    }

    // The width that its items take.
    pub fn width<'a, C: Into<Choice> + Clone>(ui: &mut Ui, items: &[C]) -> f32 {
        items.iter().map(|item| choice_width(ui, &item.clone().into()) + SELECTOR_BAR_ITEM_PADDING.horizontal() + 2.0).sum()
    }

    fn item_rects(&self, ui: &mut Ui, rect: Rect) -> Vec<Rect> {
        let items_area = rect.pad(SELECTOR_BAR_PADDING);
        let mut x = items_area.x;
        self.items
            .iter()
            .map(|item| {
                let width = choice_width(ui, item) + SELECTOR_BAR_ITEM_PADDING.horizontal() + 2.0;
                let item_rect = Rect::new(x, items_area.y, width, items_area.height);
                x += width;
                item_rect
            })
            .collect()
    }
}

impl Control for SelectorBar {
    fn kind(&self) -> &'static str {
        "SelectorBar"
    }

    fn sense(&self) -> Sense {
        Sense::NAVIGATE
    }

    fn label(&self) -> Option<String> {
        self.items.get(self.selected).map(|item| item.label.clone())
    }

    fn adopt(&mut self, previous: Box<dyn Control>) {
        if let Some(previous) = adopted::<SelectorBar>(previous) {
            self.selected = previous.selected.min(self.items.len().saturating_sub(1));
        }
    }

    fn handle(&mut self, ui: &mut Ui, ctx: &NodeContext) {
        if arrows(ui, ctx.response.focused, &mut self.selected, self.items.len()) {
            ctx.emit(ui, UiEventKind::Selected(self.selected));
        }
        self.responses.clear();
        for (index, item_rect) in self.item_rects(ui, ctx.rect).into_iter().enumerate() {
            let item_name = format!("{}/{}", ctx.name, self.items[index].label);
            let item_response = ui.interact(&item_name, item_rect, Sense::CLICK, "SelectorBarItem");
            ui.set_label(&item_name, &self.items[index].label);
            // A press on an item gives the control focus, as its one stop,
            // unless widgets can't take focus here.
            if item_response.pressed && ui.focusable() {
                ui.focus(&ctx.name);
                ui.set_focus_visible(false);
            }
            if item_response.clicked && self.selected != index {
                self.selected = index;
                ctx.emit(ui, UiEventKind::Selected(index));
            }
            self.responses.push(item_response);
        }
    }

    fn draw(&mut self, ui: &mut Ui, ctx: &NodeContext) {
        let responses = self.responses.clone();
        for (index, item_rect) in self.item_rects(ui, ctx.rect).into_iter().enumerate() {
            let item = &self.items[index];
            let selected = self.selected == index;
            let state = responses.get(index).copied().unwrap_or_default().state();
            let item_id = responses.get(index).map(|response| response.id).unwrap_or_default();
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
            // The pill grows from 4 to 16 wide as it appears.
            let pill_width = ui.animate(item_id, 0, if selected { SELECTOR_BAR_PILL_WIDTH } else { SELECTOR_BAR_PILL_WIDTH / 4.0 }, if selected { DURATION_FAST } else { 0.0 }, CURVE_FAST_OUT_SLOW_IN);
            let pill_opacity = ui.animate(item_id, 1, if selected { 1.0 } else { 0.0 }, if selected { DURATION_FAST } else { 0.0 }, CURVE_FAST_OUT_SLOW_IN);
            let content = item_rect.pad(SELECTOR_BAR_ITEM_PADDING);
            let text_line = Rect::new(content.x, content.y, content.width, BODY.line_height);
            let mut painter = ui.painter();
            let mut text_x = content.x;
            if let Some(icon) = item.icon {
                painter.icon(Point::new(text_x + ICON / 2.0, text_line.center().y), icon, ICON * 0.8, color);
                text_x += ICON + BUTTON_ICON_GAP;
            }
            painter.text_in(Rect::new(text_x, text_line.y, content.right() - text_x, text_line.height), &item.label, BODY, color, Align::Start);
            if pill_opacity > 0.0 {
                let pill = Rect::new(item_rect.center().x - pill_width / 2.0, text_line.bottom() + 3.0, pill_width, SELECTOR_BAR_PILL_HEIGHT);
                painter.rounded(pill, 1.0, pill_color.fade(pill_opacity));
            }
            if selected {
                ui.focus_visual_around(ctx.response.focused, item_rect, RADIUS_CONTROL, crate::drawing::geometry::Insets::all(2.0));
            }
        }
    }
}

pub struct Segmented {
    selected: usize,
    items: Vec<Choice>,
    responses: Vec<Response>,
}

impl Segmented {
    pub fn new<C: Into<Choice> + Clone>(items: &[C], selected: usize) -> Self {
        Self {
            selected,
            items: items.iter().map(|item| item.clone().into()).collect(),
            responses: Vec::new(),
        }
    }

    pub fn selected(&self) -> usize {
        self.selected
    }

    pub fn set_selected(&mut self, selected: usize) {
        self.selected = selected.min(self.items.len().saturating_sub(1));
    }

    fn segment(&self, rect: Rect, index: usize) -> Rect {
        let inner = rect.inset(2.0);
        let width = inner.width / self.items.len().max(1) as f32;
        Rect::new(inner.x + width * index as f32, inner.y, width, inner.height).round()
    }
}

impl Control for Segmented {
    fn kind(&self) -> &'static str {
        "Segmented"
    }

    fn sense(&self) -> Sense {
        Sense::NAVIGATE
    }

    fn label(&self) -> Option<String> {
        self.items.get(self.selected).map(|item| item.label.clone())
    }

    fn adopt(&mut self, previous: Box<dyn Control>) {
        if let Some(previous) = adopted::<Segmented>(previous) {
            self.selected = previous.selected.min(self.items.len().saturating_sub(1));
        }
    }

    fn handle(&mut self, ui: &mut Ui, ctx: &NodeContext) {
        if arrows(ui, ctx.response.focused, &mut self.selected, self.items.len()) {
            ctx.emit(ui, UiEventKind::Selected(self.selected));
        }
        self.responses.clear();
        for index in 0..self.items.len() {
            let segment = self.segment(ctx.rect, index);
            let item_name = format!("{}/{}", ctx.name, self.items[index].label);
            let item_response = ui.interact(&item_name, segment, Sense::CLICK, "SegmentedItem");
            ui.set_label(&item_name, &self.items[index].label);
            // A press on an item gives the control focus, as its one stop,
            // unless widgets can't take focus here.
            if item_response.pressed && ui.focusable() {
                ui.focus(&ctx.name);
                ui.set_focus_visible(false);
            }
            if item_response.clicked && self.selected != index {
                self.selected = index;
                ctx.emit(ui, UiEventKind::Selected(index));
            }
            self.responses.push(item_response);
        }
    }

    fn draw(&mut self, ui: &mut Ui, ctx: &NodeContext) {
        let rect = ctx.rect;
        let count = self.items.len().max(1);
        {
            let mut painter = ui.painter();
            let theme = painter.theme();
            painter.shape(rect, RADIUS_CONTROL, theme.control_alt_fill_secondary, Some(Border::new(theme.control_stroke, 1.0)));
        }
        let inner = rect.inset(2.0);
        let segment_width = inner.width / count as f32;
        // The raised segment slides to the selected one.
        let selected_x = ui.animate(ctx.response.id, 0, inner.x + segment_width * self.selected as f32, DURATION_FAST, CURVE_FAST_OUT_SLOW_IN);
        {
            let raised = Rect::new(selected_x, inner.y, segment_width, inner.height).round();
            let mut painter = ui.painter();
            let theme = painter.theme();
            let fill = if ctx.response.enabled { theme.control_fill } else { theme.control_fill_disabled };
            painter.shape(raised, RADIUS_CONTROL - 1.0, fill, Some(Elevation::Control.border(theme, raised)));
        }
        let responses = self.responses.clone();
        for (index, item) in self.items.iter().enumerate() {
            let segment = self.segment(rect, index);
            let selected = self.selected == index;
            let response = responses.get(index).copied().unwrap_or_default();
            let state = response.state();
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
            let hover = ui.animate_color(response.id, 0, hover, DURATION_FASTER);
            let mut painter = ui.painter();
            painter.rounded(segment.inset(1.0), RADIUS_CONTROL - 1.0, hover);
            let width = item.icon.map_or(0.0, |_| ICON + if item.label.is_empty() { 0.0 } else { BUTTON_ICON_GAP })
                + if item.label.is_empty() { 0.0 } else { painter.measure(&item.label, BODY).width };
            let mut x = (segment.center().x - width / 2.0).max(segment.x + 4.0).round();
            if let Some(icon) = item.icon {
                painter.icon(Point::new(x + ICON / 2.0, segment.center().y), icon, ICON, color);
                x += ICON + BUTTON_ICON_GAP;
            }
            if !item.label.is_empty() {
                painter.text_fit(Rect::new(x, segment.y, segment.right() - x - 4.0, segment.height), &item.label, BODY, color, Align::Start);
            }
            if selected {
                let pill = Rect::new(segment.center().x - 8.0, segment.bottom() - 4.0, 16.0, 3.0);
                painter.rounded(pill, 1.5, accent);
                ui.focus_visual_if(ctx.response.focused, segment, RADIUS_CONTROL);
            }
        }
    }
}
