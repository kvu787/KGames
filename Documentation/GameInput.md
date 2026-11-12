# GameInput

How GameInput reads gamepads, what makes it slow, and what KGames does about it, as found on
2026-11-12, starting from two reports about GameInputDemo: the sticks' graphs moved less smoothly
than the mouse's pad, and an 8,000 Hz controller didn't seem to show its rate. The short of it:

1. **KGames caused it.** Commit `e1d1315`, on 2026-11-09, turned on GameInput's *background
   input*, so that the input reader could follow keys held while the window didn't have focus.
   With background input on, GameInput reads a gamepad in Xbox 360 mode on a timer instead of as
   it reports, and with Windows' default timer resolution that timer fires every 15.6 ms: 64
   readings a second, so a stick's graph moved 64 times a second on a 120 Hz display.
2. **With background input off, GameInput's default, a focused window gets nearly every report.**
   Two GameSir controllers in Xbox 360 mode, set to report 8,000 times a second, gave about 7,500
   to 7,700 readings a second, with no timers or helper threads. KGames now leaves it off, and
   `Library/GameInput` doesn't bind `EnableBackgroundInput`, so turning it on takes a deliberate
   edit.
3. **GameInput handles focus changes itself, without background input.** As a window loses focus,
   it gets a reading of each device at rest, which lets go of everything held. As focus comes
   back, keys and mouse buttons held from before aren't reported at all, and a gamepad's whole
   state is, whose held buttons the interface already ignores until they're let go of. So
   background input wasn't needed for what it was turned on for.
4. **The workarounds for the timer were dead ends, but taught how GameInput polls.** 1 ms timers
   took the timer's 64 readings a second to about 700, and a thread waking every 0.5 ms took them
   to about 950, GameInput's ceiling for its timer. Both rest on GameInput's undocumented
   behavior, and both are gone.

## How it was measured

| What         | Value                                                                                   |
| ------------ | --------------------------------------------------------------------------------------- |
| CPU          | Intel Core Ultra 9 275HX, 8 performance and 16 efficiency cores, 24 threads             |
| GPU          | NVIDIA GeForce RTX 5090 Laptop GPU                                                      |
| Displays     | Three ASUS PA278QGV, 2560 x 1440 at 120 Hz, 100% scale: `CVVNY2`, `9XH36C`, `KXGR16`    |
| GameInput    | 4.0.306, installed; the same `GameInputRedist.dll` in System32, Program Files and NuGet |
| Windows      | 11 Pro 10.0.26300                                                                       |
| Toolchain    | rustc 1.99.1; MSVC from Visual Studio 18 for the C++ probes                             |
| Controller 1 | GameSir G7 Pro 8K, on its 2.4 GHz receiver, in its XInput and Switch modes              |
| Controller 2 | GameSir Tarantula 8K PC, wired, in its Xbox mode                                        |

Coding agents may not take focus, send input, or open a window any way but RunInBackground,
which runs programs with `--no-input`. So the measurements came from five kinds of tools, which
lived in the session's scratchpad and aren't in the repo:

- **A console probe**, in C++ against GameInput's v4 `GameInput.h` and `GameInput.lib`: it turns
  on background input, walks the reading stream with `GetNextReading`, and records every
  reading's timestamp, device and state, with the time it was retrieved. Options set the
  process's timer resolution (`timeBeginPeriod`, or `NtSetTimerResolution` for 0.5 ms), the kind
  of readings counted, whether background input is on, and a wait for a stick to move.
- **A temporary bisect test** in `Test/UserInterface`, which read gamepads through the library's
  own `Devices` once every 8 ms, like the main loop, and added the demo's parts one at a time by
  environment variables: power throttling's opt-out, the PerfBar's ETW session, a Direct3D 12
  device, DirectWrite, DPI awareness, a window, a swap chain, presenting, a main thread that
  blocks a whole frame, a thread that wakes on a high-resolution timer, and more.
- **Instrumented builds of GameInputDemo**, which logged every gamepad reading's timestamp, the
  window's focus each frame, how long a 1 ms wait took in the process, and later every keyboard,
  mouse and gamepad reading around focus changes. The user ran them focused, and, with the
  user's permission, the agent ran them on `CVVNY2` with `--no-activate` and GameInput on.
- **Old builds** of the demo from `git archive`: `35c0c0f`, before the runtime switch;
  `9585e64`, between the runtime switch and background input; and `e1d1315` itself.
