# Retained versus immediate mode

> This compares two versions of the library from when KGames was written in Rust. The names of
> files, the line counts and the measurements are those versions'. KGames is C++ now, as
> [CppVersusRust.md](CppVersusRust.md) says, with the architecture that
> [UiArchitectures.md](UiArchitectures.md) ends on.

KGames' UI library was immediate mode: every frame, each program rebuilt its
whole interface with `ui.add(name, rect, widget)`, widgets borrowed the
program's state as `&mut`, and the return value said what the input did. This
branch rewrote the library and all ten programs to retained mode: a tree of
named controls built once in `Program::build`, each owning its state, with
events out and setters in. Both versions pass the same test suites and input
scripts, and draw the same pixels, so the comparison below is between two
working implementations of the same programs, not between ideals.

## What was measured

UiShowcase's steady frame — build or sync plus every widget's input handling
and draw recording, no GPU — in release, on the same machine, through
RunLightly:

| Implementation | A steady frame of UiShowcase |
| -------------- | ---------------------------- |
| Immediate      | 0.192 ms                     |
| Retained       | 0.245 ms                     |

Retained mode here is about 28% slower per steady frame, not faster. The
reasons are structural: the controls still re-record their drawing every
frame (the renderer rebuilds its instance buffer and the glyph atlas can
evict, so caching draw commands per node would be a separate, risky project),
and on top of that the retained version walks the tree twice, allocates event
and context strings, and runs the program's `sync`, which formats the mirror
texts each frame. The classic retained win — skip clean subtrees — needs
dirty-tracked draw caching that neither version has; without it, retention
adds bookkeeping to the same per-frame work. Both are far under a 120 Hz
frame budget either way, and input latency is identical: both read input
right before the frame and show its effect in that frame.

## What the port actually changed, program by program

- **Code size.** The ports grew every program. UiShowcase's cards went from
  582 to 668 lines; GameInputDemo's from 737 to 831; MeshModeler's from 618
  to 729 plus a rewritten interaction file; Veehiicuul's editors grew by
  roughly a third. The growth is almost entirely the `build`/`events`/`sync`
  split: what immediate mode said once — lay out, react, display — retained
  mode says three times, in three places that must agree on names.
- **State lives twice.** Scripts and `Program::state` need the program's own
  copy of values, and so does anything the program computes from them. So the
  showcase keeps `volume`, `search`, `color` and the rest as fields, the
  controls keep them again, and `sync` reconciles the two every frame. In
  immediate mode there was one copy, borrowed.
- **Events replace return values.** `if ui.add("Save", …).clicked { … }`
  became a node in `build` plus a match arm in `update`. The event enum and
  the name-string matching are workable, but a typo in a name now fails at
  run time (a panic from `get_mut`, or an event nobody reads), where the
  borrow in immediate mode failed at compile time.
- **Dynamic structure needs a protocol.** The old code just didn't draw what
  wasn't there. The retained version needs visibility toggles for the easy
  cases (dialogs, tabs, the greeting bar), `set_items`/`set_rows` for lists
  and combos, and `request_rebuild()` for anything that moves other controls:
  the showcase's closable caution, MeshModeler's Open dialog growing a
  button, every Veehiicuul editor. Veehiicuul ended up comparing a
  fingerprint of its layout-affecting state every frame to know when to
  rebuild — which is immediate mode's "rebuild when anything changed",
  reinvented with extra steps, made correct only by the tree's adopt-by-name
  machinery preserving carets and scroll positions across rebuilds.
- **Same-frame behavior needed special cases.** The scripts and tests assume
  a click's effect is visible that frame. Keeping that true in retained mode
  took deliberate machinery: the three-phase frame, dispatch-in-draw for
  nodes shown during update, and re-registration for lists whose rows were
  swapped mid-frame. Immediate mode had all of this for free, by
  construction.
- **What got nicer.** Enabling and disabling by name
  (`ui.set_enabled("Undo", can_undo)`) reads better than scoped
  set/unset pairs. Visibility as a node flag suits dialogs. Controls owning
  their state removed the `&mut` plumbing from deep call trees, and
  `Canvas<T>` made the custom drawing (stick graphs, overlays, logs) honest
  about what data it reads. The `Control` trait also gives programs a real
  extension point: SimplePaintDemo's row faces and Veehiicuul's overlay are
  ordinary controls now.
- **What survived unchanged.** The hit-testing, focus, popup, tooltip and
  animation machinery, the renderer, the theme, `Headless`, and the script
  format — every script in the repository passed unmodified, except two test
  timing adjustments where state was mutated out-of-band. That's evidence the
  essential contract (named targets with rects, per-frame input semantics) is
  independent of which mode the program API uses.

## Pros and cons

**Retained mode**

