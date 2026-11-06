# RunLightly

Runs a command lightly, so that it doesn't make a game that you're playing stutter: builds, tests,
and RunInBackground, which runs through it. Coding agents run their builds and tests with it (see
the [AGENTS.md](../../AGENTS.md) at the root).

```powershell
Program\RunLightly\RunLightly.ps1 cargo test --workspace
Program\RunLightly\RunLightly.ps1 cargo build --release -p UiShowcase
```

It runs the command in a PowerShell process of its own (`RunLightlyInJob.ps1`), which puts itself
in a Windows job object first, so that the shell it's run from stays as it is. The job's limits hold
the command and every process that it starts, like Cargo's `rustc` and linker, which can't escape
them:

| Limit    | What it does                                                                 |
| -------- | ---------------------------------------------------------------------------- |
| Affinity | Only the CPU's efficiency cores                                              |
| Priority | Idle: anything else that wants a core gets it first                          |
| CPU time | At most 33% of the whole CPU's, or `KGAMES_LIGHT_CPU_PERCENT`'s              |
| Lifetime | The job's processes are killed when RunLightly ends, however that happens,   |
|          | unless one holds a handle to the job, by the name in `KGAMES_LIGHT_JOB`      |

The efficiency cores leave the performance cores, where a game's threads run, to the game. On a
CPU whose cores are all alike, it takes all of them but a quarter. The cap on CPU time also leaves
a laptop's power to the game's cores. The lifetime limit means that a run that's stopped partway,
like by a coding agent's time limit, leaves no build running unseen. A program that
[RunInBackground](../RunInBackground/README.md) keeps open holds a handle to the job, so it keeps
running, with the limits, after RunInBackground ends.

Jobs can't hold the rest, so RunLightly sets it itself:

| Setting         | What it does                                                                     |
| --------------- | -------------------------------------------------------------------------------- |
| I/O priority    | Low, on itself, which its processes inherit: the game's reads and writes go first |
| Memory priority | Low, the same way: when memory is short, the build's pages go before the game's   |
| Efficiency mode | On every process in the job, a few times a second, as processes don't inherit it: |
|                 | the efficiency cores run them at efficient clocks, and the chip's power budget     |
|                 | stays with the game's cores (Task Manager shows the leaf)                          |
| Membership      | The command's process is made suspended, put in the job, and then let run, and a  |
|                 | process whose parent is in the job but that Windows left out, like what a .NET    |
|                 | process starts here, is put in within a quarter second, unless it's in a job of   |
|                 | its own by then, as rustup's cargo is: so run cargo as the command, not from a    |
|                 | script                                                                            |
| GPU priority    | `KGAMES_GPU_PRIORITY=Idle` in the environment, which                              |
|                 | [Library/Graphics](../../Library/Graphics/README.md) reads: every process that    |
|                 | draws, like a test or a test run of a program, puts its GPU work after other      |
|                 | processes', like the game's frames                                                |

Cargo runs as many jobs as the cap is worth in cores, and tests run on as many threads
(`CARGO_BUILD_JOBS` and `RUST_TEST_THREADS`), instead of one for each efficiency core, so that the
work is steady rather than bursts that the cap stops, and the other efficiency cores stay free.

On an Intel Core Ultra 9 275HX, with 8 performance cores and 16 efficiency cores, a command
gets the 16 efficiency cores, and Cargo runs 8 jobs, a third of 24. Windows numbers that CPU's
cores with the two kinds mixed: its performance cores are logical processors 0, 1, 10 to 13, 22
and 23. RunLightly asks Windows each core's efficiency class instead of assuming an order.

It turns off .NET's and MSBuild's build servers, which would otherwise run outside the job and
outlive it. It exits with the command's exit code. Commands run slower than they would directly,
which is what a game that's running needs. Run from a command that's already in its job, like
RunInBackground run through it, it just runs the command, which the job holds already.

What it can't hold:

- **GPU time.** The GPU's scheduler puts the job's work after other processes', but a frame that's
  on the GPU runs to its end. The programs' test runs draw small frames, at most one a refresh.
- **Windows Defender**, which scans every file that a build writes, in its own process. A Dev
  Drive, or an exclusion for the repository's `target` folder, stops that, and only you can set
  either.
- **The last level cache and memory bandwidth**, which every core shares.

## Checking a build's effect on a game

[FrameTimes](../FrameTimes/README.md) measures a game's frame times, which shows what a build does
to it. Build FrameTimes once, and go somewhere in the game that draws steadily, to stay in for both
runs. Start a build that has work to do, like the whole workspace into a new target folder, in one
terminal:

```powershell
Program\RunLightly\RunLightly.ps1 cargo build --release --workspace --target-dir target\BuildCheck
```

While it builds, measure the game in another:

```powershell
target\release\FrameTimes.exe AceCombat8 --seconds 60
```

Then measure the game again, in the same place, without the build, and compare the two. The 99th
percentile, the maximum and the stutters show the hitches that the build adds, more than the mean
does. Run FrameTimes directly, not through RunLightly, so that it reads the game's events as they
come, not at idle priority behind the build. Delete `target\BuildCheck` afterwards, so that the
next check builds everything again.
