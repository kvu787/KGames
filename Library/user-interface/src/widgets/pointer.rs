// The pointer, which the interface draws over everything while the window
// keeps Windows' cursor, hidden (see platform/cursor.rs), where the mice's
// motion moved it (see input/events.rs), in the shape that the widget under
// it asks for (see
// Widget::cursor). It looks unlike Windows' cursors, so that the two are easy
// to tell apart: filled with the theme's accent, without an outline, and its
// arrow is a dart, without the tail of Windows' arrow.
//
// It's drawn where the pointer is, in parts of a pixel, as shapes with
// antialiased edges, not on the pixels' grid.

use crate::core::interaction::{Cursor, Sense};
use crate::core::interface::Interface;
use crate::core::tree::{Draw, Widget};
use crate::drawing::geometry::{Color, Point, Rect};
use crate::drawing::painter::Painter;

// The dart's corners from its tip, its hot spot: it points up and to the
// left, symmetric about the diagonal, with a notch in its back.
const DART_TIP: (f32, f32) = (0.0, 0.0);
const DART_RIGHT: (f32, f32) = (15.0, 6.0);
const DART_NOTCH: (f32, f32) = (8.5, 8.5);
const DART_LEFT: (f32, f32) = (6.0, 15.0);

// A cursor as a picture, the pointer's look, with the middle of its shape in
// the middle of its rectangle, like for a list of them. While the pointer is
// on it, the pointer has its look too.
pub struct CursorView {
    cursor: Cursor,
}

impl CursorView {
    pub fn new(cursor: Cursor) -> Self {
        Self { cursor }
    }
}

impl<M: 'static> Widget<M> for CursorView {
    fn kind(&self) -> &'static str {
        "Cursor"
    }

    fn sense(&self) -> Option<Sense> {
        Some(Sense::NONE)
    }

    fn cursor(&self, _ui: &Interface, hovered: bool, _held: bool) -> Option<Cursor> {
        hovered.then_some(self.cursor)
    }

    fn draw(&self, ctx: &mut Draw) {
        let center = ctx.rect().center();
        let middle = middle(self.cursor);
        draw_pointer(&mut ctx.painter(), Point::new(center.x - middle.x, center.y - middle.y), self.cursor);
    }
}

// The middle of the cursor's shape, from its hot spot: the dart's and the
// hand's hot spots are their tips, and the others' are their middles.
fn middle(cursor: Cursor) -> Point {
    match cursor {
        Cursor::Arrow => Point::new(DART_RIGHT.0 / 2.0, DART_LEFT.1 / 2.0),
        // From the thumb's side to the little finger's, and the fingertip to
        // the palm's bottom.
        Cursor::Hand => Point::new((-6.75 + 13.0) / 2.0, 17.5 / 2.0),
        _ => Point::ZERO,
    }
}

// Draws the pointer's cursor with its hot spot at the point.
pub(crate) fn draw_pointer(painter: &mut Painter, at: Point, cursor: Cursor) {
    let color = painter.theme().accent;
    let point = |x: f32, y: f32| Point::new(at.x + x, at.y + y);
    match cursor {
        Cursor::Arrow => dart(painter, at, color),
        Cursor::Hand => hand(painter, at, color),
        Cursor::Text => {
            // An I-beam, with its hot spot in its middle.
            let strokes = [((0.0, -8.0), (0.0, 8.0)), ((-3.0, -8.0), (3.0, -8.0)), ((-3.0, 8.0), (3.0, 8.0))];
            lines(painter, &strokes.map(|((x0, y0), (x1, y1))| (point(x0, y0), point(x1, y1))), color);
        }
        Cursor::Move => double_arrows(painter, at, &[(Point::new(1.0, 0.0), 9.0), (Point::new(0.0, 1.0), 9.0)], color),
        Cursor::ResizeHorizontal => double_arrows(painter, at, &[(Point::new(1.0, 0.0), 9.0)], color),
        Cursor::ResizeVertical => double_arrows(painter, at, &[(Point::new(0.0, 1.0), 9.0)], color),
        Cursor::ResizeDiagonal => double_arrows(painter, at, &[(Point::new(1.0, 1.0), 7.0)], color),
        Cursor::ResizeAntidiagonal => double_arrows(painter, at, &[(Point::new(1.0, -1.0), 7.0)], color),
        Cursor::Crosshair => {
            let strokes = [((-9.0, 0.0), (9.0, 0.0)), ((0.0, -9.0), (0.0, 9.0))];
            lines(painter, &strokes.map(|((x0, y0), (x1, y1))| (point(x0, y0), point(x1, y1))), color);
        }
        Cursor::NotAllowed => {
            painter.ring(at, 8.0, 2.0, color);
            painter.line(point(-5.0, -5.0), point(5.0, 5.0), 2.0, color);
        }
    }
}

