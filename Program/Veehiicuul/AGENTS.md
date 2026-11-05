# Veehiicuul

Follow the rules for GUI programs in the [AGENTS.md](../../AGENTS.md) at the root, including the supported window setups and the non-disruptive workflow. Run the game with `Program\RunInBackground\RunInBackground.ps1 Veehiicuul`.

- Keep the game as the [README](README.md) describes it, and the README up to date with it.
- The design notes at the end of the README, under "Design doc", are the user's. Don't change them unless the user asks.
- A vehicle looks and drives the same on every track. A vehicle that drives differently gets colors of its own in `Vehicles.json`, instead of a change on one track.
- A track is its file in `Tracks`: its outlines and a few numbers. `Source/Ground.rs` makes what it looks like from them. Don't bake a track into images or meshes.
- Everything that moves the car happens on the ground, along X and Z. Keep 3D for how it looks.
- Every frame draws the whole scene.
- Test changes with the tests and scripts in `Test/Veehiicuul`, as the [README](README.md#testing) says. A script's `Gamepad` and `Hold` steps drive the car on a fixed clock, so a script can check exactly where it is, like `Test/Veehiicuul/Scripts/Drive.json` does.
