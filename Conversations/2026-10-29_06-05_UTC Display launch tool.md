# Display launch tool

## User

i need a way to specify what display an app or game launches on. develop this in "C:\Users\K\Repository\kGames\DisplayIdentification".

## AI

I'm checking the exact manifest syntax for launching without a console window. After that I'll write the tool.

Windows already has three identical ASUS PA278QGV monitors that only differ by EDID serial, and their `\\.\DISPLAYn` numbers (5, 6, 7) are volatile. So the tool needs stable display keys as well as the launcher. I'll build it in stages: first display enumeration and identification, then primary-display switching, then launching.

Writing the command-line plumbing and the `list`/`identify` commands next, then a first build.

I left implicit usings off but wrote code that assumes them. I'll turn them on, minus the WinForms/Drawing ones so UI dependencies stay explicit.

The user's `.editorconfig` turns every enabled style suggestion into an error, so I'm matching its preferences: IDE0046 here. I'll also turn on the documentation-file setting so the build can check unused usings (IDE0005).

Clean build. Now running it against the real displays:

The table matches the WMI serials. JSON output fails because System.Text.Json won't accept a `required` property that is also `[JsonIgnore]`. Those two internal fields don't need `required`, so I'll remove it from them.

Serials line up with the WMI port IDs. I'll make the JSON skip HTML escaping (`&` shows as `\u0026`), then check the `identify` cards by capturing only the card areas at the center of each monitor.

The cards render correctly on each monitor and close on their own. Committing stage 1.

Stage 2: display selectors, primary-display switching via `SetDisplayConfig`, and crash recovery.

I'll expose the validate path as `set-primary --check`. It's a useful dry run, and it lets me test the `SetDisplayConfig` math here without actually moving the taskbar.

`SetDisplayConfig` accepts the shifted layout in validate mode for both non-primary displays, nothing changed, and the ambiguous name `PA278QGV` gets a helpful error. Committing stage 2.

Stage 3: the `launch` command. It needs ShellExecuteEx with a monitor hint, process-tree tracking, a window mover, and the temporary primary swap.

Fixing the last three IDE0046 style errors, then I'll test `launch` on real windows.

Build is clean. To test `launch`, I'll use a throwaway WinForms window (normal, maximized, and borderless-fullscreen variants). It opens on the primary display, reports where it ended up, then closes.

All three window states land correctly on the left display:

| Case       | Before                       | After                                                              |
| ---------- | ---------------------------- | ------------------------------------------------------------------ |
| Normal     | (200, 150) on DISPLAY5       | (-2360, 150) on DISPLAY6, same offset                              |
| Maximized  | maximized on DISPLAY5        | maximized on DISPLAY6, and its restore position is on DISPLAY6 too |
| Fullscreen | borderless covering DISPLAY5 | exactly covers DISPLAY6                                            |

Next I'll test the harder cases: a launcher that hands off and exits, a process started outside the tree (found via `--process-name`), error paths, and whether a shortcut-style start opens a console window.

The tool's safety check misread `cmd.exe /c` next to `Remove-Item`, so I'll put the test in a script file instead.

The hand-off case works: `cmd` exits, its `pwsh` child is still tracked, and the window lands on the right display. The `--process-name` case didn't, because the WMI-started window never appeared. I'll check whether WMI started that process at all before suspecting the tracker.

WMI can't start the Store-packaged `pwsh` (error 8), so the test setup was broken, not the tracker. I'll retest name tracking with `charmap.exe` started through WMI. Its parent is then WmiPrvSE, completely outside the launcher's process tree.

Name tracking works: `charmap` was found outside the tree and moved, and the launcher waited until it closed. Last check is the shortcut-style start. With `consoleAllocationPolicy=detached`, a launch from Explorer or a shortcut shouldn't create a console for the launcher.

No console host starts for the launcher, so shortcut launches won't flash a console window. The log file captured the run. Next, the parts of the primary method that don't change the display layout: the no-swap path, and recovery from a stale pending-restore file.

All bookkeeping paths behave, the exit code passes through (7), and the display layout is unchanged. Committing the launch stage.

Now the publish script and README. I'll also swap the help text's examples (currently this machine's real monitor serials) for a generic Dell example, since the repo may be shared.

The methods table is too wide for plaintext, and the last table isn't aligned. I'll tighten the methods section, then align every table with a small script so the padding is exact.

Tables are aligned. Now checking that the publish script produces a working single-file exe:

The published single-file exe (250 KB) works. Committing the docs and publish script:

Recording the conversation in `Conversations/` per AGENTS.md.

DisplayIdentification is in [DisplayIdentification](DisplayIdentification/README.md): a small .NET 10 command-line tool, committed in 4 commits plus the conversation record. Nothing is pushed.

**Using it**

Run `DisplayIdentification identify` to show each monitor's key on the monitor itself. Your three identical PA278QGVs are `AUSAA0F-T9LMSV003722` (left), `AUSAA0F-TCLMSV002479` (center, primary) and `AUSAA0F-TCLMSV002189` (right). Then:

```bash
DisplayIdentification launch --display AUSAA0F-T9LMSV003722 -- "C:\Games\Game\Game.exe"
```

The same line works as a shortcut target. In Steam launch options, end it with `-- %command%`.

Display names can't tell your monitors apart, and Windows' `DISPLAY5`/`6`/`7` numbers change over time. That's why keys come from each monitor's EDID model and serial number, which survive restarts and port changes.

**How it puts the program there**

- `--method move` (default) launches with a monitor hint, follows the process and its children, and moves windows to the display as they appear. Maximized windows stay maximized, and borderless fullscreen windows fill the new display.
- `--method primary` also makes the display primary until the program exits, then switches back. This is for exclusive fullscreen games.
- `--process-name Game.exe` covers launchers and `steam://` URLs that hand off to a process outside the tree.
- Started from a shortcut or Steam, there's no console window. Errors show in a message box, and each run is logged to `%LOCALAPPDATA%\DisplayIdentification\LastLaunch.log`.

**Tested here:** `list`, `identify`, and moving normal, maximized and borderless fullscreen test windows to your left and right monitors. Also: a launcher that hands off and exits, a process found only by name, the error cases, exit-code pass-through, and a shortcut-style start without a console.

**Not tested:** an actual primary display switch. Windows confirmed it would accept the new arrangement (`set-primary --check`), but I didn't apply it. A real switch moves your taskbar and can rearrange desktop icons.

To build: `DisplayIdentification\Publish.ps1` creates `DisplayIdentification\Output\DisplayIdentification.exe`.

Sources: [Console Allocation Policy (Microsoft Learn)](https://learn.microsoft.com/en-us/windows/console/console-allocation-policy)
