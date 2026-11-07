// The card that a test program floats over its scene's bottom left corner,
// which the scene never crosses: the program's name and what it shows, the
// frame rate, the GPU, the display, whether the window has focus, the keys,
// and what's wrong with Settings.json, if anything. MinimalSceneTest and
// SpinningCarTest show one, and F1 shows and hides it, with
// ui.set_visible(InfoCard::NAME, ...).

use crate::core::control::{AutoTarget, Control, NodeContext};
use crate::core::ui::{Sense, Ui};
use crate::design::tokens::{
    BODY, CARD_HEADER_GAP, CARD_HEADER_HEIGHT, CARD_PADDING, ICON, PAGE_PADDING, SPACE_L, SPACE_M, SPACE_S, SPACE_XS,
};
use crate::drawing::geometry::{Insets, Point, Rect};
use crate::input::devices::Key;
use crate::input::events::Input;
use crate::widgets::card::{Card, CardStyle};
use crate::widgets::info_bar::{InfoBar, Severity};
use crate::widgets::text::KEYCAP_HEIGHT;

// The column of the fields' labels.
const LABEL_WIDTH: f32 = 96.0;

// What the card says about the program. The rows and keys are the program's
// own, which it updates with ui.get_mut::<InfoCard>(InfoCard::NAME).
pub struct InfoCard {
    pub title: String,
    // What the scene shows, like "A sphere moving at 8 units per second".
    pub description: String,
    // Fields after the display's, each a label and its value.
    pub rows: Vec<(String, String)>,
    // Keys after F1's, F10's and F11's, each with what it does.
    pub keys: Vec<(String, String)>,
    // The frame's computed layout, from handle, for draw.
    rect: Rect,
    notice_bars: Vec<(InfoBar, f32)>,
}

impl InfoCard {
    // The card's name, which tests and scripts find it by.
    pub const NAME: &'static str = "InfoCard";
    pub const WIDTH: f32 = 480.0;

    pub fn new(title: &str, description: &str) -> Self {
        Self {
            title: title.to_string(),
            description: description.to_string(),
            rows: Vec::new(),
            keys: Vec::new(),
            rect: Rect::default(),
            notice_bars: Vec::new(),
        }
    }

    pub fn rows(mut self, rows: &[(&str, &str)]) -> Self {
        self.rows = rows.iter().map(|(label, value)| (label.to_string(), value.to_string())).collect();
        self
    }

    pub fn keys(mut self, keys: &[(&str, &str)]) -> Self {
        self.keys = keys.iter().map(|(key, action)| (key.to_string(), action.to_string())).collect();
        self
    }

    pub fn set_rows(&mut self, rows: Vec<(String, String)>) {
        self.rows = rows;
    }

    // Whether F1 went down this frame, which shows or hides the card.
    // Holding it down repeats it, which does nothing more.
    pub fn toggled(input: &Input) -> bool {
        input.key_pressed_once(Key::F1)
    }

    // The standard rows before the program's own.
    fn all_rows(&self, ui: &mut Ui) -> Vec<(String, String)> {
        let stats = ui.stats();
        let environment = ui.environment().clone();
        let rate = if stats.frames_per_second > 0.0 {
            format!("{:.0} frames a second, {:.2} ms each", stats.frames_per_second, stats.frame_time)
        } else {
            "Measuring".to_string()
        };
        let display = format!("{} {}", environment.display, environment.display_name);
        let mut rows = vec![
            ("Frame rate".to_string(), rate),
            ("GPU".to_string(), environment.gpu.clone()),
            ("Display".to_string(), display.trim().to_string()),
        ];
        rows.extend(self.rows.iter().cloned());
        rows
    }

    fn all_keys(&self) -> Vec<(String, String)> {
        let mut keys = vec![
            ("F1".to_string(), "Hide this".to_string()),
            ("F10".to_string(), "VSync".to_string()),
            ("F11".to_string(), "Fullscreen".to_string()),
        ];
        keys.extend(self.keys.iter().cloned());
        keys
    }
}

