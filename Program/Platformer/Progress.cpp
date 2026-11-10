#include "Platformer/Progress.h"

#include "Base/Files.h"
#include "Base/Numbers.h"
#include "Json/Json.h"

#include <algorithm>
#include <cmath>
#include <format>
#include <utility>

namespace platformer::progress {

using json::Value;

Progress::Progress(size_t levels) : unlocked(1), best(levels, std::nullopt), dirty(false) {}

std::expected<Progress, std::string> Progress::load(const std::filesystem::path& path, size_t levels) {
    Progress progress(levels);
    std::expected<std::string, std::string> text = base::read_to_string(path);
    if (!text) {
        if (base::is_not_found(text.error())) {
            return progress;
        }
        return std::unexpected(std::format("Couldn't read {}: {}", File, text.error()));
    }
    std::expected<Value, json::Error> parsed = json::parse(*text);
    if (!parsed) {
        return std::unexpected(std::format("{} line {}: {}", File, parsed.error().line, parsed.error().message));
    }
    const Value& value = *parsed;
    if (const Value* given = value.get("Unlocked")) {
        const Value::Number* number = base::as<Value::Number>(*given);
        if (number != nullptr && number->value >= 1.0) {
            progress.unlocked = std::clamp(base::saturating_cast<size_t>(number->value), size_t{1}, levels);
        } else {
            return std::unexpected(std::format("{}: Unlocked must be a number from 1, not {}", File, given->describe()));
        }
    }
    if (const Value* given = value.get("Best")) {
        const Value::Array* times = base::as<Value::Array>(*given);
        if (times == nullptr) {
            return std::unexpected(std::format("{}: Best must be an array, not {}", File, given->describe()));
        }
        for (size_t level = 0; level < std::min(times->values.size(), levels); level++) {
            const Value& time = times->values[level];
            const Value::Number* seconds = base::as<Value::Number>(time);
            if (seconds != nullptr && std::isfinite(seconds->value) && seconds->value > 0.0) {
                progress.best[level] = seconds->value;
            } else if (!base::is<Value::Null>(time)) {
                return std::unexpected(std::format("{}: Best[{}] must be seconds or null, not {}", File, level, time.describe()));
            }
        }
    }
    return progress;
}

std::expected<std::string, std::string> Progress::save(const std::filesystem::path& path) {
    std::vector<Value> times;
    times.reserve(best.size());
    for (const std::optional<double>& time : best) {
        times.push_back(time ? Value(Value::Number{*time}) : Value(Value::Null{}));
    }
    Value value = json::object({{"Unlocked", Value::Number{static_cast<double>(unlocked)}}, {"Best", Value::Array{std::move(times)}}});
    std::filesystem::path folder = path.parent_path();
    if (std::expected<void, std::string> made = base::create_dir_all(folder); !made) {
        return std::unexpected(std::format("Couldn't make {}: {}", base::display(folder), made.error()));
    }
    if (std::expected<void, std::string> written = base::write(path, value.to_json()); !written) {
        return std::unexpected(std::format("Couldn't write {}: {}", File, written.error()));
    }
    dirty = false;
    return std::string(File);
}

std::optional<std::optional<double>> Progress::record(size_t level, double seconds) {
    std::optional<double> previous = best[level];
    if (!previous || seconds < *previous) {
        best[level] = seconds;
        dirty = true;
        return std::optional<std::optional<double>>(std::in_place, previous);
    }
    return std::nullopt;
}

bool Progress::unlock(size_t level) {
    if (level >= unlocked && level < best.size()) {
        unlocked = level + 1;
        dirty = true;
        return true;
    }
    return false;
}

} // namespace platformer::progress
