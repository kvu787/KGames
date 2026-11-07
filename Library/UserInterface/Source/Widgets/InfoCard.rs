// The card that a test program floats over its scene's bottom left corner,
// which the scene never crosses: the program's name and what it shows, the
// frame rate, the GPU, the display, whether the window has focus, the keys,
// and what's wrong with Settings.json, if anything. MinimalSceneTest and
// SpinningCarTest show one, and F1 shows and hides it.

use crate::application::program::Notice;
use crate::core::ui::Ui;
use crate::design::tokens::{
    BODY, CARD_HEADER_GAP, CARD_HEADER_HEIGHT, CARD_PADDING, ICON, PAGE_PADDING, SPACE_L, SPACE_M, SPACE_S, SPACE_XS,
};
use crate::drawing::geometry::{Insets, Point, Rect};
use crate::input::devices::Key;
use crate::input::events::Input;
use crate::widgets::card::Card;
use crate::widgets::info_bar::{InfoBar, Severity};
use crate::widgets::text::KEYCAP_HEIGHT;

// The column of the fields' labels.
const LABEL_WIDTH: f32 = 96.0;

// What the card says about the program.
pub struct InfoCard<'a> {
    pub title: &'a str,
    // What the scene shows, like "A sphere moving at 8 units per second".
    pub description: &'a str,
    // Fields after the display's, each a label and its value.
    pub rows: &'a [(&'a str, &'a str)],
    // The program's own keys, each with what it does, on a row under the
    // library's: F1's, F9's, F10's and F11's.
    pub keys: &'a [(&'a str, &'a str)],
}

impl InfoCard<'_> {
    // The card's name, which tests and scripts find it by.
    pub const NAME: &'static str = "InfoCard";
    pub const WIDTH: f32 = 480.0;

    // Whether F1 went down this frame, which shows or hides the card.
    // Holding it down repeats it, which does nothing more.
    pub fn toggled(input: &Input) -> bool {
        input.key_pressed_once(Key::F1)
    }
}

fn info_bar(notice: &Notice) -> InfoBar<'_> {
    InfoBar::new(notice.severity, &notice.title, &notice.message)
}

impl Ui {
    // Draws the card at the bottom left of the window, as tall as what it
    // says.
    pub fn info_card(&mut self, contents: &InfoCard) {
        let stats = self.stats();
        let environment = self.environment().clone();
        let rate = if stats.frames_per_second > 0.0 {
            format!("{:.0} frames a second, {:.2} ms each", stats.frames_per_second, stats.frame_time)
        } else {
            "Measuring".to_string()
        };
        let display = format!("{} {}", environment.display, environment.display_name);
        let mut rows = vec![("Frame rate", rate.as_str()), ("GPU", environment.gpu.as_str()), ("Display", display.trim())];
        rows.extend_from_slice(contents.rows);
        let library_keys: &[(&str, &str)] = &[("F1", "Hide this"), ("F9", "Theme"), ("F10", "VSync"), ("F11", "Fullscreen")];
        let key_rows: Vec<&[(&str, &str)]> = [library_keys, contents.keys].into_iter().filter(|keys| !keys.is_empty()).collect();
        let notices = self.notices().to_vec();
        let width = InfoCard::WIDTH;
        let notice_heights: Vec<f32> =
            notices.iter().map(|notice| self.info_bar_height(width - CARD_PADDING * 2.0, &info_bar(notice))).collect();

        // The card sizes itself to its content, which goes from its top, so
        // its top is as far above the window's bottom as the card is tall:
        // under its header, the description, the fields, the window's focus
        // and the rows of keys, SPACE_M apart, then each notice, SPACE_M under
        // what's above it.
        let line = BODY.line_height;
        let fields = rows.len() as f32 * (line + SPACE_XS) - SPACE_XS;
        let keys_height = key_rows.len() as f32 * (SPACE_M + KEYCAP_HEIGHT);
        let notices_height: f32 = notice_heights.iter().map(|height| SPACE_M + height).sum();
        let content = line + SPACE_M + fields + SPACE_M + line + keys_height + notices_height;
        let height = CARD_PADDING * 2.0 + CARD_HEADER_HEIGHT + CARD_HEADER_GAP + content;
        let rect = Rect::new(PAGE_PADDING, self.size().height - PAGE_PADDING - height, width, height);
        let card = self.begin_card(rect, Card::new(contents.title).floating().named(InfoCard::NAME).label(contents.title));

        let mut area = card.area.content;
        let secondary = self.theme().text_secondary;
        self.text(area.cut_top(line, SPACE_M), contents.description, BODY, secondary);
        for (label, value) in &rows {
            self.field(area.cut_top(line, SPACE_XS), label, LABEL_WIDTH, value);
        }
        area.cut_top(SPACE_M - SPACE_XS, 0.0);

        // Whether the window has focus, with an info bar's status icon.
        let focus = area.cut_top(line, 0.0);
        let (severity, says) = if self.window_focused() {
            (Severity::Success, "The window has focus")
        } else {
            (Severity::Warning, "No focus, so input is ignored")
        };
        self.status_icon(Point::new(focus.x + ICON / 2.0, focus.center().y), severity, ICON);
        self.label(focus.pad(Insets::new(ICON + SPACE_S, 0.0, 0.0, 0.0)), says);

        // Each key on its keycap, and what it does.
        for keys in key_rows {
            area.cut_top(SPACE_M, 0.0);
            let row = area.cut_top(KEYCAP_HEIGHT, 0.0);
            let mut x = row.x;
            for (key, action) in keys {
                x += self.keys(Point::new(x, row.y), key) + SPACE_S;
                x = self.caption(Rect::new(x, row.y, (row.right() - x).max(0.0), row.height), action).right() + SPACE_L;
            }
        }

        for (index, (notice, height)) in notices.iter().zip(notice_heights).enumerate() {
            area.cut_top(SPACE_M, 0.0);
            self.add(&format!("Notice/{index}"), area.cut_top(height, 0.0), info_bar(notice));
        }
        self.end_card(card, area.y);
    }
}
