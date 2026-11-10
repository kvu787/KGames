// Points, sizes, rectangles, insets and colors, in the window's pixels, with
// +y down. Cards are laid out in fixed pixels by cutting rectangles apart:
//
//     Rect area = card.content;
//     Rect first = area.cut_top(32.0f, tokens::SpaceS);
//     auto [left, right] = area.cut_top(32.0f, 0.0f).columns<2>(tokens::SpaceS);

#pragma once

#include <array>
#include <cstddef>
#include <cstdint>
#include <optional>
#include <span>
#include <string>
#include <string_view>
#include <vector>

namespace user_interface {

struct Point {
    float x = 0.0f;
    float y = 0.0f;

    static const Point Zero;

    constexpr Point() = default;
    constexpr Point(float x, float y) : x(x), y(y) {}

    float length() const;
    float distance(Point other) const;
    Point lerp(Point other, float t) const;
    Point round() const;

    bool operator==(const Point&) const = default;
};

inline constexpr Point Point::Zero{0.0f, 0.0f};

constexpr Point operator+(Point a, Point b) {
    return Point(a.x + b.x, a.y + b.y);
}

constexpr Point& operator+=(Point& a, Point b) {
    a = a + b;
    return a;
}

constexpr Point operator-(Point a, Point b) {
    return Point(a.x - b.x, a.y - b.y);
}

constexpr Point& operator-=(Point& a, Point b) {
    a = a - b;
    return a;
}

constexpr Point operator*(Point point, float factor) {
    return Point(point.x * factor, point.y * factor);
}

constexpr Point operator-(Point point) {
    return Point(-point.x, -point.y);
}

// How a failed check in a test shows a point.
std::string describe_value(Point point);

struct Size {
    float width = 0.0f;
    float height = 0.0f;

    constexpr Size() = default;
    constexpr Size(float width, float height) : width(width), height(height) {}

    static constexpr Size square(float side) {
        return Size(side, side);
    }

    bool operator==(const Size&) const = default;
};

std::string describe_value(Size size);

// Space on each side, like WinUI's Thickness: a padding, a margin, or a
// border's width on each side.
struct Insets {
    float left = 0.0f;
    float top = 0.0f;
    float right = 0.0f;
    float bottom = 0.0f;

    static const Insets Zero;

    constexpr Insets() = default;
    constexpr Insets(float left, float top, float right, float bottom) : left(left), top(top), right(right), bottom(bottom) {}

    static constexpr Insets all(float amount) {
        return Insets(amount, amount, amount, amount);
    }

    static constexpr Insets symmetric(float horizontal, float vertical) {
        return Insets(horizontal, vertical, horizontal, vertical);
    }

    constexpr float horizontal() const {
        return left + right;
    }

    constexpr float vertical() const {
        return top + bottom;
    }

    constexpr bool is_zero() const {
        return *this == Insets::Zero;
    }

    bool operator==(const Insets&) const = default;
};

inline constexpr Insets Insets::Zero = Insets::all(0.0f);

std::string describe_value(Insets insets);

struct Rect {
    float x = 0.0f;
    float y = 0.0f;
    float width = 0.0f;
    float height = 0.0f;

    static const Rect Zero;

    constexpr Rect() = default;
    constexpr Rect(float x, float y, float width, float height) : x(x), y(y), width(width), height(height) {}

    static constexpr Rect from_origin(Point origin, Size size) {
        return Rect(origin.x, origin.y, size.width, size.height);
    }

    // The rectangle between two corners, in either order.
    static Rect from_corners(Point a, Point b);

    static constexpr Rect from_center(Point center, Size size) {
        return Rect(center.x - size.width / 2.0f, center.y - size.height / 2.0f, size.width, size.height);
    }

    constexpr float left() const {
        return x;
    }

    constexpr float top() const {
        return y;
    }

    constexpr float right() const {
        return x + width;
    }

    constexpr float bottom() const {
        return y + height;
    }

    constexpr Point origin() const {
        return Point(x, y);
    }

    constexpr Size size() const {
        return Size(width, height);
    }

    constexpr Point center() const {
        return Point(x + width / 2.0f, y + height / 2.0f);
    }

    constexpr Point top_left() const {
        return origin();
    }

    constexpr Point bottom_right() const {
        return Point(right(), bottom());
    }

    // Whether the point is in it, counting its left and top edges but not
    // its right and bottom ones, so that rectangles side by side never both
    // have a point.
    constexpr bool contains(Point point) const {
        return point.x >= x && point.y >= y && point.x < right() && point.y < bottom();
    }

    constexpr bool is_empty() const {
        return width <= 0.0f || height <= 0.0f;
    }

    bool intersects(Rect other) const;

    // The part that's in both, which is empty if they don't overlap.
    Rect intersect(Rect other) const;

    // The smallest rectangle around both.
    Rect union_(Rect other) const;

    Rect inset(float amount) const;

    // Smaller by the insets, like a control's content inside its padding.
    Rect pad(Insets insets) const;

    constexpr Rect expand(float amount) const {
        return Rect(x - amount, y - amount, width + amount * 2.0f, height + amount * 2.0f);
    }

    constexpr Rect offset(float dx, float dy) const {
        return Rect(x + dx, y + dy, width, height);
    }

    constexpr Rect translate(Point by) const {
        return offset(by.x, by.y);
    }

    // A rectangle of the size, centered in this one.
    constexpr Rect centered(Size size) const {
        return Rect::from_center(center(), size);
    }

