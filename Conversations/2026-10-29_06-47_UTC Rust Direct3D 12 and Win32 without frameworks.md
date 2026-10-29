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
