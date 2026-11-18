# Description

What KGames adds to the Rust bindings for Microsoft GameInput version 4.0.306 in
[Library/game-input-raw](../game-input-raw/README.md): `create`, which starts GameInput, and the check
that the runtime that KGames needs is installed. Bindings go in GameInputRaw, never here.

Microsoft docs: https://learn.microsoft.com/en-us/gaming/gdk/docs/features/common/input/overviews/input-overview

# Prerequisites

Before using this crate, you must install GameInput 4.0.306 with this:

```powershell
winget install --exact --id "Microsoft.GameInput" --source "winget" --version "4.0.306"
```

And the .NET SDK, 10 or later, which [Library/game-input-raw](../game-input-raw/README.md) builds with.

# How it works

GameInput comes in two parts: the `Microsoft.GameInput` NuGet package, which a game builds with,
and a runtime DLL, which the user installs. [Library/game-input-raw](../game-input-raw/README.md) takes
the package, and links its `GameInput.lib`. This crate starts the runtime.

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
GameInputRaw's `build.rs` links, with the IID of v4's `IGameInput`. That's Microsoft's loader, whose
source is the package's `native/src/GameInput.cpp`. It looks at Windows' `GameInput.dll`; the
installed `GameInputRedist.dll`, from System32, or from `RedistDir` when System32 has none; and a
`GameInputRedist.dll` beside the exe. It loads the one with the newest file version, and on a tie,
the one beside the exe, then the installed one. Then it calls that DLL's own `GameInputInitialize`.

With GameInput 4.0.306 installed and nothing beside the exe, that's
`C:\Windows\System32\GameInputRedist.dll`. The runtime connects to its service on a thread of its
own as it starts, and COM loads System32's copy then, so the process runs one copy of the runtime.
A 4.0.306 `GameInputRedist.dll` beside the exe would be loaded instead, sideloaded, so nothing in
the repo may put one there. `Test/game-input/tests/game_input.rs` pins both.

The loader loads the runtime with `LoadLibraryW`, so Windows looks for the DLLs that the runtime
imports beside the exe first, before System32: `HID.DLL` and `bcrypt.dll`, which aren't known DLLs.
`Test/game-input/tests/runtime_imports.rs` pins it.

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

# Updating GameInput

1. Set `VERSION` and `INSTALL_COMMAND` in `src/installation.rs`, the version in GameInputRaw's
   `NuGetPackages.csproj`, and the command in [AGENTS.md](../../AGENTS.md), the root
   [README](../../README.md), this file and [GameInputDemo](../../Program/game-input-demo/README.md)'s.
   Then update the lock file with
   `dotnet restore Library\game-input-raw\NuGetPackages.csproj --force-evaluate`. `Test/game-input`
   checks that they all give the same version.
2. If the package has a new API version, update GameInputRaw's bindings, as
   [its README](../game-input-raw/README.md#updating-gameinput) says.

# Layout

| Path                  | Contents                                                                             |
| --------------------- | ------------------------------------------------------------------------------------ |
| `src/lib.rs`          | `create`, which starts GameInput, and `Runtime`, the runtime that it started         |
| `src/installation.rs` | Where the runtime DLL is installed, and the check of it and its version              |
| `build.rs`            | Fails the build unless the runtime DLL is installed, as `src/installation.rs` checks |
