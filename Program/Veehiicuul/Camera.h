// The cameras: orthographic, around from +Z toward +X by 45 degrees, so that
// the screen's up is toward -X and -Z. The game's camera looks down at 45
// degrees, at a fixed place on the track or at the car, with a view whose
// height, in units, the player zooms. The editors' cameras look straight
// down by default, so that the ground is as it is, or at the game's angle,
// and the editor pans and zooms them.

#pragma once

#include "Math/Math.h"

#include <array>
#include <numbers>

namespace veehiicuul::camera {

inline constexpr double Pitch = std::numbers::pi / 4.0;
inline constexpr double Yaw = std::numbers::pi / 4.0;
// The nearest and farthest that the camera draws, in units, which take in
// the whole view at any zoom.
inline constexpr float Near = 1.0f;
inline constexpr float Far = 1000.0f;
// The view's heights that zooming stays between, in units.
inline constexpr double Lowest = 2.0;
inline constexpr double Highest = 300.0;
// How fast the gamepad zooms, in units of height a second, and how much
// each notch of the wheel does.
inline constexpr double ZoomSpeed = 50.0;
inline constexpr double Notch = 1.25;

struct Camera {
    // Whether it follows the car, instead of looking at the track's place.
    bool follow = false;
    // How tall its view is, in units.
    double height = 0.0;

    bool operator==(const Camera&) const = default;
};

// A camera's view: what it looks at on the ground, how tall the view is,
// and how far it looks down, in radians, from 0 along the ground to a
// right angle straight down.
struct Eye {
    std::array<double, 2> center{};
    double height = 0.0;
    double pitch = 0.0;

    static Eye game(std::array<double, 2> eye_center, double eye_height);

    static Eye top_down(std::array<double, 2> eye_center, double eye_height);

    // From the world to the camera's space.
    math::Matrix view() const;

    // Where a point in the world is on a screen of `size` pixels.
    std::array<float, 2> to_screen(std::array<double, 3> point, std::array<float, 2> size) const;

    std::array<float, 2> ground_to_screen(std::array<double, 2> point, std::array<float, 2> size) const;

    // The point on the ground under a point on the screen.
    std::array<double, 2> to_ground(std::array<float, 2> point, std::array<float, 2> size) const;

    // Zooms by `factor`, keeping the ground under `anchor` on the screen
    // where it is.
    void zoom_at(double factor, std::array<float, 2> anchor, std::array<float, 2> size);

    bool operator==(const Eye&) const = default;

private:
    // Toward where it looks, along the screen to the right, and along it
    // up, in the world.
    std::array<std::array<double, 3>, 3> axes() const;
};

// A direction on the screen, +X right and +Y up, as one on the ground, along
// X and Z, for a push toward where the player wants the car to go.
std::array<double, 2> to_ground(std::array<double, 2> direction);

} // namespace veehiicuul::camera
