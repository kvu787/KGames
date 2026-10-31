# GameInputDemo

A Rust program that reads gamepads, keyboards and mice with Microsoft's
[GameInput](https://learn.microsoft.com/en-us/gaming/gdk/docs/features/common/input/overviews/input-overview)
API, and shows what it reads in a Win32 window drawn with Direct3D 12.

It uses no frameworks: Microsoft's [`windows-core`](https://crates.io/crates/windows-core) crate
is the only dependency besides the repo's own [Library](../../Library) crates.

## Supported window setups

The demo is built for one machine, and supports exactly two window setups. Both are on a
2560 × 1440 monitor at 100% scaling (96 DPI), so logical and physical pixels are the same,
and the panels have fixed places in pixels:

| Setup                 | Client area | What makes it                                                              |
| --------------------- | ----------- | -------------------------------------------------------------------------- |
| Maximized             | 2560 × 1369 | A 48-pixel taskbar at the bottom leaves a 2560 × 1392 work area, and the   |
|                       |             | title bar takes its top 23 pixels                                          |
| Borderless fullscreen | 2560 × 1440 | A window without a frame that covers the whole monitor                     |

In any other setup, like a restored window, another monitor, another scale, or a moved
taskbar, the window shows what's wrong instead of the panels. It checks again whenever the
window moves or changes size, or the displays, the scale or the taskbar change.

## What it shows

- **Title bar**: the GPU that draws, and whether frames may tear (see [Input latency](#input-latency)).
- **Header**: the GameInput runtime the demo loaded, how many readings it has handled,
  whether the window has focus, and the frame rate with the time per frame. Under it,
  anything wrong with the [settings file](#settings-file) or the command line.
- **Gamepad**: an XY graph for each stick, with its recent path, the unit circle, and
  the dead zone of the [rumble test](#rumble-test), the only part of the demo that applies
  one. Everywhere else, the demo shows the sticks' and triggers' raw values. A filled bar for
  each trigger. A chip for every button that lights up while it's held. A select box
  after the panel's title picks which [gamepad the demo listens to](#choosing-a-gamepad),
  if any.
- **Latest input event**: a text box describing the newest reading that changed
  something: its device, what changed, how long after that device's previous reading
  it came, and the device's whole state.
- **Keyboard**: the keys held down, with each key's name, scan code, virtual-key code and character.
- **Mouse**: the buttons held, the motion and wheel, and a pad that moves with the
  raw relative motion, which has no pointer acceleration.

  Each of the two shows the last device whose keys, or mouse input, changed. A device can
  be both, like a keyboard with a touchpad, and its readings then have both kinds of
  input. A key press on it doesn't move the Mouse panel to it.
- **Devices**: every connected gamepad, keyboard and mouse, from GameInput's device callback.
  The gamepad that the panel shows is blue, and gamepads that the demo ignores are faint.
- **Event log**: everything above as it happens, newest first.
  A run of analog motion from one device shares a line, with a count.
- **Footer**: the shortcuts, and the [identifier](#display-identifiers) and model of the
  display the window is on.

Shortcuts, which the demo also reads with GameInput, while its window has focus:

| Key   | Action                                                                        |
| ----- | ----------------------------------------------------------------------------- |
| `F1`  | Rumble test on or off. The gamepad's sticks then drive its grip motors        |
| `F2`  | Clear the event log                                                           |
| `F11` | Switches between maximized and borderless fullscreen, for the lowest latency  |

### Rumble test

The rumble test drives only the standard rumble: the low-frequency motor in the left grip
and the high-frequency one in the right, which Xbox 360, PlayStation 4 and 5 and most
other gamepads with rumble have. The left stick drives the first and the right stick the
second. The trigger motors of Xbox One and Series gamepads stay off. The Gamepad panel's
"Motors" still lists every motor the gamepad has. With the rumble test off, no motor runs,
and while the window doesn't have focus or is minimized, which stops the demo from reading the
sticks, none does either.

A motor is off while its stick is inside the inner dead zone, the ring on the stick's
graph. Sticks don't read exactly 0 at rest, and the dead zone keeps that noise from
running the motors. Past the ring, the motor's strength grows linearly with how far the
stick is pushed, in any direction, from off at the ring to full at the edge. Both sticks
have the same dead zone: 10% of full deflection by default, set with
`RumbleTestInnerDeadZone` in the [settings file](#settings-file). Nothing else in the
demo applies a dead zone.

### Choosing a gamepad

Click the select box after the Gamepad panel's title for a list of choices, and click
one to pick it. Esc, a click anywhere else, or the window losing focus closes the list.

| Choice    | What the demo listens to                                                        |
| --------- | ------------------------------------------------------------------------------- |
| A gamepad | Only that one. If it disconnects, the demo waits for it to reconnect            |
| `None`    | No gamepad                                                                      |

The demo ignores a gamepad it doesn't listen to: its readings aren't shown, logged or
counted, and the rumble test leaves its motors off. That includes any keyboard or mouse
input from the same device. A GameSir controller in Xbox 360 mode, for example, is one
device with a gamepad, a keyboard and a mouse.

The choice isn't saved. Until one is picked, the demo chooses for itself: the first
gamepad it finds connected. If that one disconnects, it switches to another that's still
connected, or else to the next one that connects. Before any gamepad connects, the select
box shows `None`. The select box reads the mouse from the window's messages, not from
GameInput, because they say where the cursor is in the window.

## Requirements

- Rust with the `x86_64-pc-windows-msvc` toolchain
- Windows SDK (provides `dxc.exe` under `C:\Program Files (x86)\Windows Kits\10\bin\<version>\x64`)
- Internet access for the first build, which downloads the GameInput runtime (see
  [Library/GameInput](../../Library/GameInput/README.md))
- Optional, for the D3D12 debug layer in debug builds: the "Graphics Tools" optional Windows feature
- Optional, for [RunInBackground](../RunInBackground/README.md): the .NET 10 SDK

## Build and run

```powershell
cargo run --release
```

Or run `HumanRun.cmd`, which you can also double-click in File Explorer. It's for people
who want to use the demo, not for coding agents, which use
[RunInBackground](#running-it-in-the-background). It runs `HumanRun.ps1`,
which does the same as above with `--display Primary`, so the demo opens on the primary
display whatever `Settings.json` says. Other arguments go to the demo.

To open the demo on another display, find the display's identifier with `IdentifyDisplays`,
then pass it with `--display`, or put it in the [settings file](#settings-file):

```powershell
cargo run --release --bin IdentifyDisplays
cargo run --release -- --display KXGR16
```

To open the demo without taking focus from the window you're typing in, add `--no-activate`.
The window opens without taking focus from that one, and gets keyboard and mouse input once
you click it.

### Running it in the background

[RunInBackground](../RunInBackground/README.md) builds the demo, opens it without taking focus,
saves a screenshot of its window, and closes it. It's how coding agents run the demo while you
work in other windows (see [AGENTS.md](AGENTS.md)).

```powershell
Program\RunInBackground\RunInBackground.ps1 GameInputDemo --display KXGR16
```

With `--all-scripts`, it runs the demo with the scripts in `Test/GameInputDemo/Scripts`
instead (see [Testing](#testing)).

## Displays and settings

### Display identifiers

[IdentifyDisplays](../IdentifyDisplays/README.md) shows each display's identifier, like
`KXGR16`, on that display. The demo matches identifiers without regard to case.

### Settings file

The demo reads `Settings.json`, in this folder next to `Cargo.toml`, when it starts. The
folder's path is built into the exe, so the exe finds the file wherever it's run from.
The file is in Git. If it doesn't exist, the demo writes it with the defaults:

```json
{
    "Display": "Primary",
    "WindowMode": "Maximized",
    "AllowTearing": true,
    "RumbleTestInnerDeadZone": 0.1
}
```

| Setting                   | Value                                                                       |
| ------------------------- | --------------------------------------------------------------------------- |
| `Display`                 | `"Primary"`, or the identifier of the display to open on                    |
| `WindowMode`              | `"Maximized"` or `"BorderlessFullscreen"`, the two supported setups         |
| `AllowTearing`            | Whether frames may tear, which lets them reach the screen mid-refresh       |
| `RumbleTestInnerDeadZone` | The [rumble test](#rumble-test)'s stick dead zone, from 0 to less than 1    |

- If `Display` names a display that isn't connected, the demo opens on the primary display.
- `F11` switches between the two window modes, whichever the demo opened in.
- A setting that's missing keeps its default. So does one that's wrong, and the header
  says what's wrong with it. If the file isn't valid JSON, the header says where, and the
  demo uses the defaults. Values that are words, like `"Maximized"`, ignore case, but
  setting names don't.
- The demo never changes the file once it exists. Delete it to get the defaults back.

`--display` and an identifier on the command line overrides `Display`. `--display Primary`
opens on the primary display. Unlike `Display`, a display named with `--display` must be
connected. If it isn't, the demo writes why to standard error and exits, instead of opening
on the primary display.

## How it gets GameInput

The [GameInput library](../../Library/GameInput/README.md) declares the API by hand, finds
and loads the newest runtime, and puts the runtime from Microsoft's NuGet package next to the
demo's exe. The header of the demo's window shows which runtime it loaded.

## How it reads input

GameInput doesn't send input events. It keeps a short history of *readings*, where each
reading is a timestamped snapshot of one device's whole state. Every frame, the demo:

1. Drains the queue of device events that its `RegisterDeviceCallback` callback fills.
   That callback runs on one of GameInput's threads, and also reports the devices that
   were already connected when the demo started.
2. For each of gamepad, keyboard and mouse input, walks the history from the last reading
   it handled with `GetNextReading`, so no reading is missed between frames. GameInput
   keeps readings for only a short time, so after a pause in input the last reading
   handled has left the history, and `GetNextReading` fails with
   `GAMEINPUT_E_REFERENCE_READING_TOO_OLD`. The demo then walks back from the newest
   reading with `GetPreviousReading` to find the ones it hasn't handled.
3. Compares each reading with the same device's previous one to work out what changed. It
   keeps every device's last reading itself for that, including the ones it drops without
   focus. `GetPreviousReading` would give it too, but only while it's still in the history,
   so after a pause a held key would show as pressed again.

`Source/GameInputSource.rs` does the first two, and hands the devices and readings on as plain
data, so that the rest of the demo also runs on input from tests and scripts (see
[Testing](#testing)).

Like every KGames program, the demo only handles input while its window has focus, and the
header says whether it has focus. GameInput gives gamepad input to windows without focus. It's
meant to give keyboard and mouse input only to the window in focus, but the 4.0.306 runtime
that the demo loads from its folder gives it to the demo without focus too, even after
`EnableBackgroundInput(false)`. So the demo drops it all itself: readings that come while the
window doesn't have focus or is minimized, and from before it got focus, like the click that
gave it. Otherwise it would show and log what's typed into other windows, and its shortcuts
would run when they're pressed in other apps. When the window loses focus, the panels stop
showing keys, buttons and sticks as held.

## Input latency

The demo is built to show input on screen as soon as it can. Its part of the latency is
the time from a reading reaching GameInput to a frame that shows it reaching the screen.
The rest, like the device's own polling, its connection, and the monitor's scanout and
response, is outside the demo.

1. **No frame waits behind another.** The swap chain has a frame latency waitable object
   and a maximum frame latency of 1, and only one frame is on the GPU at a time. By
   default, DXGI lets 3 presented frames queue up, and each one delays the newest.
2. **Input is read last.** Each frame, the main loop first waits until the GPU has finished
   the previous frame and DXGI's queue has room. Only then does it read GameInput, build
   the frame and present it, so no readings age during the wait. The GPU finishes about
   45 µs before DXGI's queue has room, and the demo records the commands that don't depend
   on input in between.
3. **The wait spins.** The loop polls both conditions instead of blocking on events, so it
   notices them within microseconds, instead of waiting for Windows to wake its thread.
   It keeps handling window messages while it spins.
4. **Frames don't wait for vertical sync.** `Present(0, DXGI_PRESENT_ALLOW_TEARING)`
   returns at once, so the demo draws thousands of frames a second, and whenever the
   screen takes a frame, the newest one read its input a fraction of a millisecond earlier.
   When Windows flips the demo's frames straight to the screen, a frame can replace the
   one being scanned out mid-refresh, which can show as tearing. Otherwise, the desktop
   compositor shows the newest frame at its next refresh. With `"AllowTearing": false` in
   the [settings](#settings-file), the demo presents with `Present(0, 0)`: frames still
   don't wait, but the screen takes the newest one at each refresh, so none tear, even
   in fullscreen.
5. **Borderless fullscreen.** `F11`, or `"WindowMode": "BorderlessFullscreen"` in the
   settings, makes the window cover its monitor, which lets Windows flip the frames
   straight to the screen, instead of the compositor copying them into its own frame.
   That saves up to a refresh, and lets frames tear. In a window, Windows can only do
   that if the GPU supports multiplane overlays.
6. **The monitor's GPU draws.** On a laptop whose screen is wired to the integrated GPU,
   drawing with the discrete one would add a copy between the GPUs to every frame, and
   keep Windows from flipping frames straight to the screen. So the demo draws with the GPU
   that has the window's monitor as an output, which any GPU is fast enough for.
7. **Little work between reading input and presenting.** On the development machine, an
   RTX 5090 Laptop GPU and a Core Ultra 9 275HX, that takes about 0.13 ms: about 35 µs to
   read input and build the frame's geometry, 20 µs to copy its vertices and submit its
   commands, and 70 µs in `Present`. A whole frame takes about 0.2 ms.
   One vertex Vec is reused for every frame. A new one, being large, came fresh from the
   OS each frame, and filling it took a page fault every 4 KB, which made building the
   frame 4 times slower.
8. **No power throttling.** Windows lowers the quality of service of a process whose
   windows aren't in focus. On a CPU with efficiency cores, that can move its threads
   onto those, at lower clock speeds. The demo ignores input then, but keeps drawing, so it
   opts out with `SetProcessInformation`, which keeps its frames paced the same in the
   background as in the foreground.

What it doesn't do:

- **Raise its thread's priority.** The main thread never blocks, so above normal priority,
  it could hold off the normal priority threads that deliver its input when every core is
  busy: GameInput's own threads in the process, its raw input proxy, and its service.
  Windows already favors the threads of the window in the foreground.
- **Cap the frame rate.** Drawing one frame per refresh, timed to finish just before
  vertical sync, would save power, but the input it shows would be older, and a frame
  that finishes late waits a whole refresh.
- **Change how GameInput's helper is scheduled.** Keyboards and mice reach GameInput
  through its `GameInputRawInputProxy.exe`. On the development machine, a key injected
  with `SendInput` took about 0.2 ms to reach the demo. Opting the helper out of power
  throttling too made no difference, and forcing it into efficiency mode added about 0.1 ms.

The cost is power: the main thread keeps a CPU core busy, and the GPU draws thousands of
frames a second, even when nothing changes. Minimizing the window stops both.

## Testing

The demo reads devices and readings from an `InputSource`: GameInput's in a normal run, and a
scripted one, in `Source/ScriptedSource.rs`, in tests and scripted runs. So everything after
GameInput, from which gamepad it listens to, to dropping input without focus, the rumble test,
the shortcuts and the log, runs without a gamepad, and without your keyboard and mouse.

```powershell
cargo test -p game_input_demo_tests
Program\RunInBackground\RunInBackground.ps1 GameInputDemo --all-scripts
```

The tests in `Test/GameInputDemo` feed the demo's input directly, and check where the select
box and its list are in the panels' real layout. The scripts in `Test/GameInputDemo/Scripts`
run the real demo, in its window, as
[Library/Application](../../Library/Application/README.md#scripts) describes. Their `Program`
steps connect and disconnect devices, and give their readings:

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

Each reading is a device's whole state, as GameInput's are: what a step leaves out is at rest,
except that a mouse's `Motion` and `Wheel` add to its totals, as a real mouse's do.

| Step         | Members                                                                         |
| ------------ | ------------------------------------------------------------------------------- |
| `Connect`    | `Id`, a whole number from 1, `Name`, `Input` from `Gamepad`, `Keyboard` and     |
|              | `Mouse`, and `Motors` from `LowFrequency`, `HighFrequency`, `LeftTrigger` and   |
|              | `RightTrigger`                                                                  |
| `Gamepad`    | `Id`, `LeftStick` and `RightStick` as `[x, y]`, `LeftTrigger`, `RightTrigger`,  |
|              | and `Buttons`, named as the gamepad panel names them                            |
| `Keyboard`   | `Id`, and the keys `Down`, like `F1`, `A` or `Left`                             |
| `Mouse`      | `Id`, `Motion` and `Wheel` as `[x, y]`, and `Buttons`, named as the mouse panel |
|              | names them                                                                      |
| `Disconnect` | The device's `Id`                                                               |

The state that scripts save and check has what the panels show, the log's 10 newest lines, the
rumble that the demo sends each gamepad, whether the select box's list is open, as
`PickerOpen`, and where the select box and each of its choices are, as `Targets`.

What only real devices show still needs a person: GameInput's own readings and device events,
how the motors feel, and focus changing while a gamepad is held.

### With SendInput

Input sent with `SendInput` reaches GameInput as from an "Unidentified keyboard" or
"Unidentified mouse" with ID `0000:0000`. Injected input can be lost for about half a
second after the demo starts, while GameInput adds that device. It goes to the window that
has focus, so it's for trying the demo by hand, not for coding agents.

The virtual gamepad that Windows' `InputInjector.InitializeGamepadInjection` creates doesn't
reach GameInput.

## Using more APIs

For Windows APIs, which `windows-bindgen` generates, see
[Library/WindowsApi](../../Library/WindowsApi/README.md).

For GameInput APIs, see [Library/GameInput](../../Library/GameInput/README.md#using-more-apis).

## Layout

| Path                               | Contents                                                                         |
| ---------------------------------- | -------------------------------------------------------------------------------- |
| `Source/Main.rs`                   | The main loop: settings, window, input, and drawing each frame                   |
| `Source/Library.rs`                | The demo's parts, as a library that `Test/GameInputDemo` tests                   |
| `Source/Settings.rs`               | The demo's own setting, `RumbleTestInnerDeadZone`                                |
| `Source/Devices.rs`                | Devices and readings as plain data, and `InputSource`, which gives them          |
| `Source/GameInputSource.rs`        | Reads devices and readings with GameInput                                        |
| `Source/ScriptedSource.rs`         | Devices and readings from tests and scripts, instead of GameInput                |
| `Source/Input.rs`                  | Keeps what the panels show, from the devices and readings                        |
| `Source/Panels.rs`                 | Lays out the panels in fixed pixels, and draws them                              |
| `HumanRun.ps1`                     | Builds the demo and runs it on the primary display, for people to use it         |
| `HumanRun.cmd`                     | Runs `HumanRun.ps1`, for double-clicking in File Explorer                        |
| `Settings.json`                    | The [settings file](#settings-file), which the demo writes if it's missing       |
