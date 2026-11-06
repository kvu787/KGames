# UserInterface

KGames' user interface library, for Win32, Direct3D 12 and DirectWrite, after Microsoft's
[Fluent 2](https://fluent2.microsoft.design) design system as WinUI 3 implements it on Windows 11,
with WinUI 3's own values: its theme resources' colors, its controls' sizes, paddings and parts,
its elevation borders and shadows, and its motion. Every GUI program is built on it: it owns the
window, reads the input, runs the main loop, lays out text, draws the widgets on the GPU, and runs
scripts and headless tests. [UiShowcase](../../Program/UiShowcase/README.md) shows every widget.

It's immediate mode: each frame, a program's `update` lays out its cards in fixed pixels and adds
each control with `ui.add(name, rect, widget)`, which draws it and returns what the input did to
it. State that the program owns, like a slider's value, is the program's, passed to the widget as
`&mut`. State that only the widget needs, like a text field's caret or a list's scroll, the library
keeps by the widget's name.

## A program

```rust
use user_interface::{AppBar, Button, Card, Icon, Options, Program, Slider, Startup, Status, Ui};

struct Volume {
    level: f64,
}

impl Program for Volume {
    fn title(&self) -> String {
        "Volume".to_string()
    }

    fn update(&mut self, ui: &mut Ui) {
        ui.app_bar(AppBar::new(Icon::Volume, "Volume"));
        ui.status_bar(&[Status::Shortcut("F11", "Fullscreen")]);
        let page = ui.page();
        let card = ui.card(page.left_strip(400.0).top_strip(200.0), Card::new("Output"));
        let mut area = card.content;
        let slider = Slider::new(&mut self.level, 0.0, 100.0).step(1.0).header("Level").show_value(0);
        ui.add("Level", area.cut_top(Slider::<f64>::height(true), 8.0), slider);
        if ui.add("Mute", area.cut_top(32.0, 0.0).left_strip(120.0), Button::new("Mute")).clicked {
            self.level = 0.0;
        }
    }
}

fn main() -> std::process::ExitCode {
    user_interface::run(Options::new("Volume", env!("CARGO_MANIFEST_DIR")), |_startup: &mut Startup| {
        Ok(Volume { level: 50.0 })
    })
}
```

`run` reads `Settings.json` and the command line, opens the window on the chosen display, creates
the program, and runs the main loop until the window closes. A program can also draw 3D: its
`prepare` creates GPU resources once, and its `render` records commands into each frame, after
the frame is cleared to the theme's background and before the interface draws over it, where a
card made with `card_with_view` leaves a hole.

| `Program` method | What it's for                                                              |
| ---------------- | -------------------------------------------------------------------------- |
| `title`          | The window's title                                                         |
| `update`         | Builds the frame's interface with the newest input, and handles that input |
| `prepare`        | Creates the program's GPU resources, once                                  |
| `render`         | Draws the program's 3D views into the frame, under the interface           |
| `may_close`      | Whether the window may close now, or asks first, like about unsaved work   |
| `state`          | What scripts save and check, as a JSON object                              |
| `script`         | Does a script's `Program` step, like connecting a made-up device           |

## The design system

Color with the theme's roles and size with the tokens and the controls' metrics, instead of values
of your own, so that every program looks like one family, and like Windows 11.

- **Themes** (`Design/Theme.rs`): light by default, and dark. The roles are WinUI's theme
  resources, like `text_primary` for TextFillColorPrimary, `control_fill` for
  ControlFillColorDefault, and `card_background` for CardBackgroundFillColorDefault, with Windows
  11's default blue accent and its ramp. Status colors, like `critical`, and colors that tell
  graphs' series apart, `theme.data`, are there too.
- **Tokens** (`Design/Tokens.rs`): Fluent's 4-pixel spacing ramp (`SPACE_XXS` to `SPACE_XXXL`),
  corner radii (4 for controls, 8 for cards, flyouts and dialogs), the control height (32), the type
  ramp (`CAPTION`, `BODY`, `BODY_STRONG`, `BODY_LARGE`, `SUBTITLE`, `TITLE`, `TITLE_LARGE`,
  `DISPLAY`, and `MONO` in Cascadia Mono), motion (83, 167, 250 and 333 ms, and Fluent's easing
  curves), elevation as WinUI's ThemeShadow makes it from a depth (`elevation(z)`, with tooltips at
  16, flyouts at 32, submenus 8 over their menu, and dialogs at 128), and the page's sizes.
