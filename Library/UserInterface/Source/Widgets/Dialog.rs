// WinUI's ContentDialog: a box in the middle of the window, over smoke that
// keeps the page from input, with a title, a message, and up to three
// buttons, of which the default is the accent's. Esc is its close button,
// and Enter its default one.
//
// The dialog is a node on the dialog layer whose visibility is whether it's
// open: the program shows it with ui.set_visible(name, true), and hides it
// when a Dialog event tells which button was chosen. Children, laid out in
// Dialog::content_area, show while it's open. It runs in the dispatch phase,
// smoke, panel and buttons together, since the dialog layer draws over
// everything below it.

use crate::core::control::{AutoTarget, Control, NodeContext, UiEventKind};
use crate::core::ui::{Sense, Ui};
use crate::design::controls::{DIALOG_BUTTON_GAP, DIALOG_MAX_WIDTH, DIALOG_MIN_HEIGHT, DIALOG_MIN_WIDTH, DIALOG_PADDING, DIALOG_TITLE_GAP};
use crate::design::tokens::{BODY, CONTROL_HEIGHT, ELEVATION_DIALOG, RADIUS_OVERLAY, SUBTITLE};
use crate::drawing::geometry::{Rect, Size};
use crate::drawing::painter::{Border, Corners, Layer};
use crate::input::devices::Key;
use crate::widgets::buttons::{ButtonStyle, inline_button};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DialogResult {
    Primary,
    Secondary,
    Close,
}

#[derive(Clone, Debug)]
pub struct Dialog {
    title: String,
    message: String,
    primary: Option<String>,
    secondary: Option<String>,
    close: String,
    default: Option<DialogResult>,
    width: Option<f32>,
    // Room under the message for the program's children.
    content_height: f32,
}

impl Dialog {
    pub fn new(title: &str, message: &str) -> Self {
        Self {
            title: title.to_string(),
            message: message.to_string(),
            primary: None,
            secondary: None,
            close: "Close".to_string(),
            default: None,
            width: None,
        content_height: 0.0,
        }
    }

    // The primary button, which is the default, in the accent's colors.
    pub fn primary(mut self, text: &str) -> Self {
        self.primary = Some(text.to_string());
        self.default.get_or_insert(DialogResult::Primary);
        self
    }

    pub fn secondary(mut self, text: &str) -> Self {
        self.secondary = Some(text.to_string());
        self
    }

    pub fn close(mut self, text: &str) -> Self {
        self.close = text.to_string();
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

    // Room under the message for content of the program's own: its children,
    // laid out in Dialog::content_area.
    pub fn content_height(mut self, height: f32) -> Self {
        self.content_height = height;
        self
    }

    pub fn set_title(&mut self, title: &str) {
        self.title = title.to_string();
    }

    pub fn set_message(&mut self, message: &str) {
        self.message = message.to_string();
    }

    fn widths(&self) -> (f32, f32) {
        let width = self.width.unwrap_or(DIALOG_MAX_WIDTH).clamp(DIALOG_MIN_WIDTH, DIALOG_MAX_WIDTH);
        (width, width - DIALOG_PADDING * 2.0)
    }

    fn layout(&self, ui: &mut Ui) -> DialogLayout {
        let bounds = ui.bounds();
        let (width, inner) = self.widths();
        let title_height = ui.paragraph_height(inner, &self.title, SUBTITLE);
        let message_height = if self.message.is_empty() { 0.0 } else { ui.paragraph_height(inner, &self.message, BODY) };
        let gap = |height: f32| if height > 0.0 { DIALOG_TITLE_GAP } else { 0.0 };
        let top_height = DIALOG_PADDING * 2.0 + title_height + gap(message_height) + message_height + gap(self.content_height) + self.content_height;
        let commands_height = DIALOG_PADDING * 2.0 + CONTROL_HEIGHT;
        let height = (top_height + commands_height).max(DIALOG_MIN_HEIGHT);
        let rect = bounds.centered(Size::new(width, height)).round();
        DialogLayout {
            bounds,
            rect,
            inner,
            title_height,
            message_height,
            top_height,
        }
    }

    // Where the dialog's own content goes, for laying out its children at
    // build: under its message, in a dialog shown in `bounds` (the window
    // under the PerfBar).
    pub fn content_area(&self, ui: &mut Ui, content_height: f32) -> Rect {
        let layout = Dialog {
            content_height,
            ..self.clone()
        }
        .layout(ui);
        let gap = |height: f32| if height > 0.0 { DIALOG_TITLE_GAP } else { 0.0 };
        let top = layout.rect.y + DIALOG_PADDING + layout.title_height + gap(layout.message_height + content_height) + layout.message_height + gap(content_height);
        Rect::new(layout.rect.x + DIALOG_PADDING, top, layout.inner, content_height)
    }
}

struct DialogLayout {
    bounds: Rect,
    rect: Rect,
    inner: f32,
    title_height: f32,
    message_height: f32,
    top_height: f32,
}

impl Control for Dialog {
    fn kind(&self) -> &'static str {
        "ContentDialog"
    }

