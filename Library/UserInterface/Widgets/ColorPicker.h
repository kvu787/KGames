// Choosing a color: WinUI's ColorPicker's spectrum, a box of hue across and
// saturation down, with a slider of the value under it, whose thumbs are a
// ring and a slider's thumb; and a color field, a text field for a color's
// hex code with a swatch of it. Both send the color as the input leaves it.

#pragma once

#include "UserInterface/Core/Interaction.h"
#include "UserInterface/Core/Tree.h"
#include "UserInterface/Design/Controls.h"
#include "UserInterface/Design/Tokens.h"
#include "UserInterface/Drawing/Geometry.h"
#include "UserInterface/Drawing/Painter.h"
#include "UserInterface/Widgets/TextBox.h"
#include "UserInterface/Widgets/Widget.h"

#include <algorithm>
#include <array>
#include <cmath>
#include <cstddef>
#include <cstdint>
#include <optional>
#include <string>
#include <string_view>
#include <utility>

namespace user_interface {

// What only the color widgets' own code uses, here and in ColorPicker.cpp.
namespace color_picker_detail {

// The color as hue, saturation and value, kept so that a gray keeps the hue
// it had.
struct Hsv {
    float hue = 0.0f;
    float saturation = 0.0f;
    float value = 0.0f;
    std::optional<Color> last;

