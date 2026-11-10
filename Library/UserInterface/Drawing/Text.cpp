#include "UserInterface/Drawing/Text.h"

#include "Base/Panic.h"
#include "Base/Text.h"

#include <algorithm>
#include <format>
#include <limits>
#include <span>

#pragma comment(lib, "dwrite.lib")

namespace user_interface {

namespace {

using Microsoft::WRL::ComPtr;
using text_detail::Atlas;
using text_detail::AtlasGlyph;
using text_detail::Faces;
using text_detail::PlacedGlyph;
using text_detail::StyleKey;

// Layouts not used for this many frames are let go of.
constexpr std::uint64_t LayoutLifetime = 600;

// The families to try, in order.
std::span<const std::string_view> families(Font font) {
    static constexpr std::array<std::string_view, 2> Small{"Segoe UI Variable Small", "Segoe UI"};
    static constexpr std::array<std::string_view, 2> Text{"Segoe UI Variable Text", "Segoe UI"};
    static constexpr std::array<std::string_view, 2> Display{"Segoe UI Variable Display", "Segoe UI"};
    static constexpr std::array<std::string_view, 2> Mono{"Cascadia Mono", "Consolas"};
    static constexpr std::array<std::string_view, 2> Icons{"Segoe Fluent Icons", "Segoe MDL2 Assets"};
    switch (font) {
    case Font::Small:
        return Small;
    case Font::Text:
        return Text;
    case Font::Display:
        return Display;
    case Font::Mono:
        return Mono;
    case Font::Icons:
        return Icons;
    }
    base::unreachable();
}

DWRITE_FONT_WEIGHT value(Weight weight) {
    switch (weight) {
    case Weight::Regular:
        return DWRITE_FONT_WEIGHT_NORMAL;
    case Weight::Semibold:
        return DWRITE_FONT_WEIGHT_SEMI_BOLD;
    case Weight::Bold:
        return DWRITE_FONT_WEIGHT_BOLD;
    }
    base::unreachable();
}

StyleKey key(const TextStyle& style) {
    return StyleKey{
        .font = style.font,
        .weight = style.weight_,
        .size = std::bit_cast<std::uint32_t>(style.size),
        .line_height = std::bit_cast<std::uint32_t>(style.line_height),
    };
}

// Mixes a key's bits into a hash: splitmix64's finalizer.
size_t mix(std::uint64_t bits) {
    bits = (bits ^ (bits >> 30)) * 0xBF58476D1CE4E5B9;
    bits = (bits ^ (bits >> 27)) * 0x94D049BB133111EB;
    return static_cast<size_t>(bits ^ (bits >> 31));
}

// How many UTF-16 units the character takes: char::len_utf16.
size_t len_utf16(char32_t character) {
    return character >= 0x10000 ? 2 : 1;
}

// FNV-1a over the text and the style, which is fast for short text.
std::uint64_t layout_key(std::string_view text, TextStyle style, std::optional<float> wrap) {
    std::uint64_t hash = 0xcbf29ce484222325;
    auto add = [&](std::uint8_t byte) {
        hash ^= byte;
        hash *= 0x0100000001b3;
    };
    for (char byte : text) {
        add(static_cast<std::uint8_t>(byte));
    }
    StyleKey style_key = key(style);
    add(static_cast<std::uint8_t>(style_key.font));
    add(static_cast<std::uint8_t>(style_key.weight));
    for (std::uint32_t word : {style_key.size, style_key.line_height, wrap ? std::bit_cast<std::uint32_t>(*wrap) : std::numeric_limits<std::uint32_t>::max()}) {
        // Its bytes, the low one first.
        for (std::uint32_t shift = 0; shift < 32; shift += 8) {
            add(static_cast<std::uint8_t>(word >> shift));
        }
    }
    return hash;
}

// Whether the font's glyphs at this size are fitted to the pixel grid, as the
// font's own tables say. CreateGlyphRunAnalysis takes the grid-fit mode, which
// must be one or the other, not DWRITE_GRID_FIT_MODE_DEFAULT, which means
// this.
DWRITE_GRID_FIT_MODE grid_fit(IDWriteFontFace* face, float size) {
    DWRITE_RENDERING_MODE rendering = DWRITE_RENDERING_MODE_DEFAULT;
    DWRITE_GRID_FIT_MODE mode = DWRITE_GRID_FIT_MODE_ENABLED;
    ComPtr<IDWriteFontFace2> face2;
    if (SUCCEEDED(face->QueryInterface(IID_PPV_ARGS(&face2)))) {
        (void)face2->GetRecommendedRenderingMode(size, static_cast<float>(USER_DEFAULT_SCREEN_DPI), static_cast<float>(USER_DEFAULT_SCREEN_DPI), nullptr, FALSE,
                                                 DWRITE_OUTLINE_THRESHOLD_ANTIALIASED, DWRITE_MEASURING_MODE_NATURAL, nullptr, &rendering, &mode);
    }
    return mode == DWRITE_GRID_FIT_MODE_DISABLED ? DWRITE_GRID_FIT_MODE_DISABLED : DWRITE_GRID_FIT_MODE_ENABLED;
}

// The text renderer that IDWriteTextLayout::Draw calls back, which only
// collects the glyphs. It lives on the stack for that one call, so its
// reference count never frees it.
class Collector final : public IDWriteTextRenderer {
public:
    std::vector<PlacedGlyph> glyphs;
    Faces& faces;

