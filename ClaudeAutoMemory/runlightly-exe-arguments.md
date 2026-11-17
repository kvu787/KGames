---
name: runlightly-exe-arguments
description: "How to pass arguments through RunLightly: `cargo run -- args` loses the `--`, and `-name:value` is split at the colon"
metadata:
  node_type: memory
  type: project
  originSessionId: 8c85cc9b-5754-4a36-b49d-9399b8d3d91e
  modified: 2026-11-17T04:03:57.056Z
---

RunLightly.ps1 swallows a `--` separator, so `RunLightly.ps1 cargo run -p x -- --flag` fails with cargo complaining about `--flag`. To run a built program with arguments, give RunLightly the exe's absolute path: `Program\RunLightly\RunLightly.ps1 "C:\...\target\release\benchmarks.exe" --quick`. Relative paths like `.\target\release\X.exe` aren't found, since RunLightlyInJob runs in another process with its own working directory.

It also splits a `-name:value` argument at the colon, even quoted: `RunLightly.ps1 dotnet msbuild x.csproj -getProperty:Foo` reaches MSBuild as `-getProperty Foo` and fails (MSB1008/MSB1010). Use the slash form, `/getProperty:Foo`, which it passes whole (found 2026-11-11). Tools without a slash form, like 7-Zip's `-oC:\out`, hit the same split: run from the parent folder and give a relative path, `-oout` (found 2026-11-16).

Python: RunLightly can't start the Store/pymanager alias `%LOCALAPPDATA%\Microsoft\WindowsApps\python.exe` ("Access is denied" from CreateProcess suspended), and wrapping it in `pwsh -Command` worked once and then misfired. Give RunLightly the real interpreter, `C:\Users\K\AppData\Local\Python\pythoncore-3.15-64\python.exe -I script.py` (found 2026-11-16).

**Why:** Found on 2026-11-06 while running the Benchmarks program (Program/Benchmarks), which prints its table to stdout and must run lightly like everything else.

**How to apply:** Build with `RunLightly.ps1 cargo build --release -p <crate>` first, then run the exe by absolute path through RunLightly. See [[light-load-while-gaming]].
