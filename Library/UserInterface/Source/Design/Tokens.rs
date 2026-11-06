// Fluent 2's design tokens on Windows: spacing, corner radii, sizes, the type
// ramp, motion and elevation, with the values WinUI 3 gives them. Size and
// space everything with these, and color it with the theme's roles (see
// Theme.rs), so that every program looks like one family, and like Windows
// 11. Controls' own sizes, like a checkbox's box, are in Controls.rs.
//
// Fluent 2: https://fluent2.microsoft.design. WinUI 3's values come from
// microsoft/microsoft-ui-xaml's theme resources.

use crate::drawing::text::{Font, TextStyle, Weight};

// Spacing, on Fluent's 4-pixel grid, with its 2-pixel and 6-pixel nudges.
pub const SPACE_NONE: f32 = 0.0;
pub const SPACE_XXS: f32 = 2.0;
pub const SPACE_XS: f32 = 4.0;
pub const SPACE_S_NUDGE: f32 = 6.0;
pub const SPACE_S: f32 = 8.0;
pub const SPACE_M_NUDGE: f32 = 10.0;
pub const SPACE_M: f32 = 12.0;
pub const SPACE_L: f32 = 16.0;
pub const SPACE_XL: f32 = 20.0;
pub const SPACE_XXL: f32 = 24.0;
pub const SPACE_XXXL: f32 = 32.0;
pub const SPACE_HUGE: f32 = 40.0;

// Corner radii: ControlCornerRadius for controls, and OverlayCornerRadius
// for what floats or groups, like flyouts, dialogs and cards.
pub const RADIUS_NONE: f32 = 0.0;
pub const RADIUS_SMALL: f32 = 2.0;
pub const RADIUS_CONTROL: f32 = 4.0;
pub const RADIUS_OVERLAY: f32 = 8.0;
pub const RADIUS_CIRCLE: f32 = 10_000.0;

// Strokes.
pub const STROKE_THIN: f32 = 1.0;
pub const STROKE_THICK: f32 = 2.0;

// Controls' heights: WinUI's standard, compact and large.
pub const CONTROL_HEIGHT: f32 = 32.0;
pub const CONTROL_HEIGHT_COMPACT: f32 = 24.0;
pub const CONTROL_HEIGHT_LARGE: f32 = 40.0;
// Icons in controls, and on their own: WinUI's glyph sizes.
pub const ICON_SMALL: f32 = 12.0;
pub const ICON: f32 = 16.0;
pub const ICON_LARGE: f32 = 20.0;
pub const ICON_EXTRA_LARGE: f32 = 24.0;

// The type ramp: Windows' sizes and line heights, each in the optical size
// of Segoe UI Variable that Fluent 2 sets it in.
pub const CAPTION: TextStyle = TextStyle::new(Font::Small, Weight::Regular, 12.0, 16.0);
pub const CAPTION_STRONG: TextStyle = TextStyle::new(Font::Small, Weight::Semibold, 12.0, 16.0);
pub const BODY: TextStyle = TextStyle::new(Font::Text, Weight::Regular, 14.0, 20.0);
pub const BODY_STRONG: TextStyle = TextStyle::new(Font::Text, Weight::Semibold, 14.0, 20.0);
pub const BODY_LARGE: TextStyle = TextStyle::new(Font::Text, Weight::Regular, 18.0, 24.0);
pub const BODY_LARGE_STRONG: TextStyle = TextStyle::new(Font::Text, Weight::Semibold, 18.0, 24.0);
pub const SUBTITLE: TextStyle = TextStyle::new(Font::Display, Weight::Semibold, 20.0, 28.0);
pub const TITLE: TextStyle = TextStyle::new(Font::Display, Weight::Semibold, 28.0, 36.0);
pub const TITLE_LARGE: TextStyle = TextStyle::new(Font::Display, Weight::Semibold, 40.0, 52.0);
pub const DISPLAY: TextStyle = TextStyle::new(Font::Display, Weight::Semibold, 68.0, 92.0);
// Cascadia Mono, for text whose columns line up, like logs and codes.
pub const MONO: TextStyle = TextStyle::new(Font::Mono, Weight::Regular, 13.0, 20.0);
pub const MONO_STRONG: TextStyle = TextStyle::new(Font::Mono, Weight::Semibold, 13.0, 20.0);

// Motion: WinUI's ControlFasterAnimationDuration, for colors, Fast, for
// things that move, and Normal, for surfaces that open.
pub const DURATION_FASTER: f32 = 0.083;
pub const DURATION_FAST: f32 = 0.167;
pub const DURATION_NORMAL: f32 = 0.25;
pub const DURATION_SLOW: f32 = 0.333;

// How long the pointer rests on a control before its tooltip shows.
pub const TOOLTIP_DELAY: f64 = 0.5;

// An easing curve, as a cubic Bézier from (0, 0) to (1, 1) through two
// points, like XAML's KeySpline.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Curve(pub f32, pub f32, pub f32, pub f32);

