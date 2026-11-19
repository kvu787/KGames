# Description

- This contains the Rust bindings for Microsoft's GameInput API.
- Official docs: https://learn.microsoft.com/en-us/gaming/gdk/docs/features/common/input/overviews/input-overview

# Development rules

- This crate must provide bindings for the complete GameInput API for x64 PC.
- `windows-bindgen` doesn't work for automatically generating bindings for GameInput's API.
- However, the bindings must be written like how `windows-bindgen` would write them.
- This crate must not provide anything beyond `windows-bindgen`-style bindings.

# Build prerequisites

- Install Visual Studio 2026 with the ".NET desktop development" workload, which should provide the `dotnet` command.
- `build.rs` uses `dotnet` to download the GameInput NuGet package.
- The NuGet package provides `GameInput.h` and `GameInput.lib`.
- `GameInput.h` the header file that this crate wraps in `src\lib.rs`.
  - `GameInput.h` is manually translated to `src\lib.rs` as a one-time process, so `GameInput.h` isn't strictly needed to build the crate.
  - However, if you ever want to change `src\lib.rs`, then you should have `GameInput.h` to ensure you're doing the FFI wrapping correctly.
- `GameInput.lib` is a static library that provides the implementation for `GameInputInitialize`, which is also wrapped in `src\lib.rs`.
  - Unlike `GameInput.h`, `GameInput.lib` is strictly required to build the crate.

# Runtime prerequisites

This crate doesn't include any runnable stuff, but if you use this crate in any runnable things
like programs or tests, then you must install these runtime prerequisites.

- Install the GameInput runtime with `winget install --exact --id "Microsoft.GameInput" --source "winget" --version "4.0.306"`.
  - Installing this winget package runs the `GameInputRedist.msi` installer contained in the GameInput NuGet package.
  - That installs the GameInput resdistributables to the right location and installs the GameInput background service.
  - This is required to use GameInput at runtime.
- What is the intended flow for an end user and/or game player to install the GameInput runtime?
  - Typically, a game bundles in and runs `GameInputRedist.msi` as part of its own installer.
  - If the game doesn't have its own installer, then it can instruct the user to download and run
    `GameInputRedist.msi`, which can be downloaded from GameInput's GitHub repo releases page:
    https://github.com/microsoftconnect/GameInput/releases
  - Microsoft documents the winget option as another official installation method here:
    https://learn.microsoft.com/en-us/gaming/gdk/docs/features/common/input/overviews/input-nuget?view=gdk-2610
  - Another installation method is to download and extract the NuGet package from
    https://www.nuget.org/packages/Microsoft.GameInput/4.0.306, and then run the `GameInputRedist.msi` it contains.