    // Strips along an edge, of the full width or height.
    Rect top_strip(float strip_height) const;
    Rect bottom_strip(float strip_height) const;
    Rect left_strip(float strip_width) const;
    Rect right_strip(float strip_width) const;

    // Cuts a strip off an edge and returns it, leaving the rest, less the
    // gap after the strip. This is how cards lay out their controls.
    Rect cut_top(float strip_height, float gap);
    Rect cut_bottom(float strip_height, float gap);
    Rect cut_left(float strip_width, float gap);
    Rect cut_right(float strip_width, float gap);

    // N columns of equal width, with gaps between them, in whole pixels.
    template <size_t N> std::array<Rect, N> columns(float gap) const {
        std::vector<float> widths = split_widths(N, gap);
        float column_x = x;
        std::array<Rect, N> made;
        for (size_t index = 0; index < N; index++) {
            made[index] = Rect(column_x, y, widths[index], height);
            column_x += widths[index] + gap;
        }
        return made;
    }

    // N rows of equal height, with gaps between them, in whole pixels.
    template <size_t N> std::array<Rect, N> rows(float gap) const {
        Rect flipped(y, x, height, width);
        std::array<Rect, N> made = flipped.columns<N>(gap);
        for (Rect& row : made) {
            row = Rect(row.y, row.x, row.height, row.width);
        }
        return made;
    }

    // Columns of the widths that the weights give, out of the width less the
    // gaps, in whole pixels.
    std::vector<Rect> columns_weighted(std::span<const float> weights, float gap) const;

    // On whole pixels, which keeps 1-pixel strokes sharp.
    Rect round() const;

    constexpr Rect lerp(Rect other, float t) const {
        return Rect(x + (other.x - x) * t, y + (other.y - y) * t, width + (other.width - width) * t, height + (other.height - height) * t);
    }

    // Where the point is in it, as fractions of its width and height.
    constexpr Point fraction_of(Point point) const {
        return Point(width > 0.0f ? (point.x - x) / width : 0.0f, height > 0.0f ? (point.y - y) / height : 0.0f);
    }

    // The point at fractions of its width and height.
    constexpr Point at_fraction(Point fraction) const {
        return Point(x + width * fraction.x, y + height * fraction.y);
    }

    bool operator==(const Rect&) const = default;

private:
    std::vector<float> split_widths(size_t count, float gap) const;
};

inline constexpr Rect Rect::Zero{0.0f, 0.0f, 0.0f, 0.0f};

std::string describe_value(Rect rect);

// A color in sRGB, from 0 to 1, with straight alpha. The theme's colors and
// every control's are sRGB, and the interface blends in sRGB, as Windows does.
struct Color {
    float r = 0.0f;
    float g = 0.0f;
    float b = 0.0f;
    float a = 0.0f;

    static const Color Transparent;
    static const Color Black;
    static const Color White;

    static constexpr Color rgba(float r, float g, float b, float a) {
        return Color{r, g, b, a};
    }

    static constexpr Color rgb(float r, float g, float b) {
        return Color::rgba(r, g, b, 1.0f);
    }

    // From 0xRRGGBB, opaque.
    static constexpr Color hex(std::uint32_t rgb) {
        return Color::argb(0xFF000000 | rgb);
    }

    // From 0xAARRGGBB, the way XAML writes colors, like #E4000000.
    static constexpr Color argb(std::uint32_t argb) {
        return Color::rgba(static_cast<float>((argb >> 16) & 0xFF) / 255.0f, static_cast<float>((argb >> 8) & 0xFF) / 255.0f, static_cast<float>(argb & 0xFF) / 255.0f,
                           static_cast<float>((argb >> 24) & 0xFF) / 255.0f);
    }

    // From bytes, red, green, blue and alpha.
    static Color from_bytes(std::array<std::uint8_t, 4> bytes);

    // From "#RRGGBB" or "#AARRGGBB", with or without the #.
    static std::optional<Color> parse(std::string_view text);

    // Like "#1E90FF", or "#801E90FF" if it isn't opaque.
    std::string to_hex() const;

    std::array<std::uint8_t, 4> to_bytes() const;

    // Premultiplied, as the shader takes it: red in the low byte.
    std::uint32_t pack_premultiplied() const;

    constexpr Color with_alpha(float alpha) const {
        return Color::rgba(r, g, b, alpha);
    }

    // More transparent, by the opacity, like WinUI's Opacity on an element.
    constexpr Color fade(float opacity) const {
        return with_alpha(a * opacity);
    }

    // Between this color and the other, with straight alpha.
    Color lerp(Color other, float t) const;

    // This color drawn over another, as it looks.
    Color over(Color below) const;

    // From linear light, like a shader's output.
    static Color from_linear(float r, float g, float b);

    std::array<float, 3> to_linear() const;

    // Relative luminance, as WCAG defines it.
    float luminance() const;

    // Hue in degrees from 0 to 360, saturation and value from 0 to 1.
    std::array<float, 3> to_hsv() const;

    static Color from_hsv(float hue, float saturation, float value);

    bool operator==(const Color&) const = default;
};

inline constexpr Color Color::Transparent = Color::rgba(0.0f, 0.0f, 0.0f, 0.0f);
inline constexpr Color Color::Black = Color::rgba(0.0f, 0.0f, 0.0f, 1.0f);
inline constexpr Color Color::White = Color::rgba(1.0f, 1.0f, 1.0f, 1.0f);

std::string describe_value(Color color);

float encode_srgb(float linear);

float decode_srgb(float encoded);

} // namespace user_interface
