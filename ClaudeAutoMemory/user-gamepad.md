---
name: user-gamepad
description: "The user's GameSir G7 Pro 8K (3537:10C6, dongle) and Tarantula 8K PC (3537:103D, wired), both Xbox 360 mode; how to measure them, and their modes"
metadata:
  node_type: memory
  type: user
  originSessionId: 0dc694ea-f6cf-4ffc-8475-812254b1a7cb
  modified: 2026-11-12T21:40:26.751Z
---

Since 2026-11-12 the user keeps two GameSir controllers connected, both "Xbox 360 Controller for Windows" in GameInput (XInput mode):
- G7 Pro 8K on its 2.4 GHz dongle, `3537:10C6`. Holding View+Menu 2 s cycles XInput -> Switch (`057E:2009`, HID, 12-bit sticks, 1,000 reports/s) -> Android/PS4 (`054C:09CC`, DualShock 4 IDs) -> XInput. GameSir Connect only works in XInput mode.
- Tarantula 8K PC, wired, `3537:103D`. Modes: Home+X Xbox, Home+Y Switch, Home+A Android; auto by default. Its dead zones were set to 0 so its sticks give readings at rest.
Both turn themselves off after 10 minutes idle (press Home).

Measured that day with GameInput 4.0.306: a focused window with background input off gets ~7,500-7,700 readings/s from either at 8 kHz. Background input on makes GameInput poll Xbox 360-mode pads on a timer (64/s default timers, ~680 with 1 ms timers, ~950 with a 0.5 ms waker), so KGames now leaves it off. Windows' DirectInput HID copy (`HID\VID_...&IG_00`) read with ReadFile showed ~7,700 reports/s. Console probes without focus need background input on to see Xbox 360-mode readings at all, and then measure the slow path.

Related: [[gameinput-header-and-probes]], [[user-offers-hands-on-testing]]
