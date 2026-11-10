// The test, as a program that Library/UserInterface runs: where the car is,
// what S and F1 do, the card over the scene, and the scene, which fills the
// window under the card. The car keeps time while the window is minimized.

use graphics::{Frame, Gpu};
use json::Value;
use user_interface::{Context, InfoCard, Program, Startup, Tree, Ui};

use crate::controls;
use crate::scene::{Scene, car_placement};
use crate::settings::SceneSettings;

pub const NAME: &str = "Spinning Car Test";
pub const DESCRIPTION: &str = "A car moving at 8 units and turning at 90 degrees per second";

// What a frame's input means: F1, S, and the frame's time, which places the
// car. The tick comes after the toggles, so the frame that presses S places
// the car with the new state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Message {
    ToggleText,
    ToggleStationary,
    // Seconds since the program started.
    Tick(f64),
}

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
    type Message = Message;

    fn title(&self) -> String {
        NAME.to_string()
    }

    fn input(&self, ui: &Ui) -> Vec<Message> {
        let mut messages = Vec::new();
        if InfoCard::toggled(ui.input()) {
            messages.push(Message::ToggleText);
        }
        if controls::toggles_stationary(ui.input()) {
            messages.push(Message::ToggleStationary);
        }
        messages.push(Message::Tick(ui.time()));
        messages
    }

    fn update(&mut self, message: Message, _ui: &mut Context<Message>) {
        match message {
            Message::ToggleText => self.text_shown = !self.text_shown,
            Message::ToggleStationary => self.stationary = !self.stationary,
            Message::Tick(seconds) => self.placement = car_placement(seconds, self.stationary),
        }
    }

    fn view(&self, ui: &mut Ui) -> Tree<Message> {
        let mut tree = Tree::new();
        if self.text_shown {
            let car = if self.stationary { "Stationary" } else { "Moving" };
            tree.info_card(
                ui,
                &InfoCard {
                    title: NAME,
                    description: DESCRIPTION,
                    rows: &[("Car", car)],
                    keys: &[("S", "Stationary car")],
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
