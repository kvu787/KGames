// The page that every program's window shows: the PerfBar along the very
// top, which the library draws; an app bar under it, as tall as WinUI's
// TitleBar control, with the program's icon, title and its own commands; a
// menu row under that, if the program wants one, for a menu bar whose menus'
// names line up with the page's left edge; notices under them; a status bar
// along the bottom with shortcuts and status; and the page between them,
// where the cards go. Tree::shell adds the bars and the notices, and says
// where the page is:
//
//     let shell = tree.shell(ui, AppBar::new(Icon::Settings, "Settings"), false, Some(vec![Status::text("Ready")]));
//     // Cards in shell.page.

use crate::application::performance::{Run, SEPARATOR};
use crate::application::program::Notice;
use crate::core::interaction::{Interaction, Part, Sense};
use crate::core::interface::{Interface, Ui};
use crate::core::tree::{Dispatch, Draw, Parts, Tree, Widget};
use crate::design::controls::{MENU_BAR_HEIGHT, MENU_BAR_ITEM_MARGIN, MENU_BAR_ITEM_PADDING};
use crate::design::icons::Icon;
use crate::design::theme::ThemeKind;
use crate::design::tokens::{
    APP_BAR_HEIGHT, BODY, CAPTION, ICON, MONO_INK_DROP, PAGE_PADDING, PERF_BAR_HEIGHT, PERF_BAR_TEXT, RADIUS_CONTROL, SPACE_L, SPACE_M, SPACE_S,
    SPACE_XS, STATUS_BAR_HEIGHT, SUBTITLE,
};
use crate::drawing::geometry::{Color, Point, Rect, Size};
use crate::drawing::painter::{Align, Border, Painter};
use crate::drawing::text::Fonts;
use crate::widgets::info_bar::{InfoBar, Severity};
use crate::widgets::text::{KEYCAP_HEIGHT, draw_keys, keys_width};

// The app bar: the program's icon and title. The rest of the bar, which
// `commands` gives, is for the program's own tabs and commands.
#[derive(Clone, Debug)]
pub struct AppBar {
    icon: Icon,
    title: String,
    subtitle: Option<String>,
}

impl AppBar {
    pub fn new(icon: Icon, title: impl Into<String>) -> Self {
        Self {
            icon,
            title: title.into(),
            subtitle: None,
        }
    }

    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    // Where the app bar goes: under the PerfBar, the window's width.
    pub fn rect(ui: &Ui) -> Rect {
        Rect::new(0.0, ui.content_top(), ui.size().width, APP_BAR_HEIGHT)
    }

    // The rest of the bar, right of the title, for the program's own tabs
    // and commands.
    pub fn commands(&self, ui: &mut Ui) -> Rect {
        let rect = Self::rect(ui);
        let right = self.title_right(ui.fonts());
        let left = right + SPACE_L;
        Rect::new(left, rect.y + SPACE_S, rect.right() - SPACE_L - left, rect.height - SPACE_S * 2.0)
    }

    fn title_right(&self, fonts: &mut Fonts) -> f32 {
        let mut x = SPACE_L + ICON + SPACE_L + fonts.measure(&self.title, CAPTION).width;
        if let Some(subtitle) = &self.subtitle {
            x += SPACE_S + fonts.measure(subtitle, CAPTION).width;
        }
        x
    }
}

impl<M: 'static> Widget<M> for AppBar {
    fn kind(&self) -> &'static str {
        "AppBar"
    }

    fn covers(&self) -> bool {
        true
    }

    fn draw(&self, ctx: &mut Draw) {
        let rect = ctx.rect();
        let mut painter = ctx.painter();
        let theme = painter.theme();
        // Opaque, like Mica's solid stand-in, so that it covers a 3D view.
        painter.fill(rect, theme.background);
        let icon_center = Point::new(SPACE_L + ICON / 2.0, rect.center().y);
        painter.icon(icon_center, self.icon, ICON, theme.accent);
        let x = SPACE_L + ICON + SPACE_L;
        let title = painter.text_in(Rect::new(x, rect.y, rect.width, rect.height), &self.title, CAPTION, theme.text_primary, Align::Start);
        if let Some(subtitle) = &self.subtitle {
            painter.text_in(Rect::new(title.right() + SPACE_S, rect.y, rect.width, rect.height), subtitle, CAPTION, theme.text_secondary, Align::Start);
        }
    }
}

// A row under the app bar, the window's width and as tall as WinUI's
// MenuBar, for a menu bar and the program's commands. Its `area` is for
// them: from where a menu bar's first menu has its name on the page's left
// edge, in line with the PerfBar's text, the app bar's icon and the cards,
// as Windows 11 lines up a navigation item's icon under the title bar's,
// with its backplate out in the padding; to the page's right edge.
pub struct MenuRow;

