# Platformer

Follow the rules for GUI programs in the [AGENTS.md](../../AGENTS.md) at the root, including the supported window setups and the non-disruptive workflow. Run the game with `Program\RunInBackground\RunInBackground.ps1 Platformer`.

- Keep the game and its event graph as the [README](README.md) describes them, and the README up to date with them.
- Everything that happens in the game is an `Event` in `src/events.rs`, consumed in `Game::apply`. A consumer may publish more events; the queue drains to quiescence inside the frame, so no consequence may wait for a later frame, and a cascade that doesn't settle stops the program. An event carries the facts of its moment; consumers read everything else from the game as it is when they run, never from a snapshot.
- Each frame moves the game by its own time, the time since the frame before, at most `LONGEST_FRAME`, in one step (`src/physics.rs`), never in fixed steps; `Test/Platformer/tests/physics.rs` checks it at 100 frames a second and at uneven ones. The game's clock, `Game::simulated`, is the frames' times added up: time everything in the game on it.
- Effects, like the confetti and the save light, are state spawned by consumers and drawn by the view, timed on the game's clock. The banner is run state. Don't animate interface state; the design system's rule stands.
- The world draws through one `Canvas` from data the view computes; the world's own colors are the level's, and everything else colors with the theme's roles and the tokens.
- `Save.json` holds the progress, written only through the `SaveProgress` event, whose first consumer in a cascade writes once. Scripted runs save in their output folder.
- Test changes with the tests and scripts in `Test/Platformer`, as the README says: events and physics directly on `Game`, frames through `Headless`, and the scripts with `--all-scripts`. A test of a cause asserts its whole consequence graph in the frame that caused it.