- **Direct reads of Windows' HID devices** with `ReadFile`, which count the reports a device
  sends without GameInput, and Windows' device history, `DEVPKEY_Device_LastArrivalDate`, which
  showed what each controller called itself in each mode.

Only one program read GameInput at a time, except where said, as a second one changes what the
first gets (see [Two programs at once](#two-programs-at-once)).

## How GameInput reads a gamepad in Xbox 360 mode

Both controllers' XInput modes make them an Xbox 360 controller to Windows: Microsoft's XUSB
driver, `xusb22`, runs them, and GameInput names them "Xbox 360 Controller for Windows". The G7
is `3537:10C6`, and the Tarantula `3537:103D`. Most third-party PC controllers work this way, so
that every XInput game sees them.

The driver keeps the controller's latest state, and GameInput doesn't get each report as it
comes. Windows' DirectInput copy of the Tarantula's input, the HID device
`HID\VID_3537&PID_103D&IG_00`, read with `ReadFile`, gave 7,687 reports a second, 125 µs apart
at the middle, so the 8 kHz reaches Windows. What GameInput makes of it depends on background
input.

### With background input on: a timer

GameInput reads the controller on a timer of about 1 ms in the reading process, so the process's
timer resolution decides how often. Windows 11 fires a process's timers on a 15.6 ms tick unless
that process asks for finer ones, whatever other processes ask for: with the system's
resolution at 1 ms from other processes, a 1 ms `Sleep` in a process that hadn't asked still took
15.7 ms at the middle. Readings a second, from the G7 and the Tarantula, at 1 kHz and 8 kHz
alike:

| The reading process                                          | Readings a second | Gaps between readings |
| ------------------------------------------------------------ | ----------------- | --------------------- |
| Windows' default timers                                      | 64                | 15.6 ms               |
| `timeBeginPeriod(1)`, main thread blocking a frame at a time | 650 to 700        | 1.0 and 2.0 ms, mixed |
| The same, with a thread waking every 4 ms                    | 699               |                       |
| The same, with a thread waking every 2 ms                    | 757               |                       |
| The same, with a thread waking every 1 ms                    | 656 to 661        | mostly 1.5 ms         |
| The same, with a thread waking every 0.5 ms                  | 945 to 960        | 1.0 ms                |
| The same, with a thread waking every 0.25 ms                 | 957               | 1.0 ms                |
| 0.5 ms resolution (`NtSetTimerResolution`), blocking         | 789 to 811        | 1.0 and 1.5 ms        |
| 0.5 ms resolution and a thread waking every 0.5 ms           | 973               | 1.0 ms                |
| A thread waking every 0.5 ms, with Windows' default timers   | 64                | 15.6 ms               |

The waking thread waits on a high-resolution timer (`CREATE_WAITABLE_TIMER_HIGH_RESOLUTION`),
which makes Windows wake at its exact time, and Windows then also fires the process's other
timers that are due, GameInput's among them. That's the reading of these numbers; GameInput's
documentation says nothing of how it polls. What they show for sure:

- **About 1,000 readings a second is the most this timer gives**, however often the controller
  reports: a 1 kHz and an 8 kHz setting gave the same numbers.
- **A program like the demo got about 700.** Its main thread waits a whole frame at a time, for
  the swap chain, and wakes nothing in between.
- **A probe that polls in a loop measures itself.** The first probe and the first bisect test read
  about 950 a second because their own loops waited on high-resolution timers. Measure with a
  main thread that blocks a frame at a time, like the main loop's.

Things that made no difference, with 1 ms timers and a fast loop: power throttling's opt-out, the
PerfBar's ETW session, a Direct3D 12 device, DirectWrite, DPI awareness, a window shown or hidden,
a swap chain that never presented, and GPU work each frame without presenting, all 938 to 948.
Presenting with vertical sync gave 650 to 671, which first looked like the cause: it's the main
thread blocking on the swap chain between frames. Presenting without vertical sync, with short
high-resolution sleeps between frames, gave 857. The window's focus didn't matter either: the
demo got the same 690 to 700 focused and not.

### With background input off: every report

A focused window got about 7,500 readings a second from either controller set to 8 kHz in the
two old builds that left background input off, which had no 1 ms timers or waking thread, and
smooth stick graphs. The build that turned background input on, `e1d1315`, gave 64 readings a
second, and choppy graphs. The user read these on GameInputDemo's Gamepad card, focused:

| Build                                  | Runtime                | Background input | 1 ms timers | Readings a second |
| -------------------------------------- | ---------------------- | ---------------- | ----------- | ----------------- |
| `35c0c0f`, before the runtime switch   | Beside the exe         | Off              | No          | about 7,500       |
| `9585e64`, after it                    | Installed, in System32 | Off              | No          | about 7,500       |
| `e1d1315`, which turned it on          | Installed              | On               | No          | 64                |
| `77d8d76`                              | Installed              | On               | Yes         | about 700         |
| `77d8d76`, background input turned off | Installed              | Off              | Yes         | about 7,700       |
| `b585421`, the fix                     | Installed              | Off              | No          | 7,500 to 7,700    |

Without focus, with background input off, GameInput gives no gamepad readings, but device events
still come: a probe without focus saw both controllers connected, and got only one reading each
from the Tarantula's keyboard and mouse parts.

## Focus without background input

What GameInput sent the demo around focus changes, logged with GameInput's own timestamps while
the user held keys and buttons across them:

| When                                | What GameInput sends                                       |
| ----------------------------------- | ---------------------------------------------------------- |
| The window loses focus              | A reading of every device at rest, a few milliseconds      |
|                                     | before the frame that saw the change, then nothing         |
| Focus comes with Shift held         | Nothing for the keyboard: not the held Shift, nor its      |
|                                     | release                                                    |
| The click that gives focus          | Nothing for its button                                     |
| Focus comes with a gamepad's A held | The gamepad's whole state, A held, before the frame that   |
|                                     | saw the change, then A's release                           |
| A key pressed after focus came      | A reading, as usual                                        |

So for keyboards and mice, GameInput does what its documentation promises: programs need nothing
of their own for gaining and losing focus. For gamepads it reports the state, and the library's
interface ignores a gamepad's buttons held as focus comes until they're let go of, as
GameInputDemo's masks do. Both go by the readings' timestamps, so a reading from before the
window got focus is input without it even if it comes in a later frame. As focus goes, the
interface lets go of everything held on that frame, keys with a release like mouse buttons,
whichever comes first of GameInput's readings at rest and the window's focus change.
`Test/UserInterface/tests/reader.rs` and `Test/GameInputDemo/tests/input.rs` pin these, with
readings shaped like GameInput's: a key held as focus goes is released once, and counts again once
it's pressed with focus back, like an Alt+Tab away, a click back and an Alt+F4, and a gamepad
button held as focus comes counts once it's let go of, even when GameInput's reading of it comes a
frame late.

In the four times focus went and the five times it came that were logged, GameInput's readings
for them came 2.4 to 40 ms before the demo's frame that saw the change. That's an observation,
not a promise. The tests try the readings at rest both before and after the window loses focus,
and both work.

One thing changes for programs that close on a click: GameInput doesn't report the click that
gives a window focus, so IdentifyDisplays' boxes need a second click if another window took
focus. Each box now says so on its last line, with a status icon, and draws at every refresh with
vertical sync, like any program, so that a display with a variable refresh rate sees it present
steadily.

## What Microsoft says

From GameInput's documentation, paraphrased:

- **Off is the default.** By default, GameInput gives an application input only while it's in the
  foreground with focus. `EnableBackgroundInput(true)` also gives it input while another
  application has focus, for every kind of input, the Guide and Share buttons included, which it
  then claims for the application. `EnableBackgroundInput(false)` returns to the default. It came
  in GameInput v4, replacing `SetFocusPolicy`.
- **Focus changes need nothing of programs.** GameInput stops a device's haptics and force
  feedback when an application loses focus, and sends its last requested state again when focus
  comes back.
- **A sideloaded runtime ignores it.** With `GameInputRedist.dll` beside the exe instead of
  installed, enabling background input does nothing, as only the installed GameInput service can
  claim the system buttons, and that service is also what manages focus, so GameInput doesn't
  filter input by focus consistently: some devices give input without focus, and some don't. A
  probe with the sideloaded runtime and no focus got the Switch-mode G7's readings, 941 a second,
  and none of the Xbox 360-mode Tarantula's.
- **The documentation says nothing** of background input changing how often gamepads are read, of
  switching it with focus, or of how GameInput polls.

Sources: [IGameInput::EnableBackgroundInput](https://docs.xbox.com/reference/input/gameinput/interfaces/igameinput/methods/igameinput_enablebackgroundinput.md),
[GameInput fundamentals](https://docs.xbox.com/build/core-features/common/input/overviews/input-fundamentals.md),
[GameInput sideloading](https://docs.xbox.com/build/core-features/common/input/overviews/input-agility-redistributable.md),
[GameInputFocusPolicy](https://docs.xbox.com/reference/input/gameinput/deprecated/enums/gameinputfocuspolicy.md),
[GameInput work queues](https://docs.xbox.com/build/core-features/common/input/advanced/input-work-queues.md).

## GameInput's work queue

GameInput's background work runs in a queue, on a worker thread of its own at above-normal
priority, unless a program takes the queue over with `IGameInput::CreateDispatcher` and runs it
itself with `Dispatch`. The documentation names device events and callbacks as its work. With
background input on, 1 ms timers, a main thread blocking 8 ms at a time, the Tarantula in Xbox 360
mode and the G7 in Switch mode:

| How the queue runs                                       | Tarantula, Xbox 360 mode | G7, Switch mode |
| -------------------------------------------------------- | ------------------------ | --------------- |
| GameInput's own worker thread                            | 679 to 685               | 923 to 938      |
| Taken over, and never run                                | none                     | none            |
| Run whenever its wait handle says there's work           | 685                      | 939             |
| Run every 0.5 ms, on a high-resolution timer             | 959                      | 933             |
| GameInput's own thread, and a thread waking every 0.5 ms | 950                      | 937             |

So no readings come without that queue running, and polling an Xbox 360-mode controller seems
to be work that a timer puts there: the queue's own signal that it has work comes as late as the
timer. Running the queue on time gives what the waking thread gives, at the cost of making all
input depend on the program's thread.

## Two programs at once

With background input on, a second program reading GameInput changed what the first got:

- A probe with 1 ms timers read about 950 a second alone. When a second probe started, the first
  dropped to 64 a second, and stayed there after the second one ended, while the second got about
  950.
- Two probes with 1 ms timers started together each got about 400 a second, with gaps of 1 ms
  and of 12 to 16 ms mixed.

Why isn't known. It doesn't matter with background input off, where only the focused program
gets input.

## The controllers

| Controller, mode      | Windows sees                 | Reports a second | GameInput's readings a second | Sticks |
| --------------------- | ---------------------------- | ---------------- | ----------------------------- | ------ |
| G7 Pro 8K, XInput     | Xbox 360, `3537:10C6`        | 1,000 or 8,000   | 7,500 to 7,700 at 8,000       | 16-bit |
| G7 Pro 8K, Switch     | Switch Pro, `057E:2009`, HID | 1,000            | 913 to 941, background on     | 12-bit |
| G7 Pro 8K, Android    | DualShock 4, `054C:09CC`     | not measured     | not measured                  |        |
| Tarantula 8K PC, Xbox | Xbox 360, `3537:103D`        | 7,687            | 7,500 to 7,700 at 8,000       | 16-bit |

GameInput's readings are a focused window's with background input off, but where it says
background on. The G7's reports in XInput mode are its setting, as they weren't read directly;
the Tarantula's were read from Windows' HID copy of its input, and the G7's in Switch mode from
its own HID interface.

- **Switch mode** is a HID device that GameInput maps as a gamepad, and reads as each report
  comes: 923 to 941 readings a second with any timers, of the 1,000 reports a second that the
  G7 sent in that mode with its rate set to 8,000, read directly from its HID interface. Its
  sticks step by 1/2046, 12 bits, where XInput mode's step by 1/32767. Games that read only
  XInput don't see it as a controller.
- **The G7's modes** cycle XInput, Switch, Android, XInput, holding View and Menu for 2 seconds,
  on the receiver or a cable. GameSir's manual for the G7 Pro 8K names only XInput and Switch;
  its FAQ, for other models, names XInput, Switch and Android. In the third mode the G7 was a
  DualShock 4 for about 10 seconds of the device history. GameSir Connect works only in XInput
  mode, as GameSir's FAQ says.
- **The Tarantula's modes** are chosen by the platform it's plugged into, or by hand: Home + X for
  Xbox, Home + Y for Switch, Home + A for Android. With its default dead zones, its sticks don't
  change at rest, and GameInput gave no readings of it then, so it seems that in Xbox 360 mode a
  reading only comes when the state changes. With its dead zones set to 0 in GameSir Connect, its
  sticks' noise gave readings at rest, like the G7's, whose sticks are never still.
- **Both turn themselves off** after 10 minutes without use, as their manuals say.

Whether an 8 kHz rate helps when GameInput polls at most once a millisecond: it can't add
readings, but each poll gets a report at most 0.125 ms old instead of 1 ms. That wasn't
measured: GameInput's readings don't say how old their report is.

## How fresh the readings were

The age of the newest gamepad reading when a frame took its readings, by GameInput's clock, in
the demo without focus, with background input on and the G7 at 8 kHz:

| The demo's process        | Mean    | Middle  | 90th percentile | 99th percentile |
| ------------------------- | ------- | ------- | --------------- | --------------- |
| 1 ms timers               | 0.98 ms | 0.90 ms | 1.75 ms         | 2.5 ms          |
| 1 ms timers, 0.5 ms waker | 0.72 ms | 0.67 ms | 1.11 ms         | 1.5 to 2.0 ms   |
| Switch mode, 1 ms timers  | 0.85 ms | 0.75 ms | 1.17 ms         | 2.17 ms         |

This wasn't measured with background input off.

## What changed in KGames

| Commit    | What                                                                                                 |
| --------- | ---------------------------------------------------------------------------------------------------- |
| `7a2c045` | `WindowsApi`: bindings for `timeBeginPeriod` and `timeEndPeriod`                                     |
| `040c39e` | Programs that read the gamepad held 1 ms timers, which took 64 readings a second to about 700        |
| `77d8d76` | GameInputDemo's Gamepad card shows readings a second averaged over the last 2 seconds, every frame   |
| `8c66f63` | Tests of focus changes, as GameInput gives them without background input                             |
| `b585421` | Background input off, `EnableBackgroundInput` unbound, and `040c39e`'s timers gone                   |
| `33225df` | `7a2c045`'s bindings gone                                                                            |
| `fc02159` | GameInputDemo's README says that only gamepads in Xbox 360 mode were measured                        |
| `748eee9` | Overlays draw at every refresh with vertical sync, and IdentifyDisplays says whether it has focus    |
| `4b11fbd` | A flaky benchmark test, found along the way: a heap snapshot's peak is never less than what it holds |

What was considered and not done:

- **1 ms timers**, which got about 700 readings a second, and cost power in every program that
  read the gamepad.
- **A thread waking every 0.5 ms**, which got about 950, at about 40 million cycles a second, about
  1% of a core, in a test process. If it failed, input fell back to about 700, so it was safe,
  but it rested on GameInput firing its timer when another timer wakes Windows, which GameInput
  doesn't promise.
- **Running GameInput's queue every 0.5 ms**, which got about 959, but would have made every kind of
  input depend on the program's own thread.
- **0.5 ms timers with `NtSetTimerResolution`**, an undocumented call, which got about 800.
- **Background input on only without focus**, which GameInput's API allows, but no documentation
  describes, and which turned out unneeded, as GameInput handles focus itself.
- **Switch mode**, which GameInput reads as reports come, but which sends only 1,000 a second, with
  12-bit sticks.
- **Reading the DirectInput HID copy directly**, which gets every report, but KGames reads all
  input with GameInput.

## Open questions

- **How GameInput polls.** The timer explanation fits every measurement but comes from timings,
  not from GameInput's source or documentation.
- **Why a second program reading GameInput slowed the first**, with background input on.
- **Why GameInput missed about 7% of the Switch-mode G7's 1,000 reports.**
- **Whether the G7's 8 kHz reaches Windows over its receiver.** Only the wired Tarantula's was read
  directly.
- **Whether GameInput's reading of a gamepad as focus comes always has a timestamp from before the
  window's focus change.** It did the five times logged, 2.4 to 40 ms before the frame that saw the
  change. A reading that comes in a later frame is still from before focus by its timestamp, but
  if its timestamp were ever after the window's focus change, a button held as focus came would
  count as a press.
- **The Android mode**, which wasn't measured.
- **How fresh readings are with background input off**, which wasn't measured.
- **What GameInput does with keys held as focus comes on other keyboards**, which only one keyboard
  showed.

When GameInput is updated past 4.0.306, these numbers may not hold. The quickest check is
GameInputDemo's Gamepad card, focused, with a controller in Xbox 360 mode: it should say several
thousand readings a second for an 8 kHz controller, not 64.
