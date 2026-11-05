// What every program's window has around its panels: a header with the
// program's name, whether the window has focus, a status line with the frame
// rate, and notices about how the program was started; a footer with its
// shortcuts and the display that the window is on; and, when the window
// isn't in a supported setup, a page that says what's wrong instead.

use canvas::theme::{DIM, ERROR, SUCCESS, TEXT, TITLE, WARNING};
use canvas::{Canvas, Color, Lines, Rect, wrap};

use crate::Frame;
use crate::panels::{EDGE, MARGIN, NOTICE_LINES, pieces};

impl Frame<'_> {
    // The header: the title, and what the program is after it. Under them,
    // the status line, with the frame rate on the right, and `status` on the
    // left, cut short to leave room for it. Under that, the notices.
    pub fn header(&mut self, title: &str, subtitle: &str, status: &[(&str, Color)]) {
        self.header_with(title, subtitle, |canvas, area| {
            pieces(canvas, area.x, area.y, status, area.right());
        });
    }

    // Like header, with `status` drawing the left of the status line in the
    // area it's given.
    pub fn header_with(&mut self, title: &str, subtitle: &str, status: impl FnOnce(&mut Canvas, Rect)) {
        let width = self.width;
        let focused = self.focused();
        let frames_per_second = self.status.frames_per_second;
        let notices = self.status.notices;
        let canvas = &mut self.canvas;
        let line_height = canvas.line_height();
        let char_width = canvas.char_width();
        let x = canvas.text(EDGE, EDGE, title, TITLE);
        if !subtitle.is_empty() {
            canvas.text(x, EDGE, &format!("  {subtitle}"), DIM);
        }
        let focus_text = if focused {
            "Window has focus"
        } else {
            "Window doesn't have focus: input is ignored"
        };
        canvas.text(
            width - EDGE - canvas.text_width(focus_text),
            EDGE,
            focus_text,
            if focused { SUCCESS } else { WARNING },
        );

        // The frame rate is far above the monitor's refresh rate, as frames
        // don't wait for vertical sync.
        let status_y = EDGE + line_height;
        let (rate, frame_time) = match frames_per_second {
            Some(rate) => (format!("{rate:.0}"), format!("  ({:.2} ms per frame)", 1000.0 / rate)),
            None => ("measuring".to_string(), String::new()),
        };
        let rate_x = width - EDGE - canvas.text_width(&format!("FPS: {rate}{frame_time}"));
        let mut end = canvas.text(rate_x, status_y, "FPS: ", DIM);
        end = canvas.text(end, status_y, &rate, TEXT);
        canvas.text(end, status_y, &frame_time, DIM);
        status(
            canvas,
            Rect::new(EDGE, status_y, rate_x - char_width * 3.0 - EDGE, line_height),
        );

        // Notices go under the status, wrapped to the window's width, in the
        // lines the header has for them.
        let columns = ((width - EDGE * 2.0) / char_width).floor() as usize;
        let mut lines: Vec<String> = Vec::new();
        for notice in notices {
            for (index, line) in wrap(notice, columns, 2).into_iter().enumerate() {
                lines.push(if index == 0 { line } else { format!("  {line}") });
            }
        }
        if lines.len() > NOTICE_LINES {
            lines.truncate(NOTICE_LINES);
            lines[NOTICE_LINES - 1] += " ...";
        }
        for (index, line) in lines.iter().enumerate() {
            let y = status_y + line_height * (index + 1) as f32;
            canvas.text_clipped(EDGE, y, line, width - EDGE * 2.0, WARNING);
        }
    }

    // The footer: the shortcuts, each a key and what it does, then `note`,
    // and the display on the right. Returns where the panels end above it.
    pub fn footer(&mut self, shortcuts: &[(&str, &str)], note: &str) -> f32 {
        let width = self.width;
        let footer_y = self.footer_top();
        let display = self.status.display;
        let canvas = &mut self.canvas;
        let char_width = canvas.char_width();
        let display_x = width - EDGE - canvas.text_width(&format!("Display: {display}"));
        let end = canvas.text(display_x, footer_y, "Display: ", DIM);
        canvas.text(end, footer_y, display, TEXT);
        let right = display_x - char_width * 3.0;
        let mut x = EDGE;
        for (index, (key, action)) in shortcuts.iter().enumerate() {
            let separator = if index == 0 { "" } else { "   " };
            x = pieces(canvas, x, footer_y, &[(separator, DIM), (key, TEXT), (&format!(" {action}"), DIM)], right);
        }
        if !note.is_empty() {
            let separator = if shortcuts.is_empty() { "" } else { "   " };
            pieces(canvas, x, footer_y, &[(&format!("{separator}{note}"), DIM)], right);
        }
        self.content_bottom()
    }

    // The top of the footer's line of text.
    pub fn footer_top(&self) -> f32 {
        self.height - EDGE - self.line_height()
    }

    // Where the panels end, above the footer.
    pub fn content_bottom(&self) -> f32 {
        self.footer_top() - EDGE
    }

    // Instead of the program's panels, when the window isn't in a supported
    // setup: `supported` says which setups are, and `problems` what's wrong.
    // Nothing that the frame drew before shows, and none of it takes input.
    pub fn show_window_problems(&mut self, supported: &str, problems: &[String]) {
        self.clear();
        let width = self.width;
        let canvas = &mut self.canvas;
        let line_height = canvas.line_height();
        let area = Rect::new(MARGIN, MARGIN, width - MARGIN * 2.0, f32::INFINITY);
        let mut lines = Lines::new(area, line_height);
        lines.line(canvas, "Unsupported window", ERROR);
        lines.gap(0.5);
        lines.wrapped(canvas, supported, "", TEXT);
        lines.gap(0.5);
        for problem in problems {
            lines.wrapped(canvas, &format!("- {problem}"), "  ", WARNING);
        }
    }
}