pub const CURVE_LINEAR: Curve = Curve(0.0, 0.0, 1.0, 1.0);
// WinUI's ControlFastOutSlowInKeySpline, for what enters or moves into place.
pub const CURVE_FAST_OUT_SLOW_IN: Curve = Curve(0.0, 0.0, 0.0, 1.0);
// For what leaves.
pub const CURVE_SLOW_IN_FAST_OUT: Curve = Curve(1.0, 0.0, 1.0, 1.0);
// Fluent's easy ease, for what moves in place.
pub const CURVE_EASY_EASE: Curve = Curve(0.33, 0.0, 0.67, 1.0);

impl Curve {
    // The curve's progress at time t, both from 0 to 1.
    pub fn ease(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        let Curve(x1, y1, x2, y2) = *self;
        let bezier = |a: f32, b: f32, s: f32| {
            let u = 1.0 - s;
            3.0 * u * u * s * a + 3.0 * u * s * s * b + s * s * s
        };
        // x(s) = t by bisection, which never diverges, then y(s).
        let (mut low, mut high) = (0.0f32, 1.0f32);
        for _ in 0..24 {
            let middle = (low + high) / 2.0;
            if bezier(x1, x2, middle) < t {
                low = middle;
            } else {
                high = middle;
            }
        }
        bezier(y1, y2, (low + high) / 2.0)
    }
}

// A drop shadow: black, below the surface, blurred as Windows' composition
// blurs, whose Gaussian's standard deviation is a third of the radius.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shadow {
    pub offset_y: f32,
    pub blur: f32,
    pub opacity_light: f32,
    pub opacity_dark: f32,
}

impl Shadow {
    const NONE: Shadow = Shadow {
        offset_y: 0.0,
        blur: 0.0,
        opacity_light: 0.0,
        opacity_dark: 0.0,
    };

    pub fn sigma(&self) -> f32 {
        (self.blur / 3.0).max(0.5)
    }
}

// An elevation: WinUI's ThemeShadow for a surface raised by Translation Z,
// which is two drop shadows, an ambient one and a directional one.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Elevation {
    pub z: f32,
    pub shadows: [Shadow; 2],
}

// ThemeShadow's recipe for a Translation Z, from WinUI's DropShadowRecipe.h:
// the elevation is half the Z, up to 64. Up to 16, there's only the
// directional shadow; above, the ambient one too, and both are stronger.
pub const fn elevation(z: f32) -> Elevation {
    let level = if z / 2.0 < 64.0 { z / 2.0 } else { 64.0 };
    if level < 2.0 {
        return Elevation {
            z,
            shadows: [Shadow::NONE, Shadow::NONE],
        };
    }
    let (ambient, directional_light, directional_dark) = if level <= 16.0 {
        let light = level / 100.0 + 0.06;
        (Shadow::NONE, if light < 0.14 { light } else { 0.14 }, 0.26)
    } else {
        let ambient = Shadow {
            offset_y: 2.0,
            blur: level / 3.0,
            opacity_light: 0.15,
            opacity_dark: 0.37,
        };
        (ambient, 0.19, 0.37)
    };
    let directional = Shadow {
        offset_y: level * 0.5,
        blur: level,
        opacity_light: directional_light,
        opacity_dark: directional_dark,
    };
    Elevation {
        z,
        shadows: [ambient, directional],
    }
}

// The Z that WinUI raises each kind of surface by.
pub const ELEVATION_TOOLTIP: Elevation = elevation(16.0);
pub const ELEVATION_FLYOUT: Elevation = elevation(32.0);
pub const ELEVATION_SUBMENU: Elevation = elevation(40.0);
pub const ELEVATION_DIALOG: Elevation = elevation(128.0);
// A card over a 3D view floats like a flyout.
pub const ELEVATION_FLOATING_CARD: Elevation = ELEVATION_FLYOUT;

// How KGames lays out a program's window, in fixed pixels: the performance
// stats along the very top, which every program has, an app bar under them,
// which is as tall as WinUI's TitleBar control, a status bar at the bottom,
// and cards between them, PAGE_PADDING from the window's edges and CARD_GAP
// from each other.
pub const STATS_BAR_HEIGHT: f32 = 30.0;
pub const APP_BAR_HEIGHT: f32 = 48.0;
pub const STATUS_BAR_HEIGHT: f32 = 32.0;
pub const PAGE_PADDING: f32 = SPACE_L;
pub const CARD_GAP: f32 = SPACE_L;
pub const CARD_PADDING: f32 = SPACE_L;
// A card's header is a control tall, so that controls fit beside its title,
// with a gap under it.
pub const CARD_HEADER_HEIGHT: f32 = CONTROL_HEIGHT;
pub const CARD_HEADER_GAP: f32 = SPACE_S;
// A card's form rows: a label above a control, or beside it.
pub const FORM_LABEL_GAP: f32 = SPACE_XS;
pub const FORM_ROW_GAP: f32 = SPACE_M;

// The two supported window setups' client areas.
pub const WINDOW_WIDTH: f32 = 2560.0;
pub const MAXIMIZED_HEIGHT: f32 = 1369.0;
pub const FULLSCREEN_HEIGHT: f32 = 1440.0;
