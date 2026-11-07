// WinUI's InfoBar: a message of one of four severities, in its background
// color, with its status icon, its title in Body Strong and its message in
// Body after it on the same line, or under it if they don't fit, and a close
// button if it's closable, which emits Closed.

use crate::core::control::{Control, NodeContext, UiEventKind};
use crate::core::ui::{Response, Sense, Ui};
use crate::design::controls::{
    INFO_BAR_CLOSE_BUTTON_MARGIN, INFO_BAR_CLOSE_BUTTON_SIZE, INFO_BAR_ICON_MARGIN, INFO_BAR_ICON_SIZE, INFO_BAR_MESSAGE_GAP,
    INFO_BAR_MIN_HEIGHT, INFO_BAR_PADDING_LEFT, INFO_BAR_PANEL_MARGIN_RIGHT, INFO_BAR_TEXT_TOP, INFO_BAR_WRAPPED_BOTTOM,
    INFO_BAR_WRAPPED_MESSAGE_GAP,
};
use crate::design::icons::Icon;
use crate::design::tokens::{BODY, BODY_STRONG, RADIUS_CONTROL};
use crate::drawing::geometry::{Point, Rect};
use crate::drawing::painter::{Align, Border};
use crate::widgets::buttons::{ButtonFace, ButtonStyle, draw_button};

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

#[derive(Clone, Debug)]
pub struct InfoBar {
    severity: Severity,
    title: String,
    message: String,
    closable: bool,
    close_response: Option<Response>,
}

impl InfoBar {
    pub fn new(severity: Severity, title: &str, message: &str) -> Self {
        Self {
            severity,
            title: title.to_string(),
            message: message.to_string(),
            closable: false,
            close_response: None,
        }
    }

    pub fn closable(mut self) -> Self {
        self.closable = true;
        self
    }

    pub fn set_message(&mut self, severity: Severity, title: &str, message: &str) {
        self.severity = severity;
        self.title = title.to_string();
        self.message = message.to_string();
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
        let title = ui.measure(&self.title, BODY_STRONG).width;
        let message = ui.measure(&self.message, BODY).width;
        if title + INFO_BAR_MESSAGE_GAP + message <= room {
            return (true, BODY.line_height);
        }
        (false, ui.fonts().measure_wrapped(&self.message, BODY, room).height)
    }

    // How tall an info bar of the width is.
    pub fn height(&self, ui: &mut Ui, width: f32) -> f32 {
        let (one_line, message) = self.layout(ui, width);
        if one_line {
            INFO_BAR_MIN_HEIGHT
        } else {
            (INFO_BAR_TEXT_TOP + BODY_STRONG.line_height + INFO_BAR_WRAPPED_MESSAGE_GAP + message + INFO_BAR_WRAPPED_BOTTOM).max(INFO_BAR_MIN_HEIGHT)
        }
    }

    fn close_rect(rect: Rect) -> Rect {
        let size = INFO_BAR_CLOSE_BUTTON_SIZE;
        Rect::new(rect.right() - INFO_BAR_CLOSE_BUTTON_MARGIN - size, rect.y + INFO_BAR_CLOSE_BUTTON_MARGIN, size, size)
    }

    // The dispatch half, for a bar drawn inside another control, like the
    // notices under the app bar: handles its close button. Returns whether
    // it was closed.
    pub(crate) fn input(&mut self, ui: &mut Ui, name: &str, rect: Rect) -> bool {
        ui.register(name, rect, Sense::NONE, "InfoBar");
        ui.set_label(name, format!("{}: {} {}", self.severity.name(), self.title, self.message).trim_end());
        self.close_response = None;
        if self.closable {
            let close = Self::close_rect(rect);
            let response = ui.interact(&format!("{name}/Close"), close, Sense::BUTTON, "Button");
            self.close_response = Some(response);
            return response.clicked;
        }
        false
    }

    // The draw half, usable from another control too.
    pub(crate) fn paint(&mut self, ui: &mut Ui, rect: Rect) {
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
            let (title, _) = painter.text_fit(line, &self.title, BODY_STRONG, theme.text_primary, Align::Start);
            let after = if self.title.is_empty() { title.x } else { title.right() + INFO_BAR_MESSAGE_GAP };
            painter.text_fit(Rect::new(after, line.y, (line.right() - after).max(0.0), line.height), &self.message, BODY, theme.text_primary, Align::Start);
        } else {
            let title = Rect::new(left, title_top, room, BODY_STRONG.line_height);
            painter.text_fit(title, &self.title, BODY_STRONG, theme.text_primary, Align::Start);
            let message_top = title.bottom() + INFO_BAR_WRAPPED_MESSAGE_GAP;
            painter.text_wrapped(Rect::new(left, message_top, room, rect.bottom() - message_top), &self.message, BODY, theme.text_primary);
        }
        // The icon's top is 16 from the bar's, as is a one-line bar's middle.
        let icon_center = Point::new(rect.x + INFO_BAR_PADDING_LEFT + INFO_BAR_ICON_SIZE / 2.0, rect.y + INFO_BAR_ICON_MARGIN.top + INFO_BAR_ICON_SIZE / 2.0);
        ui.status_icon(icon_center, self.severity, INFO_BAR_ICON_SIZE);
        if let Some(response) = self.close_response {
            let face = ButtonFace {
                text: "",
                icon: Some(Icon::Dismiss),
                style: ButtonStyle::Subtle,
                checked: false,
                chevron: false,
            };
            draw_button(ui, &face, &response, Self::close_rect(rect));
        }
    }
}

impl Control for InfoBar {
    fn kind(&self) -> &'static str {
        "InfoBar"
    }

    fn auto_target(&self) -> crate::core::control::AutoTarget {
        crate::core::control::AutoTarget::None
    }

    fn handle(&mut self, ui: &mut Ui, ctx: &NodeContext) {
        if self.input(ui, &ctx.name, ctx.rect) {
            ctx.emit(ui, UiEventKind::Closed);
        }
    }

    fn draw(&mut self, ui: &mut Ui, ctx: &NodeContext) {
        self.paint(ui, ctx.rect);
    }
}
