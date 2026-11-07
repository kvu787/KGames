// The page that every program's window shows: the PerfBar along the very
// top, which the library draws; an app bar under it, as tall as WinUI's
// TitleBar control, with the program's icon, title and its own
// commands; a menu row under that, if the program wants one, for a menu bar
// whose menus' names line up with the page's left edge; notices under them; a
// status bar along the bottom with shortcuts and status; and the page between
// them, where the cards go.
//
// Program::build makes them as nodes with ui.app_bar, ui.menu_row,
// ui.status_bar and ui.page, in that order, which also lay the page out.

use crate::application::performance::{Run, SEPARATOR};
use crate::application::program::Notice;
use crate::core::control::{AutoTarget, Control, NodeContext};
use crate::core::ui::Ui;
use crate::design::controls::{MENU_BAR_HEIGHT, MENU_BAR_ITEM_MARGIN, MENU_BAR_ITEM_PADDING};
use crate::design::icons::Icon;
use crate::design::tokens::{
    APP_BAR_HEIGHT, BODY, CAPTION, ICON, MONO_INK_DROP, PAGE_PADDING, PERF_BAR_HEIGHT, PERF_BAR_TEXT, RADIUS_CONTROL, SPACE_L, SPACE_M,
    SPACE_S, SPACE_XS, STATUS_BAR_HEIGHT, SUBTITLE,
};
use crate::drawing::geometry::{Color, Point, Rect, Size};
use crate::drawing::painter::{Align, Border, Painter};
use crate::widgets::info_bar::{InfoBar, Severity};
use crate::widgets::text::KEYCAP_HEIGHT;

// The app bar: the program's icon, title and subtitle along the window's
// top, opaque, like Mica's solid stand-in, so that it covers a 3D view.
#[derive(Clone, Debug)]
pub struct AppBar {
    icon: Icon,
    title: String,
    subtitle: Option<String>,
}

impl AppBar {
    pub fn new(icon: Icon, title: &str) -> Self {
        Self {
            icon,
            title: title.to_string(),
            subtitle: None,
        }
    }

    pub fn subtitle(mut self, subtitle: &str) -> Self {
        self.subtitle = Some(subtitle.to_string());
        self
    }

    pub fn set_subtitle(&mut self, subtitle: &str) {
        self.subtitle = Some(subtitle.to_string());
    }

    fn title_right(&self, ui: &mut Ui) -> f32 {
        let mut x = SPACE_L + ICON + SPACE_L + ui.measure(&self.title, CAPTION).width;
        if let Some(subtitle) = &self.subtitle {
            x += SPACE_S + ui.measure(subtitle, CAPTION).width;
        }
        x
    }
}

impl Control for AppBar {
    fn kind(&self) -> &'static str {
        "AppBar"
    }

    fn auto_target(&self) -> AutoTarget {
        AutoTarget::Unlisted
    }

    fn draw(&mut self, ui: &mut Ui, ctx: &NodeContext) {
        let rect = ctx.rect;
        let mut painter = ui.painter();
        let theme = painter.theme();
        painter.fill(rect, theme.background);
        let icon_center = Point::new(SPACE_L + ICON / 2.0, rect.center().y);
        painter.icon(icon_center, self.icon, ICON, theme.accent);
        let x = SPACE_L + ICON + SPACE_L;
        let title = painter.text_in(Rect::new(x, rect.y, rect.width, rect.height), &self.title, CAPTION, theme.text_primary, Align::Start);
        if let Some(subtitle) = &self.subtitle {
            painter.text_fit(Rect::new(title.right() + SPACE_S, rect.y, rect.width, rect.height), subtitle, CAPTION, theme.text_secondary, Align::Start);
        }
    }
}

// The menu row's background, under the app bar.
#[derive(Clone, Copy, Debug, Default)]
pub struct MenuRow;

