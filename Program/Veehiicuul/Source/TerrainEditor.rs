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
use user_interface::tokens::{BODY, CONTROL_HEIGHT};
use user_interface::{Button, Card, Color, Form, Icon, Palette, Rect, Segmented, Ui};

use crate::editing::{self, Projection, SliderRange, Viewport, interface_color};
use crate::files;
use crate::terrain::{PALETTE_SIZE, Terrain, disk};

pub const TOOLS: [&str; 4] = ["Paint", "Erase", "Fill", "Pick"];
const TOOL_ICONS: [Icon; 4] = [Icon::Brush, Icon::Erase, Icon::Fill, Icon::Eyedropper];
// How far a fill reaches from the hexel it starts at, in hexels.
const FILL_REACH: i32 = 60;
// The width of the labels of the lines about the hexels.
const LABEL_WIDTH: f32 = 160.0;

pub struct TerrainEditor {
    pub tool: usize,
    // The palette's color that Paint and Fill use.
    pub color: u8,
    // The brush's radius, in hexels around the one under the pointer.
    pub radius: f64,
    // The hexel under the pointer, if it's on the view.
    pub hover: Option<(i32, i32)>,
}

impl Default for TerrainEditor {
    fn default() -> Self {
        TerrainEditor {
            tool: 0,
            color: 0,
            radius: 1.0,
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
            && input.held
            && !input.panning
        {
            self.use_tool(terrain, cell, input.response.pressed);
        }
        // The hexels that the tool would use, outlined.
        if let Some(cell) = self.hover {
            let cells = if self.tool < 2 { disk(cell, self.radius as i32) } else { vec![cell] };
            let projection = Projection::new(ui, viewport.eye);
            let mut painter = ui.painter();
            let color = painter.theme().text_primary;
            painter.push_clip(view);
            for cell in cells {
                projection.outline(&mut painter, &terrain.corners(cell), 1.5, color);
            }
            painter.pop_clip();
        }
        let page = ui.page();
        self.card(ui, page, terrain);
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

    fn card(&mut self, ui: &mut Ui, page: Rect, terrain: &mut Terrain) {
        let open = ui.begin_card(editing::left_card(page), Card::new("Terrain").subtitle("Hexels painted on the ground").floating());
        let mut form = Form::new(open.area.content);
        let tools: Vec<(Icon, &str)> = TOOL_ICONS.into_iter().zip(TOOLS).collect();
        ui.add("Terrain/Tool", form.take(CONTROL_HEIGHT), Segmented::new(&mut self.tool, &tools));
        let radius = SliderRange::new(0.0, 8.0, 1.0, 0);
        editing::slider_row(&mut form, ui, "Terrain/Radius", "Brush radius", &mut self.radius, radius);

        form.heading(ui, "Palette");
        self.color = self.color.min(terrain.palette.len().saturating_sub(1) as u8);
        let colors: Vec<Color> = terrain.palette.iter().map(|&color| interface_color(color)).collect();
        let mut picked = Some(self.color as usize);
        let palette = Palette::new(&mut picked, &colors);
        let area = form.take(palette.height(form.width()));
        ui.add("Palette", area, palette);
        if let Some(index) = picked {
            self.color = index as u8;
        }
        let [color, add, remove] = form.columns::<3>(CONTROL_HEIGHT);
        if let Some(chosen) = terrain.palette.get_mut(self.color as usize) {
            editing::color_field(ui, "Terrain/Color", color, chosen);
        }
        let add = ui.add_enabled(terrain.palette.len() < PALETTE_SIZE, "Terrain/AddColor", add, Button::new("Add a color").icon(Icon::Add));
        let remove = ui.add_enabled(terrain.palette.len() > 1, "Terrain/RemoveColor", remove, Button::new("Remove it").icon(Icon::Delete));
        ui.tooltip(&add, "Adds a copy of the color at the palette's end");
        ui.tooltip(&remove, "Removes the color, and takes it off its hexels");
        if add.clicked {
            let color = terrain.palette.get(self.color as usize).copied().unwrap_or([128, 128, 128]);
            terrain.palette.push(color);
            self.color = (terrain.palette.len() - 1) as u8;
        }
        if remove.clicked {
            terrain.remove_color(self.color);
        }
        form.caption(ui, &format!("{} of {PALETTE_SIZE} colors. Removing one takes it off its hexels.", terrain.palette.len()));

        form.heading(ui, "Grid");
        let size = SliderRange::new(0.5, 10.0, 10.0, 1);
        editing::slider_row(&mut form, ui, "Terrain/Size", "Hexel size", &mut terrain.size, size);
        let mut orientation = terrain.flat_top as usize;
        let area = form.control(ui, "Orientation");
        ui.add("Terrain/Orientation", area, Segmented::new(&mut orientation, &["Pointy tops", "Flat tops"]));
        terrain.flat_top = orientation == 1;

        form.heading(ui, "Hexels");
        ui.field(form.take(BODY.line_height), "Painted", LABEL_WIDTH, &terrain.cells.len().to_string());
        let hover = match self.hover {
            Some((q, r)) => format!("Hexel {q}, {r}"),
            None => "Nothing".to_string(),
        };
        ui.field(form.take(BODY.line_height), "Under the pointer", LABEL_WIDTH, &hover);
        form.caption(ui, "A press or a drag in the view uses the tool.");
        ui.end_card(open, form.bottom());
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
