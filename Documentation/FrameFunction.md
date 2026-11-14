# The frame function

> **Proposed.** None of this is built. The section "Where KGames is today" describes the code as it
> is at commit d45657b. Every other section describes the proposed design and its open questions.
> Settle a question, or build a part of the design, and update this document in the same change.

In this design a KGames program is one function, which the layer under it calls once a frame:

```rust
pub trait Game {
    fn frame(&mut self, input: &FrameInput) -> FrameOutput;
}
```

It takes the frame's input, a list of events in the order they happened. It gives back what to
draw, a list of GPU commands, for a frame of 2560 × 1440. The layer under it holds everything messy
that it never sees: the window, focus, Windows' cursor, minimizing, GameInput, Direct3D 12 and
DXGI.

## Goals

- **One layer at a time.** Someone can work on the program, the platform, the renderer or a
  library without reasoning about the whole stack at once. Each layer has one boundary to know.
- **The program never deals with the cursor, focus or minimizing.** The platform turns each of
  them into input, or into no frame at all.
- **Tests without hardware, all in parallel.** A test feeds a program each frame's input and
  checks each frame's output. It needs no window, no GPU and no GameInput.
- **Any input, any check.** A test makes up whatever input it wants. It sees every command that a
  frame draws, and checks them however it wants.

## The layers

| Layer      | Owns                                                                | Depends on                           |
| ---------- | ------------------------------------------------------------------- | ------------------------------------ |
| Program    | The game's or app's state, and `frame`                              | UI library, boundary, pure libraries |
| UI library | Widgets, layout, text layout, MVU and its steps                     | Boundary, DirectWrite                |
| Boundary   | `FrameInput`, `FrameOutput`, and the types in them                  | Nothing                              |
| Platform   | The window, focus, Windows' cursor, GameInput, the setup check, the | Boundary, renderer, WindowsApi,      |
|            | main loop, F10 and the PerfBar                                      | GameInput                            |
| Renderer   | What lives on the GPU, and turning commands into D3D12 work         | Boundary, Graphics, WindowsApi       |

The pure libraries are those that touch no hardware and no OS, like Math, Json, PolygonMesh,
Meshes, Gltf's parsing, and SimplePaint's materials and transforms. SimplePaint's `direct3d12`
module isn't one of them; it moves into the renderer.

The UI library changes place. Today it runs the main loop and owns the program. In this design it's
a library that a program's `frame` calls, above the boundary. The platform knows nothing of
widgets, messages or steps, and a game that wants none of them writes `frame` directly.

## The frame

```rust
pub struct FrameInput {
    // Seconds since the frame before, capped.
    pub delta: f64,
    // Open: seconds since the program started (see question 12).
    pub time: f64,
    // What happened since the frame before, in the order it happened.
    pub events: Vec<Event>,
}

pub struct FrameOutput {
    // What to draw, in order.
    pub commands: Vec<Command>,
    // What else the program asks of the platform, like to run a gamepad's
    // motors.
    pub requests: Vec<Request>,
    // Named areas, like each widget's, for tests and scripts to point at.
    // The platform ignores them.
    pub targets: Vec<Target>,
}
```

The frame's time is part of its input. AGENTS.md has simulations move by each frame's own time,
and only the platform's clock knows it. The platform caps the delta, so that a late frame, like
the first after the window was restored, doesn't carry things far. Today Veehiicuul caps its own,
with `LONGEST_FRAME`.

With one call a frame, a frame has one delta. AGENTS.md's rule that only a frame's last step has a
delta belongs to the UI library's steps, inside its `frame`, and a game that writes `frame`
directly never meets it.

The platform's loop:

1. Wait until the presenter can take a frame, with `Presenter::poll_ready`.
2. Read GameInput and the window's state, and turn them into the frame's events.
3. Call `frame`.
4. Carry out the requests, turn the commands into a command list, draw the PerfBar over the top
   17 rows, and present.

Input latency stays as it is. Input is read right before `frame`, as it's read right before
drawing now, and 2D drawing already goes through a list of instances (see below).

The law of the frame becomes a matter of structure. A frame's output can only come from the state
and that frame's input. There's one way left to break it, a request whose answer comes back as a
later frame's event, which question 4 is about.

## What the platform hides

