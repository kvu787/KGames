// Tests what the window does with Windows' cursor, in both of its modes,
// which platform/cursor.rs decides on the facts that the window gathers from
// Windows, through whole sequences, without a window, as one with focus is
// what the facts describe, and tests never take focus.

use user_interface::{ClipHolder, CursorAction, CursorFacts, CursorKeeper, CursorMode, clip_holder, on_screens};
use windows_api_raw::{POINT, RECT};

use CursorAction::{Confine, Hide, Place, PlacePointer, Release, Show};

// The window's client area on the screen, the whole of the screens, which is
// the confinement without one, and another program's window.
const AREA: RECT = RECT { left: 0, top: 0, right: 2560, bottom: 1440 };
const SCREENS: RECT = RECT { left: -2560, top: 0, right: 5120, bottom: 1440 };
const OTHER: RECT = RECT { left: 2560, top: 0, right: 5120, bottom: 1440 };

// Where Windows' cursor is, in the area, where the pointer is placed as the
// window starts to keep it.
const AT_CURSOR: POINT = POINT { x: 300, y: 400 };

fn facts(focused: bool, clip: RECT) -> CursorFacts {
    CursorFacts {
        focused,
        area: Some(AREA),
        button_held: false,
        clicked: false,
        cursor: AT_CURSOR,
        clip,
        screens: SCREENS,
    }
}

