---
name: gameinput-header-and-probes
description: "Where GameInput 4.0.306's v4 GameInput.h, GameInput.lib and loader source are on this machine (NuGet's packages folder, which every build restores); MSVC vcvars64 path for compiling small C probes"
metadata:
  node_type: memory
  type: reference
  originSessionId: 95884db2-8ff8-48dd-bb82-73a1a6ac4844
  modified: 2026-11-10T00:02:19.798Z
---

GameInput 4.0.306's NuGet package is the ground truth for the v4 API (signatures, vtable order, struct layouts). Since 2026-11-09 KGames declares it in `Library/GameInput/NuGetPackages.csproj` and Build.rs restores it with dotnet, so every build has it at:

- `C:\Users\K\.nuget\packages\microsoft.gameinput\4.0.306\` (`native\include\v4\GameInput.h`, `native\lib\x64\GameInput.lib`, `native\src\GameInput.cpp`, the loader that KGames now links)

An older copy, byte for byte the same: `C:\Users\K\Downloads\microsoft.gameinput.4.0.306\` (the user pointed to it). The main checkout's old hand-extracted `Library\GameInput\Packages` was deleted on 2026-11-09.

Visual Studio 18 Community is installed, so a small C or C++ probe can be compiled against Windows' or GameInput's headers to get real `sizeof`s or test API behavior: run `call "C:\Program Files\Microsoft Visual Studio\18\Community\VC\Auxiliary\Build\vcvars64.bat"` then `cl /nologo probe.c` from a .cmd file in the scratchpad. Library/GameInput's compile-time size asserts came from such a probe.

Related: [[gameinput-docs-markdown]], [[nuget-package-download]], [[gameinput-entry-points]]
