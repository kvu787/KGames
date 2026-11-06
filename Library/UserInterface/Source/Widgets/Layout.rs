// Laying a card's content out in fixed pixels, top to bottom: a Form gives
// rows for controls with their headers above them, as XAML's Header puts
// them, or labels beside them, and headings, captions and gaps between them.
// Its `bottom` is where the content ends, for a card that sizes itself.
//
// And a 3D view's area, which takes every mouse button without taking
// keyboard focus, so that the program's keys keep working while it's used.

use crate::core::ui::{Cursor, Response, Sense, Ui};
use crate::design::controls::TEXT_BOX_HEADER_GAP;
use crate::design::tokens::{BODY, BODY_STRONG, CAPTION, CARD_PADDING, CONTROL_HEIGHT, FORM_ROW_GAP, SPACE_S, SPACE_XS};
use crate::drawing::geometry::Rect;
use crate::widgets::widget::Widget;

pub struct Form {
    area: Rect,
    // Labels go beside controls, in a column this wide, or above them.
    label_width: Option<f32>,
    gap: f32,
    used: f32,
    // Whether everything taken so far fit in the area.
    fits: bool,
}

impl Form {
    // Headers above the controls.
    pub fn new(area: Rect) -> Self {
        Self {
            area,
            label_width: None,
            gap: FORM_ROW_GAP,
            used: area.y,
            fits: true,
        }
    }

    // Labels beside the controls, in a column `label_width` wide.
    pub fn beside(area: Rect, label_width: f32) -> Self {
        Self {
            label_width: Some(label_width),
            ..Self::new(area)
        }
    }

    // The gap between rows.
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    // A rectangle `height` tall for anything, and the gap after it.
    pub fn take(&mut self, height: f32) -> Rect {
        self.fits &= height <= self.area.height;
        let rect = self.area.cut_top(height, self.gap);
        self.used = rect.bottom();
        rect
    }

    // A control's row with its label, which it draws: above the control, or
    // beside it. Returns the control's rectangle, `height` tall.
    pub fn row(&mut self, ui: &mut Ui, label: &str, height: f32) -> Rect {
        let color = if ui.enabled() { ui.theme().text_primary } else { ui.theme().text_disabled };
        match self.label_width {
            Some(width) => {
                let row = self.take(height.max(CONTROL_HEIGHT));
                ui.text(Rect::new(row.x, row.y, width - SPACE_S, CONTROL_HEIGHT.min(row.height)), label, BODY, color);
                Rect::new(row.x + width, row.y, row.width - width, height)
            }
            None => {
                if !label.is_empty() {
                    let header = self.area.cut_top(BODY.line_height, TEXT_BOX_HEADER_GAP);
                    ui.text(header, label, BODY, color);
                }
                self.take(height)
            }
        }
    }

    // A control's row of the standard height.
    pub fn control(&mut self, ui: &mut Ui, label: &str) -> Rect {
        self.row(ui, label, CONTROL_HEIGHT)
    }

    // A heading in Body Strong, with a divider under it.
    pub fn heading(&mut self, ui: &mut Ui, text: &str) {
        let row = self.area.cut_top(BODY_STRONG.line_height + SPACE_XS + 1.0, self.gap);
        ui.heading(row.top_strip(BODY_STRONG.line_height), text);
        ui.divider(row.bottom_strip(1.0));
        self.used = row.bottom();
    }

    // A caption, wrapped, in the secondary color.
    pub fn caption(&mut self, ui: &mut Ui, text: &str) {
        let height = ui.paragraph_height(self.area.width, text, CAPTION);
        let rect = self.take(height);
        let color = ui.theme().text_secondary;
        ui.paragraph(rect, text, CAPTION, color);
    }

    // Body text, wrapped.
    pub fn paragraph(&mut self, ui: &mut Ui, text: &str) {
        let height = ui.paragraph_height(self.area.width, text, BODY);
        let rect = self.take(height);
        let color = ui.theme().text_primary;
        ui.paragraph(rect, text, BODY, color);
    }

    // N columns of equal width, `height` tall, like a row of buttons.
    pub fn columns<const N: usize>(&mut self, height: f32) -> [Rect; N] {
        self.take(height).columns::<N>(SPACE_S)
    }

    // N controls side by side, each with its header above it, as `row` puts
    // one, in columns of equal width with a gap of a card's padding between
    // them. Returns the controls' rectangles, `height` tall.
    pub fn headed_columns<const N: usize>(&mut self, ui: &mut Ui, labels: [&str; N], height: f32) -> [Rect; N] {
        let color = if ui.enabled() { ui.theme().text_primary } else { ui.theme().text_disabled };
        let headed = labels.iter().any(|label| !label.is_empty());
        let header_height = if headed { BODY.line_height + TEXT_BOX_HEADER_GAP } else { 0.0 };
        let area = self.take(header_height + height);
        let columns = area.columns::<N>(CARD_PADDING);
        for (column, label) in columns.iter().zip(labels) {
            if !label.is_empty() {
                ui.text(Rect::new(column.x, column.y, column.width, BODY.line_height), label, BODY, color);
            }
        }
        columns.map(|column| Rect::new(column.x, column.y + header_height, column.width, height))
    }

    pub fn space(&mut self, height: f32) {
        self.area.cut_top(height, 0.0);
    }

    // What's left.
    pub fn rest(&self) -> Rect {
        self.area
    }

    // Where the content ends.
    pub fn bottom(&self) -> f32 {
        self.used
    }

    // Whether everything taken fit in the area, for content that may not.
    pub fn fits(&self) -> bool {
        self.fits
    }

    pub fn width(&self) -> f32 {
        self.area.width
    }
}

// A 3D view's area. It takes presses of every mouse button and the wheel,
// but never keyboard focus, and a press in it takes focus from whatever had
// it, so that the program's keys work while it's used.
#[derive(Clone, Copy, Debug, Default)]
pub struct View {
    cursor: Option<Cursor>,
}

impl View {
    pub fn new() -> Self {
        Self::default()
    }

    // The pointer's look over the view.
    pub fn cursor(mut self, cursor: Cursor) -> Self {
        self.cursor = Some(cursor);
        self
    }
}

impl Widget for View {
    type Response = Response;

    fn show(self, ui: &mut Ui, name: &str, rect: Rect) -> Response {
        let response = ui.interact(name, rect, Sense::VIEW, "View");
        if response.pressed || response.secondary_pressed || response.middle_pressed {
            ui.clear_focus();
        }
        if let Some(cursor) = self.cursor
            && (response.hovered || response.held)
        {
            ui.set_cursor(cursor);
        }
        response
    }
}
