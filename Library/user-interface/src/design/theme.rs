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

use crate::drawing::geometry::Color;

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

// Windows' accent color and its shades, SystemAccentColor and its Light1-3
// and Dark1-3: here, the default blue.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AccentRamp {
    pub light3: Color,
    pub light2: Color,
    pub light1: Color,
    pub base: Color,
    pub dark1: Color,
    pub dark2: Color,
    pub dark3: Color,
}

pub const DEFAULT_ACCENT: AccentRamp = AccentRamp {
    light3: Color::hex(0x99EBFF),
    light2: Color::hex(0x4CC2FF),
    light1: Color::hex(0x0091F8),
    base: Color::hex(0x0078D4),
    dark1: Color::hex(0x0067C0),
    dark2: Color::hex(0x003E92),
    dark3: Color::hex(0x001A68),
};

#[derive(Clone, Debug)]
pub struct Theme {
    pub kind: ThemeKind,
    pub accent_ramp: AccentRamp,

    // Text, from most to least prominent, and on the inverse surface.
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_tertiary: Color,
    pub text_disabled: Color,
    pub text_inverse: Color,
    // Text in the accent color, like a hyperlink's.
    pub accent_text: Color,
    pub accent_text_secondary: Color,
    pub accent_text_tertiary: Color,
    pub accent_text_disabled: Color,
    // Text and icons on an accent fill.
    pub text_on_accent: Color,
    pub text_on_accent_secondary: Color,
    pub text_on_accent_disabled: Color,

    // Controls' fills: at rest, with the pointer over, pressed, and more.
    pub control_fill: Color,
    pub control_fill_secondary: Color,
    pub control_fill_tertiary: Color,
    pub control_fill_quarternary: Color,
    pub control_fill_disabled: Color,
    pub control_fill_transparent: Color,
    // A text field with focus.
    pub control_fill_input_active: Color,
    // Strong fills, like a slider's rail or a switch's knob.
    pub control_strong_fill: Color,
    pub control_strong_fill_disabled: Color,
    // Opaque, like a slider's thumb.
    pub control_solid_fill: Color,
    // Fills of controls that are invisible at rest, like a list's rows or a
    // subtle button.
    pub subtle_fill_transparent: Color,
    pub subtle_fill_secondary: Color,
    pub subtle_fill_tertiary: Color,
    pub subtle_fill_disabled: Color,
    // Fills of controls that are off, like an unchecked checkbox.
    pub control_alt_fill_transparent: Color,
    pub control_alt_fill_secondary: Color,
    pub control_alt_fill_tertiary: Color,
    pub control_alt_fill_quarternary: Color,
    pub control_alt_fill_disabled: Color,
    // Controls over images, like a 3D view.
    pub control_on_image_fill: Color,
    pub control_on_image_fill_secondary: Color,
    pub control_on_image_fill_tertiary: Color,
    pub control_on_image_fill_disabled: Color,

    // The accent, for what's on, chosen, or the one command that matters
    // most: at rest, pointer over, pressed, and disabled.
    pub accent: Color,
    pub accent_secondary: Color,
    pub accent_tertiary: Color,
    pub accent_disabled: Color,

    // Strokes.
    pub control_stroke: Color,
    // The edge of a control that makes it look raised.
    pub control_stroke_secondary: Color,
    pub control_stroke_on_accent: Color,
    pub control_stroke_on_accent_secondary: Color,
    pub control_stroke_on_accent_tertiary: Color,
    pub control_stroke_on_accent_disabled: Color,
    pub control_stroke_for_strong_fill_on_image: Color,
    pub control_strong_stroke: Color,
    pub control_strong_stroke_disabled: Color,
    pub card_stroke: Color,
    pub card_stroke_solid: Color,
    pub surface_stroke: Color,
    pub surface_stroke_flyout: Color,
    pub surface_stroke_inverse: Color,
    pub divider: Color,

