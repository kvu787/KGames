// Text with DirectWrite: the type ramp's fonts, layout, measuring, hit
// testing for carets, and the glyph atlas that the renderer draws text from.
//
// DirectWrite lays text out, with kerning, ligatures, font fallback and line
// breaking, in an IDWriteTextLayout. A text renderer of our own, a COM object
// written by hand in Text.cpp, collects the glyphs that the layout places.
// Layouts are cached by their text and style, so a frame that shows the same
// text as the last asks DirectWrite for nothing. Each glyph is rasterized
// once, with grayscale antialiasing as WinUI's text is, at a quarter-pixel
// horizontal offset, into an 8-bit atlas that the GPU reads.

#pragma once

#include "Base/Numbers.h"
#include "Base/Windows.h"
#include "UserInterface/Drawing/Geometry.h"

#include <dwrite_2.h>
#include <wrl/client.h>

#include <array>
#include <bit>
#include <cmath>
#include <cstddef>
#include <cstdint>
#include <optional>
#include <string>
#include <string_view>
#include <unordered_map>
#include <utility>
#include <vector>

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

// What only the text's own code uses, here and in Text.cpp.
namespace text_detail {

using Microsoft::WRL::ComPtr;

// A style as a key: its sizes by their bits, which compare and hash exactly.
struct StyleKey {
    Font font;
    Weight weight;
    std::uint32_t size;
    std::uint32_t line_height;

    bool operator==(const StyleKey&) const = default;
};

struct StyleKeyHash {
    size_t operator()(const StyleKey& key) const noexcept;
};

// A glyph that a layout placed: its face, by index into the faces that
// layouts used, its index in the face, and its pen position on its line's
// baseline, from the layout's top left.
struct PlacedGlyph {
    std::uint16_t face;
    std::uint16_t glyph;
    float size;
    float x;
    float y;
};

} // namespace text_detail

// Text laid out in one style, on one line or wrapped at a width.
class TextLayout {
public:
    // The widest line, with its trailing spaces.
    float width;
    float height;
    std::uint32_t line_count;

    TextLayout(const TextLayout&) = delete;
    TextLayout& operator=(const TextLayout&) = delete;
    TextLayout(TextLayout&&) = default;
    TextLayout& operator=(TextLayout&&) = default;

    Size size() const;

    // Where a caret before the character at the byte index goes, from the
    // layout's left.
    float caret_x(size_t byte_index) const;

    // The byte index between characters nearest a point in the layout, for
    // putting a caret where a click lands.
    size_t hit(Point point) const;

private:
    friend class Fonts;

    TextLayout(float width, float height, std::uint32_t line_count, std::vector<text_detail::PlacedGlyph> glyphs, text_detail::ComPtr<IDWriteTextLayout> layout, std::string text,
               std::uint64_t last_used);

    std::vector<text_detail::PlacedGlyph> glyphs;
    text_detail::ComPtr<IDWriteTextLayout> layout;
    std::string text;
    std::uint64_t last_used;
};

// The UTF-16 index of a byte index of UTF-8 text.
size_t utf16_index(std::string_view text, size_t byte_index);

// The byte index of a UTF-16 index, at a character's start.
size_t byte_index(std::string_view text, size_t utf16);

// A glyph's pixels to draw: where on screen, and where in the atlas.
struct GlyphQuad {
    float x = 0.0f;
    float y = 0.0f;
    float width = 0.0f;
    float height = 0.0f;
    float atlas_x = 0.0f;
    float atlas_y = 0.0f;
};

// Pixels that the renderer copies into its atlas texture.
struct GlyphUpload {
    std::uint32_t x;
    std::uint32_t y;
    std::uint32_t width;
    std::uint32_t height;
    std::vector<std::uint8_t> pixels;

    GlyphUpload(std::uint32_t x, std::uint32_t y, std::uint32_t width, std::uint32_t height, std::vector<std::uint8_t> pixels) :
        x(x), y(y), width(width), height(height), pixels(std::move(pixels)) {}
    GlyphUpload(const GlyphUpload&) = delete;
    GlyphUpload& operator=(const GlyphUpload&) = delete;
    GlyphUpload(GlyphUpload&&) = default;
    GlyphUpload& operator=(GlyphUpload&&) = default;
};

inline constexpr std::uint32_t AtlasSize = 2048;

namespace text_detail {

struct AtlasGlyph {
    std::uint16_t x;
    std::uint16_t y;
    std::uint16_t width;
    std::uint16_t height;
    // Where its pixels start from its pen position.
    std::int16_t left;
    std::int16_t top;
};

struct GlyphKey {
    std::uint16_t face;
    std::uint16_t glyph;
    std::uint32_t size;
    // Quarter pixels right of the pixel grid, 0 to 3.
    std::uint8_t offset;

    bool operator==(const GlyphKey&) const = default;
};

struct GlyphKeyHash {
    size_t operator()(const GlyphKey& key) const noexcept;
};

// Shelf packing: glyphs fill rows left to right, and a row is as tall as its
// tallest glyph. Once it's full, it starts over with the next frame.
struct Atlas {
    std::unordered_map<GlyphKey, std::optional<AtlasGlyph>, GlyphKeyHash> glyphs;
    std::uint32_t shelf_x = 1;
    std::uint32_t shelf_y = 1;
    std::uint32_t shelf_height = 0;
    bool full = false;
    std::vector<GlyphUpload> uploads;

