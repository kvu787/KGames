// WinUI's ComboBox: a button that shows the chosen item and a chevron, whose
// list opens over it as a popup, with the chosen item where the combo box's
// text is, as WinUI places it. Items are named "<name>/<index>", labeled with
// their text, and the combo box's own label is the chosen item. A press
// outside the list closes it.

use crate::core::interaction::{ControlState, Interaction, Part, Sense};
use crate::core::interface::Ui;
use crate::core::tree::{Dispatch, Draw, Parts, Widget};
use crate::design::controls::{
    COMBO_BOX_CHEVRON_COLUMN, COMBO_BOX_CHEVRON_MARGIN, COMBO_BOX_CHEVRON_SIZE, COMBO_BOX_ITEM_HEIGHT, COMBO_BOX_ITEM_MARGIN, COMBO_BOX_ITEM_PADDING, COMBO_BOX_ITEM_RADIUS, COMBO_BOX_MIN_WIDTH,
    COMBO_BOX_PADDING, COMBO_BOX_POPUP_MAX_ITEMS, COMBO_BOX_POPUP_MIN_WIDTH, COMBO_BOX_POPUP_PADDING, SELECTION_PILL_LENGTH,
    SELECTION_PILL_PRESSED_LENGTH, SELECTION_PILL_THICKNESS,
};
use crate::design::icons::Icon;
use crate::design::tokens::{BODY, ELEVATION_FLYOUT, RADIUS_CONTROL, RADIUS_OVERLAY};
use crate::drawing::geometry::{Point, Rect};
use crate::drawing::painter::{Align, Border, Layer};
use crate::widgets::widget::{Elevation, Reaction, StateColors, react};

pub struct ComboBox<M> {
    items: Vec<String>,
    selected: usize,
    placeholder: String,
    on_select: Option<Reaction<usize, M>>,
}

// Which item is at the top of the open list, which the wheel scrolls.
#[derive(Default, Clone, Copy)]
struct ComboState {
    first: usize,
}

impl<M: Clone + 'static> ComboBox<M> {
    pub fn new(selected: usize, items: &[impl AsRef<str>]) -> Self {
        Self {
            items: items.iter().map(|item| item.as_ref().to_string()).collect(),
            selected,
            placeholder: String::new(),
            on_select: None,
        }
    }

    // What choosing an item sends, with its index.
    pub fn on_select(mut self, reaction: impl Fn(usize) -> M + 'static) -> Self {
        self.on_select = Some(react(reaction));
        self
    }

    // What it shows while no item is selected, like with no items.
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    // How wide a combo box is for its widest item: ComboBoxPadding around
    // it, and the chevron's column.
    pub fn width(ui: &mut Ui, items: &[impl AsRef<str>]) -> f32 {
        let widest = items.iter().map(|item| ui.measure(item.as_ref(), BODY).width).fold(0.0, f32::max).ceil();
        (COMBO_BOX_PADDING.left + widest + COMBO_BOX_CHEVRON_COLUMN).max(COMBO_BOX_MIN_WIDTH)
    }

    fn chosen(&self) -> Option<&str> {
        self.items.get(self.selected).map(String::as_str)
    }

    // The list's panel and its first item's slot, with the chosen item over
    // the combo box's text and the rest around it, as WinUI opens it, inside
    // the window.
    fn list(&self, bounds: Rect, rect: Rect, first: usize) -> (Rect, Rect) {
        let count = self.items.len();
        let shown = count.min(COMBO_BOX_POPUP_MAX_ITEMS);
        let width = rect.width.max(COMBO_BOX_POPUP_MIN_WIDTH);
        let height = COMBO_BOX_POPUP_PADDING.vertical() + shown as f32 * COMBO_BOX_ITEM_HEIGHT;
        let offset = self.selected.saturating_sub(first).min(shown.saturating_sub(1)) as f32;
        let mut y = rect.y - COMBO_BOX_POPUP_PADDING.top - offset * COMBO_BOX_ITEM_HEIGHT;
        y = y.clamp(bounds.y, (bounds.bottom() - height).max(bounds.y));
        let panel = Rect::new(rect.x, y.round(), width, height);
        let first_slot = Rect::new(panel.x, panel.y + COMBO_BOX_POPUP_PADDING.top, panel.width, COMBO_BOX_ITEM_HEIGHT);
        (panel, first_slot)
    }

    fn shown_count(&self) -> usize {
        self.items.len().min(COMBO_BOX_POPUP_MAX_ITEMS)
    }
}

