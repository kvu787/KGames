# Supported window setups

The demo only supports the user's own setup, to keep development simple. It supports exactly two window setups, both on a 2560 × 1440 monitor at 100% scaling (96 DPI), where logical and physical pixels are the same:

- **Maximized**, with a client area of 2560 × 1369. The taskbar takes 48 pixels at the bottom of the monitor, which leaves a 2560 × 1392 work area. The title bar takes the top 23 pixels of that, so the client area runs from screen y = 23 down to the bottom of the work area: 1392 − 23 = 1369.
- **Borderless fullscreen**, with a client area of 2560 × 1440 that covers the whole monitor. `F11` switches between the two.

In any other setup, the window shows an error that says what's wrong, instead of the panels. [Library/Application](../../Library/Application/README.md) does the check.

So:

- Lay out in fixed pixels for these two client areas. `Source/Panels.rs` has the layout's constants. The only difference between the two is 71 pixels of height, which go to the bottom row of panels and the event log.
- Don't add DPI scaling, other window sizes, a windowed mode, minimum sizes, or layouts that adapt to the window's size. Don't keep the demo working in other setups beyond showing the error.

# Non-disruptive workflow

While working:

- The agent must not launch the app on any display except for CVVNY2.
- The agent must not steal input focus away. For example, if the user is typing in another window, then those keypresses shouldn't be rerouted to processes or windows that the agent launches.
- It is fine if the agent launches things that show up on the Windows taskbar. This isn't considered disruptive.

## How

- Run the demo with `Program\RunInBackground\RunInBackground.ps1 GameInputDemo`, from the root of the repository. It builds the demo, opens it on CVVNY2 without activating it, saves a screenshot of what it draws to `target\RunInBackground\GameInputDemo.png`, and closes it. Read the screenshot to see the result.
- Don't start the demo any other way, like with `cargo run`. `HumanRun.cmd` and `HumanRun.ps1` are for the user to launch the demo and use it. Never run them. Without `--no-activate`, its window takes focus. Without `--display CVVNY2`, it opens on the display in `Settings.json`, or on the primary display if that one isn't connected.
- If the script says CVVNY2 isn't connected, stop and tell the user. Don't pick another display.
- With `--keep-open`, the demo keeps running after the script ends. Stop it with `Stop-Process` and the process ID the script prints, once you're done with it.
- Run `IdentifyDisplays` only with `--list`. Without it, it puts a box on every display.
- Don't send keyboard or mouse input with `SendInput` or anything like it. It goes to the window that has focus, which is the user's, and the demo ignores all input without focus anyway. Ask the user to try changes to input handling.