    explicit Collector(Faces& faces) : faces(faces) {}

    HRESULT STDMETHODCALLTYPE QueryInterface(REFIID iid, void** object) override {
        if (iid == __uuidof(IUnknown) || iid == __uuidof(IDWritePixelSnapping) || iid == __uuidof(IDWriteTextRenderer)) {
            *object = static_cast<IDWriteTextRenderer*>(this);
            return S_OK;
        }
        *object = nullptr;
        return E_NOINTERFACE;
    }

    ULONG STDMETHODCALLTYPE AddRef() override {
        return 1;
    }

    ULONG STDMETHODCALLTYPE Release() override {
        return 1;
    }

    HRESULT STDMETHODCALLTYPE IsPixelSnappingDisabled(void*, BOOL* disabled) override {
        // Baselines snap to whole pixels; glyphs keep their quarter pixels
        // across.
        *disabled = FALSE;
        return S_OK;
    }

    HRESULT STDMETHODCALLTYPE GetCurrentTransform(void*, DWRITE_MATRIX* transform) override {
        *transform = DWRITE_MATRIX{
            .m11 = 1.0f,
            .m12 = 0.0f,
            .m21 = 0.0f,
            .m22 = 1.0f,
            .dx = 0.0f,
            .dy = 0.0f,
        };
        return S_OK;
    }

    HRESULT STDMETHODCALLTYPE GetPixelsPerDip(void*, FLOAT* pixels) override {
        *pixels = 1.0f;
        return S_OK;
    }

    HRESULT STDMETHODCALLTYPE DrawGlyphRun(void*, FLOAT baseline_x, FLOAT baseline_y, DWRITE_MEASURING_MODE, const DWRITE_GLYPH_RUN* run, const DWRITE_GLYPH_RUN_DESCRIPTION*,
                                           IUnknown*) override {
        if (run->fontFace == nullptr) {
            return S_OK;
        }
        std::uint16_t face = faces.index(run->fontFace);
        bool right_to_left = (run->bidiLevel & 1) == 1;
        float pen = baseline_x;
        for (size_t index = 0; index < run->glyphCount; index++) {
            float advance = run->glyphAdvances == nullptr ? 0.0f : run->glyphAdvances[index];
            float along = 0.0f;
            float up = 0.0f;
            if (run->glyphOffsets != nullptr) {
                DWRITE_GLYPH_OFFSET offset = run->glyphOffsets[index];
                along = offset.advanceOffset;
                up = offset.ascenderOffset;
            }
            float x = right_to_left ? pen - advance - along : pen + along;
            glyphs.push_back(PlacedGlyph{
                .face = face,
                .glyph = run->glyphIndices[index],
                .size = run->fontEmSize,
                .x = x,
                .y = baseline_y - up,
            });
            pen += right_to_left ? -advance : advance;
        }
        return S_OK;
    }

    HRESULT STDMETHODCALLTYPE DrawUnderline(void*, FLOAT, FLOAT, const DWRITE_UNDERLINE*, IUnknown*) override {
        return S_OK;
    }

    HRESULT STDMETHODCALLTYPE DrawStrikethrough(void*, FLOAT, FLOAT, const DWRITE_STRIKETHROUGH*, IUnknown*) override {
        return S_OK;
    }