impl MenuRow {
    pub fn rect(ui: &Ui, top: f32) -> Rect {
        Rect::new(0.0, top, ui.size().width, MENU_BAR_HEIGHT)
    }

    pub fn area(rect: Rect) -> Rect {
        let left = PAGE_PADDING - MENU_BAR_ITEM_MARGIN.left - MENU_BAR_ITEM_PADDING.left;
        Rect::new(left, rect.y, rect.right() - PAGE_PADDING - left, rect.height)
    }
}

impl<M: 'static> Widget<M> for MenuRow {
    fn kind(&self) -> &'static str {
        "MenuRow"
    }

    fn covers(&self) -> bool {
        true
    }

    fn draw(&self, ctx: &mut Draw) {
        let rect = ctx.rect();
        let mut painter = ctx.painter();
        let theme = painter.theme();
        // Opaque, like the app bar.
        painter.fill(rect, theme.background);
    }
}

// What a status bar shows, from its left, then from its right after a
// spacer.
#[derive(Clone, Debug, PartialEq)]
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
    pub fn shortcut(keys: impl Into<String>, label: impl Into<String>) -> Self {
        Status::Shortcut(keys.into(), label.into())
    }

    pub fn text(text: impl Into<String>) -> Self {
        Status::Text(text.into())
    }

    pub fn indicator(severity: Severity, text: impl Into<String>) -> Self {
        Status::Indicator(severity, text.into())
    }

    fn width(&self, fonts: &mut Fonts) -> f32 {
        match self {
            Status::Shortcut(keys, label) => keys_width(fonts, keys) + SPACE_S + fonts.measure(label, CAPTION).width,
            Status::Text(text) => fonts.measure(text, CAPTION).width,
            Status::Indicator(_, text) => 8.0 + SPACE_S + fonts.measure(text, CAPTION).width,
            Status::Spacer => 0.0,
        }
    }

    fn draw(&self, painter: &mut Painter, rect: Rect) {
        let theme = painter.theme();
        match self {
            Status::Shortcut(keys, label) => {
                let width = draw_keys(painter, Point::new(rect.x, rect.center().y - KEYCAP_HEIGHT / 2.0), keys);
                let label_rect = Rect::new(rect.x + width + SPACE_S, rect.y, (rect.width - width - SPACE_S).max(0.0), rect.height);
                painter.text_fit(label_rect, label, CAPTION, theme.text_secondary, Align::Start);
            }
            Status::Text(text) => {
                painter.text_fit(rect, text, CAPTION, theme.text_secondary, Align::Start);
            }
            Status::Indicator(severity, text) => {
                painter.circle(Point::new(rect.x + 4.0, rect.center().y), 4.0, theme.status(*severity));
                let label_rect = Rect::new(rect.x + 8.0 + SPACE_S, rect.y, (rect.width - 8.0 - SPACE_S).max(0.0), rect.height);
                painter.text_fit(label_rect, text, CAPTION, theme.text_secondary, Align::Start);
            }
            Status::Spacer => {}
        }
    }
}

// The status bar along the bottom.
pub struct StatusBar {
    items: Vec<Status>,
}

impl StatusBar {
    pub fn new(items: Vec<Status>) -> Self {
        Self { items }
    }

    pub fn rect(ui: &Ui) -> Rect {
        let size = ui.size();
        Rect::new(0.0, size.height - STATUS_BAR_HEIGHT, size.width, STATUS_BAR_HEIGHT)
    }
}

impl<M: 'static> Widget<M> for StatusBar {
    fn kind(&self) -> &'static str {
        "StatusBar"
    }

    fn covers(&self) -> bool {
        true
    }

    fn draw(&self, ctx: &mut Draw) {
        let rect = ctx.rect();
        let mut painter = ctx.painter();
        let theme = painter.theme();
        painter.fill(rect, theme.background);
        painter.fill(rect.top_strip(1.0), theme.divider);
        let spacer = self.items.iter().position(|item| matches!(item, Status::Spacer)).unwrap_or(self.items.len());
        let (left, right) = self.items.split_at(spacer);
        let right = right.get(1..).unwrap_or(&[]);
        // The right's items, from the right edge in, so that the left's can
        // be cut short where they'd meet them. They take at most the bar's
        // right half, and the one that would go past its middle is cut
        // short, like a long message.
        let middle = rect.center().x;
        let mut right_edge = rect.right() - SPACE_L;
        for item in right.iter().rev() {
            let width = item.width(painter.fonts()).min(right_edge - middle);
            if width <= 0.0 {
                break;
            }
            item.draw(&mut painter, Rect::new(right_edge - width, rect.y + 1.0, width, rect.height - 1.0));
            right_edge -= width + SPACE_L + SPACE_S;
        }
        let mut x = rect.x + SPACE_L;
        for item in left {
            let width = item.width(painter.fonts()).min((right_edge - x).max(0.0));
            if width <= 0.0 {
                break;
            }
            item.draw(&mut painter, Rect::new(x, rect.y + 1.0, width, rect.height - 1.0));
            x += width + SPACE_L + SPACE_S;
        }
    }
}

