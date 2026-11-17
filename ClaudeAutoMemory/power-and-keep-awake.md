---
name: power-and-keep-awake
description: "User's machine never sleeps, hibernates or turns its screen off on a timer, so the Claude app's keep-awake setting is off on purpose"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 30f3aecf-feeb-4dc2-b486-cca178858d5e
  modified: 2026-10-31T01:28:05.147Z
---

The user turned off the Claude app's "Keep computer awake while Claude works" setting (2026-10-30) because Windows' screen-off, sleep and hibernate timeouts are all "Never", plugged in and on battery. Don't suggest turning keep-awake back on, and don't treat it as needed for agents to keep working while the machine is locked. The repo's AGENTS.md ("Power and locking") states the same assumption for agents.

**Why:** The user considers keep-awake redundant given those power settings.

**How to apply:** When reasoning about whether the machine stays awake (unattended or locked work), rely on the power settings, not on the app. Related: [[display-setup]].
