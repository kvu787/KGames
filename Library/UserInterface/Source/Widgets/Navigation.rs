// Choosing one of a few places or modes: WinUI's SelectorBar, whose selected
// item has the accent's pill under it, and the Windows Community Toolkit's
// Segmented, whose selected segment is raised like a button. Their items are
// targets named "<name>/<label>", and the control is one focus stop whose
// arrows move the selection, as WinUI's are.

use crate::core::interaction::{ControlState, Interaction, Part, Sense};
use crate::core::interface::Ui;
use crate::core::tree::{Dispatch, Draw, Parts, Widget};
use crate::design::controls::{BUTTON_ICON_GAP, SELECTOR_BAR_ITEM_PADDING, SELECTOR_BAR_PADDING, SELECTOR_BAR_PILL_HEIGHT, SELECTOR_BAR_PILL_WIDTH};
use crate::design::icons::Icon;
use crate::design::tokens::{BODY, ICON, RADIUS_CONTROL};
use crate::drawing::geometry::{Insets, Point, Rect};
use crate::drawing::painter::{Align, Border};
use crate::drawing::text::Fonts;
use crate::input::devices::Key;
use crate::widgets::widget::{Elevation, Reaction, react};

// An item: its label, and maybe an icon before it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Choice {
    pub label: String,
    pub icon: Option<Icon>,
}

impl From<&str> for Choice {
    fn from(label: &str) -> Self {
        Choice { label: label.to_string(), icon: None }
    }
}

impl From<String> for Choice {
    fn from(label: String) -> Self {
        Choice { label, icon: None }
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

fn choice_width(fonts: &mut Fonts, choice: &Choice) -> f32 {
    let icon = if choice.icon.is_some() { ICON + BUTTON_ICON_GAP } else { 0.0 };
    (icon + fonts.measure(&choice.label, BODY).width).ceil()
}

// The arrows move the selection of a control with focus.
fn arrows<M>(ctx: &Dispatch<M>, focused: bool, selected: usize, count: usize) -> Option<usize> {
    if !focused || count == 0 {
        return None;
    }
    let input = ctx.input();
    let mut moved = selected;
    if input.key_pressed(Key::RIGHT) || input.key_pressed(Key::DOWN) {
        moved = (moved + 1).min(count - 1);
    }
    if input.key_pressed(Key::LEFT) || input.key_pressed(Key::UP) {
        moved = moved.saturating_sub(1);
    }
    if input.key_pressed(Key::HOME) {
        moved = 0;
    }
    if input.key_pressed(Key::END) {
        moved = count - 1;
    }
    (moved != selected).then_some(moved)
}

// The items' input. A press on one gives the control focus, as its one
// stop, unless widgets can't take focus here, and a click selects it.
fn items_input<M>(interaction: &Interaction, ctx: &mut Dispatch<M>, items: &[Choice], selected: usize) -> Option<usize> {
    let mut chosen = None;
    for (index, item) in items.iter().enumerate() {
        let part = Part::Named(item.label.clone());
        let Some(hit) = interaction.hit(&part) else { continue };
        if hit.pressed && ctx.focusable() {
            ctx.focus();
        }
        if hit.clicked && selected != index {
            chosen = Some(index);
        }
    }
    chosen
}

pub struct SelectorBar<M> {
    selected: usize,
    items: Vec<Choice>,
    on_select: Option<Reaction<usize, M>>,
}

impl<M: Clone + 'static> SelectorBar<M> {
    pub fn new<C: Into<Choice> + Clone>(selected: usize, items: &[C]) -> Self {
        Self {
            selected,
            items: items.iter().map(|item| item.clone().into()).collect(),
            on_select: None,
        }
    }

    // What selecting an item sends, with its index.
    pub fn on_select(mut self, reaction: impl Fn(usize) -> M + 'static) -> Self {
        self.on_select = Some(react(reaction));
        self
    }

    // The width that its items take.
    pub fn width<C: Into<Choice> + Clone>(ui: &mut Ui, items: &[C]) -> f32 {
        items.iter().map(|item| choice_width(ui.fonts(), &item.clone().into()) + SELECTOR_BAR_ITEM_PADDING.horizontal() + 2.0).sum()
    }

    // Each item's rectangle, from the bar's left.
    fn item_rects(&self, fonts: &mut Fonts, rect: Rect) -> Vec<Rect> {
        let items_area = rect.pad(SELECTOR_BAR_PADDING);
        let mut x = items_area.x;
        self.items
            .iter()
            .map(|item| {
                let width = choice_width(fonts, item) + SELECTOR_BAR_ITEM_PADDING.horizontal() + 2.0;
                let item_rect = Rect::new(x, items_area.y, width, items_area.height);
                x += width;
                item_rect
            })
            .collect()
    }
}

impl<M: Clone + 'static> Widget<M> for SelectorBar<M> {
    fn kind(&self) -> &'static str {
        "SelectorBar"
    }

    fn label(&self) -> Option<String> {
        self.items.get(self.selected).map(|item| item.label.clone())
    }

    fn sense(&self) -> Option<Sense> {
        Some(Sense::NAVIGATE)
    }

    fn parts(&self, ctx: &mut Parts) {
        let rect = ctx.rect();
        let rects = self.item_rects(ctx.fonts(), rect);
        for (item, item_rect) in self.items.iter().zip(rects) {
            ctx.part(Part::Named(item.label.clone()), &item.label, item_rect, Sense::CLICK, "SelectorBarItem", Some(item.label.clone()));
        }
    }

    fn interact(&self, interaction: &Interaction, ctx: &mut Dispatch<M>) {
        let mut chosen = arrows(ctx, interaction.focused, self.selected, self.items.len());
        if let Some(index) = items_input(interaction, ctx, &self.items, self.selected) {
            chosen = Some(index);
        }
        if let Some(index) = chosen
            && let Some(reaction) = &self.on_select
        {
            ctx.send(reaction(index));
        }
    }

    fn draw(&self, ctx: &mut Draw) {
        let rect = ctx.rect();
        let rects = self.item_rects(ctx.fonts(), rect);
        let look = ctx.look().clone();
        for (index, (item, item_rect)) in self.items.iter().zip(rects).enumerate() {
            let part = Part::Named(item.label.clone());
            let selected = self.selected == index;
            let state = look.part_state(&part);
            {
                let mut painter = ctx.painter();
                let theme = painter.theme();
                // SelectorBarItem's text: secondary with the pointer over it,
                // and tertiary pressed, unless it's selected.
                let color = match (selected, state) {
                    (_, ControlState::Disabled) => theme.text_disabled,
                    (false, ControlState::Pressed) => theme.text_tertiary,
                    (_, ControlState::PointerOver) | (true, ControlState::Pressed) => theme.text_secondary,
                    _ => theme.text_primary,
                };
                let pill_color = if state == ControlState::Disabled { theme.accent_disabled } else { theme.accent };
                let content = item_rect.pad(SELECTOR_BAR_ITEM_PADDING);
                let text_line = Rect::new(content.x, content.y, content.width, BODY.line_height);
                let mut text_x = content.x;
                if let Some(icon) = item.icon {
                    painter.icon(Point::new(text_x + ICON / 2.0, text_line.center().y), icon, ICON * 0.8, color);
                    text_x += ICON + BUTTON_ICON_GAP;
                }
                painter.text_in(Rect::new(text_x, text_line.y, content.right() - text_x, text_line.height), &item.label, BODY, color, Align::Start);
                if selected {
                    let pill = Rect::new(item_rect.center().x - SELECTOR_BAR_PILL_WIDTH / 2.0, text_line.bottom() + 3.0, SELECTOR_BAR_PILL_WIDTH, SELECTOR_BAR_PILL_HEIGHT);
                    painter.rounded(pill, 1.0, pill_color);
                }
            }
            if selected {
                ctx.focus_visual_around(look.focused, item_rect, RADIUS_CONTROL, Insets::all(2.0));
            }
        }
    }
}

