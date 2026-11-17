---
name: clipcursor-release-behavior
description: "Measured on this machine 2026-11-14 - when Windows releases a ClipCursor confinement on its own, and when it doesn't (Start menu, process exit); background programs can confine; EVENT_SYSTEM_FOREGROUND fires for Start"
metadata:
  node_type: memory
  type: project
  originSessionId: adde74bc-72a4-47b7-8444-560976075d03
  modified: 2026-11-14T14:38:50.421Z
---

A C probe (ClipCursor from one process, real Alt+Tab or SendInput from another, then GetClipCursor) measured, on Windows 11 26300:

- Windows releases a confinement when a normal top-level window is activated: by Alt+Tab, or by SetForegroundWindow from another process, even while the confining process's thread is stuck (not pumping messages).
- It does NOT release it when the Start menu ("Search", via the Windows key) takes the foreground, and NOT when the confining process exits normally while Start has the foreground. The cursor stayed confined to a dead process's rectangle until a normal window was activated (Escape closing Start, which returned the foreground to the previous app, released it).
- A program that is not the foreground window can confine the cursor (WS_EX_NOACTIVATE window, Claude app in front): Windows doesn't stop it.
- SetWinEventHook(EVENT_SYSTEM_FOREGROUND, WINEVENT_OUTOFCONTEXT) on a thread with its own message loop reports the Start menu taking the foreground, and the return.
- ClipCursor keeps the rectangle within the virtual screen (GetClipCursor gives the clipped one back); an empty rectangle pins the cursor to a point; an inverted one means no confinement. Without one, GetClipCursor gives the virtual screen. (Recorded in Documentation/LogicReview.md too.)

**Why:** KGames confines Windows' cursor while focused (its own GameInput-driven pointer); the user considers focus transitions critical.
**How to apply:** KGames must release its own confinement on losing the foreground and on exit/panic (Windows won't for Start or exit). The user chose no watcher thread (2026-11-14): the main thread checks on every pass. Compare a confinement against the client area clipped to the virtual screen. The probes' sources were in the session scratchpad (clipprobe\), not the repo.

Related: [[user-offers-hands-on-testing]]