// Opens the list, as the popup named after the combo box.
fn open_list(ctx: &mut Dispatch<impl Clone + 'static>, selected: usize, count: usize, name: &str) {
    ctx.open_popup(name);
    let state = ctx.memory::<ComboState>();
    state.first = selected.saturating_sub(COMBO_BOX_POPUP_MAX_ITEMS / 2).min(count.saturating_sub(COMBO_BOX_POPUP_MAX_ITEMS));
}

impl<M: Clone + 'static> Widget<M> for ComboBox<M> {
    fn kind(&self) -> &'static str {
        "ComboBox"
    }

    fn label(&self) -> Option<String> {
        Some(self.chosen().unwrap_or(&self.placeholder).to_string())
    }

    fn sense(&self) -> Option<Sense> {
        Some(Sense::CLICK)
    }

    fn parts(&self, ctx: &mut Parts) {
        if !ctx.popup_open() || self.items.is_empty() {
            return;
        }
        let state = *ctx.memory::<ComboState>();
        let (panel, first_slot) = self.list(ctx.bounds(), ctx.rect(), state.first);
        let list_name = format!("{}/List", ctx.name());
        ctx.part_on(Layer::Popup, Part::Named("List".to_string()), &list_name, panel, Sense::NONE, "ComboBoxList", None);
        for row in 0..self.shown_count() {
            let index = state.first + row;
            let Some(item) = self.items.get(index) else { break };
            let slot = Rect::new(first_slot.x, first_slot.y + row as f32 * COMBO_BOX_ITEM_HEIGHT, first_slot.width, first_slot.height);
            let item_rect = slot.pad(COMBO_BOX_ITEM_MARGIN);
            let name = format!("{}/{index}", ctx.name());
            ctx.part_on(Layer::Popup, Part::Index(index), &name, item_rect, Sense::CLICK.takes_wheel(), "ComboBoxItem", Some(item.clone()));
        }
    }

    fn interact(&self, interaction: &Interaction, ctx: &mut Dispatch<M>) {
        let count = self.items.len();
        let hit = interaction.whole();
        let name = ctx.name().to_string();
        let mut open = ctx.popup_open();
        // The list can't show nothing, like after its last item went while
        // it was open.
        if open && count == 0 {
            ctx.close_popup();
            open = false;
        }
        let mut chosen: Option<usize> = None;
        if hit.pressed && count > 0 {
            if open {
                ctx.close_popup();
                open = false;
            } else {
                open_list(ctx, self.selected, count, &name);
                open = true;
            }
        }
        if open {
            let mut state = *ctx.memory::<ComboState>();
            let shown = self.shown_count();
            state.first = state.first.min(count.saturating_sub(shown));
            // The wheel over the list scrolls it.
            let wheel = interaction.wheel().y;
            if wheel != 0.0 {
                let scrolled = state.first as f32 - wheel.round();
                state.first = scrolled.clamp(0.0, count.saturating_sub(shown) as f32) as usize;
            }
            for (part, item_hit) in &interaction.hits {
                if let Part::Index(index) = part
                    && item_hit.pressed
                {
                    chosen = Some(*index);
                }
            }
            *ctx.memory::<ComboState>() = state;
            if chosen.is_some() {
                ctx.close_popup();
            }
        }
        if let Some(index) = chosen
            && index != self.selected
            && let Some(reaction) = &self.on_select
        {
            ctx.send(reaction(index));
        }
    }

    fn draw(&self, ctx: &mut Draw) {
        let rect = ctx.rect();
        let look = ctx.look().clone();
        let state = look.state();
        let placeholder = self.chosen().is_none();
        let text = self.chosen().unwrap_or(&self.placeholder).to_string();
        let theme = ctx.theme().clone();
        let background = StateColors::new(theme.control_fill, theme.control_fill_secondary, theme.control_fill_tertiary, theme.control_fill_disabled);
        // ComboBoxForeground and ComboBoxPlaceHolderForeground.
        let foreground = if placeholder {
            StateColors::new(theme.text_secondary, theme.text_secondary, theme.text_tertiary, theme.text_disabled)
        } else {
            StateColors::new(theme.text_primary, theme.text_primary, theme.text_secondary, theme.text_disabled)
        }
        .get(state);
        let chevron = if state == ControlState::Disabled { theme.text_disabled } else { theme.text_secondary };
        {
            let mut painter = ctx.painter();
            let border = match state {
                ControlState::Normal | ControlState::PointerOver => Elevation::Control.border(&theme, rect),
                ControlState::Pressed | ControlState::Disabled => Border::new(theme.control_stroke, 1.0),
            };
            painter.shape(rect, RADIUS_CONTROL, background.get(state), Some(border));
            let text_rect = Rect::new(rect.x + COMBO_BOX_PADDING.left, rect.y, rect.width - COMBO_BOX_PADDING.left - COMBO_BOX_CHEVRON_COLUMN, rect.height);
            painter.text_fit(text_rect, &text, BODY, foreground, Align::Start);
            painter.icon(Point::new(rect.right() - COMBO_BOX_CHEVRON_MARGIN - COMBO_BOX_CHEVRON_SIZE / 2.0, rect.center().y), Icon::ChevronDown, COMBO_BOX_CHEVRON_SIZE, chevron);
        }

        if !ctx.popup_open() || self.items.is_empty() {
            return;
        }
        let combo_state = *ctx.memory::<ComboState>();
        let (panel, first_slot) = self.list(ctx.bounds(), rect, combo_state.first);
        let bounds = ctx.bounds();
        let mut painter = ctx.painter();
        let layer = painter.layer();
        painter.set_layer(Layer::Popup);
        painter.push_clip(bounds);
        painter.shadow(panel, RADIUS_OVERLAY, ELEVATION_FLYOUT);
        painter.shape(panel, RADIUS_OVERLAY, theme.acrylic, Some(Border::new(theme.surface_stroke_flyout, 1.0)));
        painter.push_clip(panel);
        for row in 0..self.shown_count() {
            let index = combo_state.first + row;
            let Some(item) = self.items.get(index) else { break };
            let slot = Rect::new(first_slot.x, first_slot.y + row as f32 * COMBO_BOX_ITEM_HEIGHT, first_slot.width, first_slot.height);
            let item_rect = slot.pad(COMBO_BOX_ITEM_MARGIN);
            let part = Part::Index(index);
            let item_state = look.part_state(&part);
            // ComboBoxItemBackground, selected or not.
            let fill = match (index == self.selected, item_state) {
                (false, ControlState::Pressed) => theme.subtle_fill_tertiary,
                (false, ControlState::PointerOver) => theme.subtle_fill_secondary,
                (false, _) => theme.subtle_fill_transparent,
                (true, ControlState::PointerOver) => theme.subtle_fill_tertiary,
                (true, _) => theme.subtle_fill_secondary,
            };
            let pill_length = if item_state == ControlState::Pressed { SELECTION_PILL_PRESSED_LENGTH } else { SELECTION_PILL_LENGTH };
            let text_color = if item_state == ControlState::Pressed { theme.text_secondary } else { theme.text_primary };
            painter.rounded(item_rect, COMBO_BOX_ITEM_RADIUS, fill);
            if index == self.selected {
                let pill = Rect::new(item_rect.x, item_rect.center().y - pill_length / 2.0, SELECTION_PILL_THICKNESS, pill_length);
                painter.rounded(pill, SELECTION_PILL_THICKNESS / 2.0, theme.accent);
            }
            let text_rect = Rect::new(item_rect.x + COMBO_BOX_ITEM_PADDING.left, item_rect.y, item_rect.width - COMBO_BOX_ITEM_PADDING.horizontal(), item_rect.height);
            painter.text_fit(text_rect, item, BODY, text_color, Align::Start);
        }
        painter.pop_clip();
        painter.pop_clip();
        painter.set_layer(layer);
    }
}
