// Fluent 2's color roles for Windows, in the light theme and the dark one,
// with Windows' default blue accent. Color with these roles, by what a thing
// is, instead of colors of your own: text that's secondary is
// `text_secondary`, whatever theme is on.
//
// The values are WinUI 3's theme resources, like TextFillColorPrimary and
// ControlFillColorDefault, which their names here follow. Surfaces that are
// translucent in WinUI, like Mica and Acrylic, are solid here, in the color
// they have over the desktop's default.

use crate::geometry::Color;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ThemeKind {
    #[default]
    Light,
    Dark,
}

impl ThemeKind {
    pub fn name(self) -> &'static str {
        match self {
            ThemeKind::Light => "Light",
            ThemeKind::Dark => "Dark",
        }
    }

    pub fn other(self) -> ThemeKind {
        match self {
            ThemeKind::Light => ThemeKind::Dark,
            ThemeKind::Dark => ThemeKind::Light,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Theme {
    pub kind: ThemeKind,

    // Text, from most to least prominent.
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_tertiary: Color,
    pub text_disabled: Color,
    // Text in the accent color, like a link's.
    pub accent_text: Color,
    // Text and icons on an accent fill.
    pub text_on_accent: Color,
    pub text_on_accent_secondary: Color,
    pub text_on_accent_disabled: Color,

    // The accent, for what's on, chosen or most important: rest, pointer
    // over, pressed, disabled.
    pub accent: Color,
    pub accent_secondary: Color,
    pub accent_tertiary: Color,
    pub accent_disabled: Color,

    // Controls' fills: rest, pointer over, pressed, disabled.
    pub control_fill: Color,
    pub control_fill_secondary: Color,
    pub control_fill_tertiary: Color,
    pub control_fill_disabled: Color,
    // A text field with focus.
    pub control_fill_input_active: Color,
    // Strong fills, like a slider's rail or a switch's knob.
    pub control_strong_fill: Color,
    pub control_strong_fill_disabled: Color,
    // Opaque, like a slider's thumb.
    pub control_solid_fill: Color,
    // Fills of controls that are off, like an unchecked checkbox.
    pub control_alt_fill_secondary: Color,
    pub control_alt_fill_tertiary: Color,
    pub control_alt_fill_quarternary: Color,
    pub control_alt_fill_disabled: Color,
    // Fills of controls that are invisible at rest, like a list's rows or a
    // subtle button: pointer over, and pressed.
    pub subtle_fill_secondary: Color,
    pub subtle_fill_tertiary: Color,

    // Strokes.
    pub control_stroke: Color,
    // The bottom edge of a control, which makes it look raised.
    pub control_stroke_secondary: Color,
    pub control_stroke_on_accent: Color,
    pub control_stroke_on_accent_secondary: Color,
    pub control_strong_stroke: Color,
    pub control_strong_stroke_disabled: Color,
    pub card_stroke: Color,
    pub surface_stroke_flyout: Color,
    pub divider: Color,
    // The two rings that show keyboard focus.
    pub focus_outer: Color,
    pub focus_inner: Color,

    // Surfaces, from the window's background up.
    pub background: Color,
    pub background_secondary: Color,
    pub background_tertiary: Color,
    pub background_quarternary: Color,
    pub card_background: Color,
    pub card_background_secondary: Color,
    // Layers, like a pane or a bar on the background.
    pub layer: Color,
    pub layer_alt: Color,
    // Flyouts, tooltips, and cards that float over a 3D view.
    pub flyout: Color,
    // Over the window, behind a dialog.
    pub smoke: Color,

    // Statuses: their colors, and the backgrounds of their info bars.
    pub success: Color,
    pub caution: Color,
    pub critical: Color,
    pub attention: Color,
    pub neutral: Color,
    pub success_background: Color,
    pub caution_background: Color,
    pub critical_background: Color,
    pub attention_background: Color,

    // Selected text in a text field.
    pub text_selection: Color,

    // Colors that tell series apart in graphs and the like.
    pub data: DataColors,
}

#[derive(Clone, Debug)]
pub struct DataColors {
    pub red: Color,
    pub green: Color,
    pub blue: Color,
    pub orange: Color,
    pub purple: Color,
    pub teal: Color,
    pub yellow: Color,
    pub pink: Color,
}

impl Theme {
    pub fn new(kind: ThemeKind) -> Theme {
        match kind {
            ThemeKind::Light => Theme::light(),
            ThemeKind::Dark => Theme::dark(),
        }
    }

    pub fn light() -> Theme {
        let accent = Color::hex(0x005FB8);
        Theme {
            kind: ThemeKind::Light,
            text_primary: Color::argb(0xE4000000),
            text_secondary: Color::argb(0x9E000000),
            text_tertiary: Color::argb(0x72000000),
            text_disabled: Color::argb(0x5C000000),
            accent_text: Color::hex(0x003E92),
            text_on_accent: Color::WHITE,
            text_on_accent_secondary: Color::argb(0xB3FFFFFF),
            text_on_accent_disabled: Color::WHITE,
            accent,
            accent_secondary: accent.with_alpha(0.9),
            accent_tertiary: accent.with_alpha(0.8),
            accent_disabled: Color::argb(0x37000000),
            control_fill: Color::argb(0xB3FFFFFF),
            control_fill_secondary: Color::argb(0x80F9F9F9),
            control_fill_tertiary: Color::argb(0x4DF9F9F9),
            control_fill_disabled: Color::argb(0x4DF9F9F9),
            control_fill_input_active: Color::WHITE,
            control_strong_fill: Color::argb(0x72000000),
            control_strong_fill_disabled: Color::argb(0x51000000),
            control_solid_fill: Color::WHITE,
            control_alt_fill_secondary: Color::argb(0x06000000),
            control_alt_fill_tertiary: Color::argb(0x0F000000),
            control_alt_fill_quarternary: Color::argb(0x18000000),
            control_alt_fill_disabled: Color::TRANSPARENT,
            subtle_fill_secondary: Color::argb(0x09000000),
            subtle_fill_tertiary: Color::argb(0x06000000),
            control_stroke: Color::argb(0x0F000000),
            control_stroke_secondary: Color::argb(0x29000000),
            control_stroke_on_accent: Color::argb(0x14FFFFFF),
            control_stroke_on_accent_secondary: Color::argb(0x66000000),
            control_strong_stroke: Color::argb(0x72000000),
            control_strong_stroke_disabled: Color::argb(0x37000000),
            card_stroke: Color::argb(0x0F000000),
            surface_stroke_flyout: Color::argb(0x0F000000),
            divider: Color::argb(0x0F000000),
            focus_outer: Color::argb(0xE4000000),
            focus_inner: Color::argb(0xB3FFFFFF),
            background: Color::hex(0xF3F3F3),
            background_secondary: Color::hex(0xEEEEEE),
            background_tertiary: Color::hex(0xF9F9F9),
            background_quarternary: Color::WHITE,
            card_background: Color::argb(0xB3FFFFFF),
            card_background_secondary: Color::argb(0x80F6F6F6),
            layer: Color::argb(0x80FFFFFF),
            layer_alt: Color::WHITE,
            flyout: Color::hex(0xFCFCFC),
            smoke: Color::argb(0x4D000000),
            success: Color::hex(0x0F7B0F),
            caution: Color::hex(0x9D5D00),
            critical: Color::hex(0xC42B1C),
            attention: accent,
            neutral: Color::argb(0x72000000),
            success_background: Color::hex(0xDFF6DD),
            caution_background: Color::hex(0xFFF4CE),
            critical_background: Color::hex(0xFDE7E9),
            attention_background: Color::argb(0x80F6F6F6),
            text_selection: Color::hex(0x0078D4),
            data: DataColors {
                red: Color::hex(0xD13438),
                green: Color::hex(0x107C10),
                blue: Color::hex(0x0078D4),
                orange: Color::hex(0xCA5010),
                purple: Color::hex(0x8764B8),
                teal: Color::hex(0x038387),
                yellow: Color::hex(0xC19C00),
                pink: Color::hex(0xE3008C),
            },
        }
    }

    pub fn dark() -> Theme {
        let accent = Color::hex(0x60CDFF);
        Theme {
            kind: ThemeKind::Dark,
            text_primary: Color::WHITE,
            text_secondary: Color::argb(0xC5FFFFFF),
            text_tertiary: Color::argb(0x87FFFFFF),
            text_disabled: Color::argb(0x5DFFFFFF),
            accent_text: Color::hex(0x99EBFF),
            text_on_accent: Color::BLACK,
            text_on_accent_secondary: Color::argb(0x80000000),
            text_on_accent_disabled: Color::argb(0x87FFFFFF),
            accent,
            accent_secondary: accent.with_alpha(0.9),
            accent_tertiary: accent.with_alpha(0.8),
            accent_disabled: Color::argb(0x28FFFFFF),
            control_fill: Color::argb(0x0FFFFFFF),
            control_fill_secondary: Color::argb(0x15FFFFFF),
            control_fill_tertiary: Color::argb(0x08FFFFFF),
            control_fill_disabled: Color::argb(0x0BFFFFFF),
            control_fill_input_active: Color::argb(0xB31E1E1E),
            control_strong_fill: Color::argb(0x8BFFFFFF),
            control_strong_fill_disabled: Color::argb(0x3FFFFFFF),
            control_solid_fill: Color::hex(0x454545),
            control_alt_fill_secondary: Color::argb(0x19000000),
            control_alt_fill_tertiary: Color::argb(0x0BFFFFFF),
            control_alt_fill_quarternary: Color::argb(0x12FFFFFF),
            control_alt_fill_disabled: Color::TRANSPARENT,
            subtle_fill_secondary: Color::argb(0x0FFFFFFF),
            subtle_fill_tertiary: Color::argb(0x0AFFFFFF),
            control_stroke: Color::argb(0x12FFFFFF),
            control_stroke_secondary: Color::argb(0x18FFFFFF),
            control_stroke_on_accent: Color::argb(0x14FFFFFF),
            control_stroke_on_accent_secondary: Color::argb(0x23000000),
            control_strong_stroke: Color::argb(0x8BFFFFFF),
            control_strong_stroke_disabled: Color::argb(0x28FFFFFF),
            card_stroke: Color::argb(0x19000000),
            surface_stroke_flyout: Color::argb(0x33000000),
            divider: Color::argb(0x15FFFFFF),
            focus_outer: Color::WHITE,
            focus_inner: Color::argb(0xB3000000),
            background: Color::hex(0x202020),
            background_secondary: Color::hex(0x1C1C1C),
            background_tertiary: Color::hex(0x282828),
            background_quarternary: Color::hex(0x2C2C2C),
            card_background: Color::argb(0x0DFFFFFF),
            card_background_secondary: Color::argb(0x08FFFFFF),
            layer: Color::argb(0x4C3A3A3A),
            layer_alt: Color::argb(0x0DFFFFFF),
            flyout: Color::hex(0x2C2C2C),
            smoke: Color::argb(0x4D000000),
            success: Color::hex(0x6CCB5F),
            caution: Color::hex(0xFCE100),
            critical: Color::hex(0xFF99A4),
            attention: accent,
            neutral: Color::argb(0x8BFFFFFF),
            success_background: Color::hex(0x393D1B),
            caution_background: Color::hex(0x433519),
            critical_background: Color::hex(0x442726),
            attention_background: Color::argb(0x08FFFFFF),
            text_selection: Color::hex(0x0078D4),
            data: DataColors {
                red: Color::hex(0xF1707B),
                green: Color::hex(0x6CCB5F),
                blue: Color::hex(0x4CA7F0),
                orange: Color::hex(0xF7894A),
                purple: Color::hex(0xA992D4),
                teal: Color::hex(0x4BC1C5),
                yellow: Color::hex(0xFCE100),
                pink: Color::hex(0xEE5FB7),
            },
        }
    }

    // The window's background with the color drawn over it, as it looks.
    pub fn on_background(&self, color: Color) -> Color {
        color.over(self.background)
    }

    // Every role, by its name, like "text_secondary", for showing the theme.
    pub fn roles(&self) -> Vec<(&'static str, Color)> {
        vec![
            ("text_primary", self.text_primary),
            ("text_secondary", self.text_secondary),
            ("text_tertiary", self.text_tertiary),
            ("text_disabled", self.text_disabled),
            ("accent_text", self.accent_text),
            ("accent", self.accent),
            ("accent_secondary", self.accent_secondary),
            ("accent_tertiary", self.accent_tertiary),
            ("accent_disabled", self.accent_disabled),
            ("control_fill", self.control_fill),
            ("control_fill_secondary", self.control_fill_secondary),
            ("control_fill_tertiary", self.control_fill_tertiary),
            ("control_strong_fill", self.control_strong_fill),
            ("control_solid_fill", self.control_solid_fill),
            ("control_alt_fill_secondary", self.control_alt_fill_secondary),
            ("subtle_fill_secondary", self.subtle_fill_secondary),
            ("control_stroke", self.control_stroke),
            ("control_stroke_secondary", self.control_stroke_secondary),
            ("control_strong_stroke", self.control_strong_stroke),
            ("card_stroke", self.card_stroke),
            ("divider", self.divider),
            ("background", self.background),
            ("background_secondary", self.background_secondary),
            ("background_tertiary", self.background_tertiary),
            ("card_background", self.card_background),
            ("layer", self.layer),
            ("flyout", self.flyout),
            ("success", self.success),
            ("caution", self.caution),
            ("critical", self.critical),
            ("attention", self.attention),
            ("success_background", self.success_background),
            ("caution_background", self.caution_background),
            ("critical_background", self.critical_background),
        ]
    }
}
