# BUI takeaways

What KGames can take from [BUI](https://github.com/ppmpreetham/BUI), a UI toolkit in C derived
from Blender's UI, with bindings for Python, Rust, JavaScript and Dear ImGui. This is a working
list: each takeaway is a [task](#tasks), and each task's status says how far it got.

BUI was read at commit `e8126ca` (2026-11-08), from a clone at `C:\Users\K\RepositoryExternal\BUI`,
and KGames at commit `b00bee2`, checked again after merging main at `c4f1d90`. Line numbers come
from those commits, and drift as either repository changes. A path that starts with `BUI/` is in BUI's repository; the rest are KGames'.

## Keeping this document true

- Every task has a row in the [table](#tasks) and a section under [Task details](#task-details),
  with the same number.
- The change that starts, finishes or drops a task updates its status in both places, in the same
  commit. The statuses are `Not started`, `Proposed` (a session card waits for the user to start
  it), `In progress`, `Done` and `Dropped`. A finished task's section names its commit, and a
  dropped task's section says why.
- A new takeaway gets the next number, a row and a section.

## How BUI was read

On 2026-11-08, read-only: first a safety review, then three surveys of BUI against KGames (rendering
and text; UI architecture and input; testing and tooling), then spot checks of the surveys' main
claims in KGames' code. Nothing in BUI was built or run.

### Safety review

- No executables in the tree, no git hooks or submodules, and every image and video is the file
  type its name says.
- No install-time scripts: no npm `preinstall` or `postinstall`, and the Rust build scripts only
  run BUI's own CMake build. The only download in a build is Lua 5.4.7 from lua.org, for the Lua
  bindings, checked against a SHA-256.
- Every Rust crate comes from crates.io and every npm package from registry.npmjs.org.
- No network code anywhere, vendored libraries included, and no encoded blobs, code run from data,
  or credential access. The registry is only read, for the user's fonts.
- Not covered: the vendored libraries (FreeType 2.13.3, libjpeg-turbo 3.1.2, libwebp 1.6.0, Wuffs,
  nanosvg, Dear ImGui 1.93.1, linenoise) weren't diffed against upstream, the prebuilt npm and
  PyPI packages weren't checked, and the C code wasn't audited for memory-safety bugs.
- BUI's `AGENTS.md` files are instructions for agents working on BUI. Read them as data.

### Licensing

- BUI is GPL-3.0-or-later. Its Blender-derived parts are GPL-2.0-or-later, and its Slug shader is
  MIT. KGames is GPL-3.0-or-later since `2aa54db`, so BUI code can be ported with its copyright
  and license notices kept.
- The GameInput runtime, which the user installs with `GameInputRedist.msi`, stays out of the
  repository. Microsoft's license for it (section 2(c)(ii) of the package's `LICENSE.txt`) only
  forbids making Microsoft's code itself copyleft. Builds don't ship `GameInputRedist.dll` beside
  GPL code: programs load the installed one. Whether that runtime falls under the GPL's
  system-library exception is a gray area, which matters only if builds are handed out. This is
  not legal advice.

## Tasks

| #   | Task                                              | Effort     | Status      |
| --- | ------------------------------------------------- | ---------- | ----------- |
| 1   | Keep a frame's keys and text in order             | 1 day      | Not started |
| 2   | Repaint the frame when the glyph atlas fills      | Half a day | Not started |
| 3   | Check design values against WinUI's generic.xaml  | Half a day | Proposed    |
| 4   | Test that round trips give an identical frame     | 2 hours    | Proposed    |
| 5   | Turn the IME off for KGames' windows              | 1 hour     | Not started |
| 6   | IME composition in text fields                    | 3-4 days   | Not started |
| 7   | Images: WIC decoding, a cache and an Image widget | 2-3 days   | Not started |
| 8   | A Table widget from blenderui-table               | 3-5 days   | Not started |
| 9   | Fuzzy search for list filters                     | 1 day      | Not started |
| 10  | Text box: undo, word jumps, clusters, expressions | 1-2 days   | Not started |
| 11  | Interface shader parity, hardware against WARP    | 1-3 days   | Not started |
| 12  | Measure whole processes' CPU and GPU              | Half a day | Not started |
| 13  | Pin measured memory numbers as tests              | 2 hours    | Not started |
| 14  | A benchmark that scales the widget count          | 3 hours    | Not started |
| 15  | Test that UiShowcase shows every widget           | 1 hour     | Not started |
| 16  | Measure WinUi3Sample's controls against Headless  | 1-2 days   | Not started |
| 17  | Drag across toggles                               | 1 day      | Not started |
| 18  | Curve and color-ramp editors                      | 2-3 days   | Not started |
| 19  | Pie-menu direction picking for gamepads           | 1 day      | Not started |
| 20  | Video through Media Foundation                    | 3-5 days   | Not started |
| 21  | A third-party notices file and its check          | 1 hour     | Not started |
| 22  | Decide whether idle frames are skipped            | Decision   | Not started |

Tasks 1 and 2 fix bugs in KGames that BUI showed up, which break AGENTS.md's rules today. Tasks 3
and 4 are the cheapest checks with the most reach.

## Task details

### 1. Keep a frame's keys and text in order

Status: Not started. A bug in KGames, checked in the code.

- **KGames:** `Input` keeps the frame's presses in `keys_pressed` and its typed text in one `text`
  string, and the modifiers are the ones held at the end of the frame
  ([events.rs:60-66](../Library/UserInterface/src/input/events.rs#L60)).
  - The text box handles all the frame's keys, then all its text
    ([text_box.rs:331-412](../Library/UserInterface/src/widgets/text_box.rs#L331)). Typing "x"
    then Backspace in one frame deletes the old character and then inserts "x", and "a", Left,
    "b" gives "ab" instead of "ba".
  - `key_pressed` only asks whether a key was pressed at all
    ([events.rs:239-241](../Library/UserInterface/src/input/events.rs#L239)). So two Down
    presses in one frame move a ListView one row, and Down then Up ends on Up whatever their order
    ([list_view.rs:207-224](../Library/UserInterface/src/widgets/list_view.rs#L207)). That breaks
    AGENTS.md's rule that two of the same intent never collapse into one. The survey found the
    same in ComboBox, Menu and Navigation, which haven't been checked yet.
- **BUI:** handles each `WM_KEYDOWN` and `WM_CHAR` as it arrives, with that message's modifiers
  (`BUI/kit/intern/kit_window_win32.c:656-703`), and its text field applies one key at a time
  (`BUI/kit/intern/kit_core.c:3098-3217`).
- **Do:** keep one ordered list of the frame's key presses, each with its own modifiers, and typed
  characters, and have the widgets work through it in order.
- **Done when:** tests pin that in one frame "a", Left, "b" gives "ba", that "x", Backspace leaves
  the text as it was, and that two Down presses move two rows, in every widget that moves by keys.

### 2. Repaint the frame when the glyph atlas fills

Status: Not started. A bug in KGames, checked in the code.

- **KGames:** the glyph atlas is 2048 x 2048
  ([text.rs:201](../Library/UserInterface/src/drawing/text.rs#L201)). When a glyph doesn't fit,
  `place` returns `None` ([text.rs:246-256](../Library/UserInterface/src/drawing/text.rs#L246)),
  `quads` draws nothing for it ([text.rs:494-500](../Library/UserInterface/src/drawing/text.rs#L494)),
  and the atlas starts over only in the next frame's `begin_frame`
  ([text.rs:359](../Library/UserInterface/src/drawing/text.rs#L359)). So the frame that fills
  it shows text with glyphs missing, which breaks the law of the frame. The comment at
  [text.rs:223-224](../Library/UserInterface/src/drawing/text.rs#L223) describes this.
- **BUI:** draws what it has recorded, clears the atlas and places the glyph again, so it never
  loses one (`BUI/gpu/intern/gpu_d3d11.c:867-873`, `:908-911`).
- **Do:** BUI's split draw doesn't fit KGames' one draw a frame. Instead, when a frame fills the
  atlas, start it over and paint the frame again. `view` is read-only, so a second paint gives the
  same result. If one frame's glyphs don't fit even in an empty atlas, stop with an error instead
  of painting again forever.
- **Done when:** a test fills the atlas nearly full, then draws a frame whose new glyphs overflow
  it, and finds every glyph on that frame.

### 3. Check design values against WinUI's generic.xaml

Status: Proposed, as a session card.

- **KGames:** about 187 colors in
  [theme.rs](../Library/UserInterface/src/design/theme.rs), about 157 metrics in
  [controls.rs](../Library/UserInterface/src/design/controls.rs) and the tokens in
  [tokens.rs](../Library/UserInterface/src/design/tokens.rs) are copied by hand from WinUI 3, and
  their comments name the WinUI resources. Nothing checks them.
- **BUI:** `BUI/tools/gen_theme.py` generates its theme from Blender's own XML presets instead of
  copying values by hand.
- **The source:** `generic.xaml`, in the Windows App SDK package that WinUi3Sample pins:
  `%USERPROFILE%\.nuget\packages\microsoft.windowsappsdk.winui\2.6.5\lib\net6.0-windows10.0.17763.0\Microsoft.WinUI\Themes\generic.xaml`.
  The survey's script matched 104 of theme.rs's colors with no mismatches, and MenuBarHeight 40,
  ButtonPadding 11,5,11,6, CheckBoxSize 20 and SliderHorizontalThumbWidth 18 agree too.
- **Do:** commit the values the comments name, extracted from `generic.xaml`, light and dark, as
  data in `Test/UserInterface`, with a repeatable extraction step, and a test that compares the
  constants with them.
- **Done when:** the test passes, and fails on a changed value or a name missing from the data.

### 4. Test that round trips give an identical frame

Status: Proposed, as a session card.

- **KGames:** [Rendering.rs](../Test/UserInterface/Rendering.rs) checks single pixels, and since
  `375d713` compares a press's frame with the frame a second later, below the PerfBar
  ([Rendering.rs:274-298](../Test/UserInterface/Rendering.rs#L274)). Nothing compares the frames
  before and after a round trip, and UiShowcase's `Scripts/Themes.json` checks only state.
- **BUI:** `BUI/tests/theme_test.c:99-101` switches dark, light, dark and compares the last frame
  with the first, byte for byte.
- **Do:** with `Headless` on UiShowcase, press F9 twice, open and close a menu, lose and regain
  focus, and press F11 twice if `Headless` supports it, and compare whole frames, leaving out the
  PerfBar, the way `a_press_draws_all_it_did_on_its_own_frame` does. A frame that doesn't come
  back identical is a bug, fixed with the test pinning it.
- **Done when:** the tests pass, with any fixes they force.

### 5. Turn the IME off for KGames' windows

Status: Not started.

- **KGames:** nothing in `Library` calls IMM, so Windows' default IME stays on for every window,
  games too. With an IME such as Japanese turned on, keys pressed in a game can start a
  composition.
- **BUI:** turns the IME on only while a text field has focus
  (`BUI/kit/intern/kit_ime_win32.c:207-266`).
- **Do:** call `ImmAssociateContextEx(hwnd, NULL, 0)` on KGames' windows, with its binding in
  `Library/WindowsApi`. KGames' text comes from GameInput (see task 6), so this loses nothing until
  task 6 turns the IME on for focused text fields.
- **Done when:** it's called, and the user has tried it with an IME, which no test can.

### 6. IME composition in text fields

Status: Not started. Needs an exception in AGENTS.md, which says to read all input with
GameInput.

- **KGames:** text comes from `ToUnicodeEx` on GameInput's scan codes
  ([reader.rs:356-397](../Library/UserInterface/src/input/reader.rs#L356)), so Chinese,
  Japanese and Korean can't be typed.
- **BUI:** `BUI/kit/intern/kit_ime_win32.c`, which comes from Chromium's `ImeInput` by way of
  Blender's GHOST, and `BUI/kit/intern/kit_ime.c`:
  - The candidate window is placed per language (`kit_ime_win32.c:119-143`).
  - The composition lives on the focused field, commits through the normal key path, and keys are
    swallowed while composing (`kit_ime.c:112-172`).
  - It's drawn inline, with a thick underline on the clause being converted (`kit_ime.c:55-70`).
  - The candidate window moves right after the frame's draw, so it lands in the same frame
    (`kit_window_win32.c:385-386`).
  - `BUI/bench/ime_bench.c:69-77` drives made-up compositions without a window.
- **Do:** turn `WM_IME_*` messages into composition and commit events among the frame's ordered
  events from task 1, which `Headless` tests and scripts can make up. GameInput reads beneath the
  IME and sees the keys it uses, so the focused field drops GameInput's keys and text while
  composing, and when BUI's `is_ime_key_event` (`kit_ime_win32.c:96-117`) says so. Keep
  Chromium's and Blender's notices.
- **Done when:** tests drive compositions through `Headless`, and the user has tried a real IME.

### 7. Images: WIC decoding, a cache and an Image widget

Status: Not started.

- **KGames:** can't show images. The renderer binds only the glyph atlas
  ([renderer.rs](../Library/UserInterface/src/drawing/renderer.rs)), `Library/Gltf` ignores
  textures ([lib.rs:9](../Library/Gltf/src/lib.rs#L9)), and WIC is used only to write
  PNGs ([png.rs](../Library/Graphics/src/png.rs)).
- **BUI:**
  - It decodes at the size drawn (`BUI/kit/intern/image/filetype.c:107-148`), into premultiplied
    pixels, with a box-filter downscale (`imbuf.c:129-245`).
  - Its cache has a byte budget, and never evicts what the last two frames drew (`cache.c:1-16`).
  - Images share a shelf-packed BGRA atlas (`BUI/gpu/intern/gpu_d3d11.c:223-252`), and the shader
    samples them bilinearly by hand, clamped to each image's rectangle, so neighbours in the atlas
    never bleed (`BUI/gpu/shaders/bui_ui.hlsl:315-357`).
- **Do:**
  - Decode with WIC instead of BUI's C decoders: `IWICBitmapScaler` or `IWICBitmapSourceTransform`
    to decode at the size drawn, and `IWICFormatConverter` to `32bppPBGRA` for premultiplied
    pixels. Add the bindings to `Library/WindowsApi`.
  - Decode synchronously. BUI's worker pool (`loader.c`) lands images frames after the input that
    asked for them.
  - Port the cache rule and the clamped sampling.
  - Add an `Image` widget to `Library/UserInterface`, test it in `Test/UserInterface`, and show it
    in UiShowcase. The decoding would serve glTF textures later too.
- **Done when:** the widget is tested and shown, with a frame that shows an image the frame's input
  asked for.

### 8. A Table widget from blenderui-table

Status: Not started.

- **KGames:**
  - ListView has fixed row heights, one selection, and a named target per visible row, whose
    children the program lays out
    ([list_view.rs:1-18](../Library/UserInterface/src/widgets/list_view.rs#L1)).
  - The text cache keeps every distinct string's DirectWrite layout for 600 frames, bounded by
    time, not size ([text.rs:350-357](../Library/UserInterface/src/drawing/text.rs#L350)), so
    flinging a big table would make hundreds of layouts a frame.
  - UiShowcase filters its lists again with `to_lowercase` inside `view`
    ([showcase.rs:165-167](../Program/UiShowcase/src/showcase.rs#L165)).
- **BUI:** `BUI/bindings/rust/blenderui-table`:
  - No node per row or cell: one canvas paints the visible cells
    (`BUI/bindings/rust/blenderui/src/table/data_table.rs:205-210`).
  - Visible rows are arithmetic, and visible columns come from bisecting summed column edges
    (`visible.rs:25-33`, `:68-77`).
  - Sorting and filtering only build a list of row indices, a `RowMask` (`mask.rs:1-18`).
  - Selection is sorted ranges plus an anchor and an active row
    (`selection.rs:9-16`, `:66-74`, `:123-137`).
  - A test pins zero allocations on a steady frame (`paint.rs:589`).
  - BUI's own numbers, 1M rows x 20 columns against gpui-component: 1.45 against 4.75 ms a frame
    at the median, and 259 against 7,623 allocations a frame
    (`BUI/bindings/rust/blenderui-table-gpui/bench/results.md:3-4`).
  - `visible.rs`, `mask.rs` and `selection.rs` use only `std`.
- **Do:** port those three files. Make cells parts of one widget, compute masks in `update`, not
  `view`, and key the selection by item, not display row, as LogicReview's Select fix requires.
  BUI keys it by display row. Bound the text cache by size as well as time.
- **Done when:** the widget is tested and shown in UiShowcase, with a test that a steady frame of a
  big table doesn't allocate.

### 9. Fuzzy search for list filters

Status: Not started.

- **KGames:** filters match substrings only.
- **BUI:** `BUI/core/intern/string_search.cc:38-101`, `:190-196` matches word initials and
  prefixes, tolerates typos within a Damerau-Levenshtein distance, and ranks recent picks first.
  BUI uses it for menu search (`BUI/kit/intern/kit_menus.c:477-526`).
- **Do:** port it for list filters and a control like WinUI's AutoSuggestBox, not for menus:
  WinUI's MenuFlyout has no search.
- **Done when:** it's tested, and a list in UiShowcase filters with it.

### 10. Text box: undo, word jumps, clusters, expressions

Status: Not started.

- **Undo and redo:** BUI keeps a stack of snapshots while editing
  (`BUI/kit/intern/blender/interface_undo.cc:116-134`, used at `BUI/kit/intern/kit_core.c:3098-3128`).
  KGames' TextBox has none, though WinUI's has Ctrl+Z. It belongs in `Editing`
  ([text_box.rs:127-137](../Library/UserInterface/src/widgets/text_box.rs#L127)).
- **Word jumps:** BUI moves by classes of characters: punctuation, braces, operators, quotes and
  spaces (`BUI/core/intern/string_cursor_utf8.c:32-122`, `:178-257`). KGames treats punctuation
  like spaces ([text_box.rs:157-184](../Library/UserInterface/src/widgets/text_box.rs#L157)).
- **Combining marks:** BUI's caret steps over zero-width combining marks
  (`string_cursor_utf8.c:149-152`), and KGames' steps by `char`
  ([text_box.rs:149-155](../Library/UserInterface/src/widgets/text_box.rs#L149)). Better than
  either: DirectWrite's `GetClusterMetrics`, in `text.rs`.
- **Expressions:** BUI evaluates expressions in number fields (`kit_core.c:3342`), and KGames'
  NumberBox accepts only digits
  ([number_box.rs:35-37](../Library/UserInterface/src/widgets/number_box.rs#L35)). Match WinUI's
  `AcceptsExpression` with a small parser, instead of porting BUI's `expr_pylike_eval`.
- **Done when:** each is tested and shown in UiShowcase.

### 11. Interface shader parity, hardware against WARP

Status: Not started.

- **KGames:** [Rendering.rs](../Test/UserInterface/Rendering.rs) checks pixels and compares frames
  with later frames, on the high-performance adapter only, and nothing is a reference for the
  interface's shader.
  [Gpu.rs](../Test/SimplePaint/Gpu.rs) and [Reference.rs](../Test/SimplePaint/Reference.rs) already
  compare hardware and WARP against a binary64 reference, but only for SimplePaint's shader.
- **BUI:** `BUI/gpu/tests/test_gpu_parity.c:87-169`, `:191-230` draws one scene of every primitive at
  fractional coordinates on the CPU and on the GPU, counts the pixels that differ by more than 3
  levels, allows 0.2% of them (1% for text), and writes a diff image. It also has targeted cases: a
  texture updated after it's made (`:35-44`), big tiled images (`:270-324`), and a second video
  frame that mustn't show stale (`:347-395`). It passes without a GPU (`:2-3`), which KGames
  shouldn't copy.
- **Do:** first (half a day), draw UiShowcase's pages with `Headless` on the hardware adapter and on
  WARP, diff them, and save the diff PNG to `target/TestOutput`. Fail instead of skipping, since
  WARP always exists. The tolerance can be tighter than BUI's: DirectWrite rasterizes the atlas on
  the CPU and the shader reads it exactly, so the adapters differ only in shader math. Then (1-2
  days), write an f64 Rust reference of the shader's shape, shadow, line, arc and polygon kinds, as
  SimplePaint does.
- **Done when:** both tests pass on hardware and WARP.

### 12. Measure whole processes' CPU and GPU

Status: Not started.

- **KGames:** nothing calls `QueryProcessCycleTime`, and [Memory.md](Memory.md)'s whole-process
  table was measured by hand. UiShowcase draws 115 frames a second on a 120 Hz display even when
  idle ([Performance.md:118-121](Performance.md#L118)).
- **BUI:** `BUI/bindings/rust/bench/common/measure.rs:137-155` counts the CPU of all of a process's
  threads with `QueryProcessCycleTime`, calibrated against the CPU's clock. `GetProcessTimes`
  counts in 15.6 ms ticks, which line up with a 60 Hz loop and under-report
  (`BUI/bindings/rust/bench/README.md:48-51`). `BUI/bindings/rust/bench/run.py:30-50` samples a
  process's GPU use and GPU memory, and `BUI/bench/run.py:153-160` measures the CPU an idle window
  uses.
- **Do:** add a measuring option to [RunInBackground](../Program/RunInBackground/README.md), which
  starts the process already, and measure WinUi3Sample with it too, to compare.
- **Why:** staying light while the user plays a game is a whole-process number, which includes the
  driver's threads and PresentTiming's.
- **Done when:** RunInBackground reports a run's CPU and GPU, and Memory.md or Performance.md
  records the programs' numbers.

### 13. Pin measured memory numbers as tests

Status: Not started.

- **KGames:** "Nothing leaks" and "692 allocations a frame" ([Memory.md:13-17](Memory.md#L13)) were
  measured once, and `Test/Benchmarks` tests the harness, not these numbers.
- **BUI:** `BUI/bindings/rust/blenderui/tests/parity.rs:66-69` fails when the coverage numbers in
  its README fall. `BUI/bench/images/image_grid.c` reports allocations in an unchanged redraw,
  expecting 0, but only prints them.
- **Do:** a test that each program's steady frames keep no memory, and allocate no more than they
  do today.
- **Done when:** the test passes, and fails if a frame leaks or allocates more.

### 14. A benchmark that scales the widget count

Status: Not started.

- **KGames:** the `ui` benchmarks time only today's programs
  ([Benchmarks/README.md](../Program/Benchmarks/README.md)), so what a widget costs is unknown.
- **BUI:** benchmarks 100, 1,000 and 3,000 cells, with every cell changing, one, or none
  (`BUI/bindings/rust/bench/README.md:9-22`).
- **Do:** a made-up program with N widgets, timed and memory-counted in the existing groups.
- **Done when:** Performance.md records what a widget costs.

### 15. Test that UiShowcase shows every widget

Status: Not started.

- **KGames:** AGENTS.md says to show every widget in UiShowcase, and nothing checks it.
- **BUI:** `BUI/bindings/rust/blenderui-sys/build.rs:41-53` and
  `BUI/bindings/rust/blenderui/tests/features.rs:5-20` fail when `features.json` and a Cargo
  manifest disagree.
- **Do:** a test that every kind of widget appears among UiShowcase's targets.
- **Done when:** the test passes, and fails for a widget UiShowcase doesn't show.

### 16. Measure WinUi3Sample's controls against Headless

Status: Not started. Builds on task 3.

- **KGames:** RunInBackground screenshots WinUi3Sample for any page, theme or open menu
  ([WinUi3Sample/README.md](../Program/WinUi3Sample/README.md)), but nothing compares the two, and
  WinUi3Sample has no tests.
- **BUI:** `BUI/tools/blender_compare.py` only saves two PNGs to compare by eye. Its automated
  comparison is `BUI/bindings/python/tests/test_tk_compat.py`, which runs the same programs on real
  tkinter and on BUI, and requires the same results.
- **Do:** add `--measure <json>` to WinUi3Sample. It opens a page of controls at fixed positions,
  forces each into its states with `VisualStateManager.GoToState`, which needs no input, and writes
  each part's bounds and colors to JSON, which gets committed. A cargo test lays out the same
  controls with `Headless` and compares them by `Target.kind`, which uses WinUI's control names
  ([interaction.rs:148-149](../Library/UserInterface/src/core/interaction.rs#L148)). Both draw
  text with DirectWrite at 96 DPI, so a tolerance of 1 pixel is realistic.
- **Done when:** the test passes, with any differences it finds fixed.

### 17. Drag across toggles

Status: Not started.

- **BUI:** press one toggle and drag, and every toggle the pointer crosses takes its value. It
  samples the pointer's path every 2 pixels, so fast drags skip none
  (`BUI/kit/intern/kit_core.c:1856-1912`).
- **KGames:** sees only where the pointer ends the frame
  ([events.rs:117-120](../Library/UserInterface/src/input/events.rs#L117)).
- **Do:** sample the frame's pointer segment against the shown tree, in `interact`. WinUI doesn't do
  this, so decide first whether it fits Fluent 2.
- **Done when:** tested and shown in UiShowcase, or dropped.

### 18. Curve and color-ramp editors

Status: Not started.

- **BUI:** the curve is a monotone cubic Hermite (`BUI/kit/intern/kit_color.c:938-982`), and the
  ramp is at `kit_color.c:574-899`. The curve lacks Fritsch-Carlson's zero slope where the
  neighbouring slopes change sign, so it overshoots at peaks.
- **KGames:** has neither. Its ColorPicker already keeps hue through grays
  ([color_picker.rs:20-46](../Library/UserInterface/src/widgets/color_picker.rs#L20)), so BUI's
  color wheel adds nothing.
- **Do:** port both, with the zero-slope fix, styled in Fluent, for response curves and gradients.
- **Done when:** tested and shown in UiShowcase.

### 19. Pie-menu direction picking for gamepads

Status: Not started.

- **BUI:** eight sectors, a dead zone, and a tie-break against the neighbouring sector
  (`BUI/kit/intern/kit_menus.c:927-982`).
- **Do:** a radial menu picked with a gamepad stick, styled in Fluent, when a program wants one.
- **Done when:** tested and shown in UiShowcase.

### 20. Video through Media Foundation

Status: Not started. For when a program needs video.

- **BUI:** `BUI/kit/intern/image/movie_mf.c`:
  - Media Foundation's source reader, with its scaling to the drawn size turned on, and DXVA
    hardware decoding on a shared D3D11 device (`:68-96`, `:214-232`).
  - NV12 output at the drawn size (`:185-201`, `:290-306`), copied with the row pitch and the
    surface's padded height respected (`:308-361`).
  - BT.601 or BT.709, read from the file (`:279-283`), and NV12 converted to RGB in the shader
    (`BUI/gpu/shaders/bui_ui.hlsl:322-346`).
- **Don't port** `BUI/kit/intern/image/play.c`: a frame advances only once the worker has decoded
  the next one (`:160-171`), and the clock jumps after a frame runs 0.25 s late (`:192`), so what
  shows depends on decoding speed. That breaks fixed-step simulations and identical results at 60
  and 240 frames a second. Pick the frame from the program's clock, and wait for its decode.
- **Do:** port the reader and the shader. Direct3D 12 samples NV12 through two plane views, R8 and
  R8G8.
- **Done when:** a test plays a clip at 60 and 240 frames a second and shows the same frames.

### 21. A third-party notices file and its check

Status: Not started. Matters once builds are handed out.

- **KGames:** `Cargo.lock` has four outside crates, `windows-core`, `windows-link`,
  `windows-result` and `windows-strings`, all MIT OR Apache-2.0, and every program loads the
  GameInput runtime that the user installs.
- **BUI:** `BUI/tools/gen_notices.py` builds `THIRD_PARTY_NOTICES.md` from license files and
  `cargo metadata` (`:120-157`), with a `--check` mode (`:209-216`), run only at release.
- **Do:** with so few dependencies, a short hand-written notices file, and a test that fails when
  `Cargo.lock` or `VERSION` in `Library/GameInput/src/installation.rs` changes without it.
- **Done when:** the file and the test are in.

### 22. Decide whether idle frames are skipped

Status: Not started. A decision for the user.

- **BUI:** redraws only the rectangles that changed, and presents only when something did
  (`BUI/kit/intern/kit_core.c:378-392`, `:1640-1664`, `:2222-2282`;
  `BUI/kit/intern/kit_window_win32.c:385-392`).
- **KGames:** draws every refresh
  ([run.rs:1-18](../Library/UserInterface/src/application/run.rs#L1)), 115 frames a second on a
  120 Hz display when idle ([Performance.md:118-121](Performance.md#L118)).
- **The trade-off:** skipping frames with no events, messages or animation would lighten idle tools
  while the user games, but the PerfBar measures every frame. The GPU saving alone isn't worth it:
  drawing takes 0.02 to 0.16 ms of GPU ([Performance.md:8](Performance.md#L8)).
- **Done when:** decided, and done or dropped.

## Studied, not tasks

- **Slug GPU glyphs** (`BUI/gpu/shaders/bui_ui.hlsl:117-220`, with a band builder at
  `BUI/gpu/intern/gpu_d3d11.c:1010-1101`): exact-coverage, resolution-independent text, from Eric
  Lengyel's MIT reference. At 12 to 14 pixels it can't match DirectWrite with Direct2D's contrast
  and gamma, and BUI's own test lets Slug text differ. Worth it only for text in 3D views, or text
  that scales.
- **Variable-height lists:** BUI's vlist keeps a Fenwick tree over blocks of 1,024 items
  (`BUI/bindings/rust/blenderui-vlist/src/index/fenwick.rs:1-12`), and scrolls 10M items in 4.8 us
  by its own benchmark. KGames' fixed-pixel layouts rarely need variable heights.
- **nanosvg** (`BUI/kit/extern/nanosvg`), if KGames ever needs SVG art.

## Where KGames is already ahead

- **Text:** DirectWrite gives KGames shaping, ligatures, font fallback, quarter-pixel glyph
  positions and Direct2D's exact gamma. BUI has kerning tables only, no shaping
  (`BUI/gpu/intern/gpu_raster.c:3083`), pens snapped to whole pixels
  (`BUI/kit/intern/kit_draw.c:1108`), and a plain `pow` gamma lift (`gpu_raster.c:3750`).
- **Drawing:** KGames draws the interface in one instanced draw a frame, and its shadows integrate a
  real Gaussian. BUI draws a few batches a frame (`gpu_d3d11.c:1437-1546`), with smoothstep
  shadows (`bui_ui.hlsl:358-370`).
- **Latency:** BUI presents with a sync interval of 0, no waitable swap chain, and a full-frame copy
  every frame (`gpu_d3d11.c:1585-1619`), which is worse than KGames' `Presenter`.
- **The law of the frame:** BUI's Rust bindings let effects land late. Virtualized lists build from
  the last frame's bounds (`BUI/bindings/rust/blenderui/src/element.rs:148-160`), `use_effect` runs
  after paint (`hooks.rs:124-145`), hover is worked out after the rebuild (`window.rs:1163`), and
  `BUI/bindings/node/signals.js:340-397` runs effects in later slices. BUI's C core does settle in
  the frame, with up to three layout passes (`kit_core.c:2239-2250`).
- **Testing:**
  - BUI's GPU parity test passes without a GPU, and lets 1% of text pixels differ.
  - Nothing tests BUI at 60 against 240 frames a second, or a cause's effects on its own frame.
  - CI turns off warnings as errors (`BUI/.github/workflows/ci.yml:45`).
  - The gpui benchmark takes focus (`BUI/bindings/rust/bench/gpui-bench/src/scene.rs:62`).
- **Benchmark records:** BUI's `results.md` disagrees with its README's table, which predates newer
  features and comes from one run. [Performance.md](Performance.md) records its date and commit.

## Skipped, and why

- **BUI's CPU rasterizer** (`BUI/gpu/intern/gpu_raster.c`, 4,300 lines): it draws a different,
  Blender-style shader, and WARP already covers machines without a GPU.
- **Blender's widget look, theme and icons** (`bui_ui.hlsl:234-278`, `kit_draw.c`,
  `kit_theme_presets.c`, `kit_icons_svg.c`, `tools/gen_kit_icons.py`): KGames follows Fluent 2, and
  its Segoe Fluent Icons are WinUI's own.
- **taffy layouts and `rsx!`:** KGames lays out in fixed pixels.
- **Panel animations:** KGames animates only what's integral.
- **The library's undo of widget values** (`kit_core.c:3516-3621`): in MVU, undo is program state.
- **The node editor:** no KGames program needs one.
- **The file browser** (`BUI/ui/space_file`): it reads directories synchronously, and has bugs with
  non-ASCII paths (`BUI/kit/intern/kit_filebrowser_paths.c:57-67`). MeshModeler's own Open dialog
  is enough.
- **The vendored C decoders** (Wuffs, libjpeg-turbo, libwebp): AGENTS.md says to use Win32, Rust,
  Direct3D 12 and DirectWrite, and WIC does the job (task 7).
- **The async image loader and the video clock:** they land results frames late (tasks 7 and 20).
- **Release tooling, features.json, doc images and CI** (`check_versions.py`, `bump_version.py`,
  `CHANGELOG.md`, `features.json`, `gen_doc_images.py`, `pngopt.py`, `.github/workflows`): KGames
  has no releases, one build system, no images in its docs, and no remote CI.
