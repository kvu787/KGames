// The program: the shadows' settings, what the sliders' messages do to them,
// and the state for scripts. `cards::view` describes the page, and `render`
// draws the blue box and its shadow in the middle of the page's stage,
// before the interface draws over the rest of the frame.
//
// Each way of drawing a shadow that's being tried has a module of its own,
// with its settings and the pipeline that draws it: so far, `linear`.

use std::cell::Cell;

use graphics::{Frame, Gpu};
use user_interface::json::Value;
use user_interface::{Color, Context, Program, Rect, Size, Tree, Ui};
use windows_core::Result;

use crate::cards;
use crate::linear::{Linear, LinearPipeline, Setting};

pub const TITLE: &str = "ShadowTesting";
// The box that casts the shadow.
pub const BOX_COLOR: Color = Color::hex(0x0078D4);
pub const BOX_SIZE: Size = Size::new(640.0, 400.0);

// What the input on the sliders means.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Message {
    // A linear shadow's setting moved, to the value.
    Linear(Setting, f64),
}

#[derive(Default)]
pub struct ShadowTesting {
    pub linear: Linear,
    // Where the last frame's box is, which render draws it and its shadow
    // at. The view notes it, as the one place that lays out the page.
    pub box_rect: Cell<Option<Rect>>,
    // Made by prepare.
    pipeline: Option<LinearPipeline>,
}

impl Program for ShadowTesting {
    type Message = Message;

    fn title(&self) -> String {
        TITLE.to_string()
    }

    fn update(&mut self, message: Message, _ui: &mut Context<Message>) {
        match message {
            Message::Linear(setting, value) => self.linear.set(setting, value),
        }
    }

    fn view(&self, ui: &mut Ui) -> Tree<Message> {
        cards::view(self, ui)
    }

    fn prepare(&mut self, gpu: &mut Gpu) -> Result<()> {
        self.pipeline = Some(LinearPipeline::new(gpu)?);
        Ok(())
    }

    fn render(&mut self, _gpu: &mut Gpu, frame: &Frame) -> Result<()> {
        if let (Some(pipeline), Some(rect)) = (&self.pipeline, self.box_rect.get()) {
            pipeline.draw(frame, rect, BOX_COLOR, &self.linear);
        }
        Ok(())
    }

    fn state(&self) -> Value {
        // To a ten-thousandth, so that an f32's noise doesn't show.
        let number = |value: f32| Value::from((value as f64 * 10_000.0).round() / 10_000.0);
        let linear = Value::Object(vec![
            ("Length".to_string(), number(self.linear.length)),
            ("StartOpacity".to_string(), number(self.linear.start)),
            ("EndOpacity".to_string(), number(self.linear.end)),
        ]);
        Value::Object(vec![("Linear".to_string(), linear)])
    }
}