    // Where a glyph of the size goes, or nothing once the atlas is full.
    std::optional<std::pair<std::uint32_t, std::uint32_t>> place(std::uint32_t width, std::uint32_t height);
};

// Every face that layouts used, so that glyphs name theirs by index.
struct Faces {
    std::vector<ComPtr<IDWriteFontFace>> faces;
    std::unordered_map<size_t, std::uint16_t> by_pointer;

    std::uint16_t index(IDWriteFontFace* face);
};

// The glyph's place in the atlas, with its pixels rasterized into the
// atlas' uploads, or nothing if DirectWrite fails or the atlas is full.
std::optional<AtlasGlyph> rasterize(IDWriteFactory2* factory, IDWriteFontFace* face, const PlacedGlyph& placed, std::uint8_t offset, Atlas& atlas);

} // namespace text_detail

class Fonts {
public:
    // Throws a base::WindowsError if DirectWrite can't start.
    Fonts();
    Fonts(const Fonts&) = delete;
    Fonts& operator=(const Fonts&) = delete;
    Fonts(Fonts&&) = default;
    Fonts& operator=(Fonts&&) = default;

    // The family that the font resolved to.
    std::string_view family(Font font) const;

    // Starts a frame: layouts unused for a while are let go of, and a full
    // atlas starts over.
    void begin_frame();

    // The text laid out in the style, wrapped at `wrap` pixels or not at all.
    // The layout is the cache's, and stays good only until the next call of
    // anything here that lays text out or starts a frame.
    const TextLayout& layout(std::string_view text, TextStyle style, std::optional<float> wrap);

    Size measure(std::string_view text, TextStyle style);

    Size measure_wrapped(std::string_view text, TextStyle style, float width);

    // The quads of the text's glyphs, with the layout's top left at
    // `origin`, rasterizing glyphs that aren't in the atlas yet. `emit` is
    // called with each GlyphQuad. Returns the layout's size.
    template <class Emit> Size quads(std::string_view text, TextStyle style, std::optional<float> wrap, Point origin, Emit&& emit) {
        // The layout is found first, then the atlas and the faces are read
        // beside it, so that the atlas can grow while the layout is read.
        auto [key, index] = find(text, style, wrap);
        const TextLayout& found = layouts.find(key)->second[index];
        for (const text_detail::PlacedGlyph& placed : found.glyphs) {
            float x = origin.x + placed.x;
            float baseline = std::round(origin.y + placed.y);
            float pixel = std::floor(x);
            std::uint8_t offset = base::saturating_cast<std::uint8_t>(std::round((x - pixel) * 4.0f));
            if (offset == 4) {
                pixel += 1.0f;
                offset = 0;
            }
            text_detail::GlyphKey glyph_key{
                .face = placed.face,
                .glyph = placed.glyph,
                .size = std::bit_cast<std::uint32_t>(placed.size),
                .offset = offset,
            };
            std::optional<text_detail::AtlasGlyph> entry;
            if (auto known = atlas.glyphs.find(glyph_key); known != atlas.glyphs.end()) {
                entry = known->second;
            } else {
                entry = text_detail::rasterize(factory.Get(), faces.faces[placed.face].Get(), placed, offset, atlas);
                if (entry || !atlas.full) {
                    atlas.glyphs.emplace(glyph_key, entry);
                }
            }
            if (entry && entry->width > 0) {
                emit(GlyphQuad{
                    .x = pixel + static_cast<float>(entry->left),
                    .y = baseline + static_cast<float>(entry->top),
                    .width = static_cast<float>(entry->width),
                    .height = static_cast<float>(entry->height),
                    .atlas_x = static_cast<float>(entry->x),
                    .atlas_y = static_cast<float>(entry->y),
                });
            }
        }
        return found.size();
    }

    // The glyphs rasterized since the last call, for the renderer to copy
    // into its texture.
    std::vector<GlyphUpload> take_uploads();

private:
    std::string find_family(Font font) const;

    std::optional<std::uint32_t> family_index(std::string_view family) const;

    text_detail::ComPtr<IDWriteTextFormat> format(TextStyle style);

    // The font's ascent and descent at the style's size, in pixels.
    std::pair<float, float> extent(TextStyle style) const;

    // Where the layout is in the cache, laid out now if it isn't there.
    std::pair<std::uint64_t, size_t> find(std::string_view text, TextStyle style, std::optional<float> wrap);

    TextLayout create_layout(std::string_view text, TextStyle style, std::optional<float> wrap);

    text_detail::ComPtr<IDWriteFactory2> factory;
    text_detail::ComPtr<IDWriteFontCollection> collection;
    std::unordered_map<text_detail::StyleKey, text_detail::ComPtr<IDWriteTextFormat>, text_detail::StyleKeyHash> formats;
    std::unordered_map<std::uint64_t, std::vector<TextLayout>> layouts;
    text_detail::Faces faces;
    text_detail::Atlas atlas;
    std::uint64_t frame = 0;
    // The family that each font resolved to, like "Segoe UI Variable Text".
    std::unordered_map<Font, std::string> resolved;
};

} // namespace user_interface
