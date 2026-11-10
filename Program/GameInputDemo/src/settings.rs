// The demo's own setting, after the ones every program has (see
// Library/UserInterface's settings.rs), in Settings.json next to Cargo.toml:
//
//     {
//         "Display": "Primary",
//         "WindowMode": "Maximized",
//         "AllowTearing": true,
//         "RumbleTestInnerDeadZone": 0.1
//     }

use user_interface::Settings;

pub struct DemoSettings {
    // How far a stick must be from its center, as a fraction of full
    // deflection, before it runs its motor in the rumble test.
    pub rumble_dead_zone: f32,
}

impl DemoSettings {
    // Reads the demo's setting, or its default. Library/UserInterface shows a
    // notice under the app bar for a setting that's wrong.
    pub fn read(settings: &mut Settings) -> DemoSettings {
        // Less than 1, because the strength grows from the dead zone's edge
        // to the stick's.
        let dead_zone = settings.number("RumbleTestInnerDeadZone", 0.1, |number| {
            if (0.0..1.0).contains(&number) {
                Ok(())
            } else {
                Err(format!("must be a number from 0 to less than 1, not {number}"))
            }
        });
        DemoSettings {
            rumble_dead_zone: dead_zone as f32,
        }
    }
}
