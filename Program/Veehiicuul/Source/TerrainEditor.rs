// The hexel terrain editor: paints the current scene's terrain, a hexel at a
// time or a brush of them, with the colors of its palette, under the track.
//
// | Tool  | What a press and a drag in the view do                              |
// | ----- | ------------------------------------------------------------------- |
// | Paint | Paints the hexels under the brush with the palette's color          |
// | Erase | Takes the paint off them, so that the background shows             |
// | Fill  | Paints the hexel and those joined to it that are its color, nearby  |
// | Pick  | Makes the hexel's color the palette's color                         |

use json::Value;
use user_interface::tokens::{CONTROL_HEIGHT, SPACE_S};
use user_interface::{Button, Icon, Rect, Ui};

use crate::editing::{self, Form, Overlay, SliderRange, Viewport, interface_color, subtitle};
use crate::files;
use crate::terrain::{PALETTE_SIZE, Terrain, disk};

pub const TOOLS: [&str; 4] = ["Paint", "Erase", "Fill", "Pick"];
const TOOL_ICONS: [Icon; 4] = [Icon::Brush, Icon::Erase, Icon::Fill, Icon::Eyedropper];
// How far a fill reaches from the hexel it starts at, in hexels.
const FILL_REACH: i32 = 60;
const SWATCH: f32 = CONTROL_HEIGHT;

pub struct TerrainEditor {
    pub tool: usize,
    // The palette's color that Paint and Fill use.
    pub color: u8,
    // The brush's radius, in hexels around the one under the pointer.
    pub radius: f64,
    // What's typed in the color field.
    color_text: String,
    // The hexel under the pointer, if it's on the view.
    pub hover: Option<(i32, i32)>,
}

impl Default for TerrainEditor {
    fn default() -> Self {
        TerrainEditor {
            tool: 0,
            color: 0,
            radius: 1.0,
            color_text: String::new(),
            hover: None,
        }
    }
}

impl TerrainEditor {
    // The view, where the tool paints, then the card, over it.
    pub fn update(&mut self, ui: &mut Ui, view: Rect, viewport: &mut Viewport, terrain: &mut Terrain) {
        let input = viewport.interact(ui, "Terrain/View", view);
        self.hover = input.ground.map(|point| terrain.cell_at(point));
        if let Some(cell) = self.hover
            && input.response.held
            && !input.panning
        {
            self.use_tool(terrain, cell, input.response.pressed);
        }
        // The hexels that the tool would use, outlined.
        if let Some(cell) = self.hover {
            let color = ui.theme().text_primary;
            let cells = if self.tool < 2 { disk(cell, self.radius as i32) } else { vec![cell] };
            let mut overlay = Overlay::new(ui, viewport.eye, view);
            for cell in cells {
                overlay.outline(&terrain.corners(cell), 1.5, color);
            }
        }
        self.card(ui, terrain);
    }

    fn use_tool(&mut self, terrain: &mut Terrain, cell: (i32, i32), pressed: bool) {
        let radius = self.radius as i32;
        let in_palette = (self.color as usize) < terrain.palette.len();
        match self.tool {
            0 if in_palette => {
                for cell in disk(cell, radius) {
                    terrain.set(cell, Some(self.color));
                }
            }
            1 => {
                for cell in disk(cell, radius) {
                    terrain.set(cell, None);
                }
            }
            2 if pressed && in_palette => {
                for cell in terrain.region(cell, FILL_REACH) {
                    terrain.set(cell, Some(self.color));
                }
            }
            3 => {
                if let Some(color) = terrain.get(cell) {
                    self.color = color;
                }
            }
            _ => {}
        }
    }

    fn card(&mut self, ui: &mut Ui, terrain: &mut Terrain) {
        let theme = ui.theme().clone();
        let page = ui.page();
        let card = ui.floating_card(editing::left_card(page), "Terrain");
        subtitle(ui, card.header, "Hexels painted on the ground");
        let mut form = Form::new(card.content);
        let tools: Vec<(Icon, &str)> = TOOL_ICONS.into_iter().zip(TOOLS).collect();
        form.tools(ui, "Terrain/Tool", &tools, &mut self.tool);
        let radius = SliderRange::new(0.0, 8.0, 1.0);
        form.slider(ui, "Terrain/Radius", "Brush radius", &mut self.radius, radius, |value| format!("{value:.0}"));

        form.heading(ui, "Palette");
        self.color = self.color.min(terrain.palette.len().saturating_sub(1) as u8);
        let columns = ((form.area.width + SPACE_S) / (SWATCH + SPACE_S)) as usize;
        let rows = terrain.palette.len().div_ceil(columns).max(1);
        let grid = form.next(rows as f32 * (SWATCH + SPACE_S) - SPACE_S);
        for (index, &color) in terrain.palette.iter().enumerate() {
            let (row, column) = (index / columns, index % columns);
            let area = Rect::new(
                grid.x + column as f32 * (SWATCH + SPACE_S),
                grid.y + row as f32 * (SWATCH + SPACE_S),
                SWATCH,
                SWATCH,
            );
            if ui.swatch(&format!("Palette/{index}"), area, interface_color(color), index == self.color as usize).clicked {
                self.color = index as u8;
            }
        }
        if let Some(color) = terrain.palette.get_mut(self.color as usize) {
            form.color(ui, "Terrain/Color", "Color", &mut self.color_text, color);
        }
        let add = Button::new("Add a color").icon(Icon::Add);
        let remove = Button::new("Remove it").icon(Icon::Delete);
        match form.buttons(ui, &[("Terrain/AddColor", add), ("Terrain/RemoveColor", remove)]) {
            Some(0) if terrain.palette.len() < PALETTE_SIZE => {
                let color = terrain.palette.get(self.color as usize).copied().unwrap_or([128, 128, 128]);
                terrain.palette.push(color);
                self.color = (terrain.palette.len() - 1) as u8;
            }
            Some(1) if terrain.palette.len() > 1 => terrain.remove_color(self.color),
            _ => {}
        }
        form.caption(ui, &format!("{} of {PALETTE_SIZE} colors. Removing one takes it off its hexels.", terrain.palette.len()));

        form.heading(ui, "Grid");
        let size = SliderRange::new(0.5, 10.0, 10.0);
        form.slider(ui, "Terrain/Size", "Hexel size", &mut terrain.size, size, |value| format!("{value:.1}"));
        let mut orientation = terrain.flat_top as usize;
        form.segmented(ui, "Terrain/Orientation", "Orientation", &["Pointy tops", "Flat tops"], &mut orientation);
        terrain.flat_top = orientation == 1;

        form.heading(ui, "Hexels");
        form.text(ui, &format!("Painted: {}", terrain.cells.len()), theme.text_primary);
        let hover = match self.hover {
            Some((q, r)) => format!("Under the pointer: hexel {q}, {r}"),
            None => "Under the pointer: nothing".to_string(),
        };
        form.text(ui, &hover, theme.text_primary);
        form.caption(ui, "A press or a drag in the view uses the tool.");
    }

    pub fn state(&self, terrain: &Terrain) -> Value {
        json::object([
            ("Tool", TOOLS[self.tool].into()),
            ("Color", (self.color as u32).into()),
            ("Radius", self.radius.into()),
            ("Painted", terrain.cells.len().into()),
            ("Palette", Value::Array(terrain.palette.iter().map(|&color| files::color_text(color).into()).collect())),
        ])
    }
}
