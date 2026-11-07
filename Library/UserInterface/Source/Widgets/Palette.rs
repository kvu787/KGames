// A palette: swatches of colors in rounded squares, left to right and then
// in rows, of which one may be picked, which shows an accent ring around it.
// Each swatch is a target named "<name>/<index>", labeled with its color.
//
// It reads every swatch's input before it draws any, so that a click moves
// the ring on the frame it's made, from every swatch at once.

use crate::core::interaction::{Response, Sense, id};
use crate::core::ui::Ui;
use crate::design::tokens::{CONTROL_HEIGHT, RADIUS_CONTROL, RADIUS_SMALL, SPACE_S};
use crate::drawing::geometry::{Color, Rect};
use crate::drawing::painter::Border;
use crate::widgets::widget::Widget;

pub struct Palette<'a, 'v> {
    picked: &'v mut Option<usize>,
    colors: &'a [Color],
    size: f32,
    gap: f32,
}

impl<'a, 'v> Palette<'a, 'v> {
    // Swatches a control tall, a small space apart.
    pub fn new(picked: &'v mut Option<usize>, colors: &'a [Color]) -> Self {
        Self {
            picked,
            colors,
            size: CONTROL_HEIGHT,
            gap: SPACE_S,
        }
    }

    // Each swatch's side.
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    // The space between swatches, across and down.
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    // How tall its rows of swatches are in the width.
    pub fn height(&self, width: f32) -> f32 {
        let rows = self.colors.len().div_ceil(self.columns(width)).max(1) as f32;
        rows * self.size + (rows - 1.0) * self.gap
    }

    // How many swatches a row of the width has.
    fn columns(&self, width: f32) -> usize {
        (((width + self.gap) / (self.size + self.gap)) as usize).max(1)
    }
}

impl Widget for Palette<'_, '_> {
    // Its own: hovered or clicked on any swatch, and changed when the
    // picked one changes.
    type Response = Response;

    fn show(self, ui: &mut Ui, name: &str, rect: Rect) -> Response {
        let mut response = Response {
            id: id(name),
            rect,
            enabled: ui.enabled(),
            ..Response::default()
        };
        let columns = self.columns(rect.width);
        let mut swatches = Vec::with_capacity(self.colors.len());
        for (index, color) in self.colors.iter().enumerate() {
            let (row, column) = (index / columns, index % columns);
            let area = Rect::new(rect.x + column as f32 * (self.size + self.gap), rect.y + row as f32 * (self.size + self.gap), self.size, self.size);
            let swatch_name = format!("{name}/{index}");
            let swatch = ui.interact(&swatch_name, area, Sense::BUTTON, "Swatch");
            ui.set_label(&swatch_name, &color.to_hex());
            if swatch.clicked && *self.picked != Some(index) {
                *self.picked = Some(index);
                response.changed = true;
            }
            response.hovered |= swatch.hovered;
            response.clicked |= swatch.clicked;
            swatches.push((area, swatch));
        }
        for (index, (color, (area, swatch))) in self.colors.iter().zip(swatches).enumerate() {
            let picked = *self.picked == Some(index);
            let mut painter = ui.painter();
            let theme = painter.theme();
            // A picked swatch's color is inset, inside its ring.
            let inner = area.inset(if picked { 3.0 } else { 0.0 });
            painter.shape(inner, RADIUS_SMALL.max(RADIUS_CONTROL - 1.0), *color, Some(Border::new(theme.control_stroke_secondary, 1.0)));
            if picked {
                painter.shape(area, RADIUS_CONTROL + 1.0, Color::TRANSPARENT, Some(Border::new(theme.accent, 2.0)));
            } else if swatch.hovered {
                painter.shape(area, RADIUS_CONTROL, Color::TRANSPARENT, Some(Border::new(theme.control_strong_stroke, 1.0)));
            }
            ui.focus_visual_if(swatch.focused, area, RADIUS_CONTROL);
        }
        response
    }
}
