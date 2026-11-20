# Description

- This crate generates and contains Rust bindings for part of Microsoft's GameInput API.
- It covers stuff that reads keyboards, mice and gamepads.
- Official docs: https://learn.microsoft.com/en-us/gaming/gdk/docs/features/common/input/overviews/input-overview

# Development rules

- Scope:
  - This crate must not provide anything beyond `windows-bindgen` bindings.
  - This crate must only provide bindings for what KGames is likely to use of the GameInput API for x64 PC.
  - In scope: reading keyboards, mice and gamepads, a gamepad's system buttons and rumble, finding and identifying devices, and the callbacks for readings and devices.
  - Out of scope: arcade sticks, flight sticks, racing wheels, sensors, touch surfaces, controllers that GameInput doesn't map to a gamepad, the input mapper, force feedback, haptics, lighting, raw device reports, aggregate devices and the dispatcher.
- Structure:
  - `INTERFACES`, `TYPES`, `CONSTANTS` and `FUNCTIONS` in `bindings-generator\src\lib.rs` name everything that's in scope.
  - An enum that's in scope has all of its values, since GameInput gives any of them.
  - A method that's out of scope keeps its slot in its interface's vtable, as a `usize`, so the slots after it stay where GameInput has them.
- How bindings are generated:
  - Currently, GameInput doesn't come with the Windows metadata that `windows-bindgen` needs.
  - So, the custom written `bindings-generator` crate generates this metadata from the `GameInput.h` API header.
  - Then, `windows-bindgen` generates `src\lib.rs` from that, with filters for the scope.

# Build prerequisites

- Install Visual Studio 2026 with the ".NET desktop development" workload, which should provide the `dotnet` command.
- `build.rs` uses `dotnet` to download the GameInput NuGet package.
- The NuGet package provides `GameInput.h` and `GameInput.lib`.
- `GameInput.h` is the header file that this crate wraps in `src\lib.rs`.
  - `src\lib.rs` is generated from `GameInput.h` and committed, so `GameInput.h` isn't strictly needed to build the crate.
  - To change `src\lib.rs`, change `bindings-generator`, and run it with `cargo run -p game-input-bindings-generator`.
  - To bind more of the API, add its names to the scope in `bindings-generator\src\lib.rs`, and regenerate `src\lib.rs`.
  - `Test/game-input` checks that `src\lib.rs` is what `bindings-generator` generates, and that it binds what `GameInput.h` declares of the scope, and nothing else.
- `GameInput.lib` is a static library that provides the implementation for `GameInputInitialize`, which is also wrapped in `src\lib.rs`.
  - Unlike `GameInput.h`, `GameInput.lib` is strictly required to build the crate.

# Runtime prerequisites

This crate doesn't include any runnable stuff, but if you use this crate in any runnable things
like programs or tests, then you must install these runtime prerequisites.

- Install the GameInput runtime with `winget install --exact --id "Microsoft.GameInput" --source "winget" --version "4.0.306"`.
  - Installing this winget package runs the `GameInputRedist.msi` installer contained in the GameInput NuGet package.
  - That installs the GameInput redistributables to the right location and installs the GameInput background service.
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
| `bindings-generator/src/lib.rs`    | The scope; describes them as metadata for `windows-bindgen`  |
| `bindings-generator/src/main.rs`   | Writes the bindings to `src/lib.rs`                          |
