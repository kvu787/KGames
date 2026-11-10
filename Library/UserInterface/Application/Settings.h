// A program's settings file, Settings.json, in its folder: the settings that
// every program has, then the program's own.
//
// Each setting is read with its default and what makes it valid. A missing
// setting has its default, and so does a wrong one, which a notice under the
// app bar tells of. A missing file is written with every default, in the
// order they were read; once it exists, it's never changed. Values that are
// words ignore case; names don't.

#pragma once

#include "Base/Panic.h"
#include "Base/Sum.h"
#include "Json/Json.h"

#include <cstdint>
#include <expected>
#include <filesystem>
#include <format>
#include <optional>
#include <span>
#include <string>
#include <string_view>
#include <type_traits>
#include <utility>
#include <vector>

namespace user_interface {

enum class WindowMode : std::uint8_t {
    Maximized,
    BorderlessFullscreen,
};

std::string_view name(WindowMode mode);

WindowMode other(WindowMode mode);

class Settings {
public:
    static Settings load(const std::filesystem::path& path);

    // Settings from text, without a file, for tests.
    static Settings from_text(std::string_view text);

    Settings(Settings&&) = default;
    Settings& operator=(Settings&&) = default;
    Settings(const Settings&) = delete;
    Settings& operator=(const Settings&) = delete;

    const std::filesystem::path& path() const {
        return path_;
    }

    // What's wrong with the file and its settings, a sentence each.
    std::span<const std::string> problems() const {
        return problems_;
    }

    // A setting, which `parse` makes from its JSON, or says what's wrong
    // with: it returns a std::expected of the setting, or of a std::string.
    // `default_` is the JSON of its default, which must parse.
    template <class Parse> auto value(std::string_view name, json::Value default_, Parse&& parse) -> typename std::invoke_result_t<Parse&, const json::Value&>::value_type {
        auto fallback = parse(std::as_const(default_));
        if (!fallback) {
            base::panic(std::format("the default of {} is invalid: {}", name, fallback.error()));
        }
        defaults.emplace_back(std::string(name), std::move(default_));
        const json::Value* found = nullptr;
        if (members) {
            for (const auto& [member, member_value] : *members) {
                if (member == name) {
                    found = &member_value;
                    break;
                }
            }
        }
        if (found == nullptr) {
            return std::move(*fallback);
        }
        auto parsed = parse(*found);
        if (parsed) {
            return std::move(*parsed);
        }
        std::string_view reason = parsed.error();
        while (reason.ends_with('.')) {
            reason.remove_suffix(1);
        }
        problems_.push_back(std::format("{} {}, so it's the default.", name, reason));
        return std::move(*fallback);
    }

    bool boolean(std::string_view name, bool default_);

    // A number that `check` accepts, or says why not: it returns a
    // std::expected<void, std::string>.
    template <class Check> double number(std::string_view name, double default_, Check&& check) {
        return value(name, json::Value(json::Value::Number{default_}), [&](const json::Value& given) -> std::expected<double, std::string> {
            if (const json::Value::Number* as_number = base::as<json::Value::Number>(given)) {
                std::expected<void, std::string> checked = check(as_number->value);
                if (!checked) {
                    return std::unexpected(std::move(checked.error()));
                }
                return as_number->value;
            }
            return std::unexpected(std::format("must be a number, not {}", given.describe()));
        });
    }

    std::string string(std::string_view name, std::string_view default_);

    // One of the words, whatever its case, returned as `choices` has it.
    std::string choice(std::string_view name, std::string_view default_, std::span<const std::string_view> choices);

    // Writes the file with every setting's default, if it didn't exist.
    std::expected<void, std::string> write_if_missing() const;

private:
    Settings() = default;

    static Settings parse(const std::filesystem::path& path, std::string_view text);

    std::filesystem::path path_;
    std::optional<std::vector<std::pair<std::string, json::Value>>> members;
    bool existed = false;
    std::vector<std::pair<std::string, json::Value>> defaults;
    std::vector<std::string> problems_;
};

} // namespace user_interface
