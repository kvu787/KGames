// The pointer, which the interface draws over everything while the window has
// focus, at the reader's pointer, in the shape that the widget under it asks
// for (see Widget::cursor). Windows' own cursor is hidden over the window
// then (see platform/window.rs). Its colors are Windows' cursors', black with
// a white edge, not the theme's, so that it shows on whatever is under it, in
// either theme.

use crate::core::interaction::Cursor;
use crate::drawing::geometry::{Color, Point, Rect};
use crate::drawing::painter::Painter;

const INK: Color = Color::BLACK;
const EDGE: Color = Color::WHITE;

// The arrow, a pixel a character, B in ink and W in the edge's color, with
// its hot spot at its top left: a head, and a tail that leans right.
const ARROW: [&str; 20] = [
    "B",
    "BB",
    "BWB",
    "BWWB",
    "BWWWB",
    "BWWWWB",
    "BWWWWWB",
    "BWWWWWWB",
    "BWWWWWWWB",
    "BWWWWWWWWB",
    "BWWWWWWWWWB",
    "BWWWWWWWWWWB",
    "BWWWWWWBBBBB",
    "BWWWBWWB",
    "BWWB BWWB",
    "BWB  BWWB",
    "BB    BWWB",
    "B     BWWB",
    "       BWWB",
    "        BB",
];

// A hand that points up, with its hot spot at the top left of its
// fingertip, 7 pixels in.
const HAND: [&str; 20] = [
    "       BB",
    "      BWWB",
    "      BWWB",
    "      BWWB",
    "      BWWB",
    "      BWWBBB",
    "      BWWBWWBBB",
    "      BWWBWWBWWBB",
    " BB   BWWBWWBWWBWB",
    "BWWB  BWWWWWWWWWWB",
    "BWWWB BWWWWWWWWWWB",
    " BWWWWWWWWWWWWWWWB",
    "  BWWWWWWWWWWWWWWB",
    "  BWWWWWWWWWWWWWWB",
    "   BWWWWWWWWWWWWWB",
    "    BWWWWWWWWWWWWB",
    "    BWWWWWWWWWWWB",
    "     BWWWWWWWWWWB",
    "     BWWWWWWWWWWB",
    "     BBBBBBBBBBBB",
];
const HAND_HOT_SPOT: f32 = 7.0;

// Draws the pointer's cursor with its hot spot on the pixel at the point.
pub(crate) fn draw_pointer(painter: &mut Painter, at: Point, cursor: Cursor) {
    let pixel = Point::new(at.x.floor(), at.y.floor());
    // The hot spot's pixel's middle, so that a line a pixel wide along it,
    // or three, covers whole pixels, as a cursor's do.
    let at = Point::new(pixel.x + 0.5, pixel.y + 0.5);
    let point = |x: f32, y: f32| Point::new(at.x + x, at.y + y);
    match cursor {
        Cursor::Arrow => pixels(painter, pixel, &ARROW),
        Cursor::Hand => pixels(painter, Point::new(pixel.x - HAND_HOT_SPOT, pixel.y), &HAND),
        Cursor::Text => {
            // An I-beam, with its hot spot in its middle.
            let strokes = [((0.0, -8.0), (0.0, 8.0)), ((-3.0, -8.0), (3.0, -8.0)), ((-3.0, 8.0), (3.0, 8.0))];
            edged_lines(painter, &strokes.map(|((x0, y0), (x1, y1))| (point(x0, y0), point(x1, y1))));
        }
        Cursor::Move => double_arrows(painter, at, &[(Point::new(1.0, 0.0), 9.0), (Point::new(0.0, 1.0), 9.0)]),
        Cursor::ResizeHorizontal => double_arrows(painter, at, &[(Point::new(1.0, 0.0), 9.0)]),
        Cursor::ResizeVertical => double_arrows(painter, at, &[(Point::new(0.0, 1.0), 9.0)]),
        Cursor::ResizeDiagonal => double_arrows(painter, at, &[(Point::new(1.0, 1.0), 7.0)]),
        Cursor::ResizeAntidiagonal => double_arrows(painter, at, &[(Point::new(1.0, -1.0), 7.0)]),
        Cursor::Crosshair => {
            let strokes = [((-9.0, 0.0), (9.0, 0.0)), ((0.0, -9.0), (0.0, 9.0))];
            edged_lines(painter, &strokes.map(|((x0, y0), (x1, y1))| (point(x0, y0), point(x1, y1))));
        }
        Cursor::NotAllowed => {
            painter.ring(at, 8.0, 4.0, EDGE);
            painter.line(point(-5.0, -5.0), point(5.0, 5.0), 4.0, EDGE);
            painter.ring(at, 8.0, 2.0, INK);
            painter.line(point(-5.0, -5.0), point(5.0, 5.0), 2.0, INK);
        }
    }
}

// Pixel art, its top left at the point: each row's runs of ink and of the
// edge's color, a rectangle a run.
fn pixels(painter: &mut Painter, origin: Point, rows: &[&str]) {
    for (y, row) in rows.iter().enumerate() {
        let row = row.as_bytes();
        let mut start = 0;
        while start < row.len() {
            let end = start + row[start..].iter().take_while(|&&pixel| pixel == row[start]).count();
            let color = match row[start] {
                b'B' => Some(INK),
                b'W' => Some(EDGE),
                _ => None,
            };
            if let Some(color) = color {
                painter.fill(Rect::new(origin.x + start as f32, origin.y + y as f32, (end - start) as f32, 1.0), color);
            }
            start = end;
        }
    }
}

// Lines in ink, each over a wider one in the edge's color.
fn edged_lines(painter: &mut Painter, strokes: &[(Point, Point)]) {
    for (from, to) in strokes {
        painter.line(*from, *to, 3.0, EDGE);
    }
    for (from, to) in strokes {
        painter.line(*from, *to, 1.0, INK);
    }
}

// Arrows both ways through the point, each `reach` along its direction each
// way, with their heads: all their edges, then all their ink, so that one's
// edge never covers another's ink. A diagonal one has a wider edge, and
// heavier ink, which slant across pixels and would show only part of each.
fn double_arrows(painter: &mut Painter, at: Point, arrows: &[(Point, f32)]) {
    for edge in [true, false] {
        for &(direction, reach) in arrows {
            let diagonal = direction.x != 0.0 && direction.y != 0.0;
            let margin = match (edge, diagonal) {
                (false, _) => 0.0,
                (true, false) => 1.0,
                (true, true) => 1.75,
            };
            let ink = if diagonal { 1.5 } else { 1.0 };
            let color = if edge { EDGE } else { INK };
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
            painter.line(at_distance(-shaft, 0.0), at_distance(shaft, 0.0), ink + 2.0 * margin, color);
        }
    }
}
