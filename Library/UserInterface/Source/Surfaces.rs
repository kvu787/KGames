// What holds widgets and text: cards, wells inside them, cards over a 3D
// view, info bars, and dividers. And what the widgets share in how they look:
// the raised face of a control, a shadow under what floats, and the ring
// around the widget with keyboard focus.

use canvas::{Align, Canvas, Color, Icon, Rect, TextStyle, Theme};

use crate::tokens::*;
use crate::{Button, Frame, Response};

// How serious an info bar's message is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Severity {
    #[default]
    Info,
    Success,
    Caution,
    Critical,
}

impl Severity {
    pub fn color(self, theme: &Theme) -> Color {
        match self {
            Severity::Info => theme.accent,
            Severity::Success => theme.success,
            Severity::Caution => theme.caution,
            Severity::Critical => theme.critical,
        }
    }

    pub fn background(self, theme: &Theme) -> Color {
        match self {
            Severity::Info => theme.info_background,
            Severity::Success => theme.success_background,
            Severity::Caution => theme.caution_background,
            Severity::Critical => theme.critical_background,
        }
    }

    pub fn icon(self) -> Icon {
        match self {
            Severity::Info => Icon::Info,
            Severity::Success => Icon::Success,
            Severity::Caution => Icon::Warning,
            Severity::Critical => Icon::Error,
        }
    }
}

// The two rings around a widget with keyboard focus, from Tab: a strong one
// outside, and a thin one of the background inside it, so that it shows
// against any color.
pub fn focus_ring(canvas: &mut Canvas, theme: &Theme, area: Rect, radius: f32) {
    canvas.stroke(area.outset(3.0), radius + 3.0, 2.0, theme.focus_outer);
    canvas.stroke(area.outset(1.0), radius + 1.0, 1.0, theme.focus_inner);
}

// The shadow under something that floats as high as `elevation`.
pub fn shadow(canvas: &mut Canvas, theme: &Theme, area: Rect, radius: f32, elevation: Elevation) {
    let color = theme.shadow.alpha(elevation.strength);
    canvas.shadow(area.translate([0.0, elevation.offset]), radius, elevation.blur, color);
}

// A control's raised face: a fill, an outline, and a darker bottom edge,
// like Fluent's buttons.
pub fn raised(canvas: &mut Canvas, area: Rect, radius: f32, fill: Color, stroke: Color, bottom: Color) {
    canvas.fill(area, radius, bottom);
    let top = Rect::new(area.x, area.y, area.width, area.height - 1.0);
    canvas.fill_stroke(top, radius, fill, 1.0, stroke);
}

// A card: a rounded box on the window's background, which groups what's in
// it, with a title and a few words about it after the title.
pub fn card(canvas: &mut Canvas, theme: &Theme, area: Rect, title: &str, subtitle: &str) -> Rect {
    canvas.fill_stroke(area, RADIUS_CARD, theme.card, 1.0, theme.card_stroke);
    if title.is_empty() {
        return area.inset(CARD_PADDING);
    }
    let top = area.y + CARD_PADDING;
    let right = area.right() - CARD_PADDING;
    let x = area.x + CARD_PADDING;
    let end = canvas.text_clipped([x, top], title, right - x, TextStyle::BodyStrong, theme.text);
    if !subtitle.is_empty() {
        let x = end + SPACE_S;
        canvas.text_clipped([x, top], subtitle, (right - x).max(0.0), TextStyle::Body, theme.text_secondary);
    }
    Rect::from_edges(area.x + CARD_PADDING, area.y + CARD_HEADER, right, area.bottom() - CARD_PADDING)
}

// Where a control goes on a card's title line, after its title: as tall as
// a control, and as wide as what's left of the line.
pub fn card_trailing(canvas: &Canvas, area: Rect, title: &str) -> Rect {
    let x = area.x + CARD_PADDING + canvas.measure(title, TextStyle::BodyStrong) + SPACE_M;
    let middle = area.y + CARD_PADDING + 10.0;
    Rect::from_edges(x, middle - CONTROL_HEIGHT * 0.5, area.right() - CARD_PADDING, middle + CONTROL_HEIGHT * 0.5)
}

// A well inside a card, for a plot or a log.
pub fn well(canvas: &mut Canvas, theme: &Theme, area: Rect) {
    canvas.fill_stroke(area, RADIUS_CONTROL, theme.well, 1.0, theme.well_stroke);
}

// A card over a 3D view, which floats over it.
pub fn overlay_card(canvas: &mut Canvas, theme: &Theme, area: Rect) {
    shadow(canvas, theme, area, RADIUS_CARD, RAISED);
    canvas.fill_stroke(area, RADIUS_CARD, theme.overlay, 1.0, theme.card_stroke);
}

