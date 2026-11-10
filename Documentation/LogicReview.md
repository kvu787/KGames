# The logic review

After the move to MVU, every program's domain logic - not the GUI - was reviewed end to end:
Veehiicuul's driving, collision, curves, track shaping, hexels, document and files; MeshModeler's
editing and history with PolygonMesh and SimplePaint under it; GameInputDemo's input model;
UiShowcase's model; the two scene tests' motion math; and Library/UserInterface's frame pipeline
itself, against the law that an output delayed any number of frames past its cause is a bug. This
records what was found and fixed, and what was examined and found sound. Each fix landed with the
code in its own commit, most with a test that pins it.

## Found and fixed

**Library/UserInterface's frame pipeline**

- A widget that lost keyboard focus was told only when a press moved the focus. Esc, another
  widget's press or focus call, and `Context::focus` all slipped past, so a number box's typed
  draft never committed on those paths - and NumberBox itself reset the draft before the commit
  could read it, so even the told path committed the old value. The interface now remembers every
  focus leaver and dispatch tells each one, in the same frame, with a second pass for widgets the
  loop had already passed.
- A dialog's default button fired on Enter meant for a focused widget inside the dialog. It now
  fires only when nothing reachable takes Enter. Combo boxes also never actually opened from the
  keyboard: the press that opened the list chose from it in the same interact and closed it again.
- F10's own frame drew the PerfBar with the old VSync label, though the frame's present already
  honored the new setting - a one-frame delay in the library's own bar.
- The wheel that nothing under the pointer takes went to every containing scrollable, so a list
  and the area around it scrolled together; and a disabled widget still ate the wheel. It now goes
  only to the deepest widget that scrolls, and a disabled one doesn't count.
- A popup outlived a widget that changed kind under a kept name, capturing the keys until Esc; a
  target that stopped being focusable kept focus; `target()` resolved a duplicated name to a
  different widget than `state()` described; and a message chain past the frame's cap silently
  dropped everything queued behind it. All four are closed; the cap now stops the program, as a
  cascade that doesn't settle is a bug.

**Veehiicuul**

- A crash was detected once a frame and its pause anchored to the frame clock, so a run through a
  crash diverged between 60 and 240 Hz. The car now crashes on the very drive step that touches
  the edge, and the pause runs on the drive's clock; a test drives a crash at both rates.
- Undo could strand the track builder's chosen curve past the islands that are left: adding an
  island, undoing it, and pressing Delete indexed out of bounds. The frame's last message now
  clamps the track's and the terrain's choices, as it already kept the scene's pick.
- Saving couldn't express a deletion: undoing a new scene after saving it left its file on disk
  and the status saying Unsaved forever. Save now deletes the files of scenes the document no
  longer has.
- Parked-vehicle names were compared with case, while `Vehicles::find` reads them without, so a
  hand-edited scene could dodge the delete guard or a rename and save a document that couldn't
  load.
- The track's problems now report an island inside an island, which the comment always promised.

**MeshModeler, PolygonMesh, SimplePaint**

- A keyboard command during a held Paint-tool stroke interleaved history steps: an Undo mid-stroke
  was silently re-applied by the stroke's own commit. A command now ends the stroke first, as its
  own step.
- `Document::save` wrote whatever the mesh held; positions driven to infinity serialized as JSON
  null and the saved file couldn't be reopened. Save now validates first and refuses loudly.
- A stale face index in the paint tool panics no more; a crease sharpness that underflows f32 is
  rejected at its own line instead of failing the whole file later; and a loop cut through an edge
  with more than two faces says no instead of leaving a T-junction.

**UiShowcase and GameInputDemo**

- The showcase's Watch message carried the theme it observed at input time - the stale-snapshot
  class - so a toggle earlier in the same frame made it double-log. It now reads the theme when it
  applies. The list's Select and Activate carried a row index resolved against apply-time search
  matches, so typing that re-filters the list in the same frame moved a press onto another item;
  they now carry the item the row showed, the gesture's own fact. A menu Reset kept its recorded
  choice. The input demo's combo-box pick logs at its frame's own time.

