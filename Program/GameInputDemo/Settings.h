// The demo's own setting, after the ones every program has (see
// Library/UserInterface's Application/Settings.h), in Settings.json next to
// the demo's projects:
//
//     {
//         "Display": "Primary",
//         "WindowMode": "Maximized",
//         "AllowTearing": true,
//         "RumbleTestInnerDeadZone": 0.1
//     }

#pragma once

#include "UserInterface/Application/Settings.h"

namespace game_input_demo {

struct DemoSettings {
    // How far a stick must be from its center, as a fraction of full
    // deflection, before it runs its motor in the rumble test.
    float rumble_dead_zone;

    // Reads the demo's setting, or its default. Library/UserInterface shows a
    // notice under the app bar for a setting that's wrong.
    static DemoSettings read(user_interface::Settings& settings);
};

} // namespace game_input_demo