- Pros: controls own their state, so there's no `&mut` threading; per-node
  enable/visible/rect setters; a natural home for per-control caches and, in
  principle, dirty-tracked drawing; a conventional model (WinUI, Qt) for
  tooling and accessibility trees; program-defined controls as first-class
  citizens.
- Cons: three places per control (build, events, sync) that agree only by
  name, checked at run time; duplicated state and the reconciliation code to
  keep it honest; dynamic interfaces degenerate into rebuild-on-fingerprint;
  same-frame feedback must be engineered; in this codebase it measured
  slower, because the draw recording it might have saved isn't cacheable
  under this renderer.

**Immediate mode**

- Pros: one code path per control, read top to bottom; state lives once, in
  the program, borrow-checked; dynamic structure is just an `if`; same-frame
  feedback by construction; measured faster here; smaller programs.
- Cons: the program pays full layout and draw recording every frame even
  when nothing changed (measured: 0.192 ms for the heaviest page, so this is
  theoretical at this scale); widget-internal state hides in a side table
  keyed by name rather than in a typed object; return-value plumbing gets
  awkward when one widget's result must affect an earlier one (the frame
  delay exists in both models, but immediate mode makes it less visible);
  no persistent object for tooling to inspect.

## Recommendation

**Stay with immediate mode for KGames.** Revert to it, or treat this branch
as the experiment it was; don't merge it as a replacement on its merits.

The deciding facts from doing the full rewrite:

1. The costs retained mode is meant to pay off — skipping work on clean
   frames — don't materialize here. The measured steady frame got 28% slower,
   and making it faster would need per-node draw caching that the atlas-based
   renderer makes genuinely hard. At 0.2 ms a frame for the heaviest page,
   there is no performance problem to solve.
2. KGames' programs are editors and demos over live simulations: most of
   what they show changes every frame (readings, speeds, stats, previews),
   and their structure follows their documents (picks, palettes, problems,
   tabs). That's the worst case for retained mode — the sync and fingerprint
   code is immediate mode re-implemented on top of a tree — and the best
   case for immediate mode.
3. The repository's tests and scripts drive the UI by named targets with
   same-frame semantics. Immediate mode satisfies that contract by
   construction; retained mode needed the three-phase frame, late dispatch
   and re-registration special cases to keep it, and each future control
   must remember them.
4. The real wins found along the way don't require retention. Worth keeping
   regardless of mode: `Canvas<T>`-style explicit data for custom painting,
   the `RadioGroup` grouping, and owned `MenuItem`s (the `const` borrowed
   menus were the one place immediate mode fought the borrow checker).

If a future program is mostly static forms over slow-changing data — a
settings app, not an editor — retained mode's trade-offs flip, and this
branch shows the library can host that model with the scripts and tests
intact. Until then, the immediate library is the smaller, faster and less
error-prone foundation for what KGames actually builds.

## Outcome

The recommendation was taken: the library and the programs returned to
immediate mode. The retained implementation stays in this branch's history,
ending at commit `00648f0`; commit `4b3e46f` restores the immediate
implementation, and the commits after it apply what the experiment taught:

- **The interaction types got a module of their own.** `Sense`, `Target`,
  `Response` and their kin are the contract that survived both modes
  unchanged, so they moved out of `Ui.rs` into `core/interaction.rs`.
- **Wheel interest is typed.** `hovered_takes_wheel` matched target kind
  strings; now a widget declares it in its `Sense` (`takes_wheel()`,
  `takes_wheel_while_focused()`), which the retained `Control` trait's
  explicitness suggested.
- **Enabling is scoped.** The per-name setters read better than the
  `set_enabled(x) ... set_enabled(true)` pairs; the immediate translation is
  `ui.add_enabled(enabled, ...)` for one widget and `ui.with_enabled` /
  `ui.with_focusable` for a run of them, which put the scope back themselves
  and can't leave a disabled card with a working widget.
- **`RadioGroup` was kept.** One `add` shows the whole group, with the same
  per-item targets the scripts already pointed at.
- **The one-frame modality gap closed.** A dialog now gates
  `ui.key_pressed`, `ui.key_pressed_once` and `ui.key_down` the frame it
  opens, for checks after it, not just from the next frame on.

With all of that applied, the showcase's steady frame measures 0.185 ms —
the same speed as the original immediate implementation's 0.192 ms, within
noise — and every workspace test and every program's scripts pass.

Two of the wins the recommendation listed were deliberately not carried
over, because they turned out to be retained-mode artifacts. `Canvas<T>`
earned its keep by making a tree node's reads explicit; an immediate
program's custom painting already takes its data as plain arguments, so the
wrapper adds nothing. Owned `MenuItem`s existed because a retained menu
outlives the frame that builds it; an immediate menu is shown the frame
it's built, so the borrowed `const` menus cost nothing and keep their
zero-allocation path.
