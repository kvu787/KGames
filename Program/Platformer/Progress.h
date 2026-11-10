// The persistent store: which levels are unlocked and each level's best
// time, in Save.json in the data folder - the program's folder, or a
// script's output folder, so scripted runs and tests leave the player's
// progress alone. A missing file is a fresh start; a wrong one keeps the
// defaults, and the program shows a notice saying what's wrong.

#pragma once

#include <cstddef>
#include <expected>
#include <filesystem>
#include <optional>
#include <string>
#include <string_view>
#include <vector>

namespace platformer::progress {

inline constexpr std::string_view File = "Save.json";

struct Progress {
    // How many levels are playable, at least 1.
    size_t unlocked;
    // Each level's best time, in seconds.
    std::vector<std::optional<double>> best;
    // Whether there's something the file doesn't have yet. Consumers set it;
    // the first SaveProgress in a cascade writes and clears it, so one
    // cascade saves once however many things it changed.
    bool dirty;

    explicit Progress(size_t levels);

    // Reads the file, or starts fresh. A file that can't be used says why.
    static std::expected<Progress, std::string> load(const std::filesystem::path& path, size_t levels);

    // Writes the file and comes back clean. Returns the file's name.
    std::expected<std::string, std::string> save(const std::filesystem::path& path);

    // A finished run's time, if it's the level's best. Marks dirty when it is.
    std::optional<std::optional<double>> record(size_t level, double seconds);

    // Unlocks a level, if it isn't. Marks dirty when it does.
    bool unlock(size_t level);

    bool operator==(const Progress&) const = default;
};

} // namespace platformer::progress
