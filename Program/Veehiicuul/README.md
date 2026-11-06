# Veehiicuul

Veehiicuul is a racing game seen from above: drive a car around a track as fast as you dare
without touching the road's edges. It's also where its levels are made: tabs in the app bar go
between playing and the editors that make the game's scenes and vehicles. Like every KGames GUI
program, it's built from the [Library](../../Library) crates on Win32 and Direct3D 12, with
[Library/UserInterface](../../Library/UserInterface/README.md)'s main loop, widgets and design
system, after Fluent 2, ignores input without focus, and supports the two window setups in
[Library/UserInterface](../../Library/UserInterface/README.md#window-setups).

Everything happens on flat ground, along X and Z: driving, the edges that the car touches, and
the camera's view of them. The car is the only thing that moves. The ground is flat colors, drawn
in order without a depth buffer, each part over the ones before, so nothing on it z-fights: the
scene's background, the terrain's hexels, the road with its islands cut out of it, the barriers,
and the start line. Models, like the car and the decorations, are painted with
[SimplePaint](../../Library/SimplePaint/README.md). An orthographic camera looks down at 45
degrees, from 45 degrees around from +Z toward +X.

## How a level is made

1. **Model** vehicles and decorations in Blender, and export each with File > Export > glTF 2.0,
   as a `.glb` with Y up and modifiers applied, into `Models`.
   [Library/Gltf](../../Library/Gltf/README.md) reads it. A model has one pivot, its origin, which
   sits on the ground, and faces -Y in Blender, which is +Z here. Each of its materials is a
   slot, which SimplePaint paints.
2. **Color** each slot, in Blender or here. A material's custom property `SimplePaint`, in
   [Library/PaintSettings](../../Library/PaintSettings/README.md)' format, is its paint as Blender
   says, or else its base color is. The Scene tab paints over that for the model, everywhere it's
   used, and the Vehicles tab for one vehicle.
3. **Build the track** in the Track tab, from closed curves: the road's outer edge, and the
   edges of islands inside it. The road between them, and the barriers along every edge, follow
   the curves.
4. **Paint the terrain** in the Terrain tab: a grid of hexels under the track.
5. **Make vehicles** in the Vehicles tab: a model, its acceleration, its paint, and its
   footprint, the rectangle that touches the track's edges.
6. **Arrange the scene** in the Scene tab: decorations, which are models, and vehicles parked
   as decorations, the background's color, and the camera.
7. **Play** it, and **save** it, with Save or Ctrl+S, which writes the files that changed.

## Tabs

The app bar at the top of the window has the tabs, and in the editors, "The game's angle", which
shows the view at the game's angle instead of straight down. At its right end are the scene to
play and edit, New scene, Undo and Redo, which Ctrl+Z and Ctrl+Y do too, and Save, which Ctrl+S
does too, in the accent color while there are unsaved changes. The status bar at the bottom has
the shortcuts, what the last save did, and whether there are unsaved changes. The 3D view fills
the window under them, and the cards float over it, each as tall as what's in it: each editor
has a card at the top left, and the Vehicles tab another at the top right, for the paint. Where
the cards don't cover the view, Space and a drag pan it, and the wheel zooms it around the
pointer. A color in a card is a field of its code, like `#1CDA61`, with a swatch of it, and what's
typed in it takes effect with Enter. Those fields and the vehicle's name are the only controls
that take keyboard focus, so that Space, the arrows and the keys that drive always pan and drive.

| Tab      | What it does                                                                      |
| -------- | --------------------------------------------------------------------------------- |
| Play     | Drives the scene's track, with a card over the bottom left corner that says       |
|          | what's going on, and what the gamepad's and the keyboard's controls do. Going to  |
|          | Play puts the car at the start                                                    |
| Scene    | Arranges decorations and parked vehicles: a press picks one, a drag moves it, and |
|          | the card turns and scales it, adds and deletes them, and paints a decoration's    |
|          | model. Sets the background, the view of the camera that follows the car, and the  |
|          | fixed camera, from the editor's view                                              |
| Terrain  | Paints hexels. Paint and Erase use a brush of a radius, Fill paints the hexels    |
|          | joined to the one pressed that are its color, as far as 60 hexels from it, and    |
|          | Pick takes a hexel's color. The palette has up to 62 colors, and the grid's hexel |
|          | size and orientation, with pointy or flat tops, are in the card                   |
| Track    | Edits the track's curves by their control points: Move points moves one, or the   |
|          | car at the start, or its handle, which turns it, Add points adds one to the       |
|          | chosen curve where it's nearest, Delete points deletes one, and Move track moves  |
|          | it all. The card adds and deletes islands, and sets the start's heading, the      |
|          | barriers' width and length, and the colors. It says what's wrong with the track:  |
|          | edges that cross, islands outside the road, and a start off the road              |
| Vehicles | Edits the vehicles: their names, models, acceleration, footprints and paint. Each |
|          | side of the footprint follows a drag from it in the view, and Fit makes it the    |
|          | model's rectangle. A parked vehicle can't be deleted                              |

## Play

The gamepad and the keyboard each do everything. The gamepad is the one used last, and the
keys and the gamepad's buttons do nothing while the window doesn't have focus.

| Gamepad                 | Keyboard               | Does                                                     |
| ----------------------- | ---------------------- | -------------------------------------------------------- |
| Right stick             | Arrows or WASD         | Pushes the car toward where it points on the screen      |
| Left trigger            | Space                  | Brakes                                                   |
| D-pad left and right    | Q and E                | The previous and the next vehicle                        |
| D-pad down and up       | PageDown and PageUp    | The previous and the next scene                          |
| X                       | R                      | Puts the car back at the start                           |
| Menu                    | C                      | Switches between the fixed camera and one that follows   |
|                         |                        | the car                                                  |
| RB and the left stick   | The wheel, on the view | Zooms, up or away from you to zoom in                    |
| Y                       | Home                   | Zooms back to the camera's own height                    |
| View                    | F1                     | Hides or shows the card                                  |
|                         | F11                    | Switches between maximized and borderless fullscreen     |

There's no steering wheel, mass or grip. The player pushes the car toward where they want it
to go, and it speeds up that way, at its vehicle's rates, in units per second per second:
`Forward` for the part of the push ahead of it, `Reverse` for the part behind it, and `Left`
and `Right` for the parts to its sides. It always faces the way it moves.

- **Dead zones.** They apply to the push along the car's own directions, sideways and forward
  and back, so that a push that's nearly ahead is exactly ahead. They're in the
  [settings](#settings).
- **The brake** slows the car along a straight line, at its vehicle's `Reverse` rate, times
  how far the trigger is pulled, without turning it, and never moves it backward.
- **Crashes.** When the car's footprint touches an edge of the road, which is the edge of an
  island too, the car goes back to the start, at rest, and so does switching to another vehicle
  or scene. Then the controls don't move it for 0.35 seconds, so that the player can let go
  first. Only where the car is in each frame counts, so a car that's fast enough can carry it
  past an edge.
- **Time.** Each frame moves the car on by the time since the frame before, by 0.1 seconds at
  most, for a frame that comes late, like after the window was minimized.

## Files

Everything that the editors change is in files in this folder, which Save writes in the same
format that they're read in. Points are `[X, Z]` on the ground, in units, headings are in
degrees from +Z toward +X, and colors are sRGB, like `"#1CDA61"`.

| Path                   | What it has                                                              |
| ---------------------- | ------------------------------------------------------------------------ |
| `Models/<Name>.glb`    | A model, exported from Blender                                           |
| `Models/<Name>.json`   | The model's paint: a SimplePaint material for some of its slots, by name |
| `Vehicles.json`        | The vehicles, in the order that the controls go through them             |
| `Scenes/<Name>.json`   | A scene, which is a level: its terrain, track, decorations, parked       |
|                        | vehicles, background and cameras                                         |
| `Settings.json`        | The settings, which the game writes if it's missing                      |

[Library/Meshes](../../Library/Meshes/README.md) makes its car, which SpinningCarTest draws, from
`Models/SlopeCar.glb` as it builds, by its materials' names.

### Vehicles

Each vehicle is a model with its own `Paint` over the model's, its `Acceleration`, and its
`Footprint`: how far it goes from the model's origin to the vehicle's `Left`, `Right`, `Front`
and `Back`. A vehicle drives the same on every track, so that a vehicle that looks different
drives differently, and one that drives differently looks different.

| Vehicle | Forward | Reverse | Left | Right |
| ------- | ------- | ------- | ---- | ----- |
| Green   | 10      | 100     | 40   | 40    |
| Blue    | 35      | 70      | 35   | 35    |
| Red     | 70      | 140     | 70   | 70    |
| Yellow  | 140     | 280     | 140  | 140   |
| Cyan    | 280     | 560     | 280  | 280   |
| Magenta | 1000    | 2000    | 200  | 200   |

### Scenes

Ribeye is the only scene so far: a road with two islands, a pond in the terrain, the big cone,
and the six vehicles parked in a row.

| Member         | What it is                                                                     |
| -------------- | ------------------------------------------------------------------------------ |
| `Background`   | The color where there's nothing else                                           |
| `Camera`       | Where the fixed camera looks, its `Center`, and its view's `Height`            |
| `FollowHeight` | The height of the view of the camera that follows the car                      |
| `Terrain`      | The hexels' `Size`, from a hexel's middle to its corners, their `Orientation`, |
|                | `PointyTop` or `FlatTop`, the `Palette`, and the `Rows` of hexels from the     |
|                | `Origin`, which is a hexel's axial coordinates: a character a hexel, its       |
|                | palette index, 0 to 9, A to Z, then a to z, or `.` for none                    |
| `Track`        | The control points of the `Road`'s curve and the `Islands`' curves, the car's  |
|                | `Start`, the `Colors` of the road and the two of the barriers, and the         |
|                | barriers' `BarrierWidth` and `BarrierLength`                                   |
| `Decorations`  | Each a `Model`, with its `Position`, `Heading` and `Scale`                     |
| `Vehicles`     | Each a parked `Vehicle`, with its `Position` and `Heading`                     |

A curve is a closed uniform cubic B-spline, like Blender's cyclic NURBS curves with every weight
1: smooth everywhere, near its control points, but not through them. It's sampled 8 times
between control points, and the samples' segments are the edges that the car touches. The start
line goes across the road from edge to edge, 4.25 units ahead of the car's start.

## Settings

`Settings.json` has the settings that every program has (see
[Library/UserInterface](../../Library/UserInterface/README.md#settings-and-the-command-line)), then
the game's own. The game writes it with the defaults if it's missing. A setting that's wrong keeps
its default, and one that names a scene or a vehicle that isn't there starts the game in the first,
and a notice under the app bar says so.

| Setting     | Value                                                                           |
| ----------- | ------------------------------------------------------------------------------- |
| `Scene`     | The scene to start in, `"Ribeye"`                                               |
| `Vehicle`   | The vehicle to start with, `"Yellow"`                                           |
| `Camera`    | `"Fixed"`, which looks at the scene's place, or `"Follow"`, which follows the   |
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

The tests in `Test/Veehiicuul` check, without a window: driving; what touches; curves, tracks'
shapes and what's wrong with them, cutting the road's islands out, and the ground's meshes;
hexels, brushes and fills; the models from Blender, the vehicles, the settings, and that the
files write back as they were; Play through its frames, with Library/UserInterface's `Headless`,
and its card, which fits the page at either height, under notices about the settings; the
editors through their frames, with presses and drags in their real layout: the tabs, that every
editor's controls fit in its cards at either height, that only text fields keep keyboard focus,
color fields, painting hexels, panning and zooming the view, moving, adding and deleting control
points, a track's problems, dragging a footprint's side, renaming a vehicle, picking and moving a
decoration, undo, and saving; and, with the GPU, offscreen, that the 3D view draws the scene
under the interface.

The scripts in `Test/Veehiicuul/Scripts` run the real game on a fixed clock, as
[Library/UserInterface](../../Library/UserInterface/README.md#testing) describes. Their `Gamepad`
steps put the gamepad's sticks, triggers and buttons where they need them, `Hold` steps hold
keys down, and `Scroll` steps turn the wheel. They press the tabs and the editors' controls by
name, like `"Mode/Terrain"`, and the middle of a view, like `"Terrain/View"`, and a point on the
ground by where it is in the window. A scripted run saves in its output folder, not this one.
The state that scripts save and check has the `Mode`, whether there are `Unsaved` changes, the
`Status`, how many changes `Undo` goes back, the `Scene`, the `Vehicle`, the car's `Position`,
`Heading`, `Velocity` and `Speed`, whether it's `Paused`, the `Camera`, its `ViewHeight`, the
`Crashes`, whether Play's card is shown, as `CardShown`, and each editor's, as `SceneEditor`,
`TerrainEditor`, `TrackEditor` and `VehicleEditor`, and the interface's, as `Interface`, where
`Targets` has every control's area, Play's card as `Play/Card`, and the views as `Play/View`,
`Scene/View`, `Terrain/View`, `Track/View` and `Vehicles/View`.

What only a person can check: real gamepads and keyboards, how driving feels, input latency,
and how editing feels with a real mouse.

## Layout

| Path                       | Contents                                                               |
| -------------------------- | ---------------------------------------------------------------------- |
| `Source/Main.rs`           | Runs the game with Library/UserInterface's main loop                   |
| `Source/Library.rs`        | The game's parts, as a library that `Test/Veehiicuul` tests            |
| `Source/Program.rs`        | The game as a `Program`: the bars, undo, saving, and Play's card       |
| `Source/Game.rs`           | The document, the models, the car, the camera, and Play                |
| `Source/Document.rs`       | What the editors change and Save writes                                |
| `Source/Scene.rs`          | Reads and writes a scene                                               |
| `Source/Terrain.rs`        | The hexel terrain, its hexels' places, brushes and fills               |
| `Source/Track.rs`          | A track's curves, start and colors, and its shape and problems         |
| `Source/Curve.rs`          | The closed B-spline curves                                             |
| `Source/Ground.rs`         | Makes the terrain's and the track's meshes                             |
| `Source/Models.rs`         | Reads the models from Blender, and their paint                         |
| `Source/Vehicles.rs`       | Reads and writes `Vehicles.json`                                       |
| `Source/Files.rs`          | Numbers, points and colors in the game's files                         |
| `Source/Controls.rs`       | What the gamepad and the keys do in Play                               |
| `Source/Driving.rs`        | How a car moves, its dead zones, and its brake                         |
| `Source/Collision.rs`      | Footprints, and whether a car touches an edge                          |
| `Source/Camera.rs`         | The game's camera and the editors' views                               |
| `Source/Editing.rs`        | What the editors share: their cards, views, controls, and what they    |
|                            | draw over the view                                                     |
| `Source/SceneEditor.rs`    | The Scene tab                                                          |
| `Source/TerrainEditor.rs`  | The Terrain tab                                                        |
| `Source/TrackEditor.rs`    | The Track tab                                                          |
| `Source/VehicleEditor.rs`  | The Vehicles tab, with the footprint creator                           |
| `Source/Settings.rs`       | The game's own settings in `Settings.json`, and their defaults         |
| `Source/Drawing.rs`        | Draws the ground and the models, under the interface                   |
| `Shaders/Ground.hlsl`      | Draws the ground in flat colors, with SimplePaint's root signature     |
| `Build.rs`                 | Compiles SimplePaint's shaders for 16 slots, and the ground's          |
| `Models`, `Scenes`         | The models and their paint, and the scenes                             |
| `Vehicles.json`            | The vehicles                                                           |
| `Settings.json`            | The settings                                                           |
| `HumanRun.ps1`             | Builds the game and runs it on the primary display, for people to play |
| `HumanRun.cmd`             | Runs `HumanRun.ps1`, for double-clicking in File Explorer              |

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