impl Control for MenuRow {
    fn kind(&self) -> &'static str {
        "MenuRow"
    }

    fn auto_target(&self) -> AutoTarget {
        AutoTarget::Unlisted
    }

    fn draw(&mut self, ui: &mut Ui, ctx: &NodeContext) {
        let background = ui.theme().background;
        ui.painter().fill(ctx.rect, background);
    }
}

// What a status bar shows, from its left, then from its right after a
// spacer.
#[derive(Clone, Debug)]
pub enum Status {
    // Keys, like "Ctrl+S", as keycaps, and what they do.
    Shortcut(String, String),
    Text(String),
    // A dot in a status's color, and what it says.
    Indicator(Severity, String),
    // What comes after goes along the right.
    Spacer,
}

impl Status {
    pub fn shortcut(keys: &str, label: &str) -> Self {
        Status::Shortcut(keys.to_string(), label.to_string())
    }

    pub fn text(text: &str) -> Self {
        Status::Text(text.to_string())
    }

    pub fn indicator(severity: Severity, text: &str) -> Self {
        Status::Indicator(severity, text.to_string())
    }
}

// The status bar along the window's bottom.
#[derive(Clone, Debug, Default)]
pub struct StatusBar {
    items: Vec<Status>,
}

impl StatusBar {
    pub fn new(items: Vec<Status>) -> Self {
        Self { items }
    }

    // Swaps what it shows, like the frame rate.
    pub fn set_items(&mut self, items: Vec<Status>) {
        self.items = items;
    }
}

impl Control for StatusBar {
    fn kind(&self) -> &'static str {
        "StatusBar"
    }

    fn auto_target(&self) -> AutoTarget {
        AutoTarget::Unlisted
    }

    fn draw(&mut self, ui: &mut Ui, ctx: &NodeContext) {
        let rect = ctx.rect;
        {
            let mut painter = ui.painter();
            let theme = painter.theme();
            painter.fill(rect, theme.background);
            painter.fill(rect.top_strip(1.0), theme.divider);
        }
        let items = self.items.clone();
        let spacer = items.iter().position(|item| matches!(item, Status::Spacer)).unwrap_or(items.len());
        let (left, right) = items.split_at(spacer);
        let right = right.get(1..).unwrap_or(&[]);
        // The right's items, from the right edge in, so that the left's can
        // be cut short where they'd meet them. They take at most the bar's
        // right half, and the one that would go past its middle is cut
        // short, like a long message.
        let middle = rect.center().x;
        let mut right_edge = rect.right() - SPACE_L;
        for item in right.iter().rev() {
            let width = status_width(ui, item).min(right_edge - middle);
            if width <= 0.0 {
                break;
            }
            draw_status(ui, item, Rect::new(right_edge - width, rect.y + 1.0, width, rect.height - 1.0));
            right_edge -= width + SPACE_L + SPACE_S;
        }
        let mut x = rect.x + SPACE_L;
        for item in left {
            let width = status_width(ui, item).min((right_edge - x).max(0.0));
            if width <= 0.0 {
                break;
            }
            draw_status(ui, item, Rect::new(x, rect.y + 1.0, width, rect.height - 1.0));
            x += width + SPACE_L + SPACE_S;
        }
    }
}

// The notices under the app bar: an info bar for each, closable, which
// re-lays the page out as they close.
#[derive(Default)]
pub(crate) struct Notices {
    bars: Vec<InfoBar>,
    heights: Vec<f32>,
}

fn notice_bar(notice: &Notice) -> InfoBar {
    InfoBar::new(notice.severity, &notice.title, &notice.message).closable()
}

// How tall the notices are over a page `width` wide, with the gap under
// them.
pub(crate) fn notices_height(ui: &mut Ui, width: f32) -> f32 {
    let notices = ui.notices().to_vec();
    if notices.is_empty() {
        return 0.0;
    }
    let mut height = SPACE_S;
    for notice in &notices {
        height += notice_bar(notice).height(ui, width) + SPACE_S;
    }
    height
}