pub struct Segmented<M> {
    selected: usize,
    items: Vec<Choice>,
    on_select: Option<Reaction<usize, M>>,
}

impl<M: Clone + 'static> Segmented<M> {
    pub fn new<C: Into<Choice> + Clone>(selected: usize, items: &[C]) -> Self {
        Self {
            selected,
            items: items.iter().map(|item| item.clone().into()).collect(),
            on_select: None,
        }
    }

    pub fn on_select(mut self, reaction: impl Fn(usize) -> M + 'static) -> Self {
        self.on_select = Some(react(reaction));
        self
    }

    fn segment(&self, rect: Rect, index: usize) -> Rect {
        let count = self.items.len().max(1);
        let inner = rect.inset(2.0);
        let segment_width = inner.width / count as f32;
        Rect::new(inner.x + segment_width * index as f32, inner.y, segment_width, inner.height).round()
    }
}

impl<M: Clone + 'static> Widget<M> for Segmented<M> {
    fn kind(&self) -> &'static str {
        "Segmented"
    }

    fn label(&self) -> Option<String> {
        self.items.get(self.selected).map(|item| item.label.clone())
    }

    fn sense(&self) -> Option<Sense> {
        Some(Sense::NAVIGATE)
    }

    fn parts(&self, ctx: &mut Parts) {
        let rect = ctx.rect();
        for (index, item) in self.items.iter().enumerate() {
            ctx.part(Part::Named(item.label.clone()), &item.label, self.segment(rect, index), Sense::CLICK, "SegmentedItem", Some(item.label.clone()));
        }
    }

    fn interact(&self, interaction: &Interaction, ctx: &mut Dispatch<M>) {
        let mut chosen = arrows(ctx, interaction.focused, self.selected, self.items.len());
        if let Some(index) = items_input(interaction, ctx, &self.items, self.selected) {
            chosen = Some(index);
        }
        if let Some(index) = chosen
            && let Some(reaction) = &self.on_select
        {
            ctx.send(reaction(index));
        }
    }

    fn draw(&self, ctx: &mut Draw) {
        let rect = ctx.rect();
        let look = ctx.look().clone();
        let enabled = ctx.enabled();
        {
            let mut painter = ctx.painter();
            let theme = painter.theme();
            painter.shape(rect, RADIUS_CONTROL, theme.control_alt_fill_secondary, Some(Border::new(theme.control_stroke, 1.0)));
            let raised = self.segment(rect, self.selected);
            let fill = if enabled { theme.control_fill } else { theme.control_fill_disabled };
            painter.shape(raised, RADIUS_CONTROL - 1.0, fill, Some(Elevation::Control.border(theme, raised)));
        }
        for (index, item) in self.items.iter().enumerate() {
            let part = Part::Named(item.label.clone());
            let segment = self.segment(rect, index);
            let selected = self.selected == index;
            let state = look.part_state(&part);
            {
                let mut painter = ctx.painter();
                let theme = painter.theme();
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
                }
            }
            if selected {
                ctx.focus_visual_if(look.focused, segment, RADIUS_CONTROL);
            }
        }
    }
}
