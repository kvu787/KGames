---
name: prefer-official-declarative-ways
description: "User prefers vendor-documented ways (Microsoft's GameInput.lib loader) and declarative dependency manifests (NuGet PackageReference + lock file) over custom code and imperative download scripts"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 3b11089e-6029-41b2-acc0-3fa686bfd3c5
  modified: 2026-11-09T23:37:54.847Z
---

On 2026-11-09 the user had KGames rewritten to load GameInput "the proper Microsoft way": declare Microsoft.GameInput in a NuGet manifest (`Library/GameInput/NuGetPackages.csproj` + `packages.lock.json`), restore it with dotnet, and call `GameInputInitialize` from the package's `GameInput.lib`, replacing KGames' own hardened loader (System32-only, HID from System32, held-open check). They chose this after hearing the tradeoffs (sideloaded copy wins a tie, imports searched beside the exe first).

**Why:** they value the declarative style most software uses ("saying what a project depends on") over imperative download scripts, and the vendor's documented path over custom machinery that later sessions must keep re-justifying.

**How to apply:** when adding a dependency or integrating a vendor SDK, propose the documented, declarative route first (manifest + lock + the vendor's tool), and present extra hardening as an optional follow-up rather than building it in. Don't re-propose the old custom GameInput loader.

Related: [[nuget-package-download]], [[gameinput-entry-points]]

On 2026-11-11 they also had `NuGetPackages.proj` renamed to `.csproj`, though it has no C#: a `.csproj` is what developers and tooling know best, so the familiar convention beats the technically exact one. Prefer the conventional file type or layout over a more "correct" but rarer one.
