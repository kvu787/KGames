# RunInBackground

Builds one of the GUI programs in `Program`, opens it on one display without taking focus,
saves a screenshot of its window, and closes it. Or runs it with scripts of input, which test
it without your mouse and keyboard. It's how coding agents run the programs while you work in
other windows (see the [AGENTS.md](../../AGENTS.md) at the root), or while your machine is
locked, which works the same.

```powershell
Program\RunInBackground\RunInBackground.ps1 GameInputDemo
Program\RunInBackground\RunInBackground.ps1 SimplePaintDemo --display KXGR16 --keep-open
Program\RunInBackground\RunInBackground.ps1 SimplePaintDemo --script Test\SimplePaintDemo\Scripts\DragSlider.json
Program\RunInBackground\RunInBackground.ps1 SimplePaintDemo --all-scripts
```

The program opens with `--display`, `--no-activate`, `--no-input` and `--vsync`, which every
program built on [Library/UserInterface](../../Library/UserInterface/README.md) takes, so it opens
beneath the other windows on that display, without taking focus, and typing in the window that has
focus stays there. It doesn't start GameInput, or the helper process that GameInput starts to read
keyboards and mice. Its frames wait for vertical sync, at most one a refresh, and it waits between
them instead of spinning, so it's light on the GPU and the CPU for someone who's using the
computer, like playing a game. Without focus, programs ignore all input. A run fails if the
program's window ever becomes the foreground window, which would mean that it took focus from you.
Windows reports every change of the foreground window to RunInBackground, so even one that lasts a
moment counts, and RunInBackground stops the program as soon as it does, so that it has focus for
as short a time as can be.

Every program also starts with vertical sync on by itself, but a run's `--vsync` keeps it on even if
the program's Settings.json says `"VerticalSync": false`.

Scripts run on their own fixed clock, so their frames and expectations are the same with vertical
sync or without. `--no-vsync` runs the program with `--no-vsync`, as fast as it can draw, like for
measuring frame times, which keeps a core and the GPU busy. A script's `F10` switches vertical sync
as the user's would, so the program draws as fast as it can from then until the script presses it
again, or ends. The state's `VerticalSync` says which. It starts on, or off with `--no-vsync`, and
UiShowcase's `VerticalSync.json` expects it to start on.

RunInBackground runs [lightly](../RunLightly/README.md), and so do the build and the program that it
starts: only on the CPU's efficiency cores, at idle priority, with at most a third of the CPU's time.

The program runs in a job that ends it when RunInBackground exits, unless `--keep-open` says
to leave it running. So a run that's stopped partway, like by Ctrl+C or a coding agent's time
limit, doesn't leave it running on the display. A program that's kept open holds a handle to
RunLightly's job, which would otherwise end it with RunInBackground, so it keeps running with the
job's limits until it's stopped.

The script only runs `RunInBackground.cs`, a .NET file-based app, with `dotnet run`, which
builds it into its own cache first. It needs the .NET 10 SDK.

| Argument              | Value                                                                      |
| --------------------- | -------------------------------------------------------------------------- |
| `<program>`           | The program to run, like `GameInputDemo`: its folder in `Program`, and its |
|                       | exe's name                                                                 |
| `--display <id>`      | The identifier of the display to open on. `CVVNY2` by default              |
| `--seconds <seconds>` | How long the program runs before the screenshot. 3 by default              |
| `--screenshot <path>` | Where to save the screenshot, as a PNG.                                    |
|                       | `target\RunInBackground\<program>.png` by default                          |
| `--keep-open`         | Leaves the program running after the screenshot                            |
| `--debug-build`       | Builds with Cargo's dev profile, which turns on the D3D12 debug layer      |
| `--no-vsync`          | Runs without vertical sync, drawing frames as fast as it can. Runs are     |
|                       | vertically synced by default                                               |
| `--script <path>`     | Runs the program with a script of input instead, and saves what it         |
|                       | captures in `target\RunInBackground\<program>\<script>`                    |
| `--all-scripts`       | Runs every script in `Test\<program>\Scripts`, one at a time               |
| `--timeout <seconds>` | How long a script may run before it fails. 60 by default                   |

The screenshot is of what the program draws in its window, without the title bar. It comes
from `PrintWindow`, so it doesn't matter if other windows are in front of the program's. What
the program writes to standard output and standard error, like why it couldn't start, goes to
`target\RunInBackground\<program>Output.log` and `<program>Errors.log`.

## Scripts

A script run passes the program `--script` and `--output`, and the program takes its input
from the script instead of you, on a fixed clock, as
[Library/UserInterface](../../Library/UserInterface/README.md#testing) describes. It opens on the
display without taking focus, like any run, and exits after the script's last step. The run
fails if the program exits with an error, which includes expectations in the script that
weren't met, if it runs past `--timeout`, or if its window ever becomes the foreground window.

RunInBackground prints whether each script passed, and the frames it captured. Each script's
folder in `target\RunInBackground\<program>` has its captures, as `<name>.png`, the program's
state at each, as `<name>.json`, and `Output.log` and `Errors.log`. Files that the program
writes, like SimplePaintDemo's `Paint.json`, go there too, not in the program's folder.
