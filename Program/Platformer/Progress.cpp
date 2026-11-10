#include "Platformer/Progress.h"

#include "Base/Files.h"
#include "Base/Numbers.h"
#include "Base/Text.h"
#include "Json/Json.h"

#include <algorithm>
#include <cmath>
#include <cstdint>
#include <format>
#include <utility>

namespace platformer::progress {

using json::Value;

namespace {

// Whether what base::read_to_string says went wrong is that the file isn't
// there: the system's errors that are std::io::ErrorKind::NotFound in Rust.
// Base's errors end with the system's code, like "(os error 2)".
bool is_not_found(std::string_view error) {
    constexpr std::string_view Before = "(os error ";
    size_t start = error.rfind(Before);
    if (start == std::string_view::npos || !error.ends_with(')')) {
        return false;
    }
    std::string_view digits = error.substr(start + Before.size(), error.size() - 1 - (start + Before.size()));
    std::optional<std::uint64_t> code = base::parse_u64(digits);
    // ERROR_FILE_NOT_FOUND, ERROR_PATH_NOT_FOUND, ERROR_INVALID_DRIVE,
    // ERROR_BAD_NETPATH and ERROR_BAD_NET_NAME.
    return code && (*code == 2 || *code == 3 || *code == 15 || *code == 53 || *code == 67);
}

} // namespace

Progress::Progress(size_t levels) : unlocked(1), best(levels, std::nullopt), dirty(false) {}

std::expected<Progress, std::string> Progress::load(const std::filesystem::path& path, size_t levels) {
    Progress progress(levels);
    std::expected<std::string, std::string> text = base::read_to_string(path);
    if (!text) {
        if (is_not_found(text.error())) {
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
    // A file that's named without a folder has none to make.
    std::filesystem::path folder = path.parent_path();
    if (!folder.empty()) {
        if (std::expected<void, std::string> made = base::create_dir_all(folder); !made) {
            return std::unexpected(std::format("Couldn't make {}: {}", base::display(folder), made.error()));
        }
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
