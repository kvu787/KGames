#include "Platformer/Physics.h"

#include "Base/Text.h"

#include <format>
#include <limits>

namespace platformer::physics {

using level::Zone;

std::string describe_value(const Standing& standing) {
    return base::match(
        standing, [](const Standing::Still&) { return std::string("Still"); }, [](const Standing::Patrol& patrol) { return std::format("Patrol({})", patrol.index); });
}

Player Player::at(std::array<double, 2> spawn) {
    return Player{
        .position = {spawn[0], spawn[1] + Half[1]},
        .velocity = {0.0, 0.0},
        .grounded = false,
        .standing = Standing::Still{},
        .jump_until = -std::numeric_limits<double>::infinity(),
        .coyote_until = -std::numeric_limits<double>::infinity(),
    };
}

std::string describe_value(const Player& player) {
    auto pair = [](std::array<double, 2> values) {
        return std::format("[{}, {}]", base::debug(values[0]), base::debug(values[1]));
    };
    return std::format("Player {{ position: {}, velocity: {}, grounded: {}, standing: {}, jump_until: {}, coyote_until: {} }}", pair(player.position), pair(player.velocity),
                       player.grounded, describe_value(player.standing), base::debug(player.jump_until), base::debug(player.coyote_until));
}

bool collides(const Zone& zone, std::array<double, 2> center) {
    return center[0] + Half[0] > zone.x + Skin && center[0] - Half[0] < zone.right() - Skin && center[1] + Half[1] > zone.y + Skin && center[1] - Half[1] < zone.top() - Skin;
}

void step(Player& player, double now, double push, bool jump_held, std::array<double, 2> carry, std::span<const Solid> solids) {
    // The platform the player stands on carries them.
    player.position[0] += carry[0];
    player.position[1] += carry[1];

    // Run, and resolve along X: walls stop the player where they stand.
    player.velocity[0] = push * RunSpeed;
    player.position[0] += player.velocity[0] * Step;
    for (const Solid& solid : solids) {
        if (collides(solid.zone, player.position)) {
            if (player.velocity[0] > 0.0) {
                player.position[0] = solid.zone.x - Half[0];
            } else if (player.velocity[0] < 0.0) {
                player.position[0] = solid.zone.right() + Half[0];
            }
        }
    }

    // Fall, with the rise cut short once jump is let go of.
    player.velocity[1] -= Gravity * Step;
    if (!jump_held && player.velocity[1] > JumpCut) {
        player.velocity[1] = JumpCut;
    }
    if (player.velocity[1] < -FastestFall) {
        player.velocity[1] = -FastestFall;
    }
    player.position[1] += player.velocity[1] * Step;

    // Resolve along Y: land on tops, bump heads on bottoms.
    bool was_falling = player.velocity[1] <= 0.0;
    player.grounded = false;
    player.standing = Standing::Still{};
    for (const Solid& solid : solids) {
        if (collides(solid.zone, player.position)) {
            if (was_falling) {
                player.position[1] = solid.zone.top() + Half[1];
                player.velocity[1] = 0.0;
                player.grounded = true;
                player.standing = solid.standing;
            } else {
                player.position[1] = solid.zone.y - Half[1];
                player.velocity[1] = 0.0;
            }
        }
    }

    // The ground just left still counts for a moment, and a press just
    // before landing jumps on the step of the landing.
    if (player.grounded) {
        player.coyote_until = now + Coyote;
    }
    if (player.jump_until >= now && (player.grounded || player.coyote_until >= now) && player.velocity[1] <= 0.0) {
        player.velocity[1] = JumpSpeed;
        player.grounded = false;
        player.standing = Standing::Still{};
        player.jump_until = -std::numeric_limits<double>::infinity();
        player.coyote_until = -std::numeric_limits<double>::infinity();
    }
}

} // namespace platformer::physics
