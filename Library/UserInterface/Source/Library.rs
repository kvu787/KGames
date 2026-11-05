// KGames' user interface framework, which every GUI program on Win32,
// Direct3D 12 and DirectWrite is built with: the main loop that opens the
// window and draws frames with the lowest latency, an immediate-mode
// interface whose widgets each frame draws and asks what the input did, in
// the style of Fluent 2, Windows 11's design system, and the app bar and
// status bar around a program's cards. It draws with Library/Canvas, in its
// light theme by default, which it passes on, so that programs need only this
// crate for their interface.
//
// See Interface.rs for how input reaches widgets, Tokens.rs for the design
// system's sizes, and README.md for the widgets.

#[path = "Buttons.rs"]
mod buttons;
#[path = "Chips.rs"]
mod chips;
#[path = "Chrome.rs"]
mod chrome;
#[path = "Clipboard.rs"]
mod clipboard;
#[path = "Headless.rs"]
pub mod headless;
#[path = "Interface.rs"]
mod interface;
#[path = "List.rs"]
mod list;
#[path = "Select.rs"]
mod select;
#[path = "Shell.rs"]
mod shell;
#[path = "Slider.rs"]
mod slider;
#[path = "Surfaces.rs"]
mod surfaces;
#[path = "TextField.rs"]
mod text_field;
#[path = "Toggles.rs"]
mod toggles;
#[path = "Tokens.rs"]
pub mod tokens;
#[path = "Tooltip.rs"]
mod tooltip;

pub use application::{Gamepad, GamepadButton, key_name};
pub use buttons::{Button, ButtonStyle};
pub use canvas::{Align, Axis, Cached, Canvas, Color, Fonts, ICON_SIZES, Icon, Lines, Rect, Stack, TextStyle, Theme, theme};
pub use chips::{Chip, badge, badge_width, chips_height, keycap, keycap_width, on_color, status_chips};
pub use clipboard::Clipboard;
pub use interface::{Frame, Id, Interface, Layer, Response, Sense, Status};
pub use list::{ListRow, ListView, SCROLL_BAR_WIDTH};
pub use select::{Choice, select_width};
pub use shell::{Program, Setup, run, state};
pub use slider::{SliderRange, slider_rail};
pub use surfaces::{Severity, card, card_trailing, divider, focus_ring, overlay_card, raised, shadow, well};
pub use text_field::TextField;
pub use toggles::{tab_widths, toggle_width};
pub use tokens::*;
pub use tooltip::{TOOLTIP_WIDTH, TooltipText};
