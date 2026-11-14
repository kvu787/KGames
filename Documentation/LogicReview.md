# The logic review

After the move to MVU, every program's domain logic - not the GUI - was reviewed end to end:
Veehiicuul's driving, collision, curves, track shaping, hexels, document and files; MeshModeler's
editing and history with PolygonMesh and SimplePaint under it; GameInputDemo's input model;
UiShowcase's model; the two scene tests' motion math; and Library/UserInterface's frame pipeline
itself, against the law that an output delayed any number of frames past its cause is a bug. This
records what was found and fixed, and what was examined and found sound. Each fix landed with the
code in its own commit, most with a test that pins it.

## Found and fixed

**Library/UserInterface's frame pipeline**

- A widget that lost keyboard focus was told only when a press moved the focus. Esc, another
  widget's press or focus call, and `Context::focus` all slipped past, so a number box's typed
  draft never committed on those paths - and NumberBox itself reset the draft before the commit
  could read it, so even the told path committed the old value. The interface now remembers every
  focus leaver and dispatch tells each one, in the same frame, with a second pass for widgets the
  loop had already passed.
- A dialog's default button fired on Enter meant for a focused widget inside the dialog. It now
  fires only when nothing reachable takes Enter. Combo boxes also never actually opened from the
  keyboard: the press that opened the list chose from it in the same interact and closed it again.
- F10's own frame drew the PerfBar with the old VSync label, though the frame's present already
  honored the new setting - a one-frame delay in the library's own bar.
- The wheel that nothing under the pointer takes went to every containing scrollable, so a list
  and the area around it scrolled together; and a disabled widget still ate the wheel. It now goes
  only to the deepest widget that scrolls, and a disabled one doesn't count.
- A popup outlived a widget that changed kind under a kept name, capturing the keys until Esc; a
  target that stopped being focusable kept focus; `target()` resolved a duplicated name to a
  different widget than `state()` described; and a message chain past the frame's cap silently
  dropped everything queued behind it. All four are closed; the cap now stops the program, as a
  cascade that doesn't settle is a bug.

**Veehiicuul**

- A crash was detected once a frame and its pause anchored to the frame clock, so a run through a
  crash diverged between 60 and 240 Hz. The car then crashed on the very drive step that touched
  the edge, and the pause ran on the drive's clock. The drive's fixed steps were dropped later, as
  they made the car judder at frame rates that they don't divide, like 100: the car now moves by
  each frame's time, a crash is where it is at a frame's end, and the pause runs on the frames'
  clock, which a test checks at 60, 100 and 240 frames a second.
- Undo could strand the track builder's chosen curve past the islands that are left: adding an
  island, undoing it, and pressing Delete indexed out of bounds. The frame's last message now
  clamps the track's and the terrain's choices, as it already kept the scene's pick.
- Saving couldn't express a deletion: undoing a new scene after saving it left its file on disk
  and the status saying Unsaved forever. Save now deletes the files of scenes the document no
  longer has.
- Parked-vehicle names were compared with case, while `Vehicles::find` reads them without, so a
  hand-edited scene could dodge the delete guard or a rename and save a document that couldn't
  load.
- The track's problems now report an island inside an island, which the comment always promised.

**MeshModeler, PolygonMesh, SimplePaint**

- A keyboard command during a held Paint-tool stroke interleaved history steps: an Undo mid-stroke
  was silently re-applied by the stroke's own commit. A command now ends the stroke first, as its
  own step.
- `Document::save` wrote whatever the mesh held; positions driven to infinity serialized as JSON
  null and the saved file couldn't be reopened. Save now validates first and refuses loudly.
- A stale face index in the paint tool panics no more; a crease sharpness that underflows f32 is
  rejected at its own line instead of failing the whole file later; and a loop cut through an edge
  with more than two faces says no instead of leaving a T-junction.

**UiShowcase and GameInputDemo**

- The showcase's Watch message carried the theme it observed at input time - the stale-snapshot
  class - so a toggle earlier in the same frame made it double-log. It now reads the theme when it
  applies. The list's Select and Activate carried a row index resolved against apply-time search
  matches, so typing that re-filters the list in the same frame moved a press onto another item;
  they now carry the item the row showed, the gesture's own fact. A menu Reset kept its recorded
  choice. The input demo's combo-box pick logs at its frame's own time.

