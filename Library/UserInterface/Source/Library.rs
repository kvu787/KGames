// KGames' user interface framework, which every GUI program on Win32,
// Direct3D 12 and DirectWrite is built with: the main loop that opens the
// window and draws frames with the lowest latency, an immediate-mode
// interface whose widgets each frame draws and asks what the input did, the
// header and footer around a program's panels, and the spacing that the
// programs share. It draws with Library/Canvas, in its light theme, which it
// passes on, so that programs need only this crate for their interface.
//
// See Interface.rs for how input reaches widgets, and README.md for the
// widgets.

#[path = "Chrome.rs"]
mod chrome;
#[path = "Controls.rs"]
mod controls;
#[path = "Headless.rs"]
pub mod headless;
#[path = "Interface.rs"]
mod interface;
#[path = "List.rs"]
mod list;
#[path = "Panels.rs"]
mod panels;
#[path = "Select.rs"]
mod select;
#[path = "Shell.rs"]
mod shell;
#[path = "Slider.rs"]
mod slider;
#[path = "TextField.rs"]
mod text_field;
#[path = "Tooltip.rs"]
mod tooltip;

pub use canvas::{Cached, Canvas, Color, Font, Lines, Rect, theme, wrap};
pub use controls::{Chip, Tone, chip_layout, chips};
pub use interface::{Frame, Id, Interface, Layer, Response, Sense, Status};
pub use list::{ListView, SCROLL_BAR_WIDTH};
pub use panels::{
    EDGE, FONT_SIZE, MARGIN, NOTICE_LINES, PADDING, PANELS_TOP, SHADOW, Stack, control_height, panel, pieces,
    text_box,
};
pub use select::{Choice, chevron, select_width};
pub use shell::{Program, Setup, run, state};
pub use slider::{HANDLE_RADIUS, SliderRange, track};
pub use tooltip::TOOLTIP_COLUMNS;
