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
use user_interface::theme::{ACCENT, DIM, TEXT, TITLE};
use user_interface::{Color, Frame, Rect, Sense, SliderRange, Stack};

use crate::editing::{self, Viewport, heading, label, slider, swatch_rect};
use crate::terrain::{PALETTE_SIZE, Terrain, disk};

pub const TOOLS: [&str; 4] = ["Paint", "Erase", "Fill", "Pick"];
// How far a fill reaches from the hexel it starts at, in hexels.
const FILL_REACH: i32 = 60;
const SWATCH: f32 = 30.0;

pub struct TerrainEditor {
    pub tool: usize,
    // The palette's color that Paint and Fill use.
    pub color: u8,
    // The brush's radius, in hexels around the one under the cursor.
    pub radius: f64,
    // What's typed in the color field.
    color_text: String,
    // The hexel under the cursor, if it's over the view.
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
    pub fn frame(&mut self, frame: &mut Frame, viewport: &mut Viewport, terrain: &mut Terrain) {
        let inner = frame.panel(editing::panel_area(frame), "Terrain", "paint hexels on the ground");
        let mut stack = Stack::new(inner, 8.0);
        let focusable = frame.set_focusable(false);
        let area = stack.next(frame.control_height());
        frame.tabs("Terrain/Tool", area, &TOOLS, &mut self.tool);
        slider(
            frame,
            &mut stack,
            "Terrain/Radius",
            "Brush radius",
            &mut self.radius,
            &SliderRange::new(0.0, 8.0, 1.0),
            |value| format!("{value:.0}"),
        );

        heading(frame, &mut stack, "Palette");
        self.color = self.color.min(terrain.palette.len().saturating_sub(1) as u8);
        let columns = ((inner.width + 6.0) / (SWATCH + 6.0)) as usize;
        let rows = terrain.palette.len().div_ceil(columns).max(1);
        let grid = stack.next(rows as f32 * (SWATCH + 6.0));
        for (index, &color) in terrain.palette.iter().enumerate() {
            let (row, column) = (index / columns, index % columns);
            let area = Rect::new(grid.x + column as f32 * (SWATCH + 6.0), grid.y + row as f32 * (SWATCH + 6.0), SWATCH, SWATCH);
            let response = frame.interact_labeled(&format!("Palette/{index}"), "Swatch", &crate::files::color_text(color), area, Sense::BUTTON);
            if response.clicked {
                self.color = index as u8;
            }
            swatch_rect(frame, area, color);
            if index == self.color as usize {
                frame.canvas.rect_outline(area.x - 3.0, area.y - 3.0, area.width + 6.0, area.height + 6.0, 2.0, ACCENT);
            }
        }
        frame.set_focusable(focusable);
        if let Some(color) = terrain.palette.get_mut(self.color as usize) {
            editing::color_field(frame, &mut stack, "Terrain/Color", "Color", &mut self.color_text, color);
        }
        let focusable = frame.set_focusable(false);
        match editing::buttons(frame, &mut stack, &[("Terrain/AddColor", "Add a color"), ("Terrain/RemoveColor", "Remove it")]) {
            Some(0) if terrain.palette.len() < PALETTE_SIZE => {
                let color = terrain.palette.get(self.color as usize).copied().unwrap_or([128, 128, 128]);
                terrain.palette.push(color);
                self.color = (terrain.palette.len() - 1) as u8;
            }
            Some(1) if terrain.palette.len() > 1 => terrain.remove_color(self.color),
            _ => {}
        }
        label(frame, &mut stack, "Removing a color takes it off its hexels.", DIM);

        heading(frame, &mut stack, "Grid");
        slider(
            frame,
            &mut stack,
            "Terrain/Size",
            "Hexel size",
            &mut terrain.size,
            &SliderRange::new(0.5, 10.0, 10.0),
            |value| format!("{value:.1}"),
        );
        let area = stack.next(frame.control_height());
        let mut orientation = terrain.flat_top as usize;
        frame.tabs("Terrain/Orientation", area, &["Pointy tops", "Flat tops"], &mut orientation);
        terrain.flat_top = orientation == 1;
        frame.set_focusable(focusable);

        heading(frame, &mut stack, "Help");
        for line in [
            "A press or a drag in the view uses the tool.",
            "Space and a drag pan, and the wheel zooms.",
            "Ctrl+Z undoes, and Ctrl+S saves.",
        ] {
            label(frame, &mut stack, line, DIM);
        }
        stack.skip(8.0);
        label(frame, &mut stack, &format!("Painted hexels: {}", terrain.cells.len()), TEXT);
        let hover = match self.hover {
            Some((q, r)) => format!("Under the cursor: hexel {q}, {r}"),
            None => "Under the cursor: nothing".to_string(),
        };
        label(frame, &mut stack, &hover, TEXT);

        let input = viewport.interact(frame, "Terrain/View");
        self.hover = input.ground.map(|point| terrain.cell_at(point));
        // A press and its release can come in the same frame, as a click.
        if let Some(cell) = self.hover
            && (input.response.held || input.response.pressed)
            && !input.panning
        {
            let radius = self.radius as i32;
            match self.tool {
                0 => {
                    if (self.color as usize) < terrain.palette.len() {
                        for cell in disk(cell, radius) {
                            terrain.set(cell, Some(self.color));
                        }
                    }
                }
                1 => {
                    for cell in disk(cell, radius) {
                        terrain.set(cell, None);
                    }
                }
                2 if input.response.pressed && (self.color as usize) < terrain.palette.len() => {
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
        // The brush's hexels, outlined.
        if let Some(cell) = self.hover {
            let cells = if self.tool < 2 { disk(cell, self.radius as i32) } else { vec![cell] };
            for cell in cells {
                editing::outline(frame, viewport, &terrain.corners(cell), 1.5, Color::rgb(0x14171C));
            }
        }
        let _ = TITLE;
    }

    pub fn state(&self, terrain: &Terrain) -> Value {
        json::object([
            ("Tool", TOOLS[self.tool].into()),
            ("Color", (self.color as u32).into()),
            ("Radius", self.radius.into()),
            ("Painted", terrain.cells.len().into()),
            ("Palette", Value::Array(terrain.palette.iter().map(|&color| crate::files::color_text(color).into()).collect())),
        ])
    }
}