    fn auto_target(&self) -> AutoTarget {
        AutoTarget::None
    }

    // Its children are its content, on the dialog layer.
    fn child_layer(&self) -> Option<Layer> {
        Some(Layer::Dialog)
    }

    fn handle(&mut self, ui: &mut Ui, ctx: &NodeContext) {
        let name = &ctx.name;
        let layout = self.layout(ui);
        let (bounds, rect) = (layout.bounds, layout.rect);

        let layer = ui.layer();
        ui.set_layer(Layer::Dialog);
        ui.draw.push_clip(bounds, true);
        // The smoke covers the page and takes its input.
        ui.register(&format!("{name}/Smoke"), bounds, Sense::NONE, "Smoke");
        ui.register(name, rect, Sense::NONE, "ContentDialog");
        let gap = |height: f32| if height > 0.0 { DIALOG_TITLE_GAP } else { 0.0 };
        {
            let mut painter = ui.painter();
            let theme = painter.theme();
            painter.fill(bounds, theme.smoke);
            painter.shadow(rect, RADIUS_OVERLAY, ELEVATION_DIALOG);
            painter.shape(rect, RADIUS_OVERLAY, theme.background, Some(Border::new(theme.surface_stroke, 1.0)));
            let top = Rect::new(rect.x + 1.0, rect.y + 1.0, rect.width - 2.0, layout.top_height - 1.0);
            painter.shape(top, Corners::top(RADIUS_OVERLAY - 1.0), theme.layer_alt, None);
            painter.fill(Rect::new(rect.x + 1.0, rect.y + layout.top_height - 1.0, rect.width - 2.0, 1.0), theme.card_stroke);
            let mut area = rect.pad(crate::drawing::geometry::Insets::all(DIALOG_PADDING));
            let title = area.cut_top(layout.title_height, gap(layout.message_height + self.content_height));
            painter.text_wrapped(title, &self.title, SUBTITLE, theme.text_primary);
            if layout.message_height > 0.0 {
                let message = area.cut_top(layout.message_height, gap(self.content_height));
                painter.text_wrapped(message, &self.message, BODY, theme.text_primary);
            }
        }
        // Whether a popup, like a combo box's list in the content, was open
        // as the frame began: it takes this frame's Enter, even if it closes
        // on it.
        let popup_open = ui.any_popup();

        // The buttons share the command area's width.
        let mut buttons: Vec<(&str, DialogResult)> = Vec::new();
        if let Some(text) = &self.primary {
            buttons.push((text, DialogResult::Primary));
        }
        if let Some(text) = &self.secondary {
            buttons.push((text, DialogResult::Secondary));
        }
        if !self.close.is_empty() {
            buttons.push((&self.close, DialogResult::Close));
        }
        let row = Rect::new(rect.x + DIALOG_PADDING, rect.bottom() - DIALOG_PADDING - CONTROL_HEIGHT, layout.inner, CONTROL_HEIGHT);
        let count = buttons.len().max(1) as f32;
        let button_width = ((row.width - DIALOG_BUTTON_GAP * (count - 1.0)) / count).floor();
        let mut chosen = None;
        for (index, (text, result)) in buttons.iter().enumerate() {
            let x = if index + 1 == buttons.len() { row.right() - button_width } else { row.x + (button_width + DIALOG_BUTTON_GAP) * index as f32 };
            let button_rect = Rect::new(x, row.y, button_width, row.height);
            let style = if self.default == Some(*result) { ButtonStyle::Accent } else { ButtonStyle::Standard };
            let label = match result {
                DialogResult::Primary => "Primary",
                DialogResult::Secondary => "Secondary",
                DialogResult::Close => "Close",
            };
            if inline_button(ui, &format!("{name}/{label}"), button_rect, text, style).clicked {
                chosen = Some(*result);
            }
        }
        if ui.dialog_escape() {
            chosen = Some(DialogResult::Close);
        }
        // Enter is the default button's, unless one of the dialog's own
        // widgets has focus, or a popup is open, like a combo box's list in
        // the dialog's content, which takes it.
        if chosen.is_none()
            && self.default.is_some()
            && !popup_open
            && ui.input().key_pressed_once(Key::ENTER)
            && ui.focused_name().is_none_or(|focused| !focused.starts_with(name.as_str()))
        {
            chosen = self.default;
        }
        ui.draw.pop_clip();
        ui.set_layer(layer);
        if let Some(result) = chosen {
            ctx.emit(ui, UiEventKind::Dialog(result));
        }
    }

    fn draw(&mut self, _ui: &mut Ui, _ctx: &NodeContext) {}
}
