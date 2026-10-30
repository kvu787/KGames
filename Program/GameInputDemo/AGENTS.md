# GameInputDemo

Follow the rules for GUI programs in the [AGENTS.md](../../AGENTS.md) at the root, including the supported window setups and the non-disruptive workflow. Run the demo with `Program\RunInBackground\RunInBackground.ps1 GameInputDemo`.

- `Source/Panels.rs` has the layout's constants. Of the 71 pixels that borderless fullscreen adds to the height, the bottom row of panels and the event log get all of them.
- There's no way to send the demo gamepad input, and keyboard and mouse input sent with `SendInput` goes to the user's window. Ask the user to try changes to input handling.
