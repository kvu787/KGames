// Tests what the window does with Windows' cursor, which platform/cursor.rs
// decides on the facts that the window gathers from Windows, through whole
// sequences, without a window, as one with focus is what the facts describe,
// and tests never take focus.

use user_interface::{ClipHolder, CursorAction, CursorFacts, CursorKeeper, clip_holder};
use windows_api::{POINT, RECT};

use CursorAction::{Confine, Hide, Place, Release, Show};

// The window's client area on the screen, the whole of the screens, which is
// the confinement without one, and another program's window.
const AREA: RECT = RECT { left: 0, top: 0, right: 2560, bottom: 1440 };
const SCREENS: RECT = RECT { left: -2560, top: 0, right: 5120, bottom: 1440 };
const OTHER: RECT = RECT { left: 2560, top: 0, right: 5120, bottom: 1440 };

fn facts(focused: bool, clip: RECT) -> CursorFacts {
    CursorFacts { focused, area: Some(AREA), button_held: false, clip }
}

// A keeper that keeps the cursor, with the pointer at (100, 200).
fn keeping() -> CursorKeeper {
    let mut keeper = CursorKeeper::default();
    keeper.set_pointer(POINT { x: 100, y: 200 });
    assert_eq!(keeper.update(facts(true, SCREENS)), [Confine(AREA), Hide]);
    keeper
}

const AT_POINTER: POINT = POINT { x: 100, y: 200 };

// With focus, the cursor is kept, hidden, at once, and nothing more is done
// while nothing changes. Alt+Tab, which activates another window, lets it go
// where the pointer was, and Alt+Tab back keeps it again.
#[test]
fn alt_tab_out_and_back() {
    let mut keeper = keeping();
    assert!(keeper.keeps());
    assert_eq!(keeper.update(facts(true, AREA)), []);
    assert_eq!(keeper.update(facts(false, AREA)), [Release, Show, Place(AT_POINTER)]);
    assert!(!keeper.keeps());
    assert_eq!(keeper.update(facts(false, SCREENS)), []);
    assert_eq!(keeper.update(facts(true, SCREENS)), [Confine(AREA), Hide]);
}

// Windows releases the confinement itself when a normal window is activated,
// so the window may find it released already: it only shows and places the
// cursor.
#[test]
fn a_confinement_that_windows_released_is_not_released_again() {
    let mut keeper = keeping();
    assert_eq!(keeper.update(facts(false, SCREENS)), [Show, Place(AT_POINTER)]);
}

// The Start menu takes the foreground without Windows releasing the
// confinement, so the window releases it.
#[test]
fn the_start_menu_lets_the_cursor_go() {
    let mut keeper = keeping();
    assert_eq!(keeper.update(facts(false, AREA)), [Release, Show, Place(AT_POINTER)]);
}

// A program that got focus may confine the cursor before the window finds
// that it lost focus: the window leaves its confinement, and the cursor, to
// it.
#[test]
fn another_program_s_confinement_is_left_alone() {
    let mut keeper = keeping();
    assert_eq!(keeper.update(facts(false, OTHER)), [Show]);
    assert!(!keeper.keeps());
}

// Something that changes the confinement while the window keeps the cursor,
// like the secure desktop, or a program in the background, which may, is
// undone, without hiding the cursor again.
#[test]
fn a_changed_confinement_is_confined_again() {
    let mut keeper = keeping();
    assert_eq!(keeper.update(facts(true, SCREENS)), [Confine(AREA)]);
    assert_eq!(keeper.update(facts(true, OTHER)), [Confine(AREA)]);
    assert_eq!(keeper.update(facts(true, AREA)), []);
}

// A minimized window has no area on the screen, and no focus: the cursor is
// let go of, but not placed, as there's nowhere to place it.
#[test]
fn minimizing_lets_the_cursor_go_without_placing_it() {
    let mut keeper = keeping();
    let minimized = CursorFacts { focused: false, area: None, button_held: false, clip: AREA };
    assert_eq!(keeper.update(minimized), [Release, Show]);
}

// As the window closes, the cursor is let go of where the pointer was, as
// Windows doesn't release a confinement when its program ends; and nothing
// happens if it wasn't kept.
#[test]
fn closing_lets_the_cursor_go() {
    let mut keeper = keeping();
    assert_eq!(keeper.let_go(AREA, Some(AREA)), [Release, Show, Place(AT_POINTER)]);
    assert_eq!(keeper.let_go(AREA, Some(AREA)), []);
}

// The window starts to keep the cursor only once no mouse button is held,
// like the click that gave it focus, which may be going on elsewhere. A
// button held while it keeps it, like for a drag, changes nothing.
#[test]
fn a_held_button_puts_off_keeping_the_cursor_but_doesn_t_let_it_go() {
    let mut keeper = CursorKeeper::default();
    let held = |clip| CursorFacts { button_held: true, ..facts(true, clip) };
    assert_eq!(keeper.update(held(SCREENS)), []);
    assert!(!keeper.keeps());
    assert_eq!(keeper.update(facts(true, SCREENS)), [Confine(AREA), Hide]);
    assert_eq!(keeper.update(held(AREA)), []);
    assert!(keeper.keeps());
}

// The cursor is placed in the middle of the area if the pointer was nowhere
// yet, and inside the area wherever the pointer was.
#[test]
fn the_cursor_is_placed_inside_the_area() {
    let mut keeper = CursorKeeper::default();
    keeper.update(facts(true, SCREENS));
    assert_eq!(keeper.update(facts(false, AREA)), [Release, Show, Place(POINT { x: 1280, y: 720 })]);
    keeper.update(facts(true, SCREENS));
    keeper.set_pointer(POINT { x: 5000, y: -3 });
    assert_eq!(keeper.update(facts(false, AREA)), [Release, Show, Place(POINT { x: 2559, y: 0 })]);
}

// The confinement is shared by every program, and the window tells whose it
// is by what it was before the window confined it, as Windows' documentation
// has a program save it.
#[test]
fn whose_confinement_it_is() {
    assert_eq!(clip_holder(AREA, AREA, SCREENS), ClipHolder::Window);
    assert_eq!(clip_holder(SCREENS, AREA, SCREENS), ClipHolder::Nobody);
    assert_eq!(clip_holder(OTHER, AREA, SCREENS), ClipHolder::Other);
}