    // Surfaces, from the window's background up.
    pub background: Color,
    pub background_secondary: Color,
    pub background_tertiary: Color,
    pub background_quarternary: Color,
    pub background_quinary: Color,
    pub background_senary: Color,
    pub background_base_alt: Color,
    pub card_background: Color,
    pub card_background_secondary: Color,
    pub card_background_tertiary: Color,
    // Layers on the background, like a pane or a bar.
    pub layer: Color,
    pub layer_alt: Color,
    pub layer_on_acrylic: Color,
    pub layer_on_mica_base_alt: Color,
    pub layer_on_mica_base_alt_secondary: Color,
    pub layer_on_mica_base_alt_tertiary: Color,
    // Flyouts, menus, popups and tooltips: in-app acrylic's fallback.
    pub acrylic: Color,
    pub acrylic_base: Color,
    // Over the window, under a dialog.
    pub smoke: Color,

    // Statuses: their colors, and the backgrounds of their info bars.
    pub success: Color,
    pub caution: Color,
    pub critical: Color,
    pub neutral: Color,
    pub solid_neutral: Color,
    pub attention: Color,
    pub success_background: Color,
    pub caution_background: Color,
    pub critical_background: Color,
    pub neutral_background: Color,
    pub attention_background: Color,
    pub solid_attention_background: Color,
    pub solid_neutral_background: Color,

    // Colors that tell series apart, in graphs and the like.
    pub data: DataColors,
}

// Fluent 2's shared colors, for telling series apart where the color roles
// don't apply.
#[derive(Clone, Debug)]
pub struct DataColors {
    pub red: Color,
    pub orange: Color,
    pub yellow: Color,
    pub green: Color,
    pub teal: Color,
    pub blue: Color,
    pub purple: Color,
    pub pink: Color,
}

impl DataColors {
    pub fn all(&self) -> [Color; 8] {
        [self.blue, self.orange, self.green, self.purple, self.teal, self.red, self.yellow, self.pink]
    }
}

impl Theme {
    pub fn new(kind: ThemeKind) -> Theme {
        match kind {
            ThemeKind::Light => Theme::light(),
            ThemeKind::Dark => Theme::dark(),
        }
    }

