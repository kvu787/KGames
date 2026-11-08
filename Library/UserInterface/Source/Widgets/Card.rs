// Cards: the surfaces that group a page's controls, with a title in their
// header. A card is CardBackgroundFillColorDefault with a CardStroke border
// and OverlayCornerRadius corners, on the page's background. A floating card
// is opaque acrylic, raised like a flyout, for over a 3D view. A card with a
// view leaves a hole in its content for a 3D view that the program's render
// draws.
//
// A card's controls are its children, laid out in `area(ui, rect).content`:
//
//     tree.card(ui, Some("Settings"), rect, Card::new("Settings"), |card, area| {
//         let mut form = Form::new(area.content);
//         card.add("Volume", form.control(card, "Volume"), Slider::new(...));
//     });
//
// A card can size itself to its content with Tree::sized_card: its content
// is laid out from the top of all the height it may take, and the card ends
// where the content does.

use crate::core::interaction::Sense;
use crate::core::interface::Ui;
use crate::core::tree::{Draw, Element, Tree, Widget};
use crate::design::tokens::{BODY_STRONG, CAPTION, CARD_HEADER_GAP, CARD_HEADER_HEIGHT, CARD_PADDING, ELEVATION_FLOATING_CARD, RADIUS_CONTROL, RADIUS_OVERLAY, SPACE_S};
use crate::drawing::geometry::{Color, Insets, Rect};
use crate::drawing::painter::{Align, Border, Corners};
use crate::drawing::text::Fonts;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CardStyle {
    #[default]
    Standard,
    // Opaque and raised, for over a 3D view.
    Floating,
    // CardBackgroundFillColorSecondary, for a card inside a card.
    Subtle,
}

#[derive(Clone, Debug)]
pub struct Card {
    title: String,
    subtitle: Option<String>,
    style: CardStyle,
    padding: f32,
    // Leaves a hole in the content for a 3D view.
    view: bool,
}

impl Card {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            subtitle: None,
            style: CardStyle::Standard,
            padding: CARD_PADDING,
            view: false,
        }
    }

    // A caption after the title, like a count or what the card is for.
    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
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

    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    // A hole in the content for a 3D view, which the program's render draws
    // into. Clear the view to Theme::card_opaque for it to look like the
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

    // Where the card's parts are, in the rectangle.
    pub fn area(&self, ui: &mut Ui, rect: Rect) -> CardArea {
        self.area_with(ui.fonts(), rect)
    }

    fn area_with(&self, fonts: &mut Fonts, rect: Rect) -> CardArea {
        let inner = rect.inset(self.padding);
        if self.title.is_empty() {
            return CardArea {
                rect,
                header: Rect::new(inner.x, inner.y, 0.0, 0.0),
                content: inner,
            };
        }
        let mut title_width = fonts.measure(&self.title, BODY_STRONG).width;
        if let Some(subtitle) = &self.subtitle {
            title_width += SPACE_S + fonts.measure(subtitle, CAPTION).width;
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

impl<M: 'static> Widget<M> for Card {
    fn kind(&self) -> &'static str {
        "Card"
    }

    // A named card is a target that scripts see; every card is a surface
    // that the pointer stops at.
    fn sense(&self) -> Option<Sense> {
        Some(Sense::NONE)
    }

    fn covers(&self) -> bool {
        true
    }

    fn draw(&self, ctx: &mut Draw) {
        let rect = ctx.rect();
        let area = self.area_with(ctx.fonts(), rect);
        let mut painter = ctx.painter();
        let theme = painter.theme();
        let (fill, stroke) = match self.style {
            CardStyle::Standard => (theme.card_background, theme.card_stroke),
            CardStyle::Floating => (theme.acrylic, theme.surface_stroke_flyout),
            CardStyle::Subtle => (theme.card_background_secondary, theme.card_stroke),
        };
        if self.style == CardStyle::Floating {
            painter.shadow(rect, RADIUS_OVERLAY, ELEVATION_FLOATING_CARD);
        }
        if self.view {
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
        } else {
            painter.shape(rect, RADIUS_OVERLAY, fill, Some(Border::new(stroke, 1.0)));
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

impl<M: 'static> Tree<M> {
    // A card in the rectangle, with the children that `build` adds, given
    // where the card's parts are. A named card is a target of kind "Card".
    pub fn card(&mut self, ui: &mut Ui, name: Option<&str>, rect: Rect, card: Card, build: impl FnOnce(&mut Ui, &mut Tree<M>, CardArea)) -> &mut Element<M> {
        let area = card.area(ui, rect);
        self.container(name, rect, card, |children| build(ui, children, area))
    }

    // A card at the rectangle's top, as tall as its content: `build` lays
    // the content out from the top of the area it's given, which may be as
    // tall as the rectangle, and returns where the content ends. Returns
    // where the card is.
    pub fn sized_card(&mut self, ui: &mut Ui, name: Option<&str>, rect: Rect, card: Card, build: impl FnOnce(&mut Ui, &mut Tree<M>, CardArea) -> f32) -> Rect {
        let area = card.area(ui, rect);
        let padding = card.padding;
        let mut bottom = rect.y;
        self.container(name, rect, card, |children| {
            bottom = build(ui, children, area);
        });
        let bottom = (bottom + padding).min(rect.bottom());
        let sized = Rect::new(rect.x, rect.y, rect.width, (bottom - rect.y).max(0.0));
        if let Some(element) = self.elements.last_mut() {
            element.rect = sized;
        }
        sized
    }
}
