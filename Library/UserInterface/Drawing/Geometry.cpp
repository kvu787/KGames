#include "UserInterface/Drawing/Geometry.h"

#include "Base/Numbers.h"
#include "Base/Text.h"

#include <algorithm>
#include <charconv>
#include <cmath>
#include <format>
#include <system_error>

namespace user_interface {

float Point::length() const {
    return std::hypot(x, y);
}

float Point::distance(Point other) const {
    return (*this - other).length();
}

Point Point::lerp(Point other, float t) const {
    return *this + (other - *this) * t;
}

Point Point::round() const {
    return Point(std::round(x), std::round(y));
}

std::string describe_value(Point point) {
    return std::format("Point {{ x: {}, y: {} }}", base::debug(point.x), base::debug(point.y));
}

std::string describe_value(Size size) {
    return std::format("Size {{ width: {}, height: {} }}", base::debug(size.width), base::debug(size.height));
}

std::string describe_value(Insets insets) {
    return std::format("Insets {{ left: {}, top: {}, right: {}, bottom: {} }}", base::debug(insets.left), base::debug(insets.top), base::debug(insets.right),
                       base::debug(insets.bottom));
}

Rect Rect::from_corners(Point a, Point b) {
    float left = std::min(a.x, b.x);
    float right = std::max(a.x, b.x);
    float top = std::min(a.y, b.y);
    float bottom = std::max(a.y, b.y);
    return Rect(left, top, right - left, bottom - top);
}

bool Rect::intersects(Rect other) const {
    return !intersect(other).is_empty();
}

Rect Rect::intersect(Rect other) const {
    float left = std::max(x, other.x);
    float top = std::max(y, other.y);
    float right = std::min(this->right(), other.right());
    float bottom = std::min(this->bottom(), other.bottom());
    // fmax, as Rust's max: a size that isn't a number is no size.
    return Rect(left, top, std::fmax(right - left, 0.0f), std::fmax(bottom - top, 0.0f));
}

Rect Rect::union_(Rect other) const {
    if (is_empty()) {
        return other;
    }
    if (other.is_empty()) {
        return *this;
    }
    float left = std::min(x, other.x);
    float top = std::min(y, other.y);
    return Rect(left, top, std::max(right(), other.right()) - left, std::max(bottom(), other.bottom()) - top);
}

Rect Rect::inset(float amount) const {
    return pad(Insets::all(amount));
}

Rect Rect::pad(Insets insets) const {
    return Rect(x + insets.left, y + insets.top, std::fmax(width - insets.horizontal(), 0.0f), std::fmax(height - insets.vertical(), 0.0f));
}

Rect Rect::top_strip(float strip_height) const {
    return Rect(x, y, width, std::min(strip_height, height));
}

Rect Rect::bottom_strip(float strip_height) const {
    float clipped = std::min(strip_height, height);
    return Rect(x, bottom() - clipped, width, clipped);
}

Rect Rect::left_strip(float strip_width) const {
    return Rect(x, y, std::min(strip_width, width), height);
}

Rect Rect::right_strip(float strip_width) const {
    float clipped = std::min(strip_width, width);
    return Rect(right() - clipped, y, clipped, height);
}

Rect Rect::cut_top(float strip_height, float gap) {
    Rect strip = top_strip(strip_height);
    float taken = std::min(strip_height + gap, height);
    y += taken;
    height -= taken;
    return strip;
}

Rect Rect::cut_bottom(float strip_height, float gap) {
    Rect strip = bottom_strip(strip_height);
    height -= std::min(strip_height + gap, height);
    return strip;
}

Rect Rect::cut_left(float strip_width, float gap) {
    Rect strip = left_strip(strip_width);
    float taken = std::min(strip_width + gap, width);
    x += taken;
    width -= taken;
    return strip;
}

Rect Rect::cut_right(float strip_width, float gap) {
    Rect strip = right_strip(strip_width);
    width -= std::min(strip_width + gap, width);
    return strip;
}

std::vector<Rect> Rect::columns_weighted(std::span<const float> weights, float gap) const {
    float total = 0.0f;
    for (float weight : weights) {
        total += weight;
    }
    float available = width - gap * static_cast<float>(base::saturating_sub(weights.size(), size_t{1}));
    float column_x = x;
    float consumed = 0.0f;
    std::vector<Rect> columns;
    columns.reserve(weights.size());
    for (size_t index = 0; index < weights.size(); index++) {
        consumed += weights[index];
        float end = index + 1 == weights.size() ? right() : std::round(x + available * consumed / total + gap * static_cast<float>(index));
        columns.push_back(Rect(column_x, y, end - column_x, height));
        column_x = end + gap;
    }
    return columns;
}

std::vector<float> Rect::split_widths(size_t count, float gap) const {
    float available = std::fmax(width - gap * static_cast<float>(base::saturating_sub(count, size_t{1})), 0.0f);
    std::vector<float> widths;
    widths.reserve(count);
    float used = 0.0f;
    for (size_t index = 0; index < count; index++) {
        float end = std::round(available * static_cast<float>(index + 1) / static_cast<float>(count));
        widths.push_back(end - used);
        used = end;
    }
    return widths;
}

Rect Rect::round() const {
    float left = std::round(x);
    float top = std::round(y);
    return Rect(left, top, std::round(right()) - left, std::round(bottom()) - top);
}

std::string describe_value(Rect rect) {
    return std::format("Rect {{ x: {}, y: {}, width: {}, height: {} }}", base::debug(rect.x), base::debug(rect.y), base::debug(rect.width), base::debug(rect.height));
}

Color Color::from_bytes(std::array<std::uint8_t, 4> bytes) {
    return Color::rgba(static_cast<float>(bytes[0]) / 255.0f, static_cast<float>(bytes[1]) / 255.0f, static_cast<float>(bytes[2]) / 255.0f, static_cast<float>(bytes[3]) / 255.0f);
}

std::optional<Color> Color::parse(std::string_view text) {
    std::string_view digits = base::trim(text);
    while (digits.starts_with('#')) {
        digits.remove_prefix(1);
    }
    for (char character : digits) {
        bool hex_digit = (character >= '0' && character <= '9') || (character >= 'a' && character <= 'f') || (character >= 'A' && character <= 'F');
        if (!hex_digit) {
            return std::nullopt;
        }
    }
    // The digits as a number, or nothing if there are none, or more than 32
    // bits of them.
    std::uint32_t value = 0;
    const char* end = digits.data() + digits.size();
    std::from_chars_result result = std::from_chars(digits.data(), end, value, 16);
    if (digits.empty() || result.ec != std::errc() || result.ptr != end) {
        return std::nullopt;
    }
    switch (digits.size()) {
    case 6:
        return Color::hex(value);
    case 8:
        return Color::argb(value);
    default:
        return std::nullopt;
    }
}

std::string Color::to_hex() const {
    auto [red, green, blue, alpha] = to_bytes();
    return alpha == 255 ? std::format("#{:02X}{:02X}{:02X}", red, green, blue) : std::format("#{:02X}{:02X}{:02X}{:02X}", alpha, red, green, blue);
}

std::array<std::uint8_t, 4> Color::to_bytes() const {
    auto byte = [](float value) {
        return base::saturating_cast<std::uint8_t>(std::round(std::clamp(value, 0.0f, 1.0f) * 255.0f));
    };
    return {byte(r), byte(g), byte(b), byte(a)};
}

std::uint32_t Color::pack_premultiplied() const {
    float alpha = std::clamp(a, 0.0f, 1.0f);
    auto byte = [alpha](float value) {
        return base::saturating_cast<std::uint32_t>(std::round(std::clamp(value, 0.0f, 1.0f) * alpha * 255.0f));
    };
    return byte(r) | (byte(g) << 8) | (byte(b) << 16) | (base::saturating_cast<std::uint32_t>(std::round(alpha * 255.0f)) << 24);
}

Color Color::lerp(Color other, float t) const {
    auto mix = [t](float from, float to) {
        return from + (to - from) * t;
    };
    return Color::rgba(mix(r, other.r), mix(g, other.g), mix(b, other.b), mix(a, other.a));
}

Color Color::over(Color below) const {
    float alpha = a + below.a * (1.0f - a);
    if (alpha <= 0.0f) {
        return Color::Transparent;
    }
    auto mix = [&](float top, float bottom) {
        return (top * a + bottom * below.a * (1.0f - a)) / alpha;
    };
    return Color::rgba(mix(r, below.r), mix(g, below.g), mix(b, below.b), alpha);
}

Color Color::from_linear(float r, float g, float b) {
    return Color::rgb(encode_srgb(r), encode_srgb(g), encode_srgb(b));
}

std::array<float, 3> Color::to_linear() const {
    return {decode_srgb(r), decode_srgb(g), decode_srgb(b)};
}

float Color::luminance() const {
    auto [red, green, blue] = to_linear();
    return 0.2126f * red + 0.7152f * green + 0.0722f * blue;
}

std::array<float, 3> Color::to_hsv() const {
    float max_ = std::max(std::max(r, g), b);
    float min_ = std::min(std::min(r, g), b);
    float delta = max_ - min_;
    float hue;
    if (delta <= 0.0f) {
        hue = 0.0f;
    } else if (max_ == r) {
        hue = 60.0f * base::rem_euclid((g - b) / delta, 6.0f);
    } else if (max_ == g) {
        hue = 60.0f * ((b - r) / delta + 2.0f);
    } else {
        hue = 60.0f * ((r - g) / delta + 4.0f);
    }
    float saturation = max_ <= 0.0f ? 0.0f : delta / max_;
    return {hue, saturation, max_};
}

Color Color::from_hsv(float hue, float saturation, float value) {
    float sector = base::rem_euclid(hue, 360.0f) / 60.0f;
    float chroma = value * saturation;
    float x = chroma * (1.0f - std::abs(std::fmod(sector, 2.0f) - 1.0f));
    std::array<float, 3> channels;
    switch (base::saturating_cast<std::uint32_t>(sector)) {
    case 0:
        channels = {chroma, x, 0.0f};
        break;
    case 1:
        channels = {x, chroma, 0.0f};
        break;
    case 2:
        channels = {0.0f, chroma, x};
        break;
    case 3:
        channels = {0.0f, x, chroma};
        break;
    case 4:
        channels = {x, 0.0f, chroma};
        break;
    default:
        channels = {chroma, 0.0f, x};
        break;
    }
    float m = value - chroma;
    return Color::rgb(channels[0] + m, channels[1] + m, channels[2] + m);
}

std::string describe_value(Color color) {
    return std::format("Color {{ r: {}, g: {}, b: {}, a: {} }}", base::debug(color.r), base::debug(color.g), base::debug(color.b), base::debug(color.a));
}

float encode_srgb(float linear) {
    float clamped = std::clamp(linear, 0.0f, 1.0f);
    return clamped <= 0.0031308f ? clamped * 12.92f : 1.055f * std::pow(clamped, 1.0f / 2.4f) - 0.055f;
}

float decode_srgb(float encoded) {
    float clamped = std::clamp(encoded, 0.0f, 1.0f);
    return clamped <= 0.04045f ? clamped / 12.92f : std::pow((clamped + 0.055f) / 1.055f, 2.4f);
}

} // namespace user_interface
