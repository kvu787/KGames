# Immediate, retained, MVU and callback view

KGames' UI library has now been written four ways, each complete: every program ported, every
workspace test and every input script passing, the same Fluent 2 pixels drawn by the same renderer.
This document compares the four implementations as built, not as ideals, the way
[RetainedVersusImmediate.md](RetainedVersusImmediate.md) compared the first two.

| Architecture  | Where it lives                                            | A program is                                                  |
| ------------- | --------------------------------------------------------- | ------------------------------------------------------------- |
| Immediate     | `main`'s history before the rewrite (measured at d86007e) | One `update` that adds each widget and reads what it returned |
| Retained      | The experiment ending at 00648f0                          | A tree built once, with events out and setters in             |
| MVU           | Branch `claude/mvu-ui`                                    | A `Message` enum, `input`, `update(message)`, and `view`      |
| Callback view | Branch `claude/callback-view-ui`                          | MVU whose messages are closures over the program (`Action`)   |

The MVU and callback branches share one library, identical to the line: callback view is
`Application/Callbacks.rs`, 161 lines over the message core, which makes `Action<P>` the message
type and writes `update` once. Only the programs differ between those two branches.

## The frame, in each architecture

All four read input right before building the frame, after the GPU can take one, and present what
they build, so none adds latency between a device and the screen. They differ in when input meets
state, and when state meets pixels.

- **Immediate**: one pass. `update` lays out a widget, the call returns what last frame's input did
  to that widget's area, and the program reacts before laying out the next. Reaction and
  description interleave; a widget earlier in the pass can't see a reaction later in it, which is
  invisible until it isn't (a title bar that lags a toggle by a frame, a card that resizes a frame
  after its content).
- **Retained**: three phases bolted onto a persistent tree - dispatch events, run the program's
  `sync`, draw - plus rebuild machinery for structure changes. Same-frame output had to be
  engineered case by case.
- **MVU**: two passes with a queue between them. Input is hit-tested against the tree the user saw;
  the widgets it landed on emit messages; `input` appends the program's own (shortcuts, a
  per-frame tick, a final prepare); `update` applies them in order, each seeing the state the last
  one left; then one read-only `view` describes the whole interface from the settled state, and
  layout and draw follow. Nothing drawn can predate a reaction, by construction.
- **Callback view**: the same two passes. The message is `Action<P>`, a shared closure, so the
  reaction is written beside the widget as in immediate mode, and `update` is the library's one
  line: run the closure.

The repository's rule that an output delayed any number of frames is a bug is satisfied differently:
immediate mode satisfies it per widget but lets order-of-layout exceptions slip in silently; MVU and
callback view satisfy it structurally, because every reaction runs before anything is described;
retained mode satisfied it only with special cases (dispatch-in-draw, re-registration), each a trap
for the next control.

## Performance

UiShowcase's steady frame - input handling, messages, view, layout and draw recording, no GPU - in
release, through RunLightly, 1000 frames after a 100-frame warmup. Two sessions, because absolute
numbers move with the machine's load; ratios between implementations held across both.

On an idle machine (each implementation measured in its own run):

| Implementation | Steady frame | Versus immediate |
| -------------- | ------------ | ---------------- |
| Immediate      | 0.118 ms     | 1.00x            |
| Retained       | (historical) | about 1.28x      |
| MVU            | 0.144 ms     | 1.22x            |
| Callback view  | 0.146 ms     | 1.24x            |

On a loaded machine (the three living implementations interleaved, best of six rounds each):

| Implementation | Steady frame | Versus immediate |
| -------------- | ------------ | ---------------- |
| Immediate      | 0.177 ms     | 1.00x            |
| MVU            | 0.217 ms     | 1.23x            |
| Callback view  | 0.201 ms     | 1.14x            |

MVU and callback view are indistinguishable from each other - each has measured faster than the
other in some session - and both cost roughly a fifth to a quarter over immediate mode. The frame's
phases say where (MVU, idle machine): view 0.026 ms, layout 0.016 ms, draw 0.092 ms, input and
update under 0.004 ms together. The draw pass dominates in every architecture, and is the same work
in all of them; the overhead is the separate view and layout walks over a tree that immediate mode
never materializes. All three build the same 2027 draw instances. The retained figure is from its
own session (0.245 ms against a contemporary immediate 0.192 ms); its slowdown came from walking a
persistent tree twice and syncing mirrored state, not from the queue the message designs added.

At 0.2 ms for the heaviest page against an 8.3 ms budget at 120 Hz, none of this chooses the
architecture. It only prices the structure: the one-frame guarantee by construction costs about
0.03 ms a frame here.

## Code size

Lines of each program's `Source`, same programs, same behavior, same scripts:

| Program          | Immediate | MVU    | Callback view | Retained       |
| ---------------- | --------- | ------ | ------------- | -------------- |
| UiShowcase       | 835       | 957    | 919           | grew about 15% |
| SimplePaintDemo  | 1,042     | 1,091  | 1,077         |                |
| MeshModeler      | 3,406     | 3,535  | 3,503         |                |
| GameInputDemo    | 3,189     | 3,254  | 3,247         |                |
| MinimalSceneTest | 362       | 387    | 372           |                |
| SpinningCarTest  | 471       | 500    | 483           |                |
| Veehiicuul       | 4,953     | 5,436  | 5,416         | grew about 33% |
| All seven        | 14,258    | 15,160 | 15,017        |                |

MVU costs about 6% over immediate across the programs, callback view about 5%, retained mode far
more (its growth was the build/events/sync split saying everything three times). Where the MVU
growth goes: the enum declarations themselves, and the editors' message-to-mutation plumbing -
Veehiicuul's four editors each gained a message enum and an `apply`. Callback view spends slightly
less because single-field reactions collapse to one closure, but spends it back wherever a reaction
needs borrows that closures make awkward.

