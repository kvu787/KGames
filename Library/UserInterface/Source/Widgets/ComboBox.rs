// WinUI's ComboBox: a button that shows the chosen item and a chevron, whose
// list opens over it as a popup, with the chosen item where the combo box's
// text is, as WinUI places it. Items are named "<name>/<index>", labeled with
// their text, and the combo box's own label is the chosen item.
//
// Closed, Up and Down choose the item before or after, and Enter, Space or
// Alt+Down open the list. Open, Up and Down move the highlight, Enter chooses
// it, and Esc or a press outside closes the list.

use crate::core::ui::{ControlState, Id, Response, Sense, Ui};
use crate::design::controls::{
    COMBO_BOX_CHEVRON_COLUMN, COMBO_BOX_FOCUS_MARGIN, COMBO_BOX_FOCUS_RADIUS, COMBO_BOX_FOCUS_THICKNESS, COMBO_BOX_ITEM_RADIUS,
    COMBO_BOX_MIN_WIDTH, COMBO_BOX_POPUP_MIN_WIDTH, SELECTION_PILL_PRESSED_LENGTH, COMBO_BOX_CHEVRON_MARGIN, COMBO_BOX_CHEVRON_SIZE, COMBO_BOX_ITEM_HEIGHT, COMBO_BOX_ITEM_MARGIN, COMBO_BOX_ITEM_PADDING,
    COMBO_BOX_PADDING, COMBO_BOX_POPUP_MAX_ITEMS, COMBO_BOX_POPUP_PADDING, SELECTION_PILL_LENGTH, SELECTION_PILL_THICKNESS,
};
use crate::design::icons::Icon;
use crate::design::tokens::{BODY, DURATION_FASTER, ELEVATION_FLYOUT, RADIUS_CONTROL, RADIUS_OVERLAY};
use crate::drawing::geometry::{Point, Rect};
use crate::drawing::painter::{Align, Border, Layer};
use crate::input::devices::Key;
use crate::widgets::widget::{Elevation, StateColors, Widget};

pub struct ComboBox<'a, 'v> {
    selected: &'v mut usize,
    items: &'a [&'a str],
    placeholder: &'a str,
}

impl<'a, 'v> ComboBox<'a, 'v> {
    pub fn new(selected: &'v mut usize, items: &'a [&'a str]) -> Self {
        Self { selected, items, placeholder: "" }
    }

    pub fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = placeholder;
        self
    }

    // The width that fits its widest item, its padding and its chevron.
    pub fn width(ui: &mut Ui, items: &[&str]) -> f32 {
        let widest = items.iter().map(|item| ui.measure(item, BODY).width).fold(0.0, f32::max);
        (COMBO_BOX_PADDING.left + widest + COMBO_BOX_CHEVRON_COLUMN + 2.0).ceil().max(COMBO_BOX_MIN_WIDTH)
    }
}

// The item that the keyboard highlights in the open list, and how far the
// list is scrolled, in items.
#[derive(Default)]
struct ComboState {
    highlighted: usize,
    first: usize,
}

impl Widget for ComboBox<'_, '_> {
    type Response = Response;

    fn show(self, ui: &mut Ui, name: &str, rect: Rect) -> Response {
        let mut response = ui.interact(name, rect, Sense::NAVIGATE, "ComboBox");
        let count = self.items.len();
        if *self.selected >= count && count > 0 {
            *self.selected = count - 1;
        }
        let chosen = self.items.get(*self.selected).copied().unwrap_or(self.placeholder);
        ui.set_label(name, chosen);
        let id = response.id;
        let mut open = ui.popup_open(id);
        if response.clicked && count > 0 {
            if open {
                ui.close_popup(id);
                open = false;
            } else {
                open_list(ui, id, name, *self.selected, count);
                open = true;
            }
        }
        if response.focused && !open && count > 0 {
            let input = ui.input();
            let alt_down = input.modifiers().alt && input.key_pressed(Key::DOWN);
            if alt_down || input.key_pressed_once(Key::ENTER) || input.key_pressed_once(Key::SPACE) {
                open_list(ui, id, name, *self.selected, count);
                open = true;
            } else if input.key_pressed(Key::DOWN) && *self.selected + 1 < count {
                *self.selected += 1;
                response.changed = true;
            } else if input.key_pressed(Key::UP) && *self.selected > 0 {
                *self.selected -= 1;
                response.changed = true;
            }
        }

        draw_closed(ui, &response, rect, self.items.get(*self.selected).copied().unwrap_or(self.placeholder), self.items.get(*self.selected).is_none());
        if open && let Some(chosen) = draw_list(ui, id, name, rect, self.items, *self.selected) {
            if chosen != *self.selected {
                *self.selected = chosen;
                response.changed = true;
            }
            ui.close_popup(id);
        }
        response
    }
}

