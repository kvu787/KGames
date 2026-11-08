// A palette: swatches of colors in rounded squares, left to right and then
// in rows, of which one may be picked, which shows an accent ring around it.
// Each swatch is a target named "<name>/<index>", labeled with its color.

use crate::core::interaction::{Interaction, Part, Sense};
use crate::core::tree::{Dispatch, Draw, Parts, Widget};
use crate::design::tokens::{CONTROL_HEIGHT, RADIUS_CONTROL, RADIUS_SMALL, SPACE_S};
use crate::drawing::geometry::{Color, Rect};
use crate::drawing::painter::Border;
use crate::widgets::widget::{Reaction, react};

pub struct Palette<M> {
    picked: Option<usize>,
    colors: Vec<Color>,
    size: f32,
    gap: f32,
    on_pick: Option<Reaction<usize, M>>,
}

impl<M: Clone + 'static> Palette<M> {
    // Swatches a control tall, a small space apart.
    pub fn new(picked: Option<usize>, colors: &[Color]) -> Self {
        Self {
            picked,
            colors: colors.to_vec(),
            size: CONTROL_HEIGHT,
            gap: SPACE_S,
            on_pick: None,
        }
    }

    // What a click on a swatch sends, with its index.
    pub fn on_pick(mut self, reaction: impl Fn(usize) -> M + 'static) -> Self {
        self.on_pick = Some(react(reaction));
        self
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

    fn swatch(&self, rect: Rect, index: usize) -> Rect {
        let columns = self.columns(rect.width);
        let (row, column) = (index / columns, index % columns);
        Rect::new(rect.x + column as f32 * (self.size + self.gap), rect.y + row as f32 * (self.size + self.gap), self.size, self.size)
    }
}

impl<M: Clone + 'static> Widget<M> for Palette<M> {
    fn kind(&self) -> &'static str {
        "Palette"
    }

    fn label(&self) -> Option<String> {
        self.picked.and_then(|index| self.colors.get(index)).map(Color::to_hex)
    }

    fn sense(&self) -> Option<Sense> {
        Some(Sense::NONE)
    }

    fn parts(&self, ctx: &mut Parts) {
        let rect = ctx.rect();
        for (index, color) in self.colors.iter().enumerate() {
            ctx.part(Part::Index(index), &index.to_string(), self.swatch(rect, index), Sense::BUTTON, "Swatch", Some(color.to_hex()));
        }
    }

    fn interact(&self, interaction: &Interaction, ctx: &mut Dispatch<M>) {
        if let Some(Part::Index(index)) = interaction.clicked_part()
            && self.picked != Some(*index)
            && let Some(reaction) = &self.on_pick
        {
            ctx.send(reaction(*index));
        }
    }

    fn draw(&self, ctx: &mut Draw) {
        let rect = ctx.rect();
        let look = ctx.look().clone();
        for (index, color) in self.colors.iter().enumerate() {
            let part = Part::Index(index);
            let area = self.swatch(rect, index);
            let picked = self.picked == Some(index);
            {
                let mut painter = ctx.painter();
                let theme = painter.theme();
                // A picked swatch's color is inset, inside its ring.
                let inner = area.inset(if picked { 3.0 } else { 0.0 });
                painter.shape(inner, RADIUS_SMALL.max(RADIUS_CONTROL - 1.0), *color, Some(Border::new(theme.control_stroke_secondary, 1.0)));
                if picked {
                    painter.shape(area, RADIUS_CONTROL + 1.0, Color::TRANSPARENT, Some(Border::new(theme.accent, 2.0)));
                } else if look.hovered(&part) {
                    painter.shape(area, RADIUS_CONTROL, Color::TRANSPARENT, Some(Border::new(theme.control_strong_stroke, 1.0)));
                }
            }
            let focused = look.focused && look.focused_part.as_ref() == Some(&part);
            ctx.focus_visual_if(focused, area, RADIUS_CONTROL);
        }
    }
}
