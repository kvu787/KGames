# SimplePaintDemo

Follow the rules for GUI programs in the [AGENTS.md](../../AGENTS.md) at the root, including the supported window setup and the non-disruptive workflow. Run the demo with `Program\RunInBackground\RunInBackground.ps1 simple-paint-demo`.

- `src/cards.rs` has the layout's constants. The preview and the GPU constants card get what's left of the window's height. The sphere's image in `src/preview.rs` is as big as the preview's view, so change the two together.
- Build the cards from Library/user-interface's widgets, theme roles and tokens, as [its README](../../Library/user-interface/README.md) says. What it lacks, build as a `Canvas` or a widget of the demo's own, or add it to the library.
- Test input handling with the tests and scripts in `Test/simple-paint-demo`, as the [README](README.md#testing) says. The state that scripts save has where each slider, row and button is, as `Interface.Targets`. Point at them by name, like `Light Point` or `Save`.
- To see a material other than the default, write a script that sets it with the sliders, like `Test/simple-paint-demo/Scripts/Material.json`. Or write it to `Paint.json` in the demo's folder, in the format of [Library/paint-settings](../../Library/paint-settings/README.md), run the demo without a script, and delete the file afterward unless the user wants it.
- Keep SimplePaint's math in [Library/simple-paint](../../Library/simple-paint/README.md).
