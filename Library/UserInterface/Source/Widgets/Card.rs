// Cards: the surfaces that group a page's controls, with a title in their
// header. A card is CardBackgroundFillColorDefault with a CardStroke border
// and OverlayCornerRadius corners, on the page's background. A floating card
// is opaque acrylic, raised like a flyout, for over a 3D view. A card with a
// view leaves a hole for a 3D view that the program's render draws.
//
// A card can size itself to its content: begin_card gives the content all
// the height it may take, the program cuts its controls from the top, and
// end_card draws the card behind them, as tall as what they used.

use crate::core::ui::{Sense, Ui};
use crate::design::tokens::{
    BODY_STRONG, CAPTION, CARD_HEADER_GAP, CARD_HEADER_HEIGHT, CARD_PADDING, ELEVATION_FLOATING_CARD, RADIUS_CONTROL, RADIUS_OVERLAY, SPACE_S,
};
use crate::drawing::geometry::{Color, Rect};
use crate::drawing::painter::{Align, Border, Corners, Layer};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CardStyle {
    #[default]
    Standard,
    // Opaque and raised, for over a 3D view.
    Floating,
    // CardBackgroundFillColorSecondary, for a card inside a card.
    Subtle,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Card<'a> {
    title: &'a str,
    subtitle: Option<&'a str>,
    style: CardStyle,
    // The name that scripts know it by, if they should.
    name: Option<&'a str>,
    label: Option<&'a str>,
    padding: f32,
}

impl<'a> Card<'a> {
    pub fn new(title: &'a str) -> Self {
        Self {
            title,
            subtitle: None,
            style: CardStyle::Standard,
            name: None,
            label: None,
            padding: CARD_PADDING,
        }
    }

    // A caption after the title, like a count or what the card is for.
    pub fn subtitle(mut self, subtitle: &'a str) -> Self {
        self.subtitle = Some(subtitle);
        self
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
    pub fn named(mut self, name: &'a str) -> Self {
        self.name = Some(name);
        self
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    // How tall the card is around content `content_height` tall: its
    // padding, and its header if it has a title. For a card that must end
    // where it ends, like at the window's bottom.
    pub fn height_around(&self, content_height: f32) -> f32 {
        let header = if self.title.is_empty() { 0.0 } else { CARD_HEADER_HEIGHT + CARD_HEADER_GAP };
        self.padding * 2.0 + header + content_height
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

// A card whose height its content decides, between begin_card and end_card.
pub struct OpenCard<'a> {
    card: Card<'a>,
    pub area: CardArea,
    layer: Layer,
    first_instance: usize,
    first_target: usize,
}

impl Ui {
    // A card in the rectangle.
    pub fn card(&mut self, rect: Rect, card: Card) -> CardArea {
        let area = card_area(self, rect, &card);
        self.card_contents(&card, &area, None);
        area
    }

    // A card with a hole in its content for a 3D view, which the program's
    // render draws into: the content's area. Clear the view to
    // Theme::card_opaque for it to look like the card.
    pub fn card_with_view(&mut self, rect: Rect, card: Card) -> CardArea {
        let area = card_area(self, rect, &card);
        self.card_contents(&card, &area, Some(area.content));
        area
    }

    // Starts a card at the rectangle's top, which may be as tall as the
    // rectangle. Lay its content out from the top of `area.content`, and end
    // it with end_card.
    pub fn begin_card<'a>(&mut self, rect: Rect, card: Card<'a>) -> OpenCard<'a> {
        let area = card_area(self, rect, &card);
        OpenCard {
            card,
            area,
            layer: self.layer(),
            first_instance: self.draw.layer_len(),
            first_target: self.target_count(),
        }
    }

    // Ends a card whose content ends at `content_bottom`, and draws it
    // behind its content. Returns where it is.
    pub fn end_card(&mut self, open: OpenCard, content_bottom: f32) -> Rect {
        let mut area = open.area;
        let bottom = (content_bottom + open.card.padding).min(area.rect.bottom());
        area.rect.height = (bottom - area.rect.y).max(0.0);
        area.content.height = (content_bottom - area.content.y).max(0.0);
        debug_assert_eq!(open.layer, self.layer(), "a card ends on the layer it began on");
        let before = self.draw.layer_len();
        let targets_before = self.target_count();
        self.card_contents(&open.card, &area, None);
        // Behind what's in it: its instances and its targets go first.
        self.draw.move_to_back(open.first_instance, before);
        self.move_targets_back(open.first_target, targets_before);
        area.rect
    }

    fn card_contents(&mut self, card: &Card, area: &CardArea, view: Option<Rect>) {
        let rect = area.rect;
        // A floating card covers the view under it; every card is a surface
        // that the pointer stops at.
        match card.name {
            Some(name) => {
                self.register(name, rect, Sense::NONE, "Card");
                if let Some(label) = card.label {
                    self.set_label(name, label);
                }
            }
            None => self.cover(rect),
        }
        let mut painter = self.painter();
        let theme = painter.theme();
        let (fill, stroke) = match card.style {
            CardStyle::Standard => (theme.card_background, theme.card_stroke),
            CardStyle::Floating => (theme.acrylic, theme.surface_stroke_flyout),
            CardStyle::Subtle => (theme.card_background_secondary, theme.card_stroke),
        };
        if card.style == CardStyle::Floating {
            painter.shadow(rect, RADIUS_OVERLAY, ELEVATION_FLOATING_CARD);
        }
        match view {
            None => painter.shape(rect, RADIUS_OVERLAY, fill, Some(Border::new(stroke, 1.0))),
            Some(view) => {
                // The fill goes around the view, which the program draws.
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
        }
        if !card.title.is_empty() {
            let header = Rect::new(area.content.x, rect.y + card.padding, area.content.width, CARD_HEADER_HEIGHT);
            let title = painter.text_in(header, card.title, BODY_STRONG, theme.text_primary, Align::Start);
            if let Some(subtitle) = card.subtitle {
                let after = Rect::new(title.right() + SPACE_S, header.y, header.right() - title.right() - SPACE_S, header.height);
                painter.text_fit(after, subtitle, CAPTION, theme.text_secondary, Align::Start);
            }
        }
    }
}

fn card_area(ui: &mut Ui, rect: Rect, card: &Card) -> CardArea {
    let inner = rect.inset(card.padding);
    if card.title.is_empty() {
        return CardArea {
            rect,
            header: Rect::new(inner.x, inner.y, 0.0, 0.0),
            content: inner,
        };
    }
    let mut title_width = ui.measure(card.title, BODY_STRONG).width;
    if let Some(subtitle) = card.subtitle {
        title_width += SPACE_S + ui.measure(subtitle, CAPTION).width;
    }
    let header_row = inner.top_strip(CARD_HEADER_HEIGHT);
    let header_left = (header_row.x + title_width + SPACE_S).min(header_row.right());
    CardArea {
        rect,
        header: Rect::new(header_left, header_row.y, header_row.right() - header_left, header_row.height),
        content: inner.pad(crate::drawing::geometry::Insets::new(0.0, CARD_HEADER_HEIGHT + CARD_HEADER_GAP, 0.0, 0.0)),
    }
}
