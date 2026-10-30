// The demo's own setting, after the ones every program has (see
// Library/Application), in Settings.json next to Cargo.toml:
//
//     {
//         "Display": "Primary",
//         "WindowMode": "Maximized",
//         "AllowTearing": true,
//         "RumbleTestInnerDeadZone": 0.1
//     }

use application::{ProgramSettings, Settings};
use json::Value;

pub struct DemoSettings {
    // How far a stick must be from its center, as a fraction of full
    // deflection, before it runs its motor in the rumble test.
    pub rumble_dead_zone: f32,
}

impl ProgramSettings for DemoSettings {
    const NAMES: &'static [&'static str] = &["RumbleTestInnerDeadZone"];

    fn defaults() -> (Settings, Self) {
        (
            Settings::default(),
            Self {
                rumble_dead_zone: 0.1,
            },
        )
    }

    fn apply(&mut self, _name: &str, value: &Value) -> Result<(), String> {
        // Less than 1, because the strength grows from the dead zone's edge
        // to the stick's.
        match value {
            Value::Number(number) if (0.0..1.0).contains(number) => {
                self.rumble_dead_zone = *number as f32;
                Ok(())
            }
            _ => Err(format!(
                "must be a number from 0 to less than 1, not {}",
                value.describe()
            )),
        }
    }

    fn default_lines(&self) -> Vec<String> {
        vec![format!("\"RumbleTestInnerDeadZone\": {}", self.rumble_dead_zone)]
    }
}