| What                           | What the program sees                                                           |
| ------------------------------ | ------------------------------------------------------------------------------- |
| F1                             | Nothing: the platform keeps Windows' cursor, or lets it go (see question 13)    |
| F12                            | Nothing: the program quits at once                                              |
| Focus going, like with Alt+Tab | Releases of what was held, from GameInput's readings at rest; no event of focus |
| Focus coming                   | Nothing; keys held as it came count only once they're let go of, as today       |
| Alt+F4                         | Nothing but the keys, Alt and F4, which mean nothing to the platform            |
| Minimized                      | No call to `frame`; the next frame's delta is capped                            |
| A setup that's wrong           | No call to `frame`, and its input is dropped; the platform draws the error page |
| Windows' cursor                | Nothing: the platform confines it and hides it while F1 keeps it                |
| F1 having let the cursor go    | No mouse input; keyboards and gamepads still come                               |
| F10 and vertical sync          | Nothing                                                                         |
| The PerfBar                    | Nothing: it's drawn over the top 17 rows of the program's frame                 |
| The taskbar's Close            | Open: question 10                                                               |
| Windows' window shortcuts      | Nothing, as today                                                               |

The program never hears of focus. Windows does Alt+Tab before any window sees it, so it still
switches windows, but only the platform deals with that. As focus goes, GameInput's readings at
rest let go of every key and button that was held, so the program sees them released, as if the
user had let go.

The platform still checks focus on every pass of its loop, as it does today, which costs about
1.4 µs a pass. Focus goes in more ways than Alt+Tab: the Start menu, a notification that's
clicked, a UAC prompt, Win+L, or another program coming to the front. Once it's gone, F1 and F12
don't reach the platform either, as GameInput reads nothing without focus. Without the check, the
platform would confine the cursor again on every pass while another window is in front. With it,
the platform lets go of the cursor as focus goes. Ctrl+Alt+Delete, which no program can take,
stays the way out of a program that's stuck.

F1 and F12 are the platform's alone. The program is never sent them, neither as presses nor as
releases. That has costs today:

- F1 shows and hides the info card in MinimalSceneTest and SpinningCarTest
  ([info_card.rs](../Library/UserInterface/src/widgets/info_card.rs)), and starts GameInputDemo's
  rumble test. Both move to other keys.
- F12 quits without asking. MeshModeler asks first today, in `may_close`, when it has unsaved
  changes, so it would lose them unless it saves as it goes.
- F12 is Windows' key for debuggers. While a debugger is attached to the program, F12 breaks into
  it, as `RegisterHotKey`'s documentation says. Without one, F12 is an ordinary key.

## Where KGames is today

As of d45657b:

- `Program`, in [program.rs](../Library/UserInterface/src/application/program.rs), has `title`,
  `input`, `update`, `view`, `prepare`, `render`, `may_close`, `state` and `script`. The 420 lines
  of `user_interface::run`, in [run.rs](../Library/UserInterface/src/application/run.rs), are the
  main loop. The window, focus, Windows' cursor, GameInput's reader, the presenter, the setup
  check, the PerfBar, F9, F10 and Alt+F4 all meet there.
- 2D drawing is already a list of commands. Widgets and programs draw with a `Painter`, in
  [painter.rs](../Library/UserInterface/src/drawing/painter.rs). It turns shapes, shadows, glyphs,
  lines, arcs and polygons into `Instance`s of one quad, in layers, which the GPU draws in one
  call. That is most of `Command` already.
- The input is already a list of events. `Event`, in
  [events.rs](../Library/UserInterface/src/input/events.rs), has pointer moves, mouse buttons, the
  wheel, keys, text, focus and the gamepad, in order. That is most of `Event` already, except that
  the pointer arrives already moved (see question 5).
- `Headless`, in [headless.rs](../Library/UserInterface/src/testing/headless.rs), runs a program's
  frames with made-up input and no window. It uses the GPU only for the rendering tests.
- The leaks:
  - `Program::prepare` and `Program::render` hand a program `&mut Gpu` and the frame.
  - `Startup::devices` hands it GameInput's live devices.
  - MeshModeler, Platformer, SimplePaintDemo and Veehiicuul read and write files themselves.
- Picking stays on the CPU, so it needs nothing back from the GPU. MeshModeler, for example, casts
  a ray from its camera.
- The UI library depends on `graphics`, `game_input`, `windows_api`, `displays` and
  `present_timing`, so a program that uses it does too.

The 3D that a program draws itself, counted as names of D3D12's (`D3D12_*`, `ID3D12*`) in each
crate's sources:

| Crate                    | D3D12 names | What draws there                                            |
| ------------------------ | ----------- | ----------------------------------------------------------- |
| Library/SimplePaint      | 40          | Its `direct3d12` module: the pipeline, and meshes           |
| Program/SimplePaintDemo  | 34          | `preview.rs`, its preview of SimplePaint                    |
| Program/MeshModeler      | 30          | `viewport.rs`: the surface, and an overlay pipeline its own |
| Program/Veehiicuul       | 7           | The world, through SimplePaint                              |
| Program/SpinningCarTest  | 1           | Through SimplePaint                                         |
| Program/MinimalSceneTest | 1           | Through SimplePaint                                         |
| Every other program      | 0           | Only the UI library's 2D                                    |

