# Non-disruptive workflow

While working:

- The agent must not launch the app on any display except for CVVNY2.
- The agent must not steal input focus away. For example, if the user is typing in another window, then those keypresses shouldn't be rerouted to processes or windows that the agent launches.
- It is fine if the agent launches things that show up on the Windows taskbar. This isn't considered disruptive.

## How

- Run the demo with `Scripts\RunInBackground.ps1`. It builds the demo, opens it on CVVNY2 without activating it, saves a screenshot of what it draws to `target\Screenshot.png`, and closes it. Read the screenshot to see the result.
- Don't start the demo any other way, like with `cargo run`. `HumanRun.cmd` and `HumanRun.ps1` are for the user to launch the demo and use it. Never run them. Without `--no-activate`, its window takes focus. Without `--display CVVNY2`, it opens on the display in `Settings.json`, or on the primary display if that one isn't connected.
- If the script says CVVNY2 isn't connected, stop and tell the user. Don't pick another display.
- With `--keep-open`, the demo keeps running after the script ends. Stop it with `Stop-Process` and the process ID the script prints, once you're done with it.
- Run `IdentifyDisplays` only with `--list`. Without it, it puts a box on every display.
- Don't send keyboard or mouse input with `SendInput` or anything like it. It goes to the window that has focus, which is the user's, and the demo gets no keyboard or mouse input from GameInput without focus anyway. Ask the user to try changes to keyboard and mouse handling.
