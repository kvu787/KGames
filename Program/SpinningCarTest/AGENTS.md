# SpinningCarTest

Follow the rules for GUI programs in the [AGENTS.md](../../AGENTS.md) at the root, including the supported window setups and the non-disruptive workflow. Run the test with `Program\RunInBackground\RunInBackground.ps1 SpinningCarTest`. About 3.4 seconds after it starts, the car is near the middle, so `--seconds 3.4` shows it.

- Keep the scene as the [README](README.md#scene) describes it.
- `Source/CarMesh.rs` and `Assets/SceneBackground.png` are baked assets. Don't edit the mesh by hand.
- The 3D view fills the window at either supported height. Keep the text box out of the car's path.
- Test changes with the tests and scripts in `Test/SpinningCarTest`, as the [README](README.md#testing) says. A script's fixed clock puts the car where you want it, like `Test/SpinningCarTest/Scripts/Animation.json` does, without guessing `--seconds`.
