# Rust Direct3D 12 and Win32 without frameworks

## User

explain how to create a game or gui app with rust, dx12, winapi, and minimal or no other dependencies or frameworks

## AI

Now the main program.

Sample builds and renders correctly. I'm quickly checking the manifest-embedding tip before I write up the explanation.

Before writing this up I built and ran a sample: a resizable window that draws a DX12 triangle with vsync. It's 370 lines of Rust with **one dependency**, and the release exe is 152 KB. The sample is at `C:\Users\K\AppData\Local\Temp\claude\C--Users-K-Repository-kGames\bcb41681-1034-473c-a4b4-344d3cb95df7\scratchpad\HelloDirect3D12`, which is a temporary folder, not the repo.

## The short version

The only crate you need is Microsoft's **`windows`** crate (windows-rs). Everything else comes from Windows through that crate: the window, the message loop, D3D12, DXGI, input, audio, image decoding and text. Shaders are compiled at build time by `dxc.exe` from the Windows SDK, so they don't need a crate either.

## 1. Choosing bindings

| Option        | What you get                                                                 | Verdict                                                                                                  |
| ------------- | ---------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------- |
| `windows`     | Win32 functions plus COM interfaces (D3D12, DXGI) with automatic `Release`   | Use this. Microsoft maintains it, it's generated from official metadata, and it needs no import libs     |
| `windows-sys` | Raw C functions and structs only                                             | Has no COM interfaces, so no D3D12                                                                       |
| Nothing       | You hand-write `extern "system"` declarations and COM vtable structs         | Truly zero deps, but you have to copy out dozens of vtable slots per interface in exact order            |

If you want zero dependencies later, the program structure stays the same. Only the bindings change.

Each part of the `windows` crate is turned on by a feature, one per Windows namespace. The sample needs:

```toml
[dependencies.windows]
version = "0.62"
features = [
    "Win32_Foundation",
    "Win32_Graphics_Direct3D",
    "Win32_Graphics_Direct3D12",
    "Win32_Graphics_Dxgi",
    "Win32_Graphics_Dxgi_Common",
    "Win32_Graphics_Gdi",            # WNDCLASSEXW has an HBRUSH field
    "Win32_Security",                # CreateEventW takes SECURITY_ATTRIBUTES
    "Win32_System_LibraryLoader",    # GetModuleHandleW
    "Win32_System_Threading",        # CreateEventW, WaitForSingleObject
    "Win32_UI_HiDpi",
    "Win32_UI_WindowsAndMessaging",
]
```

If a name won't resolve, look it up at microsoft.github.io/windows-docs-rs. Each item there lists the feature it needs.

## 2. Program structure

```
CreateWindowExW -> loop { drain messages (PeekMessageW) -> update -> record commands -> submit -> Present }
```

### Window

- `#![windows_subsystem = "windows"]` removes the console window.
- Call `SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2)`, or use a manifest (section 5). Without it, Windows blurs your window on high-DPI displays.
- Call `RegisterClassExW` with a window procedure, then `CreateWindowExW`.
- The window procedure is `extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT`. Handle `WM_SIZE`, `WM_DESTROY` (call `PostQuitMessage`) and input messages. Pass everything else to `DefWindowProcW`.

**Rust tip:** the window procedure is a callback with no `&mut self`. The usual C++ approach stores a pointer in `GWLP_USERDATA`, but in Rust that can easily create two `&mut` references to the same object, which is undefined behavior. It's simpler to have the window procedure only *record* events (in atomics or a queue) and let the main loop act on them. The sample does this.

### Message loop

- **Game:** call `PeekMessageW` until the queue is empty, then render a frame. Presenting with vsync caps the frame rate.
- **GUI app or tool:** call `GetMessageW`, which blocks, and render only on `WM_PAINT` or when something changes. This saves power.

**Gotcha:** while the user drags the window border, Windows runs its own modal loop inside `DispatchMessageW`. Your loop stops and the picture freezes until the user lets go. To fix this, also render from `WM_SIZE`/`WM_PAINT`, or run the game on its own thread and keep the window on the main thread.

