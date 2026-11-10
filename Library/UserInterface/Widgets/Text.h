// Text and the small things that go with it, like XAML's TextBlock in the
// type ramp's styles: labels, captions, paragraphs, label and value fields,
// lines of several styles; keycaps for shortcuts; WinUI's InfoBadge; the
// status icon of InfoBar; swatches; dividers; and a canvas for a program's
// own drawing. None of them is a target: the pointer goes through them.
//
// It ends by including Widgets/InfoBar.h, which includes this for the status
// icon, so that whoever has a StatusIcon has the severities to make it with.

#pragma once

#include "Base/Sum.h"
#include "UserInterface/Core/Interaction.h"
#include "UserInterface/Core/Tree.h"
#include "UserInterface/Design/Icons.h"
#include "UserInterface/Design/Theme.h"
#include "UserInterface/Design/Tokens.h"
#include "UserInterface/Drawing/Geometry.h"
#include "UserInterface/Drawing/Painter.h"
#include "UserInterface/Drawing/Text.h"

#include <cstdint>
#include <functional>
#include <optional>
#include <string>
#include <string_view>
#include <utility>
#include <variant>
#include <vector>

namespace user_interface {

// An info bar's severity, which Widgets/InfoBar.h defines.
enum class Severity : std::uint8_t;

// A keycap's height, and the gap between the keycaps of a shortcut.
inline constexpr float KeycapHeight = 20.0f;

// A text's color: one of the theme's roles for text, or a color of its own.
struct TextColor {
    struct Primary {
        bool operator==(const Primary&) const = default;
    };
    struct Secondary {
        bool operator==(const Secondary&) const = default;
    };
    struct Tertiary {
        bool operator==(const Tertiary&) const = default;
    };
    struct Disabled {
        bool operator==(const Disabled&) const = default;
    };
    struct Accent {
        bool operator==(const Accent&) const = default;
    };
    struct Custom {
        Color color;
        bool operator==(const Custom&) const = default;
    };
    std::variant<Primary, Secondary, Tertiary, Disabled, Accent, Custom> value;

    template <base::AlternativeOf<TextColor> T> TextColor(T&& alternative) : value(std::forward<T>(alternative)) {}
    // Not explicit, on purpose: a color converts to a text color of its own.
    TextColor(Color color) : value(Custom{color}) {}

    Color resolve(const Theme& theme, bool enabled) const;

    bool operator==(const TextColor&) const = default;
};

// A line of text, or a paragraph.
class Text : public Widget<> {
public:
    // Body text in the primary color, on one line.
    explicit Text(std::string text);
    Text(const Text&) = delete;
    Text& operator=(const Text&) = delete;
    Text(Text&&) = default;
    Text& operator=(Text&&) = default;

    static Text label(std::string text);

    // Body Strong in the primary color.
    static Text heading(std::string text);

    // Caption in the secondary color.
    static Text caption(std::string text);

    // Body text wrapped at the width, in the primary color.
    static Text paragraph(std::string text);

    Text&& style(TextStyle style) &&;

    Text&& color(TextColor color) &&;

    Text&& secondary() &&;

    Text&& tertiary() &&;

    Text&& align(Align align) &&;

    Text&& center() &&;

    Text&& end() &&;

    Text&& wrap() &&;

    // Makes the text a target the pointer can rest on, for a tooltip.
    Text&& hoverable() &&;

    std::string_view text() const;

    std::optional<Sense> sense() const override;

    std::string_view kind() const override;

    std::optional<std::string> label() const override;

    void draw(Draw& ctx) const override;

private:
    enum class Flow : std::uint8_t {
        // One line, centered in the rectangle's height, cut short with an
        // ellipsis if it's too wide.
        Fit,
        // Wrapped at the rectangle's width, from its top.
        Wrap,
    };

    std::string text_;
    TextStyle style_;
    TextColor color_;
    Align align_;
    Flow flow;
    // Whether it's a target the pointer can rest on, for a tooltip.
    bool hoverable_;
};

// A label in the secondary color, and its value after it, at `label_width`
// from the left.
class Field : public Widget<> {
public:
    Field(std::string label, float label_width, std::string value);
    Field(const Field&) = delete;
    Field& operator=(const Field&) = delete;
    Field(Field&&) = default;
    Field& operator=(Field&&) = default;

    std::string_view kind() const override;

    std::optional<std::string> label() const override;

    void draw(Draw& ctx) const override;

private:
    std::string label_;
    float label_width;
    std::string value;
};

// A run of text in its own style and color, for a line of several.
struct Run {
    std::string text;
    TextStyle style;
    TextColor color;

    Run(std::string text, TextStyle style, TextColor color) : text(std::move(text)), style(style), color(color) {}
};

// Runs of text in their own styles and colors, one after another on a
// line, from the rectangle's top left.
class Spans : public Widget<> {
public:
    explicit Spans(std::vector<Run> runs);
    Spans(const Spans&) = delete;
    Spans& operator=(const Spans&) = delete;
    Spans(Spans&&) = default;
    Spans& operator=(Spans&&) = default;

