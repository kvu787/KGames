#include "UserInterface/Drawing/Painter.h"

#include "Base/Numbers.h"
#include "Base/Text.h"

#include <algorithm>
#include <cmath>
#include <format>
#include <limits>

namespace user_interface {

namespace {

// The kinds of instance, as the shader numbers them.
constexpr std::uint32_t KindShape = 0;
constexpr std::uint32_t KindShadow = 1;
constexpr std::uint32_t KindGlyph = 2;
constexpr std::uint32_t KindLine = 3;
constexpr std::uint32_t KindArc = 4;
constexpr std::uint32_t KindPolygon = 5;

// A full turn, in radians.
constexpr float Tau = 6.28318530717958647692528676655900577f;

// The corners' radii, top left first, clockwise.
std::array<float, 4> array(Corners corners) {
    return {corners.top_left, corners.top_right, corners.bottom_right, corners.bottom_left};
}

bool is_visible(const Brush& brush) {
    return base::match(
        brush, [](const Brush::Solid& solid) { return solid.color.a > 0.0f; }, [](const Brush::Linear& linear) { return linear.start.a > 0.0f || linear.end.a > 0.0f; });
}

// The colors and the line, as the shader takes them.
std::pair<std::array<std::uint32_t, 2>, std::array<float, 4>> encode(const Brush& brush) {
    using Encoded = std::pair<std::array<std::uint32_t, 2>, std::array<float, 4>>;
    return base::match(
        brush,
        [](const Brush::Solid& solid) {
            std::uint32_t packed = solid.color.pack_premultiplied();
            return Encoded{{packed, packed}, {0.0f, 0.0f, 0.0f, 1.0f}};
        },
        [](const Brush::Linear& linear) {
            return Encoded{{linear.start.pack_premultiplied(), linear.end.pack_premultiplied()}, {linear.from.x, linear.from.y, linear.to.x, linear.to.y}};
        });
}

// char::is_whitespace: Unicode's White_Space.
bool is_whitespace(char32_t character) {
    return character == U' ' || (character >= U'\t' && character <= U'\r') || character == 0x85 || character == 0xA0 || character == 0x1680 ||
           (character >= 0x2000 && character <= 0x200A) || character == 0x2028 || character == 0x2029 || character == 0x202F || character == 0x205F || character == 0x3000;
}

// Without the whitespace at its end: str::trim_end.
std::string_view trim_end(std::string_view text) {
    while (!text.empty()) {
        // The last character starts at the last byte that isn't a
        // continuation byte.
        size_t start = text.size() - 1;
        while (start > 0 && (static_cast<unsigned char>(text[start]) & 0xC0) == 0x80) {
            start -= 1;
        }
        if (!is_whitespace(base::decode_char(text.substr(start)).character)) {
            break;
        }
        text = text.substr(0, start);
    }
    return text;
}

// The part of the segment inside the rectangle, by Liang and Barsky's
// clipping, or nothing if none of it is.
std::optional<std::pair<Point, Point>> clip_segment(Point from, Point to, Rect rect) {
    Point delta = to - from;
    float enter = 0.0f;
    float leave_ = 1.0f;
    const std::array<std::pair<float, float>, 4> edges{{
        {-delta.x, from.x - rect.left()},
        {delta.x, rect.right() - from.x},
        {-delta.y, from.y - rect.top()},
        {delta.y, rect.bottom() - from.y},
    }};
    for (auto [p, q] : edges) {
        if (p == 0.0f) {
            if (q < 0.0f) {
                return std::nullopt;
            }
        } else {
            float t = q / p;
            // With what's kept first, a t that isn't a number is ignored, as
            // f32::max and f32::min ignore it.
            if (p < 0.0f) {
                enter = std::max(enter, t);
            } else {
                leave_ = std::min(leave_, t);
            }
        }
    }
    if (enter <= leave_) {
        return std::pair(from + delta * enter, from + delta * leave_);
    }
    return std::nullopt;
}

} // namespace

Brush Brush::vertical(Rect rect, Color top, Color bottom) {
    return Brush::Linear{
        .from = Point(rect.x, rect.y),
        .to = Point(rect.x, rect.bottom()),
        .start = top,
        .end = bottom,
    };
}

Brush Brush::horizontal(Rect rect, Color left, Color right) {
    return Brush::Linear{
        .from = Point(rect.x, rect.y),
        .to = Point(rect.right(), rect.y),
        .start = left,
        .end = right,
    };
}

Border Border::sides(const Brush& brush, Insets widths) {
    Border border(brush, 0.0f);
    border.widths = widths;
    return border;
}

Fitted Fitted::borrowed(std::string_view text) {
    Fitted fitted;
    fitted.borrowed_ = text;
    return fitted;
}

Fitted Fitted::owned(std::string text) {
    Fitted fitted;
    fitted.owned_ = std::move(text);
    return fitted;
}

bool Fitted::is_owned() const {
    return owned_.has_value();
}

std::string_view Fitted::view() const {
    return owned_ ? std::string_view(*owned_) : borrowed_;
}

std::string Fitted::into_owned() && {
    return owned_ ? std::move(*owned_) : std::string(borrowed_);
}

void DrawList::clear(Size size) {
    for (std::vector<Instance>& layer : layers) {
        layer.clear();
    }
    clips.clear();
    points.clear();
    clip_stack.clear();
    clips.push_back({0.0f, 0.0f, size.width, size.height});
    layer_ = Layer::Base;
}

std::span<const Instance> DrawList::layer_instances(Layer layer) const {
    return layers[static_cast<size_t>(layer)];
}

size_t DrawList::instance_count() const {
    size_t count = 0;
    for (const std::vector<Instance>& layer : layers) {
        count += layer.size();
    }
    return count;
}

Layer DrawList::layer() const {
    return layer_;
}

size_t DrawList::layer_len() const {
    return layers[static_cast<size_t>(layer_)].size();
}

void DrawList::move_to_back(size_t first, size_t middle) {
    std::vector<Instance>& layer = layers[static_cast<size_t>(layer_)];
    size_t moved = layer.size() - middle;
    // The last `moved` instances go to before the ones from `first`.
    std::rotate(layer.begin() + static_cast<std::ptrdiff_t>(first), layer.end() - static_cast<std::ptrdiff_t>(moved), layer.end());
}

void DrawList::set_layer(Layer layer) {
    layer_ = layer;
}

std::uint32_t DrawList::clip_index() const {
    return clip_stack.empty() ? 0 : clip_stack.back();
}

Rect DrawList::clip() const {
    auto [left, top, right, bottom] = clips[clip_index()];
    return Rect(left, top, right - left, bottom - top);
}

void DrawList::push_clip(Rect rect, bool absolute) {
    Rect clipped = absolute ? rect : rect.intersect(clip());
    clips.push_back({clipped.left(), clipped.top(), clipped.right(), clipped.bottom()});
    std::uint16_t index = static_cast<std::uint16_t>(std::min<size_t>(clips.size() - 1, std::numeric_limits<std::uint16_t>::max()));
    clip_stack.push_back(index);
}

void DrawList::pop_clip() {
    if (!clip_stack.empty()) {
        clip_stack.pop_back();
    }
}

void DrawList::push(std::uint32_t kind, const Instance& instance) {
    std::vector<Instance>& layer = layers[static_cast<size_t>(layer_)];
    layer.push_back(instance);
    layer.back().kind = kind | (clip_index() << 16);
}

const Theme& Painter::theme() const {
    return theme_;
}

Layer Painter::layer() const {
    return list.layer();
}

void Painter::set_layer(Layer layer) {
    list.set_layer(layer);
}

Rect Painter::clip() const {
    return list.clip();
}

void Painter::push_clip(Rect rect) {
    list.push_clip(rect, false);
}

void Painter::pop_clip() {
    list.pop_clip();
}

void Painter::fill(Rect rect, Color color) {
    shape(rect, Corners::Zero, color, std::nullopt);
}

void Painter::rounded(Rect rect, Corners corners, Color color) {
    shape(rect, corners, color, std::nullopt);
}

void Painter::shape(Rect rect, Corners corners, const Brush& fill, const std::optional<Border>& border) {
    // The border, if any of it shows.
    const Border* shown = border && is_visible(border->brush) && !border->widths.is_zero() ? &*border : nullptr;
    if (rect.is_empty() || (!is_visible(fill) && shown == nullptr)) {
        return;
    }
    float half = std::min(rect.width, rect.height) / 2.0f;
    std::array<float, 4> radii = array(corners);
    for (float& radius : radii) {
        radius = std::clamp(radius, 0.0f, half);
    }
    auto [fill_colors, fill_line] = encode(fill);
    std::array<std::uint32_t, 2> border_colors{0, 0};
    std::array<float, 4> border_line{0.0f, 0.0f, 0.0f, 1.0f};
    std::array<float, 4> widths{};
    if (shown != nullptr) {
        auto [colors, line] = encode(shown->brush);
        border_colors = colors;
        border_line = line;
        widths = {shown->widths.left, shown->widths.top, shown->widths.right, shown->widths.bottom};
    }
    list.push(KindShape, Instance{
                             .bounds = {rect.left(), rect.top(), rect.right(), rect.bottom()},
                             .radii = radii,
                             .border = widths,
                             .fill_line = fill_line,
                             .border_line = border_line,
                             .colors = {fill_colors[0], fill_colors[1], border_colors[0], border_colors[1]},
                         });
}

void Painter::outline(Rect rect, float radius, float width, Color color) {
    shape(rect.expand(width), radius + width, Color::Transparent, Border(color, width));
}

void Painter::shadow(Rect rect, float radius, const tokens::Elevation& elevation) {
    for (const tokens::Shadow& shadow : elevation.shadows) {
        float opacity = 0.0f;
        switch (theme_.kind) {
        case ThemeKind::Light:
            opacity = shadow.opacity_light;
            break;
        case ThemeKind::Dark:
            opacity = shadow.opacity_dark;
            break;
        }
        if (opacity <= 0.0f) {
            continue;
        }
        list.push(KindShadow, Instance{
                                  .bounds = {rect.left(), rect.top(), rect.right(), rect.bottom()},
                                  .radii = {radius, shadow.sigma(), shadow.offset_y, 0.0f},
                                  .colors = {Color::Black.with_alpha(opacity).pack_premultiplied(), 0, 0, 0},
                              });
    }
}

void Painter::circle(Point center, float radius, Color color) {
    Rect rect = Rect::from_center(center, Size::square(radius * 2.0f));
    shape(rect, radius, color, std::nullopt);
}

void Painter::ring(Point center, float radius, float width, Color color) {
    Rect rect = Rect::from_center(center, Size::square(radius * 2.0f));
    shape(rect, radius, Color::Transparent, Border(color, width));
}

void Painter::arc(Point center, float radius, float width, float start, float sweep, Color color) {
    if (sweep == 0.0f) {
        return;
    }
    float from = start;
    float turn = sweep;
    if (sweep < 0.0f) {
        from = start + sweep;
        turn = -sweep;
    }
    list.push(KindArc, Instance{
                           .bounds = {center.x, center.y, radius, width},
                           // The full turn first, so that a sweep that isn't a
                           // number is a full turn, as f32::min makes it.
                           .radii = {base::rem_euclid(from, Tau), std::min(Tau, turn), 0.0f, 0.0f},
                           .colors = {color.pack_premultiplied(), 0, 0, 0},
                       });
}

void Painter::line(Point from, Point to, float width, Color color) {
    segment(from, to, width, color, true);
}

void Painter::line_flat(Point from, Point to, float width, Color color) {
    segment(from, to, width, color, false);
}

void Painter::polyline(std::span<const Point> points, float width, Color color) {
    for (size_t index = 0; index + 1 < points.size(); index++) {
        line(points[index], points[index + 1], width, color);
    }
}

void Painter::segment(Point from, Point to, float width, Color color, bool round) {
    // Only the part in the clip is drawn, so that a line that reaches far
    // past the window, like a grid's in a zoomed view, isn't a huge quad.
    std::optional<std::pair<Point, Point>> clipped = clip_segment(from, to, clip().expand(width));
    if (!clipped) {
        return;
    }
    auto [start, end] = *clipped;
    list.push(KindLine, Instance{
                            .bounds = {start.x, start.y, end.x, end.y},
                            .radii = {width, round ? 1.0f : 0.0f, 0.0f, 0.0f},
                            .colors = {color.pack_premultiplied(), 0, 0, 0},
                        });
}

void Painter::polygon(std::span<const Point> points, Color color) {
    if (points.size() < 3 || color.a <= 0.0f) {
        return;
    }
    size_t first = list.points.size();
    // The shader takes the corners clockwise on screen, where y is down.
    float area = 0.0f;
    for (size_t index = 0; index < points.size(); index++) {
        Point a = points[index];
        Point b = points[(index + 1) % points.size()];
        area += a.x * b.y - b.x * a.y;
    }
    if (area >= 0.0f) {
        for (Point point : points) {
            list.points.push_back({point.x, point.y});
        }
    } else {
        for (size_t index = points.size(); index > 0; index--) {
            Point point = points[index - 1];
            list.points.push_back({point.x, point.y});
        }
    }
    std::array<float, 4> bounds{std::numeric_limits<float>::max(), std::numeric_limits<float>::max(), std::numeric_limits<float>::lowest(), std::numeric_limits<float>::lowest()};
    for (Point point : points) {
        bounds = {std::min(bounds[0], point.x), std::min(bounds[1], point.y), std::max(bounds[2], point.x), std::max(bounds[3], point.y)};
    }
    list.push(KindPolygon, Instance{
                               .bounds = bounds,
                               .radii = {static_cast<float>(first), static_cast<float>(points.size()), 0.0f, 0.0f},
                               .colors = {color.pack_premultiplied(), 0, 0, 0},
                           });
}

Fonts& Painter::fonts() {
    return fonts_;
}

Size Painter::measure(std::string_view text, TextStyle style) {
    return fonts_.measure(text, style);
}

Size Painter::measure_wrapped(std::string_view text, TextStyle style, float width) {
    return fonts_.measure_wrapped(text, style, width);
}

Size Painter::text(Point origin, std::string_view text, TextStyle style, Color color) {
    return glyphs(origin, text, style, std::nullopt, color);
}

Size Painter::text_wrapped(Rect rect, std::string_view text, TextStyle style, Color color) {
    // The 1 first, so that a width that isn't a number is 1, as f32::max
    // makes it.
    return glyphs(rect.origin(), text, style, std::max(1.0f, rect.width), color);
}

Rect Painter::text_in(Rect rect, std::string_view text, TextStyle style, Color color, Align align) {
    Size size = measure(text, style);
    float x = rect.x;
    switch (align) {
    case Align::Start:
        break;
    case Align::Center:
        x = rect.x + (rect.width - size.width) / 2.0f;
        break;
    case Align::End:
        x = rect.right() - size.width;
        break;
    }
    float y = std::round(rect.y + (rect.height - size.height) / 2.0f);
    glyphs(Point(x, y), text, style, std::nullopt, color);
    return Rect(x, y, size.width, size.height);
}

std::pair<Rect, bool> Painter::text_fit(Rect rect, std::string_view text, TextStyle style, Color color, Align align) {
    Fitted fitted = fit(text, style, rect.width);
    bool cut = fitted.is_owned();
    return {text_in(rect, fitted.view(), style, color, align), cut};
}

Fitted Painter::fit(std::string_view text, TextStyle style, float width) {
    return fit_text(fonts_, text, style, width);
}

float Painter::spans(Point origin, std::span<const Span> spans) {
    float line_height = 0.0f;
    for (const Span& span : spans) {
        line_height = std::max(line_height, span.style.line_height);
    }
    float x = origin.x;
    for (const Span& span : spans) {
        float y = origin.y + std::round((line_height - span.style.line_height) / 2.0f);
        x += text(Point(x, y), span.text, span.style, span.color).width;
    }
    return x - origin.x;
}

Size Painter::glyphs(Point origin, std::string_view text, TextStyle style, std::optional<float> wrap, Color color) {
    if (text.empty()) {
        return Size(0.0f, style.line_height);
    }
    std::uint32_t packed = color.pack_premultiplied();
    // Direct2D's grayscale text blending, as Windows Terminal's copy of
    // DirectWrite's has it: enhanced contrast, the default 1 for dark
    // text and none for light, and the text's intensity, for the alpha
    // correction that the shader applies with them.
    float lightness = 0.30f * color.r + 0.59f * color.g + 0.11f * color.b;
    float contrast = std::clamp(4.0f * (0.75f - lightness), 0.0f, 1.0f);
    float intensity = 0.25f * color.r + 0.5f * color.g + 0.25f * color.b;
    DrawList& draw_list = list;
    return fonts_.quads(text, style, wrap, origin, [&](const GlyphQuad& quad) {
        draw_list.push(KindGlyph, Instance{
                                      .bounds = {quad.x, quad.y, quad.x + quad.width, quad.y + quad.height},
                                      .radii = {quad.atlas_x, quad.atlas_y, contrast, intensity},
                                      .colors = {packed, 0, 0, 0},
                                  });
    });
}

void Painter::icon(Point center, Icon icon, float size, Color color) {
    // One character, which a string holds without memory of its own.
    std::string text;
    base::push_char(text, glyph(icon));
    TextStyle style = TextStyle::icons(size);
    Size measured = measure(text, style);
    Point origin(std::round(center.x - measured.width / 2.0f), std::round(center.y - size / 2.0f));
    glyphs(origin, text, style, std::nullopt, color);
}

Fitted fit_text(Fonts& fonts, std::string_view text, TextStyle style, float width) {
    if (fonts.measure(text, style).width <= width) {
        return Fitted::borrowed(text);
    }
    // Where each character but the first starts.
    std::vector<size_t> boundaries;
    for (size_t offset = 0; offset < text.size(); offset += base::decode_char(text.substr(offset)).length) {
        if (offset > 0) {
            boundaries.push_back(offset);
        }
    }
    // The most characters whose text with an ellipsis fits.
    size_t low = 0;
    size_t high = boundaries.size();
    while (low < high) {
        size_t middle = (low + high + 1) / 2;
        std::string candidate = std::format("{}…", trim_end(text.substr(0, boundaries[middle - 1])));
        if (fonts.measure(candidate, style).width <= width) {
            low = middle;
        } else {
            high = middle - 1;
        }
    }
    return Fitted::owned(low == 0 ? std::string("…") : std::format("{}…", trim_end(text.substr(0, boundaries[low - 1]))));
}

} // namespace user_interface
