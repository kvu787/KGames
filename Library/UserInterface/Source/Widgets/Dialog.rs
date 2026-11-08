// WinUI's ContentDialog: a box in the middle of the window, over smoke that
// keeps the page from input, with a title, a message or content of the
// program's own, and up to three buttons, of which the default is the
// accent's. Esc is its close button, and Enter its default one.
//
// The program's view adds it while it's open, in the area that `area`
// gives, with its own content as children in `area.content`, and the
// dialog sends which button was chosen:
//
//     let dialog = Dialog::new("Quit?", "Unsaved work is lost.").primary("Quit").on_result(Message::Quit);
//     let area = dialog.area(ui);
//     tree.add("Quit", area.rect, dialog);

use crate::core::interaction::{Interaction, Part, Sense};
use crate::core::interface::Ui;
use crate::core::tree::{Dispatch, Draw, Parts, Widget};
use crate::design::controls::{DIALOG_BUTTON_GAP, DIALOG_MAX_WIDTH, DIALOG_MIN_HEIGHT, DIALOG_MIN_WIDTH, DIALOG_PADDING, DIALOG_TITLE_GAP};
use crate::design::tokens::{BODY, CONTROL_HEIGHT, ELEVATION_DIALOG, RADIUS_CONTROL, RADIUS_OVERLAY, SUBTITLE};
use crate::drawing::geometry::{Insets, Rect, Size};
use crate::drawing::painter::{Border, Corners, Layer};
use crate::drawing::text::Fonts;
use crate::input::devices::Key;
use crate::widgets::buttons::{ButtonStyle, draw_button};
use crate::widgets::widget::{Reaction, react};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DialogResult {
    Primary,
    Secondary,
    Close,
}

impl DialogResult {
    // The part of the dialog that its button is, named "<dialog>/Primary"
    // and so on.
    fn part_name(self) -> &'static str {
        match self {
            DialogResult::Primary => "Primary",
            DialogResult::Secondary => "Secondary",
            DialogResult::Close => "Close",
        }
    }
}

pub struct Dialog<M> {
    title: String,
    message: String,
    primary: Option<String>,
    secondary: Option<String>,
    close: String,
    default: Option<DialogResult>,
    width: Option<f32>,
    // How tall the program's own content under the message is.
    content_height: f32,
    on_result: Option<Reaction<DialogResult, M>>,
}

// Where a dialog is, and where its content goes.
#[derive(Clone, Copy, Debug, Default)]
pub struct DialogArea {
    pub rect: Rect,
    pub content: Rect,
}

// A dialog's layout, from its texts.
struct Metrics {
    rect: Rect,
    inner: f32,
    title_height: f32,
    message_height: f32,
    top_height: f32,
    content: Rect,
}

fn gap(height: f32) -> f32 {
    if height > 0.0 { DIALOG_TITLE_GAP } else { 0.0 }
}

impl<M: Clone + 'static> Dialog<M> {
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            primary: None,
            secondary: None,
            close: "Close".to_string(),
            default: None,
            width: None,
            content_height: 0.0,
            on_result: None,
        }
    }

    // The primary button, which is the default, in the accent's colors.
    pub fn primary(mut self, text: impl Into<String>) -> Self {
        self.primary = Some(text.into());
        self.default.get_or_insert(DialogResult::Primary);
        self
    }

    pub fn secondary(mut self, text: impl Into<String>) -> Self {
        self.secondary = Some(text.into());
        self
    }

    pub fn close(mut self, text: impl Into<String>) -> Self {
        self.close = text.into();
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

    // Room under the message for content of the program's own, which goes
    // in the dialog's children, in `area().content`.
    pub fn content_height(mut self, height: f32) -> Self {
        self.content_height = height;
        self
    }

    // What choosing a button sends.
    pub fn on_result(mut self, reaction: impl Fn(DialogResult) -> M + 'static) -> Self {
        self.on_result = Some(react(reaction));
        self
    }

    // Where the dialog goes in the window, and where its content goes.
    pub fn area(&self, ui: &mut Ui) -> DialogArea {
        let bounds = ui.bounds();
        let metrics = self.metrics(ui.fonts(), bounds);
        DialogArea {
            rect: metrics.rect,
            content: metrics.content,
        }
    }

    fn metrics(&self, fonts: &mut Fonts, bounds: Rect) -> Metrics {
        let width = self.width.unwrap_or(DIALOG_MAX_WIDTH).clamp(DIALOG_MIN_WIDTH, DIALOG_MAX_WIDTH);
        let inner = width - DIALOG_PADDING * 2.0;
        let title_height = fonts.measure_wrapped(&self.title, SUBTITLE, inner).height;
        let message_height = if self.message.is_empty() { 0.0 } else { fonts.measure_wrapped(&self.message, BODY, inner).height };
        let content_height = self.content_height;
        let top_height = DIALOG_PADDING * 2.0 + title_height + gap(message_height) + message_height + gap(content_height) + content_height;
        let commands_height = DIALOG_PADDING * 2.0 + CONTROL_HEIGHT;
        let height = (top_height + commands_height).max(DIALOG_MIN_HEIGHT);
        let rect = bounds.centered(Size::new(width, height)).round();
        let content_top = rect.y + DIALOG_PADDING + title_height + gap(message_height + content_height) + message_height + gap(content_height);
        Metrics {
            rect,
            inner,
            title_height,
            message_height,
            top_height,
            content: Rect::new(rect.x + DIALOG_PADDING, content_top, inner, content_height),
        }
    }

    // The buttons, which share the command area's width, the last at its
    // right.
    fn buttons(&self, rect: Rect) -> Vec<(Rect, DialogResult, &str)> {
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
        let inner = rect.width - DIALOG_PADDING * 2.0;
        let row = Rect::new(rect.x + DIALOG_PADDING, rect.bottom() - DIALOG_PADDING - CONTROL_HEIGHT, inner, CONTROL_HEIGHT);
        let count = buttons.len().max(1) as f32;
        let button_width = ((row.width - DIALOG_BUTTON_GAP * (count - 1.0)) / count).floor();
        buttons
            .iter()
            .enumerate()
            .map(|(index, (text, result))| {
                let x = if index + 1 == buttons.len() { row.right() - button_width } else { row.x + (button_width + DIALOG_BUTTON_GAP) * index as f32 };
                (Rect::new(x, row.y, button_width, row.height), *result, *text)
            })
            .collect()
    }
}

