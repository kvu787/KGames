// Text with DirectWrite: the type ramp's fonts, layout, measuring, hit
// testing for carets, and the glyph atlas that the renderer draws text from.
//
// So far, this has only the plain types that the design's tokens are made
// of: Font, Weight and TextStyle. The rest, which is the fonts themselves,
// DirectWrite's layouts and the glyph atlas, is still to come.

#pragma once

#include <array>
#include <cstdint>

namespace user_interface {

// Segoe UI Variable's optical sizes, as Windows names them as families,
// Cascadia Mono, and Segoe Fluent Icons. The type ramp says which of Small,
// Text and Display each style is set in.
enum class Font : std::uint8_t {
    Small,
    Text,
    Display,
    Mono,
    Icons,
};

inline constexpr std::array<Font, 5> AllFonts{Font::Small, Font::Text, Font::Display, Font::Mono, Font::Icons};

enum class Weight : std::uint8_t {
    Regular,
    Semibold,
    Bold,
};

// A font at a size, and the height of its lines, like the type ramp's
// Body: Segoe UI Variable Text, regular, 14 on 20.
struct TextStyle {
    Font font;
    Weight weight_;
    float size;
    float line_height;

    constexpr TextStyle(Font font, Weight weight, float size, float line_height) : font(font), weight_(weight), size(size), line_height(line_height) {}

    constexpr TextStyle weight(Weight weight) const {
        return TextStyle(font, weight, size, line_height);
    }

    // Segoe Fluent Icons at a size, in lines as tall as the icons.
    static constexpr TextStyle icons(float size) {
        return TextStyle(Font::Icons, Weight::Regular, size, size);
    }

    bool operator==(const TextStyle&) const = default;
};

} // namespace user_interface
