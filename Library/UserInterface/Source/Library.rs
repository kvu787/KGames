// KGames' user interface library, for Win32, Direct3D 12 and DirectWrite,
// after Microsoft's Fluent 2 design system as WinUI 3 implements it on
// Windows 11, with WinUI 3's own values. See README.md.
//
// - application: what a program is, its settings, the main loop, and its
//   PerfBar's stats.
// - platform: the window, the clipboard, the clock, and GameInput's devices.
// - input: keys, buttons and the gamepad, and a frame's input from GameInput.
// - design: Fluent's theme, tokens and icons, and WinUI's controls' sizes.
// - drawing: geometry, the painter, text, and the GPU renderer.
// - core: a frame: the view's tree, its widgets' targets, the input on the
//   tree the user saw, the messages, focus, popups and tooltips.
// - widgets: WinUI's controls, cards, and the page's bars.
// - testing: scripts, and running a program without a window.

mod application {
    #[path = "Overlays.rs"]
    pub mod overlays;
    #[path = "Performance.rs"]
    pub mod performance;
    #[path = "Program.rs"]
    pub mod program;
    #[path = "Run.rs"]
    pub mod run;
    #[path = "Settings.rs"]
    pub mod settings;
}

mod platform {
    #[path = "Clipboard.rs"]
    pub mod clipboard;
    #[path = "Clock.rs"]
    pub mod clock;
    #[path = "InputDevices.rs"]
    pub mod input_devices;
    #[path = "Window.rs"]
    pub mod window;
}

pub mod input {
    #[path = "Devices.rs"]
    pub mod devices;
    #[path = "Events.rs"]
    pub mod events;
    #[path = "Reader.rs"]
    pub mod reader;
}

pub mod design {
    #[path = "Controls.rs"]
    pub mod controls;
    #[path = "Icons.rs"]
    pub mod icons;
    #[path = "Theme.rs"]
    pub mod theme;
    #[path = "Tokens.rs"]
    pub mod tokens;
}

pub mod drawing {
    #[path = "Geometry.rs"]
    pub mod geometry;
    #[path = "Painter.rs"]
    pub mod painter;
    #[path = "Renderer.rs"]
    pub mod renderer;
    #[path = "Text.rs"]
    pub mod text;
}

mod core {
    #[path = "Interaction.rs"]
    pub mod interaction;
    #[path = "Interface.rs"]
    pub mod interface;
    #[path = "Tree.rs"]
    pub mod tree;
}

mod widgets {
    #[path = "Buttons.rs"]
    pub mod buttons;
    #[path = "Card.rs"]
    pub mod card;
    #[path = "ColorPicker.rs"]
    pub mod color_picker;
    #[path = "ComboBox.rs"]
    pub mod combo_box;
    #[path = "Dialog.rs"]
    pub mod dialog;
    #[path = "Expander.rs"]
    pub mod expander;
    #[path = "InfoCard.rs"]
    pub mod info_card;
    #[path = "InfoBar.rs"]
    pub mod info_bar;
    #[path = "Layout.rs"]
    pub mod layout;
    #[path = "ListView.rs"]
    pub mod list_view;
    #[path = "Menu.rs"]
    pub mod menu;
    #[path = "Navigation.rs"]
    pub mod navigation;
    #[path = "NumberBox.rs"]
    pub mod number_box;
    #[path = "Palette.rs"]
    pub mod palette;
    #[path = "Progress.rs"]
    pub mod progress;
    #[path = "Scrolling.rs"]
    pub mod scrolling;
    #[path = "Shell.rs"]
    pub mod shell;
    #[path = "Slider.rs"]
    pub mod slider;
    #[path = "Text.rs"]
    pub mod text;
    #[path = "TextBox.rs"]
    pub mod text_box;
    #[path = "Toggles.rs"]
    pub mod toggles;
    #[path = "ToolTip.rs"]
    pub mod tool_tip;
    #[path = "Widget.rs"]
    pub mod widget;
}

pub mod testing {
    #[path = "Headless.rs"]
    pub mod headless;
    #[path = "Script.rs"]
    pub mod script;
}

// The design system, as AGENTS.md names it: `tokens` and `controls`.
pub use design::{controls, tokens};
// The clock that input, frames and the screen's flips are timed on, and the
// stats with the input latency that every program's PerfBar shows.
pub use application::performance;
pub use platform::clock;

pub use application::overlays::{Overlay, run_overlays};
pub use application::program::{Arguments, Notice, Options, Program, Startup};
pub use application::run::{run, state};
pub use application::settings::{Settings, WindowMode};
pub use core::interaction::{ControlState, Cursor, Hit, Id, Interaction, Keys, Part, Sense, Target, Wheel, id};
pub use core::interface::{Context, Environment, FramePhases, FrameStats, Interface, Ui};
pub use core::tree::{Dispatch, Draw, Element, Look, Parts, Tooltip, TooltipLine, Tree, Widget};
pub use design::icons::Icon;
pub use design::theme::{AccentRamp, DataColors, Theme, ThemeKind};
pub use drawing::geometry::{Color, Insets, Point, Rect, Size};
pub use drawing::painter::{Align, Border, Brush, Corners, DrawList, Layer, Painter, Span};
pub use drawing::renderer::Renderer;
pub use drawing::text::{Font, Fonts, TextStyle, Weight};
pub use input::devices::{Gamepad, GamepadButtons, Key, Modifiers, MouseButton, Shortcut};
pub use input::events::{Event, Input};
pub use platform::clipboard::Clipboard;
pub use platform::input_devices::{Device, Devices, NextReadingFailure, Reading, after_next_reading_failed};
pub use platform::window::FocusChange;
pub use testing::headless::Headless;
pub use testing::script::{Script, ScriptRunner};
pub use widgets::buttons::{Button, ButtonStyle, HyperlinkButton, ToggleButton};
pub use widgets::card::{Card, CardArea, CardStyle};
pub use widgets::color_picker::{ColorField, ColorPicker};
pub use widgets::combo_box::ComboBox;
pub use widgets::dialog::{Dialog, DialogArea, DialogResult};
pub use widgets::expander::{Expander, ExpanderContent};
pub use widgets::info_bar::{InfoBar, Severity};
pub use widgets::info_card::InfoCard;
pub use widgets::layout::{Cover, Form, Group, View, ViewInput};
pub use widgets::list_view::{ListRows, ListView, WHEEL_ROWS};
pub use widgets::menu::{MenuBar, MenuItem};
pub use widgets::navigation::{Choice, Segmented, SelectorBar};
pub use widgets::number_box::NumberBox;
pub use widgets::palette::Palette;
pub use widgets::progress::{ProgressBar, ProgressRing};
pub use widgets::scrolling::{ScrollArea, ScrollBar, Scrolled, WHEEL_SCROLL};
pub use widgets::shell::{AppBar, MenuRow, Shell, Status, StatusBar, unsupported_setup};
pub use widgets::slider::{Slider, slider_rail};
pub use widgets::text::{Badge, Canvas, Divider, Field, IconView, KEYCAP_HEIGHT, Keycaps, Run, Spans, StatusIcon, Swatch, Text, TextColor};
pub use widgets::text_box::TextBox;
pub use widgets::toggles::{CheckBox, RadioButton, RadioGroup, ToggleSwitch};
pub use widgets::widget::{Elevation, Reaction, StateColors, focus_visual};

// The crates that programs use with the interface, so that they share them.
pub use game_input;
pub use graphics;
pub use json;
