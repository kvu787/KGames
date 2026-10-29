# GameInputDemo

A Rust program that reads gamepads, keyboards and mice with Microsoft's
[GameInput](https://learn.microsoft.com/en-us/gaming/gdk/docs/features/common/input/overviews/input-overview)
API, and shows what it reads in a Win32 window drawn with Direct3D 12.

It's built like [HelloDirect3D12](../HelloDirect3D12/README.md): no frameworks, and
Microsoft's [`windows-core`](https://crates.io/crates/windows-core) crate is the only dependency.

## What it shows

- **Gamepad**: an XY graph for each stick, with its recent path, the unit circle, and
  XInput's recommended dead zone. A filled bar for each trigger. A chip for every
  button that lights up while it's held. The panel follows whichever gamepad was used last.
- **Latest input event**: a text box describing the newest reading that changed
  something: its device, what changed, how long after that device's previous reading
  it came, and the device's whole state.
- **Keyboard**: the keys held down, with each key's name, scan code, virtual-key code and character.
- **Mouse**: the buttons held, the motion and wheel, and a pad that moves with the
  raw relative motion, which has no pointer acceleration.
- **Devices**: every connected gamepad, keyboard and mouse, from GameInput's device callback.
- **Event log**: everything above as it happens, newest first.
  A run of analog motion from one device shares a line, with a count.

Shortcuts, which the demo also reads with GameInput:

| Key  | Action                                                                         |
| ---- | ------------------------------------------------------------------------------ |
| `F1` | Rumble test on or off. The triggers then drive the active gamepad's motors     |
| `F2` | Clear the event log                                                            |

## Requirements

- Rust with the `x86_64-pc-windows-msvc` toolchain
- Windows SDK (provides `dxc.exe` under `C:\Program Files (x86)\Windows Kits\10\bin\<version>\x64`)
- Internet access for the first build, which downloads the GameInput runtime (see below)
- Optional, for the D3D12 debug layer in debug builds: the "Graphics Tools" optional Windows feature

## Build and run

```powershell
cargo run --release
```

## How it gets GameInput

GameInput has two parts: a header that a game compiles against, and a runtime DLL.

**The API.** This demo uses API version 4, from `native/include/v4/GameInput.h` in the
[Microsoft.GameInput](https://www.nuget.org/packages/Microsoft.GameInput) NuGet package.
GameInput isn't in the Windows metadata that `windows-bindgen` reads, so
`Source/GameInput.rs` declares the parts the demo uses by hand, in the same shape as the
generated bindings. Every vtable slot is listed in header order, so it can be checked
against the header.

Microsoft's online reference still shows older signatures for some methods. For example,
its `RegisterDeviceCallback` has the `statusFilter` and `enumerationKind` parameters that
v4 removed. When they differ, the v4 header is right.

**The runtime.** Windows 11 ships a `GameInput.dll` that only supports API version 0.
Newer runtimes come from the NuGet package, either installed with `GameInputRedist.msi`,
or as a `GameInputRedist.dll` that sits next to the game's exe. The demo does the second:
`Build.rs` downloads the package version it pins and checks its SHA-512 against NuGet's.
It then copies `GameInputRedist.dll` and its helper `GameInputRawInputProxy.exe` next to the exe.
The package is kept in `Packages`, which Git ignores.

C++ games find the runtime by linking `GameInput.lib`, whose source is in the package
as `native/src/GameInput.cpp`. `game_input::create` is a Rust port of it: of the copy
built into Windows, an installed redistributable, and one in the exe's folder, it loads
the newest. The header of the demo's window shows which one it loaded.

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
3. Compares each reading with the same device's previous one, from `GetPreviousReading`,
   to work out what changed.

GameInput only gives a process keyboard and mouse input while one of its windows has focus.
The header of the demo's window says whether it has focus.

## Testing without a gamepad

Input sent with `SendInput` reaches GameInput as from an "Unidentified keyboard" or
"Unidentified mouse" with ID `0000:0000`. Injected input can be lost for about half a
second after the demo starts, while GameInput adds that device.

There's no such way to test the gamepad panel. The virtual gamepad that Windows'
`InputInjector.InitializeGamepadInjection` creates doesn't reach GameInput, so it needs a real gamepad.

## Updating GameInput

1. Set `GAME_INPUT_VERSION` in `Build.rs` to the new version, and `GAME_INPUT_SHA512` to its hash.
   The hash is the `packageHash` in the version's NuGet catalog entry, which is Base64, written as hex.
2. If the package has a new API version, compare its header with `Source/GameInput.rs`,
   change the interface IDs and vtables to match, and update this file.

## Using more APIs

For Windows APIs, which `windows-bindgen` generates:

1. Add the API's native name to `BindingsGenerator/Filters.txt`.
   For a method that an interface inherits, name the base interface, like `IDXGISwapChain::Present`.
2. Regenerate `Source/Bindings.rs`:

   ```powershell
   cargo run --manifest-path BindingsGenerator/Cargo.toml
   ```

3. Commit `Filters.txt` and `Bindings.rs` together.

For GameInput APIs, add them to `Source/GameInput.rs` by hand: replace the method's
`usize` placeholder in the vtable with its function pointer type, and add a method that calls it.

## Layout

| Path                               | Contents                                                                         |
| ---------------------------------- | -------------------------------------------------------------------------------- |
| `Source/Main.rs`                   | Window, message loop and DPI changes                                             |
| `Source/GameInput.rs`              | Hand-written GameInput v4 bindings, and the loader for the runtime DLL           |
| `Source/Input.rs`                  | Reads devices and readings with GameInput, and keeps what the panels show        |
| `Source/Panels.rs`                 | Lays out and draws the panels                                                    |
| `Source/Canvas.rs`                 | Builds each frame's rectangles, antialiased lines and circles, and text          |
| `Source/Font.rs`                   | Rasterizes Consolas into a texture atlas with GDI                                |
| `Source/Renderer.rs`               | The Direct3D 12 renderer: one draw call per frame                                |
| `Source/Bindings.rs`               | Generated Windows API bindings; don't edit by hand                               |
| `Shaders/Canvas.hlsl`              | Vertex and pixel shaders, plus the embedded root signature                       |
| `Build.rs`                         | Compiles the shaders, and fetches and deploys the GameInput runtime              |
| `BindingsGenerator/Filters.txt`    | The Windows APIs to generate bindings for                                        |
| `BindingsGenerator/Source/Main.rs` | Runs `windows-bindgen` to write `Source/Bindings.rs`                             |
| `Packages`                         | The downloaded GameInput NuGet package; not in Git                               |
