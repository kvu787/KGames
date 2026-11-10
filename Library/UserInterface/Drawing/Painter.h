// What a frame draws: shapes and text, as instances of one quad that the GPU
// draws in one call. Each instance's pixel shader computes its shape's
// coverage from a signed distance, so every edge is antialiased and every
// corner round, at any size.
//
// Widgets draw with a Painter, and so do programs, for their own graphics
// like graphs and previews, in the theme's colors. Brushes are WinUI's:
// solid colors, and linear gradients, which is how WinUI draws the elevation
// borders that make controls look raised.

#pragma once

#include "Base/Sum.h"
#include "UserInterface/Design/Icons.h"
#include "UserInterface/Design/Theme.h"
#include "UserInterface/Design/Tokens.h"
#include "UserInterface/Drawing/Geometry.h"
#include "UserInterface/Drawing/Text.h"

#include <array>
#include <cstddef>
#include <cstdint>
#include <optional>
#include <span>
#include <string>
#include <string_view>
#include <type_traits>
#include <utility>
#include <variant>
#include <vector>

namespace user_interface {

// One quad for the GPU, laid out as Interface.hlsl's Instance.
struct Instance {
    // A shape's left, top, right and bottom, or a line's ends, or an arc's
    // center, radius and width.
    std::array<float, 4> bounds{};
    // Corner radii, top left first, clockwise; or what else the kind needs.
    std::array<float, 4> radii{};
    // The border's width on each side: left, top, right, bottom.
    std::array<float, 4> border{};
    // The fill's gradient, from (x, y) to (z, w), in the window's pixels.
    std::array<float, 4> fill_line{};
    // The border's gradient, from (x, y) to (z, w).
    std::array<float, 4> border_line{};
    // Premultiplied sRGB: the fill from and to, the border from and to.
    std::array<std::uint32_t, 4> colors{};
    // The kind, and the index of its clip in the top 16 bits.
    std::uint32_t kind = 0;
    std::array<std::uint32_t, 3> padding{};
};

// The shader reads the instances as they are in memory.
static_assert(std::is_trivially_copyable_v<Instance> && std::is_standard_layout_v<Instance>);
static_assert(sizeof(Instance) == 112 && alignof(Instance) == 4);
static_assert(offsetof(Instance, bounds) == 0);
static_assert(offsetof(Instance, radii) == 16);
static_assert(offsetof(Instance, border) == 32);
static_assert(offsetof(Instance, fill_line) == 48);
static_assert(offsetof(Instance, border_line) == 64);
static_assert(offsetof(Instance, colors) == 80);
static_assert(offsetof(Instance, kind) == 96);
static_assert(offsetof(Instance, padding) == 100);

// Layers draw in order: a dialog covers the page, a popup like a combo box's
// list covers both, and a tooltip covers everything.
enum class Layer : std::uint8_t {
    Base,
    Dialog,
    Popup,
    Tooltip,
};

inline constexpr std::array<Layer, 4> AllLayers{Layer::Base, Layer::Dialog, Layer::Popup, Layer::Tooltip};

// The radius of each corner. A radius alone is every corner's, wherever
// corners are wanted.
struct Corners {
    float top_left = 0.0f;
    float top_right = 0.0f;
    float bottom_right = 0.0f;
    float bottom_left = 0.0f;

    static const Corners Zero;

    constexpr Corners() = default;
    constexpr Corners(float top_left, float top_right, float bottom_right, float bottom_left) :
        top_left(top_left), top_right(top_right), bottom_right(bottom_right), bottom_left(bottom_left) {}
    // Not explicit, on purpose: a radius converts to corners.
    constexpr Corners(float radius) : Corners(radius, radius, radius, radius) {}

    static constexpr Corners all(float radius) {
        return Corners(radius, radius, radius, radius);
    }

    static constexpr Corners top(float radius) {
        return Corners(radius, radius, 0.0f, 0.0f);
    }

    static constexpr Corners bottom(float radius) {
        return Corners(0.0f, 0.0f, radius, radius);
    }

