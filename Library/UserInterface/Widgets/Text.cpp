#include "UserInterface/Widgets/Text.h"

#include "Base/Numbers.h"
#include "Base/Panic.h"
#include "Base/Text.h"
#include "Base/Vec.h"
#include "UserInterface/Design/Controls.h"
#include "UserInterface/Widgets/InfoBar.h"

#include <algorithm>
#include <cmath>
#include <cstddef>

namespace user_interface {

namespace {

constexpr float KeycapPadding = 6.0f;
constexpr float KeycapGap = 4.0f;

// The keys of a combination like "Ctrl+S" or "RB+Left stick", each on its
// own keycap. "+" alone is the plus key's.
std::vector<std::string> keys_of(std::string_view keys) {
    std::vector<std::string> parts;
    for (std::string_view key : base::split(keys, '+')) {
        parts.emplace_back(base::trim(key));
    }
    if (parts.size() > 1 && std::ranges::all_of(parts, [](const std::string& part) { return !part.empty(); })) {
        return parts;
    }
    return base::vec(std::string(keys));
}

} // namespace

Color TextColor::resolve(const Theme& theme, bool enabled) const {
    return base::match(
        *this, [&](const Primary&) { return enabled ? theme.text_primary : theme.text_disabled; },
        [&](const Secondary&) { return enabled ? theme.text_secondary : theme.text_disabled; }, [&](const Tertiary&) { return theme.text_tertiary; },
        [&](const Disabled&) { return theme.text_disabled; }, [&](const Accent&) { return theme.accent_text; }, [&](const Custom& custom) { return custom.color; });
}

Text::Text(std::string text) : text_(std::move(text)), style_(tokens::Body), color_(TextColor::Primary{}), align_(Align::Start), flow(Flow::Fit), hoverable_(false) {}

Text Text::label(std::string text) {
    return Text(std::move(text));
}

Text Text::heading(std::string text) {
    return Text(std::move(text)).style(tokens::BodyStrong);
}

Text Text::caption(std::string text) {
    return Text(std::move(text)).style(tokens::Caption).color(TextColor::Secondary{});
}

Text Text::paragraph(std::string text) {
    return Text(std::move(text)).wrap();
}

Text&& Text::style(TextStyle style) && {
    style_ = style;
    return std::move(*this);
}

Text&& Text::color(TextColor color) && {
    color_ = color;
    return std::move(*this);
}

Text&& Text::secondary() && {
    return std::move(*this).color(TextColor::Secondary{});
}

Text&& Text::tertiary() && {
    return std::move(*this).color(TextColor::Tertiary{});
}

Text&& Text::align(Align align) && {
    align_ = align;
    return std::move(*this);
}

Text&& Text::center() && {
    return std::move(*this).align(Align::Center);
}

Text&& Text::end() && {
    return std::move(*this).align(Align::End);
}

Text&& Text::wrap() && {
    flow = Flow::Wrap;
    return std::move(*this);
}

Text&& Text::hoverable() && {
    hoverable_ = true;
    return std::move(*this);
}

std::string_view Text::text() const {
    return text_;
}

std::optional<Sense> Text::sense() const {
    if (hoverable_) {
        return Sense::None;
    }
    return std::nullopt;
}

std::string_view Text::kind() const {
    return "TextBlock";
}

std::optional<std::string> Text::label() const {
    return text_;
}

void Text::draw(Draw& ctx) const {
    Color color = color_.resolve(ctx.theme(), ctx.enabled());
    Rect rect = ctx.rect();
    Painter painter = ctx.painter();
    switch (flow) {
    case Flow::Fit:
        painter.text_fit(rect, text_, style_, color, align_);
        break;
    case Flow::Wrap:
        painter.text_wrapped(rect, text_, style_, color);
        break;
    }
}

Field::Field(std::string label, float label_width, std::string value) : label_(std::move(label)), label_width(label_width), value(std::move(value)) {}

std::string_view Field::kind() const {
    return "Field";
}

std::optional<std::string> Field::label() const {
    return value;
}

void Field::draw(Draw& ctx) const {
    Rect rect = ctx.rect();
    bool enabled = ctx.enabled();
    Color secondary = TextColor(TextColor::Secondary{}).resolve(ctx.theme(), enabled);
    Color primary = TextColor(TextColor::Primary{}).resolve(ctx.theme(), enabled);
    Painter painter = ctx.painter();
    painter.text_fit(rect.left_strip(label_width - tokens::SpaceXs), label_, tokens::Body, secondary, Align::Start);
    Rect value_rect(rect.x + label_width, rect.y, std::max(rect.width - label_width, 0.0f), rect.height);
    painter.text_fit(value_rect, value, tokens::Body, primary, Align::Start);
}

Spans::Spans(std::vector<Run> runs) : runs(std::move(runs)) {}

float Spans::width(Fonts& fonts) const {
    // From -0, as Rust's sum of floats is.
    float total = -0.0f;
    for (const Run& run : runs) {
        total += fonts.measure(run.text, run.style).width;
    }
    return total;
}

std::string_view Spans::kind() const {
    return "TextBlock";
}

void Spans::draw(Draw& ctx) const {
    Point origin = ctx.rect().origin();
    bool enabled = ctx.enabled();
    const Theme& theme = ctx.theme();
    // Each span borrows its run's text, which this widget has.
    std::vector<Span> spans;
    spans.reserve(runs.size());
    for (const Run& run : runs) {
        spans.emplace_back(run.text, run.style, run.color.resolve(theme, enabled));
    }
    ctx.painter().spans(origin, spans);
}

IconView::IconView(Icon icon, float size) : icon(icon), size(size), color_(TextColor::Primary{}) {}

IconView&& IconView::color(TextColor color) && {
    color_ = color;
    return std::move(*this);
}

std::string_view IconView::kind() const {
    return "Icon";
}

void IconView::draw(Draw& ctx) const {
    Color color = color_.resolve(ctx.theme(), ctx.enabled());
    Point center = ctx.rect().center();
    ctx.painter().icon(center, icon, size, color);
}

StatusIcon::StatusIcon(Severity severity, float size) : severity(severity), size(size) {}

void status_icon(Painter& painter, Point center, Severity severity, float size) {
    const Theme& theme = painter.theme();
    Icon mark = [&] {
        switch (severity) {
        case Severity::Informational:
            return Icon::StatusInfo;
        case Severity::Success:
            return Icon::StatusSuccess;
        case Severity::Warning:
            return Icon::StatusWarning;
        case Severity::Error:
            return Icon::StatusError;
        }
        base::unreachable();
    }();
    painter.icon(center, Icon::StatusCircle, size, theme.status(severity));
    painter.icon(center, mark, size, theme.text_inverse);
}

std::string_view StatusIcon::kind() const {
    return "StatusIcon";
}

void StatusIcon::draw(Draw& ctx) const {
    Point center = ctx.rect().center();
    Painter painter = ctx.painter();
    status_icon(painter, center, severity, size);
}

std::string_view Divider::kind() const {
    return "Divider";
}

void Divider::draw(Draw& ctx) const {
    Rect rect = ctx.rect();
    Color color = ctx.theme().divider;
    Rect line =
        rect.width >= rect.height ? Rect(rect.x, std::round(rect.center().y - 0.5f), rect.width, 1.0f) : Rect(std::round(rect.center().x - 0.5f), rect.y, 1.0f, rect.height);
    ctx.painter().fill(line, color);
}

Swatch::Swatch(Color color) : color(color) {}

std::string_view Swatch::kind() const {
    return "Swatch";
}

std::optional<std::string> Swatch::label() const {
    return color.to_hex();
}

void Swatch::draw(Draw& ctx) const {
    Rect rect = ctx.rect();
    Painter painter = ctx.painter();
    const Theme& theme = painter.theme();
    painter.shape(rect, tokens::RadiusControl, color, Border(theme.control_stroke_secondary, 1.0f));
}

Keycaps::Keycaps(std::string keys) : keys(std::move(keys)) {}

float Keycaps::key_width(Fonts& fonts, std::string_view key) {
    return keycap_width(fonts, key);
}

float Keycaps::width(Fonts& fonts, std::string_view keys) {
    return keys_width(fonts, keys);
}

std::string_view Keycaps::kind() const {
    return "Keycaps";
}

std::optional<std::string> Keycaps::label() const {
    return keys;
}

void Keycaps::draw(Draw& ctx) const {
    Point origin = ctx.rect().origin();
    Painter painter = ctx.painter();
    draw_keys(painter, origin, keys);
}

float draw_keycap(Painter& painter, Point origin, std::string_view key) {
    float width = keycap_width(painter.fonts(), key);
    Rect rect(origin.x, origin.y, width, KeycapHeight);
    const Theme& theme = painter.theme();
    painter.shape(rect, tokens::RadiusControl, theme.control_fill, Border(theme.control_stroke_secondary, 1.0f));
    painter.text_in(rect, key, tokens::Caption, theme.text_secondary, Align::Center);
    return width;
}

float keycap_width(Fonts& fonts, std::string_view key) {
    return std::max(std::ceil(fonts.measure(key, tokens::Caption).width) + KeycapPadding * 2.0f, KeycapHeight);
}

float draw_keys(Painter& painter, Point origin, std::string_view keys) {
    float x = origin.x;
    std::vector<std::string> names = keys_of(keys);
    for (size_t index = 0; index < names.size(); index++) {
        if (index > 0) {
            x += KeycapGap;
        }
        x += draw_keycap(painter, Point(x, origin.y), names[index]);
    }
    return x - origin.x;
}

float keys_width(Fonts& fonts, std::string_view keys) {
    std::vector<std::string> names = keys_of(keys);
    // From -0, as Rust's sum of floats is.
    float total = -0.0f;
    for (const std::string& key : names) {
        total += keycap_width(fonts, key);
    }
    return total + KeycapGap * static_cast<float>(base::saturating_sub(names.size(), size_t{1}));
}

Badge::Badge(std::string text) : text(std::move(text)) {}

Badge Badge::dot() {
    return Badge("");
}

Size Badge::size(Fonts& fonts, std::string_view text) {
    if (text.empty()) {
        return Size::square(controls::InfoBadgeDot);
    }
    TextStyle style(tokens::Caption.font, tokens::Caption.weight_, controls::InfoBadgeFontSize, controls::InfoBadgeHeight);
    float width = std::max(std::ceil(fonts.measure(text, style).width) + controls::InfoBadgeTextMargin.horizontal(), controls::InfoBadgeHeight);
    return Size(width, controls::InfoBadgeHeight);
}

std::string_view Badge::kind() const {
    return "InfoBadge";
}

std::optional<std::string> Badge::label() const {
    return text;
}

void Badge::draw(Draw& ctx) const {
    Point center = ctx.rect().center();
    Painter painter = ctx.painter();
    const Theme& theme = painter.theme();
    if (text.empty()) {
        painter.circle(center, controls::InfoBadgeDot / 2.0f, theme.accent);
        return;
    }
    TextStyle style(tokens::Caption.font, tokens::Caption.weight_, controls::InfoBadgeFontSize, controls::InfoBadgeHeight);
    float width = std::max(std::ceil(painter.measure(text, style).width) + controls::InfoBadgeTextMargin.horizontal(), controls::InfoBadgeHeight);
    Rect rect = Rect::from_center(center, Size(width, controls::InfoBadgeHeight)).round();
    painter.rounded(rect, controls::InfoBadgeHeight / 2.0f, theme.accent);
    Rect text_rect(rect.x, rect.y - controls::InfoBadgeTextMargin.bottom / 2.0f, rect.width, rect.height);
    painter.text_in(text_rect, text, style, theme.text_on_accent, Align::Center);
}

Canvas::Canvas(Painting draw) : draw_(std::move(draw)) {}

std::string_view Canvas::kind() const {
    return "Canvas";
}

void Canvas::draw(Draw& ctx) const {
    Rect rect = ctx.rect();
    Painter painter = ctx.painter();
    draw_(painter, rect);
}

} // namespace user_interface