The library grew from 13,931 lines (immediate) to 17,322 (messages): the view tree, the widget
contexts, reactions on every widget, and callback view. That's paid once; programs pay the 5-6%
each.

Annotation density differs more than size does. The MVU showcase names `Message::` 80 times; the
callback one writes `act`/`on`/`on_with` 47 times. Veehiicuul in MVU has 155 message-constructor
sites across five enums; in callback view, 60 closure sites plus the editor methods they call.
Neither is obviously lighter: MVU's sites are long names, callback's are closures with their borrow
ceremony (`let scene = game.game.scene;` before touching two fields at once).

## What the ports taught, dimension by dimension

- **State lives once in both message designs**, as it did in immediate mode. Widgets take values,
  not `&mut`: a slider shows `self.level` and sends the new value back as data. The retained
  design's mirrored state and `sync` have no counterpart.
- **Where the reaction lives.** Immediate: at the widget, as a returned struct. Callback view: at
  the widget, as a closure. MVU: in `update`, a match arm away - the cost is indirection when
  reading a card top to bottom, the payoff is that every way the state can change is one list,
  exhaustively matched, and grep-able by variant name. In practice the MVU editors converged on
  small per-editor enums (`TerrainMsg`, `TrackMsg`...), which kept the lists short; the callback
  editors converged on small methods (`add_island`, `rename`) that closures call, which is the same
  code with the enum deleted.
- **The compiler's help flipped between the two.** MVU messages are plain data: `Clone`, `Debug`,
  comparable, and the borrow checker never sees the program through them - `update` has `&mut self`
  and does as it pleases. Callback closures capture, so every reaction that touches two fields
  learns Rust's disjoint-borrow rules; it compiles, but the ceremony is real, and a closure can't
  be printed, logged or compared. MVU's weakness is the opposite: carrying data in messages invites
  carrying *stale* data. One real bug during the port: a theme toggle computed "the other theme"
  when the message was *made*, not when it was *applied*, so two toggles in one frame canceled out.
  The fix - send the intent, read the state in `update` - is a discipline MVU needs and callback
  view gets for free, since closures read the program when they run.
- **Names still fail at run time** in all four architectures: targets are strings, and a typo makes
  a dead widget, found by a script. The retained experiment's conclusion stands - the named-target
  contract is independent of the program API - and so does its evidence: the scripts passed
  essentially unmodified through every rewrite (two frame-count adjustments for the new one-frame
  Headless was all the message ports needed).
- **Dynamic structure is just an `if` again.** `view` describes the frame from the state, so
  Veehiicuul's cards, MeshModeler's dialogs and the showcase's closable bar came back from the
  retained experiment's rebuild protocols to plain control flow. This is immediate mode's best
  property, kept.
- **Custom widgets got a real shape.** A program's widget implements `Widget<M>`: what it is to the
  input (`kind`, `sense`, `parts`), what input means (`interact`, sending messages), and how it
  draws (`draw`, with the frame's hover, press and focus in `Look`). SimplePaintDemo's row faces,
  Veehiicuul's ground view and the editors' overlays are ordinary widgets; in immediate mode they
  were ad-hoc `ui.interact` plus painter calls. The interface still keeps what only a widget needs
  (caret, scroll, open list) in memory keyed by name, now written from the widget's own contexts.
- **Testing.** Headless runs one frame per input call, and the frame that reads input is the frame
  whose output the test asserts on - the repository's rule, now also the test API's shape. Both
  message designs test program logic without the UI; MVU alone can also make the test's *stimulus*
  a plain value: `send_message(Message::Save)` needs no closure and can be read back in a failure
  message. The scripts already record and replay at the input level, deterministically, on a fixed
  clock - so a message log adds value below that only where messages are data: logging what a
  session did by name, asserting on emitted messages, fuzzing `update` with generated messages.
  With callback view, a log of opaque closures says nothing.
- **Logging and replay, honestly.** The original case for MVU here was message logging and replay
  as a debugging tool. Building both showed the script system already covers replay: it drives the
  real program by named targets and checks state, which is a *better* record than messages for
  reproducing a session, because it survives refactors of the message set. What the MVU log is
  still uniquely good for is the small stuff: a readable trace of what the input meant
  (`Track(AddIsland { .. })` in a log line), and unit tests that construct meanings directly. That
  is real, but it is less than the original pitch.

## Choosing

Retained mode is already decided: slower here, three places per control, and its one structural
promise (skip clean subtrees) can't be cashed under this renderer. The live choice is among the
other three, and it is close, because they share the renderer, the design system, the scripts, and
now the one-frame rule.

- **Immediate** is the smallest and fastest, and its weakness is exactly one thing: the one-frame
  rule holds per widget but not across the pass, so order-of-layout bugs are possible and silent.
  The repository elevated that rule to law; immediate mode enforces it by review, not construction.
- **MVU** buys the rule by construction, plus messages-as-data: exhaustive matching, printable
  traces, `update` tested with plain values. It costs the enum boilerplate (about 6% of a program),
  the indirection between a widget and its meaning, and the stale-snapshot discipline.
- **Callback view** buys the same rule for the same frame cost, reads like immediate mode, and
  avoids stale snapshots by reading state at run time. It costs closure-capture ceremony on
  multi-field edits, and gives up data messages: nothing to print, match, or feed to a test except
  by calling methods, which is what its programs devolve to anyway.

Both message branches are complete and green; the library is byte-identical across them, so the
decision is only about how programs read and test, and it can be made per the repository's taste by
merging one branch and keeping the other in history, as the retained experiment is kept.
