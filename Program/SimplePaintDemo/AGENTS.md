# SimplePaintDemo

Follow the rules for GUI programs in the [AGENTS.md](../../AGENTS.md) at the root, including the supported window setups and the non-disruptive workflow. Run the demo with `Program\RunInBackground\RunInBackground.ps1 SimplePaintDemo`.

- `Source/Panels.rs` has the layout's constants. Of the 71 pixels that borderless fullscreen adds to the height, the preview and the GPU constants panel get all of them.
- Test input handling with the tests and scripts in `Test/SimplePaintDemo`, as the [README](README.md#testing) says. The state that scripts save has where each slider and button is, as `Targets`.
- To see a material other than the default, write a script that sets it with the sliders, like `Test/SimplePaintDemo/Scripts/Material.json`. Or write it to `Paint.json` in the demo's folder, in the format of [Library/PaintSettings](../../Library/PaintSettings/README.md), run the demo without a script, and delete the file afterward unless the user wants it.
- Keep SimplePaint's math in [Library/SimplePaint](../../Library/SimplePaint/README.md). The tone curve plot repeats the specification's color curve only to draw it.