- **Controls** (`Design/Controls.rs`): WinUI 3's controls' own metrics, like a button's padding of
  11, 5, 11 and 6, a toggle switch's 40 × 20 track, a slider's thumb, a combo box's chevron column,
  a list item's backplate and selection pill, a menu item's padding, and the focus visual's two
  rings, each from WinUI's styles.
- **Elevation borders**: controls' borders shade from their top to their bottom, as WinUI's
  ControlElevationBorderBrush does, and text fields' bottom edge is strong, and the accent's two
  pixels while they have focus.
- **Type**: Segoe UI Variable, in its optical sizes: Small for captions, Text for body text, and
  Display for titles, laid out by DirectWrite with kerning and font fallback, each line as tall as
  the type ramp says.
- **Icons** (`Design/Icons.rs`): Fluent's system icons, from Segoe Fluent Icons.
- **The page** (`Widgets/Shell.rs`): an app bar along the top with the program's icon, title, and
  its own menus, tabs and commands, notices under it, a status bar along the bottom with shortcuts
  and status, and cards between them, `PAGE_PADDING` from the window's edges and `CARD_GAP` from
  each other.

## Widgets

A widget is a value that the program builds, like `Button::new("Save").accent()`, and adds with
`ui.add(name, rect, widget)`, which returns its response. The name is what it's known by from frame
to frame, and what scripts and tests point at. Each widget is a target, whose area, kind and label
the interface's state lists. A widget of a program's own implements `Widget`.

| Widget                             | WinUI's                   | Notes                                                          |
| ---------------------------------- | ------------------------- | -------------------------------------------------------------- |
| `Button`, `HyperlinkButton`        | Button, HyperlinkButton   | Standard, accent or subtle; text, an icon, or both             |
| `ToggleButton`                     | ToggleButton              | In the accent color while it's on                              |
| `CheckBox`, `RadioButton`          | CheckBox, RadioButton     | A radio button is checked while the value is its own           |
| `ToggleSwitch`                     | ToggleSwitch              |                                                                |
| `Slider`                           | Slider                    | Steps, the wheel, the keys, a header, and the value as dragged |
| `NumberBox`                        | NumberBox                 | Typed, stepped with its buttons, the arrows or the wheel       |
| `ComboBox`                         | ComboBox                  | Its list opens over it, with the chosen item where it is       |
| `TextBox`                          | TextBox                   | Selection, clipboard, a clear button, and a button of its own  |
| `ColorPicker`, `ColorField`        | ColorPicker               | A spectrum with a value slider, and a hex field                |
| `Swatch`                           |                           | A color to pick, with an accent ring while it's picked         |
| `ListView`                         | ListView                  | Rows that scroll and are virtualized, one selected             |
| `SelectorBar`                      | SelectorBar               | Tabs, with a pill under the selected one that slides           |
| `Segmented`                        | Toolkit's Segmented       | Segments of equal width, with text, icons, or both             |
| `MenuBar`, `ui.flyout_menu`        | MenuBar, MenuFlyout       | Icons, accelerators, checks, separators and submenus           |
| `ui.context_menu`                  | MenuFlyout                | Opens at the pointer on a right-click                          |
| `ui.dialog`, `ui.dialog_with`      | ContentDialog             | Over smoke, with up to three buttons and content of your own   |
| `Expander`                         | Expander                  | A header that shows and hides what's under it                  |
| `InfoBar`                          | InfoBar                   | Informational, success, warning or error; closable             |
| `ScrollBar`, `ui.begin_scroll`     | ScrollViewer              | Content taller than its area, with the wheel                   |
| `ui.tooltip`                       | ToolTip                   | After the pointer rests half a second, above it                |
| `ui.progress_bar`, `progress_ring` | ProgressBar, ProgressRing | Determinate, or of unknown length                              |
| `ui.card`, `card_with_view`        | Card                      | A card with a view leaves a hole for the program's 3D          |
| `ui.begin_card`, `end_card`        | Card                      | A card as tall as its content                                  |
| `ui.app_bar`, `status_bar`         |                           | The page's frame, with `ui.page` between them                  |
| `View`                             |                           | A 3D view's area, which takes every button but never focus     |
| `Form`                             |                           | Lays a card's content out top to bottom, with headers          |
| `ui.keys`, `badge`, `divider`      | InfoBadge, and others     | Keycaps of shortcuts, counts on a pill, and lines              |

