// The card that floats over the scene's bottom left corner, which the scene
// never crosses: the test's name and what it shows, the frame rate, the GPU,
// the display, whether the window has focus, the keys, and what's wrong with
// Settings.json, if anything. F1 shows and hides it. SpinningCarTest has the
// same card.

use user_interface::tokens::{
    BODY, CAPTION, CARD_HEADER_GAP, CARD_HEADER_HEIGHT, CARD_PADDING, ICON, PAGE_PADDING, SPACE_L, SPACE_M, SPACE_S,
    SPACE_XS,
};
use user_interface::{Icon, InfoBar, Input, Key, Notice, Point, Rect, Sense, Ui};

// The card's name, which tests and scripts find it by.
pub const NAME: &str = "InfoCard";
pub const WIDTH: f32 = 480.0;
// The column of the rows' labels.
const LABEL_WIDTH: f32 = 96.0;
const KEYCAP_HEIGHT: f32 = 22.0;

// What the card says about the test.
pub struct Contents<'a> {
    pub title: &'a str,
    // What the scene shows, like "A sphere moving at 8 units per second".
    pub description: &'a str,
    // Rows after the display's, each a label and its value.
    pub rows: &'a [(&'a str, &'a str)],
    // Keys after F1's and F11's, each with what it does.
    pub keys: &'a [(&'a str, &'a str)],
}

// Whether F1 went down this frame, which shows or hides the card. Holding it
// down repeats it, which does nothing more.
pub fn toggled(input: &Input) -> bool {
    input.key_pressed_once(Key::F1)
}

fn info_bar(notice: &Notice) -> InfoBar<'_> {
    InfoBar::new(notice.severity, &notice.title, &notice.message)
}

// Draws the card at the bottom left of the window, as tall as what it says.
pub fn draw(ui: &mut Ui, contents: &Contents) {
    let theme = ui.theme().clone();
    let stats = ui.stats();
    let environment = ui.environment().clone();
    let rate = if stats.frames_per_second > 0.0 {
        format!("{:.0} frames a second, {:.2} ms each", stats.frames_per_second, stats.frame_time)
    } else {
        "Measuring".to_string()
    };
    let display = format!("{} {}", environment.display, environment.display_name);
    let mut rows = vec![("Frame rate", rate.as_str()), ("GPU", environment.gpu.as_str()), ("Display", display.trim())];
    rows.extend_from_slice(contents.rows);
    let mut keys = vec![("F1", "Hide this"), ("F11", "Fullscreen")];
    keys.extend_from_slice(contents.keys);
    let notices = ui.notices().to_vec();
    let content_width = WIDTH - CARD_PADDING * 2.0;
    let notice_heights: Vec<f32> = notices.iter().map(|notice| ui.info_bar_height(content_width, &info_bar(notice))).collect();

    let line = BODY.line_height;
    let height = CARD_PADDING * 2.0
        + CARD_HEADER_HEIGHT
        + CARD_HEADER_GAP
        + line
        + SPACE_M
        + rows.len() as f32 * (line + SPACE_XS)
        - SPACE_XS
        + SPACE_M
        + line
        + SPACE_M
        + KEYCAP_HEIGHT
        + notice_heights.iter().map(|height| SPACE_M + height).sum::<f32>();
    let rect = Rect::new(PAGE_PADDING, ui.size().height - PAGE_PADDING - height, WIDTH, height);
    let card = ui.floating_card(rect, contents.title);
    ui.register(NAME, rect, Sense::NONE, "Card");
    ui.set_label(NAME, contents.title);

    let mut area = card.content;
    ui.text(area.cut_top(line, SPACE_M), contents.description, BODY, theme.text_secondary);
    for (label, value) in &rows {
        ui.field(area.cut_top(line, SPACE_XS), label, LABEL_WIDTH, value);
    }
    area.cut_top(SPACE_M - SPACE_XS, 0.0);

    // Whether the window has focus, with a status icon like an info bar's.
    let focus = area.cut_top(line, SPACE_M);
    let (color, mark, says) = if ui.window_focused() {
        (theme.success, Icon::StatusSuccess, "The window has focus")
    } else {
        (theme.caution, Icon::StatusWarning, "No focus, so input is ignored")
    };
    let mut painter = ui.painter();
    let center = Point::new(focus.x + ICON / 2.0, focus.center().y);
    painter.icon(center, Icon::StatusCircle, ICON, color);
    painter.icon(center, mark, ICON, theme.text_on_accent);
    ui.text(focus.inset_by(ICON + SPACE_S, 0.0, 0.0, 0.0), says, BODY, theme.text_primary);

    let row = area.cut_top(KEYCAP_HEIGHT, 0.0);
    let mut x = row.x;
    for (key, action) in keys {
        x += ui.keycap(Point::new(x, row.y), key) + SPACE_S;
        let text = Rect::new(x, row.y, (row.right() - x).max(0.0), KEYCAP_HEIGHT);
        x = ui.text(text, action, CAPTION, theme.text_secondary).right() + SPACE_L;
    }

    for (index, (notice, height)) in notices.iter().zip(notice_heights).enumerate() {
        area.cut_top(SPACE_M, 0.0);
        ui.info_bar(&format!("Notice/{index}"), area.cut_top(height, 0.0), info_bar(notice));
    }
}
