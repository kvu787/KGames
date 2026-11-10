// What the player does in a frame, from the gamepad and the keyboard, which
// both do everything:
//
// | Gamepad               | Keyboard              | Does                                 |
// | --------------------- | --------------------- | ------------------------------------ |
// | Right stick           | Arrows or WASD        | Pushes the car toward there          |
// | Left trigger          | Space                 | Brakes                               |
// | D-pad left and right  | Q and E               | The previous and next vehicle        |
// | D-pad down and up     | PageDown and PageUp   | The previous and next scene          |
// | X                     | R                     | Puts the car back at the start       |
// | Menu                  | C                     | The fixed or the following camera    |
// | RB and left stick     | The wheel             | Zooms                                |
// | Y                     | Home                  | Zooms back to the camera's height    |
// | View                  | F1                    | Shows or hides the card              |
//
// Without the window's focus, the interface has no input at all, so the
// controls are at rest.

#pragma once

#include <array>
#include <cstdint>

namespace veehiicuul::controls {

struct Controls {
    // Where the player pushes the car, on the screen, +X right and +Y up,
    // about 1 long at most. The dead zones apply later, along the car's
    // own directions.
    std::array<double, 2> push{};
    // How hard they brake, from 0 to 1.
    double brake = 0.0;
    // To the previous vehicle or scene, -1, or the next, 1.
    std::int32_t vehicle = 0;
    std::int32_t scene = 0;
    bool restart = false;
    bool toggle_camera = false;
    // How fast to zoom in, from -1, out, to 1, and by how many notches of the
    // wheel, positive in.
    double zoom = 0.0;
    double notches = 0.0;
    bool default_zoom = false;
    bool toggle_card = false;

    // Reading the frame's controls from the interface, `read`, comes with the
    // rest of the port to Library/UserInterface.

    bool operator==(const Controls&) const = default;
};

} // namespace veehiicuul::controls
