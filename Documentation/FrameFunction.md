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
    // Seconds since the program started, and since the frame before, capped.
    pub time: f64,
    pub delta: f32,
    // What happened since the frame before, in the order it happened.
    pub events: Vec<Event>,
}

pub struct FrameOutput {
    // What to draw, in order.
    pub commands: Vec<Command>,
    // What else the program asks of the platform, like to close.
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
| Focus going                    | A release for everything held, then `FocusLost`; no events until focus is back  |
| Focus coming                   | `FocusGained`; keys held as it came count only once they're let go of, as today |
| Minimized                      | No call to `frame`; the next frame's delta is capped                            |
| A setup that's wrong           | No call to `frame`; the platform draws the error page                           |
| Windows' cursor                | Nothing: the platform confines it, hides it, and places it on release           |
| F10 and vertical sync          | Nothing                                                                         |
| The PerfBar                    | Nothing: it's drawn over the top 17 rows of the program's frame                 |
| Alt+F4 and the taskbar's Close | `CloseRequested`; the program closes with a `Close` request                     |
| Windows' window shortcuts      | Nothing, as today                                                               |

`FocusLost` stays an event so that a game can pause on it. The releases before it mean that a
program doesn't need to follow focus to let go of what was held.

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
- Windows' cursor, focus, minimizing, Alt+F4 and `WM_CLOSE` work as they do today, from under the
  boundary.

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

- `Close`;
- running a gamepad's motors, for GameInputDemo;
- the theme, so that the platform's error page, PerfBar and background before the first frame
  match F9, which stays the program's;
- where the pointer is, if question 5 makes the pointer the program's.

A request whose answer comes back as a later frame's event delays an effect by a frame, which
AGENTS.md calls a bug. Files are the case in point: four programs read and write them today. The
options:

- Saves as requests, and loads only at startup.
- A `Files` interface that `frame` calls, and that answers at once. Tests replace it with files in
  memory.

Recommended: `Files`, passed to `frame` beside its input, as
`frame(&mut self, input: &FrameInput, files: &mut dyn Files)`.

**5. Who owns the pointer.** Today GameInput's reader, under the program, moves the pointer by
each mouse's motion times `MOUSE_SENSITIVITY` and keeps it in the window. Events then carry its
position. The options:

- Keep it as it is: events carry the pointer's position.
- Events carry each mouse's raw motion, and the UI library owns the pointer. Its sensitivity,
  where it starts, keeping it in the frame, and drawing it all move up. The output says where the
  pointer is, and the platform places Windows' cursor there when focus goes.

Recommended: raw motion. A game that turns a camera wants a mouse's motion, not a pointer kept in
the window, and the platform gets simpler. The cost is the pointer's position in every output.

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

**10. Closing while no frame runs.** While the window is minimized, or the error page shows,
`frame` isn't called, so a program can't answer `CloseRequested` with a dialog. Either the event
waits until frames run again, which is roughly today's behavior, or the platform closes without
asking. LogicReview.md left this to the user; record the answer here.

**11. Which program comes first.** Recommended: Platformer. It has no D3D12, it is a game rather
than a form, and it reads and writes a file of its own, in `progress.rs`, which tries out `Files`.

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

- **Invariants, over every frame of every test.** Nothing is drawn outside its clip, the pointer
  is drawn last, no command uses a freed resource, and every resource uploaded is used.
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
5. **The pointer** moves above the boundary, if question 5 says so.
6. **The crates split,** and the dependency check comes in.
7. **The first program written against `frame`** directly.
8. **The tests move to frames.** `Headless` becomes a loop over `frame`.
9. **AGENTS.md's section on GUI programs, and the UI library's README,** change in the same commits
   that would make them false.
10. **An adversarial review** of the new core, with its findings pinned by tests and recorded as
    LogicReview.md records the last one. This rebuilds a library core, which AGENTS.md calls a
    rare, deliberate event.
