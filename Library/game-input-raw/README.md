# Description

- This crate contains generated Rust bindings for part of Microsoft's GameInput API.
- It covers stuff that reads keyboards, mice and gamepads.
- Official docs: https://learn.microsoft.com/en-us/gaming/gdk/docs/features/common/input/overviews/input-overview

# Development rules

- Scope:
  - This crate must not provide anything beyond `windows-bindgen` bindings.
  - Exception for GameInputInitialize: `bindings-generator` changes the one declaration that `windows-bindgen` writes for it, because `windows-bindgen` can't declare a function from a static library. See [Explanation of GameInputInitialize](#explanation-of-gameinputinitialize).
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
- Generation rules:
  - `src\lib.rs` must never be edited by hand.
  - To change `src\lib.rs`, change `bindings-generator` and regenerate `src\lib.rs`.

# Explanation of GameInputInitialize

- What it is:
  - `GameInputInitialize` is how a program starts GameInput: it gives the program the `IGameInput` object, which everything else comes from.
  - It's the only function in the GameInput API. The rest of the API is types, constants, and methods of COM interfaces.
- Why it's different from the rest of the API:
  - A COM method is called through a pointer in its object's vtable, which the object brings at runtime. So, the linker never has to find a method.
  - A function is called by its name, so the linker has to find its code when it builds the exe.
  - So, `GameInputInitialize` is the only thing in the bindings that the linker has to find.
- Where its code is:
  - Its code is in `GameInput.lib`, a static library in the NuGet package, which `build.rs` links. So, its code becomes part of the exe.
  - It isn't in a DLL, unlike Windows' own functions.
  - `GameInput.lib` is only a small loader. At runtime, it picks a GameInput runtime DLL, loads it, and calls the function of the same name that the DLL exports.
  - It picks the DLL with the highest file version out of three: Windows' own `GameInput.dll`, the installed `GameInputRedist.dll`, and a `GameInputRedist.dll` beside the exe. With GameInput 4.0.306 installed and nothing beside the exe, that's the installed one. See [Library/game-input's README](../game-input/README.md#how-it-works).
  - The NuGet package has the loader's source code, in `native\src\GameInput.cpp`.
- Why `windows-bindgen` can't declare it:
  - Every function in Windows' own APIs is in a DLL. So, `windows-bindgen` declares every function with `windows_core::link!`, which says to import the function from a DLL.
  - With that declaration, the build would look for `GameInputInitialize` in a DLL, which doesn't have it.
- What `bindings-generator` does about it:
  - It describes `GameInputInitialize` as metadata, and `windows-bindgen` generates its binding, the same as for the rest of the API.
  - Then, in what `windows-bindgen` wrote, it changes the `link!` declaration to a plain `extern "system"` declaration, which only says that the linker will find the function. The linker finds it in `GameInput.lib`.
  - Nothing else about the binding changes: its name, its parameters, and the function that wraps it stay as `windows-bindgen` wrote them.
  - The changed declaration has the form that `windows-bindgen` itself writes for a function's declaration when it's run with `--sys --extern`. That style can't be used here, because `--sys` leaves out the interfaces' methods.

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
  - KGames requires this to use GameInput at runtime: `Library/game-input` checks for it before it starts GameInput.
- What is the intended flow for an end user and/or game player to install the GameInput runtime?
  - Typically, a game bundles in and runs `GameInputRedist.msi` as part of its own installer.
  - Since GameInput 4.0, a game can instead put the NuGet package's `GameInputRedist.dll` beside its exe, without installing anything, which Microsoft calls sideloading. A sideloaded runtime can do less than an installed one, and KGames doesn't use it:
    https://learn.microsoft.com/en-us/gaming/gdk/docs/features/common/input/overviews/input-agility-redistributable
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