## Decisions

### Settled

- A program is one function a frame, `frame`, from the frame's input events to GPU commands.
- It draws a frame of 2560 × 1440, the only setup that programs support (see AGENTS.md).
- `FrameInput` carries the mouse, the keyboard and the gamepads, and the frame's delta. What else
  it carries is question 12.
- No typed text in `FrameInput`. The keyboard is its keys going down and up, as GameInput reads
  them.
- The UI library's text boxes and number boxes make ASCII of keys themselves, with a fixed table
  of a US keyboard, which needs nothing of Windows, so it replays the same anywhere. The table
  covers the letters, the digits, space, the 11 keys of punctuation, each with its form under
  Shift, and the numpad's digits and `+ - * / .`. It has no Caps Lock, no accents and no key
  repeat. It replaces today's `ToUnicodeEx` with Windows' layout and its Caps Lock.
- The mouse is each mouse's raw motion, as GameInput counts it, its buttons and its wheel. The
  pointer is the program's: the UI library moves it, with `MOUSE_SENSITIVITY`, keeps it in the
  frame, and draws it.
- While F1 has let the cursor go, the platform sends no mouse input, so the pointer stays where it
  was while Windows' arrow moves. As F1 lets go, Windows' arrow shows where Windows had it, not at
  the pointer, as the platform doesn't know where the pointer is. Today it's placed at the
  pointer, SDL's way.
- F12 quits, and F1 keeps Windows' cursor or lets it go. Both are the platform's, and the program
  is never sent anything of F1 or F12.
- Alt+F4 and Alt+Tab mean nothing to the program. There are no events of focus or of closing.
- Windows' cursor, focus and minimizing are the platform's, under the boundary. The platform
  checks focus on every pass, as today.

### Open

Each question has the options, and the one that's recommended, with why.

**1. How high-level the commands are.**

- D3D12's own commands. They leak the hardware, and there's nothing in them for a test to check
  the meaning of.
- A small GPU API of KGames' own: buffers, textures, pipelines, shaders and draws, like WebGPU's.
  Any effect is possible, but programs write shaders, and tests check draw calls.
- A closed set of high-level commands:
  - today's 2D kinds;
  - images;
  - 3D as meshes, each with a material and a transform, seen from a camera, in SimplePaint's style;
  - lines over the 3D, for MeshModeler's overlay.

Recommended: the closed set. A new kind of drawing becomes a command added to the renderer and
tested in the renderer's tests, the way a new control becomes a widget in the UI library. The cost
is that MeshModeler's overlay pipeline and SimplePaintDemo's preview move into the renderer.

**2. What outlives a frame.** Meshes, glTF models and textures can't be sent again every frame.
The options:

- Send everything every frame. That's too slow for Veehiicuul's models.
- Let the renderer cache by content. That's hidden state, and it hashes everything every frame.
- Have the program create, update and free them with commands, under ids that it chooses.

Recommended: the program's ids. They're explicit, and tests can check that a mesh is uploaded once
and that no command uses a freed one. The glyph atlas stays the renderer's own: a glyph command
names a font face and a glyph, and the renderer rasterizes each glyph when it first meets it.

**3. Text.** Laying out text needs to measure it, which DirectWrite does. DirectWrite runs on the
CPU, needs no GPU, and gives the same answer every time for the same fonts. So the UI library
keeps it above the boundary, as the tests already use it. Recommended: DirectWrite is the one
Windows API above the boundary, in a crate of its own that question 6's check allows by name.

**4. What else a program asks for, and the law of the frame.** These requests carry no answer:

- running a gamepad's motors, for GameInputDemo;
- the theme, so that the platform's error page, PerfBar and background before the first frame
  match F9, which stays the program's.

A request whose answer comes back as a later frame's event delays an effect by a frame, which
AGENTS.md calls a bug. Files are the case in point: four programs read and write them today. The
options:

- Saves as requests, and loads only at startup.
- A `Files` interface that `frame` calls, and that answers at once. Tests replace it with files in
  memory.

Recommended: `Files`, passed to `frame` beside its input, as
`frame(&mut self, input: &FrameInput, files: &mut dyn Files)`.

**5. Who owns the pointer.** Decided: the program, from each mouse's raw motion (see Settled).
Today GameInput's reader, under the program, moves the pointer and keeps it in the window, and
events carry its position.

