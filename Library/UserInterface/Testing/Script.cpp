#include "UserInterface/Testing/Script.h"

#include "Base/Files.h"
#include "Base/Numbers.h"
#include "Base/Panic.h"
#include "Base/Sum.h"
#include "Base/Text.h"
#include "UserInterface/Core/Interaction.h"
#include "UserInterface/Core/Interface.h"
#include "UserInterface/Drawing/Geometry.h"

#include <algorithm>
#include <array>
#include <cmath>
#include <cstddef>
#include <format>
#include <optional>
#include <ranges>
#include <utility>

namespace user_interface {

namespace {

constexpr std::array<std::string_view, 15> Actions{
    "Move", "Press", "Release", "Click", "DoubleClick", "Drag", "Scroll", "Key", "Hold", "Type", "Gamepad", "Focus", "Capture", "Expect", "Program",
};

// The object's member with the name, if the value is an object and the
// member is a T, like a json::Value::String, or null.
template <class T> const T* member(const json::Value& value, std::string_view name) {
    const json::Value* found = value.get(name);
    return found == nullptr ? nullptr : base::as<T>(*found);
}

// The point of a list that's two numbers, [x, y].
std::optional<Point> xy(const json::Value::Array& items) {
    if (items.values.size() != 2) {
        return std::nullopt;
    }
    const json::Value::Number* x = base::as<json::Value::Number>(items.values[0]);
    const json::Value::Number* y = base::as<json::Value::Number>(items.values[1]);
    if (x == nullptr || y == nullptr) {
        return std::nullopt;
    }
    return Point(static_cast<float>(x->value), static_cast<float>(y->value));
}

// The names of the targets that the last frame listed, with ", " between
// them.
std::string listed_names(const Interface& ui) {
    std::string names;
    bool first = true;
    for (const Target& target : ui.targets()) {
        if (!target.listed) {
            continue;
        }
        if (!first) {
            names += ", ";
        }
        names += target.name;
        first = false;
    }
    return names;
}

// A point: a target's name, which is its middle; [x, y] in the window; or
// { "Target": name, "At": [x, y] }, a place in a target as fractions of its
// size from its top left, like [0.25, 0.5] for a quarter of the way along a
// slider.
std::expected<Point, std::string> point(const json::Value& value, const Interface& ui) {
    auto target = [&](std::string_view name) -> std::expected<Rect, std::string> {
        if (const Target* found = ui.target(name)) {
            return found->rect;
        }
        return std::unexpected(std::format("points at {}, which the last frame didn't show. It showed: {}", name, listed_names(ui)));
    };
    if (const json::Value::String* name = base::as<json::Value::String>(value)) {
        std::expected<Rect, std::string> rect = target(name->value);
        if (!rect) {
            return std::unexpected(std::move(rect.error()));
        }
        return rect->center();
    }
    if (const json::Value::Array* items = base::as<json::Value::Array>(value)) {
        if (std::optional<Point> at = xy(*items)) {
            return *at;
        }
        return std::unexpected("needs [x, y]");
    }
    if (base::is<json::Value::Object>(value)) {
        const json::Value::String* name = member<json::Value::String>(value, "Target");
        if (name == nullptr) {
            return std::unexpected("needs a Target's name with its At");
        }
        const json::Value::Array* fractions = member<json::Value::Array>(value, "At");
        if (fractions == nullptr) {
            return std::unexpected("needs At, as [x, y] fractions of the target");
        }
        std::optional<Point> fraction = xy(*fractions);
        if (!fraction) {
            return std::unexpected("needs At, as [x, y] fractions of the target");
        }
        std::expected<Rect, std::string> rect = target(name->value);
        if (!rect) {
            return std::unexpected(std::move(rect.error()));
        }
        return rect->at_fraction(*fraction);
    }
    return std::unexpected(std::format("needs a target's name or [x, y], not {}", value.describe()));
}

// The number, if there's a value and it's one. `value` is null for none.
std::optional<double> number(const json::Value* value) {
    if (value != nullptr) {
        if (const json::Value::Number* found = base::as<json::Value::Number>(*value)) {
            return found->value;
        }
    }
    return std::nullopt;
}

// The button that a step names, or the left one if it names none. `value`
// is null for none.
std::expected<MouseButton, std::string> button_of(const json::Value* value) {
    if (value == nullptr) {
        return MouseButton::Left;
    }
    if (const json::Value::String* name = base::as<json::Value::String>(*value)) {
        if (std::optional<MouseButton> button = mouse_button_parse(name->value)) {
            return *button;
        }
        return std::unexpected(std::format("names no button: {}", name->value));
    }
    return std::unexpected(std::format("has a Button of {}, which isn't Left, Right or Middle", value->describe()));
}

} // namespace

Script::Script(double frame_time, std::vector<Step> steps) : frame_time(frame_time), steps(std::move(steps)) {}

std::expected<Script, std::string> Script::load(const std::filesystem::path& path) {
    std::expected<std::string, std::string> text = base::read_to_string(path);
    if (!text) {
        return std::unexpected(std::format("Couldn't read the script {}: {}", base::display(path), text.error()));
    }
    std::expected<Script, std::string> script = Script::parse(*text);
    if (!script) {
        return std::unexpected(std::format("The script {} {}", base::display(path), script.error()));
    }
    return script;
}

std::expected<Script, std::string> Script::parse(std::string_view text) {
    std::expected<json::Value, json::Error> root = json::parse(text);
    if (!root) {
        const json::Error& error = root.error();
        return std::unexpected(std::format("isn't valid JSON, at line {}, column {}: {}", error.line, error.column, error.message));
    }
    double frame_time = DefaultFrameTime;
    if (const json::Value* given = root->get("FrameTime")) {
        const json::Value::Number* time = base::as<json::Value::Number>(*given);
        if (time == nullptr || !(time->value > 0.0)) {
            return std::unexpected(std::format("has a FrameTime of {}, which isn't a positive number", given->describe()));
        }
        frame_time = time->value;
    }
    const json::Value::Array* items = member<json::Value::Array>(*root, "Steps");
    if (items == nullptr) {
        return std::unexpected("must have Steps, a list");
    }
    std::vector<Step> steps;
    for (size_t index = 0; index < items->values.size(); index++) {
        const json::Value& item = items->values[index];
        const json::Value::Object* object = base::as<json::Value::Object>(item);
        if (object == nullptr) {
            return std::unexpected(std::format("has step {}, which isn't an object", index));
        }
        const json::Value::Number* frame = member<json::Value::Number>(item, "Frame");
        if (frame == nullptr || !(frame->value >= 1.0 && frame->value - std::trunc(frame->value) == 0.0)) {
            return std::unexpected(std::format("has step {} without a Frame, a whole number from 1", index));
        }
        const std::pair<std::string, json::Value>* only = nullptr;
        size_t actions = 0;
        for (const std::pair<std::string, json::Value>& named : object->members) {
            if (named.first != "Frame") {
                only = &named;
                actions += 1;
            }
        }
        if (actions != 1) {
            return std::unexpected(std::format("has step {}, which must have one action besides its Frame", index));
        }
        const auto& [action, value] = *only;
        if (!std::ranges::contains(Actions, std::string_view(action))) {
            std::string known;
            for (std::string_view name : Actions) {
                if (!known.empty()) {
                    known += ", ";
                }
                known += name;
            }
            return std::unexpected(std::format("has step {} with the action {}, which isn't one of {}", index, action, known));
        }
        steps.push_back(Step{
            .frame = base::saturating_cast<std::uint64_t>(frame->value),
            .action = action,
            .value = value,
        });
    }
    std::ranges::stable_sort(steps, {}, &Step::frame);
    return Script(frame_time, std::move(steps));
}

std::uint64_t Script::last_frame() const {
    return steps.empty() ? 1 : steps.back().frame;
}

ScriptRunner::ScriptRunner(Script script) : script(std::move(script)) {}

double ScriptRunner::frame_time() const {
    return script.frame_time;
}

std::uint64_t ScriptRunner::last_frame() const {
    return script.last_frame();
}

std::expected<ScriptFrame, std::string> ScriptRunner::frame(std::uint64_t frame, const Interface& ui) {
    ScriptFrame output;
    if (frame == 0) {
        output.events.push_back(Event::FocusGained{});
    }
    if (auto found = scheduled.find(frame); found != scheduled.end()) {
        output.events.insert(output.events.end(), std::make_move_iterator(found->second.begin()), std::make_move_iterator(found->second.end()));
        scheduled.erase(found);
    }
    // The steps aren't copied first: nothing here changes the script.
    for (const Script::Step& step : script.steps) {
        if (step.frame != frame) {
            continue;
        }
        auto fail = [&](std::string_view error) {
            return std::unexpected(std::format("Frame {}: {} {}", frame, step.action, error));
        };
        const json::Value& value = step.value;
        std::string_view action = step.action;
        if (action == "Move") {
            std::expected<Point, std::string> to = point(value, ui);
            if (!to) {
                return fail(to.error());
            }
            output.events.push_back(Event::PointerMoved{*to});
        } else if (action == "Press") {
            const json::Value* at = &value;
            MouseButton button = MouseButton::Left;
            if (base::is<json::Value::Object>(value) && value.get("Target") == nullptr) {
                at = value.get("At");
                if (at == nullptr) {
                    return fail("needs At");
                }
                std::expected<MouseButton, std::string> named = button_of(value.get("Button"));
                if (!named) {
                    return fail(named.error());
                }
                button = *named;
            }
            std::expected<Point, std::string> to = point(*at, ui);
            if (!to) {
                return fail(to.error());
            }
            output.events.push_back(Event::PointerMoved{*to});
            output.events.push_back(Event::ButtonPressed{button});
        } else if (action == "Release") {
            MouseButton button = MouseButton::Left;
            const json::Value::Bool* left = base::as<json::Value::Bool>(value);
            if (left == nullptr || !left->value) {
                std::expected<MouseButton, std::string> named = button_of(&value);
                if (!named) {
                    return fail(named.error());
                }
                button = *named;
            }
            output.events.push_back(Event::ButtonReleased{button});
        } else if (action == "Click" || action == "DoubleClick") {
            std::expected<Point, std::string> to = point(value, ui);
            if (!to) {
                return fail(to.error());
            }
            output.events.push_back(Event::PointerMoved{*to});
            output.events.push_back(Event::ButtonPressed{MouseButton::Left});
            if (action == "Click") {
                schedule(frame + 1, Event::ButtonReleased{MouseButton::Left});
            } else {
                output.events.push_back(Event::ButtonReleased{MouseButton::Left});
                schedule(frame + 1, Event::ButtonPressed{MouseButton::Left});
                schedule(frame + 1, Event::ButtonReleased{MouseButton::Left});
            }
        } else if (action == "Drag") {
            const json::Value* from_value = value.get("From");
            if (from_value == nullptr) {
                return fail("needs From");
            }
            std::expected<Point, std::string> from = point(*from_value, ui);
            if (!from) {
                return fail(from.error());
            }
            const json::Value* to_value = value.get("To");
            if (to_value == nullptr) {
                return fail("needs To");
            }
            std::expected<Point, std::string> to = point(*to_value, ui);
            if (!to) {
                return fail(to.error());
            }
            std::uint64_t frames = 10;
            if (const json::Value* given = value.get("Frames")) {
                const json::Value::Number* count = base::as<json::Value::Number>(*given);
                if (count == nullptr || !(count->value >= 1.0)) {
                    return fail(std::format("has Frames of {}, which must be at least 1", given->describe()));
                }
                frames = base::saturating_cast<std::uint64_t>(count->value);
            }
            std::expected<MouseButton, std::string> button = button_of(value.get("Button"));
            if (!button) {
                return fail(button.error());
            }
            output.events.push_back(Event::PointerMoved{*from});
            output.events.push_back(Event::ButtonPressed{*button});
            for (std::uint64_t index = 1; index <= frames; index++) {
                schedule(frame + index, Event::PointerMoved{from->lerp(*to, static_cast<float>(index) / static_cast<float>(frames))});
            }
            schedule(frame + frames + 1, Event::ButtonReleased{*button});
        } else if (action == "Scroll") {
            const json::Value* at_value = value.get("At");
            if (at_value == nullptr) {
                return fail("needs At");
            }
            std::expected<Point, std::string> at = point(*at_value, ui);
            if (!at) {
                return fail(at.error());
            }
            std::optional<double> notches = number(value.get("Notches"));
            if (!notches) {
                return fail("needs Notches, a number");
            }
            output.events.push_back(Event::PointerMoved{*at});
            output.events.push_back(Event::Wheel{Point(0.0f, static_cast<float>(*notches))});
        } else if (action == "Key") {
            const json::Value::String* name = base::as<json::Value::String>(value);
            if (name == nullptr) {
                return fail("needs a key's name, like \"F11\" or \"Ctrl+S\"");
            }
            std::optional<Shortcut> shortcut = Shortcut::parse(name->value);
            if (!shortcut) {
                return fail(std::format("names no key: {}", name->value));
            }
            std::vector<Key> keys;
            for (auto [modifier_held, key] :
                 {std::pair(shortcut->modifiers.control, Key::Control), std::pair(shortcut->modifiers.shift, Key::Shift), std::pair(shortcut->modifiers.alt, Key::Alt)}) {
                if (modifier_held) {
                    keys.push_back(key);
                }
            }
            keys.push_back(shortcut->key_);
            std::vector<Key> pressed;
            for (Key key : keys) {
                if (!std::ranges::contains(held, key)) {
                    pressed.push_back(key);
                }
            }
            for (Key key : pressed) {
                output.events.push_back(Event::KeyPressed{.key = key, .repeat = false});
            }
            for (Key key : pressed | std::views::reverse) {
                schedule(frame + 1, Event::KeyReleased{key});
            }
        } else if (action == "Hold") {
            const json::Value::Array* names = base::as<json::Value::Array>(value);
            if (names == nullptr) {
                return fail("needs a list of keys");
            }
            std::vector<Key> keys;
            for (const json::Value& item : names->values) {
                const json::Value::String* name = base::as<json::Value::String>(item);
                if (name == nullptr) {
                    return fail(std::format("has {}, which isn't a key's name", item.describe()));
                }
                std::optional<Key> key = Key::parse(name->value);
                if (!key) {
                    return fail(std::format("names no key: {}", name->value));
                }
                keys.push_back(*key);
            }
            for (Key key : held) {
                if (!std::ranges::contains(keys, key)) {
                    output.events.push_back(Event::KeyReleased{key});
                }
            }
            for (Key key : keys) {
                if (!std::ranges::contains(held, key)) {
                    output.events.push_back(Event::KeyPressed{.key = key, .repeat = false});
                }
            }
            held = std::move(keys);
        } else if (action == "Type") {
            const json::Value::String* text = base::as<json::Value::String>(value);
            if (text == nullptr) {
                return fail("needs text");
            }
            for (char32_t character : base::chars(text->value)) {
                output.events.push_back(Event::Text{character});
            }
        } else if (action == "Gamepad") {
            std::expected<Gamepad, std::string> parsed = parse_gamepad(value);
            if (!parsed) {
                return fail(parsed.error());
            }
            gamepad = *parsed;
        } else if (action == "Focus") {
            const json::Value::Bool* focused = base::as<json::Value::Bool>(value);
            if (focused == nullptr) {
                return fail(std::format("must be true or false, not {}", value.describe()));
            }
            if (focused->value) {
                output.events.push_back(Event::FocusGained{held});
            } else {
                output.events.push_back(Event::FocusLost{});
            }
        } else if (action == "Capture") {
            const json::Value::String* name = base::as<json::Value::String>(value);
            if (name == nullptr) {
                return fail("needs a name for the files");
            }
            output.captures.push_back(name->value);
        } else if (action == "Expect") {
            output.expectations.push_back(value);
        } else if (action == "Program") {
            output.program_steps.push_back(value);
        } else {
            base::unreachable("internal error: entered unreachable code: the actions were checked as the script was parsed");
        }
    }
    // The gamepad reads every frame, like a real one.
    output.events.push_back(Event::Gamepad{gamepad});
    return output;
}

void ScriptRunner::schedule(std::uint64_t frame, Event event) {
    scheduled[frame].push_back(std::move(event));
}

void ScriptRunner::check(std::uint64_t frame, const json::Value& expected, const json::Value& state) {
    std::vector<std::string> mismatches;
    std::string path;
    compare(expected, state, path, mismatches);
    for (const std::string& mismatch : mismatches) {
        failures.push_back(std::format("Frame {}: {}", frame, mismatch));
    }
}

std::expected<Gamepad, std::string> parse_gamepad(const json::Value& value) {
    Gamepad gamepad;
    const json::Value::Bool* connected = member<json::Value::Bool>(value, "Connected");
    gamepad.connected = !(connected != nullptr && !connected->value);
    auto stick = [&](std::string_view name) -> std::expected<Point, std::string> {
        const json::Value* given = value.get(name);
        if (given == nullptr) {
            return Point::Zero;
        }
        if (const json::Value::Array* items = base::as<json::Value::Array>(*given)) {
            if (std::optional<Point> at = xy(*items)) {
                return *at;
            }
            return std::unexpected(std::format("has {}, which must be [x, y]", name));
        }
        return std::unexpected(std::format("has {} of {}, which must be [x, y]", name, given->describe()));
    };
    std::expected<Point, std::string> left_stick = stick("LeftStick");
    if (!left_stick) {
        return std::unexpected(std::move(left_stick.error()));
    }
    gamepad.left_stick = *left_stick;
    std::expected<Point, std::string> right_stick = stick("RightStick");
    if (!right_stick) {
        return std::unexpected(std::move(right_stick.error()));
    }
    gamepad.right_stick = *right_stick;
    gamepad.left_trigger = static_cast<float>(number(value.get("LeftTrigger")).value_or(0.0));
    gamepad.right_trigger = static_cast<float>(number(value.get("RightTrigger")).value_or(0.0));
    if (const json::Value* buttons = value.get("Buttons")) {
        const json::Value::Array* names = base::as<json::Value::Array>(*buttons);
        if (names == nullptr) {
            return std::unexpected("has Buttons, which must be a list");
        }
        for (const json::Value& item : names->values) {
            const json::Value::String* name = base::as<json::Value::String>(item);
            if (name == nullptr) {
                return std::unexpected(std::format("has a button of {}, which isn't a name", item.describe()));
            }
            std::optional<std::uint32_t> button = GamepadButtons::parse(name->value);
            if (!button) {
                return std::unexpected(std::format("names no gamepad button: {}", name->value));
            }
            gamepad.buttons.bits |= *button;
        }
    }
    return gamepad;
}

void compare(const json::Value& expected, const json::Value& actual, std::string& path, std::vector<std::string>& mismatches) {
    const json::Value::Object* expected_object = base::as<json::Value::Object>(expected);
    const json::Value::Array* expected_items = base::as<json::Value::Array>(expected);
    const json::Value::Array* actual_items = base::as<json::Value::Array>(actual);
    const json::Value::Number* expected_number = base::as<json::Value::Number>(expected);
    const json::Value::Number* actual_number = base::as<json::Value::Number>(actual);
    if (expected_object != nullptr && base::is<json::Value::Object>(actual)) {
        for (const auto& [name, value] : expected_object->members) {
            size_t length = path.size();
            if (!path.empty()) {
                path.push_back('.');
            }
            path += name;
            if (const json::Value* actual_value = actual.get(name)) {
                compare(value, *actual_value, path, mismatches);
            } else {
                mismatches.push_back(std::format("{} is missing", path));
            }
            path.resize(length);
        }
    } else if (expected_items != nullptr && actual_items != nullptr && expected_items->values.size() == actual_items->values.size()) {
        for (size_t index = 0; index < expected_items->values.size(); index++) {
            size_t length = path.size();
            path += std::format("[{}]", index);
            compare(expected_items->values[index], actual_items->values[index], path, mismatches);
            path.resize(length);
        }
    } else if (expected_number != nullptr && actual_number != nullptr) {
        if (std::abs(expected_number->value - actual_number->value) > 1e-6 * std::fmax(std::abs(expected_number->value), 1.0)) {
            mismatches.push_back(std::format("{} is {}, not {}", path, base::display(actual_number->value), base::display(expected_number->value)));
        }
    } else if (expected != actual) {
        mismatches.push_back(std::format("{} is {}, not {}", path, base::replace(actual.to_json(), "\n", " "), base::replace(expected.to_json(), "\n", " ")));
    }
}

} // namespace user_interface
