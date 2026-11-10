#include "Platformer/Level.h"

#include "Base/Numbers.h"
#include "Base/Text.h"

#include <format>
#include <utility>

namespace platformer::level {

std::string describe_value(const Zone& zone) {
    return std::format("Zone {{ x: {}, y: {}, width: {}, height: {} }}", base::debug(zone.x), base::debug(zone.y), base::debug(zone.width), base::debug(zone.height));
}

Zone Patrol::at(double seconds) const {
    double phase = base::rem_euclid(seconds / period, 1.0);
    // A triangle wave: out over the first half, back over the second.
    double along = phase < 0.5 ? phase * 2.0 : 2.0 - phase * 2.0;
    return Zone{
        .x = zone.x + (to[0] - zone.x) * along,
        .y = zone.y + (to[1] - zone.y) * along,
        .width = zone.width,
        .height = zone.height,
    };
}

namespace {

// Level 1: one of everything. The last coin sits right at the door, so
// collecting it can open the door and win the run in the same breath.
Level first_steps() {
    Level level;
    level.name = "First Steps";
    level.spawn = {1.5, 0.0};
    level.bounds = Zone(-2.0, -4.0, 68.0, 24.0);
    level.sky = {0x8E, 0xC9, 0xE8};
    level.rock = {0x4A, 0x55, 0x68};
    level.solids = {
        // The ground, with a pit at 14..19 and another at 44..52.
        Zone(-2.0, -2.0, 16.0, 2.0),
        Zone(19.0, -2.0, 25.0, 2.0),
        Zone(52.0, -2.0, 14.0, 2.0),
        // A step, a floating platform over the first pit, and a ledge
        // that only the spring reaches.
        Zone(9.0, 0.0, 2.0, 1.0),
        Zone(15.5, 1.5, 3.0, 0.5),
        Zone(24.0, 2.5, 3.0, 0.5),
    };
    level.coins = {{5.5, 0.7}, {17.0, 2.7}, {25.5, 3.8}, {40.0, 0.7}, {57.0, 0.7}};
    level.spikes = {Zone(31.0, 0.0, 2.0, 0.6)};
    level.springs = {Zone(21.5, 0.0, 1.2, 0.4)};
    level.checkpoints = {Zone(36.0, 0.0, 0.8, 1.6)};
    level.doors = {Zone(58.0, 0.0, 0.8, 3.0)};
    // The goal starts at the door's face: strictly behind it while the
    // door is closed, and one step past it once the door opens, so the
    // last coin at the doorstep can open the door and win the run in
    // the same frame.
    level.goal = Zone(58.0, 0.0, 2.8, 3.0);
    level.patrols = {Patrol{
        .zone = Zone(44.0, -0.5, 3.0, 0.5),
        .to = {49.0, -0.5},
        .period = 4.0,
    }};
    return level;
}

// Level 2: longer, higher, and meaner, with two checkpoints.
Level ridgeline() {
    Level level;
    level.name = "Ridgeline";
    level.spawn = {1.5, 0.0};
    level.bounds = Zone(-2.0, -4.0, 92.0, 28.0);
    level.sky = {0xF2, 0xC1, 0x8C};
    level.rock = {0x5C, 0x4A, 0x52};
    level.solids = {
        Zone(-2.0, -2.0, 14.0, 2.0),
        Zone(16.0, -1.0, 6.0, 1.0),
        Zone(25.0, 0.0, 8.0, 2.0),
        Zone(36.0, -2.0, 18.0, 2.0),
        Zone(57.0, -1.0, 5.0, 1.5),
        Zone(65.0, -2.0, 25.0, 2.0),
        // Steps up the ridge.
        Zone(40.0, 0.0, 2.0, 1.0),
        Zone(44.0, 1.0, 2.0, 1.5),
        Zone(48.0, 3.8, 3.0, 0.5),
        // A high shelf over the last stretch.
        Zone(72.0, 2.8, 4.0, 0.5),
    };
    level.coins = {
        {5.0, 0.7}, {18.5, 0.7}, {29.0, 2.7}, {45.0, 3.2}, {49.5, 5.0}, {59.5, 1.2}, {74.0, 4.0},
    };
    level.spikes = {Zone(36.5, 0.0, 2.5, 0.6), Zone(67.0, 0.0, 2.0, 0.6)};
    level.springs = {Zone(47.0, 0.0, 1.2, 0.4), Zone(70.5, 0.0, 1.2, 0.4)};
    level.checkpoints = {Zone(27.0, 2.0, 0.8, 1.6), Zone(55.0, 0.0, 0.8, 1.6)};
    level.doors = {Zone(84.0, 0.0, 0.8, 3.0)};
    level.goal = Zone(84.0, 0.0, 3.0, 3.0);
    level.patrols = {
        Patrol{
            .zone = Zone(14.5, -0.5, 1.5, 0.5),
            .to = {14.5, 2.5},
            .period = 3.0,
        },
        Patrol{
            .zone = Zone(62.0, -0.5, 3.0, 0.5),
            .to = {62.0, -0.5},
            .period = 1.0,
        },
    };
    return level;
}

} // namespace

std::vector<Level> levels() {
    std::vector<Level> all;
    all.reserve(2);
    all.push_back(first_steps());
    all.push_back(ridgeline());
    return all;
}

} // namespace platformer::level
