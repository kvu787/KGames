// What every program's window has around its cards: an app bar at the top
// with the program's title, its frame rate, and a badge that says whether the
// window has focus; a status bar at the bottom with its shortcuts on
// keycaps, what the program says about itself, and the display that the
// window is on; notices about how the program was started, as info bars
// over the cards that the user can close; and, when the window isn't in a
// supported setup, a page that says what's wrong instead.

use canvas::{Align, Canvas, Color, Icon, Lines, Rect, TextStyle};

use crate::chips::{badge, badge_width, keycap};
use crate::interface::Layer;
use crate::surfaces::{Severity, info_bar_height, shadow};
use crate::tokens::*;
use crate::Frame;

// Notices are at most this wide, in the middle of the window.
const NOTICE_WIDTH: f32 = 960.0;
// The middle of the app bar's line.
const APP_BAR_MIDDLE: f32 = 28.0;

impl Frame<'_> {
    // The app bar: the program's title, and what it is after it, and on the
    // right, the frame rate and whether the window has focus. Under it, the
    // notices that the user hasn't closed.
    pub fn header(&mut self, title: &str, subtitle: &str) {
        let width = self.width;
        let focused = self.focused();
        let frames_per_second = self.status.frames_per_second;
        let theme = self.theme();
        let canvas = &mut self.canvas;
        let title_height = canvas.line_height(TextStyle::Subtitle);
        let top = APP_BAR_MIDDLE - title_height * 0.5;
        let end = canvas.text([MARGIN, top], title, TextStyle::Subtitle, theme.text);
        let baseline = top + canvas.fonts.style(TextStyle::Subtitle).baseline;

        // Whether input reaches the program, which it doesn't without focus.
        let (text, color) = if focused {
            ("Window has focus", theme.success)
        } else {
            ("No focus: input is ignored", theme.caution)
        };
        let badge_x = width - MARGIN - badge_width(canvas.fonts, text);
        badge(canvas, theme, [badge_x, APP_BAR_MIDDLE], text, color);

        // The frame rate is far above the monitor's refresh rate, as frames
        // don't wait for vertical sync.
        let rate = match frames_per_second {
            Some(rate) => format!("{rate:.0} FPS  {:.2} ms", 1000.0 / rate),
            None => "Measuring FPS".to_string(),
        };
        let rate_x = badge_x - SPACE_L - canvas.measure(&rate, TextStyle::Mono);
        canvas.text_on_baseline([rate_x, baseline], &rate, TextStyle::Mono, theme.text_secondary);
        if !subtitle.is_empty() {
            let x = end + SPACE_M;
            let fitted = canvas.fit(subtitle, TextStyle::Body, (rate_x - SPACE_XXL - x).max(0.0));
            canvas.text_on_baseline([x, baseline], &fitted, TextStyle::Body, theme.text_secondary);
        }
        self.notices();
    }

    // The notices, as info bars over the top of the cards, which a press on
    // their buttons closes. Each button is called "Notice/<index>".
    fn notices(&mut self) {
        let notices = self.status.notices;
        let width = NOTICE_WIDTH.min(self.width - MARGIN * 2.0);
        let x = ((self.width - width) * 0.5).round();
        let mut y = PANELS_TOP + SPACE_S;
        for (index, notice) in notices.iter().enumerate() {
            if self.state.dismissed.contains(&index) {
                continue;
            }
            let height = info_bar_height(&self.overlay, width, "", notice, true);
            let area = Rect::new(x, y, width, height);
            let theme = self.theme();
            shadow(&mut self.overlay, theme, area, RADIUS_CONTROL, FLYOUT);
            // Opaque, as it's over the cards.
            self.overlay.fill(area, RADIUS_CONTROL, theme.card);
            let name = format!("Notice/{index}");
            if self.info_bar_on(Layer::Overlay, Some(&name), area, Severity::Caution, "", notice).clicked {
                self.state.dismissed.push(index);
            }
            y += height + SPACE_S;
        }
    }

    // The status bar: the shortcuts, each a key and what it does, then
    // `status` on the right, before the display. Returns where the cards end
    // above it.
    pub fn footer(&mut self, shortcuts: &[(&str, &str)], status: &[(&str, Color)]) -> f32 {
        self.footer_with(shortcuts, |canvas, area| {
            let width: f32 = status.iter().map(|(text, _)| canvas.measure(text, TextStyle::Caption)).sum();
            let mut x = (area.right() - width).max(area.x);
            for (text, color) in status {
                x = canvas.text_clipped([x, area.y], text, (area.right() - x).max(0.0), TextStyle::Caption, *color);
            }
        })
    }

    // Like footer, with `status` drawing what the program says about itself
    // in the area between the shortcuts and the display, on a caption's line.
    pub fn footer_with(&mut self, shortcuts: &[(&str, &str)], status: impl FnOnce(&mut Canvas, Rect)) -> f32 {
        let (width, height) = (self.width, self.height);
        let display = self.status.display;
        let theme = self.theme();
        let canvas = &mut self.canvas;
        let middle = height - STATUS_BAR_HEIGHT * 0.5;
        let caption = canvas.line_height(TextStyle::Caption);
        let line_top = (middle - caption * 0.5).round();

        let display_width = canvas.measure(display, TextStyle::Caption);
        let display_x = width - MARGIN - display_width;
        canvas.text([display_x, line_top], display, TextStyle::Caption, theme.text_secondary);
        canvas.icon([display_x - SPACE_S - 8.0, middle], Icon::Display, 16.0, theme.text_secondary);
        let right = display_x - SPACE_S - 16.0 - SPACE_XXL;

        let mut x = MARGIN;
        for (keys, action) in shortcuts {
            for (index, key) in keys.split('+').enumerate() {
                if index > 0 {
                    x += SPACE_XXS;
                }
                x = keycap(canvas, theme, [x, middle], key).right();
            }
            x = canvas.text([x + 6.0, line_top], action, TextStyle::Caption, theme.text_secondary) + SPACE_L;
        }
        let area = Rect::from_edges(x + SPACE_S, line_top, right, line_top + caption);
        status(canvas, area);
        self.content_bottom()
    }

    // Where the cards end, above the status bar.
    pub fn content_bottom(&self) -> f32 {
        self.height - STATUS_BAR_HEIGHT
    }

    // Instead of the program's cards, when the window isn't in a supported
    // setup: `supported` says which setups are, and `problems` what's wrong.
    // Nothing that the frame drew before shows, and none of it takes input.
    pub fn show_window_problems(&mut self, supported: &str, problems: &[String]) {
        self.clear();
        let width = self.width;
        let theme = self.theme();
        let canvas = &mut self.canvas;
        let card_width = 880.0f32.min(width - MARGIN * 2.0);
        let text_width = card_width - SPACE_XXXL * 2.0;
        let body = canvas.line_height(TextStyle::Body);
        let supported_lines = canvas.fonts.wrap(supported, TextStyle::Body, text_width, 0.0).len();
        let problem_lines: usize = problems
            .iter()
            .map(|problem| canvas.fonts.wrap(problem, TextStyle::Body, text_width - 24.0, 0.0).len())
            .sum();
        let height = SPACE_XXXL * 2.0
            + canvas.line_height(TextStyle::Title)
            + SPACE_L
            + body * (supported_lines + problem_lines) as f32
            + SPACE_L
            + SPACE_S * problems.len() as f32;
        let area = Rect::new(((width - card_width) * 0.5).round(), 120.0, card_width, height);
        canvas.fill_stroke(area, RADIUS_CARD, theme.card, 1.0, theme.card_stroke);
        let inner = area.inset(SPACE_XXXL);
        let title = Rect::new(inner.x, inner.y, inner.width, canvas.line_height(TextStyle::Title));
        canvas.icon([inner.x + 8.0, title.center()[1]], Icon::Error, 16.0, theme.critical);
        let title_text = Rect::from_edges(inner.x + 16.0 + SPACE_M, title.y, title.right(), title.bottom());
        canvas.text_in(title_text, "Unsupported window", TextStyle::Title, theme.text, Align::Start);
        let mut lines = Lines::new(Rect::from_edges(inner.x, title.bottom() + SPACE_L, inner.right(), area.bottom()));
        lines.wrapped(canvas, supported, TextStyle::Body, theme.text_secondary, 0.0);
        lines.gap(SPACE_L);
        for problem in problems {
            let y = lines.y;
            canvas.icon([lines.x + 8.0, y + body * 0.5], Icon::Warning, 16.0, theme.caution);
            let mut problem_lines = Lines::new(Rect::from_edges(lines.x + 24.0, y, lines.right(), lines.bottom));
            problem_lines.wrapped(canvas, problem, TextStyle::Body, theme.text, 0.0);
            lines.y = problem_lines.y + SPACE_S;
        }
    }
}
