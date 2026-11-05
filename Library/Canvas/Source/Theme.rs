// KGames' themes: the color of each part of an interface, by its role, in
// light and dark. They follow Fluent 2, Windows 11's design system, whose
// colors Windows' own apps use, with a tonal fill for what's on, like
// Material 3's, where Fluent has none. LIGHT is the default.
//
// The colors are sRGB, as the canvas draws them, and opaque, as they'd look
// on the surface they're for, unless a role says otherwise.

use crate::Color;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Theme {
    pub name: &'static str,
    pub dark: bool,

    // Surfaces, from the back: the window behind the cards, the cards, the
    // wells in them, like plots and logs, and what floats over them, like
    // popups and tooltips.
    pub background: Color,
    pub card: Color,
    pub card_stroke: Color,
    pub well: Color,
    pub well_stroke: Color,
    pub flyout: Color,
    pub flyout_stroke: Color,
    // A card over a 3D view, which shows a little of the view through it.
    pub overlay: Color,
    pub divider: Color,
    // Under what floats, before the shadow's falloff.
    pub shadow: Color,

    // Text, from the most to the least prominent.
    pub text: Color,
    pub text_secondary: Color,
    pub text_tertiary: Color,
    pub text_disabled: Color,

    // Controls: buttons, combo boxes and text fields, and their edges. The
    // bottom edge is darker in light, which raises them off the card.
    pub control: Color,
    pub control_hover: Color,
    pub control_pressed: Color,
    pub control_disabled: Color,
    pub control_stroke: Color,
    pub control_stroke_bottom: Color,
    // An unchecked box's outline, a switch's knob while it's off, a
    // slider's rail, and a text field's bottom line.
    pub control_strong: Color,
    // Inside an unchecked box and a switch that's off.
    pub control_alt: Color,
    // Behind a row or a subtle button that the cursor is over, or that's
    // pressed or selected. Translucent, for any surface.
    pub subtle_hover: Color,
    pub subtle_pressed: Color,
    pub subtle_selected: Color,

    // The accent, for what's chosen and the action that a card is for.
    pub accent: Color,
    pub accent_hover: Color,
    pub accent_pressed: Color,
    pub accent_disabled: Color,
    // Text and icons on the accent.
    pub on_accent: Color,
    pub on_accent_secondary: Color,
    // The accent as text, on a card.
    pub accent_text: Color,
    // A tonal fill of the accent, for a chip that's on.
    pub accent_tonal: Color,

    // The ring around the widget that has keyboard focus, and the gap inside
    // it.
    pub focus_outer: Color,
    pub focus_inner: Color,

    // Status, and the backgrounds of info bars about it.
    pub success: Color,
    pub success_background: Color,
    pub caution: Color,
    pub caution_background: Color,
    pub critical: Color,
    pub critical_background: Color,
    pub info_background: Color,

    // Colors for charts that show several things, which tell each other
    // apart, in this order: blue, orange, purple, teal, green, magenta.
    pub series: [Color; 6],
}

impl Theme {
    // The window's background, for Renderer::clear.
    pub fn clear_color(&self) -> [f32; 4] {
        self.background.to_floats()
    }

    pub fn blue(&self) -> Color {
        self.series[0]
    }

    pub fn orange(&self) -> Color {
        self.series[1]
    }

    pub fn purple(&self) -> Color {
        self.series[2]
    }

    pub fn teal(&self) -> Color {
        self.series[3]
    }

    pub fn green(&self) -> Color {
        self.series[4]
    }

    pub fn magenta(&self) -> Color {
        self.series[5]
    }
}

