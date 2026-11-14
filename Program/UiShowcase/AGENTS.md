# UiShowcase

Follow the rules for GUI programs in the [AGENTS.md](../../AGENTS.md) at the root, including the supported window setup and the non-disruptive workflow. Run the showcase with `Program\RunInBackground\RunInBackground.ps1 UiShowcase`.

- The showcase shows every widget of [Library/UserInterface](../../Library/UserInterface/README.md). When the library gets a widget or a kind of input, add it here too, with a test in `Test/UiShowcase` and a script that captures it.
- `src/cards.rs` has the layout's constants. The cards at the bottom of each column get what's left of the window's height.
- Test input handling with the tests and scripts in `Test/UiShowcase`, as the [README](README.md#testing) says. The state that scripts save has where each widget is, as `Interface.Targets`.
