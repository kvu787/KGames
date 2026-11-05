# Canvas

2D drawing for KGames' user interfaces, in fixed pixels:

- `Canvas` builds a frame's shapes and text as instances, one for each shape or glyph, which the
  pipeline draws in one draw call. Shapes are rounded boxes, filled, outlined or both, Gaussian
  shadows, lines with round ends, circles and rings. The pixel shader works out how much of each
  pixel a shape covers from its outline, so edges and corners are antialiased exactly at any
  size. A clip stack keeps what's drawn inside an area, for parts of a frame that scroll.
- `Cached` keeps the instances of a part of a frame, to add again in later frames instead of
  drawing that part again while it doesn't change.
- `Fonts` rasterizes the type ramp and the icons into one texture atlas with DirectWrite, at
  startup, and measures, fits and wraps text.
- `Pipeline` draws a canvas's instances into a frame with one draw call, over whatever the
  program drew before.
- `theme` has the light theme, the default, and the dark one, which name each color by its role.
- `Rect`, `Stack` and `Lines` lay out areas and lines of text.

[Library/UserInterface](../UserInterface/README.md) builds cards and widgets on it, and passes it
on, so programs draw on a canvas through the frames of their interface. Programs only support
fixed window sizes, so nothing here adapts to the window's size or scale.

## Text

Text is proportional, in Fluent 2's type ramp, in the fonts that Windows 11 ships. Each glyph is
rasterized at three offsets across a pixel, and drawn at the nearest, so the gaps between letters
are even. Baselines are on whole pixels.

| Style        | Font                                | Size | Line | For                            |
| ------------ | ----------------------------------- | ---- | ---- | ------------------------------ |
| `Caption`    | Segoe UI Variable Small             | 12   | 16   | Small print, keys              |
| `Body`       | Segoe UI Variable Text              | 14   | 20   | Most text                      |
| `BodyStrong` | Segoe UI Variable Text, semibold    | 14   | 20   | Titles of cards, what's chosen |
| `Subtitle`   | Segoe UI Variable Display, semibold | 20   | 28   | The app's title                |
| `Title`      | Segoe UI Variable Display, semibold | 28   | 36   | Headings and big numbers       |
| `Mono`       | Cascadia Mono                       | 13   | 20   | Numbers in columns, code, logs |

Icons are Segoe Fluent Icons, at 12 and 16 pixels, by what they show, like `Icon::Search`. The
fonts have printable ASCII, Latin-1, and a few marks of typography and math, like the ellipsis
that cuts text short. Other characters are drawn as a question mark, and `Fonts::has_glyph` says
which characters the fonts have.

## Colors

A `Theme` names each color by its role, after Fluent 2's: surfaces, from the window's background
to cards, wells and what floats over them; text, from primary to disabled; controls, their edges
and the subtle fills of rows; the accent; the focus ring; success, caution and critical, and their
info bars' backgrounds; and six colors for charts. Colors are sRGB, and blend in sRGB, as they're
drawn. `Test/Canvas` checks that each theme's text has enough contrast on its surfaces.

| Path                  | Contents                                                                    |
| --------------------- | --------------------------------------------------------------------------- |
| `Source/Canvas.rs`    | Colors, and each frame's instances: shapes, text, icons and clipping        |
| `Source/Fonts.rs`     | Rasterizes the type ramp and the icons into an atlas with DirectWrite       |
| `Source/Pipeline.rs`  | Draws a canvas with Direct3D 12: a quad per instance, in one draw call      |
| `Source/Theme.rs`     | The light and dark themes                                                   |
| `Source/Layout.rs`    | Rectangles, stacks, and lines of text                                       |
| `Shaders/Canvas.hlsl` | Vertex and pixel shaders, with the shapes' outlines, plus the root          |
|                       | signature                                                                   |
| `Build.rs`            | Compiles the shaders                                                        |