impl Control for Notices {
    fn kind(&self) -> &'static str {
        "Notices"
    }

    fn auto_target(&self) -> AutoTarget {
        AutoTarget::None
    }

    fn handle(&mut self, ui: &mut Ui, ctx: &NodeContext) {
        let notices = ui.notices().to_vec();
        self.bars = notices.iter().map(notice_bar).collect();
        self.heights.clear();
        let mut area = ctx.rect;
        let mut closed = None;
        for (index, bar) in self.bars.iter_mut().enumerate() {
            let height = bar.height(ui, area.width);
            self.heights.push(height);
            let rect = area.cut_top(height, SPACE_S);
            if bar.input(ui, &format!("Notice/{index}"), rect) {
                closed = Some(index);
            }
        }
        if let Some(index) = closed {
            ui.remove_notice(index);
            ui.request_rebuild();
        }
    }

    fn draw(&mut self, ui: &mut Ui, ctx: &NodeContext) {
        let mut area = ctx.rect;
        let heights = self.heights.clone();
        for (bar, height) in self.bars.iter_mut().zip(heights) {
            let rect = area.cut_top(height, SPACE_S);
            bar.paint(ui, rect);
        }
    }
}

impl Ui {
    // The app bar node: the program's icon and title, and the rest of the
    // bar, which it returns, for the program's own tabs and commands. Only in
    // Program::build, first of the bars.
    pub fn app_bar(&mut self, bar: AppBar) -> Rect {
        let rect = Rect::new(0.0, self.content_top(), self.size().width, APP_BAR_HEIGHT);
        let left = bar.title_right(self) + SPACE_L;
        self.node("AppBar", rect, bar);
        self.page_top = rect.bottom();
        Rect::new(left, rect.y + SPACE_S, rect.right() - SPACE_L - left, rect.height - SPACE_S * 2.0)
    }

    // A row under the app bar, the window's width and as tall as WinUI's
    // MenuBar, for a menu bar and the program's commands. The page starts
    // under it. Call it after the app bar. It returns the area for them: from
    // where a menu bar's first menu has its name on the page's left edge, in
    // line with the PerfBar's text, the app bar's icon and the cards, as
    // Windows 11 lines up a navigation item's icon under the title bar's, with
    // its backplate out in the padding; to the page's right edge.
    pub fn menu_row(&mut self) -> Rect {
        let rect = Rect::new(0.0, self.page_top, self.size().width, MENU_BAR_HEIGHT);
        self.node("MenuRow", rect, MenuRow);
        self.page_top = rect.bottom();
        let left = PAGE_PADDING - MENU_BAR_ITEM_MARGIN.left - MENU_BAR_ITEM_PADDING.left;
        Rect::new(left, rect.y, rect.right() - PAGE_PADDING - left, rect.height)
    }

    // The status bar node along the bottom. Update what it shows with
    // ui.get_mut::<StatusBar>("StatusBar").set_items(...).
    pub fn status_bar(&mut self, items: Vec<Status>) {
        let size = self.size();
        let rect = Rect::new(0.0, size.height - STATUS_BAR_HEIGHT, size.width, STATUS_BAR_HEIGHT);
        self.node("StatusBar", rect, StatusBar::new(items));
        self.page_bottom = rect.y;
    }