    pub fn light() -> Theme {
        let ramp = DEFAULT_ACCENT;
        let accent = ramp.dark1;
        Theme {
            kind: ThemeKind::Light,
            accent_ramp: ramp,
            text_primary: Color::argb(0xE4000000),
            text_secondary: Color::argb(0x9E000000),
            text_tertiary: Color::argb(0x72000000),
            text_disabled: Color::argb(0x5C000000),
            text_inverse: Color::WHITE,
            accent_text: ramp.dark2,
            accent_text_secondary: ramp.dark3,
            accent_text_tertiary: ramp.dark1,
            accent_text_disabled: Color::argb(0x5C000000),
            text_on_accent: Color::WHITE,
            text_on_accent_secondary: Color::argb(0xB3FFFFFF),
            text_on_accent_disabled: Color::WHITE,
            control_fill: Color::argb(0xB3FFFFFF),
            control_fill_secondary: Color::argb(0x80F9F9F9),
            control_fill_tertiary: Color::argb(0x4DF9F9F9),
            control_fill_quarternary: Color::argb(0xC2F3F3F3),
            control_fill_disabled: Color::argb(0x4DF9F9F9),
            control_fill_transparent: Color::argb(0x00FFFFFF),
            control_fill_input_active: Color::WHITE,
            control_strong_fill: Color::argb(0x72000000),
            control_strong_fill_disabled: Color::argb(0x51000000),
            control_solid_fill: Color::WHITE,
            subtle_fill_transparent: Color::argb(0x00FFFFFF),
            subtle_fill_secondary: Color::argb(0x09000000),
            subtle_fill_tertiary: Color::argb(0x06000000),
            subtle_fill_disabled: Color::argb(0x00FFFFFF),
            control_alt_fill_transparent: Color::argb(0x00FFFFFF),
            control_alt_fill_secondary: Color::argb(0x06000000),
            control_alt_fill_tertiary: Color::argb(0x0F000000),
            control_alt_fill_quarternary: Color::argb(0x18000000),
            control_alt_fill_disabled: Color::argb(0x00FFFFFF),
            control_on_image_fill: Color::argb(0xC9FFFFFF),
            control_on_image_fill_secondary: Color::hex(0xF3F3F3),
            control_on_image_fill_tertiary: Color::hex(0xEBEBEB),
            control_on_image_fill_disabled: Color::argb(0x00FFFFFF),
            accent,
            accent_secondary: accent.fade(0.9),
            accent_tertiary: accent.fade(0.8),
            accent_disabled: Color::argb(0x37000000),
            control_stroke: Color::argb(0x0F000000),
            control_stroke_secondary: Color::argb(0x29000000),
            control_stroke_on_accent: Color::argb(0x14FFFFFF),
            control_stroke_on_accent_secondary: Color::argb(0x66000000),
            control_stroke_on_accent_tertiary: Color::argb(0x37000000),
            control_stroke_on_accent_disabled: Color::argb(0x0F000000),
            control_stroke_for_strong_fill_on_image: Color::argb(0x59FFFFFF),
            control_strong_stroke: Color::argb(0x72000000),
            control_strong_stroke_disabled: Color::argb(0x37000000),
            card_stroke: Color::argb(0x0F000000),
            card_stroke_solid: Color::hex(0xEBEBEB),
            surface_stroke: Color::argb(0x66757575),
            surface_stroke_flyout: Color::argb(0x0F000000),
            surface_stroke_inverse: Color::argb(0x15FFFFFF),
            divider: Color::argb(0x0F000000),
            background: Color::hex(0xF3F3F3),
            background_secondary: Color::hex(0xEEEEEE),
            background_tertiary: Color::hex(0xF9F9F9),
            background_quarternary: Color::WHITE,
            background_quinary: Color::hex(0xFDFDFD),
            background_senary: Color::WHITE,
            background_base_alt: Color::hex(0xDADADA),
            card_background: Color::argb(0xB3FFFFFF),
            card_background_secondary: Color::argb(0x80F6F6F6),
            card_background_tertiary: Color::WHITE,
            layer: Color::argb(0x80FFFFFF),
            layer_alt: Color::WHITE,
            layer_on_acrylic: Color::argb(0x40FFFFFF),
            layer_on_mica_base_alt: Color::argb(0xB3FFFFFF),
            layer_on_mica_base_alt_secondary: Color::argb(0x0A000000),
            layer_on_mica_base_alt_tertiary: Color::hex(0xF9F9F9),
            acrylic: Color::hex(0xF9F9F9),
            acrylic_base: Color::hex(0xEEEEEE),
            smoke: Color::argb(0x4D000000),
            success: Color::hex(0x0F7B0F),
            caution: Color::hex(0x9D5D00),
            critical: Color::hex(0xC42B1C),
            neutral: Color::argb(0x72000000),
            solid_neutral: Color::hex(0x8A8A8A),
            attention: ramp.base,
            success_background: Color::hex(0xDFF6DD),
            caution_background: Color::hex(0xFFF4CE),
            critical_background: Color::hex(0xFDE7E9),
            neutral_background: Color::argb(0x06000000),
            attention_background: Color::argb(0x80F6F6F6),
            solid_attention_background: Color::hex(0xF7F7F7),
            solid_neutral_background: Color::hex(0xF3F3F3),
            data: DataColors {
                red: Color::hex(0xD13438),
                orange: Color::hex(0xCA5010),
                yellow: Color::hex(0xC19C00),
                green: Color::hex(0x107C10),
                teal: Color::hex(0x038387),
                blue: Color::hex(0x0078D4),
                purple: Color::hex(0x8764B8),
                pink: Color::hex(0xE3008C),
            },
        }
    }