### D3D12 setup (once)

1. In debug builds, call `D3D12GetDebugInterface`, then `EnableDebugLayer`. This needs the "Graphics Tools" optional Windows feature. Messages go to the debugger output (Visual Studio, WinDbg or DebugView).
2. Call `CreateDXGIFactory2`, then `EnumAdapterByGpuPreference`, then `D3D12CreateDevice`.
3. Create a direct command queue.
4. Create the swap chain with `CreateSwapChainForHwnd`, using `DXGI_SWAP_EFFECT_FLIP_DISCARD` and 2-3 buffers.
5. Create an RTV descriptor heap with one render target view per back buffer.
6. Create one command allocator per frame in flight, plus one command list.
7. Create the root signature and the pipeline state object (shaders, blend, rasterizer, render target format).
8. Create a fence and a Win32 event for CPU/GPU sync.

### Every frame (the sample's actual loop, trimmed)

```rust
let index = self.swap_chain.GetCurrentBackBufferIndex() as usize;
self.wait_for_fence(self.frame_fence_values[index])?; // is the GPU done with this allocator?
allocator.Reset()?;
list.Reset(allocator, &self.pipeline)?;
list.SetGraphicsRootSignature(&self.root_signature);
// viewport and scissor rectangle
list.ResourceBarrier(&[transition(back_buffer, D3D12_RESOURCE_STATE_PRESENT, D3D12_RESOURCE_STATE_RENDER_TARGET)]);
list.OMSetRenderTargets(1, Some(&rtv), false, None);
list.ClearRenderTargetView(rtv, &[0.1, 0.1, 0.15, 1.0], None);
list.IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
list.DrawInstanced(3, 1, 0, 0);
list.ResourceBarrier(&[transition(back_buffer, D3D12_RESOURCE_STATE_RENDER_TARGET, D3D12_RESOURCE_STATE_PRESENT)]);
list.Close()?;
self.queue.ExecuteCommandLists(&[Some(list.cast()?)]);
self.swap_chain.Present(1, DXGI_PRESENT(0)).ok()?;
self.last_fence_value += 1;
self.queue.Signal(&self.fence, self.last_fence_value)?;
self.frame_fence_values[index] = self.last_fence_value;
```

### Resize

On `WM_SIZE`, wait until the GPU is idle, drop every reference to the back buffers, call `ResizeBuffers`, then recreate the RTVs. Skip all of this when the size is 0 (minimized).

### windows-rs specifics for D3D12

- COM objects are reference-counted Rust values, and dropping one releases it. `.cast::<IOther>()` is `QueryInterface`.
- A few D3D12 structs (barriers, the PSO description) hold interfaces as `ManuallyDrop<Option<IFoo>>`. Fill them with `std::mem::transmute_copy(&foo)`, which borrows without changing the reference count. The official windows-rs D3D12 sample does the same.
- Almost every call is `unsafe`. Keep the `unsafe` code in a small renderer module behind a safe API.

## 3. Shaders without dependencies

The Windows SDK includes `dxc.exe`, plus `dxil.dll` to sign its output. The sample's `Build.rs` finds `dxc.exe` in the newest installed SDK and runs:

```
dxc -T vs_6_0 -E VSMain -Fo $OUT_DIR/Vertex.cso Shaders/Triangle.hlsl
```

The Rust code then embeds the result with `include_bytes!(concat!(env!("OUT_DIR"), "/Vertex.cso"))`.

One trick: declare the root signature in HLSL (`[RootSignature("RootFlags(0)")]`) and create it with `device.CreateRootSignature(0, VERTEX_SHADER)`. That removes all the `D3D12SerializeRootSignature` code.

There are two alternatives. You can compile at runtime with `D3DCompile` from `d3dcompiler_47.dll`, which comes with Windows but only supports shader model 5.1. Or you can ship `dxcompiler.dll` yourself.

## 4. The rest of a game or app, still from Windows

