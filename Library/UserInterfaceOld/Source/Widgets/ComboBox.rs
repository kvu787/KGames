// A combo box: a button that shows the chosen item, and opens a list of the
// items to choose from. As WinUI's ComboBox: the list opens over the box,
// with the chosen item where the box shows it, in a flyout that floats over
// everything else. A press outside the list, or Esc, closes it.
//
// The box is a target named after it, and each item of its open list is one
// named after the box and the item's index, like "Gamepad/0".

use crate::geometry::{Color, Point, Rect};
use crate::icons::Icon;
use crate::input::Key;
use crate::painter::{Align, Corners, Layer, Stroke};
use crate::tokens::{
    BODY, CURVE_DECELERATE, DURATION_FASTER, DURATION_NORMAL, ELEVATION_28, ICON_SMALL, RADIUS_CONTROL, RADIUS_OVERLAY,
    SELECTION_PILL_LENGTH, SELECTION_PILL_THICKNESS, SPACE_M, SPACE_XS,
};
use crate::ui::{Response, Sense, Ui};
use crate::widgets::common::{Visual, control_focus_ring};

const ITEM_HEIGHT: f32 = 36.0;

#[derive(Default)]
struct ComboState {
    // The item that the arrows have moved to while the list is open.
    highlighted: usize,
    opened_at: f64,
}

impl Ui {
    pub fn combo_box<S: AsRef<str>>(&mut self, name: &str, rect: Rect, selected: &mut usize, items: &[S]) -> Response {
        let mut response = self.interact(name, rect, Sense::NAVIGATE, "ComboBox");
        let before = *selected;
        if let Some(item) = items.get(*selected) {
            self.set_label(name, item.as_ref());
        }
        let open = self.popup_open(response.id);
        let enabled = self.enabled();

        if response.pressed && enabled {
            if open {
                self.close_popup();
            } else if !items.is_empty() {
                self.open_list(&response, name, *selected);
            }
        } else if response.focused && enabled && !open {
            let input = self.input();
            if input.key_pressed_once(Key::SPACE) || input.key_pressed_once(Key::ENTER) {
                self.open_list(&response, name, *selected);
                self.set_focus_visible(true);
            } else if input.key_pressed(Key::DOWN) && *selected + 1 < items.len() {
                *selected += 1;
            } else if input.key_pressed(Key::UP) && *selected > 0 {
                *selected -= 1;
            }
        }

        let visual = Visual::of(self, &response);
        let theme = self.theme();
        let fill = visual.pick(theme.control_fill, theme.control_fill_secondary, theme.control_fill_tertiary, theme.control_fill_disabled);
        let stroke = match visual {
            Visual::Rest | Visual::Hover => Stroke::elevated(theme.control_stroke, theme.control_stroke_secondary),
            _ => Stroke::new(theme.control_stroke, 1.0),
        };
        let text = if visual == Visual::Disabled { theme.text_disabled } else { theme.text_primary };
        let chevron = if visual == Visual::Disabled { theme.text_disabled } else { theme.text_secondary };
        let fill = self.animate_color(response.id, 0, fill, DURATION_FASTER);
        let label = items.get(*selected).map(|item| item.as_ref().to_string()).unwrap_or_default();
        let mut painter = self.painter();
        painter.shape(rect, Corners::all(RADIUS_CONTROL), fill, Some(stroke));
        let text_area = rect.inset_by(SPACE_M, 0.0, 38.0, 0.0);
        painter.text_fit(text_area, &label, BODY, text, Align::Start);
        painter.icon(Point::new(rect.right() - 14.0 - ICON_SMALL / 2.0, rect.center().y), Icon::ChevronDown, ICON_SMALL, chevron);
        control_focus_ring(self, &response);

        if self.popup_open(response.id) {
            self.draw_list(&response, name, selected, items);
        }
        response.changed = *selected != before;
        response
    }

    fn open_list(&mut self, response: &Response, name: &str, selected: usize) {
        self.open_popup(response.id, name);
        let time = self.time();
        let state = self.memory::<ComboState>(response.id);
        state.highlighted = selected;
        state.opened_at = time;
    }

