// No console window in any build.
#![windows_subsystem = "windows"]

use std::path::Path;

use graphics::Renderer;
use json::Value;
use minimal_scene_test::NAME;
use minimal_scene_test::scene::{Motion, Scene};
use minimal_scene_test::settings::SceneSettings;
use windows_core::Result;

// The scene, with where the sphere is, which Library/SceneTest runs.
struct MovingSphere {
    scene: Scene,
    motion: Motion,
    // The last frame's time, for how far the sphere moves.
    last_seconds: Option<f64>,
}

impl scene_test::Scene for MovingSphere {
    fn description(&self) -> &str {
        "A sphere moving at 8 units per second"
    }

    // The sphere doesn't jump ahead after the window was minimized.
    fn pause(&mut self) {
        self.last_seconds = None;
    }

    fn advance(&mut self, seconds: f64) {
        if let Some(last) = self.last_seconds {
            self.motion.advance((seconds - last) as f32);
        }
        self.last_seconds = Some(seconds);
    }

    unsafe fn draw(&self, renderer: &Renderer) -> Result<()> {
        unsafe { self.scene.draw(renderer, self.motion.position) }
    }

    fn state(&self) -> Vec<(&'static str, Value)> {
        vec![
            ("SpherePosition", self.motion.position.into()),
            ("SphereDirection", self.motion.direction.into()),
        ]
    }
}

fn main() -> Result<()> {
    let settings_path = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), r"\Settings.json"));
    scene_test::run(NAME, settings_path, |renderer, settings: &SceneSettings| {
        Ok(MovingSphere {
            scene: unsafe { Scene::new(renderer, settings)? },
            motion: Motion::default(),
            last_seconds: None,
        })
    })
}