## Examined and sound

The reviews deliberately verified, and found correct: Veehiicuul's integration, dead zones,
rotation and collision geometry, B-spline sampling and insertion, hexel axial math, and the
camera's ground round trip; PolygonMesh's renumbering invariants, crease bookkeeping through
subdivision, Catmull-Clark's boundary and crease rules, picking's ray math, and file round trips;
SimplePaint's parameter validation and the impossibility of its divisions by zero; GameInputDemo's
focus masking, listen rules, timestamps and rumble; the scene tests' bounce and phase math; and
the frame pipeline's theme switching, popup layering, drag capture, and the fidelity of Headless
and scripts to the window's own loop.

## Still open, deliberately

- A second mouse button pressed on a view mid-drag overwrites the first press's tracking (the
  interface keeps one active press). Niche, and the fix would widen the interaction core; noted
  rather than taken.
- The reviews suggested more tests than were added: a property sweep of random mesh-edit sequences
  against `validate()`, a Headless test of GameInputDemo's focus-lost double update, and nested
  scrollables under the wheel. The fixes those would cover are in and pinned by their own tests;
  the sweeps remain good future hardening.

# What the rewrite in C++ found

KGames was rewritten in C++ once, file for file, to compare the two languages.
[CppVersusRust.md](CppVersusRust.md) has the comparison, and the C++ is on the `cpp-rewrite`
branch, whose copy of this document records the three adversarial reviews that the rewrite ended
with. Those read the C++ against the Rust, so what they found in the logic, and not in how C++
wrote it, was the Rust's too. This records what the Rust took from them, as above. Each fix has a
test that pins it.

## Found and fixed

- MeshModeler: an Undo or a Redo that took away the paint that was picked left it picked until the
  frame's end, so a second message in the same frame, an edit of the paint or a stroke with it,
  indexed past the paints and stopped the program. So did a pick of a row whose paint an Undo
  earlier in the frame took away. The pick is now kept among the paints as each of them applies.
- MeshModeler: the view drew the model it drew before after New or Open, until an edit, when the
  new document's version number was the old one's. A document counts its versions from 1, so that
  was any model opened before the first edit. What the view drew is forgotten with the document.
- `ListView` took a selection past its rows, which a program has when its list got shorter, as
  selected: it asked the program's labels for that row, which stops a program that indexes,
  activated it on Enter, and moved to the row before it on Up. It's no selection now.
- Veehiicuul: a point far enough away, or any point on a terrain of tiny cells, was in a cell at
  an `i32`'s end. The brush's and the fill's steps to the cells around it added past the end, and
  Save wrote every cell between a terrain's first and last, which for cells at both ends is rows
  that no disk holds. Cells now stop 1,024 from the origin: a point farther away is in the
  farthest cell, a brush that reaches past it leaves those cells unpainted, and a file with cells
  past it is refused.
- Library/PolygonMesh's merge by distance numbered the cube of a vertex far away with the most
  that an `i64` holds, and added one to it for the cubes around. GameInputDemo's scripted mouse
  added to running totals, and took differences of them, past an `i64`'s ends. Each stops a debug
  build and goes around in an optimized one. The cubes are clamped, and the totals wrap.

## Still open, deliberately

- A menu wider than its window, a menu's highlight past its items, and `DrawList::move_to_back`
  with marks out of order stop the program, in both languages. None can happen in the two window
  setups that programs support, and each is a mistake of the code that calls, which a stop says
  best.
- A program started with a show command of its launcher's, as Explorer and PowerShell's
  `Start-Process` give one, comes up at the maximized size without being maximized, and shows the
  error for an unsupported setup. Windows uses the launcher's command in place of the program's
  own for a process's first `ShowWindow`. `HumanRun` starts programs without one. It was found
  while measuring the programs in their windows, and fixing it needs a run that takes the focus,
  which only a person may start.

# What taking a frame's input in order found

A frame's input used to be folded into flags and totals, all hit-tested on the tree the user saw,
which lost the order it came in. It now comes in steps, one for each event that does something,
each handled on the state that the steps before it left (`Input::steps` and `Interface::frame`),
and the reader keeps the gamepad's changes, key repeats and unplugging in order. That change to
the frame machinery ended with an adversarial review of the new code. Each fix has a test that
pins it.

## Found and fixed

