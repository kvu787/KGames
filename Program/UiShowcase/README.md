# UiShowcase

Shows every widget of [Library/UserInterface](../../Library/UserInterface/README.md), the Fluent 2
user interface library of every KGames GUI program, and what the input does to each, in the light
theme or the dark one. Like every KGames GUI program, it ignores input without focus, and supports
the two window setups in [Library/UserInterface](../../Library/UserInterface/README.md#window-setups).

## What it shows

Five columns of cards, under an app bar and over a status bar:

| Card                       | What it shows                                                          |
| -------------------------- | ---------------------------------------------------------------------- |
| Buttons                    | Standard, accent, subtle and disabled buttons and how often each was   |
|                            | pressed, a button with an icon, a toggle button, a hyperlink, and      |
|                            | buttons of icons alone, which change a count                           |
| Checkboxes and switches    | A checkbox that disables the four under it, and switches for the       |
|                            | tooltips, the dark theme, and the progress bar that fills              |
| Radio buttons and segments | Radio buttons, and a square of the chosen size; segments that align a  |
|                            | line of text; and segments with icons, like a tool's modes             |
| Sliders and progress       | Sliders of whole numbers, of thousandths and of tenths of a degree,    |
|                            | and a disabled one; progress bars at the volume, filling every 4       |
|                            | seconds, and of unknown length; and progress rings                     |
| Combo boxes                | A color of the theme, with what it's for, and the list's density       |
| Text fields                | A name, which Enter or Greet says hello to in an info bar, and a       |
|                            | search of the list                                                     |
| Tooltips                   | A sentence, lines with columns, keycaps, badges, and a tooltip that    |
|                            | goes above the pointer, as there's no room under it                    |
| Tabs                       | The type ramp, the icons, the theme's colors, and shapes: elevation,   |
|                            | lines, circles, rings, arcs, polygons, gradients and corners           |
| Info bars                  | One of each severity, of which the caution closes, and comes back      |
| List                       | 120 items that scroll, of which the search shows those it matches, one |
|                            | can be selected, and a double-click opens one, each with a tooltip     |
| Interface                  | Whether the window has focus, the pointer, the widget under it, the    |
|                            | one with the press, the one with focus, the open popup and the         |
|                            | tooltip; the keys held; the gamepad's sticks, triggers and buttons;    |
|                            | and a log of what happened                                             |

The cards at the bottom of their columns take the 71 more pixels of borderless fullscreen.

## Input

| Input              | Action                                                                   |
| ------------------ | ------------------------------------------------------------------------ |
| A press            | Presses a button, flips a checkbox or a switch, picks a choice, sets a   |
|                    | slider, opens a combo box, or puts a text field's caret there            |
| A double-click     | Selects a word in a text field, or opens a row of the list               |
| Drag a slider      | Moves it, even past the window's edges                                   |
| Drag in a field    | Selects text                                                             |
| The wheel          | Moves a slider a step a notch, or scrolls the list 3 rows                |
| `Tab`, `Shift+Tab` | Moves keyboard focus to the next or previous widget                      |
| `Space`, `Enter`   | Presses the button with focus, or flips the checkbox or switch           |
| Arrow keys         | Move the slider, segments, tabs, combo box or list with focus, or a text |
|                    | field's caret, which Shift selects with, and Control moves by words      |
| `Ctrl+A`, `C`, `X` | Select everything in a text field, copy it, or cut it                    |
| `Ctrl+V`           | Pastes into a text field                                                 |
| `Esc`              | Closes a combo box's list, or else takes focus away                      |
| `F1`               | Shows or hides the tooltips                                              |
| `F2`               | Switches between the light and dark themes                               |
| `Ctrl+R`           | Resets every widget                                                      |
| `F11`              | Switches between maximized and borderless fullscreen                     |
| A gamepad          | Shows in the Interface card                                              |

The first press on the window while it doesn't have focus only gives it focus.

## Build and run

```powershell
cargo run --release
```

Or run `HumanRun.cmd`, which you can also double-click in File Explorer. It opens the showcase on
the primary display. `--display` and `--no-activate` work as in every program, and
[RunInBackground](../RunInBackground/README.md) runs it for coding agents. `Settings.json` has the
settings that every program has.

## Testing

```powershell
cargo test -p ui_showcase_tests
Program\RunInBackground\RunInBackground.ps1 UiShowcase --all-scripts
```

The tests in `Test/UiShowcase` drive the showcase through its cards' real layout, with
`user_interface::Headless`, which lays them out without a window or a GPU: presses on each kind of
widget, a press that slides off a button, drags past a slider's ends, the wheel, the keys, the combo
boxes' lists and what closes them, typing, copying and pasting, the search and its clear button, the
list's rows, scrolling and a double-click, the caution info bar, tooltips, `Tab`, `Ctrl+R`, `F11`, and
a window without focus. They also check that every widget fits both window heights, in both themes.

The scripts in `Test/UiShowcase/Scripts` run the same kinds of input through the real window and
renderer, pressing widgets by name. The state that scripts save and check has what each widget shows,
like `Volume` and `Selected`, the `Log`'s newest lines, and the interface's, as `Interface`.

## Layout

| Path                 | Contents                                                             |
| -------------------- | -------------------------------------------------------------------- |
| `Source/Main.rs`     | Runs the showcase with Library/UserInterface's main loop             |
| `Source/Library.rs`  | The showcase's parts, as a library that `Test/UiShowcase` tests      |
| `Source/Showcase.rs` | What each widget shows, the list's items, the log, and the shortcuts |
| `Source/Cards.rs`    | Lays out the cards in fixed pixels, and draws their widgets          |
| `HumanRun.ps1`       | Builds the showcase and runs it on the primary display, for people   |
| `HumanRun.cmd`       | Runs `HumanRun.ps1`, for double-clicking in File Explorer            |
| `Settings.json`      | The settings that every program has                                  |
