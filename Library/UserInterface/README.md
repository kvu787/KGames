# UserInterface

The user interface framework of every KGames GUI program on Win32, Direct3D 12 and DirectWrite.
It looks and acts like Windows 11: its widgets, sizes, type ramp and colors follow
[Fluent 2](https://fluent2.microsoft.design/), Windows 11's design system, with a few ideas from
Material 3 and Apple's guidelines where Fluent has none, like filter chips and a segmented
control. It has:

- `run`, the main loop that every program shares. It opens the window with the program's
  settings, draws frames with the lowest latency, and switches to and from borderless
  fullscreen on `F11`. A program is a `Program`, which handles each frame's input and draws its
  interface.
- An immediate-mode interface. Each frame, the program calls a method of the `Frame` for each
  widget, with where it goes and the value it shows, and the method draws it, changes the value,
  and says what the input did to it, in a `Response`.
- The widgets, in the light theme by default, or the dark one, and the cards that hold them.
- The app bar and status bar around a program's cards, notices that the user can close, and the
  page that says what's wrong when the window isn't in a supported setup.
- The design system's sizes, in `Tokens.rs`, and stacks that lay out in fixed pixels.
- `Headless`, which draws frames of a program's interface without a window or a GPU, for tests.

It draws with [Library/Canvas](../Canvas/README.md), which it passes on, so programs need only
this crate for their interface. [UiShowcase](../../Program/UiShowcase/README.md) shows every
widget.

## A program

```rust
use user_interface::{Button, Frame, PANELS_TOP, Program, Rect, Setup};

#[derive(Default)]
struct Counter {
    count: u32,
}

impl Program for Counter {
    type Settings = ();
    const NAME: &'static str = "Counter";

    fn new(_setup: Setup<'_, ()>) -> windows_core::Result<Self> {
        Ok(Counter::default())
    }

    fn frame(&mut self, frame: &mut Frame) -> windows_core::Result<()> {
        frame.header("Counter", "counts presses");
        frame.footer(&[("F11", "fullscreen")], &[]);
        let inner = frame.card(Rect::new(16.0, PANELS_TOP, 400.0, 200.0), "Count", "");
        let add = Button::new("Add").accent();
        if frame.button("Add", Rect::new(inner.x, inner.y, 96.0, 32.0), add).clicked {
            self.count += 1;
        }
        Ok(())
    }

    fn state(&self) -> json::Value {
        json::object([("Count", self.count.into())])
    }
}

fn main() -> windows_core::Result<()> {
    user_interface::run::<Counter>(std::path::Path::new(env!("CARGO_MANIFEST_DIR")))
}
```

`Program` also has `draw_under`, for a 3D view under the interface, `draw_over`, for an image over
it, like SimplePaintDemo's sphere, and `minimized`. `DEPTH` gives the renderer a depth buffer,
`FULLSCREEN_KEY` can leave `F11` to a program that reads its keys another way, like with
GameInput, and `GAMEPAD` has the loop read the gamepad, for a program that a gamepad controls.
`Setup` has what a program is made with: its folder, the command line, its settings, the notices
that the app bar shows, and the renderer. `Frame::set_theme` draws in another theme, like
`theme::DARK`, from where it's called.

## How input reaches widgets

A widget is known by its name, which is unique among the frame's widgets, and the same from frame
to frame. Each frame, each widget leaves its area, with its name and what input it takes, as a
target. Input goes to the targets of the frame before, which is what was on the screen when it
came: clicks hit what the frame before them drew.

| Input        | Where it goes                                                                  |
| ------------ | ------------------------------------------------------------------------------ |
| A press      | The topmost target under it, which holds the mouse until the release, so that  |
|              | it gets the drag even outside its area. It takes keyboard focus, if it takes   |
|              | focus at all. Buttons act on the press, which comes before the release         |
| A second one | A double-click, if it's on the same widget within half a second and 4 pixels,  |
|              | which `Response::double_clicked` says                                          |
| The release  | The widget with the press                                                      |
| The wheel    | The topmost target under it that scrolls                                       |
| Keys         | The widget with keyboard focus, if it takes them, or else the program, for its |
|              | shortcuts, which `Frame::shortcut` reads                                       |
| Ctrl+keys    | The widget with focus, if it takes them, like a text field's Ctrl+C            |
| Held keys    | The program, whatever has keyboard focus. `Frame::held` says whether a key is  |
|              | down, for what goes on while it's held, like steering                          |
| The gamepad  | The program, if it reads it, through `Frame::gamepad`: its sticks, triggers,   |
|              | the buttons held, and the ones that went down since the frame before           |
| Characters   | The text field with keyboard focus                                             |
| `Tab`        | Moves keyboard focus to the next widget that takes it, `Shift+Tab` back        |
| `Esc`        | Closes a popup, or else takes keyboard focus away, or else goes to the program |

Widgets draw on three layers, each over the ones before, in one draw call: the base, popups
like a combo box's list, and the overlay, with tooltips and notices. A popup's targets are over
all of the base's. While a popup is open, a press anywhere but on it or its combo box closes it,
and does nothing else.

Without the window's focus, a frame has no cursor and no events but the release of a press, and
popups close. `Frame::set_enabled(false)` makes the widgets after it faint, and take no input,
though they still show tooltips. `Frame::set_focusable(false)` keeps the widgets after it from
taking keyboard focus, for a program that has its own keys for them, like SimplePaintDemo's
arrow keys for its sliders.

The ring around the widget with focus shows only when focus moved by keyboard, like Windows'.
How a widget looks eases to what the input did to it, like a switch's knob sliding over, or a
button's fill under the cursor, in about 100 ms, with `Frame::ease`. Values change on the frame
that the input comes, so the response starts at once, and only the look follows.

Text fields copy and paste with Windows' clipboard, while the user is the input, and with the
interface's own, while a script or a test is, so that they never touch what the user copied.

## Widgets

| Method                       | What it is                                                         |
| ---------------------------- | ------------------------------------------------------------------ |
| `button`                     | A `Button`: a label, an icon, or both, in a `ButtonStyle`:         |
|                              | `Standard`, `Accent`, `Subtle` or `Danger`. Space and Enter press  |
|                              | it too                                                             |
| `checkbox`                   | A box that a press checks, with a label                            |
| `switch`                     | A switch that a press turns on or off, with a label                |
| `radio`                      | A column of radio buttons. The arrow keys move the choice          |
| `segmented`                  | A row of choices, as wide as each other, with a raised mark under  |
|                              | the chosen one that slides                                         |
| `tabs`                       | A row of tabs, as wide as their labels, with a mark that slides    |
| `slider`                     | A number in whole steps of a `SliderRange`, from a press and drag  |
|                              | along its rail, a step a notch of the wheel, the arrow keys, Page  |
|                              | Up, Page Down, Home and End                                        |
| `select`                     | A combo box that shows one of a list of `Choice`s, and opens the   |
|                              | list to pick another                                               |
| `text_field`                 | A line of text to type in, as a `TextField` says: a placeholder,   |
|                              | a limit, and an icon. It selects, moves by words, copies and       |
|                              | pastes, and has a button that clears it. Enter submits it          |
| `list`                       | Rows of the same height, which scroll by pixels, of which one is   |
|                              | selected. The program draws what's in each row that shows          |
| `filter_chips`               | Chips that a press turns on or off, with a check while they're on  |
| `status_chips`               | Chips that show what's on, like the buttons that are held          |
| `progress`                   | A bar filled to a fraction                                         |
| `info_bar`                   | A message with how serious it is, a `Severity`, and a button that  |
|                              | closes it, if it has a name                                        |
| `tooltip`, `tooltip_text`    | A card of text by the cursor, which the program draws while a      |
|                              | response is `hovered`                                              |
| `badge`, `keycap`            | A pill with a dot, for a status, and a key's name on a key         |
| `card`, `card_trailing`      | A card with a title, which returns the area under it, and the      |
|                              | area after its title, for a control                                |
| `well`, `overlay_card`       | A well in a card, for a plot or a log, and a card over a 3D view   |
| `label`, `divider`           | Text in an area, aligned, and a line between things                |
| `interact`                   | Makes an area a target, for a widget that the program makes itself |
|                              | `interact_labeled` gives it a label too, which scripts see         |

The parts of a widget are targets of their own, called `<name>/<part>`, like `Size/Large` for a
radio button, `Color/2` for a combo box's third choice, `Search/Clear` for a text field's
button, and `Items/7` and `Items/Scroll bar` for a list's row and scroll bar.

## Layout

Programs only support two window setups, whose client areas are 2560 × 1369 and 2560 × 1440 (see
[Library/Application](../Application/README.md#supported-window-setups)), so everything is in
fixed pixels. `Rect` splits into parts with `split_top`, `split_left`, `split_bottom`,
`split_right`, `columns` and `rows`, and places a size in itself with `place` and `centered`.
`Stack::down` and `Stack::across` lay out areas one after another, like a vertical or horizontal
stack in SwiftUI, Compose or XAML. Widgets that have a size of their own say it, like
`Button::width`, `toggle_width` and `select_width`.

`Tokens.rs` has the design system's sizes, on Fluent's 4-pixel grid:

| Constant            | Value | What it is                                                         |
| ------------------- | ----- | ------------------------------------------------------------------ |
| `SPACE_XXS`         | 2     | Spacing, which goes on: `SPACE_XS` 4, `SPACE_S` 8, `SPACE_M` 12,   |
|                     |       | `SPACE_L` 16, `SPACE_XL` 20, `SPACE_XXL` 24 and `SPACE_XXXL` 32    |
| `MARGIN`            | 16    | Between cards, and between them and the window's edges             |
| `CARD_PADDING`      | 16    | Inside a card, around its contents                                 |
| `CARD_HEADER`       | 48    | From a card's top to its contents, under its title                 |
| `PANELS_TOP`        | 56    | Where cards start, under the app bar                               |
| `STATUS_BAR_HEIGHT` | 40    | The status bar's height, at the window's bottom                    |
| `CONTROL_HEIGHT`    | 32    | A button, a combo box, a text field, and a row of a checkbox       |
| `RADIUS_CONTROL`    | 4     | A control's corners                                                |
| `RADIUS_CARD`       | 8     | A card's, a popup's and an info bar's corners                      |
| `RAISED`, `FLYOUT`  |       | How high a card over a 3D view and a popup float, as their shadows |
| `EASE_SECONDS`      | 0.025 | The time constant of how a widget's look follows the input         |

`Frame::footer` returns where the cards end above the status bar, which is 71 pixels lower in
borderless fullscreen.

## Testing

`Headless` draws frames of an interface without a window or a GPU, with input that a test makes
up, like a script would, through the same layout as the program. `Headless::program` handles a
frame like the main loop does, and `Headless::target` says where the last frame's widget of a name
is, for a test's next press:

```rust
let mut counter = Counter::default();
let mut headless = Headless::default();
headless.program(&mut counter, &[])?;
let add = headless.center("Add");
headless.program(&mut counter, &headless::click(add))?;
assert_eq!(counter.count, 1);
```

In a scripted run, the main loop adds the interface's state to the program's, as `Interface`:
each target's `Name`, `Kind`, `Label` and `Area`, and the name of what's `Hovered`, `Held`, has
`Focus`, and has its `Popup` open, the `Tooltip`'s `Name` and `Area`, and the `Theme`. Scripts
press widgets by name, which the main loop finds in the frame before, and check the state.
`Test/UserInterface` tests the framework.

| Path                  | Contents                                                                 |
| --------------------- | ------------------------------------------------------------------------ |
| `Source/Shell.rs`     | The main loop, and `Program`                                             |
| `Source/Interface.rs` | Widgets' identities and targets, where input goes, focus, layers, easing |
| `Source/Tokens.rs`    | The design system's sizes                                                |
| `Source/Surfaces.rs`  | Cards, wells, info bars, the focus ring, shadows and raised faces        |
| `Source/Buttons.rs`   | Buttons                                                                  |
| `Source/Toggles.rs`   | Checkboxes, switches, radio buttons, segmented controls and tabs         |
| `Source/Slider.rs`    | Sliders and progress bars                                                |
| `Source/Select.rs`    | Combo boxes and their lists                                              |
| `Source/TextField.rs` | Text fields                                                              |
| `Source/List.rs`      | Lists that scroll                                                        |
| `Source/Chips.rs`     | Filter and status chips, badges and keycaps                              |
| `Source/Tooltip.rs`   | Tooltips, and where they go                                              |
| `Source/Chrome.rs`    | The app bar, the status bar, notices, and the unsupported window's page  |
| `Source/Clipboard.rs` | Windows' clipboard, or the interface's own                               |
| `Source/Headless.rs`  | Frames without a window or a GPU, for tests                              |
