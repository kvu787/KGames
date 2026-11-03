// MinimalSceneTest's parts, as a library, so that Test/MinimalSceneTest can
// test the sphere's motion and the settings without a window. Main.rs is the
// program, which Library/SceneTest runs.

#[path = "Scene.rs"]
pub mod scene;
#[path = "Settings.rs"]
pub mod settings;

pub const NAME: &str = "Minimal Scene Test";
