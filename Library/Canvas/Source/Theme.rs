// KGames' light theme. The colors are sRGB, as the canvas draws them.

use crate::Color;

// The window's background, behind the panels, for Renderer::clear.
pub const BACKGROUND: [f32; 4] = [0.933, 0.941, 0.953, 1.0];
pub const PANEL: Color = Color::rgb(0xFFFFFF);
// Plots and other wells inside a panel.
pub const INSET: Color = Color::rgb(0xF4F6F8);
pub const BORDER: Color = Color::rgb(0xD0D7DE);
pub const TITLE: Color = Color::rgb(0x14171C);
pub const TEXT: Color = Color::rgb(0x2E343C);
pub const DIM: Color = Color::rgb(0x646C78);
pub const FAINT: Color = Color::rgb(0xADB4BE);
// Blue, for what's selected, and the first of a pair.
pub const ACCENT: Color = Color::rgb(0x0969DA);
// Orange, for the second of a pair.
pub const ORANGE: Color = Color::rgb(0xC2550A);
pub const PURPLE: Color = Color::rgb(0x8250DF);
// Green, for what's on or held.
pub const SUCCESS: Color = Color::rgb(0x1A7F37);
// Text on ACCENT or SUCCESS.
pub const ON_COLOR: Color = Color::rgb(0xFFFFFF);
pub const WARNING: Color = Color::rgb(0x9A6700);
pub const ERROR: Color = Color::rgb(0xCF222E);