    pub fn dark() -> Theme {
        let ramp = DEFAULT_ACCENT;
        let accent = ramp.light2;
        Theme {
            kind: ThemeKind::Dark,
            accent_ramp: ramp,
            text_primary: Color::WHITE,
            text_secondary: Color::argb(0xC5FFFFFF),
            text_tertiary: Color::argb(0x87FFFFFF),
            text_disabled: Color::argb(0x5DFFFFFF),
            text_inverse: Color::argb(0xE4000000),
            accent_text: ramp.light3,
            accent_text_secondary: ramp.light3,
            accent_text_tertiary: ramp.light2,
            accent_text_disabled: Color::argb(0x5DFFFFFF),
            text_on_accent: Color::BLACK,
            text_on_accent_secondary: Color::argb(0x80000000),
            text_on_accent_disabled: Color::argb(0x87FFFFFF),
            control_fill: Color::argb(0x0FFFFFFF),
            control_fill_secondary: Color::argb(0x15FFFFFF),
            control_fill_tertiary: Color::argb(0x08FFFFFF),
            control_fill_quarternary: Color::argb(0x0FFFFFFF),
            control_fill_disabled: Color::argb(0x0BFFFFFF),
            control_fill_transparent: Color::argb(0x00FFFFFF),
            control_fill_input_active: Color::argb(0xB31E1E1E),
            control_strong_fill: Color::argb(0x8BFFFFFF),
            control_strong_fill_disabled: Color::argb(0x3FFFFFFF),
            control_solid_fill: Color::hex(0x454545),
            subtle_fill_transparent: Color::argb(0x00FFFFFF),
            subtle_fill_secondary: Color::argb(0x0FFFFFFF),
            subtle_fill_tertiary: Color::argb(0x0AFFFFFF),
            subtle_fill_disabled: Color::argb(0x00FFFFFF),
            control_alt_fill_transparent: Color::argb(0x00FFFFFF),
            control_alt_fill_secondary: Color::argb(0x19000000),
            control_alt_fill_tertiary: Color::argb(0x0BFFFFFF),
            control_alt_fill_quarternary: Color::argb(0x12FFFFFF),
            control_alt_fill_disabled: Color::argb(0x00FFFFFF),
            control_on_image_fill: Color::argb(0xB31C1C1C),
            control_on_image_fill_secondary: Color::hex(0x1A1A1A),
            control_on_image_fill_tertiary: Color::hex(0x131313),
            control_on_image_fill_disabled: Color::hex(0x1E1E1E),
            accent,
            accent_secondary: accent.fade(0.9),
            accent_tertiary: accent.fade(0.8),
            accent_disabled: Color::argb(0x28FFFFFF),
            control_stroke: Color::argb(0x12FFFFFF),
            control_stroke_secondary: Color::argb(0x18FFFFFF),
            control_stroke_on_accent: Color::argb(0x14FFFFFF),
            control_stroke_on_accent_secondary: Color::argb(0x23000000),
            control_stroke_on_accent_tertiary: Color::argb(0x37000000),
            control_stroke_on_accent_disabled: Color::argb(0x33000000),
            control_stroke_for_strong_fill_on_image: Color::argb(0x6B000000),
            control_strong_stroke: Color::argb(0x8BFFFFFF),
            control_strong_stroke_disabled: Color::argb(0x28FFFFFF),
            card_stroke: Color::argb(0x19000000),
            card_stroke_solid: Color::hex(0x1C1C1C),
            surface_stroke: Color::argb(0x66757575),
            surface_stroke_flyout: Color::argb(0x33000000),
            surface_stroke_inverse: Color::argb(0x0F000000),
            divider: Color::argb(0x15FFFFFF),
            background: Color::hex(0x202020),
            background_secondary: Color::hex(0x1C1C1C),
            background_tertiary: Color::hex(0x282828),
            background_quarternary: Color::hex(0x2C2C2C),
            background_quinary: Color::hex(0x333333),
            background_senary: Color::hex(0x373737),
            background_base_alt: Color::hex(0x0A0A0A),
            card_background: Color::argb(0x0DFFFFFF),
            card_background_secondary: Color::argb(0x08FFFFFF),
            card_background_tertiary: Color::argb(0x12FFFFFF),
            layer: Color::argb(0x4C3A3A3A),
            layer_alt: Color::argb(0x0DFFFFFF),
            layer_on_acrylic: Color::argb(0x09FFFFFF),
            layer_on_mica_base_alt: Color::argb(0x733A3A3A),
            layer_on_mica_base_alt_secondary: Color::argb(0x0FFFFFFF),
            layer_on_mica_base_alt_tertiary: Color::hex(0x2C2C2C),
            acrylic: Color::hex(0x2C2C2C),
            acrylic_base: Color::hex(0x1C1C1C),
            smoke: Color::argb(0x4D000000),
            success: Color::hex(0x6CCB5F),
            caution: Color::hex(0xFCE100),
            critical: Color::hex(0xFF99A4),
            neutral: Color::argb(0x8BFFFFFF),
            solid_neutral: Color::hex(0x9D9D9D),
            attention: ramp.light2,
            success_background: Color::hex(0x393D1B),
            caution_background: Color::hex(0x433519),
            critical_background: Color::hex(0x442726),
            neutral_background: Color::argb(0x08FFFFFF),
            attention_background: Color::argb(0x08FFFFFF),
            solid_attention_background: Color::hex(0x2E2E2E),
            solid_neutral_background: Color::hex(0x2E2E2E),
            data: DataColors {
                red: Color::hex(0xF1707B),
                orange: Color::hex(0xF7894A),
                yellow: Color::hex(0xFCE100),
                green: Color::hex(0x6CCB5F),
                teal: Color::hex(0x4BC1C5),
                blue: Color::hex(0x4CA7F0),
                purple: Color::hex(0xA992D4),
                pink: Color::hex(0xEE5FB7),
            },
        }
    }

