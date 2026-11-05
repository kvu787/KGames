// The design system's sizes, after Fluent 2, Windows 11's: spacing on a
// 4-pixel grid, corner radii, the heights of controls, how high what floats
// is, and how fast widgets change how they look. Programs only support 100%
// scaling and two client areas (see Library/Application), so these are
// fixed pixels.
//
// Colors are the theme's, in Library/Canvas.

// Spacing, on a 4-pixel grid.
pub const SPACE_XXS: f32 = 2.0;
pub const SPACE_XS: f32 = 4.0;
pub const SPACE_S: f32 = 8.0;
pub const SPACE_M: f32 = 12.0;
pub const SPACE_L: f32 = 16.0;
pub const SPACE_XL: f32 = 20.0;
pub const SPACE_XXL: f32 = 24.0;
pub const SPACE_XXXL: f32 = 32.0;

// Corner radii: controls, like buttons and fields, and what holds them,
// like cards, popups and info bars.
pub const RADIUS_CONTROL: f32 = 4.0;
pub const RADIUS_CARD: f32 = 8.0;

// Between cards, and between them and the window's left, right and bottom
// edges.
pub const MARGIN: f32 = 16.0;
// Inside a card, around its contents.
pub const CARD_PADDING: f32 = 16.0;
// From a card's top to its contents, under its title.
pub const CARD_HEADER: f32 = CARD_PADDING + 20.0 + SPACE_M;

// The app bar at the top of the window, with the program's title, and the
// status bar at its bottom, with its shortcuts. Cards go between them.
pub const PANELS_TOP: f32 = 56.0;
pub const STATUS_BAR_HEIGHT: f32 = 40.0;

// A control with a line of text, like a button, a combo box or a field.
pub const CONTROL_HEIGHT: f32 = 32.0;
// A checkbox's box and a radio button's circle.
pub const MARK_SIZE: f32 = 20.0;
// Between a mark, like a checkbox's box, and its label.
pub const LABEL_GAP: f32 = 8.0;
// A switch's track.
pub const SWITCH_SIZE: [f32; 2] = [40.0, 20.0];
// A chip, and a tab.
pub const CHIP_HEIGHT: f32 = 28.0;
pub const TAB_HEIGHT: f32 = 40.0;
// A slider's thumb, which its track stops short of at each end.
pub const THUMB_RADIUS: f32 = 10.0;
// A list's row, by default.
pub const ROW_HEIGHT: f32 = 36.0;
// Text inside a control, from its left.
pub const TEXT_INSET: f32 = 12.0;
// The pill that marks what's selected in a list or a combo box's list.
pub const PILL_SIZE: [f32; 2] = [3.0, 16.0];

// How high something floats over what's under it, as its shadow: how far
// down it falls, how blurred it is, and how dark, of the theme's shadow.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Elevation {
    pub offset: f32,
    pub blur: f32,
    pub strength: f32,
}

// A card over a 3D view.
pub const RAISED: Elevation = Elevation { offset: 2.0, blur: 4.0, strength: 0.8 };
// A tooltip.
pub const TOOLTIP_ELEVATION: Elevation = Elevation { offset: 4.0, blur: 4.0, strength: 1.0 };
// A combo box's list, and an info bar over the cards.
pub const FLYOUT: Elevation = Elevation { offset: 8.0, blur: 8.0, strength: 1.0 };

// How fast a widget's look follows what the input did to it, as the time
// constant of an exponential ease: 95% of the way in 3 of them. Values
// change on the frame that the input comes, and only how they look eases,
// like a switch's knob sliding over, so that the response starts at once.
pub const EASE_SECONDS: f64 = 0.025;
