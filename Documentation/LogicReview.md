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

# What the rewrite in C++ found

KGames was rewritten in C++ once, file for file, to compare the two languages.
[CppVersusRust.md](CppVersusRust.md) has the comparison, and the C++ is on the `cpp-rewrite`
branch, whose copy of this document records the three adversarial reviews that the rewrite ended
with. Those read the C++ against the Rust, so what they found in the logic, and not in how C++
wrote it, was the Rust's too. This records what the Rust took from them, as above. Each fix has a
test that pins it.

## Found and fixed

- MeshModeler: an Undo or a Redo that took away the paint that was picked left it picked until the
  frame's end, so a second message in the same frame, an edit of the paint or a stroke with it,
  indexed past the paints and stopped the program. So did a pick of a row whose paint an Undo
  earlier in the frame took away. The pick is now kept among the paints as each of them applies.
- MeshModeler: the view drew the model it drew before after New or Open, until an edit, when the
  new document's version number was the old one's. A document counts its versions from 1, so that
  was any model opened before the first edit. What the view drew is forgotten with the document.
- `ListView` took a selection past its rows, which a program has when its list got shorter, as
  selected: it asked the program's labels for that row, which stops a program that indexes,
  activated it on Enter, and moved to the row before it on Up. It's no selection now.
- Veehiicuul: a point far enough away, or any point on a terrain of tiny cells, was in a cell at
  an `i32`'s end. The brush's and the fill's steps to the cells around it added past the end, and
  Save wrote every cell between a terrain's first and last, which for cells at both ends is rows
  that no disk holds. Cells now stop 1,024 from the origin: a point farther away is in the
  farthest cell, a brush that reaches past it leaves those cells unpainted, and a file with cells
  past it is refused.
- Library/PolygonMesh's merge by distance numbered the cube of a vertex far away with the most
  that an `i64` holds, and added one to it for the cubes around. GameInputDemo's scripted mouse
  added to running totals, and took differences of them, past an `i64`'s ends. Each stops a debug
  build and goes around in an optimized one. The cubes are clamped, and the totals wrap.

## Still open, deliberately

- A menu wider than its window, a menu's highlight past its items, and `DrawList::move_to_back`
  with marks out of order stop the program, in both languages. None can happen in the two window
  setups that programs support, and each is a mistake of the code that calls, which a stop says
  best.
- A program started with a show command of its launcher's, as Explorer and PowerShell's
  `Start-Process` give one, comes up at the maximized size without being maximized, and shows the
  error for an unsupported setup. Windows uses the launcher's command in place of the program's
  own for a process's first `ShowWindow`. `HumanRun` starts programs without one. It was found
  while measuring the programs in their windows, and fixing it needs a run that takes the focus,
  which only a person may start.