`ui.set_enabled(false)` disables the widgets after it, and `ui.set_focusable(false)` keeps them
from taking keyboard focus, until each is set back; each frame starts enabled and focusable.
`ui.interact(name, rect, sense, kind)` makes a target of a program's own, with the input on it.
`ui.painter()` draws a program's own graphics, like graphs and previews: rounded rectangles with
borders and gradients, Fluent's shadows, circles, rings, arcs, lines, polygons, text and icons,
with clipping.

## Input

- The pointer is on the topmost target under it, by the last frame's targets, so a combo box's
  list, a menu or a dialog takes the pointer from what's under it. `ui.cover` makes a surface, like
  a card over a 3D view, cover what's under it.
- A click is a press and a release on the same widget. While a button is held, the window captures
  the mouse, so a drag keeps going past its edges.
- Keyboard focus moves with Tab and Shift+Tab, in the order that the frame showed the widgets, and
  with a press on a widget that takes it. Its rings show only after the keyboard moved it, as
  Windows does. Esc closes a menu or a dialog, or else takes focus away. A widget with focus under
  an open menu or dialog gets no keys.
- `ui.key_pressed`, `ui.key_pressed_once` and `ui.shortcut` are for programs' shortcuts. They
  don't see the keys that the widget with focus takes, like a text field's letters or a slider's
  arrows, or any while a dialog is open. `ui.input()` has all of the frame's input, for programs
  that read keys themselves, like for driving.
- Without focus, the window has no input at all, including the gamepad's. The click that gives it
  focus does only that, and keys and gamepad buttons held when it gets focus count once they're let
  go of.
- `F11` switches between maximized and borderless fullscreen.
- With `Options::gamepad`, the library reads the gamepad with GameInput for `ui.gamepad()`.

## Window setups

Programs support exactly two window setups, both on a 2560 × 1440 display at 100% scale, where
logical and physical pixels are the same, with a 48-pixel taskbar at the bottom:

| Setup                 | Client area | What makes it                                             |
| --------------------- | ----------- | --------------------------------------------------------- |
| Maximized             | 2560 × 1369 | The 2560 × 1392 work area, less the title bar's 23 pixels |
| Borderless fullscreen | 2560 × 1440 | A window without a frame that covers the display          |

In any other setup, the window says what's wrong instead of showing the program. It checks again
whenever the window moves or changes size, or the displays, the scale or the taskbar change. Lay out
cards in fixed pixels for these two client areas; they differ by 71 pixels of height.

## Settings and the command line

`Settings.json`, in the program's folder, has the settings every program has, then the program's
own, which it reads from `startup.settings` with their defaults. A missing file is written with every
default; a wrong setting keeps its default, and a notice under the app bar says what's wrong.

