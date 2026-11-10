#include "UserInterface/Widgets/Card.h"

#include "Base/Panic.h"
#include "UserInterface/Core/Interface.h"
#include "UserInterface/Design/Tokens.h"
#include "UserInterface/Drawing/Painter.h"

#include <algorithm>

namespace user_interface {

Card::Card(std::string title) : title(std::move(title)), style_(CardStyle::Standard), padding_(tokens::CardPadding), view(false) {}

Card&& Card::subtitle(std::string subtitle) && {
    subtitle_ = std::move(subtitle);
    return std::move(*this);
}

Card&& Card::floating() && {
    style_ = CardStyle::Floating;
    return std::move(*this);
}

Card&& Card::subtle() && {
    style_ = CardStyle::Subtle;
    return std::move(*this);
}

Card&& Card::style(CardStyle style) && {
    style_ = style;
    return std::move(*this);
}

Card&& Card::padding(float padding) && {
    padding_ = padding;
    return std::move(*this);
}

Card&& Card::with_view() && {
    view = true;
    return std::move(*this);
}

float Card::height_around(float content_height) const {
    float header = title.empty() ? 0.0f : tokens::CardHeaderHeight + tokens::CardHeaderGap;
    return padding_ * 2.0f + header + content_height;
}

CardArea Card::area(Ui& ui, Rect rect) const {
    return area_with(ui.fonts(), rect);
}

CardArea Card::area_with(Fonts& fonts, Rect rect) const {
    Rect inner = rect.inset(padding_);
    if (title.empty()) {
        return CardArea{
            .rect = rect,
            .header = Rect(inner.x, inner.y, 0.0f, 0.0f),
            .content = inner,
        };
    }
    float title_width = fonts.measure(title, tokens::BodyStrong).width;
    if (subtitle_) {
        title_width += tokens::SpaceS + fonts.measure(*subtitle_, tokens::Caption).width;
    }
    Rect header_row = inner.top_strip(tokens::CardHeaderHeight);
    float header_left = std::min(header_row.x + title_width + tokens::SpaceS, header_row.right());
    return CardArea{
        .rect = rect,
        .header = Rect(header_left, header_row.y, header_row.right() - header_left, header_row.height),
        .content = inner.pad(Insets(0.0f, tokens::CardHeaderHeight + tokens::CardHeaderGap, 0.0f, 0.0f)),
    };
}

std::string_view Card::kind() const {
    return "Card";
}

std::optional<Sense> Card::sense() const {
    return Sense::None;
}

bool Card::covers() const {
    return true;
}

void Card::draw(Draw& ctx) const {
    Rect rect = ctx.rect();
    CardArea area = area_with(ctx.fonts(), rect);
    Painter painter = ctx.painter();
    const Theme& theme = painter.theme();
    auto [fill, stroke] = [&]() -> std::pair<Color, Color> {
        switch (style_) {
        case CardStyle::Standard:
            return {theme.card_background, theme.card_stroke};
        case CardStyle::Floating:
            return {theme.acrylic, theme.surface_stroke_flyout};
        case CardStyle::Subtle:
            return {theme.card_background_secondary, theme.card_stroke};
        }
        base::unreachable();
    }();
    if (style_ == CardStyle::Floating) {
        painter.shadow(rect, tokens::RadiusOverlay, tokens::ElevationFloatingCard);
    }
    if (view) {
        // The fill goes around the view, which the program draws.
        Rect hole = area.content;
        Rect top(rect.x, rect.y, rect.width, hole.y - rect.y);
        Rect bottom(rect.x, hole.bottom(), rect.width, rect.bottom() - hole.bottom());
        Rect left(rect.x, hole.y, hole.x - rect.x, hole.height);
        Rect right(hole.right(), hole.y, rect.right() - hole.right(), hole.height);
        painter.shape(top, Corners::top(tokens::RadiusOverlay), fill, std::nullopt);
        painter.shape(bottom, Corners::bottom(tokens::RadiusOverlay), fill, std::nullopt);
        painter.fill(left, fill);
        painter.fill(right, fill);
        painter.shape(rect, tokens::RadiusOverlay, Color::Transparent, Border(stroke, 1.0f));
        painter.shape(hole.expand(1.0f), tokens::RadiusControl, Color::Transparent, Border(theme.card_stroke, 1.0f));
    } else {
        painter.shape(rect, tokens::RadiusOverlay, fill, Border(stroke, 1.0f));
    }
    if (!title.empty()) {
        Rect header(area.content.x, rect.y + padding_, area.content.width, tokens::CardHeaderHeight);
        Rect title_rect = painter.text_in(header, title, tokens::BodyStrong, theme.text_primary, Align::Start);
        if (subtitle_) {
            Rect after(title_rect.right() + tokens::SpaceS, header.y, header.right() - title_rect.right() - tokens::SpaceS, header.height);
            painter.text_fit(after, *subtitle_, tokens::Caption, theme.text_secondary, Align::Start);
        }
    }
}

} // namespace user_interface