    fn draw_list<S: AsRef<str>>(&mut self, response: &Response, name: &str, selected: &mut usize, items: &[S]) {
        let window = self.bounds();
        let width = response.rect.width.max(120.0);
        let height = items.len() as f32 * ITEM_HEIGHT + SPACE_XS * 2.0;
        // The chosen item lies over the box, inside the window.
        let top = response.rect.center().y - SPACE_XS - (*selected as f32 + 0.5) * ITEM_HEIGHT;
        let top = top.clamp(window.y + SPACE_XS, (window.bottom() - height - SPACE_XS).max(window.y));
        let list = Rect::new(response.rect.x - SPACE_XS, top.round(), width + SPACE_XS * 2.0, height);

        let time = self.time();
        let state = self.memory::<ComboState>(response.id);
        let mut highlighted = state.highlighted.min(items.len().saturating_sub(1));
        let opened_at = state.opened_at;
        let input = self.input();
        if input.key_pressed(Key::DOWN) && highlighted + 1 < items.len() {
            highlighted += 1;
        }
        if input.key_pressed(Key::UP) && highlighted > 0 {
            highlighted -= 1;
        }
        if input.key_pressed(Key::HOME) {
            highlighted = 0;
        }
        if input.key_pressed(Key::END) {
            highlighted = items.len().saturating_sub(1);
        }
        let commit = input.key_pressed_once(Key::ENTER) || input.key_pressed_once(Key::SPACE);

        // The list unrolls from the chosen item, decelerating.
        let reveal = CURVE_DECELERATE.ease(((time - opened_at) as f32 / DURATION_NORMAL).clamp(0.0, 1.0));
        let theme = self.theme().clone();
        let mut painter = self.painter();
        painter.set_layer(Layer::Popup);
        let shown = Rect::new(
            list.x,
            list.y + (1.0 - reveal) * (response.rect.center().y - list.y - ITEM_HEIGHT / 2.0).max(0.0),
            list.width,
            (list.height * reveal).max(ITEM_HEIGHT.min(list.height)),
        );
        painter.shadow(shown, RADIUS_OVERLAY, ELEVATION_28);
        painter.shape(shown, Corners::all(RADIUS_OVERLAY), theme.flyout, Some(Stroke::new(theme.surface_stroke_flyout, 1.0)));
        // Targets have the whole list, whatever of it has unrolled.
        painter.push_clip_absolute(list);
        self.cover(list);
        let moved = self.input().pointer_delta != Point::default();

        let mut chosen = None;
        for (index, item) in items.iter().enumerate() {
            let row = Rect::new(list.x + SPACE_XS, list.y + SPACE_XS + index as f32 * ITEM_HEIGHT, width, ITEM_HEIGHT);
            let item_name = format!("{name}/{index}");
            let item_response = self.interact(&item_name, row.inset_by(0.0, 2.0, 0.0, 2.0), Sense::CLICK, "ComboBoxItem");
            self.set_label(&item_name, item.as_ref());
            // The pointer highlights what it moves onto; the keys, what
            // they move to.
            if item_response.hovered && moved {
                highlighted = index;
            }
            if item_response.clicked {
                chosen = Some(index);
            }
            let on = index == *selected;
            let visual = Visual::of(self, &item_response);
            let fill = match (visual, on) {
                (Visual::Pressed, _) => theme.subtle_fill_tertiary,
                (Visual::Hover, true) => theme.subtle_fill_tertiary,
                (Visual::Hover, false) => theme.subtle_fill_secondary,
                (_, true) => theme.subtle_fill_secondary,
                _ if index == highlighted => theme.subtle_fill_secondary,
                _ => Color::TRANSPARENT,
            };
            let text = if visual == Visual::Pressed { theme.text_secondary } else { theme.text_primary };
            let mut painter = self.painter();
            painter.push_clip(shown);
            let background = row.inset_by(0.0, 2.0, 0.0, 2.0);
            painter.rounded(background, RADIUS_CONTROL, fill);
            if on {
                let pill = Rect::new(
                    background.x,
                    (background.center().y - SELECTION_PILL_LENGTH / 2.0).round(),
                    SELECTION_PILL_THICKNESS,
                    SELECTION_PILL_LENGTH,
                );
                painter.rounded(pill, SELECTION_PILL_THICKNESS / 2.0, theme.accent);
            }
            painter.text_fit(row.inset_by(SPACE_M, 0.0, SPACE_M, 0.0), item.as_ref(), BODY, text, Align::Start);
            painter.pop_clip();
        }
        let mut painter = self.painter();
        painter.pop_clip();
        painter.set_layer(Layer::Base);

        if commit {
            chosen = Some(highlighted);
        }
        self.memory::<ComboState>(response.id).highlighted = highlighted;
        if let Some(index) = chosen {
            *selected = index;
            self.close_popup();
        }
    }
}
