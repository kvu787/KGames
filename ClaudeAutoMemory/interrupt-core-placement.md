---
name: interrupt-core-placement
description: "Uncapped frame rates swing 10-20% with whether the main thread shares CPU 1, where the GPU's interrupts and DPCs land; pin it when comparing"
metadata:
  node_type: memory
  type: reference
  originSessionId: c90870b0-64c1-4c3f-83b9-de8b01a0e396
  modified: 2026-11-11T21:49:21.362Z
---

Measured 2026-11-11 (SpinningCarTest against the C++ scene in C:\Users\K\Repository\Veehiicuul\3dTestScene_CppDx12,
both uncapped, borderless fullscreen, on the primary display, launched the human way): the GPU's interrupts and DPCs all
go to logical processor 1, about 10% interrupt time at idle and 26-37% interrupt plus 7-27% DPC time while a program
presents ~4000 frames a second. CPUs 0 and 1 are the favored P-cores (5.1 GHz) where Windows also puts a spinning main
thread. Sharing CPU 1 cost the C++ scene 26 us and SpinningCarTest 43 us a frame (Signal 3 us -> 57 us, Present 55 -> 110 us).
The C++ scene's thread sat on CPU 1 every run; KGames' moved, so its rate was bimodal between runs.

P-cores are 0, 1, 10-13, 22, 23 (GetSystemCpuSetInformation EfficiencyClass 1); the rest are E-cores.

**How to apply:** when comparing frame rates, pin each program's main (window) thread with SetThreadAffinityMask from
outside, to CPU 10 (clean) or CPU 1, and compare like with like; read per-core `% Interrupt Time` / `% DPC Time` with
Get-Counter. FrameTimes counts 4000+ fps fine. RunLightly's E-core runs understate the gap (13% there, 17-22% on
P-cores). See [[benchmark-methodology]], [[agent-sandbox-measuring]].
