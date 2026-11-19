# Description

- This contains the Rust bindings for Microsoft's GameInput API.
- Official docs: https://learn.microsoft.com/en-us/gaming/gdk/docs/features/common/input/overviews/input-overview

# Development rules

- This crate must provide bindings for the complete GameInput API for x64 PC.
- `windows-bindgen` can't generate bindings for GameInput's API from Windows' metadata, which doesn't have it.
  - So `bindings-generator` describes `GameInput.h` as metadata, and `windows-bindgen` generates `src\lib.rs` from that.
- The bindings must be written like how `windows-bindgen` would write them, so `src\lib.rs` is never edited by hand.
- This crate must not provide anything beyond `windows-bindgen`-style bindings.

# Build prerequisites

- Install Visual Studio 2026 with the ".NET desktop development" workload, which should provide the `dotnet` command.
- `build.rs` uses `dotnet` to download the GameInput NuGet package.
- The NuGet package provides `GameInput.h` and `GameInput.lib`.
- `GameInput.h` is the header file that this crate wraps in `src\lib.rs`.
  - `src\lib.rs` is generated from `GameInput.h` and committed, so `GameInput.h` isn't strictly needed to build the crate.
  - To change `src\lib.rs`, change `bindings-generator`, and run it with `cargo run -p game-input-bindings-generator`.
  - `Test/game-input` checks that `src\lib.rs` is what `bindings-generator` generates, and that it binds everything that `GameInput.h` declares.
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

# Layout

| Path                               | Contents                                                     |
| ---------------------------------- | ------------------------------------------------------------ |
| `src/lib.rs`                       | The generated bindings; don't edit by hand                   |
| `build.rs`                         | Restores the NuGet package, and links its `GameInput.lib`    |
| `NuGetPackages.csproj`             | The NuGet packages that `build.rs` restores                  |
| `packages.lock.json`               | Pins the packages' content by its hash                       |
| `Directory.Build.props`            | Restores them into the repository's `target` folder          |
| `bindings-generator/src/header.rs` | Reads `GameInput.h`'s declarations                           |
| `bindings-generator/src/lib.rs`    | Describes them as metadata, and runs `windows-bindgen` on it |
| `bindings-generator/src/main.rs`   | Writes the bindings to `src/lib.rs`                          |
