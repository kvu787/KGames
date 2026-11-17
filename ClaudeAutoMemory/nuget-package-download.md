---
name: nuget-package-download
description: "Official ways to download one exact NuGet package version (dotnet package download, nuget.exe install, the V3 flat container API) and how to check it; tested with Microsoft.GameInput 4.0.306 on 2026-11-09"
metadata:
  node_type: memory
  type: reference
  originSessionId: 3b11089e-6029-41b2-acc0-3fa686bfd3c5
  modified: 2026-11-09T23:19:07.670Z
---

- `dotnet package download Microsoft.GameInput@4.0.306 --source https://api.nuget.org/v3/index.json --output <folder>` is the purpose-built command (.NET SDK 10.0.2xx+; 10.0.403 is installed here). It writes NuGet's global-packages layout, `<folder>\microsoft.gameinput\4.0.306\`, extracted, plus the .nupkg and its `.nupkg.sha512`. Pass `--source` so the user's NuGet.config sources don't matter. Docs: learn.microsoft.com/dotnet/core/tools/dotnet-package-download
- `nuget.exe install <id> -Version <v> -OutputDirectory <folder>` also works; nuget.exe isn't installed here.
- Protocol for client implementers: service index `https://api.nuget.org/v3/index.json` -> `PackageBaseAddress/3.0.0` `@id` -> `{@id}{lowerid}/{lowerversion}/{lowerid}.{lowerversion}.nupkg` (docs: nuget/api/package-base-address-resource; base URL must come from the service index).
- Check: the catalog entry's `packageHash` (SHA512, base64) equals the .nupkg's SHA-512. GameInput 4.0.306's is hex `f2865d53...0be6104c` (same as the old Build.rs pinned). `dotnet nuget verify --all` checks Microsoft's author and nuget.org's repository signatures; both passed.

Declarative restore, tested 2026-11-09 for the plan to have KGames require the package: an SDK-style `.proj` (net10.0) with `<PackageReference Include="Microsoft.GameInput" Version="[4.0.306]" ExcludeAssets="all" GeneratePathProperty="true" />` and `RestorePackagesWithLockFile` restores fine (no NU1202 for this native-only package). `dotnet restore --locked-mode` fails with NU1004 when the manifest disagrees with `packages.lock.json`, and NU1403 when a downloaded package's hash differs; a no-op restore does NOT recheck the lock file's hash against an already-present package. `dotnet msbuild -getProperty:PkgMicrosoft_GameInput` gives the folder, but combined with `-restore` it doesn't fail on a lock mismatch, so restore first, then query. PowerShell splits `-property:X=Y` and `--getProperty:X`; call from a .cmd or Command. Restores left no MSBuild processes behind. Visual Studio installed this machine's .NET SDK (workloads: ManagedDesktop, NativeDesktop, NativeGame, Universal). WinUi3Sample already uses PackageReference with `Directory.Build.props` sending output to `target\DotNet`.

Related: [[gameinput-entry-points]], [[gameinput-docs-markdown]]
