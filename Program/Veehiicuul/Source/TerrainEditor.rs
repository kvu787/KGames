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
use user_interface::{Button, Canvas, Card, Color, Field, Form, Icon, Palette, Rect, Segmented, Tree, Ui};

use crate::editing::{self, GroundInput, Projection, SliderRange, Viewport, interface_color};
use crate::files;
use crate::program::Message;
use crate::terrain::{PALETTE_SIZE, Terrain, disk};

pub const TOOLS: [&str; 4] = ["Paint", "Erase", "Fill", "Pick"];
const TOOL_ICONS: [Icon; 4] = [Icon::Brush, Icon::Erase, Icon::Fill, Icon::Eyedropper];
// How far a fill reaches from the hexel it starts at, in hexels.
const FILL_REACH: i32 = 60;
// The width of the labels of the lines about the hexels.
const LABEL_WIDTH: f32 = 160.0;

// What the card's controls do.
#[derive(Clone, Debug)]
pub enum TerrainMsg {
    Tool(usize),
    Radius(f64),
    // The palette's swatch picked, and an edit to the picked color.
    PickColor(usize),
    Color(files::Color),
    AddColor,
    RemoveColor,
    Size(f64),
    Orientation(usize),
}

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
    // What the pointer in the view does: the tool, on the hexels under it.
    pub fn view_input(&mut self, input: GroundInput, terrain: &mut Terrain) {
        self.hover = input.ground.map(|point| terrain.cell_at(point));
        if let Some(cell) = self.hover
            && input.held
            && !input.panning
        {
            self.use_tool(terrain, cell, input.pressed);
        }
    }

    // A control on the card.
    pub fn apply(&mut self, message: TerrainMsg, terrain: &mut Terrain) {
        match message {
            TerrainMsg::Tool(tool) => self.tool = tool,
            TerrainMsg::Radius(radius) => self.radius = radius,
            TerrainMsg::PickColor(index) => self.color = index as u8,
            TerrainMsg::Color(color) => {
                if let Some(chosen) = terrain.palette.get_mut(self.color as usize) {
                    *chosen = color;
                }
            }
            TerrainMsg::AddColor => {
                if terrain.palette.len() < PALETTE_SIZE {
                    let color = terrain.palette.get(self.color as usize).copied().unwrap_or([128, 128, 128]);
                    terrain.palette.push(color);
                    self.color = (terrain.palette.len() - 1) as u8;
                }
            }
            TerrainMsg::RemoveColor => {
                if terrain.palette.len() > 1 {
                    terrain.remove_color(self.color);
                }
            }
            TerrainMsg::Size(size) => terrain.size = size,
            TerrainMsg::Orientation(orientation) => terrain.flat_top = orientation == 1,
        }
        self.color = self.color.min(terrain.palette.len().saturating_sub(1) as u8);
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

    // The overlay over the view, then the card, over both.
    pub fn view(&self, ui: &mut Ui, tree: &mut Tree<Message>, page: Rect, view: Rect, viewport: &Viewport, terrain: &Terrain) {
        // The hexels that the tool would use, outlined.
        if let Some(cell) = self.hover {
            let cells = if self.tool < 2 { disk(cell, self.radius as i32) } else { vec![cell] };
            let outlines: Vec<Vec<[f64; 2]>> = cells.into_iter().map(|cell| terrain.corners(cell).to_vec()).collect();
            let projection = Projection::new(viewport.eye, editing::window_size(ui));
            tree.draw(
                view,
                Canvas::new(move |painter, rect| {
                    let color = painter.theme().text_primary;
                    painter.push_clip(rect);
                    for outline in &outlines {
                        projection.outline(painter, outline, 1.5, color);
                    }
                    painter.pop_clip();
                }),
            );
        }
        self.card(ui, tree, page, terrain);
    }

    fn card(&self, ui: &mut Ui, tree: &mut Tree<Message>, page: Rect, terrain: &Terrain) {
        let card = Card::new("Terrain").subtitle("Hexels painted on the ground").floating();
        tree.sized_card(ui, None, editing::left_card(page), card, |ui, card, area| {
            let mut form = Form::new(area.content);
            let tools: Vec<(Icon, &str)> = TOOL_ICONS.into_iter().zip(TOOLS).collect();
            card.add("Terrain/Tool", form.take(CONTROL_HEIGHT), Segmented::new(self.tool, &tools).on_select(|tool| Message::Terrain(TerrainMsg::Tool(tool))));
            let radius = SliderRange::new(0.0, 8.0, 1.0, 0);
            editing::slider_row(&mut form, card, "Terrain/Radius", "Brush radius", self.radius, radius, |value| {
                Message::Terrain(TerrainMsg::Radius(value))
            });

            form.heading(card, "Palette");
            let picked = (self.color as usize).min(terrain.palette.len().saturating_sub(1));
            let colors: Vec<Color> = terrain.palette.iter().map(|&color| interface_color(color)).collect();
            let palette = Palette::new(Some(picked), &colors).on_pick(|index| Message::Terrain(TerrainMsg::PickColor(index)));
            let area = form.take(palette.height(form.rest().width));
            card.add("Palette", area, palette);
            let [color, add, remove] = form.columns::<3>(CONTROL_HEIGHT);
            if terrain.palette.get(picked).is_some() {
                editing::color_field(card, "Terrain/Color", color, terrain.palette[picked], |color| {
                    Message::Terrain(TerrainMsg::Color(color))
                });
            }
            card.add("Terrain/AddColor", add, Button::new("Add a color").icon(Icon::Add).on_press(Message::Terrain(TerrainMsg::AddColor)))
                .set_enabled(terrain.palette.len() < PALETTE_SIZE)
                .set_tooltip("Adds a copy of the color at the palette's end");
            card.add("Terrain/RemoveColor", remove, Button::new("Remove it").icon(Icon::Delete).on_press(Message::Terrain(TerrainMsg::RemoveColor)))
                .set_enabled(terrain.palette.len() > 1)
                .set_tooltip("Removes the color, and takes it off its hexels");
            form.caption(ui, card, &format!("{} of {PALETTE_SIZE} colors. Removing one takes it off its hexels.", terrain.palette.len()));

            form.heading(card, "Grid");
            let size = SliderRange::new(0.5, 10.0, 10.0, 1);
            editing::slider_row(&mut form, card, "Terrain/Size", "Hexel size", terrain.size, size, |value| {
                Message::Terrain(TerrainMsg::Size(value))
            });
            let orientation = terrain.flat_top as usize;
            let area = form.control(card, "Orientation");
            card.add("Terrain/Orientation", area, Segmented::new(orientation, &["Pointy tops", "Flat tops"]).on_select(|orientation| {
                Message::Terrain(TerrainMsg::Orientation(orientation))
            }));

            form.heading(card, "Hexels");
            card.draw(form.take(BODY.line_height), Field::new("Painted", LABEL_WIDTH, terrain.cells.len().to_string()));
            let hover = match self.hover {
                Some((q, r)) => format!("Hexel {q}, {r}"),
                None => "Nothing".to_string(),
            };
            card.draw(form.take(BODY.line_height), Field::new("Under the pointer", LABEL_WIDTH, hover));
            form.caption(ui, card, "A press or a drag in the view uses the tool.");
            form.bottom()
        });
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