// A notice under the app bar: an info bar whose close button removes it
// from the interface's notices.
struct NoticeBar<M> {
    index: usize,
    bar: InfoBar<M>,
}

impl<M: Clone + 'static> Widget<M> for NoticeBar<M> {
    fn kind(&self) -> &'static str {
        "InfoBar"
    }

    fn label(&self) -> Option<String> {
        Widget::<M>::label(&self.bar)
    }

    fn sense(&self) -> Option<Sense> {
        Some(Sense::NONE)
    }

    fn parts(&self, ctx: &mut Parts) {
        Widget::<M>::parts(&self.bar, ctx);
    }

    fn interact(&self, interaction: &Interaction, ctx: &mut Dispatch<M>) {
        if interaction.activated(&Part::Named("Close".to_string())) {
            let notices = ctx.notices_mut();
            if self.index < notices.len() {
                notices.remove(self.index);
            }
        }
    }

    fn draw(&self, ctx: &mut Draw) {
        Widget::<M>::draw(&self.bar, ctx);
    }
}

// Where the page's bars are, and the page between them.
#[derive(Clone, Copy, Debug, Default)]
pub struct Shell {
    pub app_bar: Rect,
    // The app bar right of the title, for the program's own commands.
    pub commands: Rect,
    // The menu row's area for a menu bar and commands, if there's a row.
    pub menu_row: Option<Rect>,
    pub status_bar: Option<Rect>,
    // Between the bars, inside the page's padding, under the notices.
    pub page: Rect,
}

impl<M: Clone + 'static> Tree<M> {
    // Adds the page's bars and the notices: the app bar, a menu row under
    // it if `menu_row`, a status bar with the items if there are any, and
    // each notice as "Notice/<index>". Returns where they are, and where the
    // page is.
    pub fn shell(&mut self, ui: &mut Ui, app_bar: AppBar, menu_row: bool, status: Option<Vec<Status>>) -> Shell {
        let bar = AppBar::rect(ui);
        let commands = app_bar.commands(ui);
        self.draw(bar, app_bar);
        let mut top = bar.bottom();
        let mut menu_area = None;
        if menu_row {
            let rect = MenuRow::rect(ui, top);
            self.draw(rect, MenuRow);
            top = rect.bottom();
            menu_area = Some(MenuRow::area(rect));
        }
        let mut bottom = ui.size().height;
        let mut status_rect = None;
        if let Some(items) = status {
            let rect = StatusBar::rect(ui);
            self.draw(rect, StatusBar::new(items));
            bottom = rect.y;
            status_rect = Some(rect);
        }
        let width = ui.size().width;
        let mut page = Rect::new(PAGE_PADDING, top + PAGE_PADDING, width - PAGE_PADDING * 2.0, bottom - top - PAGE_PADDING * 2.0);
        let notices = ui.notices().to_vec();
        for (index, notice) in notices.iter().enumerate() {
            let bar = notice_bar(notice);
            let height = bar.height(ui, page.width);
            let rect = page.cut_top(height, SPACE_S);
            self.add(&format!("Notice/{index}"), rect, NoticeBar { index, bar });
        }
        if !notices.is_empty() {
            page.cut_top(SPACE_S, 0.0);
        }
        Shell {
            app_bar: bar,
            commands,
            menu_row: menu_area,
            status_bar: status_rect,
            page,
        }
    }
}

fn notice_bar<M: Clone + 'static>(notice: &Notice) -> InfoBar<M> {
    InfoBar::new(notice.severity, &notice.title, &notice.message).closable()
}

