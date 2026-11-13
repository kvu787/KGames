// KGames' user interface library, for Win32, Direct3D 12 and DirectWrite,
// after Microsoft's Fluent 2 design system as WinUI 3 implements it on
// Windows 11, with WinUI 3's own values. See README.md.
//
// - application: what a program is, its settings, the main loop, and its
//   PerfBar's stats.
// - platform: the window, the clock, and GameInput's devices.
// - input: keys, buttons and the gamepad, and a frame's input from GameInput.
// - design: Fluent's theme, tokens and icons, and WinUI's controls' sizes.
// - drawing: geometry, the painter, text, and the GPU renderer.
// - core: a frame: the view's tree, its widgets' targets, the input on the
//   tree the user saw, the messages, focus, popups and tooltips.
// - widgets: WinUI's controls, cards, and the page's bars.
// - testing: scripts, and running a program without a window.

mod application {
    pub mod performance;
    pub mod program;
    pub mod run;
    pub mod settings;
}

mod platform {
    pub mod clock;
    pub mod input_devices;
    pub mod window;
}

pub mod input {
    pub mod devices;
    pub mod events;
    pub mod reader;
}

pub mod design {
    pub mod controls;
    pub mod icons;
    pub mod theme;
    pub mod tokens;
}

pub mod drawing {
    pub mod geometry;
    pub mod hashing;
    pub mod painter;
    pub mod renderer;
    pub mod text;
}

mod core {
    pub mod interaction;
    pub mod interface;
    pub mod tree;
}

mod widgets {
    pub mod buttons;
    pub mod card;
    pub mod color_picker;
    pub mod combo_box;
    pub mod dialog;
    pub mod expander;
    pub mod info_card;
    pub mod info_bar;
    pub mod layout;
    pub mod list_view;
    pub mod menu;
    pub mod navigation;
    pub mod number_box;
    pub mod palette;
    pub mod progress;
    pub mod scrolling;
    pub mod shell;
    pub mod slider;
    pub mod text;
    pub mod text_box;
    pub mod toggles;
    pub mod tool_tip;
    pub mod widget;
}

pub mod testing {
    pub mod headless;
    pub mod script;
}

// The design system, as AGENTS.md names it: `tokens` and `controls`.
pub use design::{controls, tokens};
// The clock that input, frames and the screen's flips are timed on, and the
// stats with the input latency that every program's PerfBar shows.
pub use application::performance;
pub use platform::clock;

pub use application::program::{Arguments, Notice, Options, Program, Startup};
pub use application::run::{run, state};
pub use application::settings::{CommonSettings, Settings, WindowMode};
pub use core::interaction::{ControlState, Cursor, Hit, Id, Interaction, Part, Sense, Target, Wheel, id};
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
pub use platform::input_devices::{Device, Devices, NextReadingFailure, Reading, after_next_reading_failed, frame_readings};
pub use platform::window::{FocusChange, Window};
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
pub use widgets::text::{Badge, Canvas, Divider, Field, IconView, KEYCAP_HEIGHT, Keycaps, Run, Spans, StatusIcon, Swatch, Text, TextColor, status_icon};
pub use widgets::text_box::TextBox;
pub use widgets::toggles::{CheckBox, RadioButton, RadioGroup, ToggleSwitch};
pub use widgets::widget::{Elevation, Reaction, StateColors};

// The crates that programs use with the interface, so that they share them.
pub use game_input;
pub use graphics;
pub use json;
