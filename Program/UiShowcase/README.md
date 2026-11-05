# UiShowcase

Shows every widget of [Library/UserInterface](../../Library/UserInterface/README.md), the user
interface framework of every KGames GUI program, and what the input does to each, in the light
theme, or the dark one with `F2`. Like every KGames GUI program, it ignores input without focus,
and supports the two window setups in
[Library/Application](../../Library/Application/README.md#supported-window-setups).

## What it shows

| Card                       | What it shows                                                         |
| -------------------------- | --------------------------------------------------------------------- |
| Buttons                    | Standard, accent, danger, subtle and disabled buttons, and how often  |
|                            | each was pressed, each with a tooltip. A button with an icon, and     |
|                            | buttons of icons alone, which change a count                          |
| Checkboxes and switches    | Checkboxes and switches, of which the first disables the four under   |
|                            | it, and switches for the tooltips and the dark theme                  |
| Radio buttons and segments | A column of choices, a square of the chosen size, and a segmented     |
|                            | control                                                               |
| Tabs                       | A row of tabs, which change the text under them                       |
| Sliders                    | Sliders of whole numbers, of thousandths and of tenths of a degree, a |
|                            | disabled one, and progress bars: one at the volume, and one that      |
|                            | fills every 4 seconds while the switch for it is on                   |
| Combo boxes                | A color, whose list says what each is for in the theme, and the       |
|                            | list's density, which sets how tall its rows are                      |
| Chips                      | Filter chips that a press turns on or off, and chips that show what's |
|                            | on                                                                    |
| Tooltips                   | Boxes that show a sentence, columns, and a tooltip that goes above    |
|                            | the cursor, as there's no room under it                               |
| Text fields                | A name, which Enter or Greet says hello to in an info bar, and a      |
|                            | search of the list, with a magnifying glass                           |
| Info bars                  | One of each severity, of which the caution closes, and comes back     |
| Canvas                     | The type ramp, the icons, the theme's colors, lines, circles and      |
|                            | rings, and cards that float, which never change while the theme       |
|                            | doesn't, so their instances are kept                                  |
| List                       | 120 items that scroll by pixels, of which the search shows those it   |
|                            | matches, one can be selected, and a double-click opens one. Each row  |
|                            | has a tooltip                                                         |
| Interface                  | What the input is on: the cursor, the widget under it, the one with   |
|                            | the press, the one with focus, and the open popup. The keys held      |
|                            | down, and the gamepad's sticks, triggers and buttons. And a log of    |
|                            | what changed                                                          |

The tabs, the tooltips, the canvas and the list take the 71 more pixels of borderless
fullscreen.

## Input

| Input              | Action                                                                     |
| ------------------ | -------------------------------------------------------------------------- |
| A press            | Presses a button, flips a checkbox, a switch or a chip, picks a choice,    |
|                    | sets a slider, opens a combo box, or puts a text field's caret there       |
| A double-click     | Selects a word in a text field, or opens a row of the list                 |
| Drag a slider      | Moves it, even past the window's edges                                     |
| Drag in a field    | Selects text                                                               |
| The wheel          | Moves a slider a step a notch, or scrolls the list 3 rows                  |
| `Tab`, `Shift+Tab` | Moves keyboard focus to the next or previous widget                        |
| `Space`, `Enter`   | Presses the button with focus, or flips the checkbox, switch or chip       |
| Arrow keys         | Move the slider, radio buttons, segments, tabs, combo box or list with     |
|                    | focus, or a text field's caret, which Shift selects with, and Control      |
|                    | moves by words                                                             |
| `Ctrl+A`, `C`, `X` | Select everything in a text field, copy it, or cut it                      |
| `Ctrl+V`           | Pastes into a text field                                                   |
| `Esc`              | Closes a combo box's list, or else takes focus away                        |
| `F1`               | Shows or hides the tooltips                                                |
| `F2`               | Switches between the light and dark themes                                 |
| `Ctrl+R`           | Resets every widget                                                        |
| `F11`              | Switches between maximized and borderless fullscreen                       |
| Held keys          | Show in the Interface card, whatever has keyboard focus                    |
| A gamepad          | Shows in the Interface card: its sticks, triggers and buttons              |

The first press on the window while it doesn't have focus only gives it focus.

## Build and run

```powershell
cargo run --release
```

Or run `HumanRun.cmd`, which you can also double-click in File Explorer. It opens the showcase on
the primary display. `--display` and `--no-activate` work as in every program, and
[RunInBackground](../RunInBackground/README.md) runs it for coding agents. `Settings.json` has
the settings that every program has.

## Testing

```powershell
cargo test -p ui_showcase_tests
Program\RunInBackground\RunInBackground.ps1 UiShowcase --all-scripts
```

The tests in `Test/UiShowcase` drive the showcase through its cards' real layout, with
`user_interface::headless`, which lays them out without a window or a GPU: presses on each kind
of widget, a drag past a slider's end, the combo boxes' lists, typing, the search and its clear
button, the list's rows and a double-click, the caution info bar, tooltips, `Tab` through every
widget that takes focus, `Ctrl+R`, `F11`, and a window without focus. They also check that the
cards fit both window heights, in both themes.

The scripts in `Test/UiShowcase/Scripts` run the same kinds of input through the real window and
renderer, as [Library/Application](../../Library/Application/README.md#scripts) describes,
pressing widgets by name. The state that scripts save and check has what each widget shows, like
`Volume` and `Selected`, the keys held and the gamepad, as `HeldKeys` and `Gamepad`, the `Log`'s
newest lines, and the interface's, as `Interface`, with where each widget is, and the theme. A
script copies and pastes on the interface's own clipboard, never Windows'.

## Layout

| Path                 | Contents                                                             |
| -------------------- | -------------------------------------------------------------------- |
| `Source/Main.rs`     | Runs the showcase with Library/UserInterface's main loop             |
| `Source/Library.rs`  | The showcase's parts, as a library that `Test/UiShowcase` tests      |
| `Source/Showcase.rs` | What each widget shows, the list's items, the log, and the shortcuts |
| `Source/Panels.rs`   | Lays out the cards in fixed pixels, and draws their widgets          |
| `HumanRun.ps1`       | Builds the showcase and runs it on the primary display, for people   |
| `HumanRun.cmd`       | Runs `HumanRun.ps1`, for double-clicking in File Explorer            |
| `Settings.json`      | The settings that every program has                                  |