impl Interface {
    // Draws the PerfBar along the top of the window: the frames a second,
    // the input latency's mean from GameInput's readings to the screen, how
    // frames reach the screen and whether they wait for vertical sync, in
    // sections that |s separate, one after another from the left, set in
    // Cascadia Mono, in fields whose widths never change, so that every
    // label and value stays in the same columns. It's pure black and white:
    // black on white in the light theme, and white on black in the dark one.
    // The frame draws it every frame; programs don't.
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
        let (background, text) = match self.theme().kind {
            ThemeKind::Light => (Color::hex(0xFFFFFF), Color::hex(0x000000)),
            ThemeKind::Dark => (Color::hex(0x000000), Color::hex(0xFFFFFF)),
        };
        // Vertical sync from this frame on, as F10 switches it, not as of the
        // summary, as wide whether it's on or off.
        let vertical_sync = if self.environment.vertical_sync { "on " } else { "off" };
        {
            let mut painter = self.painter();
            // Opaque, even over a 3D view.
            painter.fill(bar, background);
            painter.fill(Rect::new(0.0, bar.bottom() - 1.0, bar.width, 1.0), divider);
            // The text's ink centered above the divider.
            let above = bar.height - 1.0;
            let grid = PerfBarGrid {
                advance: painter.measure("0", PERF_BAR_TEXT).width,
                y: ((above - PERF_BAR_TEXT.line_height) / 2.0 - MONO_INK_DROP * PERF_BAR_TEXT.size).round(),
                color: text,
            };
            let columns = ((bar.width - PAGE_PADDING * 2.0) / grid.advance).floor() as usize;
            // The stats from the first column, then vertical sync, then why
            // there's no input latency, if there isn't.
            let vertical_sync = [(SEPARATOR, false), ("VSync = ", false), (vertical_sync, true)];
            grid.draw(&mut painter, columns, runs(&summary.stats).chain(vertical_sync).chain(runs(&summary.note)));
        }
        self.performance = Some(performance);
    }
}

// The PerfBar's text, as runs of labels and values.
fn runs(runs: &[Run]) -> impl Iterator<Item = (&str, bool)> {
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
    // Draws runs from the first column, cutting short what would go past the
    // end column.
    fn draw<'a>(&self, painter: &mut Painter, end: usize, runs: impl Iterator<Item = (&'a str, bool)>) {
        let mut column = 0;
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
    }
}

// What the window shows instead of the program in a setup that programs
// don't support: what's wrong, and the setups that are.
pub fn unsupported_setup<M: 'static>(problems: &[String]) -> Tree<M> {
    let mut tree = Tree::new();
    tree.draw(Rect::new(0.0, 0.0, 0.0, 0.0), UnsupportedSetup { problems: problems.to_vec() });
    tree
}

struct UnsupportedSetup {
    problems: Vec<String>,
}

impl<M: 'static> Widget<M> for UnsupportedSetup {
    fn kind(&self) -> &'static str {
        "UnsupportedSetup"
    }

    fn draw(&self, ctx: &mut Draw) {
        let bounds = ctx.bounds();
        let width = 720.0f32.min(bounds.width - SPACE_L * 2.0);
        let mut height = SPACE_L * 2.0 + 28.0 + SPACE_M + SPACE_S;
        let mut lines = Vec::new();
        for problem in &self.problems {
            let line = ctx.fonts().measure_wrapped(problem, BODY, width - SPACE_L * 2.0 - SPACE_M).height;
            lines.push(line);
            height += line + SPACE_XS;
        }
        let advice = "Programs run on a 2560 × 1440 display at 100% scale, with a 48-pixel taskbar at its bottom, maximized or in borderless fullscreen with F11.";
        let advice_height = ctx.fonts().measure_wrapped(advice, BODY, width - SPACE_L * 2.0).height;
        height += SPACE_S + advice_height;
        let rect = bounds.centered(Size::new(width, height)).round();
        let mut painter = ctx.painter();
        let theme = painter.theme();
        painter.fill(bounds, theme.background);
        painter.shape(rect, RADIUS_CONTROL * 2.0, theme.critical_background, Some(Border::new(theme.card_stroke, 1.0)));
        let mut area = rect.inset(SPACE_L);
        let title = area.cut_top(28.0, SPACE_M);
        painter.text_in(title, "This window setup isn't supported", SUBTITLE, theme.text_primary, Align::Start);
        for (problem, line) in self.problems.iter().zip(lines) {
            let row = area.cut_top(line, SPACE_XS);
            painter.circle(Point::new(row.x + 3.0, row.y + 10.0), 2.5, theme.critical);
            painter.text_wrapped(Rect::new(row.x + SPACE_M, row.y, row.width - SPACE_M, row.height), problem, BODY, theme.text_primary);
        }
        area.cut_top(SPACE_S, 0.0);
        painter.text_wrapped(area, advice, BODY, theme.text_secondary);
    }
}
