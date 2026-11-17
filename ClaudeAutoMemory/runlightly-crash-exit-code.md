---
name: runlightly-crash-exit-code
description: "RunLightly reported a test exe's access violation (0xC0000005) as exit code 5; run the exe directly, with a timeout, to see the real code"
metadata:
  node_type: memory
  type: reference
  originSessionId: 4abfa008-dd80-4838-ae26-c3d2d644a5ac
  modified: 2026-11-09T11:20:57.876Z
---

On 2026-11-09, a test binary that crashed with an access violation (0xC0000005, -1073741819)
showed up through `Program\RunLightly\RunLightly.ps1` as "Exit code 5", with its test output cut
short. Running the same `target\debug\deps\<Test>-<hash>.exe` directly showed the real code.
A flaky test that only fails in parallel also hung once, so repeated runs need a per-run timeout
(Start-Process plus WaitForExit, killing on timeout) instead of a plain loop.

Related: [[runlightly-exe-arguments]] (run built exes by absolute path through RunLightly).
