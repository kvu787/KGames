# GameInput

Microsoft's [GameInput](https://learn.microsoft.com/en-us/gaming/gdk/docs/features/common/input/overviews/input-overview)
API for KGames: hand-written bindings for its version 4, and a loader for the one runtime that
KGames uses, version 4.0.306, installed with Microsoft's `GameInputRedist.msi`.
[GameInputDemo](../../Program/GameInputDemo/README.md) shows everything it reads, and
[Library/UserInterface](../UserInterface/README.md) reads all of every program's input with it:
keyboards, mice and gamepads.

## Installing GameInput

KGames needs GameInput 4.0.306, installed once, from an elevated prompt, with:

```powershell
winget install --exact --id "Microsoft.GameInput" --source "winget" --version "4.0.306"
```

winget runs the same `GameInputRedist.msi` that the
[Microsoft.GameInput](https://www.nuget.org/packages/Microsoft.GameInput) NuGet package has in
its `redist` folder, so running that MSI installs it too. Installing it is up to you. Nothing in
KGames installs it, or fetches a runtime of its own.

Without it, KGames doesn't build: `Build.rs` fails, and says why. A program built before it was
uninstalled doesn't start, and says why on standard error. Both say so when another version is
installed instead, too, like a newer one that another game's installer put in. The MSI doesn't
replace a newer version, so uninstall that one first.

## How it works

GameInput has two parts: a header that a game compiles against, and a runtime DLL.

**The API.** KGames uses API version 4, from `native/include/v4/GameInput.h` in the NuGet package.
GameInput isn't in the Windows metadata that `windows-bindgen` reads, so `Source/Library.rs`
declares the parts that KGames uses by hand, in the same shape as the generated bindings in
[Library/WindowsApi](../WindowsApi/README.md). Every vtable slot is listed in header order, so it
can be checked against the header.

Microsoft's online reference still shows older signatures for some methods. For example,
its `RegisterDeviceCallback` has the `statusFilter` and `enumerationKind` parameters that
v4 removed. When they differ, the v4 header is right.

**The runtime.** There can be three: the `GameInput.dll` that comes with Windows 11, which only
supports API version 0; a `GameInputRedist.dll` that `GameInputRedist.msi` installs; and a
`GameInputRedist.dll` beside a game's exe, which the game sideloads. Microsoft's loader, which C++
games get by linking `GameInput.lib`, takes the newest, and the one beside the exe on a tie. A
sideloaded runtime can do less than an installed one: it can't enable background input, and
GameInput's service doesn't manage its input focus, as
[Microsoft's documentation](https://learn.microsoft.com/en-us/gaming/gdk/docs/features/common/input/overviews/input-agility-redistributable#limitations)
says.

KGames takes only the installed one. `Source/Installation.rs` asks Windows Installer whether
`GameInputRedist.msi` 4.0.306's product is installed, and where it put the x64
`GameInputRedist.dll`, in `C:\Program Files\Microsoft GameInput\x64`, and checks that the DLL's
version is 4.0.306.0. `game_input::create` loads it from there, and returns its path and version
with the API. `Build.rs` runs the same check, so the build fails where a program wouldn't start.

The installed runtime still starts its `GameInputRawInputProxy.exe`, from the same folder, for
each program that reads keyboards and mice.

## Updating GameInput

1. Read the new version's `GameInputRedist.msi`: the `ProductCode` in its `Property` table, and the
   `ComponentId` of the component whose key path is the x64 `GameInputRedist.dll`, in its
   `Component` table. Orca, from the Windows SDK, shows them, or PowerShell, with the
   `WindowsInstaller.Installer` COM object.
2. Set `VERSION`, `PRODUCT_CODE`, `RUNTIME_COMPONENT` and `INSTALL_COMMAND` in
   `Source/Installation.rs`, and the command in [AGENTS.md](../../AGENTS.md), the root
   [README](../../README.md), this file and [GameInputDemo](../../Program/GameInputDemo/README.md)'s.
   `Test/GameInput` checks that they all give it.
3. If the package has a new API version, compare its header with `Source/Library.rs`, change the
   interface IDs and vtables to match, and update this file. To read the header, extract the
   `.nupkg`, which is a zip file, into `Packages`, which Git ignores.

## Using more APIs

Add GameInput APIs to `Source/Library.rs` by hand: replace the method's
`usize` placeholder in the vtable with its function pointer type, and add a method that calls it.

## Layout

| Path                     | Contents                                                                    |
| ------------------------ | --------------------------------------------------------------------------- |
| `Source/Library.rs`      | Hand-written GameInput v4 bindings, and the loader for the runtime DLL      |
| `Source/Installation.rs` | Finds GameInput 4.0.306's runtime where `GameInputRedist.msi` installed it  |
| `Build.rs`               | Fails the build unless GameInput 4.0.306 is installed                       |
| `Packages`               | A NuGet package you extract to read its headers; not in Git                 |
