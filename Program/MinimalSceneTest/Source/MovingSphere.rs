// The test, as a program that Library/UserInterface runs: the sphere's
// motion, the card over the scene that F1 shows and hides, and the scene,
// which fills the window under the card.

use graphics::{Frame, Gpu};
use json::Value;
use user_interface::{Context, InfoCard, Program, Startup, Tree, Ui};

use crate::scene::{Motion, Scene};
use crate::settings::SceneSettings;

pub const NAME: &str = "Minimal Scene Test";
pub const DESCRIPTION: &str = "A sphere moving at 8 units per second";

// What a frame's input means: F1, and the frame's time passing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Message {
    ToggleText,
    // Seconds since the last frame, which move the sphere.
    Tick(f32),
}

pub struct MovingSphere {
    pub settings: SceneSettings,
    pub motion: Motion,
    pub text_shown: bool,
    // Made once there's a GPU, before the first frame.
    scene: Option<Scene>,
}

impl MovingSphere {
    pub fn new(settings: SceneSettings) -> Self {
        Self {
            settings,
            motion: Motion::default(),
            text_shown: true,
            scene: None,
        }
    }

    // With the settings from Settings.json.
    pub fn start(startup: &mut Startup) -> Self {
        Self::new(SceneSettings::read(&mut startup.settings))
    }
}

impl Program for MovingSphere {
    type Message = Message;

    fn title(&self) -> String {
        NAME.to_string()
    }

    fn input(&self, ui: &Ui) -> Vec<Message> {
        let mut messages = Vec::new();
        if InfoCard::toggled(ui.input()) {
            messages.push(Message::ToggleText);
        }
        messages.push(Message::Tick(ui.delta()));
        messages
    }

    fn update(&mut self, message: Message, _ui: &mut Context<Message>) {
        match message {
            Message::ToggleText => self.text_shown = !self.text_shown,
            Message::Tick(seconds) => self.motion.advance_frame(seconds),
        }
    }

    fn view(&self, ui: &mut Ui) -> Tree<Message> {
        let mut tree = Tree::new();
        if self.text_shown {
            tree.info_card(
                ui,
                &InfoCard {
                    title: NAME,
                    description: DESCRIPTION,
                    rows: &[],
                    keys: &[],
                },
            );
        }
        tree
    }

    fn prepare(&mut self, gpu: &mut Gpu) -> windows_core::Result<()> {
        self.scene = Some(Scene::new(gpu, &self.settings)?);
        Ok(())
    }

    fn render(&mut self, gpu: &mut Gpu, frame: &Frame) -> windows_core::Result<()> {
        match &mut self.scene {
            Some(scene) => scene.draw(gpu, frame, self.motion.position),
            None => Ok(()),
        }
    }

    fn state(&self) -> Value {
        json::object([
            ("SpherePosition", self.motion.position.into()),
            ("SphereDirection", self.motion.direction.into()),
            ("TextShown", self.text_shown.into()),
        ])
    }
}