    // The width of the runs, for laying them out.
    float width(Fonts& fonts) const;

    std::string_view kind() const override;

    void draw(Draw& ctx) const override;

private:
    std::vector<Run> runs;
};

// An icon, centered in its rectangle.
class IconView : public Widget<> {
public:
    IconView(Icon icon, float size);
    IconView(const IconView&) = delete;
    IconView& operator=(const IconView&) = delete;
    IconView(IconView&&) = default;
    IconView& operator=(IconView&&) = default;

    IconView&& color(TextColor color) &&;

    std::string_view kind() const override;

    void draw(Draw& ctx) const override;

private:
    Icon icon;
    float size;
    TextColor color_;
};

// InfoBar's status icon: a circle in the status's color with its mark,
// centered in its rectangle.
class StatusIcon : public Widget<> {
public:
    StatusIcon(Severity severity, float size);
    StatusIcon(const StatusIcon&) = delete;
    StatusIcon& operator=(const StatusIcon&) = delete;
    StatusIcon(StatusIcon&&) = default;
    StatusIcon& operator=(StatusIcon&&) = default;

    std::string_view kind() const override;

    void draw(Draw& ctx) const override;

private:
    Severity severity;
    float size;
};

// The library's own: a status's icon, centered on the point.
void status_icon(Painter& painter, Point center, Severity severity, float size);

// A 1-pixel divider along the rectangle's middle, across it if it's wider
// than it's tall, or down it.
class Divider : public Widget<> {
public:
    Divider() = default;
    Divider(const Divider&) = delete;
    Divider& operator=(const Divider&) = delete;
    Divider(Divider&&) = default;
    Divider& operator=(Divider&&) = default;

    std::string_view kind() const override;

    void draw(Draw& ctx) const override;
};

// A color in a rounded square, only to show, like a paint's in a row.
class Swatch : public Widget<> {
public:
    explicit Swatch(Color color);
    Swatch(const Swatch&) = delete;
    Swatch& operator=(const Swatch&) = delete;
    Swatch(Swatch&&) = default;
    Swatch& operator=(Swatch&&) = default;

    std::string_view kind() const override;

    std::optional<std::string> label() const override;

    void draw(Draw& ctx) const override;

private:
    Color color;
};

// A shortcut's keys, like "Ctrl+S", as keycaps, from the rectangle's top
// left.
class Keycaps : public Widget<> {
public:
    explicit Keycaps(std::string keys);
    Keycaps(const Keycaps&) = delete;
    Keycaps& operator=(const Keycaps&) = delete;
    Keycaps(Keycaps&&) = default;
    Keycaps& operator=(Keycaps&&) = default;

    // How wide one key's keycap is, like "Ctrl".
    static float key_width(Fonts& fonts, std::string_view key);

    // How wide the keycaps are.
    static float width(Fonts& fonts, std::string_view keys);

    std::string_view kind() const override;

    std::optional<std::string> label() const override;

    void draw(Draw& ctx) const override;

private:
    std::string keys;
};

// The library's own: a key, like "Ctrl" or "S", on a keycap whose top left
// is at the point. Returns its width.
float draw_keycap(Painter& painter, Point origin, std::string_view key);

// The library's own.
float keycap_width(Fonts& fonts, std::string_view key);

// The library's own: a shortcut's keys as keycaps from the point. Returns
// the width.
float draw_keys(Painter& painter, Point origin, std::string_view keys);

// The library's own.
float keys_width(Fonts& fonts, std::string_view keys);

// WinUI's InfoBadge: a count or a short word on a pill, or a dot if the
// text is empty, centered in its rectangle.
class Badge : public Widget<> {
public:
    explicit Badge(std::string text);
    Badge(const Badge&) = delete;
    Badge& operator=(const Badge&) = delete;
    Badge(Badge&&) = default;
    Badge& operator=(Badge&&) = default;

    static Badge dot();

    // Its size, for laying it out.
    static Size size(Fonts& fonts, std::string_view text);

    std::string_view kind() const override;

    std::optional<std::string> label() const override;

    void draw(Draw& ctx) const override;

private:
    std::string text;
};

// A program's own drawing in its rectangle, like a graph: whatever `draw`
// paints, from the state it captured when the view was built.
class Canvas : public Widget<> {
public:
    // What a canvas paints, given the painter and its rectangle.
    using Painting = std::function<void(Painter&, Rect)>;

    explicit Canvas(Painting draw);
    Canvas(const Canvas&) = delete;
    Canvas& operator=(const Canvas&) = delete;
    Canvas(Canvas&&) = default;
    Canvas& operator=(Canvas&&) = default;

    std::string_view kind() const override;

    void draw(Draw& ctx) const override;

private:
    Painting draw_;
};

} // namespace user_interface

// The severities of a status icon, which the info bar's header has, and
// which needs the status icon above.
#include "UserInterface/Widgets/InfoBar.h"