- Platformer was ticked in every step, and a step before a frame's last has no time. Its physics
  step of no time forgot the platform that the player stood on, so the frame's last step didn't
  carry them with it, and they slid off a moving platform in any frame of more than one step. A
  step of no time now moves nothing, and AGENTS.md says that a tick of no time must move nothing.
- The reader made a reading's key presses in the order of their virtual keys, so a Ctrl+Backspace
  that one reading had was a Backspace and then a Ctrl, which deleted a character in a text field.
  Shift, Ctrl and Alt now go down first and come up last.
- GameInputDemo read the devices in every step, and they hand out a frame's readings once, so the
  first step took them all with its focus: a frame with a key's press and then a loss of focus
  showed readings that came without focus. `Ui::last_step` lets a program do what it does once a
  frame in the frame's last step, and the demo reads the devices then.
- `Input::steps` took any text after a key's press as the key's, so a script's Enter and then
  typed text in one frame went together, and a number box took the text with the Enter. A key's
  press and its text were two steps then, as Win32's `WM_KEYDOWN` and `WM_CHAR` are two messages.
  Since then, the text is made from the key, so they're one step, and Enter's own step comes
  before the next key's.
- Comments in the library and the programs still said "once a frame" or "the tree the user saw"
  where it's now a step's.

## Examined and sound

`Input::steps`' edge cases: frames that are empty or only moves, moves after the last event, and
the gamepad's repeated readings; focus lost and got back in one frame; which of the interface's
state is reset once a frame and which once a step; F9, F10 and F11 pressed twice in a frame, which
cancel; drags, whose moves add up over the steps to the frame's; the widgets, which keep nothing
per call that steps would break; MinimalSceneTest, SpinningCarTest, Veehiicuul, SimplePaintDemo,
UiShowcase and MeshModeler with steps of no time; Headless and scripts, which make the same steps
as the window; and the reader's repeats, gamepad changes and unplugging.

## Changed on purpose

- A press after a letter typed in the same frame lands on the list as the letter filtered it. The
  review after the move to MVU, above, made it land on the list the user saw. Each event now sees
  what the events before it did, as a Win32 window's messages do, and the message still carries
  the item that the row showed, in the tree that the press was on.

## Still open, deliberately

- Each step runs the program's `view` and the layout, and Veehiicuul's and MeshModeler's
  `Prepare`, so a frame of many steps, like a burst of wheel readings or key repeats after a stall,
  costs that many times as much. View and layout cost about 0.04 ms a step in UiShowcase, and most
  frames are one step.
- The steps before a frame's last are at the last frame's time, so a press in one of them records
  its time up to a frame early, for a double-click and a tooltip's wait.
- The cursor is read once a frame, as Windows keeps no cursor on GameInput's clock, so a press
  lands where the cursor is as the frame reads its input, and not where it was as the button went
  down. `GetMouseMovePointsEx`, which keeps the cursor's last 64 positions with their times in
  milliseconds, could place it closer.

# What walking GameInput's stream in its order found

The frames read GameInput's readings by walking its stream a kind at a time, merging the three walks
with a reading callback's queue by sorting on timestamps, and taking the readings that shared a
timestamp and a device as one. GameInput says that a timestamp can't tell readings apart, and its
stream already has every device's readings in the order the input happened in. Each frame now
walks the stream once, in its order, as Microsoft's documentation and its GameInputSequential
sample do, with no callback, no sorting and no timestamp key, and the reader keeps each gamepad's
own state. Two adversarial reviews followed: one of the GameInput side, against Microsoft's
documentation, and one of the interface side, the steps, the widgets and the programs. Each fix
has a test that pins it.

## Found and fixed

- A frame that lost its place in the stream, and the first frame, handed out only the newest
  reading, so every other device kept what it held: a key let go of in the readings that left the
  history stayed held, and repeated, until its keyboard read again. Microsoft's sample walks one
  device, and starts that one over; the walk now starts every device over from its newest reading,
  in the stream's order as far as the history goes.
- The reader kept one gamepad, which every gamepad's reading replaced, so a gamepad at rest whose
  sticks read thousands of times a second covered the one in use. Each keeps its own state now.
- The interface takes the gamepad event right after focus comes as what's held then, and the
  reader sent none, so a press after focus came, in that frame, counted only once let go of and
  pressed again.
