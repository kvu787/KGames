#include "UserInterface/Application/Settings.h"

#include "Base/Files.h"

namespace user_interface {

namespace {

// Whether the texts are the same but for the case of their ASCII letters.
bool eq_ignore_ascii_case(std::string_view a, std::string_view b) {
    auto lower = [](char character) {
        return character >= 'A' && character <= 'Z' ? static_cast<char>(character - 'A' + 'a') : character;
    };
    if (a.size() != b.size()) {
        return false;
    }
    for (size_t index = 0; index < a.size(); index++) {
        if (lower(a[index]) != lower(b[index])) {
            return false;
        }
    }
    return true;
}

std::string list_choices(std::span<const std::string_view> choices) {
    if (choices.empty()) {
        return std::string();
    }
    if (choices.size() == 1) {
        return std::format("\"{}\"", choices[0]);
    }
    std::string rest;
    for (size_t index = 0; index + 1 < choices.size(); index++) {
        if (index > 0) {
            rest += ", ";
        }
        rest += std::format("\"{}\"", choices[index]);
    }
    return std::format("{} or \"{}\"", rest, choices.back());
}

} // namespace

std::string_view name(WindowMode mode) {
    switch (mode) {
    case WindowMode::Maximized:
        return "Maximized";
    case WindowMode::BorderlessFullscreen:
        return "BorderlessFullscreen";
    }
    base::unreachable();
}

WindowMode other(WindowMode mode) {
    switch (mode) {
    case WindowMode::Maximized:
        return WindowMode::BorderlessFullscreen;
    case WindowMode::BorderlessFullscreen:
        return WindowMode::Maximized;
    }
    base::unreachable();
}

Settings Settings::load(const std::filesystem::path& path) {
    std::expected<std::string, std::string> text = base::read_to_string(path);
    if (text) {
        return Settings::parse(path, *text);
    }
    Settings settings;
    settings.path_ = path;
    settings.existed = false;
    return settings;
}

Settings Settings::from_text(std::string_view text) {
    return Settings::parse(std::filesystem::path(), text);
}

Settings Settings::parse(const std::filesystem::path& path, std::string_view text) {
    Settings settings;
    settings.path_ = path;
    settings.existed = true;
    std::expected<json::Value, json::Error> parsed = json::parse(text);
    if (!parsed) {
        const json::Error& error = parsed.error();
        settings.problems_.push_back(std::format("It isn't valid JSON, at line {}, column {}: {}. Every setting has its default.", error.line, error.column, error.message));
    } else if (json::Value::Object* object = base::as<json::Value::Object>(*parsed)) {
        settings.members = std::move(object->members);
    } else {
        settings.problems_.push_back(std::format("It must be an object, not {}. Every setting has its default.", parsed->describe()));
    }
    return settings;
}

bool Settings::boolean(std::string_view name, bool default_) {
    return value(name, json::Value(json::Value::Bool{default_}), [](const json::Value& given) -> std::expected<bool, std::string> {
        if (const json::Value::Bool* as_bool = base::as<json::Value::Bool>(given)) {
            return as_bool->value;
        }
        return std::unexpected(std::format("must be true or false, not {}", given.describe()));
    });
}

std::string Settings::string(std::string_view name, std::string_view default_) {
    return value(name, json::Value(default_), [](const json::Value& given) -> std::expected<std::string, std::string> {
        if (const json::Value::String* text = base::as<json::Value::String>(given)) {
            return text->value;
        }
        return std::unexpected(std::format("must be text, not {}", given.describe()));
    });
}

std::string Settings::choice(std::string_view name, std::string_view default_, std::span<const std::string_view> choices) {
    return value(name, json::Value(default_), [&](const json::Value& given) -> std::expected<std::string, std::string> {
        if (const json::Value::String* text = base::as<json::Value::String>(given)) {
            for (std::string_view choice : choices) {
                if (eq_ignore_ascii_case(choice, text->value)) {
                    return std::string(choice);
                }
            }
            return std::unexpected(std::format("must be {}, not \"{}\"", list_choices(choices), text->value));
        }
        return std::unexpected(std::format("must be {}, not {}", list_choices(choices), given.describe()));
    });
}

std::expected<void, std::string> Settings::write_if_missing() const {
    if (existed || path_.empty()) {
        return {};
    }
    return base::write(path_, json::Value(json::Value::Object{defaults}).to_json() + "\n");
}

} // namespace user_interface
