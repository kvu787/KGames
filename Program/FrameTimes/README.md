# FrameTimes

Measures another process's frame times, like a game's, for checking what something running next to
it does to it, like a build, as
[RunLightly](../RunLightly/README.md#checking-a-builds-effect-on-a-game) describes. It's a
console program, without a window: it reads the process's presents with
[Library/PresentTiming](../../Library/PresentTiming/README.md) for a while, and prints the
statistics of the times between its frames on the screen.

```powershell
Program\RunLightly\RunLightly.ps1 cargo build --release --bin FrameTimes
target\release\FrameTimes.exe AceCombat8
target\release\FrameTimes.exe AceCombat8.exe --seconds 60
target\release\FrameTimes.exe 1234
```

Name the process by its exe's name, without regard to case and with or without `.exe`, or by its
ID. If more than one process has the name, FrameTimes lists their IDs, to name one by. It measures
for 10 seconds, or as many as `--seconds` says, and then prints something like:

```text
Measuring AceCombat8.exe, process 1234, for 10 seconds.

  Frames           1199
  Frames a second  119.9
  Mean             8.34 ms
  Median           8.33 ms
  99th percentile  10.12 ms
  Maximum          25.03 ms
  Stutters         3, frames over twice the median (16.67 ms)
  Present mode     Independent flip
```

| Line            | What it is                                                                       |
| --------------- | -------------------------------------------------------------------------------- |
| Frames          | How many frame times there are: one fewer than the presents that reached the     |
|                 | screen                                                                           |
| Frames a second | The frames over the time from the first one on the screen to the last            |
| Mean            | The frame times' mean                                                            |
| Median          | The middle frame time, or halfway between the middle two                         |
| 99th percentile | The nearest rank's: the shortest frame time that 99% of them are at or under     |
| Maximum         | The longest frame time                                                           |
| Stutters        | How many frames took more than twice the median, which shows as a hitch          |
| Present mode    | How most of the presents reached the screen, one of PresentTiming's present      |
|                 | modes, like a composed flip, or a legacy flip in fullscreen exclusive mode       |

A frame time is the time from one frame on the screen to the next: the difference between the
screen times of two presents in a row that reached the screen, which PresentTiming gives on the
performance counter, the same as PresentMon's. A present that never reached the screen, like one
that a newer one replaced before the display took it, doesn't count. So a game that presents
faster than the display refreshes, without tearing, has the display's frame times, not its own.

The mean, the median and the frames a second say how fast the game runs. The 99th percentile, the
maximum and the stutters say how smoothly, and they're what something running next to it changes
first.

## Privileges

Reading the presents takes an ETW session, which needs the **Performance Log Users** group, or an
elevated administrator. Without either, FrameTimes says how to join the group, as
[Library/PresentTiming](../../Library/PresentTiming/README.md#privileges) does. A FrameTimes that's
stopped with `Ctrl+C` leaves its session running, until the next FrameTimes, or the next program
that reads its presents, stops it.

## Testing

```powershell
cargo test -p frame_times_tests
```

The tests in `Test/FrameTimes` give the statistics made-up presents, some of which never reached
the screen, and none at all, and check the command line and finding a process. They don't start an
ETW session.

## Layout

| Path                | Contents                                                                  |
| ------------------- | ------------------------------------------------------------------------- |
| `Source/Main.rs`    | Reads the command line, finds the process, and reads its presents for a   |
|                     | while                                                                     |
| `Source/Library.rs` | The statistics, as functions of the presents, the command line, finding a |
|                     | process, and the report, as a library that the tests test                 |