    // How a color looks over the window's background.
    pub fn on_background(&self, color: Color) -> Color {
        color.over(self.background)
    }

    // How a card looks, opaque, over the window's background: what a 3D view
    // in a card should clear to, to match it.
    pub fn card_opaque(&self) -> Color {
        self.card_background.over(self.background)
    }

    // The color of a status: success, caution, critical or attention.
    pub fn status(&self, severity: crate::widgets::info_bar::Severity) -> Color {
        use crate::widgets::info_bar::Severity;
        match severity {
            Severity::Informational => self.attention,
            Severity::Success => self.success,
            Severity::Warning => self.caution,
            Severity::Error => self.critical,
        }
    }

    // Every role by its name, like "text_secondary", for showing the theme.
    pub fn roles(&self) -> Vec<(&'static str, Color)> {
        vec![
            ("text_primary", self.text_primary),
            ("text_secondary", self.text_secondary),
            ("text_tertiary", self.text_tertiary),
            ("text_disabled", self.text_disabled),
            ("accent_text", self.accent_text),
            ("text_on_accent", self.text_on_accent),
            ("accent", self.accent),
            ("accent_secondary", self.accent_secondary),
            ("accent_tertiary", self.accent_tertiary),
            ("accent_disabled", self.accent_disabled),
            ("control_fill", self.control_fill),
            ("control_fill_secondary", self.control_fill_secondary),
            ("control_fill_tertiary", self.control_fill_tertiary),
            ("control_fill_input_active", self.control_fill_input_active),
            ("control_strong_fill", self.control_strong_fill),
            ("control_solid_fill", self.control_solid_fill),
            ("control_alt_fill_secondary", self.control_alt_fill_secondary),
            ("control_alt_fill_tertiary", self.control_alt_fill_tertiary),
            ("subtle_fill_secondary", self.subtle_fill_secondary),
            ("subtle_fill_tertiary", self.subtle_fill_tertiary),
            ("control_stroke", self.control_stroke),
            ("control_stroke_secondary", self.control_stroke_secondary),
            ("control_strong_stroke", self.control_strong_stroke),
            ("card_stroke", self.card_stroke),
            ("surface_stroke_flyout", self.surface_stroke_flyout),
            ("divider", self.divider),
            ("background", self.background),
            ("background_secondary", self.background_secondary),
            ("background_tertiary", self.background_tertiary),
            ("card_background", self.card_background),
            ("card_background_secondary", self.card_background_secondary),
            ("layer", self.layer),
            ("acrylic", self.acrylic),
            ("smoke", self.smoke),
            ("success", self.success),
            ("caution", self.caution),
            ("critical", self.critical),
            ("attention", self.attention),
            ("success_background", self.success_background),
            ("caution_background", self.caution_background),
            ("critical_background", self.critical_background),
            ("attention_background", self.attention_background),
        ]
    }
}
