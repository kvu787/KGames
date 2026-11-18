# Description

Rust bindings for Microsoft GameInput version 4.0.306, and nothing else. What KGames adds to them,
like starting GameInput and checking that its runtime is installed, is in
[Library/GameInput](../GameInput/README.md).

Microsoft docs: https://learn.microsoft.com/en-us/gaming/gdk/docs/features/common/input/overviews/input-overview

# Prerequisites

The .NET SDK, 10 or later, whose `dotnet` restores the NuGet package below. Visual Studio
installs it with its .NET desktop development workload, or get it from
[dotnet.microsoft.com](https://dotnet.microsoft.com/download).

# How it works

GameInput comes in two parts: the `Microsoft.GameInput` NuGet package, which a game builds with,
and a runtime DLL, which the user installs. This crate takes the package.
[Library/GameInput](../GameInput/README.md) says how KGames starts the runtime.

**The package.** `NuGetPackages.csproj` declares the NuGet packages that this crate needs, the way
a .NET project declares its own: `Microsoft.GameInput`, exactly 4.0.306. It's a C# project only
because developers and tools know `.csproj` files best: it has no C#, and it's there only to get
the package. `packages.lock.json` pins its content by its hash, as `Cargo.lock` does for crates. At
the root of the repository, `nuget.config` makes nuget.org the only package source, and
`global.json` asks for the .NET SDK 10 or later.

`build.rs` restores it with `dotnet restore --locked-mode`, which fails when the project asks for
other packages than the lock file pins, or a package that it downloads has another hash. NuGet puts
it in its global packages folder, which every checkout shares, normally
`%UserProfile%\.nuget\packages\microsoft.gameinput\4.0.306`, and `build.rs` asks MSBuild for its
folder, as `PkgMicrosoft_GameInput`. Nothing builds the project: `Directory.Build.props` sends what
the restore writes to `target\DotNet`. Then `build.rs` links the package's `GameInput.lib`, which
has Microsoft's loader, `GameInputInitialize`.

**The API.** KGames uses API version 4, from `native/include/v4/GameInput.h` in the package.
GameInput isn't in the Windows metadata that `windows-bindgen` reads, so `src/lib.rs`
declares the parts that KGames uses by hand, in the same shape as the generated bindings in
[Library/WindowsApiRaw](../WindowsApiRaw/README.md). Every vtable slot is listed in header order,
so it can be checked against the header.

Microsoft's online reference shows v4's signatures in its current view, but its older views,
and its pages for earlier versions, show older ones. For example, `RegisterDeviceCallback` has
the `statusFilter` and `enumerationKind` parameters there, which v4 removed. When they differ, the
v4 header is right.

**The license.** The package ships `GameInput.lib`'s source, `native/src/GameInput.cpp`, and its
header, under the MIT license, and KGames treats `GameInput.lib` as MIT-licensed too. The rest of
the package, like its runtimes, comes under Microsoft's own license terms, and stays out of the
repository.

# Updating GameInput

[Library/GameInput](../GameInput/README.md#updating-gameinput) says what to change for a new
version. If the package has a new API version, compare its header, in the package's folder, with
`src/lib.rs`, change the interface IDs and vtables to match, and update this file.

# Using more APIs

Add GameInput APIs to `src/lib.rs` by hand: replace the method's `usize` placeholder in the vtable
with its function pointer type, and add a method that calls it, in the shape that
`windows-bindgen`'s rules give the header's annotations, which the first comment in `src/lib.rs`
lists. Code that goes beyond the API goes in [Library/GameInput](../GameInput/README.md), never
here.

# Layout

| Path                    | Contents                                                                |
| ----------------------- | ----------------------------------------------------------------------- |
| `src/lib.rs`            | Hand-written GameInput v4 bindings, and the loader in `GameInput.lib`   |
| `build.rs`              | Restores the packages, and links `GameInput.lib`                        |
| `NuGetPackages.csproj`  | The NuGet packages that this crate needs: `Microsoft.GameInput` 4.0.306 |
| `packages.lock.json`    | The hash of each package's content, which a restore checks              |
| `Directory.Build.props` | Sends what a restore writes to `target\DotNet`                          |