    // Takes the color, unless it's the one this came from.
    void sync(Color color);
};

// The hues around the color wheel, for the spectrum's gradient.
inline constexpr std::array<std::uint32_t, 7> Hues{0xFF0000, 0xFFFF00, 0x00FF00, 0x00FFFF, 0x0000FF, 0xFF00FF, 0xFF0000};

inline constexpr std::string_view Spectrum = "Spectrum";
inline constexpr std::string_view Value = "Value";

// What's typed, while the field has focus.
struct ColorDraft {
    std::string text;
    bool editing = false;
};

inline constexpr float SwatchSize = 16.0f;

bool hex_digit(char32_t character);

FieldSettings settings();

} // namespace color_picker_detail

template <class M> class ColorPicker : public Widget<M> {
public:
    explicit ColorPicker(Color color) : color(color) {}
    ColorPicker(const ColorPicker&) = delete;
    ColorPicker& operator=(const ColorPicker&) = delete;
    ColorPicker(ColorPicker&&) = default;
    ColorPicker& operator=(ColorPicker&&) = default;

    // What moving a thumb sends, with the color.
    template <class F> ColorPicker&& on_change(F&& reaction) && {
        on_change_ = react<Color, M>(std::forward<F>(reaction));
        return std::move(*this);
    }

    // How tall it is for a width: a square spectrum, and its value slider.
    static float height(float width) {
        return width + tokens::SpaceL + controls::SliderThumbSize + controls::SliderThumbOverhang * 2.0f;
    }

    std::string_view kind() const override {
        return "ColorPicker";
    }

    std::optional<std::string> label() const override {
        return color.to_hex();
    }

    std::optional<Sense> sense() const override {
        return Sense::None;
    }

    void parts(Parts& ctx) const override {
        auto [spectrum, slider_row] = areas(ctx.rect());
        ctx.part(Part::Named{std::string(color_picker_detail::Spectrum)}, color_picker_detail::Spectrum, spectrum, Sense::Click, "ColorSpectrum", color.to_hex());
        ctx.part(Part::Named{std::string(color_picker_detail::Value)}, color_picker_detail::Value, slider_row, Sense::Click, "ColorSlider", std::nullopt);
    }

    void interact(const Interaction& interaction, Dispatch<M>& ctx) const override {
        auto [spectrum, slider_row] = areas(ctx.rect());
        color_picker_detail::Hsv hsv = std::exchange(ctx.template memory<color_picker_detail::Hsv>(), color_picker_detail::Hsv{});
        hsv.sync(color);
        bool changed = false;
        if (const Hit* hit = interaction.hit(Part::Named{std::string(color_picker_detail::Spectrum)}); hit != nullptr && hit->held && interaction.pointer) {
            Point pointer = *interaction.pointer;
            Point fraction = spectrum.fraction_of(pointer);
            hsv.hue = std::clamp(fraction.x, 0.0f, 1.0f) * 360.0f;
            hsv.saturation = 1.0f - std::clamp(fraction.y, 0.0f, 1.0f);
            if (hsv.value <= 0.0f) {
                hsv.value = 1.0f;
            }
            changed = true;
        }
        if (const Hit* hit = interaction.hit(Part::Named{std::string(color_picker_detail::Value)}); hit != nullptr && hit->held && interaction.pointer) {
            Point pointer = *interaction.pointer;
            auto [left, right] = rail(slider_row);
            hsv.value = std::clamp((pointer.x - left) / std::max(right - left, 1.0f), 0.0f, 1.0f);
            changed = true;
        }
        if (changed) {
            Color picked = Color::from_hsv(hsv.hue, hsv.saturation, hsv.value).with_alpha(color.a);
            hsv.last = picked;
            if (picked != color && on_change_) {
                ctx.send(on_change_(picked));
            }
        }
        ctx.template memory<color_picker_detail::Hsv>() = hsv;
    }

    void draw(Draw& ctx) const override {
        auto [spectrum, slider_row] = areas(ctx.rect());
        auto [left, right] = rail(slider_row);
        color_picker_detail::Hsv& hsv = ctx.memory<color_picker_detail::Hsv>();
        hsv.sync(color);
        float hue = hsv.hue;
        float saturation = hsv.saturation;
        float value = hsv.value;
        Color full = Color::from_hsv(hue, saturation, 1.0f);
        Painter painter = ctx.painter();
        const Theme& theme = painter.theme();
        // The hues across, in six gradients, the ends rounded, and white over
        // them toward the bottom, where saturation is 0.
        float segment = spectrum.width / 6.0f;
        for (size_t index = 0; index < 6; index++) {
            float x = std::round(spectrum.x + segment * static_cast<float>(index));
            float part_right = std::round(spectrum.x + segment * static_cast<float>(index + 1));
            Rect part(x, spectrum.y, part_right - x, spectrum.height);
            Corners corners = index == 0 ? Corners::left(tokens::RadiusControl) : index == 5 ? Corners::right(tokens::RadiusControl) : Corners::Zero;
            painter.shape(part, corners, Brush::horizontal(part, Color::hex(color_picker_detail::Hues[index]), Color::hex(color_picker_detail::Hues[index + 1])), std::nullopt);
        }
        painter.shape(spectrum, tokens::RadiusControl, Brush::vertical(spectrum, Color::White.with_alpha(0.0f), Color::White), std::nullopt);
        painter.shape(spectrum, tokens::RadiusControl, Color::Transparent, Border(theme.control_stroke, 1.0f));
        Point ring_center(spectrum.x + spectrum.width * hue / 360.0f, spectrum.y + spectrum.height * (1.0f - saturation));
        Rect ring = Rect::from_center(ring_center, Size::square(controls::ColorSpectrumRing));
        painter.shape(ring, controls::ColorSpectrumRing / 2.0f, Color::Transparent, Border(Color::White, 2.0f));
        painter.shape(ring.expand(1.0f), controls::ColorSpectrumRing / 2.0f + 1.0f, Color::Transparent, Border(Color::Black.with_alpha(0.6f), 1.0f));

        // The value, from black to the full color.
        Rect track(slider_row.x, slider_row.center().y - controls::ColorSliderHeight / 2.0f, slider_row.width, controls::ColorSliderHeight);
        painter.shape(track, controls::ColorSliderRadius, Brush::horizontal(track, Color::Black, full), Border(theme.control_stroke, 1.0f));
        Rect thumb = Rect::from_center(Point(left + (right - left) * value, track.center().y), Size::square(controls::SliderThumbSize)).expand(controls::SliderThumbOverhang);
        painter.shape(thumb, controls::SliderThumbRadius, theme.control_solid_fill, border(Elevation::Control, theme, thumb));
        painter.circle(thumb.center(), controls::ColorSliderInnerThumb / 2.0f, Color::from_hsv(hue, saturation, value));
    }

private:
    // The spectrum, and the value slider's row under it.
    static std::pair<Rect, Rect> areas(Rect rect) {
        float slider_height = controls::SliderThumbSize + controls::SliderThumbOverhang * 2.0f;
        Rect spectrum(rect.x, rect.y, rect.width, std::max(rect.height - tokens::SpaceL - slider_height, 0.0f));
        Rect slider_row(rect.x, rect.bottom() - slider_height, rect.width, slider_height);
        return {spectrum, slider_row};
    }

    static std::pair<float, float> rail(Rect slider_row) {
        return {slider_row.x + controls::SliderThumbSize / 2.0f, slider_row.right() - controls::SliderThumbSize / 2.0f};
    }

    Color color;
    Reaction<Color, M> on_change_;
};

// A text field for a color's hex code, like "#1E90FF", with a swatch of the
// color in it. What's typed takes effect with Enter or when the field loses
// focus, if it's a color.
template <class M> class ColorField : public Widget<M> {
public:
    explicit ColorField(Color color) : color(color) {}
    ColorField(const ColorField&) = delete;
    ColorField& operator=(const ColorField&) = delete;
    ColorField(ColorField&&) = default;
    ColorField& operator=(ColorField&&) = default;

    // What a typed color taking effect sends.
    template <class F> ColorField&& on_change(F&& reaction) && {
        on_change_ = react<Color, M>(std::forward<F>(reaction));
        return std::move(*this);
    }

    std::string_view kind() const override {
        return "ColorField";
    }

    std::optional<std::string> label() const override {
        return color.to_hex();
    }

    std::optional<Sense> sense() const override {
        return Sense::Text;
    }

    void parts(Parts& ctx) const override {
        std::string text = shown_text(ctx.memory<color_picker_detail::ColorDraft>());
        field_parts(ctx, text, color_picker_detail::settings());
    }

    void interact(const Interaction& interaction, Dispatch<M>& ctx) const override {
        color_picker_detail::ColorDraft draft = std::exchange(ctx.template memory<color_picker_detail::ColorDraft>(), color_picker_detail::ColorDraft{});
        // What's typed stays until it's taken or left, even as the field
        // loses focus, when it's taken.
        if (!draft.editing) {
            draft.text = color.to_hex();
        }
        Edited edited = field_interact(draft.text, color_picker_detail::settings(), interaction, ctx);
        if (edited.text) {
            draft.text = std::move(*edited.text);
            draft.editing = true;
        }
        if (edited.submitted || (draft.editing && !ctx.is_focused())) {
            if (std::optional<Color> typed = Color::parse(draft.text); typed && *typed != color && on_change_) {
                ctx.send(on_change_(*typed));
            }
            draft.editing = false;
            draft.text = color.to_hex();
        }
        ctx.template memory<color_picker_detail::ColorDraft>() = std::move(draft);
    }

    void draw(Draw& ctx) const override {
        Rect rect = ctx.rect();
        std::string text = shown_text(ctx.memory<color_picker_detail::ColorDraft>());
        field_draw(ctx, text, color_picker_detail::settings());
        Rect swatch = Rect(rect.x + 8.0f, rect.center().y - color_picker_detail::SwatchSize / 2.0f, color_picker_detail::SwatchSize, color_picker_detail::SwatchSize).round();
        Painter painter = ctx.painter();
        const Theme& theme = painter.theme();
        painter.shape(swatch, tokens::RadiusControl - 2.0f, color, Border(theme.control_stroke_secondary, 1.0f));
    }

private:
    std::string shown_text(const color_picker_detail::ColorDraft& draft) const {
        return draft.editing ? draft.text : color.to_hex();
    }

    Color color;
    Reaction<Color, M> on_change_;
};

} // namespace user_interface