// A keeper that keeps the cursor, with the pointer at (100, 200).
fn keeping() -> CursorKeeper {
    let mut keeper = CursorKeeper::new(true);
    keeper.set_pointer(POINT { x: 100, y: 200 });
    assert_eq!(keeper.update(facts(true, SCREENS)), [Confine(AREA), Hide, PlacePointer(AT_CURSOR)]);
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
    assert_eq!(keeper.update(facts(true, SCREENS)), [Confine(AREA), Hide, PlacePointer(AT_CURSOR)]);
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

// A button held as the window gets focus, from a click that isn't on it,
// like a drag in another program, or onto the window's taskbar button, which
// brings it to the front, puts off keeping the cursor until it's let go of,
// so the drag isn't kept on the window. A button held while it keeps it,
// like for a drag, changes nothing.
#[test]
fn a_held_button_puts_off_keeping_the_cursor_but_doesn_t_let_it_go() {
    let mut keeper = CursorKeeper::new(true);
    let held = |clip| CursorFacts { button_held: true, ..facts(true, clip) };
    assert_eq!(keeper.update(held(SCREENS)), []);
    assert!(!keeper.keeps());
    assert_eq!(keeper.update(held(SCREENS)), []);
    assert_eq!(keeper.update(facts(true, SCREENS)), [Confine(AREA), Hide, PlacePointer(AT_CURSOR)]);
    assert_eq!(keeper.update(held(AREA)), []);
    assert!(keeper.keeps());
}

// A click on the window that gives it focus keeps the cursor as the button
// goes down, while it's held, and its release changes nothing.
#[test]
fn a_click_on_the_window_keeps_the_cursor_as_it_goes_down() {
    let mut keeper = CursorKeeper::new(true);
    let click = CursorFacts { button_held: true, clicked: true, ..facts(true, SCREENS) };
    assert_eq!(keeper.update(click), [Confine(AREA), Hide, PlacePointer(AT_CURSOR)]);
    assert!(keeper.keeps());
    let held = CursorFacts { button_held: true, ..facts(true, AREA) };
    assert_eq!(keeper.update(held), []);
    assert_eq!(keeper.update(facts(true, AREA)), []);
    assert!(keeper.keeps());
}

// As the window starts to keep the cursor, the pointer is placed where
// Windows' cursor is, in the client area's pixels, up to the area's edges,
// and only then: not while the window keeps it.
#[test]
fn the_pointer_is_placed_where_the_cursor_is_as_the_window_starts_to_keep_it() {
    let shifted = RECT { left: 100, top: 50, right: 2660, bottom: 1490 };
    let at = |x, y| CursorFacts { area: Some(shifted), cursor: POINT { x, y }, ..facts(true, SCREENS) };
    let kept = on_screens(shifted, SCREENS).unwrap();
    let away = CursorFacts { focused: false, clip: kept, ..at(0, 0) };
    let mut keeper = CursorKeeper::new(true);
    assert_eq!(keeper.update(at(700, 650)), [Confine(kept), Hide, PlacePointer(POINT { x: 600, y: 600 })]);
    assert_eq!(keeper.update(CursorFacts { clip: kept, ..at(900, 900) }), []);
    for ((x, y), placed) in [((100, 50), POINT { x: 0, y: 0 }), ((2659, 1439), POINT { x: 2559, y: 1389 })] {
        keeper.update(away);
        assert_eq!(keeper.update(at(x, y)), [Confine(kept), Hide, PlacePointer(placed)]);
    }
}

// A cursor outside the area that the window keeps it in, like on another
// display as Alt+Tab gives the window focus, places the pointer in the middle
// of the client area, and not at the area's edge, where the confinement puts
// Windows' cursor: the middle of the part on the screens if the middle isn't
// on them.
#[test]
fn the_pointer_is_placed_in_the_middle_if_the_cursor_is_outside() {
    let shifted = RECT { left: 100, top: 50, right: 2660, bottom: 1490 };
    let at = |x, y| CursorFacts { area: Some(shifted), cursor: POINT { x, y }, ..facts(true, SCREENS) };
    let kept = on_screens(shifted, SCREENS).unwrap();
    let away = CursorFacts { focused: false, clip: kept, ..at(0, 0) };
    let middle = POINT { x: 1280, y: 720 };
    let mut keeper = CursorKeeper::new(true);
    for (x, y) in [(-1000, 20), (4000, 700), (2660, 700), (99, 700), (700, 49), (700, 1440)] {
        assert_eq!(keeper.update(at(x, y)), [Confine(kept), Hide, PlacePointer(middle)], "from ({x}, {y})");
        keeper.update(away);
    }
    let past = RECT { left: 4000, top: 0, right: 6560, bottom: 1440 };
    let part = on_screens(past, SCREENS).unwrap();
    let off = CursorFacts { area: Some(past), cursor: POINT { x: 0, y: 0 }, ..facts(true, SCREENS) };
    assert_eq!(keeper.update(off), [Confine(part), Hide, PlacePointer(POINT { x: 1119, y: 720 })]);
}

// A click on a window that doesn't get focus from it, like one that mustn't
// take focus, keeps nothing. The window says that it clicked only on the
// check after it (see platform/window.rs), and the keeper doesn't remember
// it: focus that comes later, with a button held elsewhere, waits for it to
// be let go of.
#[test]
fn a_click_counts_only_on_the_check_that_hears_of_it() {
    let mut keeper = CursorKeeper::new(true);
    let click = CursorFacts { button_held: false, clicked: true, ..facts(false, SCREENS) };
    assert_eq!(keeper.update(click), []);
    assert!(!keeper.keeps());
    let held = CursorFacts { button_held: true, ..facts(true, SCREENS) };
    assert_eq!(keeper.update(held), []);
    assert!(!keeper.keeps());
}

// The cursor is placed in the middle of the area if the pointer was nowhere
// yet, and inside the area wherever the pointer was.
#[test]
fn the_cursor_is_placed_inside_the_area() {
    let mut keeper = CursorKeeper::new(true);
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
    let mut keeper = CursorKeeper::new(true);
    keeper.set_pointer(POINT { x: 2000, y: 700 });
    assert_eq!(keeper.update(past(true, SCREENS)), [Confine(part), Hide, PlacePointer(POINT { x: 1119, y: 720 })]);
    assert_eq!(keeper.update(past(true, part)), []);
    assert_eq!(keeper.update(past(false, part)), [Release, Show, Place(POINT { x: 5119, y: 700 })]);
}

// A confinement that another program left, like one that ended while the
// Start menu was up, which Windows doesn't release, isn't taken for nobody's
// as the window lets the cursor go: Windows' release, to the screens, is.
#[test]
fn a_confinement_left_from_before_is_not_nobody_s() {
    let mut keeper = CursorKeeper::new(true);
    keeper.set_pointer(AT_POINTER);
    assert_eq!(keeper.update(facts(true, OTHER)), [Confine(AREA), Hide, PlacePointer(AT_CURSOR)]);
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
    let mut keeper = CursorKeeper::new(true);
    let empty = CursorFacts { area: Some(RECT { left: 100, top: 100, right: 100, bottom: 100 }), ..facts(true, SCREENS) };
    assert_eq!(keeper.update(empty), []);
    assert!(!keeper.keeps());
    let off = CursorFacts { area: Some(RECT { left: 6000, top: 0, right: 8560, bottom: 1440 }), ..facts(true, SCREENS) };
    assert_eq!(keeper.update(off), []);
    assert!(!keeper.keeps());
}

// A client area that leaves the screens while the window has focus, as they
// change under it, lets the cursor go, as there's nowhere on them to keep it,
// without placing it, and one that comes back keeps it again.
#[test]
fn a_client_area_that_leaves_the_screens_lets_the_cursor_go_with_focus() {
    let mut keeper = keeping();
    let off = CursorFacts { area: Some(RECT { left: 6000, top: 0, right: 8560, bottom: 1440 }), ..facts(true, AREA) };
    assert_eq!(keeper.update(off), [Release, Show]);
    assert!(!keeper.keeps());
    assert_eq!(keeper.update(facts(true, SCREENS)), [Confine(AREA), Hide, PlacePointer(AT_CURSOR)]);
    assert!(keeper.keeps());
}

// A run whose input is a script's, or that has none, doesn't read GameInput's
// mice, so nothing moves the pointer, and the window leaves Windows' cursor
// alone, with focus too, and as it closes.
#[test]
fn without_mice_the_cursor_is_left_alone() {
    let mut keeper = CursorKeeper::new(false);
    assert_eq!(keeper.update(facts(true, SCREENS)), []);
    assert!(!keeper.keeps());
    assert_eq!(keeper.update(facts(false, SCREENS)), []);
    assert_eq!(keeper.let_go(facts(false, SCREENS)), []);
}

// Ctrl+Shift+Alt+F11 switches the cursor to Windows' mode: while the window
// keeps the cursor, it shows where the pointer is, which it went its own way
// from while it was hidden, and is still kept. Switching back hides it, where
// it is. Switching to the mode it's in does nothing.
#[test]
fn switching_while_the_window_keeps_the_cursor_shows_it_where_the_pointer_is_or_hides_it() {
    let mut keeper = keeping();
    assert_eq!(keeper.set_mode(CursorMode::Windows, facts(true, AREA)), [Show, Place(AT_POINTER)]);
    assert_eq!(keeper.mode(), CursorMode::Windows);
    assert!(keeper.keeps());
    assert_eq!(keeper.set_mode(CursorMode::Windows, facts(true, AREA)), []);
    assert_eq!(keeper.update(facts(true, AREA)), []);
    assert_eq!(keeper.set_mode(CursorMode::GameInput, facts(true, AREA)), [Hide]);
    assert_eq!(keeper.mode(), CursorMode::GameInput);
    assert!(keeper.keeps());
}

// Windows' cursor shows where the pointer is, inside the area that it's kept
// in, or in the middle of the client area if the pointer was nowhere yet.
#[test]
fn switching_to_windows_mode_places_the_cursor_inside_the_area() {
    let mut keeper = CursorKeeper::new(true);
    keeper.update(facts(true, SCREENS));
    assert_eq!(keeper.set_mode(CursorMode::Windows, facts(true, AREA)), [Show, Place(POINT { x: 1280, y: 720 })]);
    keeper.set_mode(CursorMode::GameInput, facts(true, AREA));
    keeper.set_pointer(POINT { x: 5000, y: -3 });
    assert_eq!(keeper.set_mode(CursorMode::Windows, facts(true, AREA)), [Show, Place(POINT { x: 2559, y: 0 })]);
    let shifted = RECT { left: 100, top: 50, right: 2660, bottom: 1490 };
    let mut keeper = CursorKeeper::new(true);
    let at = CursorFacts { area: Some(shifted), ..facts(true, SCREENS) };
    keeper.update(at);
    keeper.set_pointer(POINT { x: 10, y: 20 });
    assert_eq!(keeper.set_mode(CursorMode::Windows, at), [Show, Place(POINT { x: 110, y: 70 })]);
}

// In Windows' mode, the window keeps the cursor as it does in GameInput's,
// but shown: as it starts to keep it, it doesn't hide it, and the pointer is
// placed where it is. As focus goes, the confinement is released, and the
// cursor stays where it is, which the pointer followed, and shows already.
#[test]
fn windows_mode_keeps_the_cursor_shown_and_lets_it_go_where_it_is() {
    let mut keeper = CursorKeeper::new(true);
    keeper.set_pointer(AT_POINTER);
    assert_eq!(keeper.set_mode(CursorMode::Windows, facts(true, SCREENS)), []);
    assert_eq!(keeper.update(facts(true, SCREENS)), [Confine(AREA), PlacePointer(AT_CURSOR)]);
    assert!(keeper.keeps());
    assert_eq!(keeper.update(facts(true, AREA)), []);
    assert_eq!(keeper.update(facts(true, OTHER)), [Confine(AREA)]);
    assert_eq!(keeper.update(facts(false, AREA)), [Release]);
    assert!(!keeper.keeps());
    assert_eq!(keeper.update(facts(true, SCREENS)), [Confine(AREA), PlacePointer(AT_CURSOR)]);
    assert_eq!(keeper.update(facts(false, SCREENS)), []);
    keeper.update(facts(true, SCREENS));
    assert_eq!(keeper.update(facts(false, OTHER)), []);
    keeper.update(facts(true, SCREENS));
    let minimized = CursorFacts { area: None, ..facts(false, AREA) };
    assert_eq!(keeper.update(minimized), [Release]);
    keeper.update(facts(true, SCREENS));
    assert_eq!(keeper.let_go(facts(false, AREA)), [Release]);
}

// In Windows' mode, a cursor outside the area that the window keeps it in,
// like on another display as Alt+Tab gives the window focus, is put in the
// middle of the client area, where the pointer is placed, and not left at
// the area's edge, where the confinement puts it, shown.
#[test]
fn in_windows_mode_a_cursor_outside_is_put_in_the_middle() {
    let shifted = RECT { left: 100, top: 50, right: 2660, bottom: 1490 };
    let at = |x, y| CursorFacts { area: Some(shifted), cursor: POINT { x, y }, ..facts(true, SCREENS) };
    let kept = on_screens(shifted, SCREENS).unwrap();
    let away = CursorFacts { focused: false, clip: kept, ..at(0, 0) };
    let mut keeper = CursorKeeper::new(true);
    keeper.set_mode(CursorMode::Windows, at(0, 0));
    for (x, y) in [(-1000, 20), (4000, 700), (99, 700), (700, 1440)] {
        let middle = POINT { x: 1380, y: 770 };
        assert_eq!(keeper.update(at(x, y)), [Confine(kept), Place(middle), PlacePointer(POINT { x: 1280, y: 720 })], "from ({x}, {y})");
        keeper.update(away);
    }
    assert_eq!(keeper.update(at(700, 650)), [Confine(kept), PlacePointer(POINT { x: 600, y: 600 })]);
}

// Switching while the window doesn't keep the cursor, like while another
// window has focus, or a click from elsewhere is held as it gets focus,
// changes only how the window keeps it next. So does switching in a run
// without mice, which never keeps it.
#[test]
fn switching_without_keeping_the_cursor_changes_only_how_it_s_kept_next() {
    let mut keeper = CursorKeeper::new(true);
    let held = CursorFacts { button_held: true, ..facts(true, SCREENS) };
    assert_eq!(keeper.update(held), []);
    assert_eq!(keeper.set_mode(CursorMode::Windows, held), []);
    assert_eq!(keeper.update(facts(true, SCREENS)), [Confine(AREA), PlacePointer(AT_CURSOR)]);
    assert_eq!(keeper.update(facts(false, AREA)), [Release]);
    assert_eq!(keeper.set_mode(CursorMode::GameInput, facts(false, SCREENS)), []);
    assert_eq!(keeper.update(facts(true, SCREENS)), [Confine(AREA), Hide, PlacePointer(AT_CURSOR)]);

    let mut keeper = CursorKeeper::new(false);
    assert_eq!(keeper.set_mode(CursorMode::Windows, facts(true, SCREENS)), []);
    assert_eq!(keeper.update(facts(true, SCREENS)), []);
    assert!(!keeper.keeps());
    assert_eq!(keeper.mode(), CursorMode::Windows);
}

// In Windows' mode, while the window keeps the cursor, the pointer follows
// it to where Windows says it is, in the client area's pixels, as each check
// finds it moved: not to where it already was as the window started to keep
// it, and the pointer was placed there, and not again while it stays.
#[test]
fn the_pointer_follows_the_cursor_s_moves_in_windows_mode() {
    let shifted = RECT { left: 100, top: 50, right: 2660, bottom: 1490 };
    let at = |x, y| CursorFacts { area: Some(shifted), cursor: POINT { x, y }, ..facts(true, SCREENS) };
    let mut keeper = CursorKeeper::new(true);
    keeper.set_mode(CursorMode::Windows, at(0, 0));
    assert!(!keeper.follows());
    assert_eq!(keeper.follow(POINT { x: 500, y: 500 }, shifted), None);
    keeper.update(at(700, 650));
    assert!(keeper.follows());
    assert_eq!(keeper.follow(POINT { x: 700, y: 650 }, shifted), None);
    assert_eq!(keeper.follow(POINT { x: 710, y: 655 }, shifted), Some(POINT { x: 610, y: 605 }));
    assert_eq!(keeper.follow(POINT { x: 710, y: 655 }, shifted), None);
    assert_eq!(keeper.follow(POINT { x: 700, y: 650 }, shifted), Some(POINT { x: 600, y: 600 }));
}

// Where the window put Windows' cursor isn't a move of it: in the middle, as
// it started to keep a cursor that was outside, and where the pointer is, as
// it switched to Windows' mode. Moves from there are.
#[test]
fn where_the_window_put_the_cursor_is_no_move_of_it() {
    let mut keeper = CursorKeeper::new(true);
    keeper.set_mode(CursorMode::Windows, facts(true, SCREENS));
    let outside = CursorFacts { cursor: POINT { x: 4000, y: 700 }, ..facts(true, SCREENS) };
    assert_eq!(keeper.update(outside), [Confine(AREA), Place(POINT { x: 1280, y: 720 }), PlacePointer(POINT { x: 1280, y: 720 })]);
    assert_eq!(keeper.follow(POINT { x: 1280, y: 720 }, AREA), None);
    assert_eq!(keeper.follow(POINT { x: 1281, y: 720 }, AREA), Some(POINT { x: 1281, y: 720 }));

    let mut keeper = keeping();
    assert_eq!(keeper.set_mode(CursorMode::Windows, facts(true, AREA)), [Show, Place(AT_POINTER)]);
    assert_eq!(keeper.follow(AT_POINTER, AREA), None);
    assert_eq!(keeper.follow(AT_CURSOR, AREA), Some(AT_CURSOR));
}

// The pointer follows nothing in GameInput's mode, where the mice move it,
// or while the window doesn't keep the cursor. As the window keeps it again
// after letting it go, the pointer is placed where the cursor is, which is
// no move either, and switching back to Windows' mode starts over from
// where the window put the cursor.
#[test]
fn the_pointer_follows_only_while_the_window_keeps_the_cursor_in_windows_mode() {
    let mut keeper = keeping();
    assert!(!keeper.follows());
    assert_eq!(keeper.follow(POINT { x: 5, y: 5 }, AREA), None);
    keeper.set_mode(CursorMode::Windows, facts(true, AREA));
    assert_eq!(keeper.follow(POINT { x: 5, y: 5 }, AREA), Some(POINT { x: 5, y: 5 }));
    keeper.update(facts(false, AREA));
    assert!(!keeper.follows());
    assert_eq!(keeper.follow(POINT { x: 6, y: 6 }, AREA), None);
    keeper.update(facts(true, SCREENS));
    assert!(keeper.follows());
    assert_eq!(keeper.follow(AT_CURSOR, AREA), None);
    keeper.set_mode(CursorMode::GameInput, facts(true, AREA));
    assert_eq!(keeper.follow(POINT { x: 7, y: 7 }, AREA), None);
    keeper.set_pointer(POINT { x: 7, y: 7 });
    assert_eq!(keeper.set_mode(CursorMode::Windows, facts(true, AREA)), [Show, Place(POINT { x: 7, y: 7 })]);
    assert_eq!(keeper.follow(POINT { x: 7, y: 7 }, AREA), None);
}
