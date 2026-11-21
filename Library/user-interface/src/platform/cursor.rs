// Windows' cursor, while the interface has a pointer of its own, as games
// do. What the window does with it is decided here, from facts that
// platform/window.rs gathers from Windows, as actions that it carries out,
// so that every transition is tested without a window that has focus
// (Test/user-interface/tests/cursor.rs).
//
// The cursor has two modes, which Ctrl+Shift+Alt+F11 switches between (see
// input/reader.rs), whether the window keeps the cursor or not:
//
// - GameInput's, which a program starts in: GameInput's mice move the
//   interface's pointer (see input/events.rs), which the frames draw, and
//   Windows' cursor is hidden, out of the way.
// - Windows': Windows moves its cursor, with its own speed and
//   acceleration, and draws it over the frames, as its hardware cursor, in
//   the shape that the widget under the pointer asks for, and the interface's
//   pointer follows it, undrawn (see platform/window.rs). The mice's buttons
//   and wheels still come from GameInput.
//
// In both:
//
// - While the window has focus, and its frames read GameInput's mice, which
//   the run decides once, it keeps Windows' cursor inside its client area,
//   hidden in GameInput's mode, and its frames draw the pointer exactly
//   while it keeps it in that mode. A run whose input is a script's, or that
//   has none, leaves the cursor alone.
// - It starts to keep it at once if a click on the window gave it focus, as
//   the button goes down, while it's held, which Windows says by asking the
//   window with WM_MOUSEACTIVATE whether the click activates it. Otherwise
//   it starts only once no mouse button is held, by Windows' account of the
//   mouse, which, unlike GameInput's, has a click that's still going on
//   elsewhere as the window gets focus, like a drag in another program, or
//   one onto the window's taskbar button, which brings it to the front, that
//   the cursor mustn't be kept from.
// - As it starts to keep it, the interface's pointer is placed where
//   Windows' cursor is, so that it shows where the user clicked, or left the
//   cursor, and not where it was as focus last went. If Windows' cursor is
//   outside the area that it's confined to, like on another display as
//   Alt+Tab gives the window focus, the pointer is placed in the middle of
//   the client area instead, where it starts with the mice's first event
//   too, and not at the area's edge, where the confinement puts Windows'
//   cursor, hidden, or in Windows' mode, shown, so that it's put in the
//   middle too.
// - Windows keeps a confinement within the screens, the virtual screen,
//   which GetClipCursor gives without one, so the window confines the cursor
//   to the part of its client area that's on the screens, and keeps nothing
//   if none of it is, as a window with an empty client area has none.
// - The cursor's confinement is shared by every program, and a program in
//   the background can confine it too, so the window confines it again
//   whenever something else changed it while it keeps it, and releases it
//   only if it still holds it: the program that gets focus may have taken
//   it first. Windows releases a confinement itself when a normal window is
//   activated, like with Alt+Tab, but not when the Start menu opens, or when
//   the program that set it ends, so the window releases its own.
// - As focus goes, it lets the cursor go where the interface's pointer was,
//   where it shows, as SDL does, unless another program has confined the
//   cursor by then, or the window is minimized, and has no client area on
//   the screen. In Windows' mode, it lets it go where it is, which the
//   pointer followed, and where it shows already. As the window closes, the
//   same.
// - Switching to Windows' mode while the window keeps the cursor shows it
//   where the pointer is, which it went its own way from while it was
//   hidden, and switching to GameInput's hides it, and the pointer goes on
//   from where it is. Switching while the window doesn't keep it only
//   changes how the window keeps it next.

use windows_api_raw::{POINT, RECT};

// Which cursor points while the window keeps Windows' cursor: the pointer
// that the interface draws, which GameInput's mice move, with Windows'
// cursor hidden, or Windows' own cursor, which Windows moves and draws as
// its hardware cursor, and which the interface's pointer follows.
// Ctrl+Shift+Alt+F11 switches between them (see input/reader.rs). A program
// starts in the one that its Cursor setting says, the interface's pointer
// by default.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CursorMode {
    #[default]
    GameInput,
    Windows,
}

impl CursorMode {
    pub const ALL: [CursorMode; 2] = [CursorMode::GameInput, CursorMode::Windows];

