// What IdentifyDisplays shows on each display: a box with the display's
// identifier, its model and its mode, and whether a click or Esc closes the
// boxes, as a library that Test/IdentifyDisplays tests without a window.

use displays::Display;
use user_interface::tokens::{BODY, BODY_LARGE, ICON, SPACE_S, SPACE_XXXL, TITLE};
use user_interface::{Align, Border, Color, Corners, Draw, Font, Fonts, Point, Rect, Severity, TextStyle, Weight, Widget, status_icon};

// The identifier is in Cascadia Mono, whose 0 has a slash through it.
const IDENTIFIER: TextStyle = TextStyle::new(Font::Mono, Weight::Semibold, 112.0, 132.0);
const ACCENT_BAR: f32 = 4.0;
const MINIMUM_WIDTH: f32 = 360.0;

// The last line, after a status icon: whether a click or Esc closes the boxes,
// which they do only while one of them has focus, as the input comes from
// GameInput, which doesn't report the click that gives a box focus.
pub const FOCUSED_HINT: &str = "Click or press Esc to close";
pub const UNFOCUSED_HINT: &str = "Click to focus, then click or press Esc to close";

// A line of a box, at 100% scale.
#[derive(Clone)]
struct Line {
    text: String,
    style: TextStyle,
    kind: Kind,
}

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Identifier,
    Name,
    Details,
    Gap,
}

// One box: for one monitor, which shows one display, or more if monitors
// clone each other. It's a widget, drawn over its overlay's whole window.
#[derive(Clone)]
pub struct Identity {
    lines: Vec<Line>,
    // Displays at other scales get a box as big as it would look at 100%.
    scale: f32,
    // Whether one of the boxes has focus.
    focused: bool,
}

impl Identity {
    pub fn new(displays: &[&Display]) -> Identity {
        let mut lines = Vec::new();
        for (index, display) in displays.iter().enumerate() {
            if index > 0 {
                lines.push(Line {
                    text: String::new(),
                    style: BODY,
                    kind: Kind::Gap,
                });
            }
            lines.push(Line {
                text: display.identifier.clone(),
                style: IDENTIFIER,
                kind: Kind::Identifier,
            });
            lines.push(Line {
                text: display.name.clone(),
                style: TITLE,
                kind: Kind::Name,
            });
            let mut details = display.details();
            if display.primary {
                details += ", primary";
            }
            lines.push(Line {
                text: details,
                style: BODY_LARGE,
                kind: Kind::Details,
            });
        }
        lines.push(Line {
            text: String::new(),
            style: BODY,
            kind: Kind::Gap,
        });
        Identity {
            lines,
            scale: displays[0].dpi as f32 / 96.0,
            focused: true,
        }
    }

    // The box, saying whether one of the boxes has focus.
    pub fn focused(mut self, focused: bool) -> Identity {
        self.focused = focused;
        self
    }

    // The status line's icon and text.
    fn hint(&self) -> (Severity, &'static str) {
        if self.focused { (Severity::Success, FOCUSED_HINT) } else { (Severity::Warning, UNFOCUSED_HINT) }
    }

    // How wide the status line is, its icon and text, in physical pixels.
    fn hint_width(&self, text: &str, fonts: &mut Fonts) -> f32 {
        ((ICON + SPACE_S) * self.scale).round() + fonts.measure(text, self.style(BODY)).width.ceil()
    }

    fn style(&self, style: TextStyle) -> TextStyle {
        TextStyle::new(style.font, style.weight, style.size * self.scale, (style.line_height * self.scale).round())
    }

    fn padding(&self) -> f32 {
        (SPACE_XXXL * self.scale).round()
    }

    // The box's size, in physical pixels, which fits its status line whether
    // or not a box has focus, as an overlay's window keeps its size.
    pub fn size(&self, fonts: &mut Fonts) -> (f32, f32) {
        let mut width = MINIMUM_WIDTH * self.scale;
        let mut height = 0.0;
        for line in &self.lines {
            let style = self.style(line.style);
            width = width.max(fonts.measure(&line.text, style).width.ceil());
            height += style.line_height;
        }
        for hint in [FOCUSED_HINT, UNFOCUSED_HINT] {
            width = width.max(self.hint_width(hint, fonts));
        }
        height += self.style(BODY).line_height;
        let padding = self.padding();
        (width + padding * 2.0, height + padding * 2.0 + ACCENT_BAR * self.scale)
    }

}

impl<M: 'static> Widget<M> for Identity {
    fn kind(&self) -> &'static str {
        "Identity"
    }

    fn draw(&self, ctx: &mut Draw) {
        let bounds = ctx.rect();
        let mut painter = ctx.painter();
        let theme = painter.theme().clone();
        painter.fill(bounds, theme.acrylic);
        painter.fill(bounds.top_strip((ACCENT_BAR * self.scale).round()), theme.accent);
        painter.shape(bounds, Corners::ZERO, Color::TRANSPARENT, Some(Border::new(theme.surface_stroke_flyout, 1.0)));
        let padding = self.padding();
        let mut y = ACCENT_BAR * self.scale + padding;
        for line in &self.lines {
            let style = self.style(line.style);
            let color = match line.kind {
                Kind::Identifier | Kind::Name => theme.text_primary,
                Kind::Details => theme.text_secondary,
                Kind::Gap => theme.text_tertiary,
            };
            let row = Rect::new(0.0, y, bounds.width, style.line_height);
            painter.text_in(row, &line.text, style, color, Align::Center);
            y += style.line_height;
        }
        // The status line: its icon and text, centered together.
        let (severity, text) = self.hint();
        let style = self.style(BODY);
        let icon = (ICON * self.scale).round();
        let gap = (SPACE_S * self.scale).round();
        let width = icon + gap + painter.measure(text, style).width.ceil();
        let x = ((bounds.width - width) / 2.0).round();
        status_icon(&mut painter, Point::new(x + icon / 2.0, y + style.line_height / 2.0), severity, icon);
        let row = Rect::new(x + icon + gap, y, bounds.width - x - icon - gap, style.line_height);
        painter.text_in(row, text, style, theme.text_secondary, Align::Start);
    }
}
