# MeshModeler

Follow the rules for GUI programs in the [AGENTS.md](../../AGENTS.md) at the root, including the supported window setups and the non-disruptive workflow. Run the modeler with `Program\RunInBackground\RunInBackground.ps1 MeshModeler`.

- Keep the mesh's data structure and its edits in [Library/PolygonMesh](../../Library/PolygonMesh/README.md), and test them in `Test/PolygonMesh`. The modeler's `Editing.h` and `Editing.cpp` only pick, select, and run them as steps of the history.
- Every command is in `Commands`, whose table is in `Modeler.cpp`, with its label and shortcuts, which the menus, the buttons' tooltips and the keys all use. Add a command there, as an alternative of `Command` in `Modeler.h`, and run it with `Modeler::run`.
- `Cards.h` and `Cards.cpp` have the layout's constants. The view and the cards at the bottom of the side columns get the 71 pixels that borderless fullscreen adds.
- Build the cards from Library/UserInterface's widgets, theme roles and tokens, as [its README](../../Library/UserInterface/README.md) says. Color the overlay with the theme's roles too.
- Test input handling with the tests and scripts in `Test/MeshModeler`, as the [README](README.md#testing) says. Point at the view with fractions of the `View` target, and at everything else by name.
- Keep SimplePaint's math in [Library/SimplePaint](../../Library/SimplePaint/README.md). `MeshModelerLibrary.vcxproj` compiles its shader, and `Shaders/Flat.hlsl`, for 16 materials, which is `Paint.h`'s `MaximumPaints`, so change them together.
