# HelloDirect3D12

A minimal Rust program that opens a Win32 window and draws a triangle with Direct3D 12.

Its only dependency is Microsoft's [`windows`](https://crates.io/crates/windows) crate.
Shaders are compiled to DXIL at build time by `dxc.exe` from the Windows SDK.

## Requirements

- Rust with the `x86_64-pc-windows-msvc` toolchain
- Windows SDK (provides `dxc.exe` under `C:\Program Files (x86)\Windows Kits\10\bin\<version>\x64`)
- Optional, for the D3D12 debug layer in debug builds: the "Graphics Tools" optional Windows feature

## Build and run

```powershell
cargo run --release
```

## Layout

| Path                    | Contents                                                     |
| ----------------------- | ------------------------------------------------------------ |
| `Source/Main.rs`        | Window, message loop, and the Direct3D 12 renderer           |
| `Shaders/Triangle.hlsl` | Vertex and pixel shaders, plus the embedded root signature   |
| `Build.rs`              | Finds the newest Windows SDK `dxc.exe` and compiles shaders  |
