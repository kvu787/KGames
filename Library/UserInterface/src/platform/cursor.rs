// Windows' cursor, while the interface draws a pointer of its own, as games
// do: GameInput's mice move the interface's pointer (see input/reader.rs),
// and Windows' cursor is kept out of the way. What the window does with it
// is decided here, from facts that platform/window.rs gathers from Windows,
// as actions that it carries out, so that every transition is tested
// without a window that has focus (Test/UserInterface/tests/cursor.rs).
//
// - While the window has focus, reads GameInput's mice, and F1 says so, it
//   keeps Windows' cursor inside its client area, hidden, and its frames draw
//   the pointer exactly while it does. F1 lets it go, where the interface's
//   pointer is, and keeps it again (see input/reader.rs). A run whose input
//   is a script's, or that has none, leaves the cursor alone.
// - It starts to keep it only once no mouse button is held, by Windows'
//   account of the mouse, which, unlike GameInput's, has a click that's
//   still going on elsewhere, like the one that gave the window focus.
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
//   the screen. As the window closes, the same.

use windows_api::{POINT, RECT};

// What Windows says, each time the window checks its focus.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CursorFacts {
    // Whether the window has focus: it's the foreground window, and isn't
    // minimized.
    pub focused: bool,
    // Whether the window keeps the cursor while it has focus: its frames read
    // GameInput's mice, which move the pointer, not a script's input, or
    // none, and F1 last chose to keep it.
    pub mice: bool,
    // The client area, in the screen's pixels, or none while the window is
    // minimized, whose client area is off the screen then.
    pub area: Option<RECT>,
    // Whether a mouse button is held, by Windows' account of the mouse.
    pub button_held: bool,
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

// What the window does with Windows' cursor, and what it has done.
#[derive(Clone, Debug, Default)]
pub struct CursorKeeper {
    // Where the window keeps the cursor, in the screen's pixels.
    kept: Option<RECT>,
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
        let keep = facts.area.filter(|_| facts.mice).and_then(|area| on_screens(area, facts.screens));
        let mut actions = Vec::new();
        match (facts.focused, keep) {
            (true, Some(keep)) if self.kept.is_some() || !facts.button_held => {
                if facts.clip != keep {
                    actions.push(CursorAction::Confine(keep));
                }
                if self.kept.replace(keep).is_none() {
                    actions.push(CursorAction::Hide);
                }
            }
            (true, Some(_)) => {}
            _ => actions = self.let_go(facts),
        }
        actions
    }

    // Lets the cursor go, if the window keeps it, as focus goes or the window
    // closes: releases the confinement if the window still holds it, and
    // puts the cursor where the pointer was, or in the middle of the client
    // area if it was nowhere yet, on the screens, unless another program has
    // confined it, or there's no client area on the screens.
    pub fn let_go(&mut self, facts: CursorFacts) -> Vec<CursorAction> {
        let Some(kept) = self.kept.take() else {
            return Vec::new();
        };
        let holder = clip_holder(facts.clip, kept, facts.screens);
        let mut actions = Vec::new();
        if holder == ClipHolder::Window {
            actions.push(CursorAction::Release);
        }
        actions.push(CursorAction::Show);
        if holder != ClipHolder::Other
            && let Some(area) = facts.area
            && let Some(visible) = on_screens(area, facts.screens)
        {
            let middle = POINT {
                x: (area.right - area.left) / 2,
                y: (area.bottom - area.top) / 2,
            };
            let at = self.pointer.unwrap_or(middle);
            actions.push(CursorAction::Place(POINT {
                x: (area.left + at.x).clamp(visible.left, visible.right - 1),
                y: (area.top + at.y).clamp(visible.top, visible.bottom - 1),
            }));
        }
        actions
    }
}
