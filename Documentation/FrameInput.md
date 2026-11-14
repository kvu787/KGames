# The frame's input

> **Proposed.** None of this is built. What it says of today is as of commit d45657b. Settle a
> question, or build a part of it, and update this document in the same change.

A frame's input becomes one value, `FrameInput`: the frame's delta, and its events in the order
they happened, from the keyboards, the mice and the gamepads. The platform, under the UI library,
makes it from GameInput's readings, and the UI library and programs read only it. Focus, Windows'
cursor, F1, F12 and minimizing stay under it, and programs never hear of them.

This change is only the input. Programs still describe their interface with `view`, apply
messages with `update`, and draw their 3D with `render`, as they do today.

## `FrameInput`

```rust
pub struct FrameInput {
    // Seconds since the frame before.
    pub delta: f64,
    // What happened since the frame before, in the order it happened.
    pub events: Vec<Event>,
}

pub enum Event {
    KeyDown(Key),
    KeyUp(Key),
    // The mice's motion, in GameInput's counts, before MOUSE_SENSITIVITY.
    // A real mouse moves by whole counts; a test may move by fractions, to
    // land the pointer on a point.
    MouseMoved { x: f64, y: f64 },
    MouseButtonDown(MouseButton),
    MouseButtonUp(MouseButton),
    // In notches of the wheel, +y away from the user, +x to the right.
    Wheel { x: f32, y: f32 },
    // All the gamepads at once, as today.
    Gamepad(Gamepad),
}
```

| In `FrameInput`                                   | Not in it, and where it is instead                             |
| ------------------------------------------------- | -------------------------------------------------------------- |
| Keys going down and up, as GameInput reads them   | Typed text: the UI library makes ASCII of keys                 |
| The mice's motion, buttons and wheels, all as one | The pointer: the UI library moves it, from the mice's motion   |
| The gamepads, all as one, as today                | Focus: the platform's alone                                    |
| The frame's delta                                 | Closing: F12 quits, and it's the platform's                    |
|                                                   | F1 and F12: never sent, neither as presses nor as releases     |
|                                                   | Time since the program started: the UI library sums the deltas |

Today's events that go: `PointerMoved`, `Text`, `FocusGained` and `FocusLost`.

## The platform

- **F1** keeps Windows' cursor, confined to the window and hidden, or lets it go. While it's let
  go, `FrameInput` has no mouse events, so that the pointer doesn't move apart from Windows'
  arrow. Keyboards and gamepads still come.
- **F12** quits at once, without asking.
- **Focus** is checked on every pass of the main loop, as today, which costs about 1.4 µs a pass.
  As focus goes, the platform lets go of the cursor, and puts in a `KeyUp` or `MouseButtonUp` for
  everything still held, and the gamepads at rest, so that nothing stays held. Keys and gamepad
  buttons held as focus comes count only once they're let go of, as today. The program never
  hears of focus.
- **Alt+F4** is two keys, which mean nothing. **Alt+Tab** is Windows', which switches windows
  before any window sees it.
- **Without focus** GameInput reads nothing, so F1 and F12 don't work either. Focus goes in more
  ways than Alt+Tab, like the Start menu, a notification that's clicked, a UAC prompt or Win+L,
  which is why the platform still checks it. Ctrl+Alt+Delete, which no program can take, is the
  way out of a program that's stuck.
- **Minimized** and **a setup that's wrong** are as today: no frames while minimized, and the
  error page instead of the program, which gets no input. F12 quits from the error page too.

## The UI library

- **The pointer** moves by `MouseMoved` times `MOUSE_SENSITIVITY`, kept within the frame's
  2560 × 1440. There's no pointer until the first mouse event, which starts it in the middle, as
  today it starts in the middle the first time focus comes. While F1 has let Windows' cursor go,
  nothing moves it, and it's drawn where it was.
- **Typed text** is ASCII, made from keys with a fixed table of a US keyboard. The table covers:
  - the letters;
  - the digits;
  - space;
  - the 11 keys of punctuation, each with its form under Shift;
  - the numpad's digits and `+ - * / .`.

  It has no Caps Lock, no accents and no key repeat, and it needs nothing of Windows, so it
  types the same on any computer. It replaces the reader's `ToUnicodeEx`, with Windows' layout
  and its Caps Lock.
- **Steps** are as today: each event that does something is a step, and the frame's delta comes
  in its last step.
- **`Ui::time`** is the sum of the deltas, which is the time since the program started, as today.
- **`Ui::window_focused`** goes, with UiShowcase's row that shows it and the info card's line.

## What it costs

- F1 shows and hides the info card in MinimalSceneTest and SpinningCarTest
  ([info_card.rs](../Library/UserInterface/src/widgets/info_card.rs)), and starts GameInputDemo's
  rumble test. Both move to other keys.
- F12 quits without asking. MeshModeler asks first today, in `may_close`, when it has unsaved
  changes, so it would lose them.
- F12 is Windows' key for debuggers. While a debugger is attached to the program, F12 breaks into
  it, as `RegisterHotKey`'s documentation says. Without one, F12 is an ordinary key.
- As F1 lets Windows' cursor go, Windows' arrow shows where Windows had it, not at the pointer,
  as the platform doesn't know where the pointer is. Today it's placed at the pointer, SDL's way.
- GameInputDemo reads GameInput itself, outside `FrameInput`, and drops readings without focus, as
  AGENTS.md says. It goes on getting the window's focus with its devices, the one program that
  does.
- Tests and scripts that make up input change, as of d45657b:

  | What they use                                  | Uses             | What it becomes                                  |
  | ---------------------------------------------- | ---------------- | ------------------------------------------------ |
  | `Headless::type_text`                          | 24               | Keys                                             |
  | `Headless::focus`                              | 14               | Gone, or the platform's tests                    |
  | `Event::FocusGained`, `Event::FocusLost`       | 16               | Gone, or the platform's tests                    |
  | `Event::Text`                                  | 13               | Keys                                             |
  | `Event::PointerMoved`                          | 32               | `MouseMoved`, worked out to land on the point    |
  | Scripts' `Type` steps                          | 3 files          | Keys                                             |
  | Scripts' `Focus` steps                         | 4 files          | Gone                                             |

## Open

1. **The taskbar's Close,** which sends `WM_CLOSE`. Either it quits at once, as F12 does, and
   `Program::may_close` goes; or it asks the program first, as today. Recommended: quit at once,
   so that closing works one way, whether the window is minimized, on the error page or neither.
2. **F1 at start.** Either Windows' cursor is kept from when the window first has focus, or it's
   let go until the first F1. Recommended: kept, so that a program is ready to use as it opens.
3. **F1 as focus goes and comes.** Either focus going lets go of the cursor without changing what
   F1 last chose, and focus coming keeps it again if F1 chose that; or focus going counts as F1,
   so that coming back needs F1. Recommended: the first, which is today's behavior.

## Later, not in this change

The first proposal went further: a program as one function a frame, from `FrameInput` to a list of
GPU commands, with the renderer, resources under the program's ids, a `Files` interface, crates
split along the boundary with a test that checks their dependencies, and determinism, so that a
recorded run replays exactly. Commit 91dc273 has it in full, as `Documentation/FrameFunction.md`.
