# GameInput

Microsoft's [GameInput](https://learn.microsoft.com/en-us/gaming/gdk/docs/features/common/input/overviews/input-overview)
API for KGames: hand-written bindings for its version 4, a loader that finds the newest runtime,
and a build script that puts the runtime next to the exe of every program that uses this crate.
[GameInputDemo](../../Program/GameInputDemo/README.md) shows everything it reads.

## How it works

GameInput has two parts: a header that a game compiles against, and a runtime DLL.

**The API.** KGames uses API version 4, from `native/include/v4/GameInput.h` in the
[Microsoft.GameInput](https://www.nuget.org/packages/Microsoft.GameInput) NuGet package.
GameInput isn't in the Windows metadata that `windows-bindgen` reads, so
`Source/Library.rs` declares the parts that KGames uses by hand, in the same shape as the
generated bindings in [Library/WindowsApi](../WindowsApi/README.md). Every vtable slot is
listed in header order, so it can be checked against the header.

Microsoft's online reference still shows older signatures for some methods. For example,
its `RegisterDeviceCallback` has the `statusFilter` and `enumerationKind` parameters that
v4 removed. When they differ, the v4 header is right.

**The runtime.** Windows 11 ships a `GameInput.dll` that only supports API version 0.
Newer runtimes come from the NuGet package, either installed with `GameInputRedist.msi`,
or as a `GameInputRedist.dll` that sits next to the game's exe. KGames does the second:
`Build.rs` downloads the package version it pins and checks its SHA-512 against NuGet's.
It then copies `GameInputRedist.dll` and its helper `GameInputRawInputProxy.exe` next to the
exe of every program that depends on this crate. The package is kept in `Packages`, which Git
ignores, so the first build needs internet access.

C++ games find the runtime by linking `GameInput.lib`, whose source is in the package
as `native/src/GameInput.cpp`. `game_input::create` is a Rust port of it: of the copy
built into Windows, an installed redistributable, and one in the exe's folder, it loads
the newest, and returns its path and version with the API.

## Updating GameInput

1. Set `GAME_INPUT_VERSION` in `Build.rs` to the new version, and `GAME_INPUT_SHA512` to its hash.
   The hash is the `packageHash` in the version's NuGet catalog entry, which is Base64, written as hex.
2. If the package has a new API version, compare its header with `Source/Library.rs`,
   change the interface IDs and vtables to match, and update this file.

## Using more APIs

Add GameInput APIs to `Source/Library.rs` by hand: replace the method's
`usize` placeholder in the vtable with its function pointer type, and add a method that calls it.

## Layout

| Path                | Contents                                                                    |
| ------------------- | --------------------------------------------------------------------------- |
| `Source/Library.rs` | Hand-written GameInput v4 bindings, and the loader for the runtime DLL      |
| `Build.rs`          | Fetches the GameInput NuGet package, and deploys its runtime next to exes   |
| `Packages`          | The downloaded GameInput NuGet package; not in Git                          |
