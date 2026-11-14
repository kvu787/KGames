// The pointer, which the interface draws over everything while the window
// keeps Windows' cursor, hidden (see platform/cursor.rs), where the mice's
// motion moved it (see input/events.rs), in the shape that the widget under
// it asks for (see
// Widget::cursor). It looks unlike Windows' cursors, so that the two are easy
// to tell apart: filled with the theme's accent, edged in text_on_accent,
// which is made to show on it, white in the light theme and black in the
// dark, so that it shows on whatever is under it, and its arrow is a dart,
// without the tail of Windows' arrow.
//
// It's drawn where the pointer is, in parts of a pixel, as shapes with
// antialiased edges, not on the pixels' grid. Each shape's edge is drawn
// first, the edges of all its parts, then their fills over them, so that its
// parts join without an edge between them.

use crate::core::interaction::Cursor;
use crate::drawing::geometry::{Color, Point, Rect};
use crate::drawing::painter::Painter;

// The pointer's colors: its fill, and the edge around it.
#[derive(Clone, Copy)]
struct Colors {
    fill: Color,
    edge: Color,
}

// How far the edge reaches past the fill.
const EDGE: f32 = 1.5;

// The dart's corners from its tip, its hot spot: it points up and to the
// left, symmetric about the diagonal, with a notch in its back.
const DART_TIP: (f32, f32) = (0.0, 0.0);
const DART_RIGHT: (f32, f32) = (15.0, 6.0);
const DART_NOTCH: (f32, f32) = (8.5, 8.5);
const DART_LEFT: (f32, f32) = (6.0, 15.0);

// Draws the pointer's cursor with its hot spot at the point.
pub(crate) fn draw_pointer(painter: &mut Painter, at: Point, cursor: Cursor) {
    let theme = painter.theme();
    let colors = Colors {
        fill: theme.accent,
        edge: theme.text_on_accent,
    };
    let point = |x: f32, y: f32| Point::new(at.x + x, at.y + y);
    match cursor {
        Cursor::Arrow => dart(painter, at, colors),
        Cursor::Hand => hand(painter, at, colors),
        Cursor::Text => {
            // An I-beam, with its hot spot in its middle.
            let strokes = [((0.0, -8.0), (0.0, 8.0)), ((-3.0, -8.0), (3.0, -8.0)), ((-3.0, 8.0), (3.0, 8.0))];
            edged_lines(painter, &strokes.map(|((x0, y0), (x1, y1))| (point(x0, y0), point(x1, y1))), colors);
        }
        Cursor::Move => double_arrows(painter, at, &[(Point::new(1.0, 0.0), 9.0), (Point::new(0.0, 1.0), 9.0)], colors),
        Cursor::ResizeHorizontal => double_arrows(painter, at, &[(Point::new(1.0, 0.0), 9.0)], colors),
        Cursor::ResizeVertical => double_arrows(painter, at, &[(Point::new(0.0, 1.0), 9.0)], colors),
        Cursor::ResizeDiagonal => double_arrows(painter, at, &[(Point::new(1.0, 1.0), 7.0)], colors),
        Cursor::ResizeAntidiagonal => double_arrows(painter, at, &[(Point::new(1.0, -1.0), 7.0)], colors),
        Cursor::Crosshair => {
            let strokes = [((-9.0, 0.0), (9.0, 0.0)), ((0.0, -9.0), (0.0, 9.0))];
            edged_lines(painter, &strokes.map(|((x0, y0), (x1, y1))| (point(x0, y0), point(x1, y1))), colors);
        }
        Cursor::NotAllowed => {
            // A ring is inside its radius: the edge's reaches past it each way.
            painter.ring(at, 8.0 + EDGE, 2.0 + EDGE * 2.0, colors.edge);
            painter.line(point(-5.0, -5.0), point(5.0, 5.0), 2.0 + EDGE * 2.0, colors.edge);
            painter.ring(at, 8.0, 2.0, colors.fill);
            painter.line(point(-5.0, -5.0), point(5.0, 5.0), 2.0, colors.fill);
        }
    }
}

