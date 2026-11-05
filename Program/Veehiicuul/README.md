# Veehiicuul

Veehiicuul is a racing game seen from above: drive a car around a track as fast as you dare
without touching the road's edges. Like every KGames GUI program, it's built from the
[Library](../../Library) crates on Win32 and Direct3D 12, with
[Library/UserInterface](../../Library/UserInterface/README.md)'s main loop, ignores input
without focus, and supports the two window setups in
[Library/Application](../../Library/Application/README.md#supported-window-setups).

Everything happens on flat ground, along X and Z: driving, the edges that the car touches, and
the camera's view of them. The track is still, and the car is the only thing that moves. It's
the car of [Library/Meshes](../../Library/Meshes/README.md), painted with
[SimplePaint](../../Library/SimplePaint/README.md), and the ground under it is flat colors. An
orthographic camera looks down at 45 degrees, from 45 degrees around from +Z toward +X.

## Controls

The gamepad and the keyboard each do everything. The gamepad is the one used last, and the
keys and the gamepad's buttons do nothing while the window doesn't have focus.

| Gamepad                 | Keyboard               | Does                                                     |
| ----------------------- | ---------------------- | -------------------------------------------------------- |
| Right stick             | Arrows or WASD         | Pushes the car toward where it points on the screen      |
| Left trigger            | Space                  | Brakes                                                   |
| D-pad left and right    | Q and E                | The previous and the next vehicle                        |
| D-pad down and up       | PageDown and PageUp    | The previous and the next track                          |
| X                       | R                      | Puts the car back at the start                           |
| Menu                    | C                      | Switches between the fixed camera and one that follows   |
|                         |                        | the car                                                  |
| RB and the left stick   | The wheel              | Zooms, up or away from you to zoom in                    |
| Y                       | Home                   | Zooms back to the camera's own height                    |
| View                    | F1                     | Hides or shows the text                                  |
|                         | F11                    | Switches between maximized and borderless fullscreen     |

The text at the bottom left says which vehicle and track it is, how fast the car goes, the
camera, how many times the car crashed, the frame rate, whether a gamepad is connected, and
the controls.

## Driving

There's no steering wheel, mass or grip. The player pushes the car toward where they want it
to go, and it speeds up that way, at its vehicle's rates, in units per second per second:
`Forward` for the part of the push ahead of it, `Reverse` for the part behind it, and `Left`
and `Right` for the parts to its sides. It always faces the way it moves, so it turns by
speeding up to a side. A push along the screen goes through the camera to the ground, and
then to the car's own directions.

- **Dead zones.** They apply to the push along the car's own directions: sideways and forward
  and back, so that a push that's nearly ahead is exactly ahead. A part of the push inside an
  inner dead zone is 0, one past the outer is full, and one between them is scaled to cover 0
  to 1. Then the push is no longer than 1. They're in the [settings](#settings).
- **The brake** slows the car along a straight line, at its vehicle's `Reverse` rate, times
  how far the trigger is pulled. While the player brakes, the push does nothing, so the car
  doesn't turn. The brake stops the car, and never moves it backward.
- **Crashes.** When the car touches an edge of the road, which is the edge of an island too,
  it goes back to the start, at rest. So does switching to another vehicle or track, or `X`.
  Then the controls don't move it for 0.35 seconds, so that the player can let go first.
- **What touches.** The car is the rectangle around its mesh seen from above, 3 units wide and
  about 6 long, 0.165 shorter at the front. It touches an edge when a side of it meets one.
  Only where the car is in each frame counts, so a car that's fast enough, or a frame that's
  slow enough, can carry it past an edge.
- **Time.** Each frame moves the car on by the time since the frame before, so it moves the
  same at any frame rate, but by 0.1 seconds at most, for a frame that comes late, like after
  the window was minimized.

## Vehicles

`Vehicles.json` has the vehicles, in the order that the controls go through them. Each is the
car, in colors of its own, and drives the same way on every track, so that a vehicle that
looks different drives differently, and one that drives differently looks different.

| Vehicle | Forward | Reverse | Left | Right |
| ------- | ------- | ------- | ---- | ----- |
| Green   | 10      | 100     | 40   | 40    |
| Blue    | 35      | 70      | 35   | 35    |
| Red     | 70      | 140     | 70   | 70    |
| Yellow  | 140     | 280     | 140  | 140   |
| Cyan    | 280     | 560     | 280  | 280   |
| Magenta | 1000    | 2000    | 200  | 200   |

Each vehicle has its `Name`, its `Acceleration`, and the SimplePaint materials of its `Body`
and its `Cabin`, in the format of [Library/PaintSettings](../../Library/PaintSettings/README.md).
Every vehicle shares the `Axles`, `Headlights` and `Wheels` at the file's top. Tune a material
in [SimplePaintDemo](../SimplePaintDemo/README.md), and copy it from its `Paint.json`.

## Tracks

Each file in `Tracks` is a track, named by the file, in the order of their names. Ribeye is the
only one so far: a road with two islands, 800 points around. A track is the outlines of its
road and its islands, and the game makes the rest from them: the road, the islands' grass, red
and white curbs on the grass beside every edge, and the black and white start line. Points
are `[X, Z]`, in units, and headings are in degrees from +Z toward +X.

| Member         | What it is                                                                     |
| -------------- | ------------------------------------------------------------------------------ |
| `Road`         | The road's outer edge, as a list of points, which closes from its last point   |
|                | back to its first                                                              |
| `Islands`      | The edges of the islands of grass in the road, each a list of points           |
| `Start`        | Where the car starts, its `Position`, and which way it faces, its `Heading`    |
| `StartLine`    | The middle of the start line, its `Position`, and its `Width` across the road  |
|                | and its `Depth` along it. It's at a right angle to the start's heading         |
| `Camera`       | Where the fixed camera looks, its `Center`, and its view's `Height`, in units  |
| `FollowHeight` | The height of the view of the camera that follows the car                      |
| `Colors`       | The `Grass`, the `Road`, and the two `Curbs`, as sRGB, like `"#1CDA61"`        |
| `CurbWidth`    | How wide the curbs are                                                         |
| `CurbLength`   | About how long each curb is. Each edge has an even number, so that they        |
|                | alternate all the way around                                                   |

## Settings

`Settings.json` has the settings that every program has, then the game's own. The game writes
it with the defaults if it's missing, and the text says what's wrong with it.

| Setting     | Value                                                                           |
| ----------- | ------------------------------------------------------------------------------- |
| `Track`     | The track to start on, `"Ribeye"`                                               |
| `Vehicle`   | The vehicle to start with, `"Yellow"`                                           |
| `Camera`    | `"Fixed"`, which looks at the track's place, or `"Follow"`, which follows the   |
|             | car                                                                             |
| `DeadZones` | The push's `Sideways` and `ForwardAndBack` dead zones, each `[inner, outer]`,   |
|             | from 0 to 1. `[0.1, 0.95]` and `[0.05, 0.95]` by default. A gamepad whose       |
|             | sticks drift at rest needs wider inner dead zones                               |

## Build and run

```powershell
cargo run --release
```

Or run `HumanRun.cmd`, which you can also double-click in File Explorer. It opens the game on
the primary display. `--display` and `--no-activate` work as in every program, and
[RunInBackground](../RunInBackground/README.md) runs it for coding agents.

## Testing

```powershell
cargo test -p veehiicuul_tests
Program\RunInBackground\RunInBackground.ps1 Veehiicuul --all-scripts
```

The tests in `Test/Veehiicuul` check, without a window: driving, the dead zones and braking;
segments that meet, the car's footprint, and a car against edges; reading Ribeye, and what's
wrong with other tracks; cutting outlines into triangles, and Ribeye's ground; the vehicles and
the settings; and the game through its frames, with Library/UserInterface's `Headless`:
driving with the stick and the keys, crashing, the vehicles, the tracks, the camera, the text,
and a window without focus.

The scripts in `Test/Veehiicuul/Scripts` run the real game on a fixed clock, as
[Library/Application](../../Library/Application/README.md#scripts) describes. Their `Gamepad`
steps put the gamepad's sticks, triggers and buttons where they need them, and `Hold` steps
hold keys down. The state that scripts save and check has the `Track`, the `Vehicle`, the
car's `Position`, `Heading`, `Velocity` and `Speed`, whether it's `Paused` after going back to
the start, the `Camera`, its `ViewHeight`, the `Crashes`, and whether the text is shown, as
`TextShown`.

What only a person can check: real gamepads and keyboards, how driving feels, and input
latency.

## Layout

| Path                  | Contents                                                                    |
| --------------------- | --------------------------------------------------------------------------- |
| `Source/Main.rs`      | Runs the game with Library/UserInterface's main loop                        |
| `Source/Library.rs`   | The game's parts, as a library that `Test/Veehiicuul` tests                 |
| `Source/Program.rs`   | The game as a `Program`: each frame's controls, and the text                |
| `Source/Game.rs`      | The tracks, the vehicles, the car, the camera, and what the controls do     |
| `Source/Controls.rs`  | What the gamepad and the keys do                                            |
| `Source/Driving.rs`   | How a car moves, its dead zones, and its brake                              |
| `Source/Collision.rs` | The car's footprint, and whether it touches an edge                         |
| `Source/Track.rs`     | Reads a track's file                                                        |
| `Source/Ground.rs`    | Makes a track's ground from its outlines                                    |
| `Source/Vehicles.rs`  | Reads `Vehicles.json`                                                       |
| `Source/Camera.rs`    | The camera's angle, view and zoom                                           |
| `Source/Settings.rs`  | The game's own settings, and their defaults                                 |
| `Source/Scene.rs`     | Draws the ground and the car                                                |
| `Shaders/Ground.hlsl` | Draws the ground in flat colors, with SimplePaint's root signature          |
| `Build.rs`            | Compiles SimplePaint's shaders for the car's five materials, and Ground's   |
| `Tracks`              | The tracks                                                                  |
| `Vehicles.json`       | The vehicles                                                                |
| `Settings.json`       | The settings, which the game writes if it's missing                         |
| `HumanRun.ps1`        | Builds the game and runs it on the primary display, for people to play      |
| `HumanRun.cmd`        | Runs `HumanRun.ps1`, for double-clicking in File Explorer                   |

# Design doc

Question: Figure out how to maintain a consistent representation for visual objects (car, cone, house, etc.) throughout the workflow

Question: Should different underlying stats always be visually distinguisable?
- Vice versa: Should different looking vehicles always imply different underlying stats?
- In previous versions of Veehiicuul and ZoomTracks, a vehicle had different stats depending on the
  track to make it easy to experiment with different vehicle physics without affecting the
  experience in all tracks
  - This shouldn't be done this version of Veehiicuul, meaning a single visually distinct vehicle
    should globally correspond to a consistent set of stats

## Simple paint

The simple paint shader needs to be available in all parts of the flow, which currently is Blender
and Veehiicuul which is written in Rust and DX12.

The canonical implementation of simple paint for Blender will be a Blender Python script that
generates the shader node graph for simple paint.

## Blender

- Model mesh in Blender
- Types of meshes
  - Single mesh object:
    - This is when you export a single mesh object in blender
    - To assign different colors to different faces, you must use blender material slots
    - In blender, it seems like you can have zero material slots, so I'll need to test glb this exports to
    - This can be made by joining multiple separate mesh objects together
      - Alternatively, you can keep the objects separate but under a single root
  - Single root, multiple objects
    - Can contain empty or mesh object types
  - In any case, the mesh must have a single "pivot point" to make it easy to work with on the Veehiicuul side
    - This means if you have multiple mesh objects you want to export, you need to parent them to an empty first
- When an object gets in the veehiicuul program boundary, edits should be limited to this:
  - Transform: Position, scale, rotation
  - Setting colors
  - Veehiicuul shouldn't support general mesh editing like adjusting vertex positions, editing mesh object hierarchy, or modeling new meshes
- To make adjustments easy, you should be able to color in a mesh on the Blender side or the Veehiicuul side

## Veehiicuul

- Hexel terrain editor
- Track builder
- Vehicle footprint creator
- Player

## Overall creation flow

- Create mesh objects for vehicles and decorations in Blender
- Color in objects in any part of the flow
- Create curves to represent the track, which automatically generates barriers
- Paint a hex grid that will be the terrain
- Arrange the hex grid, track, decorations, and vehicles in a scene

