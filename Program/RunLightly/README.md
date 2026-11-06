# RunLightly

Runs a command lightly, so that it doesn't make a game that you're playing stutter: builds, tests,
and RunInBackground, which runs through it. Coding agents run their builds and tests with it (see
the [AGENTS.md](../../AGENTS.md) at the root).

```powershell
Program\RunLightly\RunLightly.ps1 cargo test --workspace
Program\RunLightly\RunLightly.ps1 cargo build --release -p UiShowcase
```

It puts itself in a Windows job object before it runs the command, and the job's limits hold the
command and every process that it starts, like Cargo's `rustc` and linker, which can't escape them:

| Limit    | What it does                                                    |
| -------- | --------------------------------------------------------------- |
| Affinity | Only the CPU's efficiency cores                                 |
| Priority | Idle: anything else that wants a core gets it first             |
| CPU time | At most 33% of the whole CPU's, or `KGAMES_LIGHT_CPU_PERCENT`'s |

The efficiency cores leave the performance cores, where a game's threads run, to the game. On a
CPU whose cores are all alike, it takes all of them but a quarter. The cap on CPU time also leaves
a laptop's power to the game's cores.

On an Intel Core Ultra 9 275HX, with 8 performance cores and 16 efficiency cores, a command
gets the 16 efficiency cores, and Cargo runs 16 jobs. Windows numbers that CPU's cores with
the two kinds mixed: its performance cores are logical processors 0, 1, 10 to 13, 22 and 23.
RunLightly asks Windows each core's efficiency class instead of assuming an order.

It turns off .NET's and MSBuild's build servers, which would otherwise run outside the job and
outlive it. It exits with the command's exit code. Commands run slower than they would directly,
which is what a game that's running needs.

What it can't hold:

- **The GPU.** A job has no GPU limits. Programs and tests that draw still draw on whichever GPU
  they pick.
- **Windows Defender**, which scans every file that a build writes, in its own process. A Dev
  Drive, or an exclusion for the repository's `target` folder, stops that, and only you can set
  either.
- **Reading and writing files**, which keep their normal priority.

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
