# Description

This is a Rust wrapper for Microsoft GameInput version 4.0.306.

Microsoft docs: https://learn.microsoft.com/en-us/gaming/gdk/docs/features/common/input/overviews/input-overview

# Prerequisites

Before using this crate, you must install GameInput 4.0.306 with this:

```powershell
winget install --exact --id "Microsoft.GameInput" --source "winget" --version "4.0.306"
```

# How it works

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

The installed runtime is in two places. `GameInputRedist.msi` installs it into
`C:\Program Files\Microsoft GameInput\x64`, with GameInput's service, `GameInputRedistService.exe`,
and `GameInputRawInputProxy.exe`, and writes that folder to the registry as `RedistDir`. Then it
runs the service with `/serviceconfig`, which copies `GameInputRedist.dll` into
`C:\Windows\System32`, and a 32-bit one into `SysWOW64`. The installer registers System32's copy as
what COM loads, in a 64-bit process, to carry calls to GameInput's service. Microsoft's loader
takes System32's too: it looks for `RedistDir`'s only when System32 has no `GameInputRedist.dll`.

KGames takes only the installed one, from System32:
`C:\Windows\System32\GameInputRedist.dll`. The runtime connects to its service on a thread of its
own as it starts, and COM loads System32's copy then, so a process that loaded Program Files' copy
would run two copies of the runtime, each with its own globals. `Test/GameInput/GameInput.rs` pins
that one runs.

`installed_runtime` in `Source/Installation.rs` checks that the DLL is there, and that its version
is 4.0.306.0, which it reads from the DLL's own bytes: its version resource's `VS_FIXEDFILEINFO`,
which starts with the signature `0xFEEF04BD`. That takes only the standard library, so `Build.rs`
includes the file, and runs the check with no dependencies. Every program runs it again as it
starts, even in a scripted run or with `--no-input`. `game_input::create` runs it too, then loads
the DLL, and returns its path and version with the API.

The DLL that loads is the one that was checked. `installed_runtime` reads the version through a
handle that holds the DLL open, sharing it only for reading, and `create` loads the DLL before it
lets go: until then, no one can write, rename or delete the DLL, nor rename its folder. Once it's
loaded, Windows keeps it from being written.

`create` loads it with `LoadLibraryExW` and `LOAD_LIBRARY_SEARCH_SYSTEM32`, so the DLLs that the
runtime imports, like `HID.DLL`, which isn't a known DLL, load from System32, and never from beside
the exe, where `LoadLibraryW`, which Microsoft's loader uses, looks first.
`Test/GameInput/RuntimeImports.rs` pins it.

The runtime still starts `GameInputRawInputProxy.exe`, from the Program Files folder, for each
program that reads keyboards and mice.

# Updating GameInput

1. Set `VERSION` and `INSTALL_COMMAND` in `Source/Installation.rs`, and the command in
   [AGENTS.md](../../AGENTS.md), the root [README](../../README.md), this file and
   [GameInputDemo](../../Program/GameInputDemo/README.md)'s. `Test/GameInput` checks that they all
   give it.
2. If the package has a new API version, compare its header with `Source/Library.rs`, change the
   interface IDs and vtables to match, and update this file. To read the header, extract the
   `.nupkg`, which is a zip file, into `Packages`, which Git ignores.

# Using more APIs

Add GameInput APIs to `Source/Library.rs` by hand: replace the method's
`usize` placeholder in the vtable with its function pointer type, and add a method that calls it.

# Layout

| Path                     | Contents                                                                    |
| ------------------------ | --------------------------------------------------------------------------- |
| `Source/Library.rs`      | Hand-written GameInput v4 bindings, and the loader for the runtime DLL      |
| `Source/Installation.rs` | Where the runtime DLL is installed, and the check of it and its version     |
| `Build.rs`               | Fails the build unless the runtime DLL 4.0.306.0 is installed               |
| `Packages`               | A NuGet package you extract to read its headers; not in Git                 |