**6. Keeping the boundary.** By AGENTS.md, a rule holds only if a machine check enforces it. So the
crates split: the boundary's types, the crates above it, and the crates under it. A test reads
`cargo metadata`, and fails if a crate above the boundary depends on `graphics`, `game_input` or
`displays`, whether directly or not, or reaches `windows_api` other than through the text crate.

Cargo's dependencies belong to a package, not to a target. A program's `main.rs` calls the
platform, so it moves to a package of its own, or the check reads the library's sources instead.
Recommended: a package of its own, since it's then Cargo, not a scan of the sources, that says what
the program's library may use.

**7. `&mut self` or a pure function.** A pure `(state, input) -> (state, output)` makes purity a
matter of type. But it copies or rebuilds big states every frame, like Veehiicuul's terrain and
MeshModeler's mesh and history.

Recommended: `&mut self`, with determinism as a test: the same inputs from the same start give the
same outputs, run twice. The program follows four rules:

- no clock but `FrameInput`'s;
- no randomness that isn't seeded from its state;
- no `HashMap` iteration order that reaches the output;
- no thread whose timing reaches the output.

**8. The PerfBar.** Either the program draws 2560 × 1440 and the platform draws the PerfBar over its
top 17 rows, or the program gets 2560 × 1423. Recommended: 2560 × 1440 with the PerfBar over it.
That's what a frame of 2560 × 1440 means, and AGENTS.md lets a 3D view fill the window. Widgets
stay below row 17, as `content_top` keeps them today.

**9. GameInputDemo.** It shows devices and their raw readings, and runs their motors. It fits if
events can carry device connections and readings as GameInput gave them, and requests can run
motors. Its tests then make up devices without GameInput, as its scripts do now. Recommended: yes.
Only programs that ask for these events get them, as only those that ask get the gamepad today,
with `Options::gamepad`.

**10. The taskbar's Close.** F12 quits at once, even on the error page. The taskbar's Close sends
`WM_CLOSE`, which can come while the window is minimized. The options:

- Quit at once, as F12 does.
- Ignore it, so that only F12 quits. Task Manager can still end the program.

Recommended: quit at once. No frame has to run, so it works the same while minimized and on the
error page. With it, the decision on closing while no frame runs, which LogicReview.md left to the
user, goes away.

**11. Which program comes first.** Recommended: Platformer. It has no D3D12, it is a game rather
than a form, and it reads and writes a file of its own, in `progress.rs`, which tries out `Files`.

**12. What else `FrameInput` carries.** Past the mouse, the keyboard, the gamepads and the delta,
these are left.

The four can't say these, and each would have been an event of its own. Decided: none of them,
for now.

- **Typed text.** A character isn't a key. It comes from the key, Windows' keyboard layout,
  Shift, Caps Lock, and dead keys, like the acute accent's key and then e for é. Without it, the
  UI library's text boxes and number boxes, which UiShowcase, MeshModeler and Veehiicuul's vehicle
  editor use, make ASCII of keys themselves (see Settled).
- **Close requested.** F12 quits instead, under the boundary.
- **Focus lost and gained.** GameInput's readings at rest let go of everything held as focus goes,
  so programs are correct without it. A game can't pause on Alt+Tab, though, since it can't tell
  that from no one touching anything. UiShowcase and GameInputDemo show focus today.

Programs read these today, but they could go. Recommended: leave them out.

- **Time since the program started.** `ui.time()` has 9 uses, like UiShowcase's progress,
  Platformer's clock and SpinningCarTest's tick. A program can sum the deltas, and the sum leaves
  out the time spent minimized, which is what a game wants. The delta is then an `f64`, so that a
  sum over hours doesn't drift.
- **What the platform knows of itself:** the GPU's name, the display, vertical sync, and the
  frame's statistics and input latency.
  - Today UiShowcase's, SimplePaintDemo's and GameInputDemo's cards show them, and MeshModeler has
    a menu item for vertical sync.
  - They differ from run to run, so a program that shows them can't be replayed exactly.
  - The PerfBar shows vertical sync and the input latency already, and F10 switches vertical sync.
- **Each event's time.** Events keep their order, which is all that anything needs today.
  GameInput stamps each reading, if a game ever needs time within a frame, like a rhythm game.
- **Raw devices,** for GameInputDemo: each device's connection, name and identifiers, and every
  reading as GameInput gave it. It's the only program that needs them. Decide when it's converted,
  as question 9 says.

These aren't `FrameInput`'s at all:

- `Settings.json`, the command line and the data folder: the program gets them once, as it's
  created.
- Files: `Files`, from question 4, as their answers come in the same frame.
- Devices that scripts make up: just events, made up.

