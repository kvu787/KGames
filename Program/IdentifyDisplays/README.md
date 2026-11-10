# IdentifyDisplays

Shows each display's identifier on that display, for choosing the display that a KGames
program opens on. Put the identifier in the `"Display"` setting of the program's
`Settings.json`, or pass it to the program with `--display`.

```powershell
cargo run --release --bin IdentifyDisplays
cargo run --release --bin IdentifyDisplays -- --list
```

IdentifyDisplays puts a box in the middle of each display with the display's identifier, like
`KXGR16`, in Cascadia Mono, whose 0 has a slash through it, its model and its mode. It writes the
same to the console, with each monitor's device path. Click a box, or press `Esc` while one has
focus, to close them. With `--list`, it only writes to the console.

The boxes are windows without a frame that float over the others, drawn with
[Library/UserInterface](../../Library/UserInterface/README.md)'s overlays, in its light theme. A
display at another scale than 100% gets a box as big as it would look at 100%.

An identifier is 6 characters of [Crockford's Base32](https://www.crockford.com/base32.html),
which has no I, L, O or U, so no two characters look alike. They're the top 30 bits of an
FNV-1a hash of the monitor's device path, like
`\\?\DISPLAY#AUSAA0F#5&16b35f9f&0&UID8454#{e6f07b5f-ee97-4a90-b076-33f57bf4eaa7}`.
Windows makes that path from the monitor's model, from its EDID, and from the GPU output
it's plugged into. So an identifier:

- Stays the same across restarts, and when the display's resolution, scale or position changes.
- Differs between two monitors of the same model. On the development machine, three
  ASUS PA278QGV monitors are `CVVNY2`, `9XH36C` and `KXGR16`.
- Changes when the monitor is plugged into another output or GPU.

Programs match identifiers without regard to case. Monitors that clone each other show
the same part of the desktop, so IdentifyDisplays puts all of their identifiers in one
box, and any of them opens a program there. Without a monitor device path, like over
Remote Desktop, the identifier comes from the GDI device name, like `\\.\DISPLAY1`,
which can change between restarts.

[Library/Displays](../../Library/Displays/README.md) finds the displays and makes their identifiers,
for this program and for the others.

## Testing

```powershell
cargo test -p identify_displays_tests
```

The test in `Test/IdentifyDisplays` draws each display's box offscreen, without putting windows on
the displays, saves it in `target\TestOutput`, and checks that its text fits and draws.

## Layout

| Path          | Contents                                                        |
| ------------- | --------------------------------------------------------------- |
| `src/main.rs` | Finds the displays, writes them to the console, and shows a box |
|               | on each                                                         |
| `src/lib.rs`  | Lays out and draws a box, as a library that the test draws      |
