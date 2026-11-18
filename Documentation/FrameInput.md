# The frame's input

> **Built**, in commits 854d36b to 9e69a59. Where the build went its own way from the design, the
> section "Where the build differs" says so, and "Changed since" says what changed after it. Keep
> this document true with the code.

A frame's input is one value, `FrameInput`: the frame's delta, and its events in the order they
happened, from the keyboards, the mice and the gamepads. The platform, under the UI library, makes
it from GameInput's readings, and the UI library and programs read only it. Focus, Windows' cursor,
F12 and minimizing stay under it, and programs never hear of them.

The change is only the input. Programs still describe their interface with `view`, apply messages
with `update`, and draw their 3D with `render`.

## `FrameInput`

In [events.rs](../Library/user-interface/src/input/events.rs):

```rust
pub struct FrameInput {
    // Seconds since the frame before.
    pub delta: f64,
    // What happened since the frame before, in the order it happened.
    pub events: Vec<Event>,
}

pub enum Event {
    // The mice's motion, in GameInput's counts, before MOUSE_SENSITIVITY.
    // A real mouse moves by whole counts; a test may move by parts of one,
    // to land the pointer on a point.
    MouseMoved { x: f64, y: f64 },
    // The pointer put at a point, as the mice's input starts: where Windows'
    // cursor was as the window started to keep it, or the middle if that was
    // outside the window. Not motion.
    PointerPlaced(Point),
    ButtonPressed(MouseButton),
    ButtonReleased(MouseButton),
    // In notches of the wheel, +y away from the user, +x to the right.
    Wheel(Point),
    KeyPressed(Key),
    KeyReleased(Key),
    // All the gamepads at once.
    Gamepad(Gamepad),
}
```

| In `FrameInput`                                   | Not in it, and where it is instead                             |
| ------------------------------------------------- | -------------------------------------------------------------- |
| Keys going down and up, as GameInput reads them   | Typed text: the UI library makes ASCII of keys                 |
| The mice's motion, buttons and wheels, all as one | The pointer: the UI library moves it, from the mice's motion   |
| The gamepads, all as one                          | Focus: the platform's alone                                    |
| The frame's delta                                 | Closing: F12 and the taskbar's Close quit, under the program   |
|                                                   | F12: never sent, neither as a press nor as a release           |
|                                                   | Time since the program started: the UI library sums the deltas |

The events that went: `PointerMoved`, `Text`, `FocusGained` and `FocusLost`.

## The platform

In [reader.rs](../Library/user-interface/src/input/reader.rs) and
[run.rs](../Library/user-interface/src/application/run.rs):

- **Windows' cursor** is kept, confined to the window and hidden, while the window has focus, so
  that a program is ready to use as it opens. While the window doesn't keep the cursor,
  `FrameInput` has no mouse events, so that the pointer doesn't move apart from Windows' arrow,
  and the buttons that the mice pressed are let go of as that starts, like while a click that was
  going on elsewhere as the window got focus is held, before the window keeps it. A click on the
  window that gives it focus keeps it as its button goes down. Keyboards and gamepads still come.
- **F12** quits at once, without asking, once the frame has had the input that came before it,
  like a Ctrl+S just before it, and nothing after it happens. So does the taskbar's Close, which
  sends `WM_CLOSE`, so that closing works one way, whether the window is minimized, on the error
  page or neither. `Program::may_close` is gone. A program can still close itself with
  `Context::close`.
- **F12 never reaches the interface.** The reader drops its presses and releases, the interface
  stops the program if a frame's events have it, whoever made them, and a script that presses it
  says so before it runs.
- **Focus** is checked on every pass of the main loop, which costs about 1.4 µs a pass. As focus
  goes, the platform lets go of the cursor, and the reader puts in a `KeyReleased` or
  `ButtonReleased` for each key or button whose press it gave, and the gamepads at rest, so that
  nothing stays held. Keys and gamepad buttons held as focus comes count only once they're let go
  of, and their releases are never given. The program never hears of focus.
- **Alt+F4** is two keys, which mean nothing. **Alt+Tab** is Windows', which switches windows
  before any window sees it.
- **Without focus** GameInput reads nothing, so F12 doesn't work either. Focus goes in more
  ways than Alt+Tab, like the Start menu, a notification that's clicked, a UAC prompt or Win+L,
  which is why the platform still checks it. Ctrl+Alt+Delete, which no program can take, is the
  way out of a program that's stuck.