    // As the Cursor setting says it.
    pub fn name(self) -> &'static str {
        match self {
            CursorMode::GameInput => "GameInput",
            CursorMode::Windows => "Windows",
        }
    }

    pub fn other(self) -> Self {
        match self {
            CursorMode::GameInput => CursorMode::Windows,
            CursorMode::Windows => CursorMode::GameInput,
        }
    }
}

// What Windows says, each time the window checks its focus.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CursorFacts {
    // Whether the window has focus: it's the foreground window, and isn't
    // minimized.
    pub focused: bool,
    // The client area, in the screen's pixels, or none while the window is
    // minimized, whose client area is off the screen then.
    pub area: Option<RECT>,
    // Whether a mouse button is held, by Windows' account of the mouse.
    pub button_held: bool,
    // Whether a click on the window gave it focus since the last check: the
    // button held, if one is, is that click's.
    pub clicked: bool,
    // Where Windows' cursor is, in the screen's pixels.
    pub cursor: POINT,
    // The cursor's confinement, as GetClipCursor gives it.
    pub clip: RECT,
    // The screens, the virtual screen: the confinement without one, and what
    // Windows keeps one within.
    pub screens: RECT,
}

// What the window does with Windows' cursor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CursorAction {
    // Confines it to the area, with ClipCursor.
    Confine(RECT),
    // Releases its confinement.
    Release,
    // Hides it, at once, and over the client area from then on.
    Hide,
    // Shows it over the client area again, at once while the window keeps
    // it: in the pointer's shape while the window keeps it in Windows' mode,
    // and as the arrow otherwise.
    Show,
    // Puts it there, in the screen's pixels, with SetCursorPos.
    Place(POINT),
    // Puts the interface's pointer there, in the client area's pixels, as
    // the mice's input starts: where Windows' cursor is, or the middle.
    PlacePointer(POINT),
}

// Who holds the cursor's confinement as the window lets the cursor go.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClipHolder {
    // The window, which releases it.
    Window,
    // Nobody: Windows released it, as it does when a normal window is
    // activated.
    Nobody,
    // Another program, which confined it as it got focus, before the window
    // found that it lost focus.
    Other,
}

// Who holds the confinement `clip` when the window kept the cursor in
// `kept`, with the screens as they are now.
pub fn clip_holder(clip: RECT, kept: RECT, screens: RECT) -> ClipHolder {
    if clip == kept {
        ClipHolder::Window
    } else if clip == screens {
        ClipHolder::Nobody
    } else {
        ClipHolder::Other
    }
}

// The part of `area` on the screens, which a confinement to `area` is, or
// none if it's empty.
pub fn on_screens(area: RECT, screens: RECT) -> Option<RECT> {
    let part = RECT {
        left: area.left.max(screens.left),
        top: area.top.max(screens.top),
        right: area.right.min(screens.right),
        bottom: area.bottom.min(screens.bottom),
    };
    (part.left < part.right && part.top < part.bottom).then_some(part)
}

// The middle of `area`, in the screen's pixels.
fn middle(area: RECT) -> POINT {
    POINT {
        x: area.left + (area.right - area.left) / 2,
        y: area.top + (area.bottom - area.top) / 2,
    }
}

// The pixel of `part` nearest to `at`, in the screen's pixels.
fn nearest(at: POINT, part: RECT) -> POINT {
    POINT {
        x: at.x.clamp(part.left, part.right - 1),
        y: at.y.clamp(part.top, part.bottom - 1),
    }
}

// What the window does with Windows' cursor, and what it has done.
#[derive(Clone, Debug)]
pub struct CursorKeeper {
    // Whether the frames read GameInput's mice, which move the pointer, so
    // that the window keeps the cursor while it has focus: not a run whose
    // input is a script's, or that has none.
    mice: bool,
    // Where the window keeps the cursor, in the screen's pixels.
    kept: Option<RECT>,
    // Where the interface's pointer last was, in the client area's pixels.
    pointer: Option<POINT>,
    mode: CursorMode,
}

impl CursorKeeper {
    // A keeper for a run whose frames read GameInput's mice, or don't, which
    // keeps nothing yet, in GameInput's mode.
    pub fn new(mice: bool) -> Self {
        Self {
            mice,
            kept: None,
            pointer: None,
            mode: CursorMode::default(),
        }
    }