    static constexpr Corners left(float radius) {
        return Corners(radius, 0.0f, 0.0f, radius);
    }

    static constexpr Corners right(float radius) {
        return Corners(0.0f, radius, radius, 0.0f);
    }

    bool operator==(const Corners&) const = default;
};

inline constexpr Corners Corners::Zero = Corners::all(0.0f);

// What fills a shape or its border: a color, or WinUI's LinearGradientBrush
// with two stops, between two points in the window. Past the points, the
// end colors go on, as XAML's do. A color alone is a solid brush, wherever a
// brush is wanted.
struct Brush {
    struct Solid {
        Color color;
        bool operator==(const Solid&) const = default;
    };
    struct Linear {
        Point from;
        Point to;
        Color start;
        Color end;
        bool operator==(const Linear&) const = default;
    };
    std::variant<Solid, Linear> value;

    template <base::AlternativeOf<Brush> T> Brush(T&& alternative) : value(std::forward<T>(alternative)) {}
    // Not explicit, on purpose: a color converts to a brush.
    Brush(Color color) : value(Solid{color}) {}

    // From one color at the top of the rectangle to another at its bottom.
    static Brush vertical(Rect rect, Color top, Color bottom);

    // From one color at the left of the rectangle to another at its right.
    static Brush horizontal(Rect rect, Color left, Color right);

    bool operator==(const Brush&) const = default;
};

// A shape's border, inside its edge: a width on each side, like a XAML
// Border's BorderThickness, and its brush.
struct Border {
    Insets widths;
    Brush brush;

    // The same width on every side.
    Border(const Brush& brush, float width) : widths(Insets::all(width)), brush(brush) {}

    static Border sides(const Brush& brush, Insets widths);

    bool operator==(const Border&) const = default;
};

enum class Align : std::uint8_t {
    Start,
    Center,
    End,
};

// A run of text in its own style and color, for a line of several. It
// borrows the text.
struct Span {
    std::string_view text;
    TextStyle style;
    Color color;

    Span(std::string_view text, TextStyle style, Color color) : text(text), style(style), color(color) {}
};

// Text that fits a width: the text it was made from, where that fits, without
// a copy, or a text of its own, cut short with an ellipsis. It's what
// Painter::fit returns, and what Cow<str> is in Rust.
class Fitted {
public:
    // The text itself, which must outlive this.
    static Fitted borrowed(std::string_view text);

    // A text of its own.
    static Fitted owned(std::string text);

    // Whether the text is its own, which is to say that it was cut short.
    bool is_owned() const;

    // The text, for as long as this lasts, and the text it was made from.
    std::string_view view() const;

    // The text as a string, copied only if it was borrowed.
    std::string into_owned() &&;

private:
    Fitted() = default;

    std::string_view borrowed_;
    std::optional<std::string> owned_;
};

// Everything a frame draws, layer by layer.
class DrawList {
public:
    // The library's own, for the renderer: each clip's left, top, right and
    // bottom, and polygons' corners.
    std::vector<std::array<float, 4>> clips;
    std::vector<std::array<float, 2>> points;

    DrawList() = default;
    DrawList(const DrawList&) = delete;
    DrawList& operator=(const DrawList&) = delete;
    DrawList(DrawList&&) = default;
    DrawList& operator=(DrawList&&) = default;

    // Starts over for a frame of the size, whose bounds are the first clip.
    void clear(Size size);

    std::span<const Instance> layer_instances(Layer layer) const;

    size_t instance_count() const;

    Layer layer() const;

    // How many instances the current layer has, to mark a place in it.
    size_t layer_len() const;

    // Moves the current layer's instances from `middle` on to before those
    // from `first`, so that what was drawn last draws under what was drawn
    // since `first`, like a card that's drawn after what's in it.
    void move_to_back(size_t first, size_t middle);

    void set_layer(Layer layer);

    Rect clip() const;

    // Draws only inside the rectangle, and inside the clip it's in, until
    // pop_clip. With `absolute`, the clip it's in doesn't matter, for what
    // floats over everything, like a popup from a scrolled list.
    void push_clip(Rect rect, bool absolute);