## Examined and sound

The reviews deliberately verified, and found correct: Veehiicuul's integration, dead zones,
rotation and collision geometry, B-spline sampling and insertion, hexel axial math, and the
camera's ground round trip; PolygonMesh's renumbering invariants, crease bookkeeping through
subdivision, Catmull-Clark's boundary and crease rules, picking's ray math, and file round trips;
SimplePaint's parameter validation and the impossibility of its divisions by zero; GameInputDemo's
focus masking, listen rules, timestamps and rumble; the scene tests' bounce and phase math; and
the frame pipeline's theme switching, popup layering, drag capture, and the fidelity of Headless
and scripts to the window's own loop.

## Still open, deliberately

- A second mouse button pressed on a view mid-drag overwrites the first press's tracking (the
  interface keeps one active press). Niche, and the fix would widen the interaction core; noted
  rather than taken.
- The reviews suggested more tests than were added: a property sweep of random mesh-edit sequences
  against `validate()`, a Headless test of GameInputDemo's focus-lost double update, and nested
  scrollables under the wheel. The fixes those would cover are in and pinned by their own tests;
  the sweeps remain good future hardening.

# The review of the port to C++

KGames was written in Rust when the review above was made. It was then ported to C++, file for
file, which rewrote every library core at once. [CppVersusRust.md](CppVersusRust.md) compares the
two. The port's own checks were the Rust's tests, ported one for one, and the programs' scripts,
whose frames and states were compared with the Rust's by their bytes. Those say that the C++ does
what the Rust did wherever a test or a script looks. They don't say what happens where Rust stops
the program, wraps a number or refuses to compile, and C++ leaves the result undefined. So the
port ended with three adversarial reviews of the C++ for exactly that: of Library/UserInterface's
core and widgets; of Library/Base, the window, input, text, graphics and present timing; and of
the programs and the data libraries. The reviewers read the C++ against the Rust, ran the tests
under AddressSanitizer, and drove UiShowcase and MeshModeler through `Headless` with random input
for tens of thousands of frames. This records what they found, as above. Each fix has a test that
pins it, but for the ones that say otherwise.

## Found and fixed

**Library/Base and the main loop**

- A panic on a thread other than the first stopped the program without a word: with MSVC, a
  handler set with `std::set_terminate` is the calling thread's alone. The reason is now written
  from a handler of the process's abort signal, whichever thread it's on.
- A panic didn't unwind. An exception that nothing catches ends the program where it's thrown,
  without running a destructor, so the session that times presents went on in Windows after the
  program. The main loop now catches at its top, so that everything it holds is destroyed, as in
  Rust, and then stops the program with the reason.
- In Debug, an error that the debug C runtime reports, like a write past the end of a block of the
  heap, went to a file and the program went on, so a test that corrupted the heap passed. It stops
  the program now.
- The functions that Windows calls back - the window procedure, GameInput's device callback,
  the `IDWriteTextRenderer` that collects a layout's glyphs, and the enumeration of displays -
  could let an exception unwind into Windows. They're `noexcept`, which stops the program there
  with its reason. Nothing pins this but the compiler.
- `base::is_char_boundary` said that the start of a text that begins inside a character isn't a
  boundary, where Rust's says that offset 0 always is. `base::parse_f64` sent some numbers too
  large or too small for a `double` to the wrong end. A `base::VecDeque` that grew while moving an
  item that threw lost its items.

**Library/UserInterface**

- A menu wider than its window clamped its place between bounds that cross, which `std::clamp`
  leaves undefined and Rust's `clamp` panics at. It stops the program now. So do a menu's
  highlight past its items and `DrawList::move_to_back` with marks out of order, which the port of
  the tests had found: each read or moved memory outside a vector where the Rust panicked.
- A slider's or a number box's value that isn't a number read `nan` or `-nan(ind)`, where the Rust
  writes `NaN`.
