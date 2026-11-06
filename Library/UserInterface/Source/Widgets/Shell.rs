// The page that every program's window shows: an app bar along the top, as
// tall as WinUI's TitleBar control, with the program's icon, title and its
// own commands; notices under it; a status bar along the bottom with
// shortcuts and status; and the page between them, where the cards go.

use crate::application::program::Notice;
use crate::core::ui::Ui;
use crate::design::icons::Icon;
use crate::design::tokens::{
    APP_BAR_HEIGHT, BODY, CAPTION, ICON, PAGE_PADDING, RADIUS_CONTROL, SPACE_L, SPACE_M, SPACE_S, SPACE_XS, STATUS_BAR_HEIGHT,
    SUBTITLE,
};
use crate::drawing::geometry::{Point, Rect, Size};
use crate::drawing::painter::{Align, Border};
use crate::widgets::info_bar::{InfoBar, Severity};
use crate::widgets::text::KEYCAP_HEIGHT;

#[derive(Clone, Copy, Debug)]
pub struct AppBar<'a> {
    icon: Icon,
    title: &'a str,
    subtitle: Option<&'a str>,
}

impl<'a> AppBar<'a> {
    pub fn new(icon: Icon, title: &'a str) -> Self {
        Self { icon, title, subtitle: None }
    }

    pub fn subtitle(mut self, subtitle: &'a str) -> Self {
        self.subtitle = Some(subtitle);
        self
    }
}

// What a status bar shows, from its left, then from its right after a
// spacer.
#[derive(Clone, Copy, Debug)]
pub enum Status<'a> {
    // Keys, like "Ctrl+S", as keycaps, and what they do.
    Shortcut(&'a str, &'a str),
    Text(&'a str),
    // A dot in a status's color, and what it says.
    Indicator(Severity, &'a str),
    // What comes after goes along the right.
    Spacer,
}

impl Ui {
    // The app bar: the program's icon and title, and the rest of the bar,
    // which it returns, for the program's own tabs and commands.
    pub fn app_bar(&mut self, bar: AppBar) -> Rect {
        let rect = Rect::new(0.0, 0.0, self.size().width, APP_BAR_HEIGHT);
        self.cover(rect);
        let mut painter = self.painter();
        let theme = painter.theme();
        // Opaque, like Mica's solid stand-in, so that it covers a 3D view.
        painter.fill(rect, theme.background);
        let icon_center = Point::new(SPACE_L + ICON / 2.0, rect.center().y);
        painter.icon(icon_center, bar.icon, ICON, theme.accent);
        let mut x = SPACE_L + ICON + SPACE_L;
        let title = painter.text_in(Rect::new(x, rect.y, rect.width, rect.height), bar.title, CAPTION, theme.text_primary, Align::Start);
        x = title.right();
        if let Some(subtitle) = bar.subtitle {
            let shown = painter.text_in(Rect::new(x + SPACE_S, rect.y, rect.width, rect.height), subtitle, CAPTION, theme.text_secondary, Align::Start);
            x = shown.right();
        }
        self.page_top = APP_BAR_HEIGHT;
        let left = x + SPACE_L;
        Rect::new(left, rect.y + SPACE_S, rect.right() - SPACE_L - left, rect.height - SPACE_S * 2.0)
    }

    // The status bar along the bottom.
    pub fn status_bar(&mut self, items: &[Status]) {
        let size = self.size();
        let rect = Rect::new(0.0, size.height - STATUS_BAR_HEIGHT, size.width, STATUS_BAR_HEIGHT);
        self.page_bottom = rect.y;
        self.cover(rect);
        let mut painter = self.painter();
        let theme = painter.theme();
        painter.fill(rect, theme.background);
        painter.fill(rect.top_strip(1.0), theme.divider);
        let spacer = items.iter().position(|item| matches!(item, Status::Spacer)).unwrap_or(items.len());
        let (left, right) = items.split_at(spacer);
        let right = right.get(1..).unwrap_or(&[]);
        // The right's items, from the right edge in, so that the left's can
        // be cut short where they'd meet them.
        let mut right_edge = rect.right() - SPACE_L;
        for item in right.iter().rev() {
            let width = status_width(self, item);
            draw_status(self, item, Rect::new(right_edge - width, rect.y + 1.0, width, rect.height - 1.0));
            right_edge -= width + SPACE_L + SPACE_S;
        }
        let mut x = rect.x + SPACE_L;
        for item in left {
            let width = status_width(self, item).min((right_edge - x).max(0.0));
            if width <= 0.0 {
                break;
            }
            draw_status(self, item, Rect::new(x, rect.y + 1.0, width, rect.height - 1.0));
            x += width + SPACE_L + SPACE_S;
        }
    }

    // The page between the app bar and the status bar, inside its padding,
    // after the notices, which it shows. Call it after the bars.
    pub fn page(&mut self) -> Rect {
        let width = self.size().width;
        let mut page = Rect::new(PAGE_PADDING, self.page_top + PAGE_PADDING, width - PAGE_PADDING * 2.0, self.page_bottom - self.page_top - PAGE_PADDING * 2.0);
        let notices = self.notices.clone();
        let mut closed = None;
        for (index, notice) in notices.iter().enumerate() {
            let bar = notice_bar(notice);
            let height = self.info_bar_height(page.width, &bar);
            let rect = page.cut_top(height, SPACE_S);
            if self.add(&format!("Notice/{index}"), rect, bar).closed {
                closed = Some(index);
            }
        }
        if !notices.is_empty() {
            page.cut_top(SPACE_S, 0.0);
        }
        if let Some(index) = closed {
            self.notices.remove(index);
        }
        page
    }
}

fn notice_bar(notice: &Notice) -> InfoBar<'_> {
    InfoBar::new(notice.severity, &notice.title, &notice.message).closable()
}

