# MeshModeler

A polygon mesh modeler, after Blender's edit mode: select vertices, edges or faces, move,
rotate and scale them, extrude, inset, cut loops, fill, merge and delete, crease edges, and see
the mesh subdivided by Catmull-Clark. Faces are colored with paints, which the view shades
with [SimplePaint](../../Library/SimplePaint/README.md) or with flat lighting. The mesh and its
edits are [Library/PolygonMesh](../../Library/PolygonMesh/README.md)'s.

Like every KGames GUI program, it's built on
[Library/UserInterface](../../Library/UserInterface/README.md), after Fluent 2, in its light
theme, ignores input without focus, and supports the two
[window setups](../../Library/UserInterface/README.md#window-setups).

## What it shows

- **The app bar's menus**: File, Edit, Mesh, Paint and View, with every command and its
  shortcut.
- **Select**: the selection mode, the tool, and buttons that select all, none, or the inverse.
- **Edit**: a button for each edit, which is off while the selection can't take it, and the
  sharpness that Crease gives edges.
- **Add**: a primitive, which is added at the origin in the paint picked, and selected.
- **Model**: what the mesh has and what's selected, and what Undo and Redo would do.
- **View**: the mesh, orthographic, over a grid with the X axis in red and Z in blue, with its
  cage's edges, the selection in the accent color, creased edges in pink, and what's under the
  pointer. Subdivided, the view shows the smooth surface inside the cage.
- **Shading**: SimplePaint or flat, the levels of subdivision, smooth shading, X-ray and the
  grid.
- **Paints**: up to 16 paints, with how many faces each has. The one picked is what the Paint
  tool and new primitives use, and its color, name and SimplePaint settings are under the list.

## Editing

| Input                  | Action                                                                        |
| ---------------------- | ----------------------------------------------------------------------------- |
| Click                  | Selects the vertex, edge or face under the pointer, or nothing                |
| `Shift`+click          | Adds it to the selection, or takes it away if it's selected                   |
| `Alt`+click            | Selects the edge loop through the nearest edge, or the face loop in face mode |
| Drag                   | Selects what's in the box: `Shift` adds to the selection, `Ctrl` takes away   |
| Middle drag            | Orbits around the target; with `Shift`, pans                                  |
| Wheel                  | Zooms at the pointer                                                          |
| Right-click            | Opens a menu of edits                                                         |
| `1`, `2`, `3`          | Vertex, edge or face mode, which converts the selection                       |
| `A`, `Alt+A`, `Ctrl+I` | Selects all, none, or the inverse                                             |
| `G`, `R`, `S`          | Moves, rotates or scales the selection with the pointer                       |
| `E`                    | Extrudes the selected faces along their normal, or edges, which move freely   |
| `I`                    | Insets the selected faces as a region, as far as the pointer moves in         |
| `Ctrl+R`               | The Loop cut tool: click an edge to cut the loop across it                    |
| `P`, `Q`               | The Paint tool, which paints the faces it drags over, and the Select tool     |
| `F`                    | Makes a face of the selected vertices, or fills the selected edges' hole      |
| `M`                    | Merges the selected vertices at their center                                  |
| `X`, `Delete`          | Deletes what's selected, in the mode; `Shift+X` deletes only faces            |
| `Ctrl+D`               | Subdivides the selected faces                                                 |
| `Alt+N`, `Shift+N`     | Flips the selected faces, or makes all or the selected face outward           |
| `Shift+E`, `Alt+E`     | Creases the selected edges by the Edit card's sharpness, or clears them       |
| `Ctrl+P`               | Paints the selected faces with the paint picked                               |
| `Ctrl+0` to `Ctrl+4`   | Shows the mesh subdivided by Catmull-Clark that many levels                   |
| `Shift+Z`, `Shift+S`   | Switches between SimplePaint and flat shading, and smooth shading             |
| `Alt+Z`, `Shift+G`     | X-ray, which shows the cage through the mesh, and the grid                    |
| `Home`, `Period`       | Frames the whole mesh, or the selection                                       |
| `Numpad1`, `3`, `7`    | Looks from the front, the right or the top; with `Ctrl`, the other side       |
| `Ctrl+Z`, `Ctrl+Y`     | Undoes and redoes, up to 200 steps                                            |
| `Ctrl+N`, `O`, `S`     | New, Open and Save; `Ctrl+Shift+S` saves as, `Ctrl+E` exports OBJ             |
| `F2`, `F11`            | Dark theme, and borderless fullscreen                                         |

Moving, rotating, scaling, insetting and the move after an extrusion are modal, as in Blender:

| Input              | Action                                                      |
| ------------------ | ----------------------------------------------------------- |
| Pointer            | Moves, turns, scales or insets, from where it started       |
| `X`, `Y`, `Z`      | Limits it to the axis, or frees it again                    |
| `N`                | Limits it to the selection's normal, as an extrusion starts |
| `Ctrl`             | Snaps to tenths, or rotation to 15 degrees, while held      |
| Click, `Enter`     | Keeps it, as one step of the history                        |
| Right-click, `Esc` | Takes it back, with the extrusion that began it             |

Each edit is one step of the history, including a modal edit with what began it, a stroke of
the Paint tool, and a drag of a paint's slider. Errors, like Merge without two vertices, show
in the status bar.

Picking finds the nearest vertex or edge within 12 pixels of the pointer, or the face under it,
that the mesh doesn't hide, unless X-ray is on. A box selects what's in it, hidden or not.
Subdivided, faces are picked on the smooth surface, as the faces of the cage that they come
from.

## Files

**Save** writes the model to `Models/<name>.json` in this folder, as a JSON object with the
mesh, in PolygonMesh's format, and the paints, each a `Name` and its settings in
[Library/PaintSettings](../../Library/PaintSettings/README.md)'s format. **Open** lists what's
there. **Export OBJ** writes `Models/<name>.obj` beside it, with the mesh's positions and faces.
A model that was never saved asks for its name first, and **New**, **Open** and closing the
window ask before changes are lost.

## Build and run

```powershell
cargo run --release
```

Or run `HumanRun.cmd`, which you can also double-click in File Explorer. It opens the modeler
on the primary display. `--display` and `--no-activate` work as in every program, and
[RunInBackground](../RunInBackground/README.md) runs it for coding agents. `Settings.json` has
the settings that every program has; the modeler has none of its own.

## Testing

```powershell
cargo test -p mesh_modeler_tests
Program\RunInBackground\RunInBackground.ps1 MeshModeler --all-scripts
```

The tests in `Test/MeshModeler` drive the modeler through its real layout, which
`user_interface::Headless` builds without a window or a GPU: clicks, Shift and Alt clicks and
boxes in the view, the camera, modal moves, rotations with snapping, extrusions and insets and
their cancelling, loop cuts, the Paint tool, creases, subdivision, the menus, the dialogs,
saving, opening and exporting, and closing with changes. They check that the cards fit both
window heights, and draw frames offscreen with the GPU, with either shader, subdivided and
selected.

The scripts in `Test/MeshModeler/Scripts` run the same kinds of input through the real window,
as [Library/UserInterface](../../Library/UserInterface/README.md#testing) describes. A scripted
run saves models in its output folder, not in the modeler's. The state that scripts save and
check has the model's `Name`, whether it's `Dirty`, the `Mode` and `Tool`, the modal
`Operation` with its `Kind`, `Axis` and `Amount`, what's under the pointer as `Hover`, the
`Mesh` and `Selection` counts, each face's paint as `Mesh.Materials`, the `Paints` and the
`ActivePaint`, the `Shader`, the `Subdivision` and the `SurfaceFaces` it makes, `Undo` and
`Redo`, the open dialog as `Prompt`, and the last `Message`. Point at the view with
`{ "Target": "View", "At": [x, y] }`, in fractions of it: the camera starts the same every
run, so the cube's +Z face is at the view's middle.

## How it draws

Each frame, the modeler builds its cards, and then its scene: when the mesh, its subdivision or
its shading changed, it triangulates the mesh or its subdivided surface again, with normals
smooth across edges up to 30 degrees, and uploads the triangles. The view's card leaves a hole
for the view, where `render` clears the frame to the card's color and draws the mesh with a
depth buffer, through the frame's sRGB view, with SimplePaint's shader or the flat one, for all
16 paints at once. Then it draws the overlay through the frame's plain view: the grid, the cage
and the selection, as quads that the vertex shader sizes in pixels, tested against the mesh's
depth, pulled a little toward the camera, and what shows through the mesh, untested.
SimplePaint only paints orthographic views, so the camera is orthographic, and zooms by how
much of the world the view's height shows.

## Layout

| Path                    | Contents                                                                  |
| ----------------------- | ------------------------------------------------------------------------- |
| `Source/Main.rs`        | Runs the modeler with Library/UserInterface's main loop                   |
| `Source/Library.rs`     | The modeler's parts, as a library that `Test/MeshModeler` tests           |
| `Source/Modeler.rs`     | The modeler as a program: its state, its commands, and the scene it draws |
| `Source/Cards.rs`       | Lays out the app bar, the cards and the dialogs in fixed pixels           |
| `Source/Interaction.rs` | What the keys and the pointer do in the view, and modal edits             |
| `Source/Editing.rs`     | Selection modes, tools, picking, modal edits and the mesh's edits         |
| `Source/Document.rs`    | The mesh, its paints and selection, the history, and the files            |
| `Source/Paint.rs`       | Paints: a name and SimplePaint's settings                                 |
| `Source/Camera.rs`      | The orthographic camera, and mapping between the world and pixels         |
| `Source/Overlay.rs`     | What's drawn over the mesh: the grid, the cage and the selection          |
| `Source/Viewport.rs`    | Draws the mesh and the overlay into the view with Direct3D 12             |
| `Shaders/Flat.hlsl`     | Flat lighting of the paints' base colors                                  |
| `Shaders/Overlay.hlsl`  | Lines, dots and triangles over the mesh, tested against its depth         |
| `Build.rs`              | Compiles SimplePaint's shaders and the modeler's for 16 paints            |
| `HumanRun.ps1`          | Builds the modeler and runs it on the primary display, for people         |
| `HumanRun.cmd`          | Runs `HumanRun.ps1`, for double-clicking in File Explorer                 |
| `Settings.json`         | The settings that every program has                                       |
| `Models`                | Saved models and exported OBJ files; not in Git                           |
