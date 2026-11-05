// Choosing one of a few: a selector bar, which is a row of tabs that switch
// what's shown, and a segmented control, which picks a mode. As WinUI's
// SelectorBar, and the Windows Community Toolkit's Segmented.
//
// Each item is a target named after the control and the item's label, like
// "Mode/Terrain".

use crate::geometry::{Color, Rect};
use crate::icons::Icon;
use crate::input::Key;
use crate::painter::{Align, Corners, Stroke};
use crate::tokens::{
    BODY, CURVE_DECELERATE, DURATION_FASTER, DURATION_NORMAL, RADIUS_CONTROL, SELECTION_PILL_LENGTH,
    SELECTION_PILL_THICKNESS, SPACE_M, SPACE_XXS,
};
use crate::ui::{Response, Sense, Ui, id};
use crate::widgets::common::{Visual, focus_ring};

impl Ui {
    // How wide a selector bar's items are, from their labels.
    pub fn selector_bar_width(&mut self, labels: &[&str]) -> f32 {
        labels.iter().map(|label| self.measure(label, BODY).width.ceil() + SPACE_M * 2.0).sum()
    }

    // A row of tabs. `selected` is the index of the one that's on.
    pub fn selector_bar(&mut self, name: &str, rect: Rect, selected: &mut usize, labels: &[&str]) -> Response {
        let before = *selected;
        let mut x = rect.x;
        let mut items = Vec::with_capacity(labels.len());
        for label in labels {
            let width = self.measure(label, BODY).width.ceil() + SPACE_M * 2.0;
            items.push(Rect::new(x, rect.y, width, rect.height));
            x += width;
        }
        let responses = self.choose(name, &items, labels, selected, "SelectorBarItem");
        let theme = self.theme().clone();
        for (index, (item, response)) in items.iter().zip(&responses).enumerate() {
            let on = index == *selected;
            let visual = Visual::of(self, response);
            let fill = visual.pick(Color::TRANSPARENT, theme.subtle_fill_secondary, theme.subtle_fill_tertiary, Color::TRANSPARENT);
            let text = if visual == Visual::Disabled {
                theme.text_disabled
            } else if on {
                theme.text_primary
            } else {
                visual.pick(theme.text_secondary, theme.text_primary, theme.text_tertiary, theme.text_disabled)
            };
            let fill = self.animate_color(response.id, 0, fill, DURATION_FASTER);
            let mut painter = self.painter();
            let background = item.inset_by(SPACE_XXS, SPACE_XS_HALF, SPACE_XXS, SPACE_XS_HALF);
            painter.rounded(background, RADIUS_CONTROL, fill);
            painter.text_in(*item, labels[index], BODY, text, Align::Center);
            focus_ring(self, response, background, RADIUS_CONTROL);
        }
        // The pill under the selected tab slides to a newly selected one.
        if let Some(item) = items.get(*selected) {
            let pill_color = if self.enabled() { theme.accent } else { theme.accent_disabled };
            let bar = id(name);
            let center = self.animate(bar, 0, item.center().x, DURATION_NORMAL, CURVE_DECELERATE);
            let mut painter = self.painter();
            let pill = Rect::new(
                (center - SELECTION_PILL_LENGTH / 2.0).round(),
                item.bottom() - SELECTION_PILL_THICKNESS - SPACE_XXS,
                SELECTION_PILL_LENGTH,
                SELECTION_PILL_THICKNESS,
            );
            painter.rounded(pill, SELECTION_PILL_THICKNESS / 2.0, pill_color);
        }
        Response {
            id: id(name),
            rect,
            changed: *selected != before,
            ..Default::default()
        }
    }

    // Segments of equal width, one of which is on.
    pub fn segmented(&mut self, name: &str, rect: Rect, selected: &mut usize, labels: &[&str]) -> Response {
        let items: Vec<(Option<Icon>, &str)> = labels.iter().map(|label| (None, *label)).collect();
        self.segmented_items(name, rect, selected, &items)
    }

    // Segments with an icon before each label.
    pub fn segmented_icons(&mut self, name: &str, rect: Rect, selected: &mut usize, items: &[(Icon, &str)]) -> Response {
        let items: Vec<(Option<Icon>, &str)> = items.iter().map(|(icon, label)| (Some(*icon), *label)).collect();
        self.segmented_items(name, rect, selected, &items)
    }

