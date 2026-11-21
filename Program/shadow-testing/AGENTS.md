# ShadowTesting

Follow the rules for GUI programs in the [AGENTS.md](../../AGENTS.md) at the root, including the supported window setup and the non-disruptive workflow. Run the program with `Program\RunInBackground\RunInBackground.ps1 shadow-testing`.

- Keep every way of drawing a shadow that's being tried here, not in Library/user-interface: a module of its own in `src`, with its settings and its pipeline, a shader of its own in `Shaders`, compiled by `build.rs`, and a card of sliders in `src/cards.rs`.
- Test each way's pixels against its formula with the GPU, as `Test/shadow-testing` does for the linear shadow, in both themes.
- Test input handling with the tests and scripts in `Test/shadow-testing`, as the [README](README.md#testing) says. The state that scripts save has where each slider is, as `Interface.Targets`.