pub const LIGHT: Theme = Theme {
    name: "Light",
    dark: false,

    background: Color::rgb(0xF3F3F3),
    card: Color::rgb(0xFFFFFF),
    card_stroke: Color::rgb(0xE5E5E5),
    well: Color::rgb(0xF9F9F9),
    well_stroke: Color::rgb(0xEBEBEB),
    flyout: Color::rgb(0xFCFCFC),
    flyout_stroke: Color::rgb(0xE0E0E0),
    overlay: Color::rgba(0xFFFFFF, 0.92),
    divider: Color::rgb(0xEBEBEB),
    shadow: Color::rgba(0x000000, 0.14),

    text: Color::rgb(0x1B1B1B),
    text_secondary: Color::rgb(0x5F5F5F),
    text_tertiary: Color::rgb(0x8A8A8A),
    text_disabled: Color::rgb(0xA3A3A3),

    control: Color::rgb(0xFFFFFF),
    control_hover: Color::rgb(0xF6F6F6),
    control_pressed: Color::rgb(0xF0F0F0),
    control_disabled: Color::rgb(0xF5F5F5),
    control_stroke: Color::rgb(0xE5E5E5),
    control_stroke_bottom: Color::rgb(0xCCCCCC),
    control_strong: Color::rgb(0x8A8A8A),
    control_alt: Color::rgb(0xF9F9F9),
    subtle_hover: Color::rgba(0x000000, 0.04),
    subtle_pressed: Color::rgba(0x000000, 0.025),
    subtle_selected: Color::rgba(0x000000, 0.055),

    accent: Color::rgb(0x005FB8),
    accent_hover: Color::rgb(0x1A6FC0),
    accent_pressed: Color::rgb(0x337FC6),
    accent_disabled: Color::rgb(0xBFBFBF),
    on_accent: Color::rgb(0xFFFFFF),
    on_accent_secondary: Color::rgb(0xD9E7F4),
    accent_text: Color::rgb(0x005FB8),
    accent_tonal: Color::rgb(0xDCEAF7),

    focus_outer: Color::rgb(0x1B1B1B),
    focus_inner: Color::rgb(0xFFFFFF),

    success: Color::rgb(0x0F7B0F),
    success_background: Color::rgb(0xDFF6DD),
    caution: Color::rgb(0x9D5D00),
    caution_background: Color::rgb(0xFFF4CE),
    critical: Color::rgb(0xC42B1C),
    critical_background: Color::rgb(0xFDE7E9),
    info_background: Color::rgb(0xF6F6F6),

    series: [
        Color::rgb(0x005FB8),
        Color::rgb(0xCA5010),
        Color::rgb(0x8764B8),
        Color::rgb(0x038387),
        Color::rgb(0x107C10),
        Color::rgb(0xC239B3),
    ],
};

pub const DARK: Theme = Theme {
    name: "Dark",
    dark: true,

    background: Color::rgb(0x202020),
    card: Color::rgb(0x2B2B2B),
    card_stroke: Color::rgb(0x1C1C1C),
    well: Color::rgb(0x262626),
    well_stroke: Color::rgb(0x1E1E1E),
    flyout: Color::rgb(0x2C2C2C),
    flyout_stroke: Color::rgb(0x1A1A1A),
    overlay: Color::rgba(0x2B2B2B, 0.92),
    divider: Color::rgb(0x3A3A3A),
    shadow: Color::rgba(0x000000, 0.28),

    text: Color::rgb(0xFFFFFF),
    text_secondary: Color::rgb(0xC5C5C5),
    text_tertiary: Color::rgb(0x9A9A9A),
    text_disabled: Color::rgb(0x727272),

    control: Color::rgb(0x373737),
    control_hover: Color::rgb(0x3D3D3D),
    control_pressed: Color::rgb(0x323232),
    control_disabled: Color::rgb(0x333333),
    control_stroke: Color::rgb(0x434343),
    control_stroke_bottom: Color::rgb(0x3A3A3A),
    control_strong: Color::rgb(0x9A9A9A),
    control_alt: Color::rgb(0x262626),
    subtle_hover: Color::rgba(0xFFFFFF, 0.06),
    subtle_pressed: Color::rgba(0xFFFFFF, 0.04),
    subtle_selected: Color::rgba(0xFFFFFF, 0.08),

    accent: Color::rgb(0x60CDFF),
    accent_hover: Color::rgb(0x5BB9E6),
    accent_pressed: Color::rgb(0x52A5CC),
    accent_disabled: Color::rgb(0x5A5A5A),
    on_accent: Color::rgb(0x000000),
    on_accent_secondary: Color::rgb(0x1E3A4A),
    accent_text: Color::rgb(0x99EBFF),
    accent_tonal: Color::rgb(0x1F3B4D),

    focus_outer: Color::rgb(0xFFFFFF),
    focus_inner: Color::rgb(0x000000),

    success: Color::rgb(0x6CCB5F),
    success_background: Color::rgb(0x393D1B),
    caution: Color::rgb(0xFCE100),
    caution_background: Color::rgb(0x433519),
    critical: Color::rgb(0xFF99A4),
    critical_background: Color::rgb(0x442726),
    info_background: Color::rgb(0x323232),

    series: [
        Color::rgb(0x60CDFF),
        Color::rgb(0xF7894A),
        Color::rgb(0xB4A0FF),
        Color::rgb(0x4CC2C4),
        Color::rgb(0x6CCB5F),
        Color::rgb(0xEE5FB7),
    ],
};
