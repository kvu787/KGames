// A program's frame: an app bar along the top, notices under it, a status
// bar along the bottom, and the page between them, where the cards go. And
// what the window shows instead, in a setup that programs don't support.

use crate::geometry::{Point, Rect};
use crate::icons::Icon;
use crate::painter::{Align, Corners, Stroke};
use crate::tokens::{
    APP_BAR_HEIGHT, BODY, BODY_STRONG, CAPTION, ICON, ICON_LARGE, PAGE_PADDING, RADIUS_OVERLAY, SPACE_L, SPACE_M, SPACE_S,
    SPACE_XS, SPACE_XXL, STATUS_BAR_HEIGHT, SUBTITLE,
};
use crate::ui::Ui;
use crate::widgets::surfaces::{InfoBar, Severity};

// Something the program tells of under its app bar, like a setting that's
// wrong, until it's closed.
#[derive(Clone, Debug, PartialEq)]
pub struct Notice {
    pub severity: Severity,
    pub title: String,
    pub message: String,
}

// An item of the status bar.
#[derive(Clone, Copy, Debug)]
pub enum Status<'a> {
    Text(&'a str),
    // A key and what it does, like F1 and "Rumble test".
    Shortcut(&'a str, &'a str),
    // Puts the items after it at the bar's right end.
    Spacer,
}

impl Ui {
    // Draws the app bar, with the program's icon and title at its left, and
    // returns the rest of the bar, for the program's tabs and commands.
    pub fn app_bar(&mut self, icon: Icon, title: &str) -> Rect {
        let bar = Rect::new(0.0, 0.0, self.size().width, APP_BAR_HEIGHT);
        let theme = self.theme();
        let (background, fill, divider, accent, text) = (theme.background, theme.layer, theme.divider, theme.accent, theme.text_primary);
        self.cover(bar);
        let mut painter = self.painter();
        // Opaque, even over a 3D view.
        painter.fill(bar, background);
        painter.fill(bar, fill);
        painter.fill(Rect::new(0.0, bar.bottom() - 1.0, bar.width, 1.0), divider);
        let icon_center = Point::new(PAGE_PADDING + ICON / 2.0, bar.center().y);
        painter.icon(icon_center, icon, ICON, accent);
        let title_rect = painter.text_in(
            Rect::new(PAGE_PADDING + ICON + SPACE_M, bar.y, bar.width, bar.height),
            title,
            BODY_STRONG,
            text,
            Align::Start,
        );
        self.page_top = bar.bottom();
        let left = title_rect.right() + SPACE_XXL;
        Rect::new(left, bar.y + SPACE_S, bar.width - left - PAGE_PADDING, bar.height - SPACE_S * 2.0)
    }

    // Shows a notice under the app bar, unless it already does.
    pub fn notify(&mut self, severity: Severity, title: &str, message: &str) {
        let notice = Notice {
            severity,
            title: title.to_string(),
            message: message.to_string(),
        };
        if !self.notices.contains(&notice) {
            self.notices.push(notice);
        }
    }

    pub fn notices(&self) -> &[Notice] {
        &self.notices
    }

    // The page: the area under the app bar and the notices, and above the
    // status bar, inside PAGE_PADDING. Draws the notices, which close.
    pub fn page(&mut self) -> Rect {
        let width = self.size().width;
        let mut top = self.page_top + PAGE_PADDING;
        let mut closed = None;
        for index in 0..self.notices.len() {
            let notice = self.notices[index].clone();
            let info = InfoBar::new(notice.severity, &notice.title, &notice.message).closable();
            let height = self.info_bar_height(width - PAGE_PADDING * 2.0, &info);
            let rect = Rect::new(PAGE_PADDING, top, width - PAGE_PADDING * 2.0, height);
            if self.info_bar(&format!("Notice/{index}"), rect, info).clicked {
                closed = Some(index);
            }
            top += height + SPACE_S;
        }
        if let Some(index) = closed {
            self.notices.remove(index);
        }
        let bottom = self.size().height - if self.status_bar_shown { STATUS_BAR_HEIGHT } else { 0.0 } - PAGE_PADDING;
        Rect::new(PAGE_PADDING, top, width - PAGE_PADDING * 2.0, (bottom - top).max(0.0))
    }

    // Says that the page has no status bar under it this frame.
    pub fn without_status_bar(&mut self) {
        self.status_bar_shown = false;
    }