    HRESULT STDMETHODCALLTYPE DrawInlineObject(void*, FLOAT, FLOAT, IDWriteInlineObject*, BOOL, BOOL, IUnknown*) override {
        return S_OK;
    }
};

} // namespace

namespace text_detail {

size_t StyleKeyHash::operator()(const StyleKey& key) const noexcept {
    std::uint64_t sizes = key.size | (static_cast<std::uint64_t>(key.line_height) << 32);
    std::uint64_t kind = static_cast<std::uint64_t>(key.font) | (static_cast<std::uint64_t>(key.weight) << 8);
    return mix(sizes ^ (kind * 0x9E3779B97F4A7C15));
}

size_t GlyphKeyHash::operator()(const GlyphKey& key) const noexcept {
    std::uint64_t glyph = key.face | (static_cast<std::uint64_t>(key.glyph) << 16) | (static_cast<std::uint64_t>(key.size) << 32);
    return mix(glyph ^ ((static_cast<std::uint64_t>(key.offset) + 1) * 0x9E3779B97F4A7C15));
}

std::optional<std::pair<std::uint32_t, std::uint32_t>> Atlas::place(std::uint32_t width, std::uint32_t height) {
    // A pixel between glyphs keeps them from bleeding into each other.
    if (shelf_x + width + 1 > AtlasSize) {
        shelf_x = 1;
        shelf_y += shelf_height + 1;
        shelf_height = 0;
    }
    if (shelf_y + height + 1 > AtlasSize) {
        full = true;
        return std::nullopt;
    }
    std::pair<std::uint32_t, std::uint32_t> place(shelf_x, shelf_y);
    shelf_x += width + 1;
    shelf_height = std::max(shelf_height, height);
    return place;
}

std::uint16_t Faces::index(IDWriteFontFace* face) {
    size_t pointer = reinterpret_cast<size_t>(face);
    if (auto known = by_pointer.find(pointer); known != by_pointer.end()) {
        return known->second;
    }
    std::uint16_t index = static_cast<std::uint16_t>(faces.size());
    // The copy holds a reference, so the pointer stays this face's.
    faces.emplace_back(face);
    by_pointer.emplace(pointer, index);
    return index;
}

std::optional<AtlasGlyph> rasterize(IDWriteFactory2* factory, IDWriteFontFace* face, const PlacedGlyph& placed, std::uint8_t offset, Atlas& atlas) {
    FLOAT advance = 0.0f;
    DWRITE_GLYPH_RUN run{
        .fontFace = face,
        .fontEmSize = placed.size,
        .glyphCount = 1,
        .glyphIndices = &placed.glyph,
        .glyphAdvances = &advance,
        .glyphOffsets = nullptr,
        .isSideways = FALSE,
        .bidiLevel = 0,
    };
    ComPtr<IDWriteGlyphRunAnalysis> analysis;
    if (FAILED(factory->CreateGlyphRunAnalysis(&run, nullptr, DWRITE_RENDERING_MODE_NATURAL_SYMMETRIC, DWRITE_MEASURING_MODE_NATURAL, grid_fit(face, placed.size),
                                               DWRITE_TEXT_ANTIALIAS_MODE_GRAYSCALE, static_cast<float>(offset) / 4.0f, 0.0f, &analysis))) {
        return std::nullopt;
    }
    RECT bounds{};
    if (FAILED(analysis->GetAlphaTextureBounds(DWRITE_TEXTURE_ALIASED_1x1, &bounds))) {
        return std::nullopt;
    }
    std::uint32_t width = static_cast<std::uint32_t>(std::max<LONG>(bounds.right - bounds.left, 0));
    std::uint32_t height = static_cast<std::uint32_t>(std::max<LONG>(bounds.bottom - bounds.top, 0));
    if (width == 0 || height == 0) {
        // Like a space: an advance, but no pixels.
        return AtlasGlyph{
            .x = 0,
            .y = 0,
            .width = 0,
            .height = 0,
            .left = 0,
            .top = 0,
        };
    }
    std::vector<std::uint8_t> pixels(static_cast<size_t>(width * height));
    if (FAILED(analysis->CreateAlphaTexture(DWRITE_TEXTURE_ALIASED_1x1, &bounds, pixels.data(), static_cast<UINT32>(pixels.size())))) {
        return std::nullopt;
    }
    std::optional<std::pair<std::uint32_t, std::uint32_t>> place = atlas.place(width, height);
    if (!place) {
        return std::nullopt;
    }
    auto [x, y] = *place;
    atlas.uploads.emplace_back(x, y, width, height, std::move(pixels));
    return AtlasGlyph{
        .x = static_cast<std::uint16_t>(x),
        .y = static_cast<std::uint16_t>(y),
        .width = static_cast<std::uint16_t>(width),
        .height = static_cast<std::uint16_t>(height),
        .left = static_cast<std::int16_t>(bounds.left),
        .top = static_cast<std::int16_t>(bounds.top),
    };
}

} // namespace text_detail

TextLayout::TextLayout(float width, float height, std::uint32_t line_count, std::vector<text_detail::PlacedGlyph> glyphs, text_detail::ComPtr<IDWriteTextLayout> layout,
                       std::string text, std::uint64_t last_used) :
    width(width), height(height), line_count(line_count), glyphs(std::move(glyphs)), layout(std::move(layout)), text(std::move(text)), last_used(last_used) {}

Size TextLayout::size() const {
    return Size(width, height);
}

float TextLayout::caret_x(size_t byte_index) const {
    size_t position = utf16_index(text, byte_index);
    FLOAT x = 0.0f;
    FLOAT y = 0.0f;
    DWRITE_HIT_TEST_METRICS metrics{};
    (void)layout->HitTestTextPosition(static_cast<UINT32>(position), FALSE, &x, &y, &metrics);
    return x;
}

size_t TextLayout::hit(Point point) const {
    BOOL trailing = FALSE;
    BOOL inside = FALSE;
    DWRITE_HIT_TEST_METRICS metrics{};
    (void)layout->HitTestPoint(point.x, point.y, &trailing, &inside, &metrics);
    size_t position = metrics.textPosition;
    if (trailing) {
        position += metrics.length;
    }
    return byte_index(text, position);
}

size_t utf16_index(std::string_view text, size_t byte_index) {
    size_t units = 0;
    for (char32_t character : base::chars(text.substr(0, std::min(byte_index, text.size())))) {
        units += len_utf16(character);
    }
    return units;
}

size_t byte_index(std::string_view text, size_t utf16) {
    size_t units = 0;
    size_t index = 0;
    while (index < text.size()) {
        if (units >= utf16) {
            return index;
        }
        base::Decoded decoded = base::decode_char(text.substr(index));
        units += len_utf16(decoded.character);
        index += decoded.length;
    }
    return text.size();
}

Fonts::Fonts() {
    base::check(DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED, __uuidof(IDWriteFactory2), reinterpret_cast<IUnknown**>(factory.GetAddressOf())));
    base::check(factory->GetSystemFontCollection(&collection, FALSE));
    if (!collection) {
        throw base::WindowsError(S_OK);
    }
    for (Font font : AllFonts) {
        std::string family = find_family(font);
        resolved.insert_or_assign(font, std::move(family));
    }
}