// The dart: two convex halves that overlap down its middle, as the painter
// fills only convex polygons, and two that only met there would leave a seam
// of their antialiased edges.
fn dart(painter: &mut Painter, at: Point, color: Color) {
    let corner = |(x, y): (f32, f32)| Point::new(at.x + x, at.y + y);
    let [tip, right, notch, left] = [DART_TIP, DART_RIGHT, DART_NOTCH, DART_LEFT].map(corner);
    // Halfway along each side from the tip: the halves reach across the
    // middle to them.
    let halfway = |to: Point| Point::new((tip.x + to.x) / 2.0, (tip.y + to.y) / 2.0);
    painter.polygon(&[tip, right, notch, halfway(left)], color);
    painter.polygon(&[tip, halfway(right), notch, left], color);
}

// A hand that points up, with its hot spot at its fingertip: four fingers and
// a thumb, round-ended lines, and a palm.
fn hand(painter: &mut Painter, at: Point, color: Color) {
    let point = |x: f32, y: f32| Point::new(at.x + x, at.y + y);
    let fingers = [
        (point(0.0, 1.75), point(0.0, 10.0), 3.5),
        (point(4.25, 5.75), point(4.25, 11.0), 3.5),
        (point(8.0, 6.75), point(8.0, 11.0), 3.5),
        (point(11.5, 8.5), point(11.5, 11.5), 3.0),
        (point(-5.0, 9.5), point(-1.5, 13.5), 3.5),
    ];
    for (from, to, width) in fingers {
        painter.line(from, to, width, color);
    }
    painter.shape(Rect::new(at.x - 1.75, at.y + 10.0, 14.75, 7.5), 3.0, color, None);
}

// Lines 2 pixels wide, which cover a pixel whole wherever they are, where a
// line a pixel wide on the line between two pixels covers half of each.
fn lines(painter: &mut Painter, strokes: &[(Point, Point)], color: Color) {
    for (from, to) in strokes {
        painter.line(*from, *to, 2.0, color);
    }
}

// Arrows both ways through the point, each `reach` along its direction each
// way, with their heads.
fn double_arrows(painter: &mut Painter, at: Point, arrows: &[(Point, f32)], color: Color) {
    for &(direction, reach) in arrows {
        let length = (direction.x * direction.x + direction.y * direction.y).sqrt();
        let along = Point::new(direction.x / length, direction.y / length);
        let across = Point::new(-along.y, along.x);
        let at_distance = |distance: f32, side: f32| Point::new(at.x + along.x * distance + across.x * side, at.y + along.y * distance + across.y * side);
        for end in [reach, -reach] {
            let back = end - 4.0 * end.signum();
            painter.polygon(&[at_distance(end, 0.0), at_distance(back, 3.5), at_distance(back, -3.5)], color);
        }
        painter.line(at_distance(3.0 - reach, 0.0), at_distance(reach - 3.0, 0.0), 1.5, color);
    }
}
