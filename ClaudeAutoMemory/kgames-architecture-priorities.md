---
name: kgames-architecture-priorities
description: "User's priorities for KGames' game/app layer architecture - fast understanding, then testability; virtualize little beyond core CPU logic"
metadata:
  node_type: memory
  type: user
  originSessionId: e513c95d-d133-46c9-a0c7-b51ff80aebf3
  modified: 2026-11-14T19:30:30.379Z
---

As of 2026-11-14, the user's main goals for KGames' architecture are:

- understanding the codebase and its dependencies (D3D12, Win32, Rust, DirectWrite, ...) as much and as quickly as possible;
- an architecture that optimizes for fast understanding of KGames and for writing games/apps on it;
- good testability.

They don't think it's necessary to virtualize everything outside core CPU logic (rejected the "virtualize all I/O" direction of Documentation/Virtualization.md as a goal in itself).

**How to apply:** prefer the design with fewer abstractions to learn when it tests well enough; propose a fake/trait only where a real resource (disk, GPU) can't give isolated, fast, reliable tests. Weigh "how quickly can someone read and understand this" explicitly in recommendations. Related: [[prefer-official-declarative-ways]].