std::string_view Fonts::family(Font font) const {
    return resolved.at(font);
}

std::string Fonts::find_family(Font font) const {
    for (std::string_view family : families(font)) {
        if (family_index(family)) {
            return std::string(family);
        }
    }
    // DirectWrite falls back to another family for a missing one.
    return std::string(families(font)[0]);
}

std::optional<std::uint32_t> Fonts::family_index(std::string_view family) const {
    UINT32 index = 0;
    BOOL exists = FALSE;
    std::wstring name = base::widen(family);
    base::check(collection->FindFamilyName(name.c_str(), &index, &exists));
    if (exists) {
        return index;
    }
    return std::nullopt;
}

void Fonts::begin_frame() {
    frame += 1;
    if (frame % LayoutLifetime == 0) {
        std::erase_if(layouts, [&](auto& entry) {
            std::erase_if(entry.second, [&](const TextLayout& cached) { return !(frame - cached.last_used < LayoutLifetime); });
            return entry.second.empty();
        });
    }
    if (atlas.full) {
        atlas = Atlas();
    }
}

text_detail::ComPtr<IDWriteTextFormat> Fonts::format(TextStyle style) {
    StyleKey style_key = key(style);
    if (auto known = formats.find(style_key); known != formats.end()) {
        return known->second;
    }
    std::wstring family = base::widen(resolved.at(style.font));
    ComPtr<IDWriteTextFormat> made;
    base::check(factory->CreateTextFormat(family.c_str(), nullptr, value(style.weight_), DWRITE_FONT_STYLE_NORMAL, DWRITE_FONT_STRETCH_NORMAL, style.size, L"en-us", &made));
    // Every line is the style's line height, with the font's ascent
    // and descent centered in it, as XAML sets the type ramp.
    auto [ascent, descent] = extent(style);
    float baseline = (style.line_height - (ascent + descent)) / 2.0f + ascent;
    base::check(made->SetLineSpacing(DWRITE_LINE_SPACING_METHOD_UNIFORM, style.line_height, baseline));
    formats.insert_or_assign(style_key, made);
    return made;
}

