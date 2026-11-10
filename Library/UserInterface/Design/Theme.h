// The color roles of Fluent 2 on Windows, in WinUI 3's light theme and dark
// one. Color with these roles, by what a thing is, instead of colors of your
// own: secondary text is `text_secondary`, whatever theme is on.
//
// Every role is a WinUI 3 theme resource, from Common_themeresources_any.xaml
// in microsoft/microsoft-ui-xaml, named in snake case without "Color",
// "Brush" or "Default": TextFillColorPrimary is `text_primary`,
// ControlFillColorDefault is `control_fill`, and AccentFillColorDefaultBrush
// is `accent`. The accent is Windows 11's default blue, whose ramp WinUI
// reads from the system. Mica and acrylic are solid here, in WinUI's own
// stand-ins for them: the page's background and acrylic's fallback color.

#pragma once

#include "UserInterface/Drawing/Geometry.h"

#include <array>
#include <cstdint>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

namespace user_interface {

// An info bar's severity, which Widgets/InfoBar.h defines.
enum class Severity : std::uint8_t;

enum class ThemeKind : std::uint8_t {
    Light,
    Dark,
};

std::string_view name(ThemeKind kind);

ThemeKind other(ThemeKind kind);

// How a failed check in a test shows a kind of theme.
std::string describe_value(ThemeKind kind);

// Windows' accent color and its shades, SystemAccentColor and its Light1-3
// and Dark1-3: here, the default blue.
struct AccentRamp {
    Color light3;
    Color light2;
    Color light1;
    Color base;
    Color dark1;
    Color dark2;
    Color dark3;

    bool operator==(const AccentRamp&) const = default;
};

inline constexpr AccentRamp DefaultAccent{
    .light3 = Color::hex(0x99EBFF),
    .light2 = Color::hex(0x4CC2FF),
    .light1 = Color::hex(0x0091F8),
    .base = Color::hex(0x0078D4),
    .dark1 = Color::hex(0x0067C0),
    .dark2 = Color::hex(0x003E92),
    .dark3 = Color::hex(0x001A68),
};

// Fluent 2's shared colors, for telling series apart where the color roles
// don't apply.
struct DataColors {
    Color red;
    Color orange;
    Color yellow;
    Color green;
    Color teal;
    Color blue;
    Color purple;
    Color pink;

    std::array<Color, 8> all() const;
};

struct Theme {
    ThemeKind kind;
    AccentRamp accent_ramp;

    // Text, from most to least prominent, and on the inverse surface.
    Color text_primary;
    Color text_secondary;
    Color text_tertiary;
    Color text_disabled;
    Color text_inverse;
    // Text in the accent color, like a hyperlink's.
    Color accent_text;
    Color accent_text_secondary;
    Color accent_text_tertiary;
    Color accent_text_disabled;
    // Text and icons on an accent fill.
    Color text_on_accent;
    Color text_on_accent_secondary;
    Color text_on_accent_disabled;
    Color text_on_accent_selected;

    // Controls' fills: at rest, with the pointer over, pressed, and more.
    Color control_fill;
    Color control_fill_secondary;
    Color control_fill_tertiary;
    Color control_fill_quarternary;
    Color control_fill_disabled;
    Color control_fill_transparent;
    // A text field with focus.
    Color control_fill_input_active;
    // Strong fills, like a slider's rail or a switch's knob.
    Color control_strong_fill;
    Color control_strong_fill_disabled;
    // Opaque, like a slider's thumb.
    Color control_solid_fill;
    // Fills of controls that are invisible at rest, like a list's rows or a
    // subtle button.
    Color subtle_fill_transparent;
    Color subtle_fill_secondary;
    Color subtle_fill_tertiary;
    Color subtle_fill_disabled;
    // Fills of controls that are off, like an unchecked checkbox.
    Color control_alt_fill_transparent;
    Color control_alt_fill_secondary;
    Color control_alt_fill_tertiary;
    Color control_alt_fill_quarternary;
    Color control_alt_fill_disabled;
    // Controls over images, like a 3D view.
    Color control_on_image_fill;
    Color control_on_image_fill_secondary;
    Color control_on_image_fill_tertiary;
    Color control_on_image_fill_disabled;

    // The accent, for what's on, chosen, or the one command that matters
    // most: at rest, pointer over, pressed, and disabled.
    Color accent;
    Color accent_secondary;
    Color accent_tertiary;
    Color accent_disabled;
    // Behind selected text.
    Color accent_selected_text_background;

    // Strokes.
    Color control_stroke;
    // The edge of a control that makes it look raised.
    Color control_stroke_secondary;
    Color control_stroke_on_accent;
    Color control_stroke_on_accent_secondary;
    Color control_stroke_on_accent_tertiary;
    Color control_stroke_on_accent_disabled;
    Color control_stroke_for_strong_fill_on_image;
    Color control_strong_stroke;
    Color control_strong_stroke_disabled;
    Color card_stroke;
    Color card_stroke_solid;
    Color surface_stroke;
    Color surface_stroke_flyout;
    Color surface_stroke_inverse;
    Color divider;
    // The two strokes of the focus visual.
    Color focus_outer;
    Color focus_inner;

    // Surfaces, from the window's background up.
    Color background;
    Color background_secondary;
    Color background_tertiary;
    Color background_quarternary;
    Color background_quinary;
    Color background_senary;
    Color background_base_alt;
    Color card_background;
    Color card_background_secondary;
    Color card_background_tertiary;
    // Layers on the background, like a pane or a bar.
    Color layer;
    Color layer_alt;
    Color layer_on_acrylic;
    Color layer_on_mica_base_alt;
    Color layer_on_mica_base_alt_secondary;
    Color layer_on_mica_base_alt_tertiary;
    // Flyouts, menus, popups and tooltips: in-app acrylic's fallback.
    Color acrylic;
    Color acrylic_base;
    // Over the window, under a dialog.
    Color smoke;

    // Statuses: their colors, and the backgrounds of their info bars.
    Color success;
    Color caution;
    Color critical;
    Color neutral;
    Color solid_neutral;
    Color attention;
    Color success_background;
    Color caution_background;
    Color critical_background;
    Color neutral_background;
    Color attention_background;
    Color solid_attention_background;
    Color solid_neutral_background;

    // Colors that tell series apart, in graphs and the like.
    DataColors data;

    explicit Theme(ThemeKind kind);

    static Theme light();

    static Theme dark();

    // How a color looks over the window's background.
    Color on_background(Color color) const;

    // How a card looks, opaque, over the window's background: what a 3D view
    // in a card should clear to, to match it.
    Color card_opaque() const;

    // The color of a status: success, caution, critical or attention.
    Color status(Severity severity) const;

    // Every role by its name, like "text_secondary", for showing the theme.
    std::vector<std::pair<std::string_view, Color>> roles() const;

private:
    // For light and dark, which give every role its color.
    Theme() = default;
};

} // namespace user_interface