    // Whether the window keeps Windows' cursor: hidden in GameInput's mode,
    // so that its frames draw the pointer, and shown in Windows'.
    pub fn keeps(&self) -> bool {
        self.kept.is_some()
    }

    pub fn mode(&self) -> CursorMode {
        self.mode
    }

    // Switches the cursor's mode, with what Windows says now: while the
    // window keeps the cursor, shows it where the pointer is, or the middle
    // of the client area if it was nowhere yet, in the area that it's kept
    // in, or hides it. Otherwise only how the window keeps it next changes.
    pub fn set_mode(&mut self, mode: CursorMode, facts: CursorFacts) -> Vec<CursorAction> {
        if std::mem::replace(&mut self.mode, mode) == mode {
            return Vec::new();
        }
        let Some(kept) = self.kept else {
            return Vec::new();
        };
        match mode {
            CursorMode::GameInput => vec![CursorAction::Hide],
            CursorMode::Windows => {
                let mut actions = vec![CursorAction::Show];
                if let Some(area) = facts.area {
                    actions.push(CursorAction::Place(nearest(self.pointer_on_screen(area), kept)));
                }
                actions
            }
        }
    }

    // Where the pointer is, in the screen's pixels, with the client area
    // there, or the area's middle if it was nowhere yet.
    fn pointer_on_screen(&self, area: RECT) -> POINT {
        match self.pointer {
            Some(pointer) => POINT {
                x: area.left + pointer.x,
                y: area.top + pointer.y,
            },
            None => middle(area),
        }
    }

    // Where the interface's pointer is, in the client area's pixels, where
    // Windows' cursor shows as it's let go of.
    pub fn set_pointer(&mut self, at: POINT) {
        self.pointer = Some(at);
    }

    // What to do with the cursor, on what Windows says now.
    pub fn update(&mut self, facts: CursorFacts) -> Vec<CursorAction> {
        let keep = facts.area.filter(|_| self.mice).and_then(|area| Some((area, on_screens(area, facts.screens)?)));
        let mut actions = Vec::new();
        match (facts.focused, keep) {
            (true, Some((area, keep))) if self.kept.is_some() || !facts.button_held || facts.clicked => {
                if facts.clip != keep {
                    actions.push(CursorAction::Confine(keep));
                }
                if self.kept.replace(keep).is_none() {
                    // Where Windows' cursor is, or the middle if it's
                    // outside, as far as that's on the screens, where
                    // Windows' cursor goes too in Windows' mode, where it
                    // shows.
                    let inside = nearest(facts.cursor, keep) == facts.cursor;
                    let at = if inside { facts.cursor } else { nearest(middle(area), keep) };
                    match self.mode {
                        CursorMode::GameInput => actions.push(CursorAction::Hide),
                        CursorMode::Windows if !inside => actions.push(CursorAction::Place(at)),
                        CursorMode::Windows => {}
                    }
                    actions.push(CursorAction::PlacePointer(POINT {
                        x: at.x - area.left,
                        y: at.y - area.top,
                    }));
                }
            }
            (true, Some(_)) => {}
            _ => actions = self.let_go(facts),
        }
        actions
    }

    // Lets the cursor go, if the window keeps it, as focus goes or the window
    // closes: releases the confinement if the window still holds it, and in
    // GameInput's mode, shows the cursor and puts it where the pointer was,
    // or in the middle of the client area if it was nowhere yet, on the
    // screens, unless another program has confined it, or there's no client
    // area on the screens. In Windows' mode, it shows already, where it is.
    pub fn let_go(&mut self, facts: CursorFacts) -> Vec<CursorAction> {
        let Some(kept) = self.kept.take() else {
            return Vec::new();
        };
        let holder = clip_holder(facts.clip, kept, facts.screens);
        let mut actions = Vec::new();
        if holder == ClipHolder::Window {
            actions.push(CursorAction::Release);
        }
        if self.mode == CursorMode::Windows {
            return actions;
        }
        actions.push(CursorAction::Show);
        if holder != ClipHolder::Other
            && let Some(area) = facts.area
            && let Some(visible) = on_screens(area, facts.screens)
        {
            actions.push(CursorAction::Place(nearest(self.pointer_on_screen(area), visible)));
        }
        actions
    }
}