std::pair<float, float> Fonts::extent(TextStyle style) const {
    std::optional<std::uint32_t> index = family_index(resolved.at(style.font));
    if (!index) {
        return {style.size * 0.9f, style.size * 0.25f};
    }
    ComPtr<IDWriteFontFamily> family;
    base::check(collection->GetFontFamily(*index, &family));
    ComPtr<IDWriteFont> font;
    base::check(family->GetFirstMatchingFont(value(style.weight_), DWRITE_FONT_STRETCH_NORMAL, DWRITE_FONT_STYLE_NORMAL, &font));
    ComPtr<IDWriteFontFace> face;
    base::check(font->CreateFontFace(&face));
    DWRITE_FONT_METRICS metrics{};
    face->GetMetrics(&metrics);
    float scale = style.size / static_cast<float>(metrics.designUnitsPerEm);
    return {static_cast<float>(metrics.ascent) * scale, static_cast<float>(metrics.descent) * scale};
}

const TextLayout& Fonts::layout(std::string_view text, TextStyle style, std::optional<float> wrap) {
    auto [hash, index] = find(text, style, wrap);
    return layouts.find(hash)->second[index];
}

Size Fonts::measure(std::string_view text, TextStyle style) {
    return layout(text, style, std::nullopt).size();
}

Size Fonts::measure_wrapped(std::string_view text, TextStyle style, float width) {
    return layout(text, style, width).size();
}

std::pair<std::uint64_t, size_t> Fonts::find(std::string_view text, TextStyle style, std::optional<float> wrap) {
    std::uint64_t hash = layout_key(text, style, wrap);
    if (auto known = layouts.find(hash); known != layouts.end()) {
        std::vector<TextLayout>& cached = known->second;
        for (size_t index = 0; index < cached.size(); index++) {
            if (cached[index].text == text) {
                cached[index].last_used = frame;
                return {hash, index};
            }
        }
    }
    auto lay_out = [&]() -> TextLayout {
        try {
            return create_layout(text, style, wrap);
        } catch (const base::WindowsError& error) {
            base::panic(std::format("DirectWrite couldn't lay out {}: {}", base::debug(text), error.what()));
        }
    };
    TextLayout made = lay_out();
    std::vector<TextLayout>& cached = layouts[hash];
    cached.push_back(std::move(made));
    return {hash, cached.size() - 1};
}

TextLayout Fonts::create_layout(std::string_view text, TextStyle style, std::optional<float> wrap) {
    ComPtr<IDWriteTextFormat> text_format = format(style);
    std::wstring units = base::widen(text);
    ComPtr<IDWriteTextLayout> made;
    base::check(factory->CreateTextLayout(units.data(), static_cast<UINT32>(units.size()), text_format.Get(), wrap.value_or(1.0e6f), 1.0e6f, &made));
    if (!wrap) {
        base::check(made->SetWordWrapping(DWRITE_WORD_WRAPPING_NO_WRAP));
    }
    DWRITE_TEXT_METRICS metrics{};
    base::check(made->GetMetrics(&metrics));
    Collector collector(faces);
    base::check(made->Draw(nullptr, &collector, 0.0f, 0.0f));
    std::uint32_t line_count = std::max<std::uint32_t>(metrics.lineCount, 1);
    return TextLayout(metrics.widthIncludingTrailingWhitespace, static_cast<float>(line_count) * style.line_height, line_count, std::move(collector.glyphs), std::move(made),
                      std::string(text), frame);
}

std::vector<GlyphUpload> Fonts::take_uploads() {
    return std::exchange(atlas.uploads, {});
}

} // namespace user_interface