// A line across, to separate what's on either side.
pub fn divider(canvas: &mut Canvas, theme: &Theme, from: [f32; 2], length: f32) {
    canvas.rect(Rect::new(from[0], from[1], length, 1.0), theme.divider);
}

// How tall an info bar is for its message, wrapped to its width.
pub fn info_bar_height(canvas: &Canvas, width: f32, title: &str, message: &str, closable: bool) -> f32 {
    let lines = info_bar_lines(canvas, width, title, message, closable);
    (lines.len() as f32 * canvas.line_height(TextStyle::Body) + 28.0).max(48.0)
}

// The lines of an info bar's message, after its title on the first.
fn info_bar_lines(canvas: &Canvas, width: f32, title: &str, message: &str, closable: bool) -> Vec<String> {
    let text_width = width - 16.0 - 16.0 - SPACE_M - if closable { 40.0 } else { 16.0 };
    let title_width = if title.is_empty() { 0.0 } else { canvas.measure(title, TextStyle::BodyStrong) + SPACE_M };
    let mut lines = canvas.fonts.wrap(message, TextStyle::Body, text_width - title_width, 0.0);
    // Lines after the first have the width under the title too.
    if lines.len() > 1 {
        let first = lines.remove(0);
        let rest = lines.join(" ");
        let mut rest = canvas.fonts.wrap(&rest, TextStyle::Body, text_width, 0.0);
        rest.insert(0, first);
        lines = rest;
    }
    lines
}

impl Frame<'_> {
    // Draws a card, and returns the area under its title for its contents.
    pub fn card(&mut self, area: Rect, title: &str, subtitle: &str) -> Rect {
        let theme = self.theme();
        card(&mut self.canvas, theme, area, title, subtitle)
    }

    // Where a control goes after a card's title. See card_trailing.
    pub fn card_trailing(&self, area: Rect, title: &str) -> Rect {
        card_trailing(&self.canvas, area, title)
    }

    pub fn well(&mut self, area: Rect) {
        let theme = self.theme();
        well(&mut self.canvas, theme, area);
    }

    pub fn overlay_card(&mut self, area: Rect) {
        let theme = self.theme();
        overlay_card(&mut self.canvas, theme, area);
    }

    pub fn divider(&mut self, from: [f32; 2], length: f32) {
        let theme = self.theme();
        divider(&mut self.canvas, theme, from, length);
    }

    // An info bar: a message about something the user should know, with an
    // icon of how serious it is, and its title before it, on the base layer.
    // With a name, it has a button that closes it, called "<name>", whose
    // response this returns.
    pub fn info_bar(&mut self, name: Option<&str>, area: Rect, severity: Severity, title: &str, message: &str) -> Response {
        self.info_bar_on(crate::Layer::Base, name, area, severity, title, message)
    }

    pub(crate) fn info_bar_on(
        &mut self,
        layer: crate::Layer,
        name: Option<&str>,
        area: Rect,
        severity: Severity,
        title: &str,
        message: &str,
    ) -> Response {
        let theme = self.theme();
        let lines = info_bar_lines(&self.canvas, area.width, title, message, name.is_some());
        let canvas = self.layer(layer);
        canvas.fill_stroke(area, RADIUS_CONTROL, severity.background(theme), 1.0, theme.card_stroke);
        let line_height = canvas.line_height(TextStyle::Body);
        let top = area.y + 14.0;
        canvas.icon([area.x + 16.0 + 8.0, top + line_height * 0.5], severity.icon(), 16.0, severity.color(theme));
        let x = area.x + 16.0 + 16.0 + SPACE_M;
        let mut start = x;
        if !title.is_empty() {
            start = canvas.text([x, top], title, TextStyle::BodyStrong, theme.text) + SPACE_M;
        }
        for (index, line) in lines.iter().enumerate() {
            let x = if index == 0 { start } else { x };
            let y = top + line_height * index as f32;
            canvas.text([x, y], line, TextStyle::Body, theme.text);
        }
        let Some(name) = name else {
            return Response::default();
        };
        let close = Rect::new(area.right() - 8.0 - CONTROL_HEIGHT, area.y + 8.0, CONTROL_HEIGHT, CONTROL_HEIGHT);
        self.button_on(layer, name, close, Button::icon_only(Icon::Dismiss).subtle())
    }

    // Text in an area, centered down it, and across it as `align` says.
    pub fn label(&mut self, area: Rect, text: &str, style: TextStyle, color: Color, align: Align) -> f32 {
        self.canvas.text_in(area, text, style, color, align)
    }
}