fn status_width(ui: &mut Ui, item: &Status) -> f32 {
    match item {
        Status::Shortcut(keys, label) => ui.keys_width(keys) + SPACE_S + ui.measure(label, CAPTION).width,
        Status::Text(text) => ui.measure(text, CAPTION).width,
        Status::Indicator(_, text) => 8.0 + SPACE_S + ui.measure(text, CAPTION).width,
        Status::Spacer => 0.0,
    }
}

fn draw_status(ui: &mut Ui, item: &Status, rect: Rect) {
    match item {
        Status::Shortcut(keys, label) => {
            let width = ui.keys(Point::new(rect.x, rect.center().y - KEYCAP_HEIGHT / 2.0), keys);
            let mut painter = ui.painter();
            let theme = painter.theme();
            let label_rect = Rect::new(rect.x + width + SPACE_S, rect.y, (rect.width - width - SPACE_S).max(0.0), rect.height);
            painter.text_fit(label_rect, label, CAPTION, theme.text_secondary, Align::Start);
        }
        Status::Text(text) => {
            let mut painter = ui.painter();
            let theme = painter.theme();
            painter.text_fit(rect, text, CAPTION, theme.text_secondary, Align::Start);
        }
        Status::Indicator(severity, text) => {
            let mut painter = ui.painter();
            let theme = painter.theme();
            painter.circle(Point::new(rect.x + 4.0, rect.center().y), 4.0, theme.status(*severity));
            let label_rect = Rect::new(rect.x + 8.0 + SPACE_S, rect.y, (rect.width - 8.0 - SPACE_S).max(0.0), rect.height);
            painter.text_fit(label_rect, text, CAPTION, theme.text_secondary, Align::Start);
        }
        Status::Spacer => {}
    }
}

impl Ui {
    // What the window shows instead of the program in a setup that programs
    // don't support: what's wrong, and the setups that are.
    pub fn unsupported_setup(&mut self, problems: &[String]) {
        unsupported_setup(self, problems);
    }
}

fn unsupported_setup(ui: &mut Ui, problems: &[String]) {
    let bounds = ui.bounds();
    let width = 720.0f32.min(bounds.width - SPACE_L * 2.0);
    let mut height = SPACE_L * 2.0 + 28.0 + SPACE_M + SPACE_S;
    let mut lines = Vec::new();
    for problem in problems {
        let line = ui.fonts().measure_wrapped(problem, BODY, width - SPACE_L * 2.0 - SPACE_M).height;
        lines.push(line);
        height += line + SPACE_XS;
    }
    let advice = "Programs run on a 2560 × 1440 display at 100% scale, with a 48-pixel taskbar at its bottom, maximized or in borderless fullscreen with F11.";
    let advice_height = ui.fonts().measure_wrapped(advice, BODY, width - SPACE_L * 2.0).height;
    height += SPACE_S + advice_height;
    let rect = bounds.centered(Size::new(width, height)).round();
    let mut painter = ui.painter();
    let theme = painter.theme();
    painter.fill(bounds, theme.background);
    painter.shape(rect, RADIUS_CONTROL * 2.0, theme.critical_background, Some(Border::new(theme.card_stroke, 1.0)));
    let mut area = rect.inset(SPACE_L);
    let title = area.cut_top(28.0, SPACE_M);
    painter.text_in(title, "This window setup isn't supported", SUBTITLE, theme.text_primary, Align::Start);
    for (problem, line) in problems.iter().zip(lines) {
        let row = area.cut_top(line, SPACE_XS);
        painter.circle(Point::new(row.x + 3.0, row.y + 10.0), 2.5, theme.critical);
        painter.text_wrapped(Rect::new(row.x + SPACE_M, row.y, row.width - SPACE_M, row.height), problem, BODY, theme.text_primary);
    }
    area.cut_top(SPACE_S, 0.0);
    painter.text_wrapped(area, advice, BODY, theme.text_secondary);
}
