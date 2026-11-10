#include "UserInterface/Design/Theme.h"

#include "Base/Panic.h"
#include "UserInterface/Widgets/InfoBar.h"

namespace user_interface {

std::string_view name(ThemeKind kind) {
    switch (kind) {
    case ThemeKind::Light:
        return "Light";
    case ThemeKind::Dark:
        return "Dark";
    }
    base::unreachable();
}

ThemeKind other(ThemeKind kind) {
    switch (kind) {
    case ThemeKind::Light:
        return ThemeKind::Dark;
    case ThemeKind::Dark:
        return ThemeKind::Light;
    }
    base::unreachable();
}

std::string describe_value(ThemeKind kind) {
    return std::string(name(kind));
}

std::array<Color, 8> DataColors::all() const {
    return {blue, orange, green, purple, teal, red, yellow, pink};
}

Theme::Theme(ThemeKind kind) : Theme(kind == ThemeKind::Light ? Theme::light() : Theme::dark()) {}

Theme Theme::light() {
    AccentRamp ramp = DefaultAccent;
    Color accent = ramp.dark1;
    Theme theme;
    theme.kind = ThemeKind::Light;
    theme.accent_ramp = ramp;
    theme.text_primary = Color::argb(0xE4000000);
    theme.text_secondary = Color::argb(0x9E000000);
    theme.text_tertiary = Color::argb(0x72000000);
    theme.text_disabled = Color::argb(0x5C000000);
    theme.text_inverse = Color::White;
    theme.accent_text = ramp.dark2;
    theme.accent_text_secondary = ramp.dark3;
    theme.accent_text_tertiary = ramp.dark1;
    theme.accent_text_disabled = Color::argb(0x5C000000);
    theme.text_on_accent = Color::White;
    theme.text_on_accent_secondary = Color::argb(0xB3FFFFFF);
    theme.text_on_accent_disabled = Color::White;
    theme.text_on_accent_selected = Color::White;
    theme.control_fill = Color::argb(0xB3FFFFFF);
    theme.control_fill_secondary = Color::argb(0x80F9F9F9);
    theme.control_fill_tertiary = Color::argb(0x4DF9F9F9);
    theme.control_fill_quarternary = Color::argb(0xC2F3F3F3);
    theme.control_fill_disabled = Color::argb(0x4DF9F9F9);
    theme.control_fill_transparent = Color::argb(0x00FFFFFF);
    theme.control_fill_input_active = Color::White;
    theme.control_strong_fill = Color::argb(0x72000000);
    theme.control_strong_fill_disabled = Color::argb(0x51000000);
    theme.control_solid_fill = Color::White;
    theme.subtle_fill_transparent = Color::argb(0x00FFFFFF);
    theme.subtle_fill_secondary = Color::argb(0x09000000);
    theme.subtle_fill_tertiary = Color::argb(0x06000000);
    theme.subtle_fill_disabled = Color::argb(0x00FFFFFF);
    theme.control_alt_fill_transparent = Color::argb(0x00FFFFFF);
    theme.control_alt_fill_secondary = Color::argb(0x06000000);
    theme.control_alt_fill_tertiary = Color::argb(0x0F000000);
    theme.control_alt_fill_quarternary = Color::argb(0x18000000);
    theme.control_alt_fill_disabled = Color::argb(0x00FFFFFF);
    theme.control_on_image_fill = Color::argb(0xC9FFFFFF);
    theme.control_on_image_fill_secondary = Color::hex(0xF3F3F3);
    theme.control_on_image_fill_tertiary = Color::hex(0xEBEBEB);
    theme.control_on_image_fill_disabled = Color::argb(0x00FFFFFF);
    theme.accent = accent;
    theme.accent_secondary = accent.fade(0.9f);
    theme.accent_tertiary = accent.fade(0.8f);
    theme.accent_disabled = Color::argb(0x37000000);
    theme.accent_selected_text_background = ramp.base;
    theme.control_stroke = Color::argb(0x0F000000);
    theme.control_stroke_secondary = Color::argb(0x29000000);
    theme.control_stroke_on_accent = Color::argb(0x14FFFFFF);
    theme.control_stroke_on_accent_secondary = Color::argb(0x66000000);
    theme.control_stroke_on_accent_tertiary = Color::argb(0x37000000);
    theme.control_stroke_on_accent_disabled = Color::argb(0x0F000000);
    theme.control_stroke_for_strong_fill_on_image = Color::argb(0x59FFFFFF);
    theme.control_strong_stroke = Color::argb(0x72000000);
    theme.control_strong_stroke_disabled = Color::argb(0x37000000);
    theme.card_stroke = Color::argb(0x0F000000);
    theme.card_stroke_solid = Color::hex(0xEBEBEB);
    theme.surface_stroke = Color::argb(0x66757575);
    theme.surface_stroke_flyout = Color::argb(0x0F000000);
    theme.surface_stroke_inverse = Color::argb(0x15FFFFFF);
    theme.divider = Color::argb(0x0F000000);
    theme.focus_outer = Color::argb(0xE4000000);
    theme.focus_inner = Color::argb(0xB3FFFFFF);
    theme.background = Color::hex(0xF3F3F3);
    theme.background_secondary = Color::hex(0xEEEEEE);
    theme.background_tertiary = Color::hex(0xF9F9F9);
    theme.background_quarternary = Color::White;
    theme.background_quinary = Color::hex(0xFDFDFD);
    theme.background_senary = Color::White;
    theme.background_base_alt = Color::hex(0xDADADA);
    theme.card_background = Color::argb(0xB3FFFFFF);
    theme.card_background_secondary = Color::argb(0x80F6F6F6);
    theme.card_background_tertiary = Color::White;
    theme.layer = Color::argb(0x80FFFFFF);
    theme.layer_alt = Color::White;
    theme.layer_on_acrylic = Color::argb(0x40FFFFFF);
    theme.layer_on_mica_base_alt = Color::argb(0xB3FFFFFF);
    theme.layer_on_mica_base_alt_secondary = Color::argb(0x0A000000);
    theme.layer_on_mica_base_alt_tertiary = Color::hex(0xF9F9F9);
    theme.acrylic = Color::hex(0xF9F9F9);
    theme.acrylic_base = Color::hex(0xEEEEEE);
    theme.smoke = Color::argb(0x4D000000);
    theme.success = Color::hex(0x0F7B0F);
    theme.caution = Color::hex(0x9D5D00);
    theme.critical = Color::hex(0xC42B1C);
    theme.neutral = Color::argb(0x72000000);
    theme.solid_neutral = Color::hex(0x8A8A8A);
    theme.attention = ramp.base;
    theme.success_background = Color::hex(0xDFF6DD);
    theme.caution_background = Color::hex(0xFFF4CE);
    theme.critical_background = Color::hex(0xFDE7E9);
    theme.neutral_background = Color::argb(0x06000000);
    theme.attention_background = Color::argb(0x80F6F6F6);
    theme.solid_attention_background = Color::hex(0xF7F7F7);
    theme.solid_neutral_background = Color::hex(0xF3F3F3);
    theme.data = DataColors{
        .red = Color::hex(0xD13438),
        .orange = Color::hex(0xCA5010),
        .yellow = Color::hex(0xC19C00),
        .green = Color::hex(0x107C10),
        .teal = Color::hex(0x038387),
        .blue = Color::hex(0x0078D4),
        .purple = Color::hex(0x8764B8),
        .pink = Color::hex(0xE3008C),
    };
    return theme;
}

Theme Theme::dark() {
    AccentRamp ramp = DefaultAccent;
    Color accent = ramp.light2;
    Theme theme;
    theme.kind = ThemeKind::Dark;
    theme.accent_ramp = ramp;
    theme.text_primary = Color::White;
    theme.text_secondary = Color::argb(0xC5FFFFFF);
    theme.text_tertiary = Color::argb(0x87FFFFFF);
    theme.text_disabled = Color::argb(0x5DFFFFFF);
    theme.text_inverse = Color::argb(0xE4000000);
    theme.accent_text = ramp.light3;
    theme.accent_text_secondary = ramp.light3;
    theme.accent_text_tertiary = ramp.light2;
    theme.accent_text_disabled = Color::argb(0x5DFFFFFF);
    theme.text_on_accent = Color::Black;
    theme.text_on_accent_secondary = Color::argb(0x80000000);
    theme.text_on_accent_disabled = Color::argb(0x87FFFFFF);
    theme.text_on_accent_selected = Color::White;
    theme.control_fill = Color::argb(0x0FFFFFFF);
    theme.control_fill_secondary = Color::argb(0x15FFFFFF);
    theme.control_fill_tertiary = Color::argb(0x08FFFFFF);
    theme.control_fill_quarternary = Color::argb(0x0FFFFFFF);
    theme.control_fill_disabled = Color::argb(0x0BFFFFFF);
    theme.control_fill_transparent = Color::argb(0x00FFFFFF);
    theme.control_fill_input_active = Color::argb(0xB31E1E1E);
    theme.control_strong_fill = Color::argb(0x8BFFFFFF);
    theme.control_strong_fill_disabled = Color::argb(0x3FFFFFFF);
    theme.control_solid_fill = Color::hex(0x454545);
    theme.subtle_fill_transparent = Color::argb(0x00FFFFFF);
    theme.subtle_fill_secondary = Color::argb(0x0FFFFFFF);
    theme.subtle_fill_tertiary = Color::argb(0x0AFFFFFF);
    theme.subtle_fill_disabled = Color::argb(0x00FFFFFF);
    theme.control_alt_fill_transparent = Color::argb(0x00FFFFFF);
    theme.control_alt_fill_secondary = Color::argb(0x19000000);
    theme.control_alt_fill_tertiary = Color::argb(0x0BFFFFFF);
    theme.control_alt_fill_quarternary = Color::argb(0x12FFFFFF);
    theme.control_alt_fill_disabled = Color::argb(0x00FFFFFF);
    theme.control_on_image_fill = Color::argb(0xB31C1C1C);
    theme.control_on_image_fill_secondary = Color::hex(0x1A1A1A);
    theme.control_on_image_fill_tertiary = Color::hex(0x131313);
    theme.control_on_image_fill_disabled = Color::hex(0x1E1E1E);
    theme.accent = accent;
    theme.accent_secondary = accent.fade(0.9f);
    theme.accent_tertiary = accent.fade(0.8f);
    theme.accent_disabled = Color::argb(0x28FFFFFF);
    theme.accent_selected_text_background = ramp.base;
    theme.control_stroke = Color::argb(0x12FFFFFF);
    theme.control_stroke_secondary = Color::argb(0x18FFFFFF);
    theme.control_stroke_on_accent = Color::argb(0x14FFFFFF);
    theme.control_stroke_on_accent_secondary = Color::argb(0x23000000);
    theme.control_stroke_on_accent_tertiary = Color::argb(0x37000000);
    theme.control_stroke_on_accent_disabled = Color::argb(0x33000000);
    theme.control_stroke_for_strong_fill_on_image = Color::argb(0x6B000000);
    theme.control_strong_stroke = Color::argb(0x8BFFFFFF);
    theme.control_strong_stroke_disabled = Color::argb(0x28FFFFFF);
    theme.card_stroke = Color::argb(0x19000000);
    theme.card_stroke_solid = Color::hex(0x1C1C1C);
    theme.surface_stroke = Color::argb(0x66757575);
    theme.surface_stroke_flyout = Color::argb(0x33000000);
    theme.surface_stroke_inverse = Color::argb(0x0F000000);
    theme.divider = Color::argb(0x15FFFFFF);
    theme.focus_outer = Color::White;
    theme.focus_inner = Color::argb(0xB3000000);
    theme.background = Color::hex(0x202020);
    theme.background_secondary = Color::hex(0x1C1C1C);
    theme.background_tertiary = Color::hex(0x282828);
    theme.background_quarternary = Color::hex(0x2C2C2C);
    theme.background_quinary = Color::hex(0x333333);
    theme.background_senary = Color::hex(0x373737);
    theme.background_base_alt = Color::hex(0x0A0A0A);
    theme.card_background = Color::argb(0x0DFFFFFF);
    theme.card_background_secondary = Color::argb(0x08FFFFFF);
    theme.card_background_tertiary = Color::argb(0x12FFFFFF);
    theme.layer = Color::argb(0x4C3A3A3A);
    theme.layer_alt = Color::argb(0x0DFFFFFF);
    theme.layer_on_acrylic = Color::argb(0x09FFFFFF);
    theme.layer_on_mica_base_alt = Color::argb(0x733A3A3A);
    theme.layer_on_mica_base_alt_secondary = Color::argb(0x0FFFFFFF);
    theme.layer_on_mica_base_alt_tertiary = Color::hex(0x2C2C2C);
    theme.acrylic = Color::hex(0x2C2C2C);
    theme.acrylic_base = Color::hex(0x1C1C1C);
    theme.smoke = Color::argb(0x4D000000);
    theme.success = Color::hex(0x6CCB5F);
    theme.caution = Color::hex(0xFCE100);
    theme.critical = Color::hex(0xFF99A4);
    theme.neutral = Color::argb(0x8BFFFFFF);
    theme.solid_neutral = Color::hex(0x9D9D9D);
    theme.attention = ramp.light2;
    theme.success_background = Color::hex(0x393D1B);
    theme.caution_background = Color::hex(0x433519);
    theme.critical_background = Color::hex(0x442726);
    theme.neutral_background = Color::argb(0x08FFFFFF);
    theme.attention_background = Color::argb(0x08FFFFFF);
    theme.solid_attention_background = Color::hex(0x2E2E2E);
    theme.solid_neutral_background = Color::hex(0x2E2E2E);
    theme.data = DataColors{
        .red = Color::hex(0xF1707B),
        .orange = Color::hex(0xF7894A),
        .yellow = Color::hex(0xFCE100),
        .green = Color::hex(0x6CCB5F),
        .teal = Color::hex(0x4BC1C5),
        .blue = Color::hex(0x4CA7F0),
        .purple = Color::hex(0xA992D4),
        .pink = Color::hex(0xEE5FB7),
    };
    return theme;
}

Color Theme::on_background(Color color) const {
    return color.over(background);
}

Color Theme::card_opaque() const {
    return card_background.over(background);
}

Color Theme::status(Severity severity) const {
    switch (severity) {
    case Severity::Informational:
        return attention;
    case Severity::Success:
        return success;
    case Severity::Warning:
        return caution;
    case Severity::Error:
        return critical;
    }
    base::unreachable();
}

std::vector<std::pair<std::string_view, Color>> Theme::roles() const {
    return {
        {"text_primary", text_primary},
        {"text_secondary", text_secondary},
        {"text_tertiary", text_tertiary},
        {"text_disabled", text_disabled},
        {"accent_text", accent_text},
        {"text_on_accent", text_on_accent},
        {"accent", accent},
        {"accent_secondary", accent_secondary},
        {"accent_tertiary", accent_tertiary},
        {"accent_disabled", accent_disabled},
        {"control_fill", control_fill},
        {"control_fill_secondary", control_fill_secondary},
        {"control_fill_tertiary", control_fill_tertiary},
        {"control_fill_input_active", control_fill_input_active},
        {"control_strong_fill", control_strong_fill},
        {"control_solid_fill", control_solid_fill},
        {"control_alt_fill_secondary", control_alt_fill_secondary},
        {"control_alt_fill_tertiary", control_alt_fill_tertiary},
        {"subtle_fill_secondary", subtle_fill_secondary},
        {"subtle_fill_tertiary", subtle_fill_tertiary},
        {"control_stroke", control_stroke},
        {"control_stroke_secondary", control_stroke_secondary},
        {"control_strong_stroke", control_strong_stroke},
        {"card_stroke", card_stroke},
        {"surface_stroke_flyout", surface_stroke_flyout},
        {"divider", divider},
        {"focus_outer", focus_outer},
        {"background", background},
        {"background_secondary", background_secondary},
        {"background_tertiary", background_tertiary},
        {"card_background", card_background},
        {"card_background_secondary", card_background_secondary},
        {"layer", layer},
        {"acrylic", acrylic},
        {"smoke", smoke},
        {"success", success},
        {"caution", caution},
        {"critical", critical},
        {"attention", attention},
        {"success_background", success_background},
        {"caution_background", caution_background},
        {"critical_background", critical_background},
        {"attention_background", attention_background},
    };
}

} // namespace user_interface
