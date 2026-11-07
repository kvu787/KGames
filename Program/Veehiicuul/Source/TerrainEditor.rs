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
use user_interface::tokens::{BODY, CONTROL_HEIGHT, SPACE_S};
use user_interface::{Button, Card, Field, Form, Icon, Rect, Segmented, Swatch, Ui, UiEventKind};

use crate::editing::{self, Projection, Role, SliderRange, Viewport, interface_color};
use crate::files;
use crate::terrain::{PALETTE_SIZE, Terrain, disk};

pub const TOOLS: [&str; 4] = ["Paint", "Erase", "Fill", "Pick"];
const TOOL_ICONS: [Icon; 4] = [Icon::Brush, Icon::Erase, Icon::Fill, Icon::Eyedropper];
// How far a fill reaches from the hexel it starts at, in hexels.
const FILL_REACH: i32 = 60;
const SWATCH: f32 = CONTROL_HEIGHT;
// The width of the labels of the lines about the hexels.
const LABEL_WIDTH: f32 = 160.0;

const RADIUS: SliderRange = SliderRange::new(0.0, 8.0, 1.0, 0);
const SIZES: SliderRange = SliderRange::new(0.5, 10.0, 10.0, 1);

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
    // The palette's size shapes the card.
    pub fn fingerprint(&self, terrain: &Terrain) -> String {
        format!("palette {}", terrain.palette.len())
    }

    pub fn build(&self, ui: &mut Ui, view: Rect, terrain: &Terrain) {
        ui.node("Terrain/View", view, user_interface::View::new());
        editing::overlay_node(ui, "Terrain/Overlay", view);
        let page = ui.page();
        self.card(ui, page, terrain);
    }

    fn card(&self, ui: &mut Ui, page: Rect, terrain: &Terrain) {
        let card = Card::new("Terrain").subtitle("Hexels painted on the ground").floating();
        let area = card.area(ui, editing::left_card(page));
        let mut bottom = area.content.y;
        ui.group("TerrainCard", editing::left_card(page), Card::new("Terrain").subtitle("Hexels painted on the ground").floating(), |ui| {
            let mut form = Form::new("TerrainForm", area.content);
            let tools: Vec<(Icon, &str)> = TOOL_ICONS.into_iter().zip(TOOLS).collect();
            ui.node("Terrain/Tool", form.take(CONTROL_HEIGHT), Segmented::new(&tools, self.tool));
            editing::slider_row(&mut form, ui, "Terrain/Radius", "Brush radius", self.radius, RADIUS);

            form.heading(ui, "Palette");
            let color = self.color.min(terrain.palette.len().saturating_sub(1) as u8);
            let columns = ((form.width() + SPACE_S) / (SWATCH + SPACE_S)) as usize;
            let rows = terrain.palette.len().div_ceil(columns).max(1);
            let grid = form.take(rows as f32 * (SWATCH + SPACE_S) - SPACE_S);
            for (index, &swatch_color) in terrain.palette.iter().enumerate() {
                let (row, column) = (index / columns, index % columns);
                let swatch_area = Rect::new(
                    grid.x + column as f32 * (SWATCH + SPACE_S),
                    grid.y + row as f32 * (SWATCH + SPACE_S),
                    SWATCH,
                    SWATCH,
                );
                ui.node(&format!("Palette/{index}"), swatch_area, Swatch::new(interface_color(swatch_color), index == color as usize));
            }
            let [field, add, remove] = form.columns::<3>(CONTROL_HEIGHT);
            let chosen = terrain.palette.get(color as usize).copied().unwrap_or([128, 128, 128]);
            editing::color_field_node(ui, "Terrain/Color", field, chosen);
            ui.node("Terrain/AddColor", add, Button::new("Add a color").icon(Icon::Add));
            ui.set_enabled("Terrain/AddColor", terrain.palette.len() < PALETTE_SIZE);
            ui.node("Terrain/RemoveColor", remove, Button::new("Remove it").icon(Icon::Delete));
            ui.set_enabled("Terrain/RemoveColor", terrain.palette.len() > 1);
            ui.set_tooltip("Terrain/AddColor", "Adds a copy of the color at the palette's end");
            ui.set_tooltip("Terrain/RemoveColor", "Removes the color, and takes it off its hexels");
            form.caption(ui, &format!("{} of {PALETTE_SIZE} colors. Removing one takes it off its hexels.", terrain.palette.len()));

            form.heading(ui, "Grid");
            editing::slider_row(&mut form, ui, "Terrain/Size", "Hexel size", terrain.size, SIZES);
            let area = form.control(ui, "Orientation");
            ui.node("Terrain/Orientation", area, Segmented::new(&["Pointy tops", "Flat tops"], terrain.flat_top as usize));

            form.heading(ui, "Hexels");
            ui.node("Terrain/Painted", form.take(BODY.line_height), Field::new("Painted", LABEL_WIDTH, ""));
            ui.node("Terrain/Hover", form.take(BODY.line_height), Field::new("Under the pointer", LABEL_WIDTH, ""));
            form.caption(ui, "A press or a drag in the view uses the tool.");
            bottom = form.bottom();
        });
        let rect = Rect::new(page.x, page.y, editing::CARD_WIDTH, card.height_around(bottom - area.content.y));
        ui.set_rect("TerrainCard", rect);
    }

    // The view, where the tool paints, the frame's events, and what the card
    // shows.
    pub fn update(&mut self, ui: &mut Ui, _view: Rect, viewport: &mut Viewport, terrain: &mut Terrain) {
        let input = viewport.apply(ui, "Terrain/View");
        self.hover = input.ground.map(|point| terrain.cell_at(point));
        if let Some(cell) = self.hover
            && input.held
            && !input.panning
        {
            self.use_tool(terrain, cell, input.response.pressed);
        }
        self.events(ui, terrain);

        // The hexels that the tool would use, outlined.
        let mut shapes = Vec::new();
        if let Some(cell) = self.hover {
            let cells = if self.tool < 2 { disk(cell, self.radius as i32) } else { vec![cell] };
            let projection = Projection::new(ui, viewport.eye);
            for cell in cells {
                shapes.push(projection.outline(&terrain.corners(cell), 1.5, Role::Primary));
            }
        }
        editing::set_overlay(ui, "Terrain/Overlay", shapes);
        self.sync(ui, terrain);
    }

    fn events(&mut self, ui: &mut Ui, terrain: &mut Terrain) {
        let events: Vec<user_interface::UiEvent> = ui.events().to_vec();
        for event in events {
            match (event.source.as_str(), &event.kind) {
                ("Terrain/Tool", UiEventKind::Selected(index)) => self.tool = *index,
                ("Terrain/Radius", UiEventKind::Changed(value)) => self.radius = RADIUS.nearest(*value),
                (source, UiEventKind::Clicked) if source.starts_with("Palette/") => {
                    if let Ok(index) = source["Palette/".len()..].parse::<usize>()
                        && index < terrain.palette.len()
                    {
                        self.color = index as u8;
                    }
                }
                ("Terrain/Color", UiEventKind::ColorChanged(color)) => {
                    let [red, green, blue, _] = color.to_bytes();
                    if let Some(chosen) = terrain.palette.get_mut(self.color as usize) {
                        *chosen = [red, green, blue];
                    }
                }
                ("Terrain/AddColor", UiEventKind::Clicked) => {
                    if terrain.palette.len() < PALETTE_SIZE {
                        let color = terrain.palette.get(self.color as usize).copied().unwrap_or([128, 128, 128]);
                        terrain.palette.push(color);
                        self.color = (terrain.palette.len() - 1) as u8;
                    }
                }
                ("Terrain/RemoveColor", UiEventKind::Clicked) => {
                    if terrain.palette.len() > 1 {
                        terrain.remove_color(self.color);
                    }
                }
                ("Terrain/Size", UiEventKind::Changed(value)) => terrain.size = SIZES.nearest(*value),
                ("Terrain/Orientation", UiEventKind::Selected(index)) => terrain.flat_top = *index == 1,
                _ => {}
            }
        }
        self.color = self.color.min(terrain.palette.len().saturating_sub(1) as u8);
    }

    fn sync(&self, ui: &mut Ui, terrain: &Terrain) {
        ui.get_mut::<Segmented>("Terrain/Tool").set_selected(self.tool);
        editing::sync_slider(ui, "Terrain/Radius", self.radius);
        for (index, color) in terrain.palette.iter().enumerate() {
            let name = format!("Palette/{index}");
            if ui.has_node(&name) {
                let swatch = ui.get_mut::<Swatch>(&name);
                swatch.set_selected(index == self.color as usize);
                swatch.set_color(interface_color(*color));
            }
        }
        if let Some(chosen) = terrain.palette.get(self.color as usize) {
            editing::sync_color_field(ui, "Terrain/Color", *chosen);
        }
        ui.set_enabled("Terrain/AddColor", terrain.palette.len() < PALETTE_SIZE);
        ui.set_enabled("Terrain/RemoveColor", terrain.palette.len() > 1);
        editing::sync_slider(ui, "Terrain/Size", terrain.size);
        ui.get_mut::<Segmented>("Terrain/Orientation").set_selected(terrain.flat_top as usize);
        ui.get_mut::<Field>("Terrain/Painted").set_value(&terrain.cells.len().to_string());
        let hover = match self.hover {
            Some((q, r)) => format!("Hexel {q}, {r}"),
            None => "Nothing".to_string(),
        };
        ui.get_mut::<Field>("Terrain/Hover").set_value(&hover);
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
