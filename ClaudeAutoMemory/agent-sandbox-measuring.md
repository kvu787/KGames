---
name: agent-sandbox-measuring
description: "PresentMon/FrameTimes see no presents while the PC is locked; the agent's shell is not sandboxed for ETW; measure with cycle counts and GPU counters"
metadata:
  node_type: memory
  type: reference
  originSessionId: c7985cf0-97df-495a-9846-8c7c2f20afeb
  modified: 2026-11-11T01:48:13.040Z
---

On 2026-11-08, PresentMon 2.6 wrote no CSV at all and FrameTimes (`frame-times.exe`) reported 0 presents, even
for dwm, from the agent's shell. It was NOT a sandbox: the shell runs as the user at medium
integrity, in Performance Log Users, and `logman` traced Microsoft-Windows-DXGI and DxgKrnl
events fine, to a file and real-time (`tracerpt -rt`). The PC was locked (LogonUI running), so no
frame reached the display, and both tools only report a present once its fate is known: the
likely cause, not yet confirmed by a run while unlocked. Check `Get-Process LogonUI` first.

What works whether or not it's locked, for a kept-open program (RunInBackground --keep-open):
- `QueryProcessCycleTime` / `QueryThreadCycleTime` via Add-Type P/Invoke. Use cycles, not
  `TotalProcessorTime`: Windows charges CPU time at 15.6 ms ticks, so a program doing ~0.1 ms a
  frame can read as 0 ms over seconds.
- `Get-Counter "\GPU Engine(pid_<id>_*)\Running Time"` (100 ns units) and
  `"\GPU Process Memory(pid_<id>*)\Dedicated Usage"`.
- A library program's frame rate is on its PerfBar in the RunInBackground screenshot; crop and
  enlarge the top 17 px with System.Drawing to read it.

Confirmed 2026-11-10 with the PC unlocked: PresentMon 2.6 records fine from the agent's shell
(`--process_id <id> --timed 8 --terminate_after_timed --qpc_time --output_file <csv>`). Its limits:
- It records at most 2,048 presents a second, so uncapped (`--no-vsync`) frame rates read low;
  use the program's own PerfBar count.
- Unelevated, `--process_name` misses a process that lives under about 2 s (no CSV at all); keep
  the program open 3 s, and use `--timed`, not `--terminate_existing_session`.
- `MsAllInputToPhotonLatency` stays NA for these programs.
Launching a GUI program with plain `Start-Process` (its "show normal" command) gives a window that
isn't maximized, so the library shows its unsupported-setup error; use `Start-Process -NoNewWindow`
(as HumanRun does) or `[Diagnostics.Process]::Start` with `UseShellExecute = $false`.

`Program/BareSceneTest/Measure.ps1` does all of this; see [[light-load-while-gaming]].
