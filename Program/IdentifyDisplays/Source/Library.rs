// What IdentifyDisplays shows on each display: a box with the display's
// identifier, its model and its mode, as a library that
// Test/IdentifyDisplays tests without a window.

use displays::Display;
use user_interface::tokens::{BODY, BODY_LARGE, SPACE_XXXL, TITLE};
use user_interface::{Align, Color, Corners, Font, Fonts, Rect, Stroke, TextStyle, Ui, Weight};

// The identifier is in Cascadia Mono, whose 0 has a slash through it.
const IDENTIFIER: TextStyle = TextStyle::new(Font::Mono, Weight::Semibold, 112.0, 132.0);
const ACCENT_BAR: f32 = 4.0;
const MINIMUM_WIDTH: f32 = 360.0;

// A line of a box, at 100% scale.
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
    Hint,
    Gap,
}

// One box: for one monitor, which shows one display, or more if monitors
// clone each other.
pub struct Identity {
    lines: Vec<Line>,
    // Displays at other scales get a box as big as it would look at 100%.
    scale: f32,
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
        lines.push(Line {
            text: "Click or press Esc to close".to_string(),
            style: BODY,
            kind: Kind::Hint,
        });
        Identity {
            lines,
            scale: displays[0].dpi as f32 / 96.0,
        }
    }

    fn style(&self, style: TextStyle) -> TextStyle {
        TextStyle::new(style.font, style.weight, style.size * self.scale, (style.line_height * self.scale).round())
    }

    fn padding(&self) -> f32 {
        (SPACE_XXXL * self.scale).round()
    }

    // The box's size, in physical pixels.
    pub fn size(&self, fonts: &mut Fonts) -> (f32, f32) {
        let mut width = MINIMUM_WIDTH * self.scale;
        let mut height = 0.0;
        for line in &self.lines {
            let style = self.style(line.style);
            width = width.max(fonts.measure(&line.text, style).width.ceil());
            height += style.line_height;
        }
        let padding = self.padding();
        (width + padding * 2.0, height + padding * 2.0 + ACCENT_BAR * self.scale)
    }

    pub fn draw(&self, ui: &mut Ui) {
        let theme = ui.theme().clone();
        let bounds = ui.bounds();
        let mut painter = ui.painter();
        painter.fill(bounds, theme.flyout);
        painter.fill(bounds.top_strip((ACCENT_BAR * self.scale).round()), theme.accent);
        painter.shape(bounds, Corners::default(), Color::TRANSPARENT, Some(Stroke::new(theme.surface_stroke_flyout, 1.0)));
        let padding = self.padding();
        let mut y = ACCENT_BAR * self.scale + padding;
        for line in &self.lines {
            let style = self.style(line.style);
            let color = match line.kind {
                Kind::Identifier | Kind::Name => theme.text_primary,
                Kind::Details => theme.text_secondary,
                Kind::Hint | Kind::Gap => theme.text_tertiary,
            };
            let row = Rect::new(0.0, y, bounds.width, style.line_height);
            painter.text_in(row, &line.text, style, color, Align::Center);
            y += style.line_height;
        }
    }
}
