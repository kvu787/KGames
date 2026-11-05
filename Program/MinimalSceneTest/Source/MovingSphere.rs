// The test, as a program that Library/UserInterface runs: the sphere's
// motion, the card over the scene that F1 shows and hides, and the scene,
// which fills the window under the card.

use graphics::{Frame, Gpu};
use json::Value;
use user_interface::{Program, Startup, Ui};

use crate::info_card::{self, Contents};
use crate::scene::{Motion, Scene};
use crate::settings::SceneSettings;

pub const NAME: &str = "Minimal Scene Test";
pub const DESCRIPTION: &str = "A sphere moving at 8 units per second";

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
    fn title(&self) -> String {
        NAME.to_string()
    }

    fn update(&mut self, ui: &mut Ui) {
        if info_card::toggled(ui.input()) {
            self.text_shown = !self.text_shown;
        }
        self.motion.advance_frame(ui.delta());
        if self.text_shown {
            info_card::draw(
                ui,
                &Contents {
                    title: NAME,
                    description: DESCRIPTION,
                    rows: &[],
                    keys: &[],
                },
            );
        }
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
