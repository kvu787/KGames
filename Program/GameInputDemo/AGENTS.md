# GameInputDemo

Follow the rules for GUI programs in the [AGENTS.md](../../AGENTS.md) at the root, including the supported window setups and the non-disruptive workflow. Run the demo with `Program\RunInBackground\RunInBackground.ps1 GameInputDemo`.

- The demo is a `Program` of [Library/UserInterface](../../Library/UserInterface/README.md), in `Source/Demo.rs`, which its main loop runs. `Source/Panels.rs` draws the cards with the framework's app bar, status bar, cards, wells, combo box, chips and tooltips, in the theme's colors, and draws the rest, like the stick graphs, itself.
- The demo reads its shortcuts, `F11` too, with GameInput, so `Demo` sets `FULLSCREEN_KEY` to false, and asks for fullscreen with `Frame::toggle_fullscreen`. No widget takes keyboard focus, so that the keys stay the user's, to try the Keyboard card with.
- `Source/Panels.rs` has the layout's constants. Of the 71 pixels that borderless fullscreen adds to the height, the bottom row of cards and the event log get all of them.
- Test input handling with the tests and scripts in `Test/GameInputDemo`, as the [README](README.md#testing) says. Scripts connect gamepads, keyboards and mice, and give their readings, with `Program` steps, which `Source/ScriptedSource.rs` reads. The state that scripts save has the interface's, as `Interface`: where the combo box, `Gamepad`, its choices, `Gamepad/0` and on, and the Devices card's rows, `Devices/0` and on, are, as `Targets`, whether the combo box's list is open, as `Popup`, and which device's tooltip shows, as `Tooltip`. Steps point at them by name.
- Keep GameInput's calls in `Source/GameInputSource.rs`, so that the rest of the demo runs on scripted input too. Ask the user to try what only real devices show, like GameInput's own readings and the motors.
