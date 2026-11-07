# UserInterface

KGames' user interface library, for Win32, Direct3D 12 and DirectWrite, after Microsoft's
[Fluent 2](https://fluent2.microsoft.design) design system as WinUI 3 implements it on Windows 11,
with WinUI 3's own values: its theme resources' colors, its controls' sizes, paddings and parts,
and its elevation borders and shadows, but not its animations. Every GUI program is built on it: it
owns the window, reads the input, runs the main loop, lays out text, draws the widgets on the GPU,
and runs scripts and headless tests. [UiShowcase](../../Program/UiShowcase/README.md) shows every
widget.

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

- **Themes** (`Design/Theme.rs`): light by default, and dark, which `F9` switches to and from in
  every program. The roles are WinUI's theme resources, like `text_primary` for
  TextFillColorPrimary, `control_fill` for ControlFillColorDefault, and `card_background` for CardBackgroundFillColorDefault, with Windows
  11's default blue accent and its ramp. Status colors, like `critical`, and colors that tell
  graphs' series apart, `theme.data`, are there too.
- **Tokens** (`Design/Tokens.rs`): Fluent's 4-pixel spacing ramp (`SPACE_XXS` to `SPACE_XXXL`),
  corner radii (4 for controls, 8 for cards, flyouts and dialogs), the control height (32), the type
  ramp (`CAPTION`, `BODY`, `BODY_STRONG`, `BODY_LARGE`, `SUBTITLE`, `TITLE`, `TITLE_LARGE`,
  `DISPLAY`, and `MONO` and `MONO_CAPTION` in Cascadia Mono), elevation as WinUI's ThemeShadow makes
  it from a depth (`elevation(z)`, with tooltips at 16, flyouts at 32, submenus 8 over their menu,
  and dialogs at 128), and the page's sizes.
- **Controls** (`Design/Controls.rs`): WinUI 3's controls' own metrics, like a button's padding of
  11, 5, 11 and 6, a toggle switch's 40 × 20 track, a slider's thumb, a combo box's chevron column,
  a list item's backplate and selection pill, a menu item's padding, and the focus visual's two
  rings, each from WinUI's styles.
- **Elevation borders**: controls' borders shade from their top to their bottom, as WinUI's
  ControlElevationBorderBrush does, and text fields' bottom edge is strong, and the accent's two
  pixels while they have focus.
- **Motion**: none between states. A hover, a press, a selection, a switch that's flipped, and a
  menu, submenu or dialog that opens or closes all show whole on the frame that makes them, without
  WinUI's fades, slides and eases. Only what has to move to do its job moves: the indeterminate
  progress bar and ring, whose motion says that the work goes on and the program isn't frozen, and
  a text field's caret, which blinks. Widgets of a program's own follow the same rule.
- **Type**: Segoe UI Variable, in its optical sizes: Small for captions, Text for body text, and
  Display for titles, laid out by DirectWrite with kerning and font fallback, each line as tall as
  the type ramp says.
- **Icons** (`Design/Icons.rs`): Fluent's system icons, from Segoe Fluent Icons.
- **The page** (`Widgets/Shell.rs`): an app bar along the top with the program's icon, title, and
  its own menus, tabs and commands, a menu row under it for a menu bar whose menus' names line up
  with the page's left edge, if the program wants one, notices under them, a status bar along the
  bottom with shortcuts and status, and cards between them, `PAGE_PADDING` from the window's edges
  and `CARD_GAP` from each other.

## Widgets

A widget is a value that the program builds, like `Button::new("Save").accent()`, and adds with
`ui.add(name, rect, widget)`, which returns its response. The name is what it's known by from frame
to frame, and what scripts and tests point at. Each widget is a target, whose area, kind and label
the interface's state lists. A widget of a program's own implements `Widget`.

| Widget                             | WinUI's                   | Notes                                                           |
| ---------------------------------- | ------------------------- | --------------------------------------------------------------- |
| `Button`, `HyperlinkButton`        | Button, HyperlinkButton   | Standard, accent or subtle; text, an icon, or both              |
| `ToggleButton`                     | ToggleButton              | In the accent color while it's on                               |
| `CheckBox`, `RadioButton`          | CheckBox, RadioButton     | A radio button is checked while the value is its own            |
| `RadioGroup`                       | RadioButtons              | A column of radios that picks one of its items, by index        |
| `ToggleSwitch`                     | ToggleSwitch              |                                                                 |
| `Slider`                           | Slider                    | Steps, the wheel, the keys, a header, and the value as dragged  |
| `NumberBox`                        | NumberBox                 | Typed, stepped with its buttons, the arrows or the wheel        |
| `ComboBox`                         | ComboBox                  | Its list opens over it, with the chosen item where it is        |
| `TextBox`                          | TextBox                   | Selection, clipboard, a clear button, and a button of its own   |
| `ColorPicker`, `ColorField`        | ColorPicker               | A spectrum with a value slider, and a hex field                 |
| `Swatch`                           |                           | A color to pick, with an accent ring while it's picked          |
| `ListView`                         | ListView                  | Rows that scroll and are virtualized, one selected              |
| `SelectorBar`                      | SelectorBar               | Tabs, with a pill under the selected one                        |
| `Segmented`                        | Toolkit's Segmented       | Segments of equal width, with text, icons, or both              |
| `MenuBar`, `ui.flyout_menu`        | MenuBar, MenuFlyout       | Icons, accelerators, checks, separators and submenus            |
| `ui.context_menu`                  | MenuFlyout                | Opens at the pointer on a right-click                           |
| `ui.dialog`, `ui.dialog_with`      | ContentDialog             | Over smoke, with up to three buttons and content of your own    |
| `Expander`                         | Expander                  | A header that shows and hides what's under it                   |
| `InfoBar`                          | InfoBar                   | Informational, success, warning or error; closable              |
| `ScrollBar`, `ui.begin_scroll`     | ScrollViewer              | Content taller than its area, with the wheel                    |
| `ui.tooltip`, `tooltip_for_wide`   | ToolTip                   | After the pointer rests half a second, above it; wider if asked |
| `ui.progress_bar`, `progress_ring` | ProgressBar, ProgressRing | Determinate, or of unknown length                               |
| `ui.card`, `card_with_view`        | Card                      | A card with a view leaves a hole for the program's 3D           |
| `ui.begin_card`, `end_card`        | Card                      | A card as tall as its content                                   |
| `ui.app_bar`, `status_bar`         |                           | The page's frame, with `ui.page` between them                   |
| `ui.menu_row`                      |                           | A row under the app bar, for a menu bar in line with the page   |
| `View`                             |                           | A 3D view's area, which takes every button but never focus      |
| `ui.info_card`                     |                           | A test program's floating card: frame rate, GPU, display, keys  |
| `Form`                             |                           | Lays a card's content out top to bottom, with headers           |
| `ui.keys`, `badge`, `divider`      | InfoBadge, and others     | Keycaps of shortcuts, counts on a pill, and lines               |

`ui.add_enabled(enabled, name, rect, widget)` shows one widget enabled or not, and
`ui.with_enabled(enabled, |ui| ...)` a run of them; both put the scope back after, and inside a
disabled scope everything stays disabled, so a disabled card can't have a working widget.
`ui.with_focusable(focusable, |ui| ...)` scopes which widgets take keyboard focus the same way,
except that it overrides the scope around it, so a program that keeps focus off everything, like a
game, opts its text fields back in with it. `ui.set_enabled` and `ui.set_focusable` set each until
they're set back; each frame starts enabled and focusable.
`ui.interact(name, rect, sense, kind)` makes a target of a program's own, with the input on it. A
widget that takes the wheel says so in its `Sense` - `takes_wheel()`, like a slider's, or
`takes_wheel_while_focused()`, like a number box's - so what it's in doesn't scroll under it;
`Sense::VIEW` takes it already.
`ui.painter()` draws a program's own graphics, like graphs and previews: rounded rectangles with
borders and gradients, Fluent's shadows, circles, rings, arcs, lines, polygons, text and icons,
with clipping.

## Input

All input comes from [GameInput](../GameInput/README.md): keyboards, mice and gamepads.
`Platform/InputDevices.rs` reads every device's readings once a frame, right before the frame,
and `Input/Reader.rs` turns them into the interface's events, in the order they happened, with
the window's focus changes between them. The window's messages bring none of it.

- Keys are the keys that keyboards hold down. Shift, Ctrl and Alt are the same keys on the left
  and the right. Typed text is the keys translated with Windows' keyboard layout, with Shift, AltGr
  and dead keys, but no IME. A held key repeats at Windows' keyboard delay and rate.
- The mouse's buttons and wheels come from mice. The pointer is where Windows' cursor is, so it
  moves at Windows' pointer speed, and a press while the cursor isn't on the window is another
  window's.
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
  arrows, or any while a dialog is open - from the very frame it opens, for checks after it.
  `ui.input()` has all of the frame's input, for programs that read keys themselves, like for
  driving.
- Without focus, the window has no input at all, including the gamepad's. The click that gives it
  focus does only that, and keys and gamepad buttons held when it gets focus count once they're let
  go of.
- `F9` switches between the light and dark themes. Every program starts light. The frame that `F9`
  is pressed in is all in the new theme, title bar too. A program's own command for it calls
  `ui.toggle_theme()`, which switches it from the next frame on, so that no frame mixes the two,
  and `ui.theme().kind` says which it is. Every widget, card and color role follows by itself, as
  long as the program colors with `ui.theme()` each frame.
- `F10` switches vertical sync on or off, whatever `VerticalSync`, `--vsync` or `--no-vsync`
  started it as, and the [PerfBar](#perfbar) says which. A program's own command for it calls
  `ui.toggle_vertical_sync()`, and `ui.vertical_sync()` says whether it's on.
- `F11` switches between maximized and borderless fullscreen. A program's own command for it calls
  `ui.toggle_fullscreen()`.
- `F9`, `F10` and `F11` are the library's in every program, even while a text field has focus or a
  dialog is open, so a program mustn't also use them for something else.
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

The top `PERF_BAR_HEIGHT`, 17 pixels, of every program's client area is the library's: its
[PerfBar](#perfbar). The program has the rest, `ui.bounds()`, which starts at
`ui.content_top()`, with the app bar at its top. Lay out cards in fixed pixels for the two client
areas, which leaves the program 2560 × 1352 maximized and 2560 × 1423 in borderless fullscreen;
they differ by 71 pixels of height. A 3D view may still fill the window, under the PerfBar.

## Settings and the command line

`Settings.json`, in the program's folder, has the settings every program has, then the program's
own, which it reads from `startup.settings` with their defaults. A missing file is written with every
default; a wrong setting keeps its default, and a notice under the app bar says what's wrong.

| Setting        | Value                                                                    |
| -------------- | ------------------------------------------------------------------------ |
| `Display`      | `"Primary"`, or the identifier of the display to open on                 |
| `WindowMode`   | `"Maximized"`, by default, or `"BorderlessFullscreen"`                   |
| `AllowTearing` | Whether frames may tear, which lets them reach the screen mid-refresh    |
| `VerticalSync` | Whether frames wait for the screen's refresh, which also stops tearing.  |
|                | `true` by default                                                        |

| Argument          | What it does                                                              |
| ----------------- | ------------------------------------------------------------------------- |
| `--display <id>`  | Opens on the display, which must be connected. `Primary` for the primary  |
| `--no-activate`   | Opens beneath the other windows, without taking focus, in efficiency mode |
| `--no-input`      | Doesn't start GameInput, so the program reads no input at all             |
| `--vsync`         | Waits for vertical sync, whatever `VerticalSync` says                     |
| `--no-vsync`      | Doesn't wait for vertical sync, whatever `VerticalSync` says              |
| `--script <path>` | Runs a script instead of the user's input, with `--output <folder>`       |

## Input latency

The main loop (`Application/Run.rs`) handles the window's messages without waiting, and waits until
the GPU has finished the last frame and DXGI can take another. Only then does it read the input,
build the frame's interface, draw it and present it, so a frame's input is as fresh as it can be.
The swap chain has a frame latency waitable object and a maximum frame latency of 1, and draws on
the GPU that the window's monitor is an output of. The whole interface is one draw call of
instanced quads, whose pixel shader computes each shape's coverage from its signed distance. Text
layouts are cached, so a frame that shows the same text as the last one asks DirectWrite for
nothing, and glyphs are rasterized once each into an atlas. Minimizing the window stops the loop
drawing.

Programs start with vertical sync on, unless `VerticalSync` or `--no-vsync` says otherwise: frames
wait for the screen's refresh, and DXGI can take another only once the last one has gone to the
screen, so the loop waits for that, and for messages, instead of spinning: a frame couldn't begin
any sooner, and the input is still read as soon as one can. `F10` switches it on and off while the
program runs. With it off, frames present as soon as they're drawn, and may tear, and the loop
spins, so that the input that reaches the screen is as fresh as it can be.

RunInBackground's test runs use `--no-activate`, `--vsync` and `--no-input`, so that they share the
computer lightly with someone using it, like playing a game: the window opens beneath the others
and the process asks Windows for efficiency mode, where a window that someone uses asks for full
execution speed; frames wait for vertical sync, whatever the program's settings say, at most one a
refresh, and the loop waits on DXGI between them, instead of spinning; GameInput, with its helper
process that reads keyboards and mice, doesn't start; and without input there's no input latency to
measure, so the ETW session that every process's presents would go through isn't started either.
RunLightly, which RunInBackground runs through, also has the process put its GPU work after other
processes' (see [Library/Graphics](../Graphics/README.md#gpu-priority)).

## PerfBar

Every program's window shows its performance along its top, in the PerfBar
(`Application/Performance.rs`, drawn by `Widgets/Shell.rs`): the frames over the last second, and
the input latency over the second before the newest frame known on the screen, since Windows' events
about presents come late: about 15 ms, as PresentTiming flushes ETW's buffers every 8 ms. Its
sections go one after another from its left, in this order, a `|` between each two, and a `=`
between each label and its value, as in `FPS = 100.00`. It changes what it says 10 times a second:

| Stat            | What it is                                                                                    |
| --------------- | --------------------------------------------------------------------------------------------- |
| `FPS`           | Frames a second that the main loop draws, with two decimals                                   |
| `Input latency` | The input latency's mean                                                                      |
| `Lookback`      | How many seconds back the stats look, 1                                                       |
| `Mode`          | How frames reach the screen, one of PresentTiming's present modes                             |
| `Flip`          | `immediate` if the display took the newest frame at once, or `at refresh` at its next refresh |
| `VSync`         | Whether frames wait for vertical sync, `on` or `off`. `F10` switches it                       |

`Flip` is what Windows' events say about the flip of the newest frame on the screen, like
PresentMon's `AllowsTearing`, not whether the program lets frames tear. Without a variable refresh
rate, a frame that the display takes at once replaces the one that it's scanning out, mid-refresh,
which tears. With G-SYNC, or another variable refresh rate, the display waits for the frame and
starts a new refresh for it instead, which doesn't tear, so `Flip` can say `immediate` with VSync
on too.

The PerfBar is pure black and white: `#000000` text on `#FFFFFF` in the light theme, and
`#FFFFFF` on `#000000` in the dark one. Its text is set in Cascadia Mono at 14 pixels,
`PERF_BAR_TEXT`, centered above its divider, in fields whose widths never change, so that every
label and value stays in the same place: numbers are right-aligned in their fields, a number that
isn't known yet, like the input latency's before the first present, is a dash, and one too wide for
its field fills it with `#`. Without input latency, the PerfBar says why at its end, after `VSync`,
so that the stats before it stay in place when it comes and goes.

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
Without either, the PerfBar says that input latency needs the group. A scripted run, or a test, has
no input latency.

## Testing

**Headless.** `Headless` runs a program's frames without a window or a GPU, with input that the test
makes up, on a fixed clock, through the same interface as a real run, so presses land on the cards'
real layout. Each input method runs a frame with the input and a frame after it. `queue` only
queues events, for `frame` to run, so that a test can look at the very frame that reads them:

```rust
let mut app = Headless::new(Volume { level: 50.0 });
app.click("Mute");
assert_eq!(app.program.level, 0.0);
app.key(Key::F11);
assert!(app.fullscreen());
```

`render` draws the last frame offscreen with the GPU, with the program's 3D, for tests that check
pixels, and `record` draws it into a frame of the caller's, for timing frames into one target, as
[Benchmarks](../../Program/Benchmarks/README.md) does.

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
`VerticalSync`, `WindowFocused`, the `Pointer`, what's `Hovered`, `Pressed` and `Focused`, the open
`Popup` and `Dialog`, the `Tooltip` that shows, and `Targets`: every widget's `Area`, `Kind`,
`Enabled` and `Label`, by name. Menu items are named after their menu and label, like
`Menu/File/Menu/Save`, and a dialog's buttons like `Question/Primary`. Scripts copy and paste on the
interface's own clipboard, never Windows'. A scripted run reads and writes its files in its output
folder, through `startup.data_folder`.

`Test/UserInterface` tests the library itself, with a GPU test that draws offscreen.

## Layout

| Path                                | Contents                                                                    |
| ----------------------------------- | --------------------------------------------------------------------------- |
| `Source/Library.rs`                 | The library's modules and what it exports                                   |
| `Source/Application/Program.rs`     | `Program`, `Options`, `Startup`, and the command line                       |
| `Source/Application/Run.rs`         | The main loop                                                               |
| `Source/Application/Settings.rs`    | `Settings.json`                                                             |
| `Source/Application/Overlays.rs`    | Frameless windows over each display, like IdentifyDisplays' boxes           |
| `Source/Application/Performance.rs` | What the PerfBar shows, and the input latency from GameInput to the screen  |
| `Source/Core/Interaction.rs`        | What a widget is to the input: `Sense`, `Target`, `Response` and their kin  |
| `Source/Core/Ui.rs`                 | A frame's interface: targets, presses, focus, popups, tooltips              |
| `Source/Platform/Window.rs`         | The Win32 window, its focus, its cursor, its two setups, and their check    |
| `Source/Platform/InputDevices.rs`   | Reads gamepads, keyboards and mice with GameInput, once a frame             |
| `Source/Platform/Clock.rs`          | The performance counter, which input and frames are timed on                |
| `Source/Platform/Clipboard.rs`      | Windows' clipboard, and the interface's own for scripts and tests           |
| `Source/Input/Devices.rs`           | Keys, shortcuts, mouse buttons and the gamepad                              |
| `Source/Input/Events.rs`            | A frame's input, from its events                                            |
| `Source/Input/Reader.rs`            | Turns a frame's GameInput readings into events                              |
| `Source/Design/Theme.rs`            | WinUI's theme resources, light and dark                                     |
| `Source/Design/Tokens.rs`           | Fluent's spacing, radii, type ramp, elevation, and the page's sizes         |
| `Source/Design/Controls.rs`         | WinUI 3's controls' sizes, paddings and parts                               |
| `Source/Design/Icons.rs`            | Segoe Fluent Icons                                                          |
| `Source/Drawing/Geometry.rs`        | Points, sizes, insets, rectangles and colors                                |
| `Source/Drawing/Text.rs`            | DirectWrite: fonts, cached layouts, hit testing, and the glyph atlas        |
| `Source/Drawing/Painter.rs`         | The draw list, and the Painter that programs draw their own graphics with   |
| `Source/Drawing/Renderer.rs`        | Draws the draw list with Direct3D 12, in one draw call                      |
| `Shaders/Interface.hlsl`            | The shapes' and glyphs' shader                                              |
| `Source/Widgets/*.rs`               | The widgets, cards, menus, dialogs, and the page's bars                     |
| `Source/Testing/Script.rs`          | Scripts                                                                     |
| `Source/Testing/Headless.rs`        | Runs a program's frames without a window, for tests                         |
| `Build.rs`                          | Compiles the shader at build time                                           |