| Need                     | Windows API (all in the `windows` crate)                                                  |
| ------------------------ | ----------------------------------------------------------------------------------------- |
| Timing                   | `QueryPerformanceCounter`, `QueryPerformanceFrequency`                                    |
| Keyboard and mouse       | `WM_KEYDOWN`, `WM_CHAR`, `WM_MOUSEMOVE`; Raw Input (`WM_INPUT`) for mouse look            |
| Gamepad                  | XInput (`XInputGetState`) or GameInput                                                    |
| Audio                    | XAudio2 (made for games) or WASAPI (lower level)                                          |
| Image files              | WIC (`IWICImagingFactory`) decodes PNG/JPEG/BMP into pixels for textures                  |
| Text                     | DirectWrite to rasterize glyphs into a texture atlas, or a bitmap font baked offline      |
| Files, threads           | Rust's `std` is fine                                                                      |
| Math                     | Write your own `Vector2/3/4` and `Matrix4`, a few hundred lines                           |
| Native widgets (GUI)     | Common Controls (`BUTTON`, `EDIT`, `SysListView32` windows); needs a manifest for v6 look |

For a GUI app there are two paths:

1. **Native Win32 controls.** You get real widgets, accessibility and IME (input for languages like Chinese and Japanese) for free. You only need DX12 for a viewport child window, if at all.
2. **Draw your own UI with DX12**, usually in immediate-mode style. You get full control and a consistent look, but you write layout, text, focus and IME handling yourself. Text is the biggest part.

## 5. Embedding a manifest without a crate

I tested this. MSVC's linker embeds the manifest directly:

```rust
// Build.rs
println!("cargo:rustc-link-arg-bins=/MANIFEST:EMBED");
println!("cargo:rustc-link-arg-bins=/MANIFESTINPUT:Application.manifest");
```

Use it for DPI awareness, the Common Controls v6 look, `longPathAware` and the UTF-8 code page.

## 6. Suggested order to build from the sample

1. Clear the screen, then draw a triangle (the sample does both)
2. Timing and input, so something moves
3. Root constants, then constant buffers, then vertex/index buffers via an upload heap
4. Textures (WIC, upload, SRV descriptor heap)
5. Depth buffer
6. Text
7. Audio

For reference code, Microsoft's C++ DirectX-Graphics-Samples (`D3D12HelloWorld` and others) translate to windows-rs almost line for line. The windows-rs repo also has its own D3D12 sample.

I recorded this conversation in [Conversations/2026-10-29_06-47_UTC Rust Direct3D 12 and Win32 without frameworks.md](Conversations/2026-10-29_06-47_UTC%20Rust%20Direct3D%2012%20and%20Win32%20without%20frameworks.md) and committed it as `[cnv]` (not pushed).

## User

Add it to repo

## AI

Built from the repo location; checking the rendered frame, then committing.

The sample is now in the repo at [HelloDirect3D12/](HelloDirect3D12/), in three commits on `main` (not pushed):

- `19053c8` Use 4-space indents for Rust, TOML and HLSL files. This adds `rs`, `toml` and `hlsl` to the existing 4-space section of `.editorconfig`.
- `5446a71` Add HelloDirect3D12 project setup, shader build and shaders: `Cargo.toml`, `Cargo.lock`, `Build.rs`, `Shaders/Triangle.hlsl`, `.gitignore`, `README.md`.
- `dbb9d05` Add HelloDirect3D12 window and Direct3D 12 renderer: `Source/Main.rs`.

Compared with the scratchpad version, I ran `cargo fmt` (it only rewrapped long lines), added a README with requirements and build steps, and added a `.gitignore` for `target/`. I rebuilt debug and release from the repo folder and ran the release exe. The triangle renders, and the program exits cleanly with code 0.

To run it from the repo root:

```bash
cargo run --release --manifest-path HelloDirect3D12/Cargo.toml
```

## User

C:\Users\K\Repository\kGames\HelloDirect3D12