- **Minimized** and **a setup that's wrong** are as before: no frames while minimized, and the
  error page instead of the program, which gets no input. F12 quits from the error page too.
- **The delta** is the time between the frames' reads, 0 for the first, and a script's
  `FrameTime` after its frame 0.

## The UI library

- **The pointer** moves by `MouseMoved` times `MOUSE_SENSITIVITY`, worked out in f64, kept within
  the frame's 2560 × 1440. There's no pointer until the mice's first event, which starts it in the
  middle. `PointerPlaced` puts it where Windows' cursor was as the window started to keep it, like
  as it gets focus, or in the middle if that was outside the window, without counting in
  `pointer_delta`. Tests and scripts point at places, which
  [gestures.rs](../Library/user-interface/src/testing/gestures.rs) turns into the motion that lands
  on them exactly. `Headless::queue_move` queues such a move among a test's own events.
- **Typed text** is ASCII, made from keys with a fixed table of a US keyboard, in
  [typing.rs](../Library/user-interface/src/input/typing.rs). The table covers:
  - the letters;
  - the digits;
  - space;
  - the 11 keys of punctuation, each with its form under Shift;
  - the numpad's digits and `+ - * / .`.

  It has no Caps Lock, no accents and no key repeat, and a key held with Ctrl or Alt types
  nothing. It needs nothing of Windows, so it types the same on any computer. It replaced the
  reader's `ToUnicodeEx`, with Windows' layout and its Caps Lock. A key's press and the text it
  types are one step.
- **Steps** are as before: each event that does something is a step, and the frame's delta comes
  in its last step. `Headless::frame_input` runs a frame of a test's own delta.
- **`Ui::time`** is the sum of the deltas, the time since the program started.
- **`Ui::window_focused`** is gone, with UiShowcase's row that showed it and the info card's line.

## Where the build differs

- **Windows' cursor is placed at the pointer as it's let go of**, SDL's way, as before. The design
  said it would show where Windows had it, as the platform wouldn't know where the pointer is, but
  the main loop reads the pointer from the interface, as it did before.
- **The pointer isn't drawn while the window doesn't keep the cursor.** The design said it would
  stay drawn where it was while F1 had let the cursor go. It isn't, as before, so that Windows'
  arrow is the only one that shows, and the pointer is away: nothing is under it, no tooltip
  shows, and the popups that were open close as it goes, as they did as focus went. The program
  still never hears of focus.
- **More programs used F1** than the design counted. UiShowcase's tooltips, Veehiicuul's and
  Platformer's cards, and the info card in MinimalSceneTest and SpinningCarTest moved to F2.
  GameInputDemo's rumble test moved to F3, as its F2 already cleared the log. They stayed there
  when F1 went.
- **GameInputDemo** gets the window's focus from its input source: `Devices::focused` and
  `Devices::focus_changes` for GameInput, and a `Focus` program step for scripts and tests, as
  the interface's `Focus` script step is gone.

## What it costs

- F12 and the taskbar's Close quit without asking. MeshModeler asked first, in `may_close`, when it
  had unsaved changes, and loses them now.
- F12 is Windows' key for debuggers. While a debugger is attached to the program, F12 breaks into
  it, as `RegisterHotKey`'s documentation says. Without one, F12 is an ordinary key.
- A button held as focus goes is let go of as the program sees it. Widgets act on a press, not a
  release, so a release only ends a drag.
- Only ASCII can be typed, in a US keyboard's layout, whatever Windows' layout is.

## Changed since

- **F1 went.** It let Windows' cursor go while the window kept focus, and kept it again. Alt+Tab
  and the Windows key already let it go, as they leave the window, and while F1 had let it go,
  Windows' arrow over the window clicked nothing, even after focus went and came back. Now
  Windows' cursor is kept while the window has focus, as above, F1 is a key like any other, and
  F12 is the platform's only key. The main loop checks focus once a pass, as it did before F1,
  without the checks around the frame that F1's presses needed.

## Later, not in this change

The first proposal went further: a program as one function a frame, from `FrameInput` to a list of
GPU commands, with the renderer, resources under the program's ids, a `Files` interface, crates
split along the boundary with a test that checks their dependencies, and determinism, so that a
recorded run replays exactly. Commit 91dc273 has it in full, as `Documentation/FrameFunction.md`.
