---
name: disruptive-validation
description: "What a 'disruptive' validation of KGames covered on 2026-11-16, and what works from the agent's shell: focus, GameInput, PrintWindow, WM_CLOSE, FrameTimes"
metadata:
  node_type: memory
  type: project
  originSessionId: 11034d31-8295-473b-9072-544b7a8fc72d
  modified: 2026-11-16T10:25:38.346Z
---

On 2026-11-16 the user asked for a complete validation "using both non-disruptive and disruptive workflows" after `rustup update` (1.99.1 -> 1.100.0). The disruptive half, which they authorized for that request only, was:
- Release exes started directly (Start-Process, not RunLightly/RunInBackground), without `--no-activate`/`--no-input`, on `--display CVVNY2`, vsync on and `--no-vsync`, plus one `--display Primary` run like HumanRun. Launched from the agent's PowerShell they DID take focus (GetForegroundWindow matched) and GameInput started (GameInputDemo listed 7 devices, runtime GameInputRedist.dll 4.0.306.0 in system32, G-SYNC indicator on).
- PrintWindow(PW_CLIENTONLY|PW_RENDERFULLCONTENT) captured them; PostMessage WM_CLOSE (the taskbar's Close) made each exit 0 within 60-180 ms.
- FrameTimes ran concurrently on a focused program fine (sessions are per program and process): MinimalSceneTest 120.0 fps, 0 stutters, overlay plane.
- `cargo test --workspace` at full speed three times, and `benchmarks.exe` directly.

**Why:** the non-disruptive rules (no focus, no GameInput, light CPU) can't exercise the real input and present paths; the user wanted those covered too.

**How to apply:** never do this unasked ([[user-offers-hands-on-testing]]); check no game is running and the PC is unlocked first, and say in chat that windows will take focus. Real device input and input latency still need the user's hands. See [[light-load-while-gaming]], [[benchmark-methodology]].
