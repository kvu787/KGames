---
name: worktree-lfs-pointers
description: "A fresh git worktree of KGames has LFS pointer files for the .glb models, so Gltf tests fail until `git lfs checkout` runs"
metadata:
  node_type: memory
  type: project
  originSessionId: 8788b840-3f3e-4fdb-aab6-6edaeeb5828c
  modified: 2026-11-06T12:47:03.191Z
---

In a freshly made git worktree of KGames (the app makes one per session under `.claude/worktrees`), the Git LFS files
(`Program/veehiicuul/Models/*.glb`, `Blender/Terrain.blend`) are 130-byte pointer files, not the models.
`cargo test` then fails in `gltf-tests` (Test/gltf/tests/models.rs, "reads_the_car_that_blender_exported") with a panic at the
`unwrap_or_else` that loads the model. Seen on 2026-11-05.
It also breaks builds: `Library/meshes`' build script refuses the pointer file ("SlopeCar.glb is a Git LFS pointer file"),
so every program that depends on meshes fails to build, SimplePaintDemo included. On 2026-11-06 the user's own
`HumanRun.cmd` failed this way in a session's worktree.

**Why:** the worktree checkout doesn't run the LFS smudge filter, but the objects are already in the shared `.git/lfs` store.

**How to apply:** run `git lfs checkout` in the worktree before building or testing anything there, and first when a build
or Gltf test fails (it's instant and needs no network).
Also, `cargo test --workspace` stops at the first failing test binary; use `--no-fail-fast` to see the rest.
Related: [[light-load-while-gaming]].
