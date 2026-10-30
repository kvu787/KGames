# MinimalSceneTest

Follow the rules for GUI programs in the [AGENTS.md](../../AGENTS.md) at the root, including the supported window setups and the non-disruptive workflow. Run the test with `Program\RunInBackground\RunInBackground.ps1 MinimalSceneTest`. The sphere is in the middle of the screen 3.5 seconds after the test starts, so `--seconds 3.5` shows it in front of the cube.

- Keep the scene as the [README](README.md#scene) describes it.
- The 3D view fills the window at either supported height. Keep the text box out of the sphere's path.
- Test changes with the tests and scripts in `Test/MinimalSceneTest`, as the [README](README.md#testing) says. A script's fixed clock puts the sphere where you want it, like `Test/MinimalSceneTest/Scripts/Motion.json` does, without guessing `--seconds`.
