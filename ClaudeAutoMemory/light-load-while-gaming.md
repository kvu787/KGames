---
name: light-load-while-gaming
description: "The user plays games (AoE4, Ace Combat 8) while agents work; every build, test and program run must go through RunLightly or RunInBackground"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 8788b840-3f3e-4fdb-aab6-6edaeeb5828c
  modified: 2026-11-06T08:28:52.229Z
---

The user games while coding agents work in this repo: Age of Empires IV (RelicCardinal.exe) on 2025-11-05, Ace Combat 8
(AceCombat8.exe, ~17 GB working set, ~37% of the GPU) later that session. The game runs on the primary display (9XH36C then; KXGR16, the middle one, on 2026-11-16 per IdentifyDisplays);
RunInBackground's programs go to CVVNY2, the left HDMI monitor.

**Why:** a stutter in the game is the one thing the user asked to avoid above all else ("especially ensure that the
reusable agent workflow doesn't interrupt" the game).

**How to apply:** run every build, test and generator with `Program\RunLightly\RunLightly.ps1 <command>` and every GUI
program with `Program\RunInBackground\RunInBackground.ps1 <program>`, as AGENTS.md says. Give `cargo` as RunLightly's
command directly, not from a PowerShell script: rustup's cargo puts itself in a job of its own, which can't be adopted
into RunLightly's afterwards. Keep your own tool commands light too (no heavy scans outside RunLightly). Measure the
game's load with the process names above.
