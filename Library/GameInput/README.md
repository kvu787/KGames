# Description

This is a Rust wrapper for Microsoft GameInput version 4.0.306.

Microsoft docs: https://learn.microsoft.com/en-us/gaming/gdk/docs/features/common/input/overviews/input-overview

# Prerequisites

Before using this crate, you must install GameInput 4.0.306 with this:

```powershell
winget install --exact --id "Microsoft.GameInput" --source "winget" --version "4.0.306"
```

And the .NET SDK, 10 or later, whose `dotnet` restores the NuGet package below. Visual Studio
installs it with its .NET desktop development workload, or get it from
[dotnet.microsoft.com](https://dotnet.microsoft.com/download).

# How it works

GameInput comes in two parts: the `Microsoft.GameInput` NuGet package, which a game builds with,
and a runtime DLL, which the user installs.

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
the restore writes to `target\DotNet`.

**The API.** KGames uses API version 4, from `native/include/v4/GameInput.h` in the package.
GameInput isn't in the Windows metadata that `windows-bindgen` reads, so `src/lib.rs`
declares the parts that KGames uses by hand, in the same shape as the generated bindings in
[Library/WindowsApi](../WindowsApi/README.md). Every vtable slot is listed in header order, so it
can be checked against the header.

Microsoft's online reference shows v4's signatures in its current view, but its older views,
and its pages for earlier versions, show older ones. For example, `RegisterDeviceCallback` has
the `statusFilter` and `enumerationKind` parameters there, which v4 removed. When they differ, the
v4 header is right.

**The runtime.** There can be three: the `GameInput.dll` that comes with Windows 11, which only
supports API version 0; a `GameInputRedist.dll` that `GameInputRedist.msi` installs; and a
`GameInputRedist.dll` beside a game's exe, which the game sideloads, like the ones in the package's
`native\bin`. A sideloaded runtime can do less than an installed one: it can't enable background
input, and GameInput's service doesn't manage its input focus, as
[Microsoft's documentation](https://learn.microsoft.com/en-us/gaming/gdk/docs/features/common/input/overviews/input-agility-redistributable#limitations)
says.

The installed runtime is in two places. `GameInputRedist.msi` installs it into
`C:\Program Files\Microsoft GameInput\x64`, with GameInput's service, `GameInputRedistService.exe`,
and `GameInputRawInputProxy.exe`, and writes that folder to the registry as `RedistDir`. Then it
runs the service with `/serviceconfig`, which copies `GameInputRedist.dll` into
`C:\Windows\System32`, and a 32-bit one into `SysWOW64`. The installer registers System32's copy as
what COM loads, in a 64-bit process, to carry calls to GameInput's service.

**Starting it.** KGames starts GameInput the way Microsoft's v4 header does for C++ games:
`game_input::create` calls `GameInputInitialize`, from the package's `GameInput.lib`, which
`build.rs` links, with the IID of v4's `IGameInput`. That's Microsoft's loader, whose source is the
package's `native/src/GameInput.cpp`. It looks at Windows' `GameInput.dll`; the installed
`GameInputRedist.dll`, from System32, or from `RedistDir` when System32 has none; and a
`GameInputRedist.dll` beside the exe. It loads the one with the newest file version, and on a tie,
the one beside the exe, then the installed one. Then it calls that DLL's own `GameInputInitialize`.

With GameInput 4.0.306 installed and nothing beside the exe, that's
`C:\Windows\System32\GameInputRedist.dll`. The runtime connects to its service on a thread of its
own as it starts, and COM loads System32's copy then, so the process runs one copy of the runtime.
A 4.0.306 `GameInputRedist.dll` beside the exe would be loaded instead, sideloaded, so nothing in
the repo may put one there. `Test/GameInput/tests/game_input.rs` pins both.

The loader loads the runtime with `LoadLibraryW`, so Windows looks for the DLLs that the runtime
imports beside the exe first, before System32: `HID.DLL` and `bcrypt.dll`, which aren't known DLLs.
`Test/GameInput/tests/runtime_imports.rs` pins it.

`installed_runtime` in `src/installation.rs` checks that the installed DLL is there, and that its
version is 4.0.306.0, which it reads from the DLL's own bytes: its version resource's
`VS_FIXEDFILEINFO`, which starts with the signature `0xFEEF04BD`. That takes only the standard
library, so `build.rs` includes the file, and runs the check with no dependencies. Every program
runs it again as it starts, even in a scripted run or with `--no-input`, and `create` runs it before
the loader, so that a machine without it says how to install it, instead of failing to start
GameInput. `create` returns the API with the runtime that the loader loaded: the DLL that holds the
code of the `IGameInput` that it returns, and that DLL's version.

The loader keeps the runtime that it loaded in globals that it doesn't guard, so `create` calls it
one thread at a time.

The runtime still starts `GameInputRawInputProxy.exe`, from the Program Files folder, for each
program that reads keyboards and mice.

**The license.** The package ships `GameInput.lib`'s source, `native/src/GameInput.cpp`, and its
header, under the MIT license, and KGames treats `GameInput.lib` as MIT-licensed too. The rest of
the package, like its runtimes, comes under Microsoft's own license terms, and stays out of the
repository.

# Updating GameInput

1. Set `VERSION` and `INSTALL_COMMAND` in `src/installation.rs`, the version in
   `NuGetPackages.csproj`, and the command in [AGENTS.md](../../AGENTS.md), the root
   [README](../../README.md), this file and [GameInputDemo](../../Program/GameInputDemo/README.md)'s.
   Then update the lock file with
   `dotnet restore Library\GameInput\NuGetPackages.csproj --force-evaluate`. `Test/GameInput` checks
   that they all give the same version.
2. If the package has a new API version, compare its header, in the package's folder, with
   `src/lib.rs`, change the interface IDs and vtables to match, and update this file.

# Using more APIs

Add GameInput APIs to `src/lib.rs` by hand: replace the method's
`usize` placeholder in the vtable with its function pointer type, and add a method that calls it.

# Layout

| Path                    | Contents                                                                 |
| ----------------------- | ------------------------------------------------------------------------ |
| `src/lib.rs`            | Hand-written GameInput v4 bindings, and `create`, which starts GameInput |
| `src/installation.rs`   | Where the runtime DLL is installed, and the check of it and its version  |
| `build.rs`              | Checks the runtime DLL, restores the packages, and links `GameInput.lib` |
| `NuGetPackages.csproj`  | The NuGet packages that this crate needs: `Microsoft.GameInput` 4.0.306  |
| `packages.lock.json`    | The hash of each package's content, which a restore checks               |
| `Directory.Build.props` | Sends what a restore writes to `target\DotNet`                           |
