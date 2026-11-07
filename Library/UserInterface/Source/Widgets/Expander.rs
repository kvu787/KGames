// WinUI's Expander: a header, as tall as ExpanderMinHeight, with a chevron
// button that shows or hides the content under it. The header is a card's
// fill and stroke; the content is the secondary card fill. The node's
// rectangle holds the header and the content; its children, laid out inside
// Expander::content_area, show while it's expanded.

use crate::core::control::{Control, NodeContext, UiEventKind, adopted};
use crate::core::ui::{ControlState, Sense, Ui};
use crate::design::controls::{EXPANDER_CHEVRON_BUTTON, EXPANDER_CHEVRON_GLYPH, EXPANDER_CHEVRON_MARGIN, EXPANDER_CONTENT_PADDING, EXPANDER_HEADER_PADDING_LEFT};
use crate::design::icons::Icon;
use crate::design::tokens::{BODY, CAPTION, DURATION_FASTER, RADIUS_CONTROL};
use crate::drawing::geometry::{Insets, Rect};
use crate::drawing::painter::{Align, Border, Corners};

// The header's height when none is given: WinUI's ExpanderMinHeight.
pub const EXPANDER_HEADER_HEIGHT: f32 = 48.0;

pub struct Expander {
    expanded: bool,
    header: String,
    caption: Option<String>,
    header_height: f32,
}

impl Expander {
    pub fn new(expanded: bool, header: &str) -> Self {
        Self {
            expanded,
            header: header.to_string(),
            caption: None,
            header_height: EXPANDER_HEADER_HEIGHT,
        }
    }

    // A caption under the header's text.
    pub fn caption(mut self, caption: &str) -> Self {
        self.caption = Some(caption.to_string());
        self
    }

    pub fn header_height(mut self, height: f32) -> Self {
        self.header_height = height;
        self
    }

    pub fn is_expanded(&self) -> bool {
        self.expanded
    }

    pub fn set_expanded(&mut self, expanded: bool) {
        self.expanded = expanded;
    }

    // Where the expanded content's children go: under a header
    // `header_height` tall, inside the content's padding.
    pub fn content_area(rect: Rect, header_height: f32) -> Rect {
        Rect::new(rect.x, rect.y + header_height, rect.width, (rect.height - header_height).max(0.0)).inset(EXPANDER_CONTENT_PADDING)
    }

    fn header_rect(&self, rect: Rect) -> Rect {
        Rect::new(rect.x, rect.y, rect.width, self.header_height.min(rect.height))
    }
}

impl Control for Expander {
    fn kind(&self) -> &'static str {
        "Expander"
    }

    fn sense(&self) -> Sense {
        Sense::BUTTON
    }

    // The target is the header; the content under it is its children's.
    fn target_rect(&self, rect: Rect) -> Rect {
        self.header_rect(rect)
    }

    fn label(&self) -> Option<String> {
        Some(if self.expanded { "Expanded".to_string() } else { "Collapsed".to_string() })
    }

    fn children_visible(&self) -> bool {
        self.expanded
    }

    fn adopt(&mut self, previous: Box<dyn Control>) {
        if let Some(previous) = adopted::<Expander>(previous) {
            self.expanded = previous.expanded;
        }
    }

    fn handle(&mut self, ui: &mut Ui, ctx: &NodeContext) {
        if ctx.response.clicked {
            self.expanded = !self.expanded;
            ctx.emit(ui, UiEventKind::Expanded(self.expanded));
        }
        // The content covers what's under it. Registered here, before the
        // children's targets, so they stay over it.
        if self.expanded && ctx.rect.height > self.header_height {
            let header = self.header_rect(ctx.rect);
            let content = Rect::new(ctx.rect.x, header.bottom(), ctx.rect.width, ctx.rect.bottom() - header.bottom());
            ui.cover(content);
        }
    }

    fn draw(&mut self, ui: &mut Ui, ctx: &NodeContext) {
        let rect = self.header_rect(ctx.rect);
        let response = &ctx.response;
        let expanded = self.expanded;
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
        match &self.caption {
            None => {
                painter.text_fit(room, &self.header, BODY, text, Align::Start);
            }
            Some(caption) => {
                let top = rect.center().y - (BODY.line_height + CAPTION.line_height) / 2.0;
                painter.text_fit(Rect::new(room.x, top, room.width, BODY.line_height), &self.header, BODY, text, Align::Start);
                painter.text_fit(Rect::new(room.x, top + BODY.line_height, room.width, CAPTION.line_height), caption, CAPTION, caption_color, Align::Start);
            }
        }
        ui.focus_visual_if(response.focused, rect, RADIUS_CONTROL);

        // The content's fill, under the header, behind the children, which
        // draw after this.
        if expanded && ctx.rect.height > self.header_height {
            let content = Rect::new(ctx.rect.x, rect.bottom(), ctx.rect.width, ctx.rect.bottom() - rect.bottom());
            let mut painter = ui.painter();
            let theme = painter.theme();
            painter.shape(content, Corners::bottom(RADIUS_CONTROL), theme.card_background_secondary, Some(Border::sides(theme.card_stroke, Insets::new(1.0, 0.0, 1.0, 1.0))));
        }
    }
}
