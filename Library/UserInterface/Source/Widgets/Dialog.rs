// WinUI's ContentDialog: a box in the middle of the window, over smoke that
// keeps the page from input, with a title, a message or content of the
// program's own, and up to three buttons, of which the default is the
// accent's. Esc is its close button, and Enter its default one.
//
// The program shows it each frame while it's open, and closes it when it
// says which button was chosen.

use crate::core::ui::{Sense, Ui};
use crate::design::controls::{DIALOG_BUTTON_GAP, DIALOG_MAX_WIDTH, DIALOG_MIN_HEIGHT, DIALOG_MIN_WIDTH, DIALOG_PADDING, DIALOG_TITLE_GAP};
use crate::design::tokens::{BODY, CONTROL_HEIGHT, ELEVATION_DIALOG, RADIUS_OVERLAY, SUBTITLE};
use crate::drawing::geometry::{Rect, Size};
use crate::drawing::painter::{Border, Corners, Layer};
use crate::input::devices::Key;
use crate::widgets::buttons::Button;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DialogResult {
    Primary,
    Secondary,
    Close,
}

#[derive(Clone, Copy, Debug)]
pub struct Dialog<'a> {
    title: &'a str,
    message: &'a str,
    primary: Option<&'a str>,
    secondary: Option<&'a str>,
    close: &'a str,
    default: Option<DialogResult>,
    width: Option<f32>,
}

impl<'a> Dialog<'a> {
    pub fn new(title: &'a str, message: &'a str) -> Self {
        Self {
            title,
            message,
            primary: None,
            secondary: None,
            close: "Close",
            default: None,
            width: None,
        }
    }

    // The primary button, which is the default, in the accent's colors.
    pub fn primary(mut self, text: &'a str) -> Self {
        self.primary = Some(text);
        self.default.get_or_insert(DialogResult::Primary);
        self
    }

    pub fn secondary(mut self, text: &'a str) -> Self {
        self.secondary = Some(text);
        self
    }

    pub fn close(mut self, text: &'a str) -> Self {
        self.close = text;
        self
    }

    pub fn default_button(mut self, button: DialogResult) -> Self {
        self.default = Some(button);
        self
    }

    // Its width, between ContentDialogMinWidth and ContentDialogMaxWidth.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }
}

impl Ui {
    // Shows the dialog, and returns the button that was chosen, if one was.
    pub fn dialog(&mut self, name: &str, dialog: Dialog) -> Option<DialogResult> {
        self.dialog_with(name, dialog, 0.0, |_, _| {})
    }

    // Shows the dialog with content of the program's own, `content_height`
    // tall, under its message, which `content` draws in the rectangle it's
    // given.
    pub fn dialog_with(&mut self, name: &str, dialog: Dialog, content_height: f32, content: impl FnOnce(&mut Ui, Rect)) -> Option<DialogResult> {
        let bounds = self.bounds();
        let width = dialog.width.unwrap_or(DIALOG_MAX_WIDTH).clamp(DIALOG_MIN_WIDTH, DIALOG_MAX_WIDTH);
        let inner = width - DIALOG_PADDING * 2.0;
        let title_height = self.paragraph_height(inner, dialog.title, SUBTITLE);
        let message_height = if dialog.message.is_empty() { 0.0 } else { self.paragraph_height(inner, dialog.message, BODY) };
        let gap = |height: f32| if height > 0.0 { DIALOG_TITLE_GAP } else { 0.0 };
        let top_height = DIALOG_PADDING * 2.0 + title_height + gap(message_height) + message_height + gap(content_height) + content_height;
        let commands_height = DIALOG_PADDING * 2.0 + CONTROL_HEIGHT;
        let height = (top_height + commands_height).max(DIALOG_MIN_HEIGHT);
        let rect = bounds.centered(Size::new(width, height)).round();

        let layer = self.layer();
        self.set_layer(Layer::Dialog);
        self.draw.push_clip(bounds, true);
        // The smoke covers the page and takes its input.
        self.register(&format!("{name}/Smoke"), bounds, Sense::NONE, "Smoke");
        self.register(name, rect, Sense::NONE, "ContentDialog");
        {
            let mut painter = self.painter();
            let theme = painter.theme();
            painter.fill(bounds, theme.smoke);
            painter.shadow(rect, RADIUS_OVERLAY, ELEVATION_DIALOG);
            painter.shape(rect, RADIUS_OVERLAY, theme.background, Some(Border::new(theme.surface_stroke, 1.0)));
            let top = Rect::new(rect.x + 1.0, rect.y + 1.0, rect.width - 2.0, top_height - 1.0);
            painter.shape(top, Corners::top(RADIUS_OVERLAY - 1.0), theme.layer_alt, None);
            painter.fill(Rect::new(rect.x + 1.0, rect.y + top_height - 1.0, rect.width - 2.0, 1.0), theme.card_stroke);
            let mut area = rect.pad(crate::drawing::geometry::Insets::all(DIALOG_PADDING));
            let title = area.cut_top(title_height, gap(message_height + content_height));
            painter.text_wrapped(title, dialog.title, SUBTITLE, theme.text_primary);
            if message_height > 0.0 {
                let message = area.cut_top(message_height, gap(content_height));
                painter.text_wrapped(message, dialog.message, BODY, theme.text_primary);
            }
        }
        if content_height > 0.0 {
            let top = rect.y + DIALOG_PADDING + title_height + gap(message_height + content_height) + message_height + gap(content_height);
            content(self, Rect::new(rect.x + DIALOG_PADDING, top, inner, content_height));
        }

        // The buttons share the command area's width.
        let mut buttons: Vec<(&str, DialogResult)> = Vec::new();
        if let Some(text) = dialog.primary {
            buttons.push((text, DialogResult::Primary));
        }
        if let Some(text) = dialog.secondary {
            buttons.push((text, DialogResult::Secondary));
        }
        if !dialog.close.is_empty() {
            buttons.push((dialog.close, DialogResult::Close));
        }
        let row = Rect::new(rect.x + DIALOG_PADDING, rect.bottom() - DIALOG_PADDING - CONTROL_HEIGHT, inner, CONTROL_HEIGHT);
        let count = buttons.len().max(1) as f32;
        let button_width = ((row.width - DIALOG_BUTTON_GAP * (count - 1.0)) / count).floor();
        let mut chosen = None;
        for (index, (text, result)) in buttons.iter().enumerate() {
            let x = if index + 1 == buttons.len() { row.right() - button_width } else { row.x + (button_width + DIALOG_BUTTON_GAP) * index as f32 };
            let button_rect = Rect::new(x, row.y, button_width, row.height);
            let mut button = Button::new(text);
            if dialog.default == Some(*result) {
                button = button.accent();
            }
            let label = match result {
                DialogResult::Primary => "Primary",
                DialogResult::Secondary => "Secondary",
                DialogResult::Close => "Close",
            };
            if self.add(&format!("{name}/{label}"), button_rect, button).clicked {
                chosen = Some(*result);
            }
        }
        if self.dialog_escape() {
            chosen = Some(DialogResult::Close);
        }
        if chosen.is_none() && dialog.default.is_some() && self.input().key_pressed_once(Key::ENTER) && self.focused_name().is_none_or(|focused| !focused.starts_with(name)) {
            chosen = dialog.default;
        }
        self.draw.pop_clip();
        self.set_layer(layer);
        chosen
    }
}
