---
name: nvidia-app-7zip
description: A full 7-Zip 26.01 (with NSIS support) ships with the NVIDIA App; no standalone 7-Zip is installed
metadata:
  node_type: memory
  type: reference
  originSessionId: dde52c3b-d94c-4898-95e3-0ed630e58887
  modified: 2026-11-17T04:04:00.582Z
---

`C:\Program Files\NVIDIA Corporation\NVIDIA App\7z.exe` (+ `7z.dll`) is 7-Zip 26.01 x64, Authenticode-signed by NVIDIA, with the full format set, including NSIS installers (`7z l -tnsis`, `7z x -tnsis`). There's no standalone 7-Zip, NanaZip or 7za on the machine (checked 2026-11-16). This build doesn't emit the decompiled `[NSIS].nsi` script.

**How to apply:** use it to list or extract archives and installers read-only, through RunLightly, into a new empty scratch folder. Pass the output folder as a relative `-o` path (see [[runlightly-exe-arguments]]).
