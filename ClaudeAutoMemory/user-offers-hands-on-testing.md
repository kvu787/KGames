---
name: user-offers-hands-on-testing
description: "The user will run disruptive steps or test with real devices on request; ask them when that's what settles a question"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 0dc694ea-f6cf-4ffc-8475-812254b1a7cb
  modified: 2026-11-12T19:15:54.679Z
---

The user said (2026-11-12): if a disruptive workflow is needed, or they need to actively test things to find out what's really going on, tell them.

**Why:** the non-disruptive rules (no focus stealing, no SendInput, CVVNY2 only) stop agents from settling some questions themselves, like how a focused window behaves with a real gamepad, and the user would rather help than have a guess.

**How to apply:** first get as far as background measurement allows (console probes, scripts, tests). Then ask for the specific hands-on step, with exact steps to try and what to look for, instead of reporting an unverified conclusion. Never do the disruptive thing without asking.

Related: [[user-gamepad]], [[light-load-while-gaming]]