| Setting        | Value                                                                  |
| -------------- | ---------------------------------------------------------------------- |
| `Display`      | `"Primary"`, or the identifier of the display to open on               |
| `WindowMode`   | `"Maximized"` or `"BorderlessFullscreen"`                              |
| `AllowTearing` | Whether frames may tear, which lets them reach the screen mid-refresh  |
| `VerticalSync` | Whether frames wait for the screen's refresh, which also stops tearing |
| `FrameLimit`   | The most frames a second, or 0 for no limit                            |

| Argument              | What it does                                                             |
| --------------------- | ------------------------------------------------------------------------ |
| `--display <id>`      | Opens on the display, which must be connected. `Primary` for the primary |
| `--no-activate`       | Opens beneath the other windows, without taking focus                    |
| `--vsync`             | Waits for vertical sync, as `VerticalSync` does                          |
| `--frame-limit <fps>` | Starts no more frames a second than this, sleeping between them          |
| `--script <path>`     | Runs a script instead of the user's input, with `--output <folder>`      |

## Input latency

The main loop (`Application/Run.rs`) handles the window's messages without waiting, and waits until
the GPU has finished the last frame and DXGI can take another. Only then does it read the input,
build the frame's interface, draw it and present it, without waiting for vertical sync, so a
frame's input is as fresh as it can be. The swap chain has a frame latency waitable object and a
maximum frame latency of 1, and draws on the GPU that the window's monitor is an output of. The
whole interface is one draw call of instanced quads, whose pixel shader computes each shape's
coverage from its signed distance. Text layouts are cached, so a frame that shows the same text as
the last one asks DirectWrite for nothing, and glyphs are rasterized once each into an atlas.
Minimizing the window stops the loop drawing.

With vertical sync or a frame limit, as coding agents' test runs have, the loop sleeps while it
waits instead of spinning a core, so that it leaves the machine to whoever's using it.

## Testing

**Headless.** `Headless` runs a program's frames without a window or a GPU, with input that the test
makes up, on a fixed clock, through the same interface as a real run, so presses land on the cards'
real layout. Each input method runs a frame with the input and a frame after it:

```rust
let mut app = Headless::new(Volume { level: 50.0 });
app.click("Mute");
assert_eq!(app.program.level, 0.0);
app.key(Key::F11);
assert!(app.fullscreen());
```

`render` draws the last frame offscreen with the GPU, with the program's 3D, for tests that check
pixels.

