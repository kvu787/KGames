// Laying out in fixed pixels: rectangles split into parts, columns and rows,
// stacks down and across, places in a rectangle, the app bar and status bar
// that fit both supported window heights, and cards.

use user_interface::headless::{FULLSCREEN, Headless, MAXIMIZED};
use user_interface::{Align, CARD_HEADER, CARD_PADDING, PANELS_TOP, Rect, STATUS_BAR_HEIGHT, Stack, theme};

#[test]
fn a_rectangle_splits_into_parts_that_add_up() {
    let area = Rect::new(10.0, 20.0, 300.0, 200.0);
    let (top, rest) = area.split_top(50.0, 10.0);
    assert_eq!((top, rest), (Rect::new(10.0, 20.0, 300.0, 50.0), Rect::new(10.0, 80.0, 300.0, 140.0)));
    let (left, rest) = area.split_left(100.0, 10.0);
    assert_eq!((left.right(), rest.x, rest.right()), (110.0, 120.0, 310.0));
    let (bottom, rest) = area.split_bottom(40.0, 0.0);
    assert_eq!((bottom.y, rest.bottom()), (180.0, 180.0));
    let (right, rest) = area.split_right(40.0, 0.0);
    assert_eq!((right.x, rest.right()), (270.0, 270.0));
    assert_eq!(area.inset(10.0), Rect::new(20.0, 30.0, 280.0, 180.0));
    assert_eq!(area.inset(10.0).outset(10.0), area);
    assert_eq!(area.inset_by(10.0, 0.0), Rect::new(20.0, 20.0, 280.0, 200.0));
    assert_eq!(Rect::from_edges(1.0, 2.0, 11.0, 22.0), Rect::new(1.0, 2.0, 10.0, 20.0));
    assert_eq!(area.at([0.5, 1.0]), [160.0, 220.0]);
}

#[test]
fn columns_are_whole_pixels_and_the_last_takes_the_rest() {
    let columns = Rect::new(0.0, 0.0, 100.0, 10.0).columns(3, 5.0);
    let lefts: Vec<f32> = columns.iter().map(|column| column.x).collect();
    let widths: Vec<f32> = columns.iter().map(|column| column.width).collect();
    assert_eq!(lefts, [0.0, 35.0, 70.0]);
    assert_eq!(widths, [30.0, 30.0, 30.0]);
    let columns = Rect::new(0.0, 0.0, 101.0, 10.0).columns(2, 0.0);
    assert_eq!(columns[1].right(), 101.0);
    let rows = Rect::new(0.0, 0.0, 10.0, 100.0).rows(3, 20.0, 4.0);
    assert_eq!(rows.iter().map(|row| row.y).collect::<Vec<_>>(), [0.0, 24.0, 48.0]);
}

#[test]
fn a_rectangle_places_another_in_it_on_whole_pixels() {
    let area = Rect::new(10.0, 10.0, 101.0, 51.0);
    assert_eq!(area.place([20.0, 10.0], Align::Start, Align::End), Rect::new(10.0, 51.0, 20.0, 10.0));
    assert_eq!(area.centered([21.0, 11.0]), Rect::new(50.0, 30.0, 21.0, 11.0));
    assert_eq!(area.place([21.0, 11.0], Align::End, Align::Start), Rect::new(90.0, 10.0, 21.0, 11.0));
}

#[test]
fn a_stack_lays_out_down_or_across_its_area() {
    let mut stack = Stack::down(Rect::new(0.0, 100.0, 50.0, 200.0), 10.0);
    assert_eq!(stack.next(30.0), Rect::new(0.0, 100.0, 50.0, 30.0));
    stack.skip(5.0);
    assert_eq!(stack.next(20.0).y, 145.0);
    assert_eq!(stack.rest(), Rect::new(0.0, 175.0, 50.0, 125.0));
    let mut row = Stack::across(Rect::new(10.0, 0.0, 300.0, 32.0), 8.0);
    assert_eq!(row.next(100.0), Rect::new(10.0, 0.0, 100.0, 32.0));
    assert_eq!(row.next_sized([20.0, 16.0]), Rect::new(118.0, 0.0, 20.0, 16.0));
    assert_eq!(row.rest(), Rect::new(146.0, 0.0, 164.0, 32.0));
}

#[test]
fn the_app_bar_and_status_bar_leave_the_cards_between_them() {
    for size in [MAXIMIZED, FULLSCREEN] {
        let mut harness = Headless::new(size);
        let (bottom, dropped) = harness.frame(&[], |frame| {
            frame.header("Test", "A test");
            let theme = frame.theme();
            let bottom = frame.footer(&[("F11", "fullscreen"), ("Ctrl+S", "save")], &[("Test GPU", theme.text)]);
            (bottom, frame.dropped_lines())
        });
        assert_eq!(bottom, size[1] - STATUS_BAR_HEIGHT);
        assert!(bottom - PANELS_TOP > 1200.0);
        assert_eq!(dropped, 0);
    }
}

#[test]
fn a_card_holds_its_contents_under_its_title() {
    let mut harness = Headless::default();
    let area = Rect::new(16.0, 56.0, 400.0, 300.0);
    let (with_title, without) = harness.frame(&[], |frame| (frame.card(area, "Title", "a few words"), frame.card(area, "", "")));
    assert_eq!(with_title, Rect::from_edges(16.0 + CARD_PADDING, 56.0 + CARD_HEADER, 416.0 - CARD_PADDING, 356.0 - CARD_PADDING));
    assert_eq!(without, area.inset(CARD_PADDING));
    // What goes after the title is on its line, past it.
    let trailing = harness.frame(&[], |frame| frame.card_trailing(area, "Title"));
    assert!(trailing.x > area.x + CARD_PADDING + 20.0 && trailing.y < area.y + CARD_PADDING);
    assert_eq!(theme::LIGHT.name, "Light");
}
