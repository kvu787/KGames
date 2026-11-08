# Platformer

A 2D platformer built to show what it takes to run a game on events without
anything landing a frame late: every occurrence - a coin touched, a spring
landed on, a death, the goal - is an `Event`, and each one's consumers run,
and whatever they publish runs too, inside the frame that published it. The
Events card shows the log as it happens, newest first, on the game's clock.

## The game

Run with the arrows (or A/D, the left stick or the d-pad), jump with Space
(or W, up, or the gamepad's A) - held jumps higher, a press just before a
landing jumps on the landing, and a press just after a ledge still jumps.
Collect every coin to open the door, and reach the flag behind it. Springs
bounce, spikes and falling out kill, checkpoints move the respawn, and a run
has 3 lives. `R` starts the run over; finishing a level unlocks the next,
and PageUp and PageDown go between unlocked levels.

## The event graph

The simulation runs in fixed steps of 1/240 s on its own clock, so a run is
the same at any frame rate. Each step moves the platforms and the player,
then publishes what the movement did; the queue then drains to quiescence
before the next step, so chains like this resolve inside one frame:

```text
CoinCollected (the last one)
  -> AllCoinsCollected
       -> DoorOpened                        the same frame's movement can now
GoalReached                                 pass the door and reach the goal
  -> RunFinished
       -> RecordSet        -> SaveProgress ---+
       -> LevelUnlocked    -> SaveProgress ---+-> one write -> ProgressSaved
       (confetti, banner)                            (save light)
```

Two consumers can ask for a save in one cascade; the first `SaveProgress`
writes `Save.json` and the second finds the progress clean, so one cascade
saves once. A cascade that doesn't settle within 1,000 events in a frame is
a bug in the graph, and stops the program instead of hiding.

An event carries the facts of its moment - which coin, how many seconds -
and its consumers read everything else from the game as it is when they run,
never from a snapshot. `Test/Platformer/Events.rs` pins the one-frame law:
its deepest test collects the last coin, opens the door, reaches the goal,
sets the record, writes the file and flashes the save light in one
`Game::update`, and asserts all of it, the file on disk included.

## Save.json

The progress - unlocked levels and each level's best time - lives in
`Save.json` in the data folder: this folder in a normal run, or a script's
output folder, so scripted runs and tests leave the player's progress alone.
The file isn't checked in; delete it for a fresh start.

## Layout

| Path                  | Contents                                                        |
| --------------------- | --------------------------------------------------------------- |
| `Source/Level.rs`     | The levels: zones, coins, spikes, springs, doors, patrols       |
| `Source/Physics.rs`   | Fixed-step movement: run, gravity, jump buffer, coyote, carry   |
| `Source/Events.rs`    | Every occurrence, as data, with the cascade cap                 |
| `Source/Game.rs`      | The simulation: steps, detections, and the drained event graph  |
| `Source/Progress.rs`  | Save.json: unlocked levels and best times                       |
| `Source/Controls.rs`  | The frame's input, from the keyboard and the gamepad            |
| `Source/Program.rs`   | The Program: one Tick message a frame, and the app bar's two    |
| `Source/Cards.rs`     | The view: the world's canvas, the Run and Events cards, banner  |

## Testing

`Program\RunLightly\RunLightly.ps1 cargo test -p platformer_tests` runs the
tests: the physics (determinism at 60 against 240 Hz, buffer, coyote, carry),
the event graph's one-frame laws, the progress file, and the frames through
`Headless`. `Program\RunInBackground\RunInBackground.ps1 Platformer
--all-scripts` runs the input scripts in `Test/Platformer/Scripts` in a real
window.
