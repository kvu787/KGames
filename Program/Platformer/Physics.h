// How the player moves, in fixed steps of Step seconds: run, gravity, jump,
// and axis-separated collision against the solids, where a box resting
// against a face, less than Skin into it, touches it without colliding.
// Jumps are forgiving the way platformers are: a press up to Buffer seconds
// before landing jumps on the very step of the landing, and a press up to
// Coyote seconds after walking off a ledge still jumps. All of it runs on the
// simulation's clock, so a run is the same at any frame rate.

#pragma once

#include "Base/Sum.h"
#include "Platformer/Level.h"

#include <array>
#include <cstddef>
#include <span>
#include <string>
#include <utility>
#include <variant>

namespace platformer::physics {

// The fixed step, a 240th of a second: four steps in a 60 Hz frame.
inline constexpr double Step = 1.0 / 240.0;
// How fast the player runs, in units a second.
inline constexpr double RunSpeed = 8.0;
inline constexpr double Gravity = 36.0;
// The jump's upward speed, which reaches about 2.5 units.
inline constexpr double JumpSpeed = 13.4;
// Letting go of jump while rising cuts the rise to this, for short hops.
inline constexpr double JumpCut = 4.0;
// A spring's upward speed, which reaches about 5 units.
inline constexpr double SpringSpeed = 19.0;
inline constexpr double FastestFall = 24.0;
inline constexpr double Coyote = 0.1;
inline constexpr double Buffer = 0.12;
// The player's box, as half-widths around its center.
inline constexpr std::array<double, 2> Half = {0.35, 0.55};
// How deep the box has to be in a solid, on both axes, to collide with it.
// The box is stored as its center, and a center resolved against a face
// doesn't always round back to it: a player landed on a top at 2.0 is at
// 2.0 + 0.55, and less 0.55 again that's 1.9999999999999998. Without the
// skin, that hair inside the platform made the next step's run take it for a
// wall, and push the player back to its edge and off it. Rounding at the
// levels' sizes is around 1e-15, and a standing player's step of gravity
// sinks the box 0.000625 into the ground.
inline constexpr double Skin = 1e-9;

// What the player stands on, so a moving platform can carry them.
struct Standing {
    struct Still {
        bool operator==(const Still&) const = default;
    };
    // A patrol, by its index in the level's.
    struct Patrol {
        size_t index;
        bool operator==(const Patrol&) const = default;
    };
    std::variant<Still, Patrol> value;

    template <base::AlternativeOf<Standing> T> Standing(T&& alternative) : value(std::forward<T>(alternative)) {}

    bool operator==(const Standing&) const = default;
};

// How a failed check in a test shows what the player stands on.
std::string describe_value(const Standing& standing);

struct Player {
    // The box's center.
    std::array<double, 2> position;
    std::array<double, 2> velocity;
    bool grounded;
    // What the player stands on, while grounded.
    Standing standing;
    // Until when a jump press still counts, and until when the ground the
    // player left still does, on the simulation's clock.
    double jump_until;
    double coyote_until;

    // At the spawn point, which is the feet's middle.
    static Player at(std::array<double, 2> spawn);
};

// How a failed check in a test shows a player.
std::string describe_value(const Player& player);

// A solid the player can collide with this step, and what it is.
struct Solid {
    level::Zone zone;
    Standing standing;

    Solid(level::Zone zone, Standing standing) : zone(zone), standing(standing) {}
    Solid(const Solid&) = delete;
    Solid& operator=(const Solid&) = delete;
    Solid(Solid&&) = default;
    Solid& operator=(Solid&&) = default;
};

// Whether the player's box at `center` is more than Skin into a solid, on
// both axes.
bool collides(const level::Zone& zone, std::array<double, 2> center);

// One fixed step of movement against the solids, at `now` on the simulation's
// clock, with the stick's push and the jump's state. The solids are where
// they are AFTER the platforms moved this step, and `carry` is how far the
// platform the player stood on moved, which moves the player first.
void step(Player& player, double now, double push, bool jump_held, std::array<double, 2> carry, std::span<const Solid> solids);

} // namespace platformer::physics
