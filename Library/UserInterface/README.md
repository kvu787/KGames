# UserInterface

KGames' user interface library, for Win32, Direct3D 12 and DirectWrite, after Microsoft's
[Fluent 2](https://fluent2.microsoft.design) design system as WinUI 3 implements it on Windows 11.
Every GUI program is built on it: it owns the window, reads the input, runs the main loop, lays out
text, draws the widgets on the GPU, and runs scripts and headless tests.
[UiShowcase](../../Program/UiShowcase/README.md) shows every widget.

It's immediate mode: each frame, a program's `update` lays out its cards in fixed pixels and calls
a method of `Ui` for each control, which draws it and says what the input did to it. State that
the program owns, like a slider's value, is the program's, passed as `&mut`. State that only the
widget needs, like a text field's caret or a list's scroll, the library keeps by the widget's name.

## A program

```rust
use user_interface::{Button, Icon, Options, Program, Slider, Startup, Status, Ui};

struct Volume {
    level: f32,
}

impl Program for Volume {
    fn title(&self) -> String {
        "Volume".to_string()
    }

    fn update(&mut self, ui: &mut Ui) {
        ui.app_bar(Icon::Volume, "Volume");
        ui.status_bar(&[Status::Shortcut("F11", "Fullscreen")]);
        let page = ui.page();
        let card = ui.card(page.left_strip(400.0).top_strip(160.0), "Output");
        let mut area = card.content;
        ui.slider("Level", area.cut_top(32.0, 8.0), &mut self.level, Slider::new(0.0, 100.0).step(1.0));
        if ui.button("Mute", area.cut_top(32.0, 0.0).left_strip(120.0), Button::new("Mute")).clicked {
            self.level = 0.0;
        }
    }
}

fn main() -> std::process::ExitCode {
    user_interface::run(Options::new("Volume", env!("CARGO_MANIFEST_DIR")), |startup: &mut Startup| {
        Ok(Volume { level: 50.0 })
    })
}
```

`run` reads `Settings.json` and the command line, opens the window on the chosen display, creates
the program, and runs the main loop until the window closes. A program can also draw 3D: its
`prepare` creates GPU resources once, and its `render` records commands into each frame, after
the frame is cleared to the theme's background and before the interface draws over it.

| `Program` method | What it's for                                                              |
| ---------------- | -------------------------------------------------------------------------- |
| `title`          | The window's title                                                         |
| `update`         | Builds the frame's interface with the newest input, and handles that input |
| `prepare`        | Creates the program's GPU resources, once                                  |
| `render`         | Draws the program's 3D views into the frame, under the interface           |
| `state`          | What scripts save and check, as a JSON object                              |
| `script`         | Does a script's `Program` step, like connecting a made-up device           |

## The design system

Color with the theme's roles and size with the tokens, instead of values of your own, so that every
program looks like one family, and like Windows 11.

- **Themes** (`Theme.rs`): light by default, and dark. The roles are WinUI's theme resources, like
  `text_primary` for TextFillColorPrimary, `control_fill` for ControlFillColorDefault, and
  `card_background` for CardBackgroundFillColorDefault, with Windows' default blue accent. Status
  colors, like `critical`, and colors that tell graphs' series apart, `theme.data`, are there too.
- **Tokens** (`Tokens.rs`): Fluent's 4-pixel spacing ramp (`SPACE_XS` to `SPACE_XXXL`), corner radii
  (4 for controls, 8 for cards and flyouts), control heights (32), the type ramp (`CAPTION`, `BODY`,
  `BODY_STRONG`, `BODY_LARGE`, `SUBTITLE`, `TITLE`, `TITLE_LARGE`, `DISPLAY`, and `MONO` in Cascadia
  Mono), motion (83, 167 and 250 ms, and Fluent's easing curves), and elevation (the two-shadow levels
  2, 4, 8, 16, 28 and 64).
- **Type**: Segoe UI Variable, in its optical sizes: Small for captions, Text for body text, and
  Display for titles, laid out by DirectWrite with kerning and font fallback, each line as tall as
  the type ramp says, with the font centered in it.
- **Icons** (`Icons.rs`): Fluent's system icons, from Segoe Fluent Icons.
- **The page** (`Shell.rs`): an app bar along the top with the program's icon and title and its own
  tabs and commands, notices under it, a status bar along the bottom with shortcuts and status,
  and cards between them, `PAGE_PADDING` from the window's edges and `CARD_GAP` from each other.

## Widgets

Each widget takes a name, which is what it's known by from frame to frame, and what scripts and
tests point at, and the rectangle it fills. Each is a target, whose area the interface state lists.

| Method                                                   | WinUI's                   | Notes                                                                               |
| -------------------------------------------------------- | ------------------------- | ----------------------------------------------------------------------------------- |
| `button`                                                 | Button                    | Standard, accent or subtle; text, an icon, or both                                  |
| `toggle_button`                                          | ToggleButton              | In the accent color while it's on                                                   |
| `hyperlink`                                              | HyperlinkButton           |                                                                                     |
| `checkbox`                                               | CheckBox                  |                                                                                     |
| `radio`                                                  | RadioButton               | One of a group: checked while the value is its own                                  |
| `toggle_switch`                                          | ToggleSwitch              |                                                                                     |
| `slider`                                                 | Slider                    | Steps, the wheel, the keys, and the value above it as dragged                       |
| `progress_bar`, `progress_ring`                          | ProgressBar, ProgressRing | Determinate, or of unknown length                                                   |
| `combo_box`                                              | ComboBox                  | Its list opens over it, with the chosen item where it is                            |
| `text_box`                                               | TextBox                   | Selection, copy, cut, paste, a clear button, and a button of its own, like search's |
| `list_view`                                              | ListView                  | Rows that scroll and are virtualized, one selected                                  |
| `selector_bar`                                           | SelectorBar               | Tabs, with a pill under the selected one that slides                                |
| `segmented`, `segmented_icons`                           | Toolkit's Segmented       | Segments of equal width                                                             |
| `info_bar`                                               | InfoBar                   | Informational, success, warning or error; closable                                  |
| `tooltip`, `tooltip_lines`                               | ToolTip                   | After the pointer rests half a second; above it if there's no room under it         |
| `card`, `floating_card`, `card_with_view`                | Card                      | A floating card is opaque and raised, for over a 3D view; a card with a view has a  |
|                                                          |                           | hole for a 3D view that the program's `render` draws                                |
| `keycap`, `badge`, `swatch`, `divider`                   | InfoBadge, and others     |                                                                                     |
| `app_bar`, `page`, `status_bar`, `notify`                |                           | The page's frame                                                                    |
| `text`, `label`, `caption`, `paragraph`, `field`, `icon` | TextBlock                 |                                                                                     |

`ui.set_enabled(false)` disables the widgets after it, and `ui.set_focusable(false)` keeps them
from taking keyboard focus, until each is set back; each frame starts enabled and focusable.
`ui.painter()` draws a program's own graphics, like graphs and previews: rounded rectangles with
strokes, Fluent's elevation borders and shadows, circles, rings, arcs, lines, convex polygons,
gradients, text and icons, with clipping and layers.

## Input

All input comes from [GameInput](../GameInput/README.md): keyboards, mice and gamepads.
`Devices.rs` reads every device's readings once a frame, right before the frame, and
`InputReader.rs` turns them into the interface's events, in the order they happened, with the
window's focus changes between them. The window's messages bring none of it.

- Keys are the keys that keyboards hold down. Shift, Ctrl and Alt are the same keys on the left
  and the right. Typed text is the keys translated with Windows' keyboard layout, with Shift, AltGr
  and dead keys, but no IME. A held key repeats at Windows' keyboard delay and rate.
- The mouse's buttons and wheels come from mice. The pointer is where Windows' cursor is, so it
  moves at Windows' pointer speed, and a press while the cursor isn't on the window is another
  window's.
- The pointer is on the topmost target under it, by the last frame's targets, so a combo box's
  list or a tooltip takes the pointer from what's under it. `ui.cover` makes a surface, like a card
  over a 3D view, cover what's under it.
- A click is a press and a release on the same widget. While a button is held, the window captures
  the mouse, so a drag keeps going past its edges.
- Keyboard focus moves with Tab and Shift+Tab, in the order that the frame showed the widgets, and
  with a press on a widget that takes it. Its ring shows only after the keyboard moved it, as
  Windows does. Esc closes a popup, or else takes focus away.
- `ui.key_pressed` and `ui.shortcut` are for programs' shortcuts. They don't see the keys that the
  widget with focus takes, like a text field's letters or a slider's arrows. `ui.input()` has all
  of the frame's input, for programs that read keys themselves, like for driving.
- Without focus, the window has no input at all, including the gamepad's. The click that gives it
  focus does only that, and keys and gamepad buttons held when it gets focus count once they're let
  go of.
- `F11` switches between maximized and borderless fullscreen.
- With `Options::gamepad`, the newest reading of any gamepad goes to `ui.gamepad()`.
- Programs that show the devices themselves, or run gamepads' motors, like GameInputDemo, read the
  frame's readings and device events from `startup.devices`, which the interface reads them with.

## Window setups

Programs support exactly two window setups, both on a 2560 × 1440 display at 100% scale, where
logical and physical pixels are the same, with a 48-pixel taskbar at the bottom:

| Setup                 | Client area | What makes it                                             |
| --------------------- | ----------- | --------------------------------------------------------- |
| Maximized             | 2560 × 1369 | The 2560 × 1392 work area, less the title bar's 23 pixels |
| Borderless fullscreen | 2560 × 1440 | A window without a frame that covers the display          |

In any other setup, the window says what's wrong instead of showing the program. It checks again
whenever the window moves or changes size, or the displays, the scale or the taskbar change.

The top `STATS_BAR_HEIGHT`, 30 pixels, of every program's client area is the library's: its
[performance stats](#performance-stats). The program has the rest, `ui.bounds()`, which starts at
`ui.content_top()`, with the app bar at its top. Lay out cards in fixed pixels for the two client
areas, which leaves the program 2560 × 1339 maximized and 2560 × 1410 in borderless fullscreen;
they differ by 71 pixels of height. A 3D view may still fill the window, under the stats.

## Settings and the command line

`Settings.json`, in the program's folder, has the settings every program has, then the program's
own, which it reads from `startup.settings` with their defaults. A missing file is written with every
default; a wrong setting keeps its default, and a notice under the app bar says what's wrong.

| Setting        | Value                                                                 |
| -------------- | --------------------------------------------------------------------- |
| `Display`      | `"Primary"`, or the identifier of the display to open on              |
| `WindowMode`   | `"Maximized"` or `"BorderlessFullscreen"`                             |
| `AllowTearing` | Whether frames may tear, which lets them reach the screen mid-refresh |

| Argument                 | What it does                                                             |
| ------------------------ | ------------------------------------------------------------------------ |
| `--display <id>`         | Opens on the display, which must be connected. `Primary` for the primary |
| `--no-activate`          | Opens without taking focus                                               |
| `--no-input`             | Doesn't start GameInput, so the program reads no input at all            |
| `--vsync`                | Frames wait for vertical sync, as RunInBackground's test runs do         |
| `--frame-rate-limit <n>` | At most n frames a second, which wait between them instead of spinning   |
| `--script <path>`        | Runs a script instead of the user's input, with `--output <folder>`      |

## Input latency

The main loop (`Run.rs`) handles the window's messages without waiting, and spins until the GPU
has finished the last frame and DXGI can take another. Only then does it read the input, build the
frame's interface, draw it and present it, without waiting for vertical sync, so a frame's input
is as fresh as it can be. RunInBackground's test runs use `--vsync`, `--frame-rate-limit 100` and
`--no-input` instead, so that they share the computer lightly with someone using it, like playing
a game: frames wait for vertical sync, at most 100 a second, the loop waits on a timer and on DXGI
between them, instead of spinning, and GameInput, with its helper process that reads keyboards and
mice, doesn't start. The swap chain has a frame latency waitable object and a maximum frame
latency of 1, and draws on the GPU that the window's monitor is an output of. The whole interface
is one draw call of instanced quads, whose pixel shader computes each shape's coverage from its
signed distance. Text layouts are cached, so a frame that shows the same text as the last one asks
DirectWrite for nothing, and glyphs are rasterized once each into an atlas. The process opts out of
power throttling, so frames are paced the same in the background. Minimizing the window stops the
loop drawing.

## Performance stats

Every program's window shows its performance along its top (`Performance.rs`, drawn by `Shell.rs`),
over the last 2 seconds, and changes what it says 4 times a second:

| Stat              | What it is                                                                                                    |
| ----------------- | ------------------------------------------------------------------------------------------------------------- |
| `Frames`          | Frames a second that the main loop draws, the mean time between them, and the longest                         |
| `On screen`       | Frames a second that reach the screen                                                                         |
| `Input to screen` | The input latency's median and 99th percentile, and how many frames they're of                                |
| `Medians`         | The median of each of the input latency's three parts                                                         |
| At the right      | How frames reach the screen: an independent flip, an overlay plane, or composed by DWM, and whether they tear |

The input latency runs from when GameInput got the input that a frame took to when the display took
that frame, all on the performance counter's clock, in three parts:

```text
input --- read ----------- present ------------- screen
   waiting       building           presenting
```

| Time      | What it is                                                                             |
| --------- | -------------------------------------------------------------------------------------- |
| `input`   | The newest GameInput reading that the frame took: when Windows got the device's report |
| `read`    | When the main loop read the frame's input                                              |
| `present` | Right before the frame's `Present` call                                                |
| `screen`  | When the display took the frame, from Windows' own events about presents               |

The screen time comes from [Library/PresentTiming](../PresentTiming/README.md), which reads it as
PresentMon does, so the input latency is comparable with PresentMon's: it's the counterpart of its
`MsAllInputToPhotonLatency` for every device that GameInput reads, gamepads too, and `presenting`
is its `MsUntilDisplayed`. Like PresentMon's, a frame's input is the newest that it took, and a
frame that never reaches the screen, like one that a newer one replaced, passes its input on to the
next frame that does, unless that one has input of its own. Readings of the gamepad count only in
programs whose `Options::gamepad` is on.

What it leaves out is what software can't see: the time before Windows gets the device's report,
in the device and on its way, and after the display takes the frame, as scanout reaches the part of
the screen you look at, and the display's own processing and response. With tearing, the display
takes a frame mid-refresh, where it shows from the tear down, so the screen time is when the frame
starts to show anywhere.

Reading Windows' events about presents needs an elevated administrator, or the
**Performance Log Users** group (see [Library/PresentTiming](../PresentTiming/README.md#privileges)).
Without either, the stats say that input latency needs the group. A scripted run, or a test, has
no input latency.

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

`render` draws the last frame offscreen with the GPU, for tests that check pixels.

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
`WindowFocused`, the `Pointer`, what's `Hovered`, `Pressed` and `Focused`, the open `Popup`, the
`Tooltip` that shows, and `Targets`: every widget's `Area`, `Kind`, `Enabled` and `Label`, by name.
Scripts copy and paste on the interface's own clipboard, never Windows'. A scripted run reads and
writes its files in its output folder, through `startup.data_folder`.

`Test/UserInterface` tests the library itself, with a GPU test that draws offscreen.

## Layout

| Path                     | Contents                                                                  |
| ------------------------ | ------------------------------------------------------------------------- |
| `Source/Library.rs`      | The library's modules and what it exports                                 |
| `Source/Program.rs`      | `Program`, `Options`, `Startup`, and the command line                     |
| `Source/Run.rs`          | The main loop                                                             |
| `Source/Ui.rs`           | A frame's interface: targets, presses, focus, popups, tooltips, motion    |
| `Source/Window.rs`       | The Win32 window, its focus, its cursor, its two setups, and their check  |
| `Source/Input.rs`        | Keys, buttons, the gamepad, and a frame's input from events               |
| `Source/Devices.rs`      | Reads gamepads, keyboards and mice with GameInput, once a frame           |
| `Source/InputReader.rs`  | Turns a frame's GameInput readings into events                            |
| `Source/Performance.rs`  | The performance stats, and the input latency from GameInput to the screen |
| `Source/Clock.rs`        | The performance counter, which input and frames are timed on              |
| `Source/Clipboard.rs`    | Windows' clipboard, and the interface's own for scripts and tests         |
| `Source/Settings.rs`     | `Settings.json`                                                           |
| `Source/Theme.rs`        | Fluent's color roles, light and dark                                      |
| `Source/Tokens.rs`       | Fluent's spacing, sizes, radii, type ramp, motion and elevation           |
| `Source/Icons.rs`        | Segoe Fluent Icons                                                        |
| `Source/Geometry.rs`     | Points, sizes, rectangles and colors                                      |
| `Source/Text.rs`         | DirectWrite: fonts, cached layouts, hit testing, and the glyph atlas      |
| `Source/Painter.rs`      | The draw list, and the Painter that programs draw their own graphics with |
| `Source/Renderer.rs`     | Draws the draw list with Direct3D 12, in one draw call                    |
| `Shaders/Interface.hlsl` | The shapes' and glyphs' shader                                            |
| `Source/Widgets/*.rs`    | The widgets, and the page's app bar, notices and status bar               |
| `Source/Script.rs`       | Scripts                                                                   |
| `Source/Headless.rs`     | Runs a program's frames without a window, for tests                       |
| `Build.rs`               | Compiles the shader at build time                                         |
