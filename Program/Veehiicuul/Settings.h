// The game's own settings, after the ones every program has (see
// Library/UserInterface's README), in Settings.json in the program's folder:
// the scene and the vehicle to start with, the camera, and the push's dead
// zones. Each one that's wrong keeps its default, and a notice under the app
// bar says what's wrong with it.

#pragma once

#include "Veehiicuul/Driving.h"

#include <string>

namespace user_interface {

class Settings;

} // namespace user_interface

namespace veehiicuul::settings {

struct GameSettings {
    // The names of the scene and the vehicle to start with.
    std::string scene = "Ribeye";
    std::string vehicle = "Yellow";
    // Whether the camera starts following the car, instead of fixed.
    bool follow = false;
    driving::DeadZones dead_zones;

    // Reads the game's settings, in this order, which a missing
    // Settings.json is written in.
    static GameSettings read(user_interface::Settings& settings);

    bool operator==(const GameSettings&) const = default;
};

// How a failed check in a test shows the settings.
std::string describe_value(const GameSettings& settings);

} // namespace veehiicuul::settings
