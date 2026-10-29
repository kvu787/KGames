# DisplayIdentification

Launches apps and games on the display you choose. Windows 11 only.

Windows has no setting for which display a program opens on. Most programs open on the primary display, or wherever they were last time.
Windows display numbers such as `DISPLAY5` don't help much either.
They change when the computer restarts, when drivers update, and when monitors reconnect, and identical monitors share the same name.

DisplayIdentification gives every display a key built from the monitor's own EDID data: its model plus its serial number, such as `DEL41B6-ABC1234`.
The key stays the same across restarts and ports.
With it, `DisplayIdentification launch` starts a program and puts its windows on that display.

## Building

Requires the .NET 10 SDK.

```powershell
.\Publish.ps1
```

This creates `Output\DisplayIdentification.exe`, which needs the .NET 10 Desktop Runtime (included with the SDK).
Put it wherever you like; shortcuts and Steam point to it.

## Finding a display's key

```powershell
DisplayIdentification list
```

```text
Key              Name         Device    Resolution  Position  Primary
DEL41B6-ABC1234  DELL U2720Q  DISPLAY2  3840x2160   -3840,0
DEL41B6-XYZ9876  DELL U2720Q  DISPLAY1  3840x2160   0,0       Yes
```

To see which key belongs to which monitor, show each key on its own display:

```powershell
DisplayIdentification identify
```

`list --json` also prints each display's device path and connection type.

Wherever a command takes `--display`, any of these work:

| Display           | Meaning                                                                       |
| ----------------- | ----------------------------------------------------------------------------- |
| `primary`         | The current primary display.                                                  |
| `DEL41B6-ABC1234` | A key. Recommended, since it stays the same.                                  |
| `ABC1234`         | A serial number.                                                              |
| `DELL U2720Q`     | A name or model (`DEL41B6`), if only one connected display has it.            |
| `DISPLAY2`        | A Windows display device name. These change over time.                        |
| `\\?\DISPLAY#...` | A device path from `list --json`, which identifies a monitor on a given port. |

## Launching

```powershell
DisplayIdentification launch --display DEL41B6-ABC1234 -- "C:\Games\Game\Game.exe" --some-game-argument
```

Everything after `--` is the program and its arguments.
The program can also be a shortcut (`.lnk`), a document, or a URL such as `steam://rungameid/...`.
The launcher waits until the program exits and returns its exit code.

Started from a shortcut or Steam, the launcher has no console window.
Errors show in a message box, and the last launch is logged to `%LOCALAPPDATA%\DisplayIdentification\LastLaunch.log`.

### Methods

| Method             | Use it for                                                                     |
| ------------------ | ------------------------------------------------------------------------------ |
| `--method move`    | Default. Apps, and windowed or borderless fullscreen games.                    |
| `--method primary` | Exclusive fullscreen games, and games that always open on the primary display. |

`move` moves the program's windows to the display as they appear, for `--watch-seconds` (default 30).
Windows keep their place relative to the display, maximized windows stay maximized, and borderless fullscreen windows fill the new display.

`primary` does the same, and also makes the display the primary display until the program exits.
Then the original display becomes primary again.
The arrangement of the displays doesn't change, but the taskbar and desktop icons move to that display while the program runs.
`--restore-delay <seconds>` restores the primary display after that many seconds instead.
Some games handle that well once they are running; others jump back to the primary display.

If the launcher is killed before it can restore the primary display, the next launch restores it.
`DisplayIdentification restore` also does it right away.

### Launchers and URLs

The launcher follows the process it starts and every process that one starts.
Some programs hand off to a process that isn't their child, such as a game started through a `steam://` URL, or a game that restarts itself through Steam.
Name the game's process with `--process-name`, which can be given more than once:

```powershell
DisplayIdentification launch --display DEL41B6-ABC1234 --process-name Game.exe -- steam://rungameid/123456
```

### Shortcuts

Set a shortcut's target to the launcher, followed by the arguments:

```text
"C:\Tools\DisplayIdentification.exe" launch --display DEL41B6-ABC1234 -- "C:\Games\Game\Game.exe"
```

### Steam

In a game's Properties, under Launch Options:

```text
"C:\Tools\DisplayIdentification.exe" launch --display DEL41B6-ABC1234 -- %command%
```

Steam replaces `%command%` with the game, so Steam starts the launcher and the launcher starts the game.

## Other commands

| Command                                   | What it does                                                                      |
| ----------------------------------------- | --------------------------------------------------------------------------------- |
| `set-primary --display <display>`         | Makes a display the primary display, like Settings does.                          |
| `set-primary --display <display> --check` | Only checks that Windows would accept the change.                                 |
| `restore`                                 | Undoes a temporary primary display change that an interrupted launch left behind. |
| `help`                                    | Shows all commands and options.                                                   |

## Limitations

- Windows of programs that run as administrator can only be moved when the launcher also runs as administrator.
- A game in exclusive fullscreen can't be moved. Use `--method primary`.
- Identical monitors that don't report serial numbers get the same key. Use their device paths from `list --json` instead.
- Starting without a console window needs Windows 11 24H2 or later.
