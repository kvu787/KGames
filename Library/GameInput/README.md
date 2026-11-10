# Description

This is a C++ library for Microsoft GameInput version 4.0.306.

Microsoft docs: https://learn.microsoft.com/en-us/gaming/gdk/docs/features/common/input/overviews/input-overview

# Prerequisites

Before using this library, you must install GameInput 4.0.306 with this:

```powershell
winget install --exact --id "Microsoft.GameInput" --source "winget" --version "4.0.306"
```

And the .NET SDK, 10 or later, whose `dotnet` restores the NuGet package below. Visual Studio
installs it with its .NET desktop development workload, or get it from
[dotnet.microsoft.com](https://dotnet.microsoft.com/download).

# How it works

GameInput comes in two parts: the `Microsoft.GameInput` NuGet package, which a game builds with,
and a runtime DLL, which the user installs.

**The package.** `NuGetPackages.proj` declares the NuGet packages that this library needs, the way
a .NET project declares its own: `Microsoft.GameInput`, exactly 4.0.306. `packages.lock.json` pins
its content by its hash. At the root of the repository, `NuGet.config` makes nuget.org the only
package source, and `global.json` asks for the .NET SDK 10 or later.

`Program\Build\Build.ps1` restores it with `dotnet restore --locked-mode` before it builds, which
fails when the project asks for other packages than the lock file pins, or a package that it
downloads has another hash. It restores only when what declares the packages is newer than the last
restore, so a build that has them doesn't start `dotnet`. NuGet puts the package in its global
packages folder, which every checkout shares, normally
`%UserProfile%\.nuget\packages\microsoft.gameinput\4.0.306`, and writes that folder, as
`PkgMicrosoft_GameInput`, to a file of properties. Nothing builds the project:
`Directory.Build.props` sends what the restore writes to `target\DotNet`.

`GameInput.props`, which `KGames.props` imports into every C++ project, reads the package's folder
from that file, as `GameInputPackage`, and puts the package's `native\include` on the include path.
Without a restore, like in a build that `Build.ps1` didn't start, `GameInputPackage` is empty, and
`GameInput.vcxproj` fails, and says to build with `Build.ps1`.

**The API.** KGames uses API version 4, from Microsoft's own header, `native/include/GameInput.h` in
the package, which is its `v4/GameInput.h`. C++ includes it as `<GameInput.h>`, and it puts the API
in the namespace `GameInput::v4`. This library's `GameInput.h`, which is `"GameInput/GameInput.h"`,
includes it, and names all of it in `game_input` too, so `game_input::IGameInput` is
`GameInput::v4::IGameInput`. It adds `==` for the structs that KGames compares, like
`GameInputKeyState`, which the header leaves out. `GameInput.cpp` checks the header's API version,
and the sizes that v4 gives those structs, as it compiles, so another version's header fails the
build.

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
`game_input::create` calls `GameInputInitialize`, from the package's `GameInput.lib`, with the IID
of v4's `IGameInput`. `GameInput.vcxproj` takes that library into this one, so every program that
links this library links it. That's Microsoft's loader, whose source is the package's
`native/src/GameInput.cpp`. It looks at Windows' `GameInput.dll`; the installed
`GameInputRedist.dll`, from System32, or from `RedistDir` when System32 has none; and a
`GameInputRedist.dll` beside the exe. It loads the one with the newest file version, and on a tie,
the one beside the exe, then the installed one. Then it calls that DLL's own `GameInputInitialize`.

With GameInput 4.0.306 installed and nothing beside the exe, that's
`C:\Windows\System32\GameInputRedist.dll`. The runtime connects to its service on a thread of its
own as it starts, and COM loads System32's copy then, so the process runs one copy of the runtime.
A 4.0.306 `GameInputRedist.dll` beside the exe would be loaded instead, sideloaded, so nothing in
the repo may put one there. `Test/GameInput/GameInput.cpp` pins both.

The loader loads the runtime with `LoadLibraryW`, so Windows looks for the DLLs that the runtime
imports beside the exe first, before System32: `HID.DLL` and `bcrypt.dll`, which aren't known DLLs.
`Test/GameInput/RuntimeImports.cpp` pins it, in a copy of the test program, in a folder of its own.

`installed_runtime` in `Installation.cpp` checks that the installed DLL is there, and that its
version is 4.0.306.0, which it reads from the DLL's own bytes: its version resource's
`VS_FIXEDFILEINFO`, which starts with the signature `0xFEEF04BD`. That takes only `Library/Base`,
so `InstallationCheck`, a console program, compiles the same file, and `GameInput.vcxproj` runs it
before it compiles, on every build, and fails the build with what it says. Every program runs the
check again as it starts, even in a scripted run or with `--no-input`, and `create` runs it before
the loader, so that a machine without it says how to install it, instead of failing to start
GameInput. `create` returns the API with the runtime that the loader loaded: the DLL that holds the
code of the `IGameInput` that it returns, and that DLL's version.

The loader keeps the runtime that it loaded in globals that it doesn't guard, so `create` calls it
one thread at a time. It also keeps a reference to the first `IGameInput` that it creates, for the
life of the process, since the runtime crashes when one thread releases the last one while another
creates one.

The runtime still starts `GameInputRawInputProxy.exe`, from the Program Files folder, for each
program that reads keyboards and mice.

**The license.** The package ships `GameInput.lib`'s source, `native/src/GameInput.cpp`, and its
header, under the MIT license, and KGames treats `GameInput.lib` as MIT-licensed too. The rest of
the package, like its runtimes, comes under Microsoft's own license terms, and stays out of the
repository.

# Updating GameInput

1. Set `Version` and `InstallCommand` in `Installation.h`, the version that
   `not_installed_message` names in `Installation.cpp`, the version in `NuGetPackages.proj`, and
   the command in [AGENTS.md](../../AGENTS.md), the root [README](../../README.md), this file and
   [GameInputDemo](../../Program/GameInputDemo/README.md)'s. Then update the lock file with
   `dotnet restore Library\GameInput\NuGetPackages.proj --force-evaluate`. `Test/GameInput` checks
   that they all give the same version.
2. If the package has a new API version, its `native/include/GameInput.h` is that version's, in
   another namespace. Change the namespace that `GameInput.h` names, `GameInput::v4`, and the
   version and sizes that `GameInput.cpp` checks, fix what no longer compiles, and update this file.

# Using more APIs

Microsoft's header declares every API, and `game_input` names them all, so there's nothing to add.
A struct of GameInput's that code compares needs an `==` in `GameInput.h`, beside the others.

# Layout

| Path                                 | Contents                                                                      |
| ------------------------------------ | ----------------------------------------------------------------------------- |
| `GameInput.h`, `GameInput.cpp`       | Microsoft's GameInput v4 API in `game_input`, and `create`, which starts it   |
| `Installation.h`, `Installation.cpp` | Where the runtime DLL is installed, and the check of it and its version       |
| `GameInput.vcxproj`                  | Runs the check of the runtime DLL, and takes `GameInput.lib` into the library |
| `InstallationCheck`                  | The program that a build checks the runtime DLL with                          |
| `GameInput.props`                    | Where the package is, and its header on every project's include path          |
| `NuGetPackages.proj`                 | The NuGet packages that this library needs: `Microsoft.GameInput` 4.0.306     |
| `packages.lock.json`                 | The hash of each package's content, which a restore checks                    |
| `Directory.Build.props`              | Sends what a restore writes to `target\DotNet`                                |