fn open_list(ui: &mut Ui, id: Id, name: &str, selected: usize, count: usize) {
    ui.open_popup(id, name);
    let state = ui.memory::<ComboState>(id);
    state.highlighted = selected.min(count.saturating_sub(1));
    state.first = selected.saturating_sub(COMBO_BOX_POPUP_MAX_ITEMS / 2).min(count.saturating_sub(COMBO_BOX_POPUP_MAX_ITEMS));
}

fn draw_closed(ui: &mut Ui, response: &Response, rect: Rect, text: &str, placeholder: bool) {
    let state = response.state();
    let theme = ui.theme();
    let background = StateColors::new(theme.control_fill, theme.control_fill_secondary, theme.control_fill_tertiary, theme.control_fill_disabled);
    // ComboBoxForeground and ComboBoxPlaceHolderForeground.
    let foreground = if placeholder {
        StateColors::new(theme.text_secondary, theme.text_secondary, theme.text_tertiary, theme.text_disabled)
    } else {
        StateColors::new(theme.text_primary, theme.text_primary, theme.text_secondary, theme.text_disabled)
    }
    .get(state);
    let keyboard_focus = response.focused && ui.focus_visible();
    let chevron = if state == ControlState::Disabled { theme.text_disabled } else { theme.text_secondary };
    let background = ui.animate_color(response.id, 0, background.get(state), DURATION_FASTER);
    let mut painter = ui.painter();
    let theme = painter.theme();
    // A combo box draws its own focus cue instead of the system's: a ring 4
    // pixels outside it, and the pill at its left.
    if keyboard_focus {
        let ring = rect.expand(COMBO_BOX_FOCUS_MARGIN);
        painter.shape(ring, COMBO_BOX_FOCUS_RADIUS, theme.control_fill, Some(Border::new(theme.focus_outer, COMBO_BOX_FOCUS_THICKNESS)));
    }
    let border = match state {
        ControlState::Normal | ControlState::PointerOver => Elevation::Control.border(theme, rect),
        _ => Border::new(theme.control_stroke, 1.0),
    };
    painter.shape(rect, RADIUS_CONTROL, background, Some(border));
    if keyboard_focus {
        let pill = Rect::new(rect.x + 1.0, rect.center().y - SELECTION_PILL_LENGTH / 2.0, SELECTION_PILL_THICKNESS, SELECTION_PILL_LENGTH);
        painter.rounded(pill, SELECTION_PILL_THICKNESS / 2.0, theme.accent);
    }
    let chevron_center = Point::new(rect.right() - COMBO_BOX_CHEVRON_MARGIN - COMBO_BOX_CHEVRON_SIZE / 2.0, rect.center().y);
    let text_left = rect.x + COMBO_BOX_PADDING.left;
    let text_right = rect.right() - COMBO_BOX_CHEVRON_COLUMN;
    let text_rect = Rect::new(text_left, rect.y, (text_right - text_left).max(0.0), rect.height);
    painter.text_fit(text_rect, text, BODY, foreground, Align::Start);
    painter.icon(chevron_center, Icon::ChevronDown, COMBO_BOX_CHEVRON_SIZE, chevron);
}

