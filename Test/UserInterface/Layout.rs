// Laying out in fixed pixels: rectangles split into parts, columns and rows,
// stacks, and the header and footer that fit both supported window heights.

use user_interface::headless::{self, Headless, MAXIMIZED};
use user_interface::{EDGE, PANELS_TOP, Rect, Stack};

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
fn a_stack_lays_out_down_its_area() {
    let mut stack = Stack::new(Rect::new(0.0, 100.0, 50.0, 200.0), 10.0);
    assert_eq!(stack.next(30.0), Rect::new(0.0, 100.0, 50.0, 30.0));
    stack.skip(5.0);
    assert_eq!(stack.next(20.0).y, 145.0);
    assert_eq!(stack.rest(), Rect::new(0.0, 175.0, 50.0, 125.0));
}

#[test]
fn the_header_and_footer_leave_the_panels_between_them() {
    let mut harness = Headless::default();
    let notices = vec!["A long notice about a setting. ".repeat(30), "Another notice".to_string()];
    harness.notices = notices;
    let (bottom, dropped) = harness.frame(&[], |frame| {
        frame.header("Test", "A test", &[("GPU: ", user_interface::theme::DIM), ("Test GPU", user_interface::theme::TEXT)]);
        let bottom = frame.footer(&[("F11", "borderless fullscreen")], "A note");
        (bottom, frame.dropped_lines())
    });
    let line_height = headless::font().cell_height as f32;
    assert_eq!(bottom, MAXIMIZED[1] - EDGE * 2.0 - line_height);
    // The title, the status and two lines of notices fit over the panels.
    assert!(EDGE + line_height * 4.0 <= PANELS_TOP);
    assert_eq!(dropped, 0);
}
