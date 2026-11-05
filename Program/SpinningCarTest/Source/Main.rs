// No console window in any build.
#![windows_subsystem = "windows"]

use std::path::Path;

use graphics::Renderer;
use json::Value;
use scene_test::Line;
use spinning_car_test::NAME;
use spinning_car_test::controls;
use spinning_car_test::scene::{self, Scene};
use spinning_car_test::settings::SceneSettings;
use user_interface::theme::{DIM, TEXT};
use windows_core::Result;

// The scene, with where the car is, which Library/SceneTest runs. The car
// keeps time while the window is minimized.
struct SpinningCar {
    scene: Scene,
    stationary: bool,
    // Where the car is along X, and how far it has turned.
    placement: (f32, f32),
}

impl scene_test::Scene for SpinningCar {
    type Settings = SceneSettings;
    const NAME: &'static str = NAME;

    unsafe fn new(renderer: &mut Renderer, settings: &SceneSettings) -> Result<Self> {
        Ok(SpinningCar {
            scene: unsafe { Scene::new(renderer, settings)? },
            stationary: false,
            placement: (0.0, 0.0),
        })
    }

    fn description(&self) -> &str {
        "A car moving at 8 units and turning at 90 degrees per second"
    }

    fn lines(&self) -> Vec<Line> {
        let car = if self.stationary { "stationary" } else { "moving" };
        vec![vec![("Car: ".to_string(), DIM), (car.to_string(), TEXT)]]
    }

    fn shortcuts(&self) -> &[(&'static str, &'static str)] {
        &[("S", "stationary car")]
    }

    fn input(&mut self, event: &application::Event) {
        if controls::toggles_stationary(event) {
            self.stationary = !self.stationary;
        }
    }

    fn advance(&mut self, seconds: f64) {
        self.placement = scene::car_placement(seconds, self.stationary);
    }

    unsafe fn draw(&self, renderer: &Renderer) -> Result<()> {
        unsafe { self.scene.draw(renderer, self.placement) }
    }

    fn state(&self) -> Vec<(&'static str, Value)> {
        vec![
            ("CarPosition", self.placement.0.into()),
            ("CarTurn", self.placement.1.into()),
            ("Stationary", self.stationary.into()),
        ]
    }
}

fn main() -> Result<()> {
    scene_test::run::<SpinningCar>(Path::new(env!("CARGO_MANIFEST_DIR")))
}
