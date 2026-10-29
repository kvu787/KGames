# GameInputDemo

A Rust program that reads gamepads, keyboards and mice with Microsoft's
[GameInput](https://learn.microsoft.com/en-us/gaming/gdk/docs/features/common/input/overviews/input-overview)
API, and shows what it reads in a Win32 window drawn with Direct3D 12.

It's built like [HelloDirect3D12](../HelloDirect3D12/README.md): no frameworks, and
Microsoft's [`windows-core`](https://crates.io/crates/windows-core) crate is the only dependency.

## What it shows

- **Title bar**: the GPU that draws, and whether frames may tear (see [Input latency](#input-latency)).
- **Header**: the GameInput runtime the demo loaded, how many readings it has handled,
  whether the window has focus, and the frame rate with the time per frame.
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

| Key   | Action                                                                        |
| ----- | ----------------------------------------------------------------------------- |
| `F1`  | Rumble test on or off. The triggers then drive the active gamepad's motors    |
| `F2`  | Clear the event log                                                           |
| `F11` | Borderless fullscreen on or off, for the lowest latency                       |

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
   compositor shows the newest frame at its next refresh.
5. **Borderless fullscreen.** `F11` makes the window cover its monitor, which lets Windows
   flip the frames straight to the screen, instead of the compositor copying them into its
   own frame. That saves up to a refresh, and lets frames tear. In a window, Windows can
   only do that if the GPU supports multiplane overlays.
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
   onto those, at lower clock speeds. GameInput still delivers gamepad input then, so the
   demo opts out with `SetProcessInformation`.

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
| `Source/Main.rs`                   | Window, message loop, frame pacing, fullscreen and DPI changes                   |
| `Source/FrameCounter.rs`           | Counts frames per second, for the header                                         |
| `Source/GameInput.rs`              | Hand-written GameInput v4 bindings, and the loader for the runtime DLL           |
| `Source/Input.rs`                  | Reads devices and readings with GameInput, and keeps what the panels show        |
| `Source/Panels.rs`                 | Lays out and draws the panels                                                    |
| `Source/Canvas.rs`                 | Builds each frame's rectangles, antialiased lines and circles, and text          |
| `Source/Font.rs`                   | Rasterizes Consolas into a texture atlas with GDI                                |
| `Source/Renderer.rs`               | The Direct3D 12 renderer: one draw call per frame, and one frame in flight       |
| `Source/Bindings.rs`               | Generated Windows API bindings; don't edit by hand                               |
| `Shaders/Canvas.hlsl`              | Vertex and pixel shaders, plus the embedded root signature                       |
| `Build.rs`                         | Compiles the shaders, and fetches and deploys the GameInput runtime              |
| `BindingsGenerator/Filters.txt`    | The Windows APIs to generate bindings for                                        |
| `BindingsGenerator/Source/Main.rs` | Runs `windows-bindgen` to write `Source/Bindings.rs`                             |
| `Packages`                         | The downloaded GameInput NuGet package; not in Git                               |
