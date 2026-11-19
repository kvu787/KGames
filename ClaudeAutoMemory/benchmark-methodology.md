---
name: benchmark-methodology
description: "Frame-time benchmarks on this machine shift ~1.5x with gaming load; interleave variants and compare ratios, not absolutes"
metadata:
  node_type: memory
  type: project
  originSessionId: 078024e2-ccf2-4799-98dd-bb98ba890867
  modified: 2026-11-08T02:59:50.364Z
---

UiShowcase's `benchmark_frame_time` (release, via RunLightly on efficiency cores) measured 0.118 ms on an idle machine and 0.176-0.19 ms for the same binary under gaming load - a uniform ~1.5x scale across all phases.

**Why:** RunLightly pins to shared efficiency cores at idle priority, so absolute numbers track the machine's load, while ratios between implementations stay stable across sessions.

**How to apply:** To compare implementations, keep each variant's exe (copy to the scratchpad before switching branches, since the target dir is shared), interleave runs (several rounds of A, B, C), and report minimums and ratios. Run exes by absolute path through RunLightly; `--ignored` filters OUT non-ignored tests. See also [[light-load-while-gaming]].

To tell a toolchain's effect from code changes (done 2026-11-16 after 1.99.1 -> 1.100.0), build the same code with both toolchains (`rustup toolchain install 1.99.1 --profile minimal`, `cargo +1.99.1 build --target-dir <separate>`), copy the exes aside, and run them in 5 interleaved rotating rounds through RunLightly. That put every benchmark within 0.97-1.024, allocation counts identical, where a comparison against single runs from another day had shown JSON parse 23% slower and mesh triangulate/topology 28-36% faster: the outliers were day-to-day noise. Never conclude a regression from a comparison against numbers from another day alone.