// The dart: its edge, round-ended lines along its outline, which round its
// corners, and its fill, two convex halves that overlap down its middle, as
// the painter fills only convex polygons, and two that only met there would
// leave a seam of their antialiased edges.
fn dart(painter: &mut Painter, at: Point, colors: Colors) {
    let corner = |(x, y): (f32, f32)| Point::new(at.x + x, at.y + y);
    let [tip, right, notch, left] = [DART_TIP, DART_RIGHT, DART_NOTCH, DART_LEFT].map(corner);
    painter.polyline(&[tip, right, notch, left, tip], EDGE * 2.0, colors.edge);
    // Halfway along each side from the tip: the halves reach across the
    // middle to them.
    let halfway = |to: Point| Point::new((tip.x + to.x) / 2.0, (tip.y + to.y) / 2.0);
    painter.polygon(&[tip, right, notch, halfway(left)], colors.fill);
    painter.polygon(&[tip, halfway(right), notch, left], colors.fill);
}

// A hand that points up, with its hot spot at its fingertip: four fingers and
// a thumb, round-ended lines, and a palm.
fn hand(painter: &mut Painter, at: Point, colors: Colors) {
    let point = |x: f32, y: f32| Point::new(at.x + x, at.y + y);
    let fingers = [
        (point(0.0, 1.75), point(0.0, 10.0), 3.5),
        (point(4.25, 5.75), point(4.25, 11.0), 3.5),
        (point(8.0, 6.75), point(8.0, 11.0), 3.5),
        (point(11.5, 8.5), point(11.5, 11.5), 3.0),
        (point(-5.0, 9.5), point(-1.5, 13.5), 3.5),
    ];
    let palm = Rect::new(at.x - 1.75, at.y + 10.0, 14.75, 7.5);
    for (from, to, width) in fingers {
        painter.line(from, to, width + EDGE * 2.0, colors.edge);
    }
    painter.shape(palm.expand(EDGE), 3.0 + EDGE, colors.edge, None);
    for (from, to, width) in fingers {
        painter.line(from, to, width, colors.fill);
    }
    painter.shape(palm, 3.0, colors.fill, None);
}

// Lines in the fill's color, each over a wider one in the edge's: 2 pixels
// wide, which cover a pixel whole wherever they are, where a line a pixel wide
// on the line between two pixels covers half of each.
fn edged_lines(painter: &mut Painter, strokes: &[(Point, Point)], colors: Colors) {
    for (from, to) in strokes {
        painter.line(*from, *to, 2.0 + EDGE * 2.0, colors.edge);
    }
    for (from, to) in strokes {
        painter.line(*from, *to, 2.0, colors.fill);
    }
}

// Arrows both ways through the point, each `reach` along its direction each
// way, with their heads: all their edges, then all their fill, so that one's
// edge never covers another's fill.
fn double_arrows(painter: &mut Painter, at: Point, arrows: &[(Point, f32)], colors: Colors) {
    for edge in [true, false] {
        for &(direction, reach) in arrows {
            let margin = if edge { EDGE } else { 0.0 };
            let color = if edge { colors.edge } else { colors.fill };
            let length = (direction.x * direction.x + direction.y * direction.y).sqrt();
            let along = Point::new(direction.x / length, direction.y / length);
            let across = Point::new(-along.y, along.x);
            let at_distance = |distance: f32, side: f32| Point::new(at.x + along.x * distance + across.x * side, at.y + along.y * distance + across.y * side);
            for end in [reach, -reach] {
                let tip = end + 1.5 * margin * end.signum();
                let back = end - (4.0 + 0.5 * margin) * end.signum();
                let half = 3.5 + 1.5 * margin;
                painter.polygon(&[at_distance(tip, 0.0), at_distance(back, half), at_distance(back, -half)], color);
            }
            let shaft = if edge { reach } else { reach - 3.0 };
            painter.line(at_distance(-shaft, 0.0), at_distance(shaft, 0.0), 1.5 + 2.0 * margin, color);
        }
    }
}
