# UserInterface

KGames' user interface library, for Win32, Direct3D 12 and DirectWrite, after Microsoft's
[Fluent 2](https://fluent2.microsoft.design) design system as WinUI 3 implements it on Windows 11,
with WinUI 3's own values: its theme resources' colors, its controls' sizes, paddings and parts,
and its elevation borders and shadows, but not its animations. Every GUI program is built on it: it
owns the window, reads the input, runs the main loop, lays out text, draws the widgets on the GPU,
and runs scripts and headless tests. [UiShowcase](../../Program/UiShowcase/README.md) shows every
widget.

A program is its state, what each frame's input means to it, and a description of its interface
from that state. Each frame, the main loop reads the newest input and hit-tests it against the tree
the user saw; the widgets it landed on turn it into the program's messages, and the program's
`input` adds its own, like shortcuts; `update` applies each message to the state, in order, each
seeing what the one before left; and one read-only `view` builds the frame's tree of widgets from
the state, which is laid out, drawn and presented. So everything a frame's input causes shows on
that same frame, and nothing is retained between frames but the program's state and what only a
widget needs, like a text field's caret or a list's scroll, which the library keeps by the widget's
name.

## A program

```rust
use user_interface::{AppBar, Button, Card, Context, Icon, Options, Program, Slider, Startup, Status, Tree, Ui};

struct Volume {
    level: f64,
}

#[derive(Clone)]
enum Message {
    Set(f64),
    Mute,
}

impl Program for Volume {
    type Message = Message;

    fn title(&self) -> String {
        "Volume".to_string()
    }

    fn update(&mut self, message: Message, _ui: &mut Context<Message>) {
        match message {
            Message::Set(level) => self.level = level,
            Message::Mute => self.level = 0.0,
        }
    }

    fn view(&self, ui: &mut Ui) -> Tree<Message> {
        let mut tree = Tree::new();
        let shell = tree.shell(ui, AppBar::new(Icon::Volume, "Volume"), false, Some(vec![Status::shortcut("F11", "Fullscreen")]));
        tree.card(ui, None, shell.page.left_strip(400.0).top_strip(200.0), Card::new("Output"), |_, card, area| {
            let mut area = area.content;
            let slider = Slider::new(self.level, 0.0, 100.0).step(1.0).header("Level").show_value(0).on_change(Message::Set);
            card.add("Level", area.cut_top(Slider::<Message>::height(true), 8.0), slider);
            card.add("Mute", area.cut_top(32.0, 0.0).left_strip(120.0), Button::new("Mute").on_press(Message::Mute));
        });
        tree
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

| `Program` item | What it's for                                                                 |
| -------------- | ----------------------------------------------------------------------------- |
| `Message`      | What a frame's input means to the program; `Clone`, and usually an enum       |
| `title`        | The window's title                                                            |
| `input`        | The program's own messages from the frame's input, like shortcuts or a tick,  |
|                | queued after the widgets'                                                     |
| `update`       | Applies one message to the program's state, with the `Context` to send more,  |
|                | ask the time, or close the window                                             |
| `view`         | Describes the frame's interface from the state, read-only, as a `Tree` of     |
|                | widgets                                                                       |
| `prepare`      | Creates the program's GPU resources, once                                     |
| `render`       | Draws the program's 3D views into the frame, under the interface              |
| `may_close`    | Whether the window may close now, or asks first, like about unsaved work      |
| `state`        | What scripts save and check, as a JSON object                                 |
| `script`       | Does a script's `Program` step, like connecting a made-up device              |

A message sent from `update`, with `Context::send`, joins the same frame's queue. A per-frame
message from `input`, queued last, is the place for what runs after everything else, like
committing an edit once the button is up, or rebuilding what a 3D view shows.

## The design system

Color with the theme's roles and size with the tokens and the controls' metrics, instead of values
of your own, so that every program looks like one family, and like Windows 11.

- **Themes** (`design/theme.rs`): light and dark, which `F9` switches between in every program,
  which starts in the one that `Theme` in its [settings](#settings-and-the-command-line) says.
  The roles are WinUI's theme resources, like `text_primary` for TextFillColorPrimary,
  `control_fill` for ControlFillColorDefault, and `card_background` for
  CardBackgroundFillColorDefault, with Windows 11's default blue accent and its ramp. Status
  colors, like `critical`, and colors that tell graphs' series apart, `theme.data`, are there too.
- **Tokens** (`design/tokens.rs`): Fluent's 4-pixel spacing ramp (`SPACE_XXS` to `SPACE_XXXL`),
  corner radii (4 for controls, 8 for cards, flyouts and dialogs), the control height (32), the type
  ramp (`CAPTION`, `BODY`, `BODY_STRONG`, `BODY_LARGE`, `SUBTITLE`, `TITLE`, `TITLE_LARGE`,
  `DISPLAY`, and `MONO` and `MONO_CAPTION` in Cascadia Mono), elevation as WinUI's ThemeShadow makes
  it from a depth (`elevation(z)`, with tooltips at 16, flyouts at 32, submenus 8 over their menu,
  and dialogs at 128), and the page's sizes.
- **Controls** (`design/controls.rs`): WinUI 3's controls' own metrics, like a button's padding of
  11, 5, 11 and 6, a toggle switch's 40 × 20 track, a slider's thumb, a combo box's chevron column,
  a list item's backplate and selection pill, and a menu item's padding, each from WinUI's styles.
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
- **Icons** (`design/icons.rs`): Fluent's system icons, from Segoe Fluent Icons.
- **The page** (`widgets/shell.rs`): an app bar along the top with the program's icon, title, and
  its own menus, tabs and commands, a menu row under it for a menu bar whose menus' names line up
  with the page's left edge, if the program wants one, notices under them, a status bar along the
  bottom with shortcuts and status, and cards between them, `PAGE_PADDING` from the window's edges
  and `CARD_GAP` from each other.

## Widgets

A widget is a value that `view` builds, like `Button::new("Save").accent()`, and adds to the
frame's tree with `tree.add(name, rect, widget)`, or `tree.draw(rect, widget)` for what's only
looked at, like text. What it does is a reaction given where it's built: a message, like
`on_press(Message::Save)`, or a function of the widget's value that makes one, like
`on_change(Message::Set)`. The name is what it's known by from frame to frame, and what scripts and
tests point at. Each named widget is a target, whose area, kind and label the interface's state
lists. A widget of a program's own implements `Widget<M>`: its `kind`, `sense` and `parts` say what
it is to the input, `interact` turns what happened to it into messages through `Dispatch`, and
`draw` paints it through `Draw`, whose `look` has the frame's hover, press and focus. Every
context, the program's `Ui` and `Context` and a widget's `Parts`, `Dispatch` and `Draw`, reads the
frame's state through `Interface`'s getters, like `theme()`, `time()`, `input()` and `bounds()`.

| Widget                             | WinUI's                   | Notes                                                           |
| ---------------------------------- | ------------------------- | --------------------------------------------------------------- |
| `Button`, `HyperlinkButton`        | Button, HyperlinkButton   | Standard, accent or subtle; text, an icon, or both              |
| `ToggleButton`                     | ToggleButton              | In the accent color while it's on                               |
| `CheckBox`, `RadioButton`          | CheckBox, RadioButton     | A radio button is checked while the value is its own            |
| `RadioGroup`                       | RadioButtons              | A column of radios that picks one of its items, by index        |
| `ToggleSwitch`                     | ToggleSwitch              |                                                                 |
| `Slider`                           | Slider                    | Steps, the wheel, a header, and the value as dragged            |
| `NumberBox`                        | NumberBox                 | Typed, stepped with its buttons or the wheel                    |
| `ComboBox`                         | ComboBox                  | Its list opens over it, with the chosen item where it is        |
| `TextBox`                          | TextBox                   | Typing and Backspace, a clear button, and a button of its own   |
| `ColorPicker`, `ColorField`        | ColorPicker               | A spectrum with a value slider, and a hex field                 |
| `Palette`                          |                           | Swatches of colors to pick one from, with a ring around it      |
| `ListView`                         | ListView                  | Rows that scroll and are virtualized, one selected              |
| `SelectorBar`                      | SelectorBar               | Tabs, with a pill under the selected one                        |
| `Segmented`                        | Toolkit's Segmented       | Segments of equal width, with text, icons, or both              |
| `MenuBar`, `Button::menu`          | MenuBar, MenuFlyout       | Icons, accelerators, checks, separators and submenus            |
| `Element::context_menu`            | MenuFlyout                | Opens at the pointer on a right press, on any element           |
| `Dialog`                           | ContentDialog             | Over smoke, with up to three buttons and content of your own    |
| `Expander`                         | Expander                  | A header that shows and hides what's under it                   |
| `InfoBar`                          | InfoBar                   | Informational, success, warning or error; closable              |
| `ScrollArea`, `ScrollBar`          | ScrollViewer              | Content taller than its area, with the wheel                    |
| `Element::tooltip`, `tooltip_wide` | ToolTip                   | After the pointer rests half a second, above it; wider if asked |
| `ProgressBar`, `ProgressRing`      | ProgressBar, ProgressRing | Determinate, or of unknown length                               |
| `Tree::card`                       | Card                      | A card with children; `with_view` leaves a hole for 3D          |
| `Tree::sized_card`                 | Card                      | A card as tall as its content                                   |
| `Tree::shell`                      |                           | The page's bars and notices, with its `page` between them       |
| `View`                             |                           | A 3D view's area, which takes every button but never focus,     |
|                                    |                           | and sends what the pointer does in it                           |
| `Canvas`                           |                           | Draws a program's own graphics, with the frame's `Painter`      |
| `Cover`, `Group`                   |                           | A surface that stops the pointer, and a bare grouping           |
| `Tree::info_card`                  |                           | A test program's floating card: frame rate, GPU, display, keys  |
| `Form`                             |                           | Lays a card's content out top to bottom, with headers           |
| `Keycaps`, `Badge`, `Divider`      | InfoBadge, and others     | Keycaps of shortcuts, counts on a pill, and lines               |
| `Text`, `Field`, `Swatch`, `Spans` |                           | Labels, captions, label-value rows, color squares, mixed runs   |

`tree.add(...)` returns the element, whose `set_enabled`, `set_tooltip`, `set_label` and
`set_context_menu` say more about it. A container's children, like a card's, inherit its enabling,
so a disabled card can't have a working widget.
A widget that takes the wheel says so in its `Sense` - `takes_wheel()`, like a slider's, or
`takes_wheel_while_focused()`, like a number box's - so what it's in doesn't scroll under it;
`Sense::VIEW` takes it already.
`Canvas::new(|painter, rect| ...)` draws a program's own graphics, like graphs and previews:
rounded rectangles with borders and gradients, Fluent's shadows, circles, rings, arcs, lines,
polygons, text and icons, with clipping. State that a widget of a program's own keeps between
frames, like a scroll offset, lives in the interface's memory, `ctx.memory::<T>()` in its contexts,
by the widget's id; `Ui::memory` reads it in `view`, like for laying out a scrolled list.

## Input

All input comes from [GameInput](../GameInput/README.md): keyboards, mice and gamepads. Every
program needs GameInput 4.0.306 installed, as that README says, and doesn't start without it, not
even in a scripted run or with `--no-input`, which don't read it. `platform/input_devices.rs`
reads every device's readings once a frame, right before the frame, and `input/reader.rs` turns
them into the interface's events, in the order they happened, with the window's focus changes
between them. The window's messages bring none of it. Each frame walks GameInput's history of
readings, for the freshest, and takes the ones that GameInput's reading callback queued, as the
history keeps only about 512 readings, under a tenth of a second of a gamepad's, so that a long
frame misses none.

The library leaves GameInput's background input off, as it is by default, so GameInput gives a
program input only while it has focus. On, it would give input without focus too, but GameInput
then reads gamepads in Xbox 360 mode on a timer instead of as they report: on the development
machine, with GameInput 4.0.306, two GameSir controllers in Xbox 360 mode that report 8,000 times a
second gave a focused window 64 readings a second with it on, and about 7,500 with it off.
`Library/GameInput` doesn't bind `EnableBackgroundInput`, so that it stays off. As a window loses
focus, GameInput gives it a reading of each device at rest, which lets go of everything held, and
none after that. As focus comes back, it doesn't report keys or mouse buttons that are held from
before, but reports gamepads' whole state, their held buttons too.

- Keys are the keys that keyboards hold down. Shift, Ctrl and Alt are the same keys on the left
  and the right. Typed text is the keys translated with Windows' keyboard layout, with Shift, AltGr
  and dead keys, but no IME. A held key repeats at Windows' keyboard delay and rate.
- The mouse's buttons and wheels come from mice. The pointer is where Windows' cursor is, so it
  moves at Windows' pointer speed, and a press while the cursor isn't on the window is another
  window's.
- The pointer is on the topmost target under it, in the tree the user saw, so a combo box's
  list, a menu or a dialog takes the pointer from what's under it. A card, and `Cover`, cover
  what's under them, like a 3D view.
- Widgets act on a press, on the frame that the button goes down, and never wait for its release,
  which only ends a drag: a button sends its message, a switch flips, a combo box's list or a menu
  opens, an item is chosen, and a right press opens a context menu. Sliding off before letting go
  doesn't take a press back. `Hit::pressed` is a left press, and `ViewInput::pressed` is a click
  in a 3D view. While a button is held, the window captures the
  mouse, so a drag keeps going past its edges.
- Only text fields take keyboard focus, and only they take keys: a press on one gives it focus,
  and a press anywhere else takes it away, once the field has taken what was typed in that frame.
  What's typed goes in at the caret, which a press places, Backspace deletes the character before
  it, and Enter takes what's typed in a number box or a color field, which goes if the field loses
  focus first.
  No other widget does anything with the keyboard, so menus, combo boxes' lists and dialogs close
  with a press, and there's no Tab, Esc or arrows between widgets. A text field under an open
  dialog gets no keys.
- `ui.key_pressed`, `ui.key_pressed_once` and `ui.shortcut` are for programs' shortcuts. They
  don't see the keys that a text field with focus takes - every key without Ctrl - or any while
  a dialog is open - from the very frame it opens, for checks after it.
  `ui.input()` has all of the frame's input, for programs that read keys themselves, like for
  driving.
- Without focus, the window has no input at all, including the gamepad's. As focus goes,
  everything held is let go of on that frame: keys and mouse buttons are released, the gamepad
  goes to rest, and `ui.input().focus_lost` says why. The click that gives it focus does only
  that, and keys and gamepad buttons held when it gets focus count once they're let go of:
  GameInput doesn't report the keys and the click, and the interface ignores the gamepad's held
  buttons until they're let go of. A reading from before the window got focus is input without
  it, even if GameInput is late with it and it comes in a later frame.
- `F9` switches between the light and dark themes, whichever the `Theme` setting started it in.
  The frame that `F9` is pressed in is all in the new theme, and the title bar asks for it too,
  which DWM draws, as its documentation promises while Windows' own dark mode is on. A program's
  own command for it calls `ui.toggle_theme()` in `update`, and `ui.theme()` reports the new theme
  for the rest of that frame, `view` too, so no frame mixes the two; `ui.theme().kind` says which
  it is. Every widget, card and color role follows by itself, as long as the program colors with
  `ui.theme()` each frame.
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
| Windowed maximized    | 2560 × 1369 | The 2560 × 1392 work area, less the title bar's 23 pixels |
| Borderless fullscreen | 2560 × 1440 | A window without a frame that covers the display          |

In any other setup, the window says what's wrong instead of showing the program. It checks again
whenever the window moves or changes size, or the displays, the scale or the taskbar change.

The top `PERF_BAR_HEIGHT`, 17 pixels, of every program's client area is the library's: its
[PerfBar](#perfbar). The program has the rest, `ui.bounds()`, which starts at
`ui.content_top()`, with the app bar at its top. Lay out cards in fixed pixels for the two client
areas, which leaves the program 2560 × 1352 maximized and 2560 × 1423 in borderless fullscreen;
they differ by 71 pixels of height. A 3D view may still fill the window, under the PerfBar.

## Settings and the command line

`Settings.json`, in the program's folder, has the settings every program has, which the library
reads first, into `startup.common`, then the program's own, which it reads from `startup.settings`
with their defaults. A missing file is written with every default; a wrong setting keeps its
default, and a notice under the app bar says what's wrong. `Headless::start` starts a program in
the theme that its `Startup`'s settings say, as a run does. Every program's `Settings.json` lists
every one of the settings that every program has, which `Test/UserInterface` checks.

| Setting        | Value                                                                    |
| -------------- | ------------------------------------------------------------------------ |
| `Display`      | `"Primary"`, or the identifier of the display to open on                 |
| `WindowMode`   | `"WindowedMaximized"`, by default, or `"BorderlessFullscreen"`           |
| `VerticalSync` | `true`, by default: frames wait for the screen's refresh, and never      |
|                | tear. `false`: frames present at once, and may tear                      |
| `Theme`        | `"Light"`, by default, or `"Dark"`: the theme that the program starts in |

| Argument               | What it does                                                              |
| ---------------------- | ------------------------------------------------------------------------- |
| `--display <id>`       | Opens on the display, which must be connected. `Primary` for the primary  |
| `--no-activate`        | Opens beneath the other windows, without taking focus, in efficiency mode |
| `--no-input`           | Doesn't start GameInput, so the program reads no input at all             |
| `--vsync`              | Waits for vertical sync, whatever `VerticalSync` says                     |
| `--no-vsync`           | Doesn't wait for vertical sync, whatever `VerticalSync` says              |
| `--theme <theme>`      | Starts `Light` or `Dark`, whatever `Theme` says                           |
| `--window-mode <mode>` | Starts `WindowedMaximized` or `BorderlessFullscreen`, whatever            |
|                        | `WindowMode` says                                                         |
| `--script <path>`      | Runs a script instead of the user's input, with `--output <folder>`       |

What the command line says goes into `startup.common` too, so a program sees the theme, window mode
and vertical sync that it starts in. Of an argument given twice, the last one says.

## Input latency

The main loop (`application/run.rs`) handles the window's messages without waiting, and waits until
the GPU has finished the last frame and DXGI can take another. Only then does it read the input,
build the frame's interface, draw it and present it, so a frame's input is as fresh as it can be.
The swap chain has a frame latency waitable object and a maximum frame latency of 1, and draws on
the GPU that the window's monitor is an output of. The whole interface is one draw call of
instanced quads, whose pixel shader computes each shape's coverage from its signed distance. Text
layouts are cached, so a frame that shows the same text as the last one asks DirectWrite for
nothing, and glyphs are rasterized once each into an atlas. Minimizing the window stops the loop
drawing.

Programs start with vertical sync on or off as `VerticalSync` says, unless `--vsync` or
`--no-vsync` says otherwise. With it on, frames wait for the screen's refresh, and DXGI can take
another only once the last one has gone to the screen, so the loop waits for that, and for
messages, instead of spinning: a frame couldn't begin any sooner, and the input is still read as
soon as one can. `F10` switches it on and off while the program runs. With it off, frames present
as soon as they're drawn, and may tear, and the loop spins, so that the input that reaches the
screen is as fresh as it can be.

RunInBackground's test runs also start light and windowed maximized, with `--theme Light` and
`--window-mode WindowedMaximized`, whatever the program's settings say, so that scripts start the
same way whatever someone set for themselves. They use `--no-activate`, `--vsync` and `--no-input`,
so that they share the computer lightly with someone using it, like playing a game: the window opens
beneath the others and the process asks Windows for efficiency mode, where a window that someone
uses asks for full execution speed; frames wait for vertical sync, whatever the program's settings
say, at most one a refresh, and the loop waits on DXGI between them, instead of spinning; GameInput,
with its helper process that reads keyboards and mice, doesn't start; and without input there's no
input latency to measure, so the ETW session that every process's presents would go through isn't
started either. RunLightly, which RunInBackground runs through, also has the process put its GPU
work after other processes' (see [Library/Graphics](../Graphics/README.md#gpu-priority)).

## PerfBar

Every program's window shows its performance along its top, in the PerfBar
(`application/performance.rs`, drawn by `widgets/shell.rs`): the frames over the last second, and
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
real layout. Each input method runs one frame with the input, whose output came from it, so a test
asserts on exactly what that input made. `queue` only queues events, for `frame` to run, and
`send_message` and `queue_message` feed the program messages directly, for testing `update` on the
real pipeline:

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
`Menu/File/Menu/Save`, and a dialog's buttons like `Question/Primary`. A scripted run reads and
writes its files in its output folder, through `startup.data_folder`.

`Test/UserInterface` tests the library itself, with a GPU test that draws offscreen.

## Layout

| Path                             | Contents                                                                   |
| -------------------------------- | -------------------------------------------------------------------------- |
| `src/lib.rs`                     | The library's modules and what it exports                                  |
| `src/application/program.rs`     | `Program`, `Options`, `Startup`, and the command line                      |
| `src/application/run.rs`         | The main loop                                                              |
| `src/application/settings.rs`    | `Settings.json`                                                            |
| `src/application/performance.rs` | What the PerfBar shows, and the input latency from GameInput to the screen |
| `src/core/interaction.rs`        | What a widget is to the input: `Sense`, `Target`, `Interaction` and `Hit`  |
| `src/core/tree.rs`               | The view tree: `Tree`, `Element`, `Widget`, and its contexts               |
| `src/core/interface.rs`          | The interface: input against the shown tree, messages, layout, focus,      |
|                                  | popups, tooltips, widget memory, and the frame's phases                    |
| `src/platform/window.rs`         | The Win32 window, its focus, its cursor, its two setups, and their check,  |
|                                  | and the theme's background that it shows until its first frame             |
| `src/platform/input_devices.rs`  | Reads gamepads, keyboards and mice with GameInput, once a frame            |
| `src/platform/clock.rs`          | The performance counter, which input and frames are timed on               |
| `src/input/devices.rs`           | Keys, shortcuts, mouse buttons and the gamepad                             |
| `src/input/events.rs`            | A frame's input, from its events                                           |
| `src/input/reader.rs`            | Turns a frame's GameInput readings into events                             |
| `src/design/theme.rs`            | WinUI's theme resources, light and dark                                    |
| `src/design/tokens.rs`           | Fluent's spacing, radii, type ramp, elevation, and the page's sizes        |
| `src/design/controls.rs`         | WinUI 3's controls' sizes, paddings and parts                              |
| `src/design/icons.rs`            | Segoe Fluent Icons                                                         |
| `src/drawing/geometry.rs`        | Points, sizes, insets, rectangles and colors                               |
| `src/drawing/text.rs`            | DirectWrite: fonts, cached layouts, hit testing, and the glyph atlas       |
| `src/drawing/painter.rs`         | The draw list, and the Painter that programs draw their own graphics with  |
| `src/drawing/renderer.rs`        | Draws the draw list with Direct3D 12, in one draw call                     |
| `Shaders/Interface.hlsl`         | The shapes' and glyphs' shader                                             |
| `src/widgets/*.rs`               | The widgets, cards, menus, dialogs, and the page's bars                    |
| `src/testing/script.rs`          | Scripts                                                                    |
| `src/testing/headless.rs`        | Runs a program's frames without a window, for tests                        |
| `build.rs`                       | Compiles the shader at build time                                          |
