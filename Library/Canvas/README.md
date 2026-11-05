# Canvas

2D drawing for KGames' user interfaces, in fixed pixels:

- `Canvas` builds a frame's rectangles, antialiased lines, circles and rings, and monospaced
  text as one triangle list.
- `Cached` keeps the triangles of a part of a frame, to add again in later frames instead of
  drawing that part again while it doesn't change.
- `Font` rasterizes Consolas into a texture atlas with DirectWrite, at startup.
- `Pipeline` draws a canvas's triangles into a frame with one draw call, over whatever the
  program drew before.
- `theme` has the colors of KGames' light theme.
- `Rect` splits into parts, columns and rows, and `Lines` and `wrap` lay out lines of text.

[Library/UserInterface](../UserInterface/README.md) builds panels and widgets on it, and passes
it on, so programs draw on a canvas through the frames of their interface. Programs only support
fixed window sizes, so nothing here adapts to the window's size or scale.

| Path                  | Contents                                                                    |
| --------------------- | --------------------------------------------------------------------------- |
| `Source/Canvas.rs`    | Builds each frame's shapes and text, and keeps parts that don't change      |
| `Source/Font.rs`      | Rasterizes Consolas into a texture atlas with DirectWrite                   |
| `Source/Pipeline.rs`  | Draws a canvas with Direct3D 12: one draw call, alpha blending              |
| `Source/Theme.rs`     | The light theme's colors                                                    |
| `Source/Layout.rs`    | Rectangles, and lines of text                                               |
| `Shaders/Canvas.hlsl` | Vertex and pixel shaders, plus the embedded root signature                  |
| `Build.rs`            | Compiles the shaders                                                        |
