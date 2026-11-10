// The levels: rectangles in world units, +X right and +Y up, with one tile
// 1 unit. Platforms to stand on, coins to collect, springs that bounce,
// spikes that kill, checkpoints that move the respawn, a door that opens
// once every coin is collected, and the goal behind it. The player's jump
// reaches about 2.5 units, so anything higher needs a spring or a step.

#pragma once

#include <array>
#include <cstdint>
#include <string>
#include <string_view>
#include <vector>

namespace platformer::level {

// An axis-aligned rectangle in world units, from its bottom left:
// Zone(x, y, width, height).
struct Zone {
    double x;
    double y;
    double width;
    double height;

    constexpr double right() const {
        return x + width;
    }

    constexpr double top() const {
        return y + height;
    }

    constexpr std::array<double, 2> center() const {
        return {x + width / 2.0, y + height / 2.0};
    }

    // Whether a centered box overlaps it.
    constexpr bool overlaps(std::array<double, 2> center, std::array<double, 2> half) const {
        return center[0] + half[0] > x && center[0] - half[0] < right() && center[1] + half[1] > y && center[1] - half[1] < top();
    }

    bool operator==(const Zone&) const = default;
};

// How a failed check in a test shows a zone.
std::string describe_value(const Zone& zone);

// A platform that patrols between where its zone is and `to`, there and back
// over `period` seconds, carrying whoever stands on it. Its place is a pure
// function of the simulation's time, so runs are the same at any frame rate.
struct Patrol {
    Zone zone;
    std::array<double, 2> to;
    double period;

    // Where the platform is at a time, as a zone.
    Zone at(double seconds) const;
};

// An sRGB color, a byte a channel, for the level's own look.
using LevelColor = std::array<std::uint8_t, 3>;

struct Level {
    std::string_view name;
    // Where the player starts, as the feet's middle on the ground.
    std::array<double, 2> spawn{};
    // What the camera stays inside. Falling below its bottom is a death.
    Zone bounds{};
    LevelColor sky{};
    LevelColor rock{};
    std::vector<Zone> solids;
    std::vector<std::array<double, 2>> coins;
    std::vector<Zone> spikes;
    std::vector<Zone> springs;
    std::vector<Zone> checkpoints;
    // Solid until every coin is collected.
    std::vector<Zone> doors;
    Zone goal{};
    std::vector<Patrol> patrols;

    Level() = default;
    Level(const Level&) = delete;
    Level& operator=(const Level&) = delete;
    Level(Level&&) = default;
    Level& operator=(Level&&) = default;
};

std::vector<Level> levels();

} // namespace platformer::level