// Draws the open list, and returns the item chosen from it, if one was.
fn draw_list(ui: &mut Ui, id: Id, name: &str, anchor: Rect, items: &[&str], selected: usize) -> Option<usize> {
    let count = items.len();
    let shown = count.min(COMBO_BOX_POPUP_MAX_ITEMS);
    let pitch = COMBO_BOX_ITEM_HEIGHT + COMBO_BOX_ITEM_MARGIN.vertical();
    let height = COMBO_BOX_POPUP_PADDING.vertical() + pitch * shown as f32;
    let widest = items.iter().map(|item| ui.measure(item, BODY).width).fold(0.0, f32::max);
    let width = anchor.width.max(widest + COMBO_BOX_ITEM_PADDING.horizontal() + COMBO_BOX_ITEM_MARGIN.horizontal() + 2.0).max(COMBO_BOX_POPUP_MIN_WIDTH).ceil();

    // Keys move the highlight while the list is on top.
    let mut chosen = None;
    let (mut highlighted, mut first) = {
        let state = ui.memory::<ComboState>(id);
        (state.highlighted.min(count - 1), state.first)
    };
    if ui.popup_on_top(id) {
        let input = ui.input();
        if input.key_pressed(Key::DOWN) {
            highlighted = (highlighted + 1).min(count - 1);
        }
        if input.key_pressed(Key::UP) {
            highlighted = highlighted.saturating_sub(1);
        }
        if input.key_pressed(Key::HOME) {
            highlighted = 0;
        }
        if input.key_pressed(Key::END) {
            highlighted = count - 1;
        }
        if input.key_pressed_once(Key::ENTER) || input.key_pressed_once(Key::SPACE) {
            chosen = Some(highlighted);
        }
    }
    if highlighted < first {
        first = highlighted;
    } else if highlighted >= first + shown {
        first = highlighted + 1 - shown;
    }

    // The chosen item goes where the combo box's text is, and the list stays
    // in the window.
    let bounds = ui.bounds();
    let selected_row = selected.saturating_sub(first).min(shown.saturating_sub(1)) as f32;
    let mut y = anchor.center().y - COMBO_BOX_POPUP_PADDING.top - pitch * selected_row - pitch / 2.0;
    y = y.clamp(bounds.y + 8.0, (bounds.bottom() - height - 8.0).max(bounds.y));
    let x = (anchor.x - COMBO_BOX_ITEM_MARGIN.left).clamp(bounds.x + 8.0, (bounds.right() - width - 8.0).max(bounds.x));
    let popup = Rect::new(x.round(), y.round(), width, height);

    let layer = ui.layer();
    ui.draw.set_layer(Layer::Popup);
    ui.draw.push_clip(bounds, true);
    ui.register(&format!("{name}/List"), popup, Sense::NONE, "ComboBoxPopup");
    {
        let mut painter = ui.painter();
        let theme = painter.theme();
        painter.shadow(popup, RADIUS_OVERLAY, ELEVATION_FLYOUT);
        painter.shape(popup, RADIUS_OVERLAY, theme.acrylic, Some(Border::new(theme.surface_stroke_flyout, 1.0)));
    }
    ui.draw.push_clip(popup.inset(1.0), false);
    // The wheel scrolls a list that's longer than the popup.
    if ui.pointer_in(popup) && ui.input().wheel.y != 0.0 && count > shown {
        let notches = ui.input().wheel.y.round() as isize;
        first = (first as isize - notches).clamp(0, (count - shown) as isize) as usize;
    }
    for row in 0..shown {
        let index = first + row;
        let slot = Rect::new(popup.x, popup.y + COMBO_BOX_POPUP_PADDING.top + pitch * row as f32, popup.width, pitch);
        let item = slot.pad(COMBO_BOX_ITEM_MARGIN);
        let item_name = format!("{name}/{index}");
        let response = ui.interact(&item_name, item, Sense::CLICK, "ComboBoxItem");
        ui.set_label(&item_name, items[index]);
        // The pointer moves the highlight only as it moves, so that the
        // keys can move it while the pointer rests on the list.
        if response.hovered && ui.input().pointer_delta != Point::ZERO {
            highlighted = index;
        }
        if response.clicked {
            chosen = Some(index);
        }
        let state = response.state();
        let theme = ui.theme();
        // ComboBoxItemBackground, selected or not.
        let fill = match (index == selected, state) {
            (false, ControlState::Pressed) => theme.subtle_fill_tertiary,
            (false, ControlState::PointerOver) => theme.subtle_fill_secondary,
            (false, _) => theme.subtle_fill_transparent,
            (true, ControlState::PointerOver) => theme.subtle_fill_tertiary,
            (true, _) => theme.subtle_fill_secondary,
        };
        let pill_length = if state == ControlState::Pressed { SELECTION_PILL_PRESSED_LENGTH } else { SELECTION_PILL_LENGTH };
        let text_color = if state == ControlState::Pressed { theme.text_secondary } else { theme.text_primary };
        let pill_length = ui.animate(response.id, 1, pill_length, crate::design::tokens::DURATION_FAST, crate::design::tokens::CURVE_FAST_OUT_SLOW_IN);
        let fill = ui.animate_color(response.id, 0, fill, DURATION_FASTER);
        let mut painter = ui.painter();
        let theme = painter.theme();
        painter.rounded(item, COMBO_BOX_ITEM_RADIUS, fill);
        if index == selected {
            let pill = Rect::new(item.x, item.center().y - pill_length / 2.0, SELECTION_PILL_THICKNESS, pill_length);
            painter.rounded(pill, SELECTION_PILL_THICKNESS / 2.0, theme.accent);
        }
        let text = Rect::new(item.x + COMBO_BOX_ITEM_PADDING.left, item.y, item.width - COMBO_BOX_ITEM_PADDING.horizontal(), item.height);
        painter.text_fit(text, items[index], BODY, text_color, Align::Start);
        if index == highlighted && ui.focus_visible() && !response.hovered {
            ui.focus_visual_if(true, item, COMBO_BOX_ITEM_RADIUS);
        }
    }
    ui.draw.pop_clip();
    ui.draw.pop_clip();
    ui.draw.set_layer(layer);
    let state = ui.memory::<ComboState>(id);
    state.highlighted = highlighted;
    state.first = first;
    chosen
}
