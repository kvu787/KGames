# PixelFontTester

Follow the rules for GUI programs in the [AGENTS.md](../../AGENTS.md) at the root, including the supported window setup and the non-disruptive workflow. Run the tester with `Program\RunInBackground\RunInBackground.ps1 pixel-font-tester`, and give it text to show with `--arguments '--text "..." --size 3'`, as it can't be typed into there.

- It isn't built on Library/user-interface, and doesn't use DirectWrite: don't add either. So it has no PerfBar, and its box is laid out from the window's top left, and it has no scripts. `src/window.rs` does what that library's window does for every program, as far as the tester needs it.
- Keep the whole GUI pixel perfect, like Aseprite's: everything is whole pixels of its own, an integer number of the display's across and down, with no antialiasing or filtering. Lay out what isn't the text in the GUI's pixels, `GUI_SCALE` in `src/layout.rs`, on their grid. The renderer draws only rectangles of whole pixels, filled with a color or a glyph, which `Test/pixel-font-tester/tests/rendering.rs` checks pixel for pixel against the same frame drawn on the CPU.
- The font is what `Glyphs.aseprite` at the root draws, which the user edits. Don't edit it, and keep tests of the real font to what holds however its glyphs are drawn. Test how glyphs are found with the made-up sheet in `Test/pixel-font-tester/tests/common`.
- Input is the window's keyboard messages, which `src/window.rs` turns into the tester's `Input`. Test what input does with `Tester::handle` in `Test/pixel-font-tester`.