- The reader's keys and mouse buttons now go with focus, and a gamepad is connected from when it
  connects, rather than from its first reading.
- GameInputDemo took the frame's focus in its last step, where a frame that lost focus and got it
  back looked as if nothing happened, and timed getting focus at the frame's end. Devices now give
  the window's focus changes on GameInput's clock, and the demo puts them among its readings.
- The message cap counted a frame's steps together, so a frame of many settling steps stopped the
  program as a chain without end. It counts each step's chain.
- NumberBox and ComboBox rounded each step's wheel, so a wheel that turns by less than a notch at a
  time, a step each, never moved them. They keep the remainder, as Slider did.
- A text field scrolled to its caret only as it was drawn, so a press after typing in one frame
  placed the caret on last frame's scroll. It scrolls as each step's tree is laid out.
- MeshModeler applied a modal edit again in every step, each a new document version that remade
  the subdivided surface, for the same pointer. It applies it again only when its inputs change.
- The cursor came from the tree that a step's input came on, so a press that put a text field under
  a still pointer showed the old cursor for a frame. Widgets now say their cursor, which the
  interface takes from the drawn tree.

## Examined and sound

The COM objects' ownership and identity, the walk's ends and its kind filter, a reading with
several kinds of input, devices that disconnect within a frame, the wheel's running totals across a
lost place, modifiers in one reading, `ToUnicodeEx` on the main thread, the conversion between
GameInput's clock and the performance counter, the steps' edge cases, which state is per frame and
which per step, F9, F10 and F11 pressed twice in a frame, drags over many steps, the widgets'
memory, the programs' ticks of no time, Veehiicuul's commits per step, and the fidelity of Headless
and scripts to the window.

## Decisions left to the user

- **How several gamepads combine:** a button held while any gamepad holds it, a trigger as far as
  any pulls it, each stick the one pushed furthest. The mask of buttons held as focus came is on
  the combination, so a button that one gamepad held then doesn't count from another until both
  let go.
- **The time of a frame's steps before its last:** decided. Every step sees the frame's time, and
  only the last has the frame's delta, so that a simulation moves once a frame, by `ui.delta()`.
  A press gets the frame's time wherever it is in the frame, for a double-click's and a tooltip's
  timing.
- **What a frame of many steps costs:** each step runs `view`, the layout and the programs' messages,
  like Veehiicuul's and MeshModeler's `Prepare`, so a burst of wheel readings, or a held key's
  repeats after a stall, costs that many times. Merging wheel events that nothing separates would
  cut it, at the cost of a notch that scrolls a widget under the pointer no longer going to it.
- **A held key's repeats:** decided. GameInput doesn't repeat a held key, its keyboard readings
  being the keys held down, which come only when they change, so KGames no longer makes repeats
  of its own, which every stall would have handed out at once, and which could come after a
  release that GameInput hadn't delivered yet. They can come back when they're wanted.
- **When focus changes:** at the time the window handles `WM_SETFOCUS`, after GameInput's own switch,
  so input in between counts as without focus. Since superseded: when the main loop sees the
  foreground change, as the section on simplifying the window, focus and the cursor says.
- **The pointer:** decided. It's KGames' own, as a game's is: GameInput's mice move it, in the
  stream's order, so a press lands where it was as the button went down, and a key after a move
  sees it where it went. While the window has focus, Windows' cursor is hidden and kept inside it, and the
  interface draws the pointer, a frame's input latency behind the mouse, where Windows' own cursor
  is drawn by the hardware. The window starts to keep the cursor only once no mouse button is
  held, by Windows' account, which has the click that gave it focus, so that a click on the title
  bar's minimize or close button works. Nothing reads where Windows' cursor is. The review below
  settled the rest.

## Still open

- On the user's devices: whether GameInput reports the Left Ctrl that Windows adds for AltGr; what
  GameSir's controllers in Xbox 360 mode report as their kinds of input; what `GetNextReading` does
  with a last reading whose device disconnected; and how often `Devices::restarts` counts, like
  after an Alt+Tab back.
- GameInputDemo's log merges a run of motion only with its newest line, so two devices moving at
  once cut each other's runs short, as they did before the stream walk.

# What reviewing the pointer against established practice found

