// The demo, as a program that Library/UserInterface runs: the editor, its
// cards, and the sphere's preview, which render draws into the preview
// card's view before the interface draws over the rest of the frame.

use std::path::PathBuf;

use graphics::{Frame, Gpu};
use json::Value;
use simple_paint::Parameters;
use user_interface::{Color, Program, Rect, Severity, Startup, Ui};
use windows_core::Result;

use crate::cards;
use crate::editor::{self, Editor};
use crate::preview::Preview;

pub const TITLE: &str = "SimplePaint Demo";

pub struct Demo {
    pub editor: Editor,
    // Where Ctrl+S and Save write the material: the demo's folder, or a
    // script's output folder.
    pub paint_path: PathBuf,
    // The cards whose content didn't fit in them in the last frame, which
    // the tests check is none of them.
    pub overflow: Vec<&'static str>,
    // Where the last frame's preview card has its view, which render fills
    // with the sphere, and the card's color there.
    pub view: Option<Rect>,
    pub background: Color,
    // Made by prepare.
    preview: Option<Preview>,
}

impl Demo {
    pub fn new(parameters: Parameters, paint_path: PathBuf) -> Self {
        Self {
            editor: Editor::new(parameters),
            paint_path,
            overflow: Vec::new(),
            view: None,
            background: Color::WHITE,
            preview: None,
        }
    }

    // The demo, with the material in Paint.json in its data folder, if it's
    // there, which a notice says what's wrong with if it can't be used. A
    // scripted run starts from SimplePaint's default material, so that its
    // scripts start the same every time.
    pub fn start(startup: &mut Startup) -> Self {
        let paint_path = startup.data_folder.join("Paint.json");
        let parameters = if startup.scripted {
            Parameters::default()
        } else {
            let mut notices = Vec::new();
            let parameters = editor::load(&paint_path, &mut notices);
            for notice in notices {
                startup.notify(Severity::Warning, "Paint.json", &notice);
            }
            parameters
        };
        Demo::new(parameters, paint_path)
    }
}

impl Program for Demo {
    fn title(&self) -> String {
        TITLE.to_string()
    }

    fn update(&mut self, ui: &mut Ui) {
        cards::draw(self, ui);
    }

    fn prepare(&mut self, gpu: &mut Gpu) -> Result<()> {
        self.preview = Some(unsafe { Preview::new(gpu)? });
        Ok(())
    }

    fn render(&mut self, gpu: &mut Gpu, frame: &Frame) -> Result<()> {
        if let (Some(preview), Some(view)) = (&mut self.preview, self.view) {
            unsafe { preview.render(gpu, frame, &self.editor.material, self.background, view)? };
        }
        Ok(())
    }

    fn state(&self) -> Value {
        self.editor.state()
    }
}