What's still open inside the four:

- **The mice:** all of them as one, as today, which is recommended, or each on its own.
- **The keyboard:** whether F10, the platform's, leaves the stream, as F1 and F12 do, so that every
  key has one owner. Keys held as focus came stay the platform's to hold back, as today.
- **The gamepads:**
  - all of them as one, as `Ui::gamepad` has them today, or each on its own, which local
    multiplayer needs;
  - every reading, about 77 a frame from an 8,000 Hz gamepad at 100 frames a second, or only each
    change of the buttons and the sticks as the frame ends.
- **The delta:** 0 on the first frame, and capped after a long one. Veehiicuul caps it at 0.1
  seconds today.

**13. What F1 does, exactly.** F1 keeps Windows' cursor, confined and hidden, or lets it go. What's
left to decide:

- **At start.** Kept from when the window first has focus, or let go until the first F1.
  Recommended: kept, so that a program is ready to use as it opens.
- **As focus goes and comes.** Either focus going lets go of the cursor without changing what F1
  last chose, and focus coming keeps it again if F1 chose that; or focus going counts as F1, so
  that coming back needs F1. Recommended: the first, which is today's behavior. Windows lets go of
  a confinement when most windows come to the front, but not the Start menu, so the platform lets
  go itself either way.

While it's let go, the program gets no mouse input (see Settled). When F1 keeps the cursor again,
the pointer carries on from where it was, as nothing moved it.

## Replay

If `frame` is deterministic, recording every `FrameInput` of a real run and feeding it back
reproduces the run exactly, in a test. A bug found by hand becomes the test that pins its fix, as
AGENTS.md asks of every fix.

Scripts in `Test/<program>/Scripts` become lists of frames' input, written by hand or recorded.
They keep pointing at widgets by name, through the output's targets. RunInBackground's runs of the
real program remain only for what's under the boundary.

## Testing

| Layer      | What its tests do                                                       | GPU  |
| ---------- | ----------------------------------------------------------------------- | ---- |
| Program    | Feed frames' input; check the state, the targets and the commands       | No   |
| UI library | The widgets' behavior and layout, through test programs                 | No   |
| Renderer   | Commands to pixels, offscreen, against reference images                 | Yes  |
| Platform   | GameInput to events, focus and the cursor; real runs in RunInBackground | Some |

Comparing exact lists of commands breaks every test that draws whenever a padding token changes.
So tests check commands in these ways:

- **Invariants, over every frame of every test.** Nothing is drawn outside its clip, no command
  uses a freed resource, and every resource uploaded is used.
- **Targets.** Tests and scripts point at, and check, the named targets that the output carries,
  as they do with `Interface.Targets` today.
- **Golden lists.** A few frames' commands are compared in full.
- **Pixels.** Only the renderer's tests look at pixels.

The tests that need no GPU run all in parallel, and never meet the DXGI factory's race that the
rendering tests met (see LogicReview.md).

## Prior art

- Casey Muratori's Handmade Hero splits a platform layer from a game layer. The platform calls the
  game once a frame with the frame's input, and the game hands back what to draw. Its looped
  recording of input replays a session.
- "Functional core, imperative shell", from Gary Bernhardt's talk "Boundaries".
- KGames' own MVU, which [UiArchitectures.md](UiArchitectures.md) compares. It already separates
  `update` from `view`. This design moves the boundary from inside the library's loop to under the
  whole program.

## Getting there

Each step is a series of commits of its own, and every test and script passes after each one:

1. **The boundary's types,** in a crate of their own. The platform's loop calls an adapter that
   runs a `Program` through today's interface as `frame`, so no program changes.
2. **2D through commands.** The painter's instances become commands, which the renderer draws from
   `FrameOutput`.
3. **3D through commands:** SimplePaint's meshes and materials, and MeshModeler's overlay. Then
   `prepare`, `render` and `Gpu` leave `Program`.
4. **Devices and files.** `Startup::devices` becomes events and requests, and files go through
   `Files`.
5. **The pointer and typed text move above the boundary.** The UI library moves the pointer from
   each mouse's raw motion, and makes ASCII of keys.
6. **The crates split,** and the dependency check comes in.
7. **The first program written against `frame`** directly.
8. **The tests move to frames.** `Headless` becomes a loop over `frame`.
9. **AGENTS.md's section on GUI programs, and the UI library's README,** change in the same commits
   that would make them false.
10. **An adversarial review** of the new core, with its findings pinned by tests and recorded as
    LogicReview.md records the last one. This rebuilds a library core, which AGENTS.md calls a
    rare, deliberate event.