    void pop_clip();

private:
    friend class Painter;

    std::uint32_t clip_index() const;

    // Adds the instance to the current layer, as the kind, in the current
    // clip.
    void push(std::uint32_t kind, const Instance& instance);

    std::array<std::vector<Instance>, 4> layers;
    std::vector<std::uint16_t> clip_stack;
    Layer layer_ = Layer::Base;
};

// What widgets and programs draw with: the frame's draw list, the fonts and
// the theme, which it only refers to, as Painter{list, fonts, theme}.
class Painter {
public:
    // The library's own.
    DrawList& list;
    Fonts& fonts_;
    const Theme& theme_;

    // The theme, which can be read while the painter draws.
    const Theme& theme() const;

    Layer layer() const;

    void set_layer(Layer layer);

    Rect clip() const;

    void push_clip(Rect rect);

    void pop_clip();

    void fill(Rect rect, Color color);

    void rounded(Rect rect, Corners corners, Color color);

    // A rounded rectangle with a fill and maybe a border inside its edge,
    // like a XAML Border.
    void shape(Rect rect, Corners corners, const Brush& fill, const std::optional<Border>& border);

    // An outline outside the rectangle, like a focus visual.
    void outline(Rect rect, float radius, float width, Color color);

    // The shadows of a surface at an elevation, only outside the surface, so
    // that they never darken a translucent surface from below.
    void shadow(Rect rect, float radius, const tokens::Elevation& elevation);

    void circle(Point center, float radius, Color color);

    // A circle's outline, `width` wide, inside the radius.
    void ring(Point center, float radius, float width, Color color);

    // Part of a ring centered on the radius, from `start` clockwise by
    // `sweep`, in radians from straight up, with round ends, like a progress
    // ring's.
    void arc(Point center, float radius, float width, float start, float sweep, Color color);

    // A line `width` wide, with round ends.
    void line(Point from, Point to, float width, Color color);

    // A line `width` wide, with square ends that stop at its points.
    void line_flat(Point from, Point to, float width, Color color);

    void polyline(std::span<const Point> points, float width, Color color);

    // A convex polygon, like a hexagon or a triangle, with its corners in
    // either order around it.
    void polygon(std::span<const Point> points, Color color);

    Fonts& fonts();

    Size measure(std::string_view text, TextStyle style);

    // The size of the text wrapped at a width.
    Size measure_wrapped(std::string_view text, TextStyle style, float width);

    // A line of text with its line box's top left at `origin`. Returns its
    // size.
    Size text(Point origin, std::string_view text, TextStyle style, Color color);

    // Text wrapped at the rectangle's width, from its top left. Returns its
    // size.
    Size text_wrapped(Rect rect, std::string_view text, TextStyle style, Color color);

    // A line of text centered vertically in the rectangle and aligned across
    // it. Returns where its line box went.
    Rect text_in(Rect rect, std::string_view text, TextStyle style, Color color, Align align);

    // Like text_in, but text too wide for the rectangle is cut short with an
    // ellipsis, as XAML's TextTrimming does. Returns where it went, and
    // whether it was cut.
    std::pair<Rect, bool> text_fit(Rect rect, std::string_view text, TextStyle style, Color color, Align align);

    // The text, cut short with an ellipsis if it's wider than `width`, or
    // the text itself if it fits, without a copy.
    Fitted fit(std::string_view text, TextStyle style, float width);

    // A line of runs in their own styles and colors, one after another,
    // sharing a baseline. Returns the width.
    float spans(Point origin, std::span<const Span> spans);

    // An icon of `size` pixels, centered on the point.
    void icon(Point center, Icon icon, float size, Color color);

private:
    void segment(Point from, Point to, float width, Color color, bool round);

    Size glyphs(Point origin, std::string_view text, TextStyle style, std::optional<float> wrap, Color color);
};

// The library's own: the text, cut short with an ellipsis if it's wider than
// `width`, or the text itself if it fits, without a copy: most labels fit,
// every frame.
Fitted fit_text(Fonts& fonts, std::string_view text, TextStyle style, float width);

} // namespace user_interface