    // The page between the app bar, or the menu row, and the status bar,
    // inside its padding, after the notices, whose node this makes. Call it
    // after the bars, and lay the cards out in what it returns.
    pub fn page(&mut self) -> Rect {
        let width = self.size().width;
        let mut page = Rect::new(PAGE_PADDING, self.page_top + PAGE_PADDING, width - PAGE_PADDING * 2.0, self.page_bottom - self.page_top - PAGE_PADDING * 2.0);
        let notices = notices_height(self, page.width);
        if notices > 0.0 {
            let strip = page.cut_top(notices - SPACE_S, SPACE_S);
            self.node("Notices", strip, Notices::default());
        }
        page
    }
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
    // Draws the PerfBar along the top of the window: the frames a second,
    // the input latency's mean from GameInput's readings to the screen,
    // and at the right, how frames reach the screen and whether they wait for
    // vertical sync, in sections that |s separate, set in Cascadia Mono,
    // black on white, in fields whose widths never change, so that every
    // label and value stays in the same columns. The main loop draws it every frame; programs don't.
    pub(crate) fn perf_bar(&mut self) {
        // Out of self while the painter borrows it, and back after: the
        // summary's words are made 10 times a second, not copied here
        // every frame.
        let Some(performance) = self.performance.take() else {
            return;
        };
        let summary = performance.summary();
        let bar = Rect::new(0.0, 0.0, self.size().width, PERF_BAR_HEIGHT);
        let divider = self.theme().divider;
        // Vertical sync from this frame on, as F10 switches it, not as of the
        // summary, as wide whether it's on or off.
        let vertical_sync = if self.environment.vertical_sync { "on " } else { "off" };
        self.cover(bar);
        {
            let mut painter = self.painter();
            // Opaque, even over a 3D view.
            painter.fill(bar, Color::hex(0xFFFFFF));
            painter.fill(Rect::new(0.0, bar.bottom() - 1.0, bar.width, 1.0), divider);
            // The text's ink centered above the divider.
            let above = bar.height - 1.0;
            let grid = PerfBarGrid {
                advance: painter.measure("0", PERF_BAR_TEXT).width,
                y: ((above - PERF_BAR_TEXT.line_height) / 2.0 - MONO_INK_DROP * PERF_BAR_TEXT.size).round(),
                color: Color::hex(0x000000),
            };
            let columns = ((bar.width - PAGE_PADDING * 2.0) / grid.advance).floor() as usize;
            // The right's runs end at the last column, the left's start at
            // the first, and why there's no input latency goes right after
            // the left's, which a space at least keeps from the right's.
            let right = runs(&summary.right).chain([(SEPARATOR, false), ("VSync = ", false), (vertical_sync, true)]);
            let right_start = columns.saturating_sub(right.clone().map(|(text, _)| text.chars().count()).sum());
            let end = right_start.saturating_sub(1);
            let left_end = grid.draw(&mut painter, 0, end, runs(&summary.left));
            grid.draw(&mut painter, left_end, end, runs(&summary.note));
            grid.draw(&mut painter, right_start, columns, right);
        }
        self.performance = Some(performance);
    }

    // What the window shows instead of the program in a setup that programs
    // don't support: what's wrong, and the setups that are.
    pub fn unsupported_setup(&mut self, problems: &[String]) {
        unsupported_setup(self, problems);
    }
}

// The PerfBar's text, as runs of labels and values.
fn runs(runs: &[Run]) -> impl Iterator<Item = (&str, bool)> + Clone {
    runs.iter().map(|run| (run.text.as_str(), run.value))
}

// The PerfBar's columns, each a character of Cascadia Mono wide, from the
// bar's padding, and its text's color.
struct PerfBarGrid {
    advance: f32,
    y: f32,
    color: Color,
}

impl PerfBarGrid {
    // Draws runs from a column, cutting short what would reach another, and
    // returns the column after them.
    fn draw<'a>(&self, painter: &mut Painter, mut column: usize, end: usize, runs: impl Iterator<Item = (&'a str, bool)>) -> usize {
        for (text, _) in runs {
            let count = text.chars().count();
            let at = Point::new(PAGE_PADDING + column as f32 * self.advance, self.y);
            if column + count <= end {
                painter.text(at, text, PERF_BAR_TEXT, self.color);
            } else if column < end {
                let fitted = painter.fit(text, PERF_BAR_TEXT, (end - column) as f32 * self.advance + 0.5);
                painter.text(at, &fitted, PERF_BAR_TEXT, self.color);
            }
            column += count;
        }
        column
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