    fn segmented_items(&mut self, name: &str, rect: Rect, selected: &mut usize, items: &[(Option<Icon>, &str)]) -> Response {
        let before = *selected;
        let count = items.len().max(1) as f32;
        let width = rect.width / count;
        let rects: Vec<Rect> = (0..items.len())
            .map(|index| Rect::new((rect.x + width * index as f32).round(), rect.y, width.round(), rect.height))
            .collect();
        let labels: Vec<&str> = items.iter().map(|(_, label)| *label).collect();
        let theme = self.theme().clone();
        let mut painter = self.painter();
        painter.shape(
            rect,
            Corners::all(RADIUS_CONTROL),
            theme.control_alt_fill_secondary,
            Some(Stroke::new(theme.control_stroke, 1.0)),
        );
        let responses = self.choose(name, &rects, &labels, selected, "Segment");
        for (index, (segment, response)) in rects.iter().zip(&responses).enumerate() {
            let on = index == *selected;
            let visual = Visual::of(self, response);
            let inner = segment.inset(SPACE_XXS + 1.0);
            let (fill, stroke) = if on {
                (
                    visual.pick(theme.control_fill, theme.control_fill_secondary, theme.control_fill_tertiary, theme.control_fill_disabled),
                    Some(Stroke::elevated(theme.control_stroke, theme.control_stroke_secondary)),
                )
            } else {
                (visual.pick(Color::TRANSPARENT, theme.subtle_fill_secondary, theme.subtle_fill_tertiary, Color::TRANSPARENT), None)
            };
            let text = if visual == Visual::Disabled {
                theme.text_disabled
            } else if on {
                theme.text_primary
            } else {
                visual.pick(theme.text_secondary, theme.text_primary, theme.text_tertiary, theme.text_disabled)
            };
            let fill = self.animate_color(response.id, 0, fill, DURATION_FASTER);
            let mut painter = self.painter();
            painter.shape(inner, Corners::all(RADIUS_CONTROL - 1.0), fill, stroke);
            crate::widgets::buttons::draw_content(&mut painter, inner, items[index].1, items[index].0, text);
            if on {
                let pill_color = if visual == Visual::Disabled { theme.accent_disabled } else { theme.accent };
                let pill = Rect::new(
                    (inner.center().x - SELECTION_PILL_LENGTH / 2.0).round(),
                    inner.bottom() - SELECTION_PILL_THICKNESS,
                    SELECTION_PILL_LENGTH,
                    SELECTION_PILL_THICKNESS,
                );
                painter.shape(pill, Corners::top(SELECTION_PILL_THICKNESS / 2.0), pill_color, None);
            }
            focus_ring(self, response, inner, RADIUS_CONTROL);
        }
        Response {
            id: id(name),
            rect,
            changed: *selected != before,
            ..Default::default()
        }
    }

    // The items of a choice, each a target: a press picks one, and the arrows
    // pick the one before or after the one with focus.
    fn choose(&mut self, name: &str, rects: &[Rect], labels: &[&str], selected: &mut usize, kind: &'static str) -> Vec<Response> {
        let mut responses = Vec::with_capacity(rects.len());
        let mut moved = None;
        for (index, rect) in rects.iter().enumerate() {
            let item_name = format!("{name}/{}", labels[index]);
            let response = self.interact(&item_name, *rect, Sense::NAVIGATE, kind);
            self.set_label(&item_name, labels[index]);
            if response.clicked && self.enabled() {
                *selected = index;
            }
            if response.focused && self.enabled() {
                let input = self.input();
                if (input.key_pressed(Key::RIGHT) || input.key_pressed(Key::DOWN)) && index + 1 < rects.len() {
                    moved = Some(index + 1);
                } else if (input.key_pressed(Key::LEFT) || input.key_pressed(Key::UP)) && index > 0 {
                    moved = Some(index - 1);
                } else if input.key_pressed(Key::HOME) {
                    moved = Some(0);
                } else if input.key_pressed(Key::END) {
                    moved = Some(rects.len() - 1);
                } else if input.key_pressed(Key::SPACE) || input.key_pressed(Key::ENTER) {
                    *selected = index;
                }
            }
            responses.push(response);
        }
        if let Some(index) = moved {
            *selected = index;
            self.focus(&format!("{name}/{}", labels[index]));
            self.set_focus_visible(true);
        }
        responses
    }
}

const SPACE_XS_HALF: f32 = 4.0;
