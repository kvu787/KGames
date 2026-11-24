# PixelFontTester

A box to type ASCII into, which shows it in the pixel font that [Glyphs.aseprite](Glyphs.aseprite)
draws and [Glyphs.txt](Glyphs.txt) maps, at 1 to 10 times its size. Its whole GUI is pixel perfect,
like Aseprite's, as a starting point for a GUI that's only that.

It's built on Win32 and Direct3D 12 directly, with [Library/graphics](../../Library/graphics/README.md),
not on [Library/user-interface](../../Library/user-interface/README.md), and draws its text with
the font alone, without DirectWrite. So it has none of that library's widgets, PerfBar, GameInput
or scripts. Its window does what that library's does for every program, as far as the tester needs
it: borderless fullscreen on its display, the error instead of the box in an unsupported setup,
no focus taken with `--no-activate`, and `F9`, `F10` and `Ctrl + Shift + Alt + F12`.

## Keys

| Keys                        | What they do                                                       |
| --------------------------- | ------------------------------------------------------------------ |
| Typing                      | ASCII's printable characters go in. Anything past ASCII is ignored |
| `Enter`                     | Breaks the line                                                    |
| `Tab`                       | Types a tab, which has no glyph, so it shows the box               |
| `Backspace`, `Delete`       | Delete the character before the caret, or after it, across lines   |
| Arrows                      | Move the caret. Up and down keep to the column they started in     |
| `Home`, `End`               | Go to the start or the end of the line                             |
| `Ctrl + Home`, `Ctrl + End` | Go to the start or the end of the text                             |
| `Ctrl + 1` to `Ctrl + 9`    | Show the text at 1 to 9 times the font's size, from the top row or |
|                             | the numpad. `Ctrl + 1` is the font's own size                      |
| `Ctrl + 0`                  | Shows the text at 10 times the font's size                         |
| `Ctrl + Shift + 0` to `9`   | Puts 0 to 9 of the font's pixels between characters. The sheet's   |
|                             | spacing, 2 now, is where it starts                                 |
| `F9`                        | Switches between the light and the dark theme                      |
| `F10`                       | Switches vertical sync on and off                                  |
| `Ctrl + Shift + Alt + F12`  | Quits at once                                                      |

The number keys alone type digits, and with `Shift`, what they type with it, like `!`. `AltGr`
and a number key types what the keyboard's layout says, like `{` in German's, as Windows sees
`AltGr` as `Ctrl + Alt`. With the spacing at 0, the caret covers the last column of the
character before it, as there's no gap between them for it.

The box scrolls, by whole columns and lines, to show the caret and the character after it, as
soon as a key moves it. Text doesn't wrap: it goes on to the right, and down, from the box's top
left.

## The font

The tester reads `Glyphs.aseprite` and `Glyphs.txt` as it starts, `Glyphs.aseprite` with
[Library/aseprite](../../Library/aseprite/README.md), so start it again after saving either. It reads
two of `Glyphs.aseprite`'s layers, in its first frame, whether they're visible or not, and leaves
out the rest, like its background:

| Layer    | What it has                                                                   |
| -------- | ----------------------------------------------------------------------------- |
| `grid`   | A cell for each glyph, which its outline marks, over the glyph's outer pixels |
| `glyphs` | The glyphs, each in its cell                                                  |

`Glyphs.txt`'s lines are the grid's rows of cells, top to bottom, and each line's characters are
its row's cells, left to right:

| Row | Glyphs                                                   |
| --- | -------------------------------------------------------- |
| 1   | `0` to `9`                                               |
| 2   | `A` to `Z`                                               |
| 3   | `a` to `z`                                               |
| 4   | Space, which is blank, then the symbols, from `'` to `_` |

So every one of ASCII's printable characters has a glyph. Every other character, like tab, shows
the box: the outline of a glyph's whole cell. `src/font.rs` finds the cells in the grid, rather than
at fixed places, so rows and cells can be added:

- A row of cells is a run of the grid's rows of pixels with ink, and a cell is a run of columns of
  pixels with ink, in its row, from the first to the last of its rows with ink in them. So a cell's
  outline and the lines across it, like its baseline, are all one cell. A pixel has ink if it isn't
  wholly transparent.
- The cells are all the same size, which is a glyph's, so the font is monospaced. A glyph is every
  pixel of the `glyphs` layer in its cell, with its alpha as its coverage. The `glyphs` layer
  draws nothing outside the cells.
- A character advances by a glyph's width and the narrowest gap between two cells side by side,
  and a line by a glyph's height and the narrowest gap between two rows of cells.
- `Glyphs.txt` has as many lines as the grid has rows, each with as many characters as its row has
  cells, and each character once, of ASCII's printable characters and tab.

If any of that doesn't hold, the tester doesn't open, and prints what's wrong. With the cells as
they are, a glyph is 6 × 13 pixels, characters are 8 apart, and lines 15.

## Pixel perfect

Everything that the tester draws is made of whole pixels of its own, each an integer number of the
display's pixels across and down, at whole pixels of the display, with nothing between them: no
antialiasing, no filtering, no blending across pixels' edges. There are two kinds of those pixels:

- The GUI's, 2 × 2 of the display's (`GUI_SCALE` in `src/layout.rs`), like Aseprite's screen
  scaling: the box, its border and padding, and the error in an unsupported setup are laid out
  in them, on their grid.
- The text's, which `Ctrl` and a number set: the font's pixels, the spacing between characters,
  which `Ctrl + Shift` and a number set, and the caret, one of them wide.

The renderer draws nothing but rectangles of whole pixels, filled with a color or with a glyph,
whose pixel shader finds the glyph's pixel under each of the display's with integer math. Its
tests draw frames on the GPU and on the CPU, and check that they match, pixel for pixel.

## Running it

`HumanRun.cmd` builds it and runs it on the primary display. Coding agents run it with
[RunInBackground](../RunInBackground/README.md), which can't type, so it takes the text to start
with, and the size:

```powershell
Program\RunInBackground\RunInBackground.ps1 pixel-font-tester --arguments '--text "Hello\nWorld" --size 4'
```

| Argument                     | Value                                                           |
| ---------------------------- | --------------------------------------------------------------- |
| `--display <id>`             | The display to open on, as IdentifyDisplays prints it, or       |
|                              | `Primary`, which it is by default                               |
| `--no-activate`              | Opens beneath the other windows, without taking focus           |
| `--no-input`                 | Ignores the keyboard                                            |
| `--vsync`, `--no-vsync`      | Starts with vertical sync on, as by default, or off             |
| `--theme Light`, `Dark`      | Starts in the theme. Light by default                           |
| `--size <1-10>`              | Starts at the size. 1 by default                                |
| `--spacing <0-9>`            | Starts with that many of the font's pixels between characters.  |
|                              | The sheet's spacing by default                                  |
| `--text <text>`              | Starts with the text, typed: `\n` breaks the line, and `\\` is  |
|                              | a backslash                                                     |

It has no scripts, as they're Library/user-interface's. Its tests, in
[Test/pixel-font-tester](../../Test/pixel-font-tester), test what typing and the keys do, where
everything is, and what the GPU draws, without a window.
