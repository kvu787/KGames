// Fluent's shadow, for reference: the one Library/user-interface draws for
// its flyouts and dialogs, WinUI's ThemeShadow for a surface raised by a Z,
// a directional Gaussian and, above a Z of 32, an ambient one, which are
// stronger in the dark theme. The interface draws it, in a Canvas on the
// stage, with Painter::shadow, rather than a shader of the program's.

use user_interface::tokens::elevation;
use user_interface::{Painter, Rect};

use crate::approaches::Setting;
use crate::shadow_testing::BOX_COLOR;

pub const DESCRIPTION: &str = "The library's own shadow, WinUI's ThemeShadow for a surface raised by Z: 16 for tooltips, 32 flyouts, 128 dialogs.";

pub const SETTINGS: &[Setting] = &[Setting {
    name: "Z",
    label: "Z",
    minimum: 0.0,
    maximum: 128.0,
    step: 1.0,
    default: 32.0,
    decimals: 0,
    suffix: "",
}];

// Draws the box over its shadow, only inside the stage.
pub fn draw(painter: &mut Painter, values: &[f32], stage: Rect, rect: Rect) {
    painter.push_clip(stage);
    painter.shadow(rect, 0.0, elevation(values[0]));
    painter.fill(rect, BOX_COLOR);
    painter.pop_clip();
}