**Scripts.** A program run with `--script` takes its input from the script, on a fixed clock,
instead of from the user, and saves frames and its state. RunInBackground runs them (see its
[README](../../Program/RunInBackground/README.md#scripts)). A script is a JSON object with its
`Steps`, each at a `Frame`. Frame 0 gives the window focus and draws what frame 1's steps point at.

```json
{
    "Steps": [
        { "Frame": 1, "Click": "Mute" },
        { "Frame": 3, "Drag": { "From": "Level", "To": [400, 300], "Frames": 10 } },
        { "Frame": 20, "Key": "Ctrl+S" },
        { "Frame": 22, "Capture": "Saved" },
        { "Frame": 22, "Expect": { "Level": 100, "Interface": { "Popup": null } } }
    ]
}
```

| Step          | Value                                                                            |
| ------------- | -------------------------------------------------------------------------------- |
| `Move`        | A point: a target's name, which is its middle, `[x, y]` in the window, or        |
|               | `{ "Target": name, "At": [x, y] }`, a place in a target as fractions of its size |
| `Press`       | A point, or `{ "At": point, "Button": "Right" }`                                 |
| `Release`     | `true` for the left button, or a button's name                                   |
| `Click`       | A point: a press, released the next frame                                        |
| `DoubleClick` | A point: two clicks, a frame apart                                               |
| `Drag`        | `{ "From", "To", "Frames", "Button" }`: a press, a move a frame, and a release   |
| `Scroll`      | `{ "At": point, "Notches": n }`, up for more than 0                              |
| `Key`         | A key, like `"F11"` or `"Ctrl+S"`: pressed, and let go of the next frame         |
| `Hold`        | The keys to hold from now on, like `["W", "Shift"]`; `[]` lets go of them        |
| `Type`        | Text, as if typed                                                                |
| `Gamepad`     | The gamepad from now on: `LeftStick`, `RightStick`, `LeftTrigger`,               |
|               | `RightTrigger`, and `Buttons`                                                    |
| `Focus`       | Whether the window has focus                                                     |
| `Capture`     | A name: saves the frame as `<name>.png` and the state as `<name>.json`           |
| `Expect`      | What the state must have: every member it names must match, numbers to a         |
|               | millionth                                                                        |
| `Program`     | A step for the program's `script`                                                |

The state is the program's, and the interface's as `Interface`: the `Theme`, `Size`, `Fullscreen`,
`WindowFocused`, the `Pointer`, what's `Hovered`, `Pressed` and `Focused`, the open `Popup` and
`Dialog`, the `Tooltip` that shows, and `Targets`: every widget's `Area`, `Kind`, `Enabled` and
`Label`, by name. Menu items are named after their menu and label, like `Menu/File/Menu/Save`, and
a dialog's buttons like `Question/Primary`. Scripts copy and paste on the interface's own
clipboard, never Windows'. A scripted run reads and writes its files in its output folder, through
`startup.data_folder`.

`Test/UserInterface` tests the library itself, with a GPU test that draws offscreen.

## Layout

| Path                             | Contents                                                                    |
| -------------------------------- | --------------------------------------------------------------------------- |
| `Source/Library.rs`              | The library's modules and what it exports                                   |
| `Source/Application/Program.rs`  | `Program`, `Options`, `Startup`, and the command line                       |
| `Source/Application/Run.rs`      | The main loop                                                               |
| `Source/Application/Settings.rs` | `Settings.json`                                                             |
| `Source/Application/Overlays.rs` | Frameless windows over each display, like IdentifyDisplays' boxes           |
| `Source/Core/Ui.rs`              | A frame's interface: targets, presses, focus, popups, tooltips, motion      |
| `Source/Platform/Window.rs`      | The Win32 window, its input, its two setups, and their check                |
| `Source/Platform/Gamepad.rs`     | Reads the gamepad with GameInput                                            |
| `Source/Platform/Clipboard.rs`   | Windows' clipboard, and the interface's own for scripts and tests           |
| `Source/Input/Devices.rs`        | Keys, shortcuts, mouse buttons and the gamepad                              |
| `Source/Input/Events.rs`         | A frame's input, from the window's events                                   |
| `Source/Design/Theme.rs`         | WinUI's theme resources, light and dark                                     |
| `Source/Design/Tokens.rs`        | Fluent's spacing, radii, type ramp, motion, elevation, and the page's sizes |
| `Source/Design/Controls.rs`      | WinUI 3's controls' sizes, paddings and parts                               |
| `Source/Design/Icons.rs`         | Segoe Fluent Icons                                                          |
| `Source/Drawing/Geometry.rs`     | Points, sizes, insets, rectangles and colors                                |
| `Source/Drawing/Text.rs`         | DirectWrite: fonts, cached layouts, hit testing, and the glyph atlas        |
| `Source/Drawing/Painter.rs`      | The draw list, and the Painter that programs draw their own graphics with   |
| `Source/Drawing/Renderer.rs`     | Draws the draw list with Direct3D 12, in one draw call                      |
| `Shaders/Interface.hlsl`         | The shapes' and glyphs' shader                                              |
| `Source/Widgets/*.rs`            | The widgets, cards, menus, dialogs, and the page's bars                     |
| `Source/Testing/Script.rs`       | Scripts                                                                     |
| `Source/Testing/Headless.rs`     | Runs a program's frames without a window, for tests                         |
| `Build.rs`                       | Compiles the shader at build time                                           |
