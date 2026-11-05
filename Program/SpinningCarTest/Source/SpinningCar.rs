// The test, as a program that Library/UserInterface runs: where the car is,
// what S and F1 do, the card over the scene, and the scene, which fills the
// window under the card. The car keeps time while the window is minimized.

use graphics::{Frame, Gpu};
use json::Value;
use user_interface::{Program, Startup, Ui};

use crate::controls;
use crate::info_card::{self, Contents};
use crate::scene::{Scene, car_placement};
use crate::settings::SceneSettings;

pub const NAME: &str = "Spinning Car Test";
pub const DESCRIPTION: &str = "A car moving at 8 units and turning at 90 degrees per second";

pub struct SpinningCar {
    pub settings: SceneSettings,
    pub stationary: bool,
    // Where the car is along X, and how far it has turned.
    pub placement: (f32, f32),
    pub text_shown: bool,
    // Made once there's a GPU, before the first frame.
    scene: Option<Scene>,
}

impl SpinningCar {
    pub fn new(settings: SceneSettings) -> Self {
        Self {
            settings,
            stationary: false,
            placement: car_placement(0.0, false),
            text_shown: true,
            scene: None,
        }
    }

    // With the settings from Settings.json.
    pub fn start(startup: &mut Startup) -> Self {
        Self::new(SceneSettings::read(&mut startup.settings))
    }
}

impl Program for SpinningCar {
    fn title(&self) -> String {
        NAME.to_string()
    }

    fn update(&mut self, ui: &mut Ui) {
        if info_card::toggled(ui.input()) {
            self.text_shown = !self.text_shown;
        }
        if controls::toggles_stationary(ui.input()) {
            self.stationary = !self.stationary;
        }
        self.placement = car_placement(ui.time(), self.stationary);
        if self.text_shown {
            let car = if self.stationary { "Stationary" } else { "Moving" };
            info_card::draw(
                ui,
                &Contents {
                    title: NAME,
                    description: DESCRIPTION,
                    rows: &[("Car", car)],
                    keys: &[("S", "Stationary car")],
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
            Some(scene) => scene.draw(gpu, frame, self.placement),
            None => Ok(()),
        }
    }

    fn state(&self) -> Value {
        json::object([
            ("CarPosition", self.placement.0.into()),
            ("CarTurn", self.placement.1.into()),
            ("Stationary", self.stationary.into()),
            ("TextShown", self.text_shown.into()),
        ])
    }
}
