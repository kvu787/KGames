// The demo, as a program that Library/UserInterface runs: the editor, its
// cards, and the sphere's preview, which render draws into the preview
// card's view before the interface draws over the rest of the frame.

use std::cell::{Cell, RefCell};
use std::path::PathBuf;

use graphics::{Frame, Gpu};
use json::Value;
use simple_paint::Parameters;
use user_interface::{Color, Context, Program, Rect, Severity, Startup, Tree, Ui};
use windows_core::Result;

use crate::cards;
use crate::editor::{self, Editor, SETTINGS};
use crate::preview::Preview;

pub const TITLE: &str = "SimplePaint Demo";

// What the input on the cards means.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Message {
    // A key's command: select, nudge or save.
    Command(editor::Command),
    Reset,
    Save,
    // A press on a setting's row, by the setting's index.
    Select(usize),
    // A setting's slider moved, dragged or by the wheel.
    Set(usize, f64),
    // A press took hold of a setting's slider, or let go of it.
    Drag(usize, bool),
}

pub struct Demo {
    pub editor: Editor,
    // Where Ctrl+S and Save write the material: the demo's folder, or a
    // script's output folder.
    pub paint_path: PathBuf,
    // The cards whose content didn't fit in them in the last frame, which
    // the tests check is none of them. The view notes it, as the one place
    // that lays the cards out.
    pub overflow: RefCell<Vec<&'static str>>,
    // Where the last frame's preview card has its view, which render fills
    // with the sphere, and the card's color, opaque, around the sphere. The
    // view notes them, as the one place that knows the layout and the theme.
    pub view: Cell<Option<Rect>>,
    pub background: Cell<Color>,
    // Made by prepare.
    preview: Option<Preview>,
}

impl Demo {
    pub fn new(parameters: Parameters, paint_path: PathBuf) -> Self {
        Self {
            editor: Editor::new(parameters),
            paint_path,
            overflow: RefCell::new(Vec::new()),
            view: Cell::new(None),
            background: Cell::new(Color::WHITE),
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
    type Message = Message;

    fn title(&self) -> String {
        TITLE.to_string()
    }

    // Each key pressed this frame, in order, with its repeats, unless a
    // widget with focus takes it, like a button takes Space.
    fn input(&self, ui: &Ui) -> Vec<Message> {
        let modifiers = ui.input().modifiers();
        let mut messages = Vec::new();
        for &(key, repeat) in ui.input().keys_pressed() {
            if ui.key_pressed(key)
                && let Some(command) = editor::command(key, repeat, modifiers)
            {
                messages.push(Message::Command(command));
            }
        }
        messages
    }

    fn update(&mut self, message: Message, _ui: &mut Context<Message>) {
        match message {
            Message::Command(command) => self.editor.apply(command, &self.paint_path),
            Message::Reset => self.editor.reset(),
            Message::Save => self.editor.save(&self.paint_path),
            Message::Select(index) => self.editor.selected = index,
            Message::Set(index, value) => {
                self.editor.selected = index;
                self.editor.set(index, SETTINGS[index].nearest(value));
            }
            Message::Drag(index, held) => {
                if held {
                    self.editor.selected = index;
                    self.editor.dragging = Some(index);
                } else {
                    self.editor.dragging = None;
                }
            }
        }
    }

    fn view(&self, ui: &mut Ui) -> Tree<Message> {
        cards::view(self, ui)
    }

    fn prepare(&mut self, gpu: &mut Gpu) -> Result<()> {
        self.preview = Some(unsafe { Preview::new(gpu)? });
        Ok(())
    }

    fn render(&mut self, gpu: &mut Gpu, frame: &Frame) -> Result<()> {
        if let (Some(preview), Some(view)) = (&mut self.preview, self.view.get()) {
            unsafe { preview.render(gpu, frame, &self.editor.material, self.background.get(), view)? };
        }
        Ok(())
    }

    fn state(&self) -> Value {
        self.editor.state()
    }
}
