// WinUI's InfoBar: a message of one of four severities, in its background
// color, with its status icon, its title in Body Strong and its message in
// Body after it on the same line, or under it if they don't fit, and a close
// button if it's closable.

use crate::core::interaction::Sense;
use crate::core::ui::Ui;
use crate::design::controls::{
    INFO_BAR_CLOSE_BUTTON_MARGIN, INFO_BAR_CLOSE_BUTTON_SIZE, INFO_BAR_ICON_MARGIN, INFO_BAR_ICON_SIZE, INFO_BAR_MESSAGE_GAP,
    INFO_BAR_MIN_HEIGHT, INFO_BAR_PADDING_LEFT, INFO_BAR_PANEL_MARGIN_RIGHT, INFO_BAR_TEXT_TOP, INFO_BAR_WRAPPED_BOTTOM,
    INFO_BAR_WRAPPED_MESSAGE_GAP,
};
use crate::design::icons::Icon;
use crate::design::tokens::{BODY, BODY_STRONG, RADIUS_CONTROL};
use crate::drawing::geometry::{Point, Rect};
use crate::drawing::painter::{Align, Border};
use crate::widgets::buttons::Button;
use crate::widgets::widget::Widget;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Severity {
    #[default]
    Informational,
    Success,
    Warning,
    Error,
}

impl Severity {
    pub fn name(self) -> &'static str {
        match self {
            Severity::Informational => "Informational",
            Severity::Success => "Success",
            Severity::Warning => "Warning",
            Severity::Error => "Error",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct InfoBar<'a> {
    severity: Severity,
    title: &'a str,
    message: &'a str,
    closable: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct InfoBarResponse {
    // Its close button was clicked.
    pub closed: bool,
}

impl<'a> InfoBar<'a> {
    pub fn new(severity: Severity, title: &'a str, message: &'a str) -> Self {
        Self {
            severity,
            title,
            message,
            closable: false,
        }
    }

    pub fn closable(mut self) -> Self {
        self.closable = true;
        self
    }

    fn text_left(&self) -> f32 {
        INFO_BAR_PADDING_LEFT + INFO_BAR_ICON_SIZE + INFO_BAR_ICON_MARGIN.right
    }

    fn text_right(&self) -> f32 {
        INFO_BAR_PANEL_MARGIN_RIGHT + if self.closable { INFO_BAR_CLOSE_BUTTON_SIZE + INFO_BAR_CLOSE_BUTTON_MARGIN * 2.0 - INFO_BAR_PANEL_MARGIN_RIGHT } else { 0.0 }
    }

    // Whether the title and message fit on one line, and if not, the
    // message's height, wrapped.
    fn layout(&self, ui: &mut Ui, width: f32) -> (bool, f32) {
        let room = (width - self.text_left() - self.text_right()).max(1.0);
        let title = ui.measure(self.title, BODY_STRONG).width;
        let message = ui.measure(self.message, BODY).width;
        if title + INFO_BAR_MESSAGE_GAP + message <= room {
            return (true, BODY.line_height);
        }
        (false, ui.fonts().measure_wrapped(self.message, BODY, room).height)
    }
}

impl Ui {
    // How tall an info bar of the width is.
    pub fn info_bar_height(&mut self, width: f32, bar: &InfoBar) -> f32 {
        let (one_line, message) = bar.layout(self, width);
        if one_line {
            INFO_BAR_MIN_HEIGHT
        } else {
            (INFO_BAR_TEXT_TOP + BODY_STRONG.line_height + INFO_BAR_WRAPPED_MESSAGE_GAP + message + INFO_BAR_WRAPPED_BOTTOM).max(INFO_BAR_MIN_HEIGHT)
        }
    }
}

impl Widget for InfoBar<'_> {
    type Response = InfoBarResponse;

    fn show(self, ui: &mut Ui, name: &str, rect: Rect) -> InfoBarResponse {
        ui.register(name, rect, Sense::NONE, "InfoBar");
        ui.set_label(name, format!("{}: {} {}", self.severity.name(), self.title, self.message).trim_end());
        let (one_line, _) = self.layout(ui, rect.width);
        let mut painter = ui.painter();
        let theme = painter.theme();
        let background = match self.severity {
            Severity::Informational => theme.attention_background,
            Severity::Success => theme.success_background,
            Severity::Warning => theme.caution_background,
            Severity::Error => theme.critical_background,
        };
        painter.shape(rect, RADIUS_CONTROL, background, Some(Border::new(theme.card_stroke, 1.0)));
        let left = rect.x + self.text_left();
        let room = (rect.right() - self.text_right() - left).max(0.0);
        let title_top = rect.y + INFO_BAR_TEXT_TOP;
        if one_line {
            let line = Rect::new(left, title_top, room, BODY.line_height);
            let (title, _) = painter.text_fit(line, self.title, BODY_STRONG, theme.text_primary, Align::Start);
            let after = if self.title.is_empty() { title.x } else { title.right() + INFO_BAR_MESSAGE_GAP };
            painter.text_fit(Rect::new(after, line.y, (line.right() - after).max(0.0), line.height), self.message, BODY, theme.text_primary, Align::Start);
        } else {
            let title = Rect::new(left, title_top, room, BODY_STRONG.line_height);
            painter.text_fit(title, self.title, BODY_STRONG, theme.text_primary, Align::Start);
            let message_top = title.bottom() + INFO_BAR_WRAPPED_MESSAGE_GAP;
            painter.text_wrapped(Rect::new(left, message_top, room, rect.bottom() - message_top), self.message, BODY, theme.text_primary);
        }
        // The icon's top is 16 from the bar's, as is a one-line bar's middle.
        let icon_center = Point::new(rect.x + INFO_BAR_PADDING_LEFT + INFO_BAR_ICON_SIZE / 2.0, rect.y + INFO_BAR_ICON_MARGIN.top + INFO_BAR_ICON_SIZE / 2.0);
        ui.status_icon(icon_center, self.severity, INFO_BAR_ICON_SIZE);
        let mut closed = false;
        if self.closable {
            let size = INFO_BAR_CLOSE_BUTTON_SIZE;
            let close = Rect::new(rect.right() - INFO_BAR_CLOSE_BUTTON_MARGIN - size, rect.y + INFO_BAR_CLOSE_BUTTON_MARGIN, size, size);
            closed = ui.add(&format!("{name}/Close"), close, Button::icon_only(Icon::Dismiss).subtle()).clicked;
        }
        InfoBarResponse { closed }
    }
}
