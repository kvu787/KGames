# PresentTiming

When each of a process's presents reached the screen: the time that the display took the frame,
which [Library/UserInterface](../UserInterface/README.md#input-latency) measures input latency to.
It's KGames' own code, which follows the rules that
[PresentMon](https://github.com/GameTechDev/PresentMon) 2.6.1 follows, so its times are the same as
PresentMon's: its "screen time", which PresentMon's `MsUntilDisplayed`, `DisplayLatency` and
`MsAllInputToPhotonLatency` end at.

## How it works

Windows says what happens to every present in its own events, which a real-time ETW session
(`Source/Session.rs`) reads on a thread of its own, as they happen:

| Provider                     | What it says                                                                    |
| ---------------------------- | ------------------------------------------------------------------------------- |
| `Microsoft-Windows-DXGI`     | A DXGI `Present` call started and ended, on which thread                        |
| `Microsoft-Windows-D3D9`     | The same, for Direct3D 9                                                        |
| `Microsoft-Windows-Win32k`   | The present's token for composition, and whether DWM composed it or skipped it  |
| `Microsoft-Windows-DxgKrnl`  | The present's copy or flip, its queue packet, and the display's sync interrupts |
| `Microsoft-Windows-Dwm-Core` | DWM's frames, which composed presents show in, and the copies that DWM takes    |
| `NVIDIA-DD-External`         | When NVIDIA's driver means to flip, if that's later than the kernel's flip      |

The session enables the same events, levels and keywords as PresentMon. `Source/Decoder.rs` finds
the events' fields by name, with the layouts that Windows' trace data helper (TDH) gives from the
providers' manifests, or from a trace file's own metadata for versions that this computer's
manifests don't have. `Source/Events.rs` turns them into plain data, and `Source/Tracker.rs`
follows each present through them, by the keys that each event has: its thread while its `Present`
call runs, then Win32k's token, the kernel's present history token, and its queue packet's submit
sequence, which the flips and sync interrupts name.

The presents are of any API: DXGI's and Direct3D 9's, whose `Present` calls have events of their
own, and others', like OpenGL's or Vulkan's, which only the kernel's and Win32k's events tell of.
Windows 7's events aren't read.

A present that a newer one replaced before the screen took it, that the kernel found it didn't need
to copy, or that DXGI or Direct3D 9 says is occluded, never reaches the screen. Like PresentMon, the
first present to complete, and the ones under way then, are taken as lost, since their earlier
events may not have been seen. PresentMon's console then puts a present that NVIDIA's flip delay
would put before the one before it on the screen with that one, which `Corrector` does too.

## Present modes

A present's mode is how it got to the screen. There's one for every mode that PresentMon tells
apart:

| Mode                | PresentMon's name                       |
| ------------------- | --------------------------------------- |
| `Independent flip`  | `Hardware: Independent Flip`            |
| `Overlay plane`     | `Hardware Composed: Independent Flip`   |
| `Composed flip`     | `Composed: Flip`                        |
| `Legacy flip`       | `Hardware: Legacy Flip`                 |
| `Legacy copy`       | `Hardware: Legacy Copy to front buffer` |
| `Composed GPU copy` | `Composed: Copy with GPU GDI`           |
| `Composed CPU copy` | `Composed: Copy with CPU GDI`           |
| `Unknown`           | `Other`                                 |

Each follows its own path through the events, which says when the present was on the screen:

- **Independent flip**: the display took a flip model swap chain's buffer itself, without DWM,
  like in borderless fullscreen. It's on the screen when the flip happens, if it's immediate and
  can tear, or else at the sync interrupt of the refresh that takes it.
- **Overlay plane**: the same, on one of the display's overlay planes, over the desktop's, which
  lets it skip DWM while other things on the screen, like a title bar or the taskbar, don't. The
  same.
- **Composed flip**: DWM drew a flip model swap chain's buffer into its own frame. It's on the
  screen when DWM's frame is, which shows only the newest present of each window.
- **Legacy flip**: the process owned the display, in fullscreen exclusive mode, which showed its
  buffer. As an independent flip. DWM's own frames go to the screen this way too.
- **Legacy copy**: the process owned the display, and copied its buffer into the one that the
  display showed. When the copy is done.
- **Composed GPU copy**: a bitblt model swap chain's buffer was copied on the GPU into the
  window's surface, which DWM drew into its own frame. As a composed flip.
- **Composed CPU copy**: the same, copied on the CPU. As a composed flip.
- **Unknown**: its events didn't say.

Which a process can have depends on how it presents. KGames' programs present with Direct3D 12,
which only has flip model swap chains, in a window or borderless fullscreen, never fullscreen
exclusive, so theirs are independent flips, overlay planes and composed flips. A game that
[FrameTimes](../../Program/FrameTimes/README.md) measures can have any: Direct3D 9, 10 and 11 have
bitblt model swap chains, which copy, and every API but Direct3D 12 can take fullscreen exclusive
mode.

As in PresentMon, two more ways aren't modes of their own. DWM flipping the process's buffer to
the screen itself, without drawing it into a frame of its own, is a composed flip, as Windows'
events don't tell them apart. And presents to DirectComposition's composition swap chains are left
out, as they can't be followed to DWM's frames.

## Privileges

Starting an ETW session with these providers needs an elevated administrator, or membership in
the **Performance Log Users** group. Without either, `PresentTiming::start` fails with
`StartError::AccessDenied`, and the programs' PerfBars say so. To join the group, once,
from an elevated PowerShell, then sign out and back in:

```powershell
Add-LocalGroupMember -Group 'Performance Log Users' -Member $env:USERNAME
```

The session is named after the program and its process, like `KGames UiShowcase 4172`. Sessions
are the system's, and outlive a process that ends without stopping its own, like one that's
killed, so a program stops the `KGames` sessions whose processes are gone before it starts its
own, and leaves another process's running session alone.

## Checking against PresentMon

`present_timing::read_file` reads a trace file, like one that xperf or WPR saved with these
providers, and gives a process's presents with the performance counter's frequency, to compare
with what `PresentMon --etl_file` says of the same file: a present's `TimeInQPC` is its `start`,
and `MsUntilDisplayed` is its `screen` less its `start`, in milliseconds.

`Test/PresentTiming/Gold.rs` does that for the six gold traces in PresentMon's repository
(`Tests/Gold`), from Windows 10, without running PresentMon: it compares with the gold CSVs beside
them, which PresentMon 2.6.1's own tests require its output for the traces to match, row for row.
All 1,448 presents in them, of every process, DXGI's, Direct3D 9's and others', have the same
mode, tearing, and time on the screen, to a millionth of a millisecond: 858 legacy flips, 329
composed flips, 147 composed GPU copies, 81 independent flips, 31 overlay planes and 2 unknown.
`read_file` gives two presents of each swap chain that PresentMon's console doesn't write: its
first, and its last, which PresentMon writes only once the next one reaches the screen.

The traces aren't in this repository, so the test is ignored unless it's asked for, with
`PRESENTMON_GOLD` naming a folder that has them:

```powershell
$env:PRESENTMON_GOLD = 'C:\PresentMon\Tests\Gold'
Program\RunLightly\RunLightly.ps1 cargo test -p present_timing_tests --test Gold '--' --ignored --nocapture
```

The traces don't have legacy copies, composed CPU copies, or Windows 11's hardware flip queue,
whose flips' status after the flip tells immediate flips from ones that wait for the sync
interrupt. `Test/PresentTiming/Tracker.rs` tests the tracker with made-up events in the order that
Windows sends them, for every mode.

## Layout

| Path                | Contents                                                                        |
| ------------------- | ------------------------------------------------------------------------------- |
| `Source/Library.rs` | `PresentTiming`, `Present`, `read_file`, and the NVIDIA flip delay's correction |
| `Source/Session.rs` | The real-time ETW session, and reading trace files                              |
| `Source/Events.rs`  | The providers, their events, and the events as plain data                       |
| `Source/Decoder.rs` | Finds events' fields by name, with TDH's layouts                                |
| `Source/Tracker.rs` | Follows presents through the events to when the display took them, and how      |
