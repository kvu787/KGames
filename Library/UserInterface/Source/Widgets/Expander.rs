// WinUI's Expander: a header, as tall as ExpanderMinHeight, with a chevron
// button that shows or hides the content under it. The header is a card's
// fill and stroke; the content is the secondary card fill, which
// ExpanderContent draws under the header, with the program's widgets as its
// children.

use crate::core::interaction::{ControlState, Interaction, Sense};
use crate::core::tree::{Dispatch, Draw, Widget};
use crate::design::controls::{EXPANDER_CHEVRON_BUTTON, EXPANDER_CHEVRON_GLYPH, EXPANDER_CHEVRON_MARGIN, EXPANDER_CONTENT_PADDING, EXPANDER_HEADER_PADDING_LEFT};
use crate::design::icons::Icon;
use crate::design::tokens::{BODY, CAPTION, RADIUS_CONTROL};
use crate::drawing::geometry::{Insets, Rect};
use crate::drawing::painter::{Align, Border, Corners};
use crate::widgets::widget::{Reaction, react};

pub struct Expander<M> {
    expanded: bool,
    header: String,
    caption: Option<String>,
    on_toggle: Option<Reaction<bool, M>>,
}

impl<M: Clone + 'static> Expander<M> {
    // The header, which the element's rectangle is.
    pub fn new(expanded: bool, header: impl Into<String>) -> Self {
        Self {
            expanded,
            header: header.into(),
            caption: None,
            on_toggle: None,
        }
    }

    // A caption under the header's text.
    pub fn caption(mut self, caption: impl Into<String>) -> Self {
        self.caption = Some(caption.into());
        self
    }

    // What a click sends, with whether it's expanded then.
    pub fn on_toggle(mut self, reaction: impl Fn(bool) -> M + 'static) -> Self {
        self.on_toggle = Some(react(reaction));
        self
    }
}

impl<M: Clone + 'static> Widget<M> for Expander<M> {
    fn kind(&self) -> &'static str {
        "Expander"
    }

    fn label(&self) -> Option<String> {
        Some(if self.expanded { "Expanded".to_string() } else { "Collapsed".to_string() })
    }

    fn sense(&self) -> Option<Sense> {
        Some(Sense::BUTTON)
    }

    fn interact(&self, interaction: &Interaction, ctx: &mut Dispatch<M>) {
        if interaction.whole().activated
            && let Some(reaction) = &self.on_toggle
        {
            ctx.send(reaction(!self.expanded));
        }
    }

    fn draw(&self, ctx: &mut Draw) {
        let rect = ctx.rect();
        let state = ctx.state();
        let expanded = self.expanded;
        {
            let mut painter = ctx.painter();
            let theme = painter.theme();
            // ExpanderChevronBackground: only the chevron's button changes.
            let chevron_fill = match state {
                ControlState::PointerOver => theme.subtle_fill_secondary,
                ControlState::Pressed => theme.subtle_fill_tertiary,
                _ => theme.subtle_fill_transparent,
            };
            let text = if state == ControlState::Disabled { theme.text_disabled } else { theme.text_primary };
            let caption_color = if state == ControlState::Disabled { theme.text_disabled } else { theme.text_secondary };
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
        }
        let focused = ctx.focused();
        ctx.focus_visual_if(focused, rect, RADIUS_CONTROL);
    }
}

// The content of an expanded expander, under its header: its fill, with
// the program's widgets as its children, inside `inner`.
pub struct ExpanderContent;

impl ExpanderContent {
    // The area inside the content's padding.
    pub fn inner(rect: Rect) -> Rect {
        rect.inset(EXPANDER_CONTENT_PADDING)
    }
}

impl<M: 'static> Widget<M> for ExpanderContent {
    fn kind(&self) -> &'static str {
        "ExpanderContent"
    }

    fn covers(&self) -> bool {
        true
    }

    fn draw(&self, ctx: &mut Draw) {
        let rect = ctx.rect();
        let mut painter = ctx.painter();
        let theme = painter.theme();
        painter.shape(rect, Corners::bottom(RADIUS_CONTROL), theme.card_background_secondary, Some(Border::sides(theme.card_stroke, Insets::new(1.0, 0.0, 1.0, 1.0))));
    }
}
