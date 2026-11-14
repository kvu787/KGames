# SimplePaintDemo

Follow the rules for GUI programs in the [AGENTS.md](../../AGENTS.md) at the root, including the supported window setup and the non-disruptive workflow. Run the demo with `Program\RunInBackground\RunInBackground.ps1 SimplePaintDemo`.

- `src/cards.rs` has the layout's constants. The preview and the GPU constants card get what's left of the window's height. The sphere's image in `src/preview.rs` is as big as the preview's view, so change the two together.
- Build the cards from Library/UserInterface's widgets, theme roles and tokens, as [its README](../../Library/UserInterface/README.md) says. What it lacks, build as a `Canvas` or a widget of the demo's own, or add it to the library.
- Test input handling with the tests and scripts in `Test/SimplePaintDemo`, as the [README](README.md#testing) says. The state that scripts save has where each slider, row and button is, as `Interface.Targets`. Point at them by name, like `Light Point` or `Save`.
- To see a material other than the default, write a script that sets it with the sliders, like `Test/SimplePaintDemo/Scripts/Material.json`. Or write it to `Paint.json` in the demo's folder, in the format of [Library/PaintSettings](../../Library/PaintSettings/README.md), run the demo without a script, and delete the file afterward unless the user wants it.
- Keep SimplePaint's math in [Library/SimplePaint](../../Library/SimplePaint/README.md).