KGames draws its own pointer, at a position of its own that GameInput's mice move, and hides and
confines Windows' cursor while the window has focus. The review compared it with Microsoft's
documentation (SetCursor, WM_SETCURSOR, ClipCursor and GetClipCursor, "Taking Advantage of
High-Definition Mouse Movement", and GameInput's mouse state), with Microsoft's DirectXTK `Mouse`,
and with SDL 3's and Godot 4.8's Windows code. Neither engine draws a cursor of its own on Windows.
The closest practice is SDL's relative mode: a position of its own, kept in the window, that raw
motion moves, unscaled by default. Microsoft's documentation has a pointer that a menu uses follow
Windows' cursor, with the user's speed and acceleration, and raw motion move a camera.

## Found and fixed

- The window kept the cursor as it got focus, while a click on the title bar's minimize or close
  button that gave it focus was still held, so that click was never let go of on the button. It
  keeps it only once no mouse button is held, as SDL does.
- A minimized window that got focus kept the cursor in its client area, which is off the screen
  then.
- A window can get focus without being the foreground window, as SDL's code notes, and it kept the
  cursor then. It keeps it only while it's the foreground window.
- The window released the cursor's confinement as it lost focus whoever held it, and the program
  that got focus may have confined it first, as the two hear of the change in no fixed order. It
  releases only its own, saving the confinement before it confines, as Microsoft's documentation
  has a program do, and moves the cursor to its middle only if no other program holds one.
- A frame drew the pointer whenever the window had focus, while the window hid Windows' cursor only
  once it kept it, after the frame was presented, so a frame could show both or neither. The
  window keeps the cursor, or not, as the frame reads its input, and the frame draws the pointer
  exactly when it does.

## Examined and sound

Hiding the cursor with SetCursor(NULL) in WM_SETCURSOR over the client area, with no class cursor,
as Microsoft's documentation, SDL and Godot do; the arrow over the frame; confining to the client
area on the screen, and again as it moves or changes size; the pointer clamped to the client area,
as SDL's is; the pointer drawn last, over everything, from the frame's newest input; and the click
that gives the window focus doing nothing in the program, which GameInput doesn't report.

## Decisions left to the user

- **How far the pointer moves:** decided. A mouse's counts times `MOUSE_SENSITIVITY`, a constant
  in `input/reader.rs`, 0.8, without acceleration. A port of SDL's emulation of Windows' pointer
  speed and "Enhance pointer precision" came and went, and then a setting in `Settings.json`: a
  constant is simpler.
- **Where Windows' cursor shows as the window loses focus:** decided, as SDL does. Where the
  pointer was. It was the middle of the window.
- **Where the pointer starts as the window gets focus:** decided, as SDL does. In the middle the
  first time, and where it was after that. It was the middle each time.
- **Where the pointer is:** decided. Anywhere, in parts of a pixel, as the scaled motion moves it,
  and drawn there with antialiased edges, not on the pixels' grid.
- **Devices with absolute positions,** like pens, touch screens, Remote Desktop and virtual
  machines, which GameInput gives as `absolutePositionX` and `absolutePositionY`, in units that its
  documentation doesn't give: they don't move the pointer, which the user doesn't need yet. SDL
  handles each.
- **How the pointer looks:** decided. Unlike Windows' cursors, so that the two are easy to tell
  apart: in the theme's accent with an edge in `text_on_accent`, and its arrow a dart. Its size is
  fixed, where Windows' cursor follows the size that the user sets in its accessibility
  settings.

# What simplifying the window, focus and the cursor decided

The user narrowed what programs support, to make the window, focus and Windows' cursor simpler and
sound, and asked for the cursor's transitions to be right above all.

## Measured

A C probe, outside the repository, confined the cursor from one process and changed the foreground
from another, with a real Alt+Tab, then read `GetClipCursor`, on Windows 11 26300:

| What happened while a process confined the cursor                       | The confinement after |
| ----------------------------------------------------------------------- | --------------------- |
| Alt+Tab, while the process handled its messages                         | released by Windows   |
| Alt+Tab, while its thread was stuck, handling none                      | released by Windows   |
| Another window brought to the front without Alt+Tab, thread stuck       | released by Windows   |
| The Windows key opened the Start menu                                   | still confined        |
| The process exited while the Start menu had the foreground              | still confined        |
| A process that wasn't the foreground window confined it                 | confined              |

So Windows releases a confinement when a normal window is activated, whatever the process that set
it is doing, but not for the Start menu, and not when that process ends; and it doesn't stop a
program in the background from confining the cursor.

## Decided

- **One window setup:** borderless fullscreen on a 2560 × 1440 display at 100%. The windowed
  maximized setup, F11, the `WindowMode` setting and `--window-mode` went.
- **Windows' own window shortcuts do nothing:** the window swallows the system keys' messages, so
  Alt and F10 don't open the menu's keyboard mode, Alt+Space doesn't open the window's menu, and
  Windows doesn't close it on Alt+F4. Alt+F4 comes from GameInput's keys, and asks
  `Program::may_close` in the step with the key, so that a prompt about unsaved work is in that
  frame. The shortcuts that Windows handles before any window sees them, like Alt+Tab and the
  Windows key, stay Windows'. The window's `WM_CLOSE` stays as it was, so the taskbar's Close works.
- **Focus:** the window has focus while it's the foreground window and isn't minimized, which the
  main loop checks on every pass, whatever woke it, and not from Windows' focus messages, which a
  window can get without being the foreground window. A focus change is timed when the check finds
  it.
- **Windows' cursor:** `platform/cursor.rs` decides what the window does with it, on facts that the
  window gathers on each check, and its tests run whole sequences. The window keeps the cursor,
  hidden, while it has focus, once no mouse button is held; confines it again when something else
  changed its confinement; and lets it go as focus goes and as the window closes, releasing only its
  own confinement, and putting the cursor where the pointer was unless another program confined it
  or the window is minimized. There's no thread of its own for this: if the main thread is stuck,
  the cursor stays confined until it recovers, or a normal window is activated, like with Alt+Tab,
  or Ctrl+Alt+Del.
- **Minimized:** the main loop runs no frames and waits for messages; the program hears what came
  meanwhile, like losing focus, in the first frame after the window is restored. GameInput turns
  gamepads' motors off without focus itself, and back on as the program last asked when focus comes
  back.
- **The pointer:** a mouse's counts times `MOUSE_SENSITIVITY`, 0.8. Each mouse starts over as
  focus goes, and only readings with relative positions move the pointer: the section after this
  one says why.

## Only the user can check

Tests never take focus, and Windows acts on a window's system keys only while it has focus, so
these aren't machine-checked, and the user tries them by hand:

- Alt+Tab out and back: the cursor shows where the pointer was, and is confined and hidden again
  on return.
- The Windows key: the Start menu opens, and the cursor is free.
- Minimizing and restoring, like with Win+D.
- Alt+F4 closes, and MeshModeler with unsaved changes asks first; the taskbar's Close too.
- Alt+Space and Alt alone do nothing.
- The pointer's motion and the wheel right after Alt+Tab back, with a real mouse, which the
  checks with injected input in the section after this one can't stand for: that a real mouse's
  readings, the wheel's too, have relative positions, as injected ones do.

# What validating with real input found

With the user away, and their leave to disturb the desktop, the release UiShowcase ran fullscreen on
CVVNY2, in front, while a C program outside the repository sent it real input with `SendInput`,
changed the foreground, and read what Windows said: the foreground, `GetClipCursor`,
`GetCursorInfo` and `GetGUIThreadInfo`'s menu modes. Another printed every reading that GameInput
gave a window of its own. Three adversarial reviews, which only read, went over the window, focus
and the cursor, the input path from GameInput's stream to the steps, and the docs and comments
against the code.

## Measured

| What the probe did                                  | What happened                                         |
| --------------------------------------------------- | ----------------------------------------------------- |
| Brought the window to the front                     | the cursor confined to the window, and hidden         |
| Alt+Tab                                             | the cursor free and showing, where the pointer was    |
| The Windows key, then Escape                        | free with the Start menu up, kept again after         |
| Win+D, then brought the window back                 | free while minimized, kept again once restored        |
| Alt+Space, and Alt alone                            | no menu mode, and no menu                             |
| Posted `WM_CLOSE`, as the taskbar's Close does      | the program closed, and the cursor was free           |
| Alt+F4 with `SendInput`                             | nothing: GameInput doesn't report injected keys       |

GameInput 4.0.306 did five things that its documentation doesn't say:

- As focus goes, it gives each mouse a reading with no positions, `GameInputMouseNoPosition`, and
  totals and wheels of 0, whatever they were.
- It goes on adding a mouse's motion and wheel turns up while the program doesn't have focus, and
  its first reading after focus comes back has them: totals of (200, 150) and a wheel of 240, then
  (30, 0) and 120 in another program, then (-100, 0) with focus, read (130, 150) and 360.
- `SendInput`'s mouse input is one device, "Unidentified mouse (0000:0000)", which connects with
  the first input sent, and the reading of that input never comes: its first reading in the stream
  has the totals of the first two moves.
- Until the process's first reading, `GetCurrentReading` gives a reading that isn't in the history,
  a snapshot of a device at rest: `GetNextReading` and `GetPreviousReading` from it fail with
  `GAMEINPUT_E_REFERENCE_READING_TOO_OLD`.
- Its readings at rest as focus goes are stamped about 1.4 ms after `GetForegroundWindow` says
  that the window lost it, as a probe that checked without sleeping saw, three times out of three.

`ClipCursor` keeps a confinement within the virtual screen, here (-2560, 0) to (5120, 1440), as a
probe that confined the cursor and read `GetClipCursor` back saw:

| Confined to                       | `GetClipCursor` gave                     |
| --------------------------------- | ---------------------------------------- |
| (-2560, 0) to (0, 1440)           | the same                                 |
| (4000, 0) to (6560, 1440)         | (4000, 0) to (5120, 1440)                |
| (-2560, 500) to (0, 1940)         | (-2560, 500) to (0, 1440)                |
| (-3000, -100) to (-500, 1000)     | (-2560, 0) to (-500, 1000)               |
| (100, 100) to (100, 100), empty   | the same: the cursor pinned to a point   |
| (100, 100) to (50, 50), inverted  | the virtual screen: no confinement       |

The main loop checks focus and the cursor on every pass, which spins without vertical sync. Timed
a million times each, `GetForegroundWindow` and `IsIconic` took 2 to 3 ns, `GetClientRect` and
`ClientToScreen` 8 ns, the four `GetSystemMetrics` 270 ns, and `GetClipCursor` 1.1 µs, so a pass
with focus costs about 1.4 µs: 0.4% of a core at 3,000 passes a second.

After the fixes below, the checks of the first table ran again on the release UiShowcase, with the
same results.

## Found and fixed

- **The pointer jumped, and the wheel turned, around Alt+Tab.** The reader took a mouse's motion and
  turns from its last reading, which focus didn't change, so the reading at rest as focus went
  moved the pointer back by the mouse's totals and turned the wheel back by its wheels' totals, and
  the first reading after focus came back moved and turned it forward again, by everything since,
  in the other program too. Now each mouse starts over as focus goes, and a reading without
  relative positions neither moves the pointer nor turns the wheel. The probe then saw the pointer
  carry on from where it was after Alt+Tab, and Windows' cursor placed where the pointer was drawn.
  Pinned by `reader.rs`'s `a_mouse_starts_over_across_focus_changes` and
  `a_reading_with_only_absolute_positions_moves_nothing`.
- **A walk that lost its place lost the taps that the history still had.** It handed out only each
  device's newest reading, so after a frame longer than the history lasts, a key tapped in its last
  70 ms was lost though GameInput still had it, and the first frame with input after the program
  started, whose walks start from GameInput's snapshot outside the history, kept only each device's
  newest. Now it hands out every reading in the history after the last one handed out. Pinned by
  `a_tap_still_in_the_history_counts_after_losing_the_place` and
  `the_walk_starts_from_the_state_at_rest_that_gameinput_gives_before_any_reading`.
- **A key right after focus came saw no pointer.** The reader told the interface where the pointer
  was only at the frame's end. Pinned by `a_key_right_after_focus_comes_sees_the_pointer`.
- **A gamepad that connected and then changed in the same frame wasn't connected.** The reader
  went by each device's last event of any kind, like an XInput index update. Pinned by
  `a_device_s_last_connection_in_a_frame_counts`.
- **The steps after an Alt+F4 that closed still ran,** and could change what the program closed
  with without asking again. Pinned by `close.rs`'s `nothing_happens_after_an_alt_f4_that_closes`.
- **Veehiicuul drove the car in a tick of no time,** every step of a frame but its last, and its
  first frame back in Play from an editor moved by the time away, up to `LONGEST_FRAME`. This
  document's first sections had found its ticks sound. Pinned by `a_tick_of_no_time_moves_nothing`
  and `the_first_frame_back_in_play_moves_by_its_own_time`.
- **A client area past the virtual screen's edge never held its own confinement,** by the table
  above, so the window confined the cursor again on every pass, and left it confined as focus
  went. An empty client area pinned the cursor to a point, hidden. A confinement left from before
  the window kept the cursor made Windows' release read as another program's, so the cursor wasn't
  placed where the pointer was. And scripted runs, and runs without input, kept the cursor when
  their window came to the front. Pinned by `cursor.rs`'s
  `a_client_area_past_the_screens_edge_keeps_its_part_on_them`,
  `an_empty_client_area_keeps_nothing`, `a_confinement_left_from_before_is_not_nobody_s`,
  `a_release_is_to_the_screens_as_they_are` and `without_mice_the_cursor_is_left_alone`.
- **A window moved off its display had no error page,** as the setup check compared sizes only.
  Pinned by `window.rs`'s `a_window_moved_off_its_display_says_so`.
- **The rendering tests failed now and then,** as a debug build's `CreateDXGIFactory2` with its
  debug layer failed with `DXGI_ERROR_INVALID_CALL` while another test's thread made a device: 3 of
  30 runs failed, and none of 40 once Graphics made devices one at a time. The rendering tests are
  the reproduction; stress tests that made devices on many threads at once never failed.
- **Docs and comments** that the code no longer kept true, in 20 places, most from before the
  simplification of the window.

## Decisions left to the user

- **Where Windows' hidden cursor is kept.** In the whole client area, where it goes its own way,
  apart from the pointer, so a window that's always on top over the client area, like Task
  Manager's, gets the clicks and the wheel under it, and shows Windows' arrow there. SDL's
  relative mode keeps it in one pixel in the window's middle by default, where that's much rarer.
- **Alt held from before focus, then F4.** GameInput doesn't report keys held as focus comes, so
  Alt+Tab in with Alt still held, then F4, doesn't close the program, which Windows' own Alt+F4
  would. Since then, Alt+F4 is only two keys, and F12 quits.
- **Closing on the error page, or while minimized.** A program with unsaved work that says no to
  Alt+F4 or the taskbar's Close sets its prompt, which only shows once the setup is right or the
  window is restored, so it can't be closed until then. Since then, F12 and the taskbar's Close
  quit at once, without asking, and `Program::may_close` is gone.
- **The click that gives focus, while it's held.** The window is focused, but doesn't keep the
  cursor yet, so Windows' arrow shows while the reader's pointer, not drawn, moves, and what's
  under it is hovered; once let go of, the pointer shows where the reader has it, not where the
  click was. Since then, the mice's input reaches the interface only while the window keeps the
  cursor, so the pointer stays where it was until then.
- **A gamepad button held as focus comes.** The reader ignores it until it's let go of only if
  GameInput's reading of it as focus comes is stamped before the window's focus change, and its
  readings at rest as focus goes came 1.4 ms after. If a real gamepad shows the same as focus
  comes, a button held through Alt+Tab counts as a press once back. The fix would ignore the
  buttons held in each gamepad's first reading after focus comes, if GameInput always sends one.
- **Caps Lock and Num Lock** came from Windows' state as of the frame's messages, not from where
  each key comes in the stream, so a frame that typed a letter and then pressed Caps Lock typed it
  in the new case. Since then, typed text is ASCII from a US keyboard's table, without Caps Lock,
  so nothing is left to decide.

## Still open

- Whether `GetForegroundWindow` can give no window for a moment while the window stays in front,
  which would let the cursor go and keep it again: not seen.
- Whether releasing the confinement can fail, like as the secure desktop or the lock screen comes,
  which nothing tries again: not measured, as agents don't lock the machine.
- Whether the window stays the foreground window when another virtual desktop is shown with
  Win+Ctrl+arrows, which would keep the cursor confined there: not measured, as it would change
  the user's desktops.

## Only the user can check, after this

- A gamepad button held through Alt+Tab out and back: whether it counts as a press once back.
- A real mouse's motion and wheel right after Alt+Tab back, and the laptop's touchpad, which
  GameInput should give as a mouse with relative positions.
- Alt+F4 closes, and MeshModeler with unsaved changes asks first.