    // Draws the status bar along the bottom of the window.
    pub fn status_bar(&mut self, items: &[Status]) {
        let size = self.size();
        let bar = Rect::new(0.0, size.height - STATUS_BAR_HEIGHT, size.width, STATUS_BAR_HEIGHT);
        let theme = self.theme();
        let (background, fill, divider, text) = (theme.background, theme.layer, theme.divider, theme.text_secondary);
        self.cover(bar);
        let mut painter = self.painter();
        painter.fill(bar, background);
        painter.fill(bar, fill);
        painter.fill(Rect::new(0.0, bar.y, bar.width, 1.0), divider);
        // Items after a spacer go at the right end, measured first.
        let split = items.iter().position(|item| matches!(item, Status::Spacer)).unwrap_or(items.len());
        let (left, right) = items.split_at(split);
        let right = right.get(1..).unwrap_or(&[]);
        let mut x = PAGE_PADDING;
        for item in left {
            x = self.status_item(item, x, bar, text) + SPACE_XXL;
        }
        let width: f32 = right.iter().map(|item| self.status_width(item) + SPACE_XXL).sum::<f32>() - SPACE_XXL;
        let mut x = bar.right() - PAGE_PADDING - width.max(0.0);
        for item in right {
            x = self.status_item(item, x, bar, text) + SPACE_XXL;
        }
    }

    fn status_width(&mut self, item: &Status) -> f32 {
        match item {
            Status::Text(text) => self.measure(text, CAPTION).width,
            Status::Shortcut(key, action) => {
                let key_width = (self.measure(key, CAPTION).width + SPACE_S * 2.0).max(22.0).ceil();
                key_width + SPACE_S + self.measure(action, CAPTION).width
            }
            Status::Spacer => 0.0,
        }
    }

    // Draws an item from x, and returns where it ends.
    fn status_item(&mut self, item: &Status, x: f32, bar: Rect, color: crate::geometry::Color) -> f32 {
        match item {
            Status::Text(text) => {
                let rect = self.painter().text_in(Rect::new(x, bar.y, bar.right() - x, bar.height), text, CAPTION, color, Align::Start);
                rect.right()
            }
            Status::Shortcut(key, action) => {
                let width = self.keycap(Point::new(x, (bar.center().y - 11.0).round()), key);
                let rect = self.painter().text_in(
                    Rect::new(x + width + SPACE_S, bar.y, bar.right() - x, bar.height),
                    action,
                    CAPTION,
                    color,
                    Align::Start,
                );
                rect.right()
            }
            Status::Spacer => x,
        }
    }

    // What the window shows in a setup that programs don't support: what's
    // wrong, and what the supported setups are.
    pub(crate) fn unsupported_setup(&mut self, problems: &[String]) {
        let size = self.size();
        let theme = self.theme().clone();
        let width = 640.0f32.min(size.width - SPACE_L * 2.0).max(1.0);
        let text_width = width - SPACE_XXL * 2.0;
        let explanation = "KGames' programs support two window setups, both on a 2560 × 1440 display at 100% scale, \
                           with a 48-pixel taskbar at the bottom: maximized, and borderless fullscreen, which F11 \
                           switches to and from.";
        let mut height = SPACE_XXL * 2.0 + SUBTITLE.line_height + SPACE_M;
        for problem in problems {
            height += self.fonts.layout(problem, BODY, Some(text_width - ICON - SPACE_S)).height + SPACE_XS;
        }
        height += SPACE_M + self.fonts.layout(explanation, CAPTION, Some(text_width)).height;
        let card = Rect::from_center(size_center(size), crate::geometry::Size::new(width, height)).round();
        let mut painter = self.painter();
        painter.fill(Rect::new(0.0, 0.0, size.width, size.height), theme.background);
        painter.shadow(card, RADIUS_OVERLAY, crate::tokens::ELEVATION_16);
        painter.shape(card, Corners::all(RADIUS_OVERLAY), theme.flyout, Some(Stroke::new(theme.surface_stroke_flyout, 1.0)));
        let mut y = card.y + SPACE_XXL;
        let x = card.x + SPACE_XXL;
        painter.icon(Point::new(x + ICON_LARGE / 2.0, y + SUBTITLE.line_height / 2.0), Icon::Warning, ICON_LARGE, theme.caution);
        painter.text(Point::new(x + ICON_LARGE + SPACE_S, y), "This window setup isn't supported", SUBTITLE, theme.text_primary);
        y += SUBTITLE.line_height + SPACE_M;
        for problem in problems {
            painter.icon(Point::new(x + ICON / 2.0, y + BODY.line_height / 2.0), Icon::Dismiss, crate::tokens::ICON_SMALL, theme.critical);
            y += painter
                .text_wrapped(Rect::new(x + ICON + SPACE_S, y, text_width - ICON - SPACE_S, 0.0), problem, BODY, theme.text_primary)
                .height
                + SPACE_XS;
        }
        y += SPACE_M;
        painter.text_wrapped(Rect::new(x, y, text_width, 0.0), explanation, CAPTION, theme.text_secondary);
    }
}

fn size_center(size: crate::geometry::Size) -> Point {
    Point::new(size.width / 2.0, size.height / 2.0)
}
