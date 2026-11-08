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
