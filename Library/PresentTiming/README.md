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

| Provider                     | What it says                                                                   |
| ---------------------------- | ------------------------------------------------------------------------------ |
| `Microsoft-Windows-DXGI`     | A `Present` call started and ended, on which thread                            |
| `Microsoft-Windows-Win32k`   | The present's token for composition, and whether DWM composed it or skipped it |
| `Microsoft-Windows-DxgKrnl`  | The present's queue packet, the flips, and the display's sync interrupts       |
| `Microsoft-Windows-Dwm-Core` | DWM's frames, which composed presents show in                                  |
| `NVIDIA-DD-External`         | When NVIDIA's driver means to flip, if that's later than the kernel's flip     |

The session enables the same events, levels and keywords as PresentMon. `Source/Decoder.rs` finds
the events' fields by name, with the layouts that Windows' trace data helper (TDH) gives from the
providers' manifests, or from a trace file's own metadata for versions that this computer's
manifests don't have. `Source/Events.rs` turns them into plain data, and `Source/Tracker.rs`
follows each present through them, by the keys that each event has: its thread while its `Present`
call runs, then Win32k's token, the kernel's present history token, and its queue packet's submit
sequence, which the flips and sync interrupts name.

A present is on the screen:

- **Independent flip** (the display takes the swap chain's buffer itself, like in borderless
  fullscreen): when the flip happens, if it's immediate and can tear, or else at the sync
  interrupt of the refresh that takes it.
- **Overlay plane** (the same, on one of the display's overlay planes): the same.
- **Composed** (DWM draws it into its own frame): when DWM's frame is, of which only the newest
  present of each window is shown.

A present that a newer one replaced before the screen took it, or that DXGI says is occluded,
never reaches the screen. Like PresentMon, the first present to complete, and the ones under way
then, are taken as lost, since their earlier events may not have been seen. PresentMon's console
then puts a present that NVIDIA's flip delay would put before the one before it on the screen with
that one, which `Corrector` does too.

Only the paths that DXGI's flip-model swap chains take are followed, which are the only ones
that KGames' programs use. Copies to the front buffer and composition atlases aren't, and legacy
fullscreen flips only as far as DWM's own frames need, never as a mode of the process's presents.

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

On the six gold traces in PresentMon's repository, from Windows 10, every flip-model present that
PresentMon reports, in 11 processes, independent flips, overlay planes and composed, has the same
screen time to PresentMon's printed 0.1 µs, and the same mode. `read_file` gives two presents of
each swap chain that PresentMon's console doesn't write: its first, and its last, which PresentMon
writes only once the next one reaches the screen. Those traces don't have Windows 11's hardware
flip queue, whose flips' status after the flip tells immediate flips from ones that wait for the
sync interrupt.

`Test/PresentTiming` tests the tracker with made-up events in the order that Windows sends them.

## Layout

| Path                | Contents                                                                        |
| ------------------- | ------------------------------------------------------------------------------- |
| `Source/Library.rs` | `PresentTiming`, `Present`, `read_file`, and the NVIDIA flip delay's correction |
| `Source/Session.rs` | The real-time ETW session, and reading trace files                              |
| `Source/Events.rs`  | The providers, their events, and the events as plain data                       |
| `Source/Decoder.rs` | Finds events' fields by name, with TDH's layouts                                |
| `Source/Tracker.rs` | Follows presents through the events to when the display took them               |
