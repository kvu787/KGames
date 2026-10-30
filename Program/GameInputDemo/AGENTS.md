# GameInputDemo

Follow the rules for GUI programs in the [AGENTS.md](../../AGENTS.md) at the root, including the supported window setups and the non-disruptive workflow. Run the demo with `Program\RunInBackground\RunInBackground.ps1 GameInputDemo`.

- `Source/Panels.rs` has the layout's constants. Of the 71 pixels that borderless fullscreen adds to the height, the bottom row of panels and the event log get all of them.
- Test input handling with the tests and scripts in `Test/GameInputDemo`, as the [README](README.md#testing) says. Scripts connect gamepads, keyboards and mice, and give their readings, with `Program` steps, which `Source/ScriptedSource.rs` reads. The state that scripts save has where the select box and its choices are, as `Targets`.
- Keep GameInput's calls in `Source/GameInputSource.rs`, so that the rest of the demo runs on scripted input too. Ask the user to try what only real devices show, like GameInput's own readings and the motors.
