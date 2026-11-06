// The design tokens of Fluent 2 for Windows: spacing, sizes, corner radii,
// the type ramp, motion, and elevation. Size and space everything with these,
// and color it with the theme's roles (see Theme.rs), so that every program
// looks like one family, and like Windows 11.
//
// Fluent 2: https://fluent2.microsoft.design. The values for controls are
// WinUI 3's, which is Fluent 2 on Windows.

use crate::text::{Font, TextStyle, Weight};

// Spacing, on Fluent's 4-pixel grid.
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

// Corner radii. Controls have 4, and surfaces that float or group, like
// cards, flyouts and dialogs, have 8.
pub const RADIUS_SMALL: f32 = 2.0;
pub const RADIUS_CONTROL: f32 = 4.0;
pub const RADIUS_OVERLAY: f32 = 8.0;

// Strokes.
pub const STROKE: f32 = 1.0;
pub const STROKE_THICK: f32 = 2.0;

// Controls' heights: standard, compact and large.
pub const CONTROL_HEIGHT: f32 = 32.0;
pub const CONTROL_HEIGHT_SMALL: f32 = 24.0;
pub const CONTROL_HEIGHT_LARGE: f32 = 40.0;
// A row of a list or a combo box's list.
pub const LIST_ROW_HEIGHT: f32 = 36.0;
// The pill that marks a list's or a navigation's selection.
pub const SELECTION_PILL_LENGTH: f32 = 16.0;
pub const SELECTION_PILL_THICKNESS: f32 = 3.0;
// Icons in controls, and on their own.
pub const ICON_SMALL: f32 = 12.0;
pub const ICON: f32 = 16.0;
pub const ICON_LARGE: f32 = 20.0;
// The ring that shows keyboard focus: 2 pixels outside a 1-pixel inner ring,
// 3 pixels away from the control.
pub const FOCUS_OFFSET: f32 = 3.0;

// The type ramp. Segoe UI Variable has optical sizes, which Windows names as
// families: Small for captions, Text for body text, and Display for titles.
pub const CAPTION: TextStyle = TextStyle::new(Font::Small, Weight::Regular, 12.0, 16.0);
pub const BODY: TextStyle = TextStyle::new(Font::Text, Weight::Regular, 14.0, 20.0);
pub const BODY_STRONG: TextStyle = TextStyle::new(Font::Text, Weight::Semibold, 14.0, 20.0);
pub const BODY_LARGE: TextStyle = TextStyle::new(Font::Text, Weight::Regular, 18.0, 24.0);
pub const SUBTITLE: TextStyle = TextStyle::new(Font::Display, Weight::Semibold, 20.0, 28.0);
pub const TITLE: TextStyle = TextStyle::new(Font::Display, Weight::Semibold, 28.0, 36.0);
pub const TITLE_LARGE: TextStyle = TextStyle::new(Font::Display, Weight::Semibold, 40.0, 52.0);
pub const DISPLAY: TextStyle = TextStyle::new(Font::Display, Weight::Semibold, 68.0, 92.0);
// Cascadia Mono, for text whose columns must line up, like logs and codes.
pub const MONO: TextStyle = TextStyle::new(Font::Mono, Weight::Regular, 13.0, 20.0);
pub const MONO_STRONG: TextStyle = TextStyle::new(Font::Mono, Weight::Semibold, 13.0, 20.0);

// Motion. Controls change color over DURATION_FASTER, move over
// DURATION_FAST, and surfaces open over DURATION_NORMAL, decelerating.
pub const DURATION_FASTER: f32 = 0.083;
pub const DURATION_FAST: f32 = 0.167;
pub const DURATION_NORMAL: f32 = 0.25;
pub const DURATION_SLOW: f32 = 0.333;

// Easing curves, as CSS's cubic-bezier.
#[derive(Clone, Copy, Debug)]
pub struct Curve(pub f32, pub f32, pub f32, pub f32);

pub const CURVE_LINEAR: Curve = Curve(0.0, 0.0, 1.0, 1.0);
// Fast out, slow in: for things that enter or move into place.
pub const CURVE_DECELERATE: Curve = Curve(0.0, 0.0, 0.0, 1.0);
// For things that leave.
pub const CURVE_ACCELERATE: Curve = Curve(1.0, 0.0, 1.0, 1.0);
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
        // Solve x(s) = t for s by bisection, which is exact enough and never
        // diverges, then take y(s).
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

// Elevation: each level is two shadows, a soft one all around and a sharper
// one below, as Fluent's shadow tokens are. Blur is CSS's: twice the
// Gaussian's standard deviation.
#[derive(Clone, Copy, Debug)]
pub struct Shadow {
    pub offset_y: f32,
    pub blur: f32,
    // In the light theme and the dark one, whose shadows are stronger.
    pub opacity_light: f32,
    pub opacity_dark: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct Elevation {
    pub ambient: Shadow,
    pub key: Shadow,
}

const fn elevation(ambient_blur: f32, key_offset: f32, key_blur: f32) -> Elevation {
    Elevation {
        ambient: Shadow {
            offset_y: 0.0,
            blur: ambient_blur,
            opacity_light: 0.12,
            opacity_dark: 0.24,
        },
        key: Shadow {
            offset_y: key_offset,
            blur: key_blur,
            opacity_light: 0.14,
            opacity_dark: 0.28,
        },
    }
}

// Cards and controls that rest on a surface.
pub const ELEVATION_2: Elevation = elevation(2.0, 1.0, 2.0);
pub const ELEVATION_4: Elevation = elevation(2.0, 2.0, 4.0);
pub const ELEVATION_8: Elevation = elevation(2.0, 4.0, 8.0);
// Tooltips and the cards that float over a 3D view.
pub const ELEVATION_16: Elevation = elevation(2.0, 8.0, 16.0);
// Flyouts, like a combo box's list.
pub const ELEVATION_28: Elevation = elevation(8.0, 14.0, 28.0);
// Dialogs.
pub const ELEVATION_64: Elevation = elevation(8.0, 32.0, 64.0);

// How KGames lays out a program's window, in fixed pixels: the performance
// stats along the very top, which every program has, the app bar under them,
// a status bar at the bottom, and cards between them, PAGE_PADDING from the
// window's edges and CARD_GAP from each other.
pub const STATS_BAR_HEIGHT: f32 = 30.0;
pub const APP_BAR_HEIGHT: f32 = 48.0;
pub const STATUS_BAR_HEIGHT: f32 = 32.0;
pub const PAGE_PADDING: f32 = SPACE_L;
pub const CARD_GAP: f32 = SPACE_L;
pub const CARD_PADDING: f32 = SPACE_L;
// A card's header, which is a control tall, so that controls fit beside its
// title, and the space under it.
pub const CARD_HEADER_HEIGHT: f32 = CONTROL_HEIGHT;
pub const CARD_HEADER_GAP: f32 = SPACE_S;
// The pixels that cards and bars have in each of the two supported window
// setups, below the title bar.
pub const MAXIMIZED_HEIGHT: f32 = 1369.0;
pub const FULLSCREEN_HEIGHT: f32 = 1440.0;
pub const WINDOW_WIDTH: f32 = 2560.0;
