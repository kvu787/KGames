# Retained-mode user interface

`Library/UserInterface` is a retained-mode UI library: a program builds a tree
of controls once, each control owns its state, and the program talks to the
tree through events and setters. This replaced the immediate-mode library,
where a program rebuilt the whole interface every frame with `ui.add`.
[RetainedVersusImmediate.md](RetainedVersusImmediate.md) compares the two.

## The model

- **Tree.** `Program::build` creates named nodes with fixed-pixel rectangles:
  `ui.node("Save", rect, Button::new("Save"))`. Groups nest:
  `ui.group("Card", rect, Card::new("Buttons"), |ui| { ... })`. A node's name
  is unique in the window and is what scripts, tests and the program point at.
  `build` runs before the first frame and again whenever the client area
  changes (F11) or the program calls `ui.request_rebuild()`.
- **Controls own their state.** A `Slider` owns its value, a `TextBox` its
  text, a `ToggleSwitch` whether it's on. The program reads state with typed
  access (`ui.get::<Slider>("Volume").value()`, or sugar like
  `ui.value("Volume")`) and writes it with `ui.get_mut` or sugar
  (`ui.set_value`, `ui.set_text`). Mutating never re-enters layout: rects are
  fixed at build.
- **Events.** Input lands on controls, which update themselves and emit
  `UiEvent { source, kind }` into a frame queue. The program's `update` reads
  them: `if ui.clicked("Save") { ... }`,
  `if let Some(value) = ui.changed("Volume") { ... }`. Kinds: Clicked,
  DoubleClicked, SecondaryClicked, Toggled(bool), Changed(f64),
  Selected(usize), Activated(usize), TextChanged(String), Submitted(String),
  ColorChanged(Color), Chose(String), Dialog(DialogResult), Closed,
  Expanded(bool).
- **Three-phase frame.** `begin_frame` reads input and hit-tests against the
  last frame's targets, as before. Then: (1) **dispatch** — every visible
  node registers its target and reacts to input (`Control::handle`), emitting
  events; (2) **update** — the program drains events and mutates the tree;
  (3) **draw** — every visible node paints (`Control::draw`). So a click's
  effect, including program-driven text changes, shows the same frame, which
  keeps the input-latency story and the scripts' timing identical to the
  immediate library's.
- **Visibility and enabling cascade.** `ui.set_visible` / `ui.set_enabled` on
  a group applies to its subtree. Controls that own children (Expander,
  Dialog) show and hide them through `Control::children_visible`.

## What stayed from the immediate library

The per-frame machinery below the program API is unchanged, because it's
mode-agnostic and the scripts and tests contract depends on it:

- The flat **target list** (name, rect, layer, kind, label), rebuilt each
  frame by nodes and their internal parts (menu items, combo items, list rows,
  dialog buttons register targets under "<name>/<part>" as before). Hit-test:
  topmost target under the pointer, layers above base win, latest registered
  wins within a layer.
- `interact` / `Response`, focus, Tab order, popups, modality, tooltips,
  animations, `memory` for internal sub-part state, layers and clips, the
  PerfBar, F10/F11, the unsupported-setup overlay.
- Drawing (Painter/DrawList/Renderer/Text), Design (Theme/Tokens/Controls/
  Icons), Input, Platform, Settings, Performance.
- `Headless` and the script format: names, `Interface.Targets`, timing.

## Porting recipe (immediate widget -> retained control)

1. The widget struct loses its borrows (`&'v mut T`, `&'a str`) and owns its
   data (`T`, `String`). Builder methods stay.
2. `impl Widget for X { fn show(self, ui, name, rect) -> Response }` becomes
   `impl Control for X` with:
   - `kind()`, `sense()`, `label()` (what `show` passed to `interact` /
     `set_label`).
   - `handle(&mut self, ui, ctx)`: the input half of `show`. Instead of
     writing through a borrow and setting `response.changed`, mutate `self`
     and `ui.emit(ctx, kind)`. `ctx.response` is the Response that `Ui`
     computed from `interact`.
   - `draw(&mut self, ui, ctx)`: the painting half, unchanged except reading
     `self` instead of borrowed values. Geometry computed in `handle` that
     `draw` needs is cached in `self`.
3. Program `ui.add("Name", rect, X::new(&mut value, ...))` at a call site
   becomes `ui.node("Name", rect, X::new(initial, ...))` in `build`, plus an
   event read or getter in `update`.
4. Per-frame text and custom painting go through setters (`set_text`) and
   `Canvas<T>` nodes (`ui.get_mut::<Canvas<T>>("Sticks").data = ...`).

## Status

- [x] Design
- [ ] Core: tree, Control trait, events, builder, frame phases
- [ ] Widgets ported
- [ ] Application layer: Program::build, run loop, Headless
- [ ] UiShowcase + tests
- [ ] SimplePaintDemo, MinimalSceneTest, SpinningCarTest, IdentifyDisplays
- [ ] FrameTimes, Benchmarks
- [ ] GameInputDemo, MeshModeler, Veehiicuul
- [ ] Scripts pass via RunInBackground --all-scripts
- [ ] Comparison document
