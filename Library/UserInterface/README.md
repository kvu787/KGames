# UserInterface

The user interface framework of every KGames GUI program on Win32, Direct3D 12 and DirectWrite.
It has:

- `run`, the main loop that every program shares. It opens the window with the program's
  settings, draws frames with the lowest latency, and switches to and from borderless
  fullscreen on `F11`. A program is a `Program`, which handles each frame's input and draws its
  interface.
- An immediate-mode interface. Each frame, the program calls a method of the `Frame` for each
  widget, with where it goes and the value it shows, and the method draws it, changes the value,
  and says what the input did to it, in a `Response`.
- The header and footer around a program's panels, and the page that says what's wrong when the
  window isn't in a supported setup.
- The spacing that the programs share, and panels.
- `Headless`, which draws frames of a program's interface without a window or a GPU, for tests.

It draws with [Library/Canvas](../Canvas/README.md), in its light theme, which it passes on, so
programs need only this crate for their interface. [UiShowcase](../../Program/UiShowcase/README.md)
shows every widget.

## A program

```rust
use user_interface::{Frame, Program, Rect, Setup};

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
        frame.header("Counter", "counts presses", &[]);
        frame.footer(&[("F11", "borderless fullscreen")], "");
        let inner = frame.panel(Rect::new(16.0, 96.0, 400.0, 200.0), "Count", "");
        if frame.button("Add", Rect::new(inner.x, inner.y, 80.0, 30.0), "Add").clicked {
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
it, like SimplePaintDemo's sphere, and `minimized`. `DEPTH` gives the renderer a depth buffer, and
`FULLSCREEN_KEY` can leave `F11` to a program that reads its keys another way, like with
GameInput. `Setup` has what a program is made with: its folder, the command line, its settings,
the notices that the header shows, and the renderer.

## How input reaches widgets

A widget is known by its name, which is unique among the frame's widgets, and the same from frame
to frame. Each frame, each widget leaves its area, with its name and what input it takes, as a
target. Input goes to the targets of the frame before, which is what was on the screen when it
came: clicks hit what the frame before them drew.

| Input       | Where it goes                                                                  |
| ----------- | ------------------------------------------------------------------------------ |
| A press     | The topmost target under it, which holds the mouse until the release, so that  |
|             | it gets the drag even outside its area. It takes keyboard focus, if it takes   |
|             | focus at all. Buttons act on the press, which comes before the release         |
| The release | The widget with the press                                                      |
| The wheel   | The topmost target under it that scrolls                                       |
| Keys        | The widget with keyboard focus, if it takes them, or else the program, for its |
|             | shortcuts, which `Frame::shortcut` reads                                       |
| Characters  | The text field with keyboard focus                                             |
| `Tab`       | Moves keyboard focus to the next widget that takes it, `Shift+Tab` back        |
| `Esc`       | Closes a popup, or else takes keyboard focus away, or else goes to the program |

Widgets draw on three layers, each over the ones before, in one draw call: the base, popups
like a select box's list, and tooltips. A popup's targets are over all of the base's. While a
popup is open, a press anywhere but on it or its select box closes it, and does nothing else.

Without the window's focus, a frame has no cursor and no events but the release of a press, and
popups close. `Frame::set_enabled(false)` makes the widgets after it faint, and take no input,
though they still show tooltips. `Frame::set_focusable(false)` keeps the widgets after it from
taking keyboard focus, for a program that has its own keys for them, like SimplePaintDemo's
arrow keys for its sliders.

## Widgets

| Method                    | What it is                                                              |
| ------------------------- | ----------------------------------------------------------------------- |
| `button`, `button_with`   | A button, `Normal`, `Primary` or `Danger`. Space and Enter press it too |
| `checkbox`                | A box that a press checks, with a label                                 |
| `switch`                  | A switch that a press turns on or off, with a label                     |
| `radio`                   | A column of radio buttons. Up and Down move the choice                  |
| `tabs`                    | A row of tabs. Left and Right move the choice                           |
| `slider`                  | A number in whole steps of a `SliderRange`, from a press and drag along |
|                           | its track, a step a notch of the wheel, and Left, Right, Home and End   |
| `select`                  | A box that shows one of a list of `Choice`s, and opens the list to pick |
|                           | another                                                                 |
| `text_field`              | A line of text to type in, with a caret, which Enter submits            |
| `list`                    | Rows of the same height, which scroll, of which one is selected. The    |
|                           | program draws what's in each row that shows                             |
| `chips`, `chip_switches`  | Chips that show what's on, and chips that a press turns on or off       |
| `progress`                | A bar filled to a fraction                                              |
| `tooltip`, `tooltip_text` | A box of text by the cursor, which the program draws while a response   |
|                           | is `hovered`                                                            |
| `panel`                   | A box with a title, which returns the area under it                     |
| `text_box`                | A box of colored lines, over a 3D view                                  |
| `interact`                | Makes an area a target, for a widget that the program makes itself.     |
|                           | `interact_labeled` gives it a label too, which scripts see              |

The parts of a widget are targets of their own, called `<name>/<part>`, like `Size/Large` for a
radio button, `Color/2` for a select box's third choice, and `Items/7` and `Items/Scroll bar` for
a list's row and scroll bar.

## Layout

Programs only support two window setups, whose client areas are 2560 × 1369 and 2560 × 1440 (see
[Library/Application](../Application/README.md#supported-window-setups)), so everything is in
fixed pixels. `Rect` splits into parts with `split_top`, `split_left`, `split_bottom`,
`split_right`, `columns` and `rows`, and `Stack` lays out areas down a rectangle.

| Constant     | Value | What it is                                                        |
| ------------ | ----- | ----------------------------------------------------------------- |
| `FONT_SIZE`  | 15    | The em height of the font, Consolas, in pixels                    |
| `MARGIN`     | 16    | Between panels, and between them and the window's edges           |
| `EDGE`       | 12    | Around the header's and footer's text                             |
| `PADDING`    | 12    | Inside a panel, around its contents                               |
| `PANELS_TOP` | 96    | Where panels start, under the header and its two lines of notices |

`Frame::footer` returns where the panels end above it, which is 71 pixels lower in borderless
fullscreen.

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
each target's `Name`, `Kind` and `Area`, and the name of what's `Hovered`, `Held`, has `Focus`,
and has its `Popup` open, and the `Tooltip`'s `Name` and `Area`. Scripts find where to press in
it, and check it. `Test/UserInterface` tests the framework.

| Path                  | Contents                                                                 |
| --------------------- | ------------------------------------------------------------------------ |
| `Source/Shell.rs`     | The main loop, and `Program`                                             |
| `Source/Interface.rs` | Widgets' identities and targets, where input goes, focus, and layers     |
| `Source/Controls.rs`  | Buttons, checkboxes, switches, radio buttons, tabs, chips, progress bars |
| `Source/Slider.rs`    | Sliders                                                                  |
| `Source/Select.rs`    | Select boxes and their lists                                             |
| `Source/TextField.rs` | Text fields                                                              |
| `Source/List.rs`      | Lists that scroll                                                        |
| `Source/Tooltip.rs`   | Tooltips, and where they go                                              |
| `Source/Chrome.rs`    | The header, the footer, and the page about an unsupported window         |
| `Source/Panels.rs`    | Spacing, panels, boxes of text, and `Stack`                              |
| `Source/Headless.rs`  | Frames without a window or a GPU, for tests                              |
