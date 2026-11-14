// Windows' cursor, while the interface draws a pointer of its own, as games
// do: GameInput's mice move the interface's pointer (see input/reader.rs),
// and Windows' cursor is kept out of the way. What the window does with it
// is decided here, from facts that platform/window.rs gathers from Windows,
// as actions that it carries out, so that every transition is tested
// without a window that has focus (Test/UserInterface/tests/cursor.rs).
//
// - While the window has focus, it keeps Windows' cursor inside its client
//   area, hidden, and its frames draw the pointer exactly while it does.
// - It starts to keep it only once no mouse button is held, by Windows'
//   account of the mouse, which, unlike GameInput's, has a click that's
//   still going on elsewhere, like the one that gave the window focus.
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
//   the screen. As the window closes, the same.

use windows_api::{POINT, RECT};

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
    // The cursor's confinement, as GetClipCursor gives it: the whole of the
    // screens without one.
    pub clip: RECT,
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
    // Shows Windows' arrow over the client area again.
    Show,
    // Puts it there, in the screen's pixels, with SetCursorPos.
    Place(POINT),
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
// `area`, and the confinement was `free` before, as Windows' documentation
// has a program save it.
pub fn clip_holder(clip: RECT, area: RECT, free: RECT) -> ClipHolder {
    if clip == area {
        ClipHolder::Window
    } else if clip == free {
        ClipHolder::Nobody
    } else {
        ClipHolder::Other
    }
}

// Where the window keeps the cursor, and the confinement as it was before.
#[derive(Clone, Copy, Debug)]
struct Kept {
    area: RECT,
    free: RECT,
}

// What the window does with Windows' cursor, and what it has done.
#[derive(Clone, Debug, Default)]
pub struct CursorKeeper {
    kept: Option<Kept>,
    // Where the interface's pointer last was, in the client area's pixels.
    pointer: Option<POINT>,
}

impl CursorKeeper {
    // Whether the window keeps Windows' cursor, hidden, so that its frames
    // draw the pointer.
    pub fn keeps(&self) -> bool {
        self.kept.is_some()
    }

    // Where the interface's pointer is, in the client area's pixels, where
    // Windows' cursor shows as it's let go of.
    pub fn set_pointer(&mut self, at: POINT) {
        self.pointer = Some(at);
    }

    // What to do with the cursor, on what Windows says now.
    pub fn update(&mut self, facts: CursorFacts) -> Vec<CursorAction> {
        let mut actions = Vec::new();
        match (facts.focused, facts.area, self.kept) {
            (true, Some(area), Some(kept)) => {
                if facts.clip != area {
                    actions.push(CursorAction::Confine(area));
                }
                self.kept = Some(Kept { area, ..kept });
            }
            (true, Some(area), None) if !facts.button_held => {
                if facts.clip != area {
                    actions.push(CursorAction::Confine(area));
                }
                actions.push(CursorAction::Hide);
                self.kept = Some(Kept { area, free: facts.clip });
            }
            (true, Some(_), None) => {}
            _ => actions = self.let_go(facts.clip, facts.area),
        }
        actions
    }

    // Lets the cursor go, if the window keeps it, as focus goes or the window
    // closes: releases the confinement if the window still holds it, and
    // puts the cursor where the pointer was, or in the middle of the area if
    // it was nowhere yet, unless another program has confined it, or there's
    // no area.
    pub fn let_go(&mut self, clip: RECT, area: Option<RECT>) -> Vec<CursorAction> {
        let Some(kept) = self.kept.take() else {
            return Vec::new();
        };
        let holder = clip_holder(clip, kept.area, kept.free);
        let mut actions = Vec::new();
        if holder == ClipHolder::Window {
            actions.push(CursorAction::Release);
        }
        actions.push(CursorAction::Show);
        if holder != ClipHolder::Other
            && let Some(area) = area
        {
            let middle = POINT {
                x: (area.right - area.left) / 2,
                y: (area.bottom - area.top) / 2,
            };
            let at = self.pointer.unwrap_or(middle);
            actions.push(CursorAction::Place(POINT {
                x: (area.left + at.x).clamp(area.left, (area.right - 1).max(area.left)),
                y: (area.top + at.y).clamp(area.top, (area.bottom - 1).max(area.top)),
            }));
        }
        actions
    }
}
