---
name: cpp-port-branch
description: "KGames stays in Rust (decided 2026-11-11); the C++ rewrite is kept on branch cpp-rewrite (also on origin) as a frozen reference; the Rust that took its findings is on main"
metadata:
  node_type: memory
  type: project
  originSessionId: faeb427f-1da3-4f99-9eac-52042df25006
  modified: 2026-11-11T09:11:02.908Z
---

On 2026-11-10 the user asked, out of curiosity about performance and code structure, for all of
KGames' Rust to be rewritten in C++ with MSVC. On 2026-11-11, after the comparison
(`Documentation/CppVersusRust.md`), the user chose to stay with Rust and to keep the C++ as a
historical reference that they push to GitHub themselves.

- `cpp-rewrite` (local and `origin/cpp-rewrite`): the C++ rewrite, frozen. Don't develop it, and
  don't delete it.
- The fixes the C++ reviews found in the Rust, three speed-ups (quick hasher, `#[inline]` math,
  JSON runs) and the comparison document are on `main` (`5e51934`). The branches
  `rust-after-cpp-comparison` and `claude/kgames-rust-cpp-port-ff3f52` and their worktree were
  deleted on 2026-11-11 as redundant.

**Why:** the C++ bought nothing a user would notice, builds nine times slower, loses the
compiler's checks that agent-written code leans on, and is a quarter more code.

**How to apply:** work on the Rust. Check which one a checkout is before using commands from
memory: on the Rust (`Cargo.toml` at the root), the cargo notes in [[light-load-while-gaming]],
[[runlightly-exe-arguments]] and [[worktree-lfs-pointers]] hold. Only on `cpp-rewrite`
(`KGames.slnx` at the root): `Program\Build\Build.ps1 [--release] [<Project>]` and
`Program\RunTests\RunTests.ps1`, through RunLightly, with MSBuild arguments written `/p:Name=Value`.
A known open defect in both: a program started with a launcher's show command (Explorer,
plain `Start-Process`) comes up not maximized and shows the unsupported-setup error; see
`Documentation/LogicReview.md`.