impl<M: Clone + 'static> Widget<M> for Dialog<M> {
    fn kind(&self) -> &'static str {
        "ContentDialog"
    }

    fn label(&self) -> Option<String> {
        Some(self.title.clone())
    }

    fn sense(&self) -> Option<Sense> {
        Some(Sense::NONE)
    }

    fn layer(&self) -> Layer {
        Layer::Dialog
    }

    fn always_interacts(&self) -> bool {
        true
    }

    fn parts(&self, ctx: &mut Parts) {
        // The smoke covers the page and takes its input.
        let bounds = ctx.bounds();
        ctx.cover_on(Layer::Dialog, bounds);
        for (rect, result, text) in self.buttons(ctx.rect()) {
            let name = result.part_name();
            ctx.part(Part::Named(name.to_string()), name, rect, Sense::BUTTON, "Button", Some(text.to_string()));
        }
    }

    fn interact(&self, interaction: &Interaction, ctx: &mut Dispatch<M>) {
        let name = ctx.name().to_string();
        let mut chosen = None;
        for (_, result, _) in self.buttons(ctx.rect()) {
            if interaction.clicked(&Part::Named(result.part_name().to_string())) {
                chosen = Some(result);
            }
        }
        if ctx.key_pressed_once(Key::ESCAPE) {
            chosen = Some(DialogResult::Close);
            ctx.consume_escape();
        }
        // Enter is the default button's, unless one of the dialog's own
        // widgets has focus, or a popup is open, like a combo box's list in
        // the dialog's content, which takes it.
        if chosen.is_none()
            && self.default.is_some()
            && !ctx.any_popup()
            && ctx.key_pressed_once(Key::ENTER)
            && ctx.focused_name().is_none_or(|focused| !focused.starts_with(&name))
        {
            chosen = self.default;
        }
        if let Some(result) = chosen
            && let Some(reaction) = &self.on_result
        {
            ctx.send(reaction(result));
        }
    }

    fn draw(&self, ctx: &mut Draw) {
        let rect = ctx.rect();
        let bounds = ctx.bounds();
        let metrics = self.metrics(ctx.fonts(), bounds);
        let look = ctx.look().clone();
        {
            let mut painter = ctx.painter();
            let theme = painter.theme();
            painter.fill(bounds, theme.smoke);
            painter.shadow(rect, RADIUS_OVERLAY, ELEVATION_DIALOG);
            painter.shape(rect, RADIUS_OVERLAY, theme.background, Some(Border::new(theme.surface_stroke, 1.0)));
            let top = Rect::new(rect.x + 1.0, rect.y + 1.0, rect.width - 2.0, metrics.top_height - 1.0);
            painter.shape(top, Corners::top(RADIUS_OVERLAY - 1.0), theme.layer_alt, None);
            painter.fill(Rect::new(rect.x + 1.0, rect.y + metrics.top_height - 1.0, rect.width - 2.0, 1.0), theme.card_stroke);
            let mut area = rect.pad(Insets::all(DIALOG_PADDING));
            let title = area.cut_top(metrics.title_height, gap(metrics.message_height + self.content_height));
            painter.text_wrapped(title, &self.title, SUBTITLE, theme.text_primary);
            if metrics.message_height > 0.0 {
                let message = area.cut_top(metrics.message_height, gap(self.content_height));
                painter.text_wrapped(message, &self.message, BODY, theme.text_primary);
            }
            let _ = metrics.inner;
        }
        for (button_rect, result, text) in self.buttons(rect) {
            let part = Part::Named(result.part_name().to_string());
            let style = if self.default == Some(result) { ButtonStyle::Accent } else { ButtonStyle::Standard };
            let state = look.part_state(&part);
            draw_button(ctx, button_rect, text, None, style, false, false, state);
            let focused = look.focused && look.focused_part.as_ref() == Some(&part);
            ctx.focus_visual_if(focused, button_rect, RADIUS_CONTROL);
        }
    }
}
