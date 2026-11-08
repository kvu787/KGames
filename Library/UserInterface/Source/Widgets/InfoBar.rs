// WinUI's InfoBar: a message of one of four severities, in its background
// color, with its status icon, its title in Body Strong and its message in
// Body after it on the same line, or under it if they don't fit, and a close
// button if it's closable, named "<name>/Close".

use crate::core::interaction::{Interaction, Part, Sense};
use crate::core::interface::Ui;
use crate::core::tree::{Dispatch, Draw, Parts, Widget};
use crate::design::controls::{
    INFO_BAR_CLOSE_BUTTON_MARGIN, INFO_BAR_CLOSE_BUTTON_SIZE, INFO_BAR_ICON_MARGIN, INFO_BAR_ICON_SIZE, INFO_BAR_MESSAGE_GAP, INFO_BAR_MIN_HEIGHT,
    INFO_BAR_PADDING_LEFT, INFO_BAR_PANEL_MARGIN_RIGHT, INFO_BAR_TEXT_TOP, INFO_BAR_WRAPPED_BOTTOM, INFO_BAR_WRAPPED_MESSAGE_GAP,
};
use crate::design::icons::Icon;
use crate::design::tokens::{BODY, BODY_STRONG, RADIUS_CONTROL};
use crate::drawing::geometry::{Point, Rect};
use crate::drawing::painter::{Align, Border};
use crate::drawing::text::Fonts;
use crate::widgets::buttons::{ButtonStyle, draw_button};
use crate::widgets::text::status_icon;

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

pub struct InfoBar<M> {
    severity: Severity,
    title: String,
    message: String,
    closable: bool,
    on_close: Option<M>,
}

const CLOSE: &str = "Close";

impl<M: Clone + 'static> InfoBar<M> {
    pub fn new(severity: Severity, title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity,
            title: title.into(),
            message: message.into(),
            closable: false,
            on_close: None,
        }
    }

    pub fn closable(mut self) -> Self {
        self.closable = true;
        self
    }

    // What its close button sends, which it has then.
    pub fn on_close(mut self, message: M) -> Self {
        self.closable = true;
        self.on_close = Some(message);
        self
    }

    // How tall the bar is in the width.
    pub fn height(&self, ui: &mut Ui, width: f32) -> f32 {
        self.height_with(ui.fonts(), width)
    }

    fn height_with(&self, fonts: &mut Fonts, width: f32) -> f32 {
        let (one_line, message) = self.layout(fonts, width);
        if one_line {
            INFO_BAR_MIN_HEIGHT
        } else {
            (INFO_BAR_TEXT_TOP + BODY_STRONG.line_height + INFO_BAR_WRAPPED_MESSAGE_GAP + message + INFO_BAR_WRAPPED_BOTTOM).max(INFO_BAR_MIN_HEIGHT)
        }
    }

    fn text_left(&self) -> f32 {
        INFO_BAR_PADDING_LEFT + INFO_BAR_ICON_SIZE + INFO_BAR_ICON_MARGIN.right
    }

    fn text_right(&self) -> f32 {
        INFO_BAR_PANEL_MARGIN_RIGHT + if self.closable { INFO_BAR_CLOSE_BUTTON_SIZE + INFO_BAR_CLOSE_BUTTON_MARGIN * 2.0 - INFO_BAR_PANEL_MARGIN_RIGHT } else { 0.0 }
    }

    // Whether the title and message fit on one line, and if not, the
    // message's height, wrapped.
    fn layout(&self, fonts: &mut Fonts, width: f32) -> (bool, f32) {
        let room = (width - self.text_left() - self.text_right()).max(1.0);
        let title = fonts.measure(&self.title, BODY_STRONG).width;
        let message = fonts.measure(&self.message, BODY).width;
        if title + INFO_BAR_MESSAGE_GAP + message <= room {
            return (true, BODY.line_height);
        }
        (false, fonts.measure_wrapped(&self.message, BODY, room).height)
    }

    fn close_rect(rect: Rect) -> Rect {
        let size = INFO_BAR_CLOSE_BUTTON_SIZE;
        Rect::new(rect.right() - INFO_BAR_CLOSE_BUTTON_MARGIN - size, rect.y + INFO_BAR_CLOSE_BUTTON_MARGIN, size, size)
    }
}

impl<M: Clone + 'static> Widget<M> for InfoBar<M> {
    fn kind(&self) -> &'static str {
        "InfoBar"
    }

    fn label(&self) -> Option<String> {
        Some(format!("{}: {} {}", self.severity.name(), self.title, self.message).trim_end().to_string())
    }

    fn sense(&self) -> Option<Sense> {
        Some(Sense::NONE)
    }

    fn parts(&self, ctx: &mut Parts) {
        if self.closable {
            ctx.part(Part::Named(CLOSE.to_string()), CLOSE, Self::close_rect(ctx.rect()), Sense::BUTTON, "Button", None);
        }
    }

    fn interact(&self, interaction: &Interaction, ctx: &mut Dispatch<M>) {
        if interaction.clicked(&Part::Named(CLOSE.to_string()))
            && let Some(message) = &self.on_close
        {
            ctx.send(message.clone());
        }
    }

    fn draw(&self, ctx: &mut Draw) {
        let rect = ctx.rect();
        let (one_line, _) = self.layout(ctx.fonts(), rect.width);
        let look = ctx.look().clone();
        {
            let mut painter = ctx.painter();
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
            // The icon's top is 16 from the bar's, as is a one-line bar's
            // middle.
            let icon_center = Point::new(rect.x + INFO_BAR_PADDING_LEFT + INFO_BAR_ICON_SIZE / 2.0, rect.y + INFO_BAR_ICON_MARGIN.top + INFO_BAR_ICON_SIZE / 2.0);
            status_icon(&mut painter, icon_center, self.severity, INFO_BAR_ICON_SIZE);
        }
        if self.closable {
            let part = Part::Named(CLOSE.to_string());
            let close = Self::close_rect(rect);
            let state = look.part_state(&part);
            draw_button(ctx, close, "", Some(Icon::Dismiss), ButtonStyle::Subtle, false, false, state);
            let focused = look.focused && look.focused_part.as_ref() == Some(&part);
            ctx.focus_visual_if(focused, close, RADIUS_CONTROL);
        }
    }
}
