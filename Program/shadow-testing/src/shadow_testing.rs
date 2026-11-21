// The program: each way's settings, what the sliders and Ctrl+R do to them,
// and the state for scripts. `cards::view` lays out a tile for each way, a
// card of sliders beside a stage with the same blue box in the middle, and
// `render` draws each way's box and shadow on its stage, before the
// interface draws over the rest of the frame.

use std::cell::Cell;

use graphics::{Frame, Gpu};
use user_interface::json::Value;
use user_interface::{Color, Context, Key, Modifiers, Program, Rect, Size, Tree, Ui};
use windows_core::Result;

use crate::approaches::{Approach, KINDS, Kind};
use crate::cards;
use crate::pipeline::ShadowPipeline;

pub const TITLE: &str = "ShadowTesting";
// The box that casts every way's shadow.
pub const BOX_COLOR: Color = Color::hex(0x0078D4);
pub const BOX_SIZE: Size = Size::new(200.0, 120.0);

// What the input means.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Message {
    // A way's setting moved, by its index, to the value.
    Set(Kind, usize, f64),
    // Every setting back to its default.
    Reset,
}

// Where a way's tile is: all of it, its stage, and the box on the stage.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Tile {
    pub area: Rect,
    pub stage: Rect,
    pub boxed: Rect,
}

pub struct ShadowTesting {
    // Every way, in KINDS' order.
    pub approaches: Vec<Approach>,
    // Where the last frame's tiles are, which render draws the boxes and
    // their shadows in. The view notes them, as the one place that lays out
    // the page.
    pub tiles: Cell<Option<[Tile; KINDS.len()]>>,
    // Made by prepare: each way's, in KINDS' order.
    pipelines: Vec<ShadowPipeline>,
}

impl Default for ShadowTesting {
    fn default() -> Self {
        Self {
            approaches: KINDS.iter().map(|kind| Approach::new(*kind)).collect(),
            tiles: Cell::new(None),
            pipelines: Vec::new(),
        }
    }
}

impl ShadowTesting {
    pub fn approach(&self, kind: Kind) -> &Approach {
        &self.approaches[index(kind)]
    }

    // Where the way's tile is in the last frame.
    pub fn tile(&self, kind: Kind) -> Tile {
        self.tiles.get().expect("the view lays out the tiles")[index(kind)]
    }
}

fn index(kind: Kind) -> usize {
    KINDS.iter().position(|each| *each == kind).unwrap()
}

impl Program for ShadowTesting {
    type Message = Message;

    fn title(&self) -> String {
        TITLE.to_string()
    }

    fn input(&self, ui: &Ui) -> Vec<Message> {
        if ui.shortcut(Modifiers::CONTROL, Key::R) { vec![Message::Reset] } else { Vec::new() }
    }

    fn update(&mut self, message: Message, _ui: &mut Context<Message>) {
        match message {
            Message::Set(kind, setting, value) => self.approaches[index(kind)].set(setting, value),
            Message::Reset => self.approaches = KINDS.iter().map(|kind| Approach::new(*kind)).collect(),
        }
    }

    fn view(&self, ui: &mut Ui) -> Tree<Message> {
        cards::view(self, ui)
    }

    fn prepare(&mut self, gpu: &mut Gpu) -> Result<()> {
        self.pipelines = KINDS
            .iter()
            .map(|kind| {
                let (vertex, pixel) = kind.shaders();
                ShadowPipeline::new(gpu, vertex, pixel)
            })
            .collect::<Result<_>>()?;
        Ok(())
    }

    // Draws each way's box and its shadow on its stage.
    fn render(&mut self, _gpu: &mut Gpu, frame: &Frame) -> Result<()> {
        let Some(tiles) = self.tiles.get() else {
            return Ok(());
        };
        for ((approach, pipeline), tile) in self.approaches.iter().zip(&self.pipelines).zip(tiles) {
            let kind = approach.kind;
            let area = kind.area(&approach.values, tile.boxed);
            pipeline.draw(frame, tile.stage, area, tile.boxed, BOX_COLOR, kind.params(&approach.values));
        }
        Ok(())
    }

    fn state(&self) -> Value {
        // To a ten-thousandth, so that an f32's noise doesn't show.
        let number = |value: f32| Value::from((value as f64 * 10_000.0).round() / 10_000.0);
        Value::Object(
            self.approaches
                .iter()
                .map(|approach| {
                    let settings = approach.kind.settings().iter().zip(&approach.values);
                    let values = settings.map(|(setting, value)| (setting.name.to_string(), number(*value))).collect();
                    (approach.kind.name().to_string(), Value::Object(values))
                })
                .collect(),
        )
    }
}