- Three `std::max` in the geometry gave a NaN back where Rust's `f32::max` gives the other number.
  They're `std::fmax`.

**The programs and the data libraries**

- MeshModeler: an Undo or a Redo that took away the paint that was picked left it picked until the
  frame's end, so a second message in the same frame - an edit of the paint, or a stroke with it -
  indexed past the paints. In Rust that stopped the program. In C++ it read and wrote past a
  vector. The pick is now kept among the paints right after the step.
- Library/Gltf's own check for UTF-8 took a character cut short at the end of the JSON for text.
  It's `base::is_utf8` now.
- Veehiicuul: a point far enough away, or any point on a terrain of tiny cells, was in a cell at an
  `int32`'s end, and the brush's and the fill's steps to the cells around it added past the end,
  which is undefined. Cells now stop 2^30 from the origin: a point farther away is in the farthest
  cell, a file with cells past it is refused, and a brush that reaches past it leaves those cells
  unpainted. The limit is the C++'s own. The Rust read any `i32`.
- Library/PolygonMesh's merge by distance numbered the cube of a vertex far away with the most
  that an `int64` holds, and added one to it for the cubes around. GameInputDemo's scripted mouse
  added to running totals, and took differences of them, past an `int64`'s ends. The cubes are
  clamped and the totals wrap, with `base::wrapping_add` and `base::wrapping_sub`. MSVC wraps both
  already, so their tests pass without the fixes too: they pin the results, not the defect.
- MinimalSceneTest's frame time went through `std::min`, which gives a NaN back where `f32::min`
  gives the longest frame, so one frame without a time made the sphere's position NaN for good.
- Veehiicuul's Save button was move-assigned from itself through its builder, which worked only by
  how the standard library's strings happen to move. It's made in one expression. No test pins it.

## Examined and sound

The reviewers hunted these through the C++ and, apart from the above, found them sound: views,
spans and references that outlive what they view, in members, in what functions return and in the
lambdas that widgets keep; references into a container across a call that can grow it, the
`Element` that `Tree::add` returns and the layout that `Fonts::layout` returns among them; calls
whose arguments have side effects that depend on their order, which C++ leaves open and MSVC
often runs right to left; the order that members are destroyed in, which is the reverse of
Rust's, for the window and what draws into it, the GPU's resources and what waits on them, and
the threads; uses of what was moved from; members without an initializer, and structs that go to
the GPU or to Windows; shifts, casts of floats to integers, subtractions of sizes, and divisions;
indexes that input, a file or a program using a library can reach; the present timing's and the
input devices' threads and their locks; and pointers that stand for an `Option`.

Two checks that the port itself made belong here. Numbers written as text: `base::display` and
`base::debug` write all of 1,248,746 floats and doubles, chosen at the edges of every format, as
Rust's `{}` and `{:?}` do, after `std::to_chars` alone disagreed on 159,270 of them. And every
frame and state that the 52 scripts of the eight scripted programs save is the Rust's, byte for
byte, but for the status line of 12 that has the run's folder in it.

## Still open, deliberately

- Three defects that the review found are the Rust's too, so the port kept them, to stay the same
  program: MeshModeler draws the model it drew before after New or Open, when the new document's
  version number is the old one's, until an edit; `ListView` can ask a program's labels function
  for a row past its rows, when the selection is; and a Veehiicuul terrain with cells at both ends
  of its range makes Save write a grid of everything between. Each wants its own fix and test.
- A list of two texts for a `std::span<const std::string_view>` is written `{{"A", "B"}}`
  throughout. With C++26's `std::span` constructor from an initializer list (P2447), the same
  characters would mean one text, from where "A" starts to where "B" does, and compile without a
  warning. MSVC's library doesn't have it yet. `Test/Base/Language.cpp` fails on the toolset that
  does, and says what to change then.
- Nothing here covers real devices, Windows' own input path, the user taking or giving focus, the
  clipboard, or `GameInputSource` and IdentifyDisplays' boxes, which no test or script runs.
