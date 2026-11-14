// Tests what the window does with Windows' cursor, which platform/cursor.rs
// decides on the facts that the window gathers from Windows, through whole
// sequences, without a window, as one with focus is what the facts describe,
// and tests never take focus.

use user_interface::{ClipHolder, CursorAction, CursorFacts, CursorKeeper, clip_holder, on_screens};
use windows_api::{POINT, RECT};

use CursorAction::{Confine, Hide, Place, Release, Show};

// The window's client area on the screen, the whole of the screens, which is
// the confinement without one, and another program's window.
const AREA: RECT = RECT { left: 0, top: 0, right: 2560, bottom: 1440 };
const SCREENS: RECT = RECT { left: -2560, top: 0, right: 5120, bottom: 1440 };
const OTHER: RECT = RECT { left: 2560, top: 0, right: 5120, bottom: 1440 };

fn facts(focused: bool, clip: RECT) -> CursorFacts {
    CursorFacts {
        focused,
        mice: true,
        area: Some(AREA),
        button_held: false,
        clip,
        screens: SCREENS,
    }
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

// F1 lets the cursor go while the window has focus, where the pointer is, as
// focus going does, and keeps it again.
#[test]
fn f1_lets_the_cursor_go_and_keeps_it_again() {
    let mut keeper = keeping();
    let let_go = |clip| CursorFacts { mice: false, ..facts(true, clip) };
    assert_eq!(keeper.update(let_go(AREA)), [Release, Show, Place(AT_POINTER)]);
    assert!(!keeper.keeps());
    assert_eq!(keeper.update(let_go(SCREENS)), []);
    assert_eq!(keeper.update(facts(true, SCREENS)), [Confine(AREA), Hide]);
    assert!(keeper.keeps());
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
    let minimized = CursorFacts { area: None, ..facts(false, AREA) };
    assert_eq!(keeper.update(minimized), [Release, Show]);
}

// As the window closes, the cursor is let go of where the pointer was, as
// Windows doesn't release a confinement when its program ends; and nothing
// happens if it wasn't kept.
#[test]
fn closing_lets_the_cursor_go() {
    let mut keeper = keeping();
    assert_eq!(keeper.let_go(facts(false, AREA)), [Release, Show, Place(AT_POINTER)]);
    assert_eq!(keeper.let_go(facts(false, AREA)), []);
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
// is by where it kept the cursor, and by the screens, which are the
// confinement without one.
#[test]
fn whose_confinement_it_is() {
    assert_eq!(clip_holder(AREA, AREA, SCREENS), ClipHolder::Window);
    assert_eq!(clip_holder(SCREENS, AREA, SCREENS), ClipHolder::Nobody);
    assert_eq!(clip_holder(OTHER, AREA, SCREENS), ClipHolder::Other);
}

// Windows keeps a confinement within the screens: a client area that goes
// past their edge, as on a display set up wrong, is confined to its part on
// them, which is the window's own confinement from then on, and the cursor
// is let go of on them.
#[test]
fn a_client_area_past_the_screens_edge_keeps_its_part_on_them() {
    let area = RECT { left: 4000, top: 0, right: 6560, bottom: 1440 };
    let part = RECT { left: 4000, top: 0, right: 5120, bottom: 1440 };
    assert_eq!(on_screens(area, SCREENS), Some(part));
    let past = |focused, clip| CursorFacts { area: Some(area), ..facts(focused, clip) };
    let mut keeper = CursorKeeper::default();
    keeper.set_pointer(POINT { x: 2000, y: 700 });
    assert_eq!(keeper.update(past(true, SCREENS)), [Confine(part), Hide]);
    assert_eq!(keeper.update(past(true, part)), []);
    assert_eq!(keeper.update(past(false, part)), [Release, Show, Place(POINT { x: 5119, y: 700 })]);
}

// A confinement that another program left, like one that ended while the
// Start menu was up, which Windows doesn't release, isn't taken for nobody's
// as the window lets the cursor go: Windows' release, to the screens, is.
#[test]
fn a_confinement_left_from_before_is_not_nobody_s() {
    let mut keeper = CursorKeeper::default();
    keeper.set_pointer(AT_POINTER);
    assert_eq!(keeper.update(facts(true, OTHER)), [Confine(AREA), Hide]);
    assert_eq!(keeper.update(facts(false, SCREENS)), [Show, Place(AT_POINTER)]);
}

// The screens can change while the window keeps the cursor, like with a
// display that's connected: Windows' release is to the screens as they are
// then.
#[test]
fn a_release_is_to_the_screens_as_they_are() {
    let mut keeper = keeping();
    let wider = RECT { right: 7680, ..SCREENS };
    let after = CursorFacts { screens: wider, ..facts(false, wider) };
    assert_eq!(keeper.update(after), [Show, Place(AT_POINTER)]);
}

// A client area with nothing on the screens, like an empty one, keeps
// nothing: a confinement to it would pin the cursor to a point, hidden.
#[test]
fn an_empty_client_area_keeps_nothing() {
    let mut keeper = CursorKeeper::default();
    let empty = CursorFacts { area: Some(RECT { left: 100, top: 100, right: 100, bottom: 100 }), ..facts(true, SCREENS) };
    assert_eq!(keeper.update(empty), []);
    assert!(!keeper.keeps());
    let off = CursorFacts { area: Some(RECT { left: 6000, top: 0, right: 8560, bottom: 1440 }), ..facts(true, SCREENS) };
    assert_eq!(keeper.update(off), []);
    assert!(!keeper.keeps());
}

// A run whose input is a script's, or that has none, doesn't read GameInput's
// mice, so nothing moves the pointer, and the window leaves Windows' cursor
// alone, with focus too.
#[test]
fn without_mice_the_cursor_is_left_alone() {
    let mut keeper = CursorKeeper::default();
    let without = |focused, clip| CursorFacts { mice: false, ..facts(focused, clip) };
    assert_eq!(keeper.update(without(true, SCREENS)), []);
    assert!(!keeper.keeps());
    assert_eq!(keeper.update(without(false, SCREENS)), []);
}
