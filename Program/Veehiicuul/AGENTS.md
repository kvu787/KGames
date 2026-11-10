# Veehiicuul

Follow the rules for GUI programs in the [AGENTS.md](../../AGENTS.md) at the root, including the supported window setups and the non-disruptive workflow. Run the game with `Program\RunInBackground\RunInBackground.ps1 Veehiicuul`.

- Keep the game and its editors as the [README](README.md) describes them, and the README up to date with them.
- The design notes at the end of the README, under "Design doc", are the user's. Don't change them unless the user asks.
- Models come from Blender, as the `.glb` files in `Models`. Don't edit them, or bake models into code. The game only moves, turns, scales and paints them.
- A vehicle looks and drives the same in every scene. A vehicle that drives differently gets paint of its own in `Vehicles.json`, instead of a change in one scene.
- A scene is its file in `Scenes`. The terrain, the road and its barriers are made from it as the game runs. Don't bake them into images or meshes.
- The files in this folder are what Save writes, in its format. Edit them with the editors, or keep a hand edit in that format, which `Test/Veehiicuul/Document.cpp` checks.
- Everything that moves the car happens on the ground, along X and Z. Keep 3D for how it looks.
- Every frame draws the whole scene, in `render`, over the whole window, under the interface. The cards float over it, as floating `Card`s, in fixed pixels for the two window setups. What an editor draws over its view, like control points, is a `Canvas` added before its cards, so that they cover it.
- Build the cards from Library/UserInterface's widgets, laid out with its `Form`, and the editors' sliders, color fields and paint controls in `Editing.h`, with the theme's roles and the tokens. Only text fields take keyboard focus, so that Space, the arrows and the keys that drive and pan never go to a control.
- Test changes with the tests and scripts in `Test/Veehiicuul`, as the [README](README.md#testing) says. A script's `Gamepad` and `Hold` steps drive the car on a fixed clock, so a script can check exactly where it is, like `Test/Veehiicuul/Scripts/Drive.json` does, and its presses edit in the editors' views, like `Editors.json`. A scripted run saves in its output folder.