impl Control for InfoCard {
    fn kind(&self) -> &'static str {
        "Card"
    }

    fn auto_target(&self) -> AutoTarget {
        AutoTarget::None
    }

    fn handle(&mut self, ui: &mut Ui, ctx: &NodeContext) {
        // The card sizes itself to its content, which goes from its top, so
        // its top is as far above the window's bottom as the card is tall:
        // under its header, the description, the fields, the window's focus
        // and the keys, SPACE_M apart, then each notice, SPACE_M under what's
        // above it.
        let rows = self.all_rows(ui);
        let notices = ui.notices().to_vec();
        let width = InfoCard::WIDTH;
        self.notice_bars = notices
            .iter()
            .map(|notice| {
                let bar = InfoBar::new(notice.severity, &notice.title, &notice.message);
                let height = bar.height(ui, width - CARD_PADDING * 2.0);
                (bar, height)
            })
            .collect();
        let line = BODY.line_height;
        let fields = rows.len() as f32 * (line + SPACE_XS) - SPACE_XS;
        let notices_height: f32 = self.notice_bars.iter().map(|(_, height)| SPACE_M + height).sum();
        let content = line + SPACE_M + fields + SPACE_M + line + SPACE_M + KEYCAP_HEIGHT + notices_height;
        let height = CARD_PADDING * 2.0 + CARD_HEADER_HEIGHT + CARD_HEADER_GAP + content;
        self.rect = Rect::new(PAGE_PADDING, ui.size().height - PAGE_PADDING - height, width, height);
        ui.register(InfoCard::NAME, self.rect, Sense::NONE, "Card");
        ui.set_label(InfoCard::NAME, &self.title.clone());
        // The notices' targets, under their names, where draw will put them.
        let card = Card::new(&self.title).floating();
        let area = card.area(ui, self.rect);
        let mut top = area.content.y + line + SPACE_M + fields + SPACE_M + line + SPACE_M + KEYCAP_HEIGHT;
        let mut bars = std::mem::take(&mut self.notice_bars);
        for (index, (bar, bar_height)) in bars.iter_mut().enumerate() {
            top += SPACE_M;
            let rect = Rect::new(area.content.x, top, area.content.width, *bar_height);
            bar.input(ui, &format!("Notice/{index}"), rect);
            top += *bar_height;
        }
        self.notice_bars = bars;
        let _ = ctx;
    }

    fn draw(&mut self, ui: &mut Ui, ctx: &NodeContext) {
        let _ = ctx;
        let mut card = Card::new(&self.title).style(CardStyle::Floating);
        let area = card.area(ui, self.rect);
        let card_ctx = NodeContext {
            name: InfoCard::NAME.to_string(),
            rect: self.rect,
            response: Default::default(),
        };
        card.draw(ui, &card_ctx);

        let mut area_rect = area.content;
        let line = BODY.line_height;
        let secondary = ui.theme().text_secondary;
        let description = self.description.clone();
        ui.text(area_rect.cut_top(line, SPACE_M), &description, BODY, secondary);
        for (label, value) in self.all_rows(ui) {
            ui.field(area_rect.cut_top(line, SPACE_XS), &label, LABEL_WIDTH, &value);
        }
        area_rect.cut_top(SPACE_M - SPACE_XS, 0.0);

        // Whether the window has focus, with an info bar's status icon.
        let focus = area_rect.cut_top(line, SPACE_M);
        let (severity, says) = if ui.window_focused() {
            (Severity::Success, "The window has focus")
        } else {
            (Severity::Warning, "No focus, so input is ignored")
        };
        ui.status_icon(Point::new(focus.x + ICON / 2.0, focus.center().y), severity, ICON);
        ui.label(focus.pad(Insets::new(ICON + SPACE_S, 0.0, 0.0, 0.0)), says);

        // Each key on its keycap, and what it does.
        let row = area_rect.cut_top(KEYCAP_HEIGHT, 0.0);
        let mut x = row.x;
        for (key, action) in self.all_keys() {
            x += ui.keys(Point::new(x, row.y), &key) + SPACE_S;
            x = ui.caption(Rect::new(x, row.y, (row.right() - x).max(0.0), row.height), &action).right() + SPACE_L;
        }

        let mut bars = std::mem::take(&mut self.notice_bars);
        for (bar, height) in &mut bars {
            area_rect.cut_top(SPACE_M, 0.0);
            let rect = area_rect.cut_top(*height, 0.0);
            bar.paint(ui, rect);
        }
        self.notice_bars = bars;
    }
}
