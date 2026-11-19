# Description

- This contains the Rust bindings for Microsoft's GameInput API.
- Official docs: https://learn.microsoft.com/en-us/gaming/gdk/docs/features/common/input/overviews/input-overview

# Prerequisites

- Install Visual Studio 2026 with the ".NET desktop development" workload, which should provide the `dotnet` command.
  - `build.rs` uses this to ensure that the GameInput NuGet package is installed.
- Install the GameInput runtime with `winget install --exact --id "Microsoft.GameInput" --source "winget" --version "4.0.306"`
- The NuGet package and the winget package are different and both are needed.
  - The NuGet package is needed to get the `GameInput.h` and `GameInput.lib` files used in this crate's build process.
  - The winget package is needed to use GameInput at runtime.
