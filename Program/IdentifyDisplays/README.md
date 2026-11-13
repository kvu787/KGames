# IdentifyDisplays

Prints each display's identifier, for choosing the display that a KGames program opens on. Put the
identifier in the `"Display"` setting of the program's `Settings.json`, or pass it to the program
with `--display`. It's a console program, without a window, and takes no arguments.

```powershell
cargo build --release --bin identify_displays
target\release\identify_displays.exe
```

Or `cargo run --release --bin identify_displays`. Or run `HumanRun.cmd`, which you can also
double-click in File Explorer. It's for people, not for coding agents, which run the exe instead. It
runs `HumanRun.ps1`, which builds IdentifyDisplays, runs it, and waits for Enter before it closes,
so that the console window stays open to read.

It prints something like:

```text
3 displays, from left to right:

  CVVNY2  PA278QGV
          2560 x 1440, 120 Hz, 100% scale, HDMI
          At -2560, 0 on the desktop
          \\?\DISPLAY#AUSAA0F#5&16b35f9f&0&UID8448#{e6f07b5f-ee97-4a90-b076-33f57bf4eaa7}

  9XH36C  PA278QGV, the primary display
          2560 x 1440, 120 Hz, 100% scale, DisplayPort
          At 0, 0 on the desktop
          \\?\DISPLAY#AUSAA0F#5&16b35f9f&0&UID8454#{e6f07b5f-ee97-4a90-b076-33f57bf4eaa7}

  KXGR16  PA278QGV
          2560 x 1440, 120 Hz, 100% scale, DisplayPort
          At 2560, 0 on the desktop
          \\?\DISPLAY#AUSAA0F#5&16b35f9f&0&UID8450#{e6f07b5f-ee97-4a90-b076-33f57bf4eaa7}

To open a program on a display, set "Display" in the Settings.json in its
folder to the display's identifier, or run it with --display and the identifier.
```

Each display has its identifier, its model, its mode, where it is on the desktop, and its monitor's
device path, which the identifier is made from.

An identifier is 6 characters of [Crockford's Base32](https://www.crockford.com/base32.html),
which has no I, L, O or U, so no two characters look alike. They're the top 30 bits of an
FNV-1a hash of the monitor's device path, like `KXGR16`'s,
`\\?\DISPLAY#AUSAA0F#5&16b35f9f&0&UID8450#{e6f07b5f-ee97-4a90-b076-33f57bf4eaa7}`.
Windows makes that path from the monitor's model, from its EDID, and from the GPU output
it's plugged into. So an identifier:

- Stays the same across restarts, and when the display's resolution, scale or position changes.
- Differs between two monitors of the same model. On the development machine, three
  ASUS PA278QGV monitors are `CVVNY2`, `9XH36C` and `KXGR16`.
- Changes when the monitor is plugged into another output or GPU.

Programs match identifiers without regard to case. Monitors that clone each other show the same
part of the desktop, at the same place, and any of their identifiers opens a program there.
Without a monitor device path, like over Remote Desktop, the identifier comes from the GDI device
name, like `\\.\DISPLAY1`, which can change between restarts.

[Library/Displays](../../Library/Displays/README.md) finds the displays and makes their identifiers,
for this program and for the others.

## Testing

```powershell
cargo test -p identify_displays_tests
```

The tests in `Test/IdentifyDisplays` check what it prints, for made-up displays and for the real
ones, and that Library/Displays finds the displays as they are whatever DPI awareness the thread
that asks has.

## Layout

| Path           | Contents                                                       |
| -------------- | -------------------------------------------------------------- |
| `src/main.rs`  | Finds the displays and prints them                             |
| `src/lib.rs`   | What it prints, and its command line, as a library to test     |
| `HumanRun.ps1` | Builds IdentifyDisplays, runs it, and waits for Enter          |
| `HumanRun.cmd` | Runs `HumanRun.ps1`, for double-clicking in File Explorer      |
