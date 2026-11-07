// Cards: the surfaces that group a page's controls, with a title in their
// header. A card is CardBackgroundFillColorDefault with a CardStroke border
// and OverlayCornerRadius corners, on the page's background. A floating card
// is opaque acrylic, raised like a flyout, for over a 3D view. A card with a
// view leaves a hole for a 3D view that the program's render draws.
//
// A card is a group: `ui.card(name, rect, Card::new("Buttons"), |ui, area| ...)`
// makes it and its children, laid out in `area.content`.

use crate::core::control::{AutoTarget, Control, NodeContext};
use crate::core::ui::{Sense, Ui};
use crate::design::tokens::{
    BODY_STRONG, CAPTION, CARD_HEADER_GAP, CARD_HEADER_HEIGHT, CARD_PADDING, ELEVATION_FLOATING_CARD, RADIUS_CONTROL, RADIUS_OVERLAY, SPACE_S,
};
use crate::drawing::geometry::{Color, Insets, Rect};
use crate::drawing::painter::{Align, Border, Corners};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CardStyle {
    #[default]
    Standard,
    // Opaque and raised, for over a 3D view.
    Floating,
    // CardBackgroundFillColorSecondary, for a card inside a card.
    Subtle,
}

#[derive(Clone, Debug, Default)]
pub struct Card {
    title: String,
    subtitle: Option<String>,
    style: CardStyle,
    // Whether scripts see it, as a target of kind "Card".
    listed: bool,
    label: Option<String>,
    padding: f32,
    // The hole for a 3D view that the program's render draws.
    view: bool,
}

impl Card {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            subtitle: None,
            style: CardStyle::Standard,
            listed: false,
            label: None,
            padding: CARD_PADDING,
            view: false,
        }
    }

    // A caption after the title, like a count or what the card is for.
    pub fn subtitle(mut self, subtitle: &str) -> Self {
        self.subtitle = Some(subtitle.to_string());
        self
    }

    pub fn set_subtitle(&mut self, subtitle: &str) {
        self.subtitle = Some(subtitle.to_string());
    }

    pub fn floating(mut self) -> Self {
        self.style = CardStyle::Floating;
        self
    }

    pub fn subtle(mut self) -> Self {
        self.style = CardStyle::Subtle;
        self
    }

    pub fn style(mut self, style: CardStyle) -> Self {
        self.style = style;
        self
    }

    // Makes the card a target that scripts see, of kind "Card".
    pub fn listed(mut self) -> Self {
        self.listed = true;
        self
    }

    pub fn label(mut self, label: &str) -> Self {
        self.label = Some(label.to_string());
        self
    }

    pub fn set_label(&mut self, label: &str) {
        self.label = Some(label.to_string());
    }

    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    // Leaves a hole in the content for a 3D view, which the program's render
    // draws. Clear the view to Theme::card_opaque for it to look like the
    // card.
    pub fn with_view(mut self) -> Self {
        self.view = true;
        self
    }

    // How tall the card is around content `content_height` tall: its
    // padding, and its header if it has a title. For a card that must end
    // where it ends, like at the window's bottom.
    pub fn height_around(&self, content_height: f32) -> f32 {
        let header = if self.title.is_empty() { 0.0 } else { CARD_HEADER_HEIGHT + CARD_HEADER_GAP };
        self.padding * 2.0 + header + content_height
    }

    // Where the card's parts are, for laying out its children at build.
    pub fn area(&self, ui: &mut Ui, rect: Rect) -> CardArea {
        let inner = rect.inset(self.padding);
        if self.title.is_empty() {
            return CardArea {
                rect,
                header: Rect::new(inner.x, inner.y, 0.0, 0.0),
                content: inner,
            };
        }
        let mut title_width = ui.measure(&self.title, BODY_STRONG).width;
        if let Some(subtitle) = &self.subtitle {
            title_width += SPACE_S + ui.measure(subtitle, CAPTION).width;
        }
        let header_row = inner.top_strip(CARD_HEADER_HEIGHT);
        let header_left = (header_row.x + title_width + SPACE_S).min(header_row.right());
        CardArea {
            rect,
            header: Rect::new(header_left, header_row.y, header_row.right() - header_left, header_row.height),
            content: inner.pad(Insets::new(0.0, CARD_HEADER_HEIGHT + CARD_HEADER_GAP, 0.0, 0.0)),
        }
    }
}

