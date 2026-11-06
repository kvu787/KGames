// KGames' user interface library, for Win32, Direct3D 12 and DirectWrite,
// after Microsoft's Fluent 2 design system: its tokens and themes, WinUI's
// controls, a window and its input, text, a GPU renderer, the main loop of
// every GUI program, scripts that test programs in their real windows, and a
// harness that tests them without one. See README.md.

#[path = "Clipboard.rs"]
mod clipboard;
#[path = "Clock.rs"]
pub mod clock;
#[path = "Devices.rs"]
pub mod devices;
#[path = "Geometry.rs"]
pub mod geometry;
#[path = "Headless.rs"]
pub mod headless;
#[path = "Icons.rs"]
mod icons;
#[path = "Input.rs"]
pub mod input;
#[path = "InputReader.rs"]
mod input_reader;
#[path = "Overlay.rs"]
mod overlay;
#[path = "Painter.rs"]
pub mod painter;
#[path = "Program.rs"]
mod program;
#[path = "Renderer.rs"]
mod renderer;
#[path = "Run.rs"]
mod run;
#[path = "Script.rs"]
pub mod script;
#[path = "Settings.rs"]
mod settings;
#[path = "Text.rs"]
pub mod text;
#[path = "Theme.rs"]
mod theme;
#[path = "Tokens.rs"]
pub mod tokens;
#[path = "Ui.rs"]
mod ui;
#[path = "Window.rs"]
pub mod window;

mod widgets {
    #[path = "Buttons.rs"]
    pub mod buttons;
    #[path = "ComboBox.rs"]
    pub mod combo_box;
    #[path = "Common.rs"]
    pub mod common;
    #[path = "List.rs"]
    pub mod list;
    #[path = "Navigation.rs"]
    pub mod navigation;
    #[path = "Shell.rs"]
    pub mod shell;
    #[path = "Slider.rs"]
    pub mod slider;
    #[path = "Surfaces.rs"]
    pub mod surfaces;
    #[path = "TextBox.rs"]
    pub mod text_box;
    #[path = "Toggles.rs"]
    pub mod toggles;
}

pub use clipboard::Clipboard;
pub use devices::{Device, Devices, Reading};
pub use geometry::{Color, Point, Rect, Size};
pub use headless::Headless;
pub use icons::Icon;
pub use input::{Button as MouseButton, Event, Gamepad, GamepadButtons, Input, Key, Modifiers};
pub use overlay::{Overlay, run_overlays};
pub use painter::{Align, Corners, DrawList, Layer, Painter, Stroke};
pub use program::{Arguments, Options, Program, Startup};
pub use renderer::Renderer;
pub use run::{run, state};
pub use settings::{Settings, WindowMode};
pub use text::{Font, Fonts, TextStyle, Weight};
pub use theme::{DataColors, Theme, ThemeKind};
pub use ui::{Cursor, Environment, FrameStats, Id, Keys, Response, Sense, Target, TOOLTIP_DELAY, Ui, id};
pub use widgets::buttons::{BUTTON_PADDING, Button, ButtonStyle};
pub use widgets::common::{Visual, focus_ring};
pub use widgets::list::{ListResponse, RowState, WHEEL_ROWS};
pub use widgets::shell::{Notice, Status};
pub use widgets::slider::{Slider, slider_rail};
pub use widgets::surfaces::{Card, InfoBar, Severity, TooltipLine};
pub use widgets::text_box::{TextBox, TextBoxResponse};

// The crates that programs need with the interface, so that they share them.
pub use game_input;
pub use graphics;
pub use json;
