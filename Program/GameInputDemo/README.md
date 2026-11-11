# GameInputDemo

A Rust program that reads gamepads, keyboards and mice with Microsoft's
[GameInput](https://learn.microsoft.com/en-us/gaming/gdk/docs/features/common/input/overviews/input-overview)
API, and shows what it reads in a Win32 window drawn with Direct3D 12.

It uses no outside frameworks: Microsoft's [`windows-core`](https://crates.io/crates/windows-core)
crate is the only dependency besides the repo's own [Library](../../Library) crates. Like every
KGames GUI program, it's built on [Library/UserInterface](../../Library/UserInterface/README.md),
whose main loop runs it, and whose widgets draw its app bar, status bar, cards, combo box, list,
scroll bars, keycaps, badges, status icons and tooltips, after Microsoft's Fluent 2 design system.

## Supported window setups

The demo supports exactly the two window setups that every KGames program does, both on a
2560 × 1440 monitor at 100% scaling: maximized, with a 2560 × 1369 client area, and borderless
fullscreen, with 2560 × 1440 (see
[Library/UserInterface](../../Library/UserInterface/README.md#window-setups)). The cards have fixed
places in pixels, and the 71 pixels that fullscreen adds go to the bottom row of cards and to the
event log. In any other setup, the window shows what's wrong instead of the cards.

## What it shows

- **Title bar**: the GPU that draws.
- **App bar**: whether the window has focus, which the demo needs to read input, and the frame rate
  with the time per frame. Under it, anything wrong with the [settings file](#settings-file), or
  GameInput not being available, as info bars that close.
- **Gamepad**: an XY graph for each stick, with the unit circle, a ring half way, and the dead zone
  of the [rumble test](#rumble-test), the only part of the demo that applies one. Everywhere else,
  the demo shows the sticks' and triggers' raw values. A bar for each trigger. A chip for every
  button, in the accent color while it's held, and faint if the gamepad says it lacks it. A combo
  box after the card's title picks which [gamepad the demo listens to](#choosing-a-gamepad), if any.
- **Latest input event**: the newest reading that changed something: its device, what changed, how
  long after that device's previous reading it came, and the device's whole state. If that doesn't
  all fit, the card's details scroll.
- **Keyboard**: how many keys are held down, on a badge, and each of them as a keycap with the key's
  name, then its scan code, virtual-key code and character, in Cascadia Mono, so that they line up.
  Keys that don't fit scroll.
- **Mouse**: the buttons held, the motion and wheel, and a pad that moves with the raw relative
  motion, which has no pointer acceleration.

  Each of the two shows the last device whose keys, or mouse input, changed. A device can be both,
  like a keyboard with a touchpad, and its readings then have both kinds of input. A key press on
  it doesn't move the Mouse card to it.
- **Devices**: every connected gamepad, keyboard and mouse, from GameInput's device callback, as a
  list that scrolls if they don't all fit. The gamepad that the Gamepad card shows is in the accent
  color, with the accent pill of a selection, and gamepads that the demo ignores are faint. The
  pointer resting on a device's row shows a [tooltip](#device-tooltips) of everything known about
  it.
- **Event log**: everything above as it happens, newest first, in Cascadia Mono, so that its columns
  line up. A run of analog motion from one device shares a line, with a count.
- **Status bar**: the shortcuts, the GameInput runtime the demo loaded, how many readings it has
  handled, and the [identifier](#display-identifiers) and model of the display the window is on. A
  runtime deep in folders is cut short where it would reach the readings.

Shortcuts, while the demo's window has focus:

| Key   | Action                                                                                    |
| ----- | ----------------------------------------------------------------------------------------- |
| `F1`  | Rumble test on or off. The gamepad's sticks then drive its grip motors                    |
| `F2`  | Clear the event log                                                                       |
| `F9`  | Switches between the light and dark themes                                                |
| `F10` | Switches vertical sync on or off                                                          |
| `F11` | Switches between maximized and borderless fullscreen, for the lowest latency              |

The demo reads `F1` and `F2` from GameInput's readings, like any other key. `F9`, `F10` and `F11`
are Library/UserInterface's, which reads them from the same readings, as in every KGames program, so
the demo doesn't also switch on them, which would switch them back.

### Rumble test

The rumble test drives only the standard rumble: the low-frequency motor in the left grip and the
high-frequency one in the right, which Xbox 360, PlayStation 4 and 5 and most other gamepads with
rumble have. The left stick drives the first and the right stick the second. The trigger motors of
Xbox One and Series gamepads stay off. The Gamepad card's "Motors" still lists every motor the
gamepad has. With the rumble test off, no motor runs, and while the window doesn't have focus,
which stops the demo from reading the sticks, none does either.

Minimizing the window takes its focus. The main loop draws no frames while the window is minimized,
but it still runs the demo's update once with the focus lost, so the motors stop as the window
minimizes.

A motor is off while its stick is inside the inner dead zone, the ring on the stick's graph. Sticks
don't read exactly 0 at rest, and the dead zone keeps that noise from running the motors. Past the
ring, the motor's strength grows linearly with how far the stick is pushed, in any direction, from
off at the ring to full at the edge. Both sticks have the same dead zone: 10% of full deflection by
default, set with `RumbleTestInnerDeadZone` in the [settings file](#settings-file). Nothing else in
the demo applies a dead zone.

### Choosing a gamepad

Click the combo box after the Gamepad card's title for a list of choices, and click one to pick it.
Esc, a click anywhere else in the window, or the window losing focus, like when it's minimized,
closes the list. While the list is open, Up, Down, Home and End move to another choice, and Space or
Enter pick it, like in any combo box of Library/UserInterface. Otherwise the combo box never takes
keyboard focus, so the keys that you press to try the Keyboard card leave it alone.

| Choice    | What the demo listens to                                                        |
| --------- | ------------------------------------------------------------------------------- |
| A gamepad | Only that one. If it disconnects, the demo waits for it to reconnect            |
| `None`    | No gamepad                                                                      |

The demo ignores a gamepad it doesn't listen to: its readings aren't shown, logged or counted, and
the rumble test leaves its motors off. That includes any keyboard or mouse input from the same
device. A GameSir controller in Xbox 360 mode, for example, is one device with a gamepad, a keyboard
and a mouse.

The choice isn't saved. Until one is picked, the demo chooses for itself: the first gamepad it finds
connected. If that one disconnects, it switches to another that's still connected, or else to the
next one that connects. Before any gamepad connects, the combo box shows `None`, and a note after it
says that no gamepad is connected. The combo box is the library's, which reads the mouse's buttons
from GameInput too, and the pointer from where Windows' cursor is in the window.

### Device tooltips

A device's name and IDs often don't say what it is. Two keyboards can both be
`HID Keyboard Device (0000:0000)`, for example. So once the pointer rests on a row of the Devices
card for half a second, a tooltip shows all that GameInput and Windows say about that device, in
three parts:

| Part              | What it has                                                                     |
| ----------------- | ------------------------------------------------------------------------------- |
| GameInput         | The device's family, kinds of input, vendor, product and revision IDs, HID      |
|                   | usage, versions, what its keyboard, mouse or gamepad has, its motors, its       |
|                   | container, and the PnP path of its device interface                             |
| Windows           | The device's node in Windows' device tree, from that PnP path, as Device        |
|                   | Manager shows it: its instance ID, names, manufacturer, class, driver and       |
|                   | hardware IDs                                                                    |
| Connected through | The nodes above it, nearest first, up to the computer's own buses, like a USB   |
|                   | hub, or a driver like the Intel HID Event Filter that makes up devices for a    |
|                   | laptop's buttons                                                                |

Devices that share a container are parts of one physical device, like a mouse and the keyboard it
has for its macros. Windows gives devices built into the computer the container
`{00000000-0000-0000-FFFF-FFFFFFFFFFFF}`.

Windows' configuration manager takes a few milliseconds to read a device's nodes, so a thread of its
own does that for each device as it connects, instead of the main loop. The tooltip reads what it
found. Long values, like the PnP path, wrap, in a tooltip up to 720 pixels wide, wider than
WinUI's 320, so that a device deep in Windows' device tree, behind hubs, still fits in the window.
The pointer is the library's, like the combo box's, so the tooltip only shows while the window has
focus. Library/UserInterface places it above the pointer, or under it if there isn't room, and
always inside the window.

## Requirements

- Rust with the `x86_64-pc-windows-msvc` toolchain
- Windows SDK (provides `dxc.exe` under `C:\Program Files (x86)\Windows Kits\10\bin\<version>\x64`)
- GameInput 4.0.306, installed once, from an elevated prompt, with
  `winget install --exact --id "Microsoft.GameInput" --source "winget" --version "4.0.306"`. The
  demo doesn't build or start without it (see [Library/GameInput](../../Library/GameInput/README.md))
- Optional, for the D3D12 debug layer in debug builds: the "Graphics Tools" optional Windows feature
- Optional, for [RunInBackground](../RunInBackground/README.md): the .NET 10 SDK

## Build and run

```powershell
cargo run --release
```

Or run `HumanRun.cmd`, which you can also double-click in File Explorer. It's for people who want to
use the demo, not for coding agents, which use [RunInBackground](#running-it-in-the-background). It
runs `HumanRun.ps1`, which does the same as above with `--display Primary`, so the demo opens on the
primary display whatever `Settings.json` says. Other arguments go to the demo.

To open the demo on another display, find the display's identifier with `IdentifyDisplays`, then
pass it with `--display`, or put it in the [settings file](#settings-file):

```powershell
cargo run --release --bin identify_displays
cargo run --release -- --display KXGR16
```

To open the demo without taking focus from the window you're typing in, add `--no-activate`. The
window opens without taking focus from that one, and gets keyboard and mouse input once you click
it.

### Running it in the background

[RunInBackground](../RunInBackground/README.md) builds the demo, opens it without taking focus, saves
a screenshot of its window, and closes it. It's how coding agents run the demo while you work in
other windows (see [AGENTS.md](AGENTS.md)).

```powershell
Program\RunInBackground\RunInBackground.ps1 GameInputDemo --display KXGR16
```

With `--all-scripts`, it runs the demo with the scripts in `Test/GameInputDemo/Scripts` instead (see
[Testing](#testing)).

## Displays and settings

### Display identifiers

[IdentifyDisplays](../IdentifyDisplays/README.md) shows each display's identifier, like `KXGR16`, on
that display. The demo matches identifiers without regard to case.

### Settings file

The demo reads `Settings.json`, in this folder next to `Cargo.toml`, when it starts. The folder's
path is built into the exe, so the exe finds the file wherever it's run from. The file is in Git. If
it doesn't exist, the demo writes it with the defaults:

```json
{
  "Display": "Primary",
  "WindowMode": "WindowedMaximized",
  "VerticalSync": true,
  "Theme": "Light",
  "RumbleTestInnerDeadZone": 0.1
}
```

| Setting                   | Value                                                                       |
| ------------------------- | --------------------------------------------------------------------------- |
| `Display`                 | `"Primary"`, or the identifier of the display to open on                    |
| `WindowMode`              | `"WindowedMaximized"` or `"BorderlessFullscreen"`, the two supported setups |
| `VerticalSync`            | Whether frames wait for the screen's refresh, and never tear, or present at |
|                           | once, and may tear. `F10` switches it                                       |
| `Theme`                   | `"Light"` or `"Dark"`, the theme that the demo starts in. `F9` switches it  |
| `RumbleTestInnerDeadZone` | The [rumble test](#rumble-test)'s stick dead zone, from 0 to less than 1    |

The first four are the settings that every KGames program has, which
[Library/UserInterface](../../Library/UserInterface/README.md#settings-and-the-command-line) reads,
and the last is the demo's own.

- If `Display` names a display that isn't connected, the demo opens on the primary display, and a
  notice under the app bar says so.
- `F11` switches between the two window modes, whichever the demo opened in.
- A setting that's missing keeps its default. So does one that's wrong, and a notice under the app
  bar says what's wrong with it. If the file isn't valid JSON, it says where, and the demo uses the
  defaults. Values that are words, like `"WindowedMaximized"`, ignore case, but setting names don't.
- The demo never changes the file once it exists. Delete it to get the defaults back.

`--display` and an identifier on the command line overrides `Display`. `--display Primary` opens on
the primary display. Unlike `Display`, a display named with `--display` must be connected. If it
isn't, the demo writes why to standard error and exits, instead of opening on the primary display.

## How it gets GameInput

The [GameInput library](../../Library/GameInput/README.md) declares the API by hand, and starts
GameInput with `GameInputInitialize` from Microsoft's `GameInput.lib`, which loads GameInput
4.0.306's installed runtime, from System32, unless a `GameInputRedist.dll` is beside the exe.
The status bar of the demo's window shows which runtime it loaded. Without it installed, the demo
doesn't build, and doesn't start. With `--no-input`, it reads no input, and the status bar says
that GameInput isn't available.

## How it reads input

GameInput's input isn't presses and releases. It keeps a short history of *readings*, where each
reading is a timestamped snapshot of one device's whole state. Library/UserInterface reads every program's input
with GameInput, once a frame, right before the frame, and the demo shows the same readings that the
library makes its events from, from `startup.devices`. Every frame, the library:

1. Drains the queue of device events that its `RegisterDeviceCallback` callback fills. That callback
   runs on one of GameInput's threads, and also reports the devices that were already connected when
   the program started.
2. For each of gamepad, keyboard and mouse input, walks the history from the last reading it handled
   with `GetNextReading`, so no reading is missed between frames. GameInput keeps readings for only
   a short time, so after a pause in input the last reading handled has left the history, and
   `GetNextReading` fails with `GAMEINPUT_E_REFERENCE_READING_TOO_OLD`. The library then walks back
   from the newest reading with `GetPreviousReading` to find the ones it hasn't handled.

Then the demo:

3. Compares each reading with the same device's previous one to work out what changed. It keeps
   every device's last reading itself for that, including the ones it drops without focus.
   `GetPreviousReading` would give it too, but only while it's still in the history, so after a
   pause a held key would show as pressed again.

`src/game_input_source.rs` takes the library's devices and readings, once each a frame, as plain
data, so that the rest of the demo also runs on input from tests and scripts (see
[Testing](#testing)). Its `Options::gamepad` is on, so that gamepads' readings count as input that
its frames take, for the [input latency](../../Library/UserInterface/README.md) that every program
measures.

Like every KGames program, the demo only handles input while its window has focus, and the app bar
says whether it has focus. The library turns on GameInput's background input, so that it follows
the devices without focus too, and sees what's already held when focus comes. So the demo drops
what comes without focus itself, by the window's own focus changes: readings that come while the
window doesn't have focus, including while it's minimized, and from before it got focus. Otherwise it would show and log what's typed into
other windows, and its shortcuts would run when they're pressed in other apps. When the window loses
focus, the cards stop showing keys, buttons and sticks as held, and a gamepad that the demo switches
to then shows at rest. The frame that restores a minimized window counts as the window getting
focus, after losing it.

Keys and buttons that are still held when the window gets focus, like the button of the click that
gave it, or the Alt of the Alt+Tab that switched to it, are ignored until they're let go of: they
don't show as held, and neither their press nor their release is logged. After that, they work as
usual. Sticks, triggers and mouse motion show as usual right away.

## Input latency

The demo is built to show input on screen as soon as it can. Its part of the latency is the time
from a reading reaching GameInput to a frame that shows it reaching the screen. The rest, like the
device's own polling, its connection, and the monitor's scanout and response, is outside the demo.

Library/UserInterface's main loop does most of it (see its
[README](../../Library/UserInterface/README.md#input-latency)): only one frame is on the GPU at a
time, the loop waits until the GPU and DXGI can take another, and only then calls the demo's
`update`, which reads GameInput, so no readings age during the wait. The demo starts with vertical
sync on, as every program does, so its frames wait for the screen's refresh, and never tear. With
`F10`, or `"VerticalSync": false` in the [settings](#settings-file), they're presented without
waiting, and the loop spins instead of waiting, so the demo draws thousands of frames a second, and
whenever the screen takes a frame, the newest one read its input a fraction of a millisecond
earlier. The window draws with the GPU that its monitor is an output of, and the process opts out
of power throttling, so its frames are paced the same in the background. On the development
machine, an RTX 5090 Laptop GPU and a Core Ultra 9 275HX, a whole frame takes about 0.2 ms.

Without vertical sync, when Windows flips the demo's frames straight to the screen, a frame can
replace the one being scanned out mid-refresh, which can show as tearing. Otherwise, the desktop
compositor shows the newest frame at its next refresh. `F11`, or
`"WindowMode": "BorderlessFullscreen"`, makes the window cover its monitor, which lets Windows flip
the frames straight to the screen, instead of the compositor copying them into its own frame. That
saves up to a refresh, and lets frames tear. In a window, Windows can only do that if the GPU
supports multiplane overlays.

What it doesn't do:

- **Raise its thread's priority.** The main thread never blocks, so above normal priority, it could
  hold off the normal priority threads that deliver its input when every core is busy: GameInput's
  own threads in the process, its raw input proxy, and its service. Windows already favors the
  threads of the window in the foreground.
- **Cap the frame rate.** Drawing one frame per refresh, timed to finish just before vertical sync,
  would save power, but the input it shows would be older, and a frame that finishes late waits a
  whole refresh.
- **Change how GameInput's helper is scheduled.** Keyboards and mice reach GameInput through its
  `GameInputRawInputProxy.exe`. On the development machine, a key injected with `SendInput` took
  about 0.2 ms to reach the demo. Opting the helper out of power throttling too made no difference,
  and forcing it into efficiency mode added about 0.1 ms.

Without vertical sync, the cost is power: the main thread keeps a CPU core busy, and the GPU draws
thousands of frames a second, even when nothing changes. Minimizing the window stops both.

## Testing

The demo reads devices and readings from an `InputSource`: GameInput's in a normal run, and a
scripted one, in `src/scripted_source.rs`, in tests and scripted runs. So everything after
GameInput, from which gamepad it listens to, to dropping input without focus, the rumble test, the
shortcuts and the log, runs without a gamepad, and without your keyboard and mouse.

```powershell
cargo test -p game_input_demo_tests
Program\RunInBackground\RunInBackground.ps1 GameInputDemo --all-scripts
```

The tests in `Test/GameInputDemo` feed the demo's input directly, and press and move the mouse in its
cards' real layout, which Library/UserInterface's `Headless` draws without a window: the combo box
and its list, the Devices card's list, its rows and their tooltips, and what scrolls when it doesn't
fit. One draws a frame with the GPU, offscreen, and checks where the sticks' dots are. The scripts in
`Test/GameInputDemo/Scripts` run the real demo, in its window, as
[Library/UserInterface](../../Library/UserInterface/README.md#testing) describes. Their `Program`
steps, which the demo's `Program::script` does, connect and disconnect devices, and give their
readings:

```json
{
  "Steps": [
    { "Frame": 1, "Program": { "Connect": { "Id": 1, "Name": "Pad", "Input": ["Gamepad"],
                                            "Motors": ["LowFrequency", "HighFrequency"] } } },
    { "Frame": 1, "Program": { "Connect": { "Id": 2, "Name": "Keys", "Input": ["Keyboard"] } } },
    { "Frame": 1, "Program": { "Connect": { "Id": 3, "Name": "Rodent", "Input": ["Mouse"] } } },
    { "Frame": 2, "Program": { "Gamepad": { "Id": 1, "LeftStick": [0.5, 0], "Buttons": ["A"] } } },
    { "Frame": 2, "Program": { "Keyboard": { "Id": 2, "Down": ["F1"] } } },
    { "Frame": 3, "Program": { "Mouse": { "Id": 3, "Motion": [10, -5], "Wheel": [0, 120] } } },
    { "Frame": 4, "Program": { "Disconnect": 1 } },
    { "Frame": 5, "Expect": { "Listen": "Pad", "RumbleTest": true } }
  ]
}
```

Each step gives one kind of a device's input in whole, as GameInput's readings do: what the step
leaves out of that kind is at rest, except that a mouse's `Motion` and `Wheel` add to its totals, as
a real mouse's do. A device with more than one kind keeps the others as its last steps left them, so
a `Keyboard` step on a gamepad that's also a keyboard doesn't let go of its buttons. A step's
readings are at the time of its frame.

| Step         | Members                                                                         |
| ------------ | ------------------------------------------------------------------------------- |
| `Connect`    | `Id`, a whole number from 1, `Name`, `Input` from `Gamepad`, `Keyboard` and     |
|              | `Mouse`, and `Motors` from `LowFrequency`, `HighFrequency`, `LeftTrigger` and   |
|              | `RightTrigger`                                                                  |
| `Gamepad`    | `Id`, `LeftStick` and `RightStick` as `[x, y]`, `LeftTrigger`, `RightTrigger`,  |
|              | and `Buttons`, named as the Gamepad card names them                             |
| `Keyboard`   | `Id`, and the keys `Down`, like `F1`, `A` or `Left`                             |
| `Mouse`      | `Id`, `Motion` and `Wheel` as `[x, y]`, and `Buttons`, named as the Mouse card  |
|              | names them                                                                      |
| `Disconnect` | The device's `Id`                                                               |

The window's own keys and mouse are the script's `Key`, `Click`, `Move` and other steps, which the
combo box, the Devices card's rows and `F11` read, like `{ "Frame": 10, "Key": "F11" }`.

The state that scripts save and check has what the cards show, the log's 10 newest lines, and the
rumble that the demo sends each gamepad. The main loop adds the interface's, as `Interface`. Its
`Targets` say where the combo box, `Gamepad`, and each of its choices, `Gamepad/0`, `Gamepad/1` and
so on, are, where the Devices card's list, `Devices`, is, and each of its rows in sight,
`Devices/0`, `Devices/1` and so on, in the card's order, with its device's label as its `Label`.
Steps point at them by name, like
`{ "Frame": 3, "Move": "Devices/1" }`, or `{ "Frame": 6, "Click": "Gamepad/1" }` for a choice. Each
choice's `Label` is its gamepad's, and the combo box's is the chosen one's. `Popup` is `Gamepad`
while the combo box's list is open, and `null` otherwise. `Tooltip` has the name of the row whose
tooltip shows, like `Devices/1`, and where it is, or is `null` without one. A device from a script
has no PnP path, so its tooltip only has what the `Connect` step gives, and says that Windows has
nothing on it.

| Script          | What it does                                                                    |
| --------------- | ------------------------------------------------------------------------------- |
| `DeviceTooltip` | Rests the pointer on two devices' rows, and shows their tooltips                |
| `Gamepad`       | Presses buttons and moves sticks and triggers, lets go, and disconnects         |
| `Keyboard`      | Holds keys, clears the log with `F2`, and switches fullscreen with `F11`        |
| `Mouse`         | Moves a mouse, presses its buttons, and turns its wheel                         |
| `Overview`      | Gives every card something to show, for a look at the whole window              |
| `Picker`        | Picks gamepads with the combo box, with clicks and keys, and closes its list    |
| `RumbleTest`    | Drives the motors with the sticks, and stops them without focus                 |

What only real devices show still needs a person: GameInput's own readings and device events, how
the motors feel, and focus changing while a gamepad is held.

### With SendInput

Input sent with `SendInput` reaches GameInput as from an "Unidentified keyboard" or "Unidentified
mouse" with ID `0000:0000`. Injected input can be lost for about half a second after the demo starts,
while GameInput adds that device. It goes to the window that has focus, so it's for trying the demo
by hand, not for coding agents.

A precision touchpad, like a laptop's, moves the pointer the same way. Windows' touchpad driver reads
where the fingers are, and moves the pointer with mouse input of its own, which doesn't say what
device it came from. So the touchpad is the "Unidentified mouse" too: it's only in the Devices card
once it's used, its tooltip has no PnP path, and its input can't be told apart from `SendInput`'s.

The virtual gamepad that Windows' `InputInjector.InitializeGamepadInjection` creates doesn't reach
GameInput.

## Using more APIs

For Windows APIs, which `windows-bindgen` generates, see
[Library/WindowsApi](../../Library/WindowsApi/README.md).

For GameInput APIs, see [Library/GameInput](../../Library/GameInput/README.md#using-more-apis).

## Layout

| Path                       | Contents                                                                      |
| -------------------------- | ----------------------------------------------------------------------------- |
| `src/main.rs`              | Runs the demo with Library/UserInterface's main loop                          |
| `src/lib.rs`               | The demo's parts, as a library that `Test/GameInputDemo` tests                |
| `src/demo.rs`              | The demo as a `Program`: its input source, and what it does each frame        |
| `src/settings.rs`          | The demo's own setting, `RumbleTestInnerDeadZone`                             |
| `src/devices.rs`           | Devices and readings as plain data, and `InputSource`, which gives them       |
| `src/device_tree.rs`       | Looks up a device's nodes in Windows' device tree, from its PnP path          |
| `src/game_input_source.rs` | Takes the devices and readings that the library reads with GameInput          |
| `src/scripted_source.rs`   | Devices and readings from tests and scripts, instead of GameInput             |
| `src/input.rs`             | Keeps what the cards show, from the devices and readings                      |
| `src/cards.rs`             | Lays out the cards in fixed pixels, and draws them with Library/UserInterface |
| `HumanRun.ps1`             | Builds the demo and runs it on the primary display, for people to use it      |
| `HumanRun.cmd`             | Runs `HumanRun.ps1`, for double-clicking in File Explorer                     |
| `Settings.json`            | The [settings file](#settings-file), which the demo writes if it's missing    |