// Where a card's parts are.
#[derive(Clone, Copy, Debug, Default)]
pub struct CardArea {
    pub rect: Rect,
    // The header, right of the title and its subtitle, for the card's own
    // commands; empty if the card has no title.
    pub header: Rect,
    // Inside the padding, under the header.
    pub content: Rect,
}

impl Control for Card {
    fn kind(&self) -> &'static str {
        "Card"
    }

    fn sense(&self) -> Sense {
        Sense::NONE
    }

    fn auto_target(&self) -> AutoTarget {
        if self.listed { AutoTarget::Listed } else { AutoTarget::Unlisted }
    }

    fn label(&self) -> Option<String> {
        self.label.clone()
    }

    fn draw(&mut self, ui: &mut Ui, ctx: &NodeContext) {
        let area = self.area(ui, ctx.rect);
        let rect = area.rect;
        let mut painter = ui.painter();
        let theme = painter.theme();
        let (fill, stroke) = match self.style {
            CardStyle::Standard => (theme.card_background, theme.card_stroke),
            CardStyle::Floating => (theme.acrylic, theme.surface_stroke_flyout),
            CardStyle::Subtle => (theme.card_background_secondary, theme.card_stroke),
        };
        if self.style == CardStyle::Floating {
            painter.shadow(rect, RADIUS_OVERLAY, ELEVATION_FLOATING_CARD);
        }
        if !self.view {
            painter.shape(rect, RADIUS_OVERLAY, fill, Some(Border::new(stroke, 1.0)));
        } else {
            // The fill goes around the view, which the program draws.
            let view = area.content;
            let top = Rect::new(rect.x, rect.y, rect.width, view.y - rect.y);
            let bottom = Rect::new(rect.x, view.bottom(), rect.width, rect.bottom() - view.bottom());
            let left = Rect::new(rect.x, view.y, view.x - rect.x, view.height);
            let right = Rect::new(view.right(), view.y, rect.right() - view.right(), view.height);
            painter.shape(top, Corners::top(RADIUS_OVERLAY), fill, None);
            painter.shape(bottom, Corners::bottom(RADIUS_OVERLAY), fill, None);
            painter.fill(left, fill);
            painter.fill(right, fill);
            painter.shape(rect, RADIUS_OVERLAY, Color::TRANSPARENT, Some(Border::new(stroke, 1.0)));
            painter.shape(view.expand(1.0), RADIUS_CONTROL, Color::TRANSPARENT, Some(Border::new(theme.card_stroke, 1.0)));
        }
        if !self.title.is_empty() {
            let header = Rect::new(area.content.x, rect.y + self.padding, area.content.width, CARD_HEADER_HEIGHT);
            let title = painter.text_in(header, &self.title, BODY_STRONG, theme.text_primary, Align::Start);
            if let Some(subtitle) = &self.subtitle {
                let after = Rect::new(title.right() + SPACE_S, header.y, header.right() - title.right() - SPACE_S, header.height);
                painter.text_fit(after, subtitle, CAPTION, theme.text_secondary, Align::Start);
            }
        }
    }
}

impl Ui {
    // A card and its children, laid out in its content area. Only in
    // Program::build.
    pub fn card(&mut self, name: &str, rect: Rect, card: Card, children: impl FnOnce(&mut Ui, CardArea)) -> CardArea {
        let area = card.area(self, rect);
        self.group(name, rect, card, |ui| children(ui, area));
        area
    }
}
