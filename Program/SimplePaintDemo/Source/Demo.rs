// The demo, as a program that Library/UserInterface runs: the editor, its
// panels, and the sphere's preview, which is copied over the preview panel
// after the panels are drawn.

use std::path::PathBuf;

use graphics::Renderer;
use json::Value;
use user_interface::{Frame, Program, Setup};
use windows_core::Result;

use crate::NAME;
use crate::editor::{self, Editor};
use crate::panels::{self, Cache};
use crate::preview::{IMAGE_SIZE, Preview};

pub struct Demo {
    editor: Editor,
    // Where Ctrl+S and Save write the material: the demo's folder, or a
    // script's output folder.
    paint_path: PathBuf,
    preview: Preview,
    cache: Cache,
    // Where the last frame's preview panel has the sphere's image.
    image: Option<[u32; 2]>,
}

impl Program for Demo {
    type Settings = ();
    const NAME: &'static str = NAME;

    // Loads Paint.json, if it's there. Problems with it go in the notices.
    fn new(setup: Setup<'_, ()>) -> Result<Self> {
        let paint_path = setup.arguments.file(setup.folder, "Paint.json");
        let editor = Editor::new(editor::load(&paint_path, setup.notices));
        Ok(Demo {
            editor,
            paint_path,
            preview: unsafe { Preview::new(setup.renderer)? },
            cache: Cache::default(),
            image: None,
        })
    }

    fn frame(&mut self, frame: &mut Frame) -> Result<()> {
        let layout = panels::draw(frame, &mut self.cache, &mut self.editor, &self.paint_path);
        self.image = layout.image;
        Ok(())
    }

    // The sphere goes over the preview panel's middle, if it fits.
    unsafe fn draw_over(&mut self, renderer: &Renderer) -> Result<()> {
        if let Some([x, y]) = self.image
            && x + IMAGE_SIZE <= renderer.width
            && y + IMAGE_SIZE <= renderer.height
        {
            unsafe { self.preview.draw(renderer, &self.editor.material, [x, y]) };
        }
        Ok(())
    }

    fn state(&self) -> Value {
        self.editor.state()
    }
}