explain how cargo, crate, dependencies, etc work in rust.
i am concerned about the umbrella windows crate at https://crates.io/crates/windows because it hasn't been updated for a year. i thought you were supposed to only add the sub-crates for the windows functionality you need, or something like that?

## AI

Still researching: the windows-rs docs moved, so I'm reading the current readme and the latest monthly updates to see where the umbrella crate stands now.

Your instinct is mostly right, and my earlier "use `windows`" advice needs an update.

- The `windows` umbrella crate's last release is 0.62.2, from 2025-10-06. On 2026-09-03 the windows-rs project released 0.100.0 of nearly all its other crates (`windows-core`, `windows-bindgen`, `windows-link` and others) but held back `windows` and `windows-sys`. Microsoft's August 2026 project update says they are still checking the quality of a new metadata pipeline, which now parses the Windows SDK headers directly, before publishing the umbrella crates. So `windows` is on hold, not abandoned, and its next version will have big breaking changes (new feature names, many changed signatures).
- There are no per-API sub-crates like a `windows-d3d12`. The sub-crates you've seen are support crates (`windows-core`, `windows-result`, ...) and focused safe wrappers (`windows-registry`, `windows-window`, ...). Inside `windows`, *features* are what limit compilation to the parts you need.
- Microsoft's current guidance: use a focused crate where one fits, and generate the remaining APIs with **`windows-bindgen`**. Reusable libraries should avoid the umbrella crates. Binary applications may still use them for convenience.

For this project, an app that aims for minimal dependencies, I'd switch to `windows-bindgen`. The details are at the end, after the Cargo basics.

## How Cargo works

### Packages and crates

- A **crate** is one unit of compilation: a library (`lib.rs`) or an executable (`main.rs`).
- A **package** is a folder with a `Cargo.toml`. It holds at most one library crate and any number of binary crates. `HelloDirect3D12` is a package with one binary crate (`Source/Main.rs`) and a build script (`Build.rs`).
- **Cargo** is Rust's build tool and package manager. It reads `Cargo.toml`, downloads dependencies, runs build scripts and calls `rustc` once per crate.

### Cargo.toml

| Section                | Purpose                                                                    |
| ---------------------- | -------------------------------------------------------------------------- |
| `[package]`            | Name, version, edition (`2024`), build script path                         |
| `[[bin]]`, `[lib]`     | Crate targets; needed here only because we use `Source/` instead of `src/` |
| `[dependencies]`       | Crates your code uses                                                      |
| `[build-dependencies]` | Crates only `Build.rs` uses (they never end up in your exe)                |
| `[dev-dependencies]`   | Crates only tests, examples and benchmarks use                             |
| `[features]`           | Your own compile-time switches                                             |
| `[profile.release]`    | Optimization, debug info, LTO, panic strategy                              |

### crates.io and downloads

crates.io is the public registry. Crates are published as **source code**, and a published version can't be modified. An author can "yank" a version, which stops new lockfiles from choosing it but doesn't break existing ones. Cargo downloads sources to `%USERPROFILE%\.cargo\registry` and compiles them together with your code. Rust dependencies are statically linked, so there are no extra DLLs to ship.

### Version requirements

`version = "0.62"` means `^0.62`. Cargo assumes semantic versioning:

- `"1.2"` allows `>=1.2.0, <2.0.0`.
- `"0.62"` allows `>=0.62.0, <0.63.0`. Before 1.0, the minor number counts as the breaking-change number.

So going from 0.62 to 0.100 takes a deliberate edit to `Cargo.toml`. It never happens automatically.

### Cargo.lock

