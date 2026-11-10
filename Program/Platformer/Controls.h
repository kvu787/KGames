// What the player does in a frame, from the keyboard and the gamepad, which
// both do everything:
//
// | Gamepad               | Keyboard              | Does                        |
// | --------------------- | --------------------- | --------------------------- |
// | Left stick, D-pad     | S and F, or arrows    | Runs left and right         |
// | A                     | Space, E, or up       | Jumps; held jumps higher    |
// | X                     | R                     | Starts the run over         |
// | LB and RB             | PageDown and PageUp   | The previous and next level |
// | View                  | F1                    | Shows or hides the cards    |
//
// Without the window's focus, the interface has no input at all, so the
// controls are at rest.

#pragma once

#include <cstdint>

namespace user_interface {

class Ui;

} // namespace user_interface

namespace platformer::controls {

struct Controls {
    // The run, from -1, left, to 1, right.
    double push = 0.0;
    // The jump's press this frame, and whether it's still held.
    bool jump_pressed = false;
    bool jump_held = false;
    bool restart = false;
    // To the previous level, -1, or the next, 1.
    std::int32_t level = 0;
    bool toggle_cards = false;

    static Controls read(const user_interface::Ui& ui);

    bool operator==(const Controls&) const = default;
};

} // namespace platformer::controls
