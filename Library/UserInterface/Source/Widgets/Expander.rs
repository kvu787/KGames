// WinUI's Expander: a header, as tall as ExpanderMinHeight, with a chevron
// button that shows or hides the content under it. The header is a card's
// fill and stroke; the content is the secondary card fill, and the program
// draws it under the header with expander_content.

use crate::core::interaction::{ControlState, Response, Sense};
use crate::core::ui::Ui;
use crate::design::controls::{EXPANDER_CHEVRON_BUTTON, EXPANDER_CHEVRON_GLYPH, EXPANDER_CHEVRON_MARGIN, EXPANDER_CONTENT_PADDING, EXPANDER_HEADER_PADDING_LEFT};
use crate::design::icons::Icon;
use crate::design::tokens::{BODY, CAPTION, DURATION_FASTER, RADIUS_CONTROL};
use crate::drawing::geometry::{Insets, Rect};
use crate::drawing::painter::{Align, Border, Corners};
use crate::widgets::widget::Widget;

pub struct Expander<'a, 'v> {
    expanded: &'v mut bool,
    header: &'a str,
    caption: Option<&'a str>,
}

impl<'a, 'v> Expander<'a, 'v> {
    pub fn new(expanded: &'v mut bool, header: &'a str) -> Self {
        Self { expanded, header, caption: None }
    }

    // A caption under the header's text.
    pub fn caption(mut self, caption: &'a str) -> Self {
        self.caption = Some(caption);
        self
    }
}

impl Widget for Expander<'_, '_> {
    type Response = Response;

    // The rectangle is the header's.
    fn show(self, ui: &mut Ui, name: &str, rect: Rect) -> Response {
        let mut response = ui.interact(name, rect, Sense::BUTTON, "Expander");
        if response.clicked {
            *self.expanded = !*self.expanded;
            response.changed = true;
        }
        let expanded = *self.expanded;
        ui.set_label(name, if expanded { "Expanded" } else { "Collapsed" });
        let state = response.state();
        let theme = ui.theme();
        // ExpanderChevronBackground: only the chevron's button changes.
        let chevron_fill = match state {
            ControlState::PointerOver => theme.subtle_fill_secondary,
            ControlState::Pressed => theme.subtle_fill_tertiary,
            _ => theme.subtle_fill_transparent,
        };
        let text = if state == ControlState::Disabled { theme.text_disabled } else { theme.text_primary };
        let caption_color = if state == ControlState::Disabled { theme.text_disabled } else { theme.text_secondary };
        let chevron_fill = ui.animate_color(response.id, 0, chevron_fill, DURATION_FASTER);
        let mut painter = ui.painter();
        let theme = painter.theme();
        let corners = if expanded { Corners::top(RADIUS_CONTROL) } else { Corners::all(RADIUS_CONTROL) };
        painter.shape(rect, corners, theme.card_background, Some(Border::new(theme.card_stroke, 1.0)));
        let button = Rect::new(rect.right() - EXPANDER_CHEVRON_MARGIN.right - EXPANDER_CHEVRON_BUTTON, rect.center().y - EXPANDER_CHEVRON_BUTTON / 2.0, EXPANDER_CHEVRON_BUTTON, EXPANDER_CHEVRON_BUTTON);
        painter.rounded(button, RADIUS_CONTROL, chevron_fill);
        painter.icon(button.center(), if expanded { Icon::ChevronUp } else { Icon::ChevronDown }, EXPANDER_CHEVRON_GLYPH, text);
        let left = rect.x + EXPANDER_HEADER_PADDING_LEFT;
        let room = Rect::new(left, rect.y, button.x - EXPANDER_CHEVRON_MARGIN.left - left, rect.height);
        match self.caption {
            None => {
                painter.text_fit(room, self.header, BODY, text, Align::Start);
            }
            Some(caption) => {
                let top = rect.center().y - (BODY.line_height + CAPTION.line_height) / 2.0;
                painter.text_fit(Rect::new(room.x, top, room.width, BODY.line_height), self.header, BODY, text, Align::Start);
                painter.text_fit(Rect::new(room.x, top + BODY.line_height, room.width, CAPTION.line_height), caption, CAPTION, caption_color, Align::Start);
            }
        }
        ui.focus_visual_if(response.focused, rect, RADIUS_CONTROL);
        response
    }
}

impl Ui {
    // The content of an expanded expander, under its header: draws its
    // fill and returns the area inside its padding.
    pub fn expander_content(&mut self, rect: Rect) -> Rect {
        self.cover(rect);
        let mut painter = self.painter();
        let theme = painter.theme();
        painter.shape(rect, Corners::bottom(RADIUS_CONTROL), theme.card_background_secondary, Some(Border::sides(theme.card_stroke, Insets::new(1.0, 0.0, 1.0, 1.0))));
        rect.inset(EXPANDER_CONTENT_PADDING)
    }
}