`Cargo.lock` records the exact version and checksum of every crate in the graph. With it committed (HelloDirect3D12's is), every build uses the same code until you run `cargo update` or edit `Cargo.toml`. That's why a crate that stops getting releases can't silently break your build. It can only fall behind.

### Resolution and duplicates

Cargo picks one version per semver-compatible range, but **incompatible versions can coexist**. If one library wants `windows` 0.62 and another wants `windows` 0.100, both get compiled into the same program. With a crate as large as `windows`, that's the "version churn" Microsoft warns libraries about.

### Features

Features are compile-time switches that a crate defines. They are additive and **unified**: if any crate in the graph turns on a feature of `windows`, everyone gets it. The `windows` crate compiles almost nothing until you turn features on, so the 11 features in our `Cargo.toml` are what limit it to "only the functionality you need".

### Build scripts and proc macros

- A **build script** (`build.rs`, `Build.rs` here) is compiled and run before your crate. Ours runs `dxc.exe`.
- A **proc macro** crate runs inside the compiler at build time and generates code. `syn`, `quote` and `proc-macro2` are the standard helpers for writing them. They make builds slower but never end up in your exe.

### Everyday commands

| Command           | What it does                                                   |
| ----------------- | -------------------------------------------------------------- |
| `cargo build`     | Compile into `target/debug` (`--release` for `target/release`) |
| `cargo run`       | Build, then run the binary                                     |
| `cargo check`     | Type-check without generating code (fastest feedback)          |
| `cargo tree`      | Show the dependency graph                                      |
| `cargo update`    | Move `Cargo.lock` to the newest versions `Cargo.toml` allows   |
| `cargo add NAME`  | Add a dependency to `Cargo.toml`                               |
| `cargo info NAME` | Show a crate's versions, features and links                    |
| `cargo fmt`       | Format code with rustfmt                                       |
| `cargo clippy`    | Run the linter                                                 |

## What HelloDirect3D12 actually pulls in

Earlier I said "one dependency". To be precise, it's one *direct* dependency. `cargo tree` shows 15 crates in total:

```
windows 0.62.2
├── windows-core 0.62.2      COM support: IUnknown, Interface, HRESULT, Result
│   ├── windows-implement    proc macro (build time only)
│   ├── windows-interface    proc macro (build time only)
│   │   └── syn, quote, proc-macro2, unicode-ident
│   ├── windows-link         links to system DLLs without import libraries
│   ├── windows-result       error types
│   └── windows-strings      PCWSTR, w!, HSTRING
├── windows-collections      WinRT collections (we don't use it)
├── windows-future           WinRT async (we don't use it)
│   └── windows-threading
└── windows-numerics         vector and matrix types (we don't use it)
```

Eleven of them come from Microsoft's windows-rs project. The other four are the standard proc-macro helpers, used only at build time. Three of the crates are WinRT support that the umbrella crate always pulls in, even though we don't use them.

## Options for this project

| Option                                | Dependencies                      | Trade-off                                                                                       |
| ------------------------------------- | --------------------------------- | ----------------------------------------------------------------------------------------------- |
| Keep `windows` 0.62                   | 15 crates, as above               | Works today and is pinned by `Cargo.lock`. Older metadata, and a big migration when 0.100 ships |
| `windows-bindgen` 0.100 (recommended) | `windows-core` and its small deps | Bindings for only the functions, interfaces and structs we use, from the current metadata       |
| Hand-written FFI                      | None                              | Zero dependencies, but you write and maintain every COM vtable yourself                         |

How the bindgen route works: you list the APIs you use (`D3D12CreateDevice`, `ID3D12Device`, `CreateWindowExW`, ...) one per line in a filter file. `windows-bindgen` generates a Rust source file with just those, plus whatever types their signatures need. You can run it from `Build.rs` as a build-dependency. For minimal dependencies it's better to run it once and commit the generated file, so normal builds don't compile the generator at all. The generated code depends on `windows-core`, and with its `proc-macros` feature off, `windows-core` drops the proc-macro crates too.

Two caveats. The 0.100 bindings follow the C headers more literally: some calls return a raw `HRESULT` or take a pointer and a count instead of a slice, so the code gets slightly more C-like. And I haven't generated D3D12 bindings with 0.100 yet, so I'd confirm they compile as part of the switch.

On your concern itself: a year without a release isn't dangerous for this kind of crate. D3D12 and Win32 have stable ABIs, and bindings from 2025 still call the same DLL functions correctly. The real costs are missing newer APIs and the breaking change that's coming. The bindgen route avoids both.
