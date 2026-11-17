---
name: gameinput-entry-points
description: "Which GameInput header, lib and DLL declares, defines or exports GameInputCreate and GameInputInitialize, checked with dumpbin, link probes and a runtime probe on 2026-11-09"
metadata:
  node_type: memory
  type: reference
  originSessionId: 3b11089e-6029-41b2-acc0-3fa686bfd3c5
  modified: 2026-11-09T22:11:10.693Z
---

Checked on this machine 2026-11-09 (dumpbin /exports, /linkermember, C/C++ link probes, a console probe), GameInput 4.0.306:

- Windows' `System32\GameInput.dll` (0.2309.26100.9278): exports only `GameInputCreate`. With the redist installed, calling it loads `GameInputRedist.dll` and the object answers QueryInterface for v1 and v4, so it hands off to the redist.
- `GameInputRedist.dll` (System32, SysWOW64, Program Files, NuGet bin x64/x86/arm64/xbox): exports `GameInputCreate` (ordinal 1) and `GameInputInitialize` (ordinal 2); PC copies also COM exports and `GetProxyDllInfo`.
- Windows SDK `um\gameinput.lib`: import library, `GameInputCreate` -> `GameInput.dll` by ordinal.
- NuGet `native\lib\<arch>\GameInput.lib`: static, defines only `GameInputInitialize` (x86 `_GameInputInitialize@8`), the loader from `native\src\GameInput.cpp`.
- Headers: v0 declares `extern "C" GameInputCreate(IGameInput**)`; v1/v2 declare `extern "C" GameInputCreate(GameInputLegacy**)` plus a C++ inline overload; v3/v4 declare `GameInputInitialize` and define an inline `GameInputCreate`, and `#error` in C.
- Link probes with the 4.0.306 NuGet lib: v3/v4 link (no DLL import); v0, v1, v2 fail with unresolved `GameInputCreate`. v4 with the SDK lib fails on `GameInputInitialize`.

Related: [[gameinput-header-and-probes]], [[gameinput-docs-markdown]]
