// The game's own settings, after the ones every program has (see
// Library/UserInterface's README), in Settings.json in the program's folder:
// the scene and the vehicle to start with, the camera, and the push's dead
// zones. Each one that's wrong keeps its default, and a notice under the app
// bar says what's wrong with it.

#pragma once

#include "Veehiicuul/Driving.h"

#include <string>

namespace veehiicuul::settings {

struct GameSettings {
    // The names of the scene and the vehicle to start with.
    std::string scene = "Ribeye";
    std::string vehicle = "Yellow";
    // Whether the camera starts following the car, instead of fixed.
    bool follow = false;
    driving::DeadZones dead_zones;

    // Reading the game's settings from Library/UserInterface's, `read`, comes
    // with the rest of the port to it.

    bool operator==(const GameSettings&) const = default;
};

} // namespace veehiicuul::settings
