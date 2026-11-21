# ShadowTesting

Follow the rules for GUI programs in the [AGENTS.md](../../AGENTS.md) at the root, including the supported window setup and the non-disruptive workflow. Run the program with `Program\RunInBackground\RunInBackground.ps1 shadow-testing`.

- Keep every way of drawing a shadow that's being tried here, not in Library/user-interface. Add one as the [README](README.md#adding-a-way) says: a shader of its own that includes `Shaders/Common.hlsli`, a module of its own in `src`, and a `Kind` in `src/approaches.rs`. The page makes its card of sliders from its settings.
- Keep every way's box the same, in the middle of a stage of the same size, so that the ways can be compared side by side.
- Give each way a CPU copy of its shader's formula, and test every pixel that it draws against it with the GPU, in both themes, as `Test/shadow-testing` does.
- Test input handling with the tests and scripts in `Test/shadow-testing`, as the [README](README.md#testing) says. The state that scripts save has where each slider is, as `Interface.Targets`, named after its way and setting, like `Gaussian/Blur`.
