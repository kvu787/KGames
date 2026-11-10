// What GameInputDemo makes of devices and readings, from a scripted source
// instead of GameInput, so without a gamepad, and without the user's
// keyboard and mouse: which devices it listens to, what it ignores without
// focus, its shortcuts, the rumble test, and the log.

#include "GameInputDemo/Input.h"
#include "GameInput/GameInput.h"
#include "GameInputDemo/Devices.h"
#include "GameInputDemo/ScriptedSource.h"
#include "Testing/Testing.h"
#include "Json/Json.h"

#include "Base/Text.h"

#include <algorithm>
#include <cmath>
#include <cstdint>
#include <expected>
#include <format>
#include <memory>
#include <optional>
#include <string>
#include <string_view>
#include <tuple>
#include <utility>
#include <vector>

using game_input::GameInputGamepadState;
using game_input::GameInputKindGamepad;
using game_input::GameInputKindMouse;
using game_input::GameInputRumbleHighFrequency;
using game_input::GameInputRumbleLowFrequency;
using game_input::GameInputRumbleParams;
using game_input_demo::Device;
using game_input_demo::Input;
using game_input_demo::key_name;
using game_input_demo::key_state;
using game_input_demo::Listen;
using game_input_demo::LogEntry;
using game_input_demo::Reading;
using game_input_demo::ScriptedSource;

namespace {

constexpr std::string_view PadOne = R"({ "Connect": { "Id": 1, "Name": "Pad One", "Input": ["Gamepad"],
                                        "Motors": ["LowFrequency", "HighFrequency"] } })";
constexpr std::string_view PadTwo = R"({ "Connect": { "Id": 2, "Name": "Pad Two", "Input": ["Gamepad"],
                                        "Motors": ["LowFrequency", "HighFrequency"] } })";
constexpr std::string_view Keys = R"({ "Connect": { "Id": 3, "Name": "Keys", "Input": ["Keyboard"] } })";
constexpr std::string_view Rodent = R"({ "Connect": { "Id": 4, "Name": "Rodent", "Input": ["Mouse"] } })";

// Whether `actual` has everything that `expected` has: objects match if each
// member that `expected` names matches, lists match item by item, and
// numbers match to a millionth. Each mismatch says where it is.
//
// It's Library/UserInterface's, which checks a script's "Expect" steps with
// it. It's written out here until the tests of the demo's parts that use the
// library are ported, which take the library's.
void compare(const json::Value& expected, const json::Value& actual, std::string& path, std::vector<std::string>& mismatches) {
    const json::Value::Object* expected_object = base::as<json::Value::Object>(expected);
    const json::Value::Array* expected_array = base::as<json::Value::Array>(expected);
    const json::Value::Array* actual_array = base::as<json::Value::Array>(actual);
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
    } else if (expected_array != nullptr && actual_array != nullptr && expected_array->values.size() == actual_array->values.size()) {
        for (size_t index = 0; index < expected_array->values.size(); index++) {
            size_t length = path.size();
            path += std::format("[{}]", index);
            compare(expected_array->values[index], actual_array->values[index], path, mismatches);
            path.resize(length);
        }
    } else if (expected_number != nullptr && actual_number != nullptr) {
        if (std::abs(expected_number->value - actual_number->value) > 1e-6 * std::max(std::abs(expected_number->value), 1.0)) {
            mismatches.push_back(std::format("{} is {}, not {}", path, base::display(actual_number->value), base::display(expected_number->value)));
        }
    } else if (expected != actual) {
        mismatches.push_back(std::format("{} is {}, not {}", path, base::replace(actual.to_json(), "\n", " "), base::replace(expected.to_json(), "\n", " ")));
    }
}

// The demo's input, from a test's devices and readings, with the rumble
// test's dead zone at a tenth of the sticks' deflection.
Input input() {
    return Input(std::make_unique<ScriptedSource>(), 0.1f);
}

ScriptedSource& source(Input& input) {
    ScriptedSource* scripted = input.scripted();
    CHECK(scripted != nullptr);
    return *scripted;
}

// Does what a script's "Program" step says.
void step(Input& input, std::string_view step) {
    std::expected<void, std::string> applied = source(input).apply(json::parse(step).value());
    CHECK(applied.has_value(), "{}", applied.error());
}

// Moves the source's clock on.
void wait(Input& input, std::uint64_t microseconds) {
    source(input).now_ += microseconds;
}

// Checks the demo's state, as a script's "Expect" step would.
void expect(const Input& input, std::string_view expected) {
    std::vector<std::string> problems;
    std::string path;
    compare(json::parse(expected).value(), input.state(), path, problems);
    CHECK(problems.empty(), "{}", testing::describe(problems));
}

std::vector<std::string_view> log(const Input& input) {
    std::vector<std::string_view> lines;
    for (const LogEntry& entry : input.log_) {
        lines.push_back(entry.text);
    }
    return lines;
}

// The log's newest lines, as many as the count.
std::vector<std::string_view> newest(const Input& input, size_t count) {
    std::vector<std::string_view> lines = log(input);
    CHECK(lines.size() >= count, "the log has {} lines", lines.size());
    lines.resize(count);
    return lines;
}

Listen gamepad(const Input& input, std::string_view label) {
    for (const Device& device : input.gamepads()) {
        if (device.label == label) {
            return Listen::Gamepad{device};
        }
    }
    testing::fail(std::format("no gamepad is {}", label));
}

} // namespace

TEST(listens_to_the_first_gamepad_until_one_is_picked) {
    Input input = ::input();
    for (std::string_view device : {Keys, PadOne, PadTwo}) {
        step(input, device);
    }
    input.update(true);
    // Gamepads come first.
    expect(input, R"({ "Devices": ["Pad One", "Pad Two", "Keys"], "Listen": "Pad One" })");
    CHECK(!input.listen_picked);
    CHECK_EQ(log(input)[0], "Listening to Pad One only");

    step(input, R"({ "Disconnect": 1 })");
    input.update(true);
    expect(input, R"({ "Listen": "Pad Two" })");
}

TEST(a_picked_gamepad_is_kept_while_it_is_disconnected) {
    Input input = ::input();
    step(input, PadOne);
    step(input, PadTwo);
    input.update(true);
    input.pick_listen(gamepad(input, "Pad Two"));
    step(input, R"({ "Gamepad": { "Id": 2, "Buttons": ["A"] } })");
    input.update(true);
    expect(input, R"({ "Gamepad": { "Device": "Pad Two", "Buttons": ["A"] } })");
    step(input, R"({ "Disconnect": 2 })");
    input.update(true);
    // Its view starts over, without what it held.
    expect(input, R"({ "Listen": "Pad Two", "Gamepad": { "Device": null, "Buttons": [] } })");

    step(input, PadTwo);
    step(input, R"({ "Gamepad": { "Id": 2, "Buttons": ["B"] } })");
    input.update(true);
    expect(input, R"({ "Listen": "Pad Two", "Gamepad": { "Device": "Pad Two", "Buttons": ["B"] } })");
}

TEST(ignores_gamepads_that_it_does_not_listen_to) {
    Input input = ::input();
    step(input, PadOne);
    step(input, PadTwo);
    // A gamepad that's also a keyboard, like a GameSir in Xbox 360 mode.
    step(input, R"({ "Connect": { "Id": 5, "Name": "Combo", "Input": ["Gamepad", "Keyboard"] } })");
    step(input, Keys);
    input.update(true);
    input.pick_listen(gamepad(input, "Pad One"));
    step(input, R"({ "Gamepad": { "Id": 2, "Buttons": ["A"] } })");
    step(input, R"({ "Keyboard": { "Id": 5, "Down": ["A"] } })");
    input.update(true);
    expect(input, R"({ "ReadingCount": 0, "Latest": null, "Gamepad": { "Device": "Pad One", "Buttons": [] } })");

    // Keyboards that aren't gamepads are always listened to.
    step(input, R"({ "Keyboard": { "Id": 3, "Down": ["A"] } })");
    step(input, R"({ "Gamepad": { "Id": 1, "Buttons": ["X"] } })");
    input.update(true);
    expect(input, R"({ "ReadingCount": 2, "Keyboard": { "Device": "Keys", "Keys": ["A"] },
             "Gamepad": { "Buttons": ["X"] } })");
}

TEST(a_device_that_disconnects_lets_go_of_its_keys_and_buttons) {
    Input input = ::input();
    step(input, Keys);
    step(input, Rodent);
    input.update(true);
    step(input, R"({ "Keyboard": { "Id": 3, "Down": ["A"] } })");
    step(input, R"({ "Mouse": { "Id": 4, "Buttons": ["Left"] } })");
    input.update(true);
    expect(input, R"({ "Keyboard": { "Keys": ["A"] }, "Mouse": { "Buttons": ["Left"] } })");
    step(input, R"({ "Disconnect": 3 })");
    step(input, R"({ "Disconnect": 4 })");
    input.update(true);
    expect(input, R"({ "Keyboard": { "Device": null, "Keys": [] }, "Mouse": { "Device": null, "Buttons": [] } })");
}

TEST(a_gamepad_no_longer_listened_to_lets_go_of_its_keys) {
    Input input = ::input();
    step(input, R"({ "Connect": { "Id": 5, "Name": "Combo", "Input": ["Gamepad", "Keyboard"] } })");
    step(input, PadOne);
    input.update(true);
    input.pick_listen(gamepad(input, "Combo"));
    step(input, R"({ "Keyboard": { "Id": 5, "Down": ["A"] } })");
    input.update(true);
    expect(input, R"({ "Keyboard": { "Device": "Combo", "Keys": ["A"] } })");
    // Its release isn't handled from now on.
    input.pick_listen(gamepad(input, "Pad One"));
    expect(input, R"({ "Keyboard": { "Device": null, "Keys": [] } })");
}

TEST(a_device_that_reconnects_starts_idle_even_after_a_reading_with_its_disconnect) {
    Input input = ::input();
    step(input, PadOne);
    input.update(true);
    // The reading and the disconnect come in one update, which handles the
    // disconnect first.
    step(input, R"({ "Gamepad": { "Id": 1, "Buttons": ["A"] } })");
    step(input, R"({ "Disconnect": 1 })");
    input.update(true);
    step(input, PadOne);
    step(input, R"({ "Gamepad": { "Id": 1, "Buttons": ["A"] } })");
    input.update(true);
    CHECK_EQ(log(input)[0], "Pad One: A pressed");
    expect(input, R"({ "Gamepad": { "Device": "Pad One", "Buttons": ["A"] } })");
}

TEST(no_gamepad_ignores_every_gamepad) {
    Input input = ::input();
    step(input, PadOne);
    step(input, Keys);
    input.update(true);
    input.pick_listen(Listen::NoGamepad{});
    step(input, R"({ "Gamepad": { "Id": 1, "Buttons": ["A"] } })");
    step(input, R"({ "Keyboard": { "Id": 3, "Down": ["Space"] } })");
    input.update(true);
    expect(input, R"({ "Listen": "None", "ReadingCount": 1, "Gamepad": { "Device": null } })");
    CHECK_EQ(newest(input, 2), (std::vector<std::string_view>{"Keys: Space down", "Ignoring all gamepads"}));
    // Picking the same thing again changes nothing.
    input.pick_listen(Listen::NoGamepad{});
    CHECK_EQ(log(input)[0], "Keys: Space down");
}

TEST(input_without_focus_is_dropped) {
    Input input = ::input();
    step(input, PadOne);
    input.update(false);
    step(input, R"({ "Gamepad": { "Id": 1, "Buttons": ["A"] } })");
    input.update(false);
    expect(input, R"({ "ReadingCount": 0, "Gamepad": { "Buttons": [] } })");

    // Input from before the window got focus, like the click that gave it,
    // is dropped too.
    wait(input, 1000);
    step(input, R"({ "Gamepad": { "Id": 1, "Buttons": ["B"] } })");
    wait(input, 1000);
    input.update(true);
    expect(input, R"({ "ReadingCount": 0, "Gamepad": { "Buttons": [] } })");

    wait(input, 1000);
    step(input, R"({ "Gamepad": { "Id": 1, "Buttons": ["X"] } })");
    input.update(true);
    expect(input, R"({ "ReadingCount": 1, "Gamepad": { "Buttons": ["X"] } })");
}

TEST(losing_focus_lets_go_of_held_input) {
    Input input = ::input();
    for (std::string_view device : {PadOne, Keys, Rodent}) {
        step(input, device);
    }
    input.update(true);
    step(input, R"({ "Gamepad": { "Id": 1, "LeftStick": [1, 0], "Buttons": ["A"] } })");
    step(input, R"({ "Keyboard": { "Id": 3, "Down": ["A"] } })");
    step(input, R"({ "Mouse": { "Id": 4, "Buttons": ["Left"] } })");
    input.update(true);
    expect(input, R"({ "Gamepad": { "LeftStick": [1.0, 0.0], "Buttons": ["A"] }, "Keyboard": { "Keys": ["A"] },
             "Mouse": { "Buttons": ["Left"] } })");

    // What's let go of without focus is never seen, so nothing is shown
    // held, even once the window has focus again.
    std::string_view released = R"({ "Gamepad": { "Device": "Pad One", "LeftStick": [0.0, 0.0], "Buttons": [] },
                        "Keyboard": { "Keys": [] }, "Mouse": { "Buttons": [] } })";
    input.update(false);
    expect(input, released);
    input.update(true);
    expect(input, released);
}

TEST(input_while_minimized_is_dropped_and_restoring_counts_as_getting_focus) {
    Input input = ::input();
    step(input, PadOne);
    step(input, Keys);
    input.update(true);
    step(input, R"({ "Keyboard": { "Id": 3, "Down": ["F1"] } })");
    step(input, R"({ "Keyboard": { "Id": 3, "Down": [] } })");
    step(input, R"({ "Gamepad": { "Id": 1, "LeftStick": [1, 0] } })");
    input.update(true);
    CHECK_EQ(source(input).rumble(1).value().lowFrequency, 1.0f);

    // Minimized, the window doesn't have focus, and the main loop says so.
    // The motors stop.
    wait(input, 1000);
    input.update(false);
    CHECK(!source(input).rumble(1).has_value());

    // F1, pressed in another window while the demo is minimized, doesn't
    // turn the rumble test off once it's restored.
    wait(input, 1000);
    step(input, R"({ "Keyboard": { "Id": 3, "Down": ["F1"] } })");
    step(input, R"({ "Keyboard": { "Id": 3, "Down": [] } })");
    wait(input, 1000);
    input.update(true);
    CHECK(input.rumble_enabled);
    std::vector<std::string_view> lines = log(input);
    size_t presses = static_cast<size_t>(std::ranges::count_if(lines, [](std::string_view line) { return line.contains("F1"); }));
    CHECK_EQ(presses, 2, "{}", testing::describe(lines));
    // The sticks count as at rest until they move with focus.
    CHECK(!source(input).rumble(1).has_value());
}

TEST(the_click_that_gives_focus_does_only_that) {
    Input input = ::input();
    step(input, Rodent);
    input.update(false);
    // The press that gives the window focus comes just before it has it.
    wait(input, 1000);
    step(input, R"({ "Mouse": { "Id": 4, "Buttons": ["Left"] } })");
    wait(input, 1000);
    input.update(true);

    // While it's still held, the button doesn't show as held, but motion does.
    wait(input, 1000);
    step(input, R"({ "Mouse": { "Id": 4, "Motion": [5, 0], "Buttons": ["Left"] } })");
    input.update(true);
    expect(input, R"json({ "Mouse": { "Buttons": [], "Delta": [5, 0] }, "Latest": ["moved (+5, +0)"] })json");

    // Letting go of it isn't reported either.
    step(input, R"({ "Mouse": { "Id": 4 } })");
    input.update(true);
    expect(input, R"json({ "Mouse": { "Buttons": [] }, "Latest": ["moved (+5, +0)"] })json");
    std::vector<std::string_view> lines = log(input);
    CHECK(!std::ranges::any_of(lines, [](std::string_view line) { return line.contains("Left"); }), "{}", testing::describe(lines));

    // Pressed again, it's a click like any other.
    step(input, R"({ "Mouse": { "Id": 4, "Buttons": ["Left"] } })");
    input.update(true);
    expect(input, R"({ "Mouse": { "Buttons": ["Left"] }, "Latest": ["Left pressed"] })");
}

TEST(keys_held_when_the_window_gets_focus_are_ignored_until_let_go_of) {
    Input input = ::input();
    step(input, Keys);
    input.update(true);
    // Held from before the window loses focus, through it getting it back.
    step(input, R"({ "Keyboard": { "Id": 3, "Down": ["Shift"] } })");
    input.update(true);
    input.update(false);
    // F1 held down in another window, and the Alt of the Alt+Tab that
    // switches back to the demo.
    wait(input, 1000);
    step(input, R"({ "Keyboard": { "Id": 3, "Down": ["Shift", "F1", "Alt"] } })");
    wait(input, 1000);
    input.update(true);
    expect(input, R"({ "Keyboard": { "Keys": [] } })");

    wait(input, 1000);
    step(input, R"({ "Keyboard": { "Id": 3, "Down": ["Shift", "F1", "Alt", "A"] } })");
    input.update(true);
    expect(input, R"({ "Keyboard": { "Keys": ["A"] }, "Latest": ["A down"], "RumbleTest": false })");
    step(input, R"({ "Keyboard": { "Id": 3, "Down": ["F1"] } })");
    input.update(true);
    expect(input, R"({ "Keyboard": { "Keys": [] }, "Latest": ["A up"] })");
    step(input, R"({ "Keyboard": { "Id": 3, "Down": [] } })");
    input.update(true);
    expect(input, R"({ "Latest": ["A up"], "RumbleTest": false })");

    // Once let go of, F1 works again.
    step(input, R"({ "Keyboard": { "Id": 3, "Down": ["F1"] } })");
    input.update(true);
    expect(input, R"({ "Keyboard": { "Keys": ["F1"] }, "Latest": ["F1 down"], "RumbleTest": true })");
}

TEST(gamepad_buttons_held_when_the_window_gets_focus_are_ignored_until_let_go_of) {
    Input input = ::input();
    step(input, PadOne);
    input.update(false);
    wait(input, 1000);
    step(input, R"({ "Gamepad": { "Id": 1, "LeftStick": [1, 0], "Buttons": ["A"] } })");
    wait(input, 1000);
    input.update(true);

    // The stick shows as usual, without A.
    wait(input, 1000);
    step(input, R"({ "Gamepad": { "Id": 1, "LeftStick": [0.5, 0], "Buttons": ["A", "B"] } })");
    input.update(true);
    expect(input, R"json({ "Gamepad": { "LeftStick": [0.5, 0.0], "Buttons": ["B"] },
             "Latest": ["B pressed", "left stick (+0.500, +0.000)"] })json");
    step(input, R"({ "Gamepad": { "Id": 1, "LeftStick": [0.5, 0] } })");
    input.update(true);
    expect(input, R"({ "Gamepad": { "Buttons": [] }, "Latest": ["B released"] })");
    step(input, R"({ "Gamepad": { "Id": 1, "LeftStick": [0.5, 0], "Buttons": ["A"] } })");
    input.update(true);
    expect(input, R"({ "Gamepad": { "Buttons": ["A"] }, "Latest": ["A pressed"] })");
}

TEST(a_gamepad_listened_to_shows_none_of_its_input_from_without_focus) {
    Input input = ::input();
    step(input, PadOne);
    step(input, PadTwo);
    input.update(false);
    wait(input, 1000);
    step(input, R"({ "Gamepad": { "Id": 2, "LeftStick": [1, 0], "Buttons": ["A"] } })");
    input.update(false);
    // The demo switches to Pad Two while it doesn't have focus, and shows
    // it at rest.
    step(input, R"({ "Disconnect": 1 })");
    input.update(false);
    std::string_view idle = R"({ "Listen": "Pad Two", "Gamepad": { "Device": "Pad Two", "LeftStick": [0.0, 0.0], "Buttons": [] } })";
    expect(input, idle);
    wait(input, 1000);
    input.update(true);
    expect(input, idle);

    // With focus, a gamepad picked shows its last reading, if the window had
    // focus for it.
    step(input, PadOne);
    input.update(true);
    step(input, R"({ "Gamepad": { "Id": 1, "LeftStick": [0, 1], "Buttons": ["B"] } })");
    input.update(true);
    input.pick_listen(gamepad(input, "Pad One"));
    expect(input, R"({ "Gamepad": { "Device": "Pad One", "LeftStick": [0.0, 1.0], "Buttons": ["B"] } })");
    input.pick_listen(gamepad(input, "Pad Two"));
    expect(input, idle);
}

TEST(readings_of_a_disconnected_gamepad_are_ignored) {
    Input input = ::input();
    step(input, PadOne);
    input.update(true);
    // Its last reading comes in the same frame as its disconnect.
    step(input, R"({ "Gamepad": { "Id": 1, "LeftStick": [1, 0], "Buttons": ["A"] } })");
    step(input, R"({ "Disconnect": 1 })");
    input.update(true);
    expect(input, R"({ "ReadingCount": 0, "Listen": "Pad One", "Gamepad": { "Device": null, "Buttons": [] }, "Rumble": [] })");
    CHECK_EQ(log(input)[0], "Disconnected: Pad One");
}

TEST(readings_are_compared_with_the_devices_own_last_one) {
    Input input = ::input();
    step(input, Keys);
    step(input, Rodent);
    input.update(true);
    // GameInput forgets readings after a short while, so after a pause, it
    // no longer has the one before. The demo keeps it, so a key held through
    // the pause isn't pressed again, and the first motion after it moves.
    step(input, R"({ "Keyboard": { "Id": 3, "Down": ["Shift"] } })");
    step(input, R"({ "Mouse": { "Id": 4 } })");
    input.update(true);
    wait(input, 10'000'000);
    step(input, R"({ "Keyboard": { "Id": 3, "Down": ["Shift", "A"] } })");
    step(input, R"({ "Mouse": { "Id": 4, "Motion": [3, 4] } })");
    input.update(true);
    CHECK_EQ(newest(input, 2), (std::vector<std::string_view>{"Rodent: moved (+3, +4)", "Keys: A down"}));

    // Readings without focus count too, so a key let go of meanwhile isn't
    // let go of again.
    input.update(false);
    wait(input, 1000);
    step(input, R"({ "Keyboard": { "Id": 3, "Down": [] } })");
    wait(input, 1000);
    input.update(true);
    step(input, R"({ "Keyboard": { "Id": 3, "Down": ["B"] } })");
    input.update(true);
    expect(input, R"({ "Latest": ["B down"] })");

    // A device that reconnects starts at rest.
    step(input, R"({ "Disconnect": 3 })");
    step(input, Keys);
    input.update(true);
    step(input, R"({ "Keyboard": { "Id": 3, "Down": ["B"] } })");
    input.update(true);
    CHECK_EQ(newest(input, 2), (std::vector<std::string_view>{"Keys: B down", "Connected: Keys [keyboard]"}));
}

TEST(the_rumble_test_drives_the_motors_past_the_dead_zone) {
    Input input = ::input();
    step(input, PadOne);
    step(input, Keys);
    input.update(true);
    step(input, R"({ "Keyboard": { "Id": 3, "Down": ["F1"] } })");
    step(input, R"({ "Keyboard": { "Id": 3, "Down": [] } })");
    input.update(true);
    CHECK(input.rumble_enabled);

    // Halfway from the dead zone's edge to the stick's, and inside it.
    step(input, R"({ "Gamepad": { "Id": 1, "LeftStick": [0.55, 0], "RightStick": [0, 0.05] } })");
    input.update(true);
    GameInputRumbleParams rumble = source(input).rumble(1).value();
    CHECK(std::abs(rumble.lowFrequency - 0.5f) < 1e-6f, "{}", rumble.lowFrequency);
    CHECK_EQ(rumble.highFrequency, 0.0f);

    // Diagonal deflection counts, up to full.
    step(input, R"({ "Gamepad": { "Id": 1, "LeftStick": [0, 0], "RightStick": [-0.8, 0.8] } })");
    input.update(true);
    rumble = source(input).rumble(1).value();
    CHECK_EQ(std::pair(rumble.lowFrequency, rumble.highFrequency), std::pair(0.0f, 1.0f));
    CHECK_EQ(std::pair(rumble.leftTrigger, rumble.rightTrigger), std::pair(0.0f, 0.0f));

    // Without focus the motors stop, and they stay stopped with focus
    // until the sticks move again.
    input.update(false);
    CHECK(!source(input).rumble(1).has_value());
    input.update(true);
    CHECK(!source(input).rumble(1).has_value());
    step(input, R"({ "Gamepad": { "Id": 1, "RightStick": [1, 0] } })");
    input.update(true);
    CHECK_EQ(source(input).rumble(1).value().highFrequency, 1.0f);

    // Off, the test stops the motors, and the sticks no longer drive them.
    step(input, R"({ "Keyboard": { "Id": 3, "Down": ["F1"] } })");
    input.update(true);
    CHECK(!input.rumble_enabled);
    CHECK_EQ(source(input).rumble(1).value().highFrequency, 0.0f);
}

TEST(gamepads_no_longer_listened_to_stop_rumbling) {
    Input input = ::input();
    step(input, PadOne);
    step(input, PadTwo);
    step(input, Keys);
    input.update(true);
    step(input, R"({ "Keyboard": { "Id": 3, "Down": ["F1"] } })");
    step(input, R"({ "Gamepad": { "Id": 1, "LeftStick": [1, 0] } })");
    input.update(true);
    CHECK_EQ(source(input).rumble(1).value().lowFrequency, 1.0f);

    input.pick_listen(gamepad(input, "Pad Two"));
    CHECK(!source(input).rumble(1).has_value());
    expect(input, R"({ "Rumble": [] })");
}

TEST(shortcuts_run_once_for_each_press) {
    Input input = ::input();
    step(input, Keys);
    input.update(true);
    step(input, R"({ "Keyboard": { "Id": 3, "Down": ["F1"] } })");
    input.update(true);
    CHECK(input.rumble_enabled);
    // Held while another key goes down.
    step(input, R"({ "Keyboard": { "Id": 3, "Down": ["F1", "A"] } })");
    input.update(true);
    CHECK(input.rumble_enabled);
    // Let go of and pressed again, between two frames.
    step(input, R"({ "Keyboard": { "Id": 3, "Down": [] } })");
    step(input, R"({ "Keyboard": { "Id": 3, "Down": ["F1"] } })");
    input.update(true);
    CHECK(!input.rumble_enabled);

    // F11 is the library's, which reads the same readings, so to the demo
    // it's only a key, which is logged.
    step(input, R"({ "Keyboard": { "Id": 3, "Down": ["F11"] } })");
    input.update(true);
    CHECK(log(input)[0].starts_with("Keys: F11 down"), "{}", testing::describe(log(input)));

    // The log is cleared after the key press is logged.
    step(input, R"({ "Keyboard": { "Id": 3, "Down": ["F2"] } })");
    input.update(true);
    CHECK(input.log_.empty());
}

TEST(keys_are_named_like_the_keyboard_layout_names_them) {
    for (std::string_view name : {"A", "F1", "Left", "Delete"}) {
        CHECK_EQ(key_name(key_state(name).value()), name);
    }
    // The arrow keys have the prefix that sets them apart from the keypad's.
    CHECK_EQ(key_state("Left").value().scanCode, 0xE04B);
    CHECK_EQ(key_state("A").value().codePoint, static_cast<std::uint32_t>(U'a'));
    CHECK(!key_state("Nope").has_value());
}

TEST(runs_of_motion_share_a_log_line) {
    Input input = ::input();
    step(input, PadOne);
    step(input, Rodent);
    input.update(true);
    // The first reading is the mouse's starting point.
    step(input, R"({ "Mouse": { "Id": 4 } })");
    step(input, R"({ "Mouse": { "Id": 4, "Motion": [10, -5] } })");
    step(input, R"({ "Mouse": { "Id": 4, "Motion": [3, 4] } })");
    input.update(true);
    expect(input, R"({ "ReadingCount": 3, "Mouse": { "Device": "Rodent", "Delta": [3, 4] } })");
    CHECK_EQ(input.log_[0].text, "Rodent: moved (+3, +4)");
    CHECK_EQ(input.log_[0].count, 2);

    // A press starts a line of its own, and so does motion after it.
    step(input, R"({ "Mouse": { "Id": 4, "Buttons": ["Left"] } })");
    step(input, R"({ "Mouse": { "Id": 4, "Wheel": [0, -120] } })");
    step(input, R"({ "Gamepad": { "Id": 1, "LeftStick": [0.25, 0] } })");
    step(input, R"({ "Gamepad": { "Id": 1, "LeftStick": [0.5, 0] } })");
    input.update(true);
    CHECK_EQ(newest(input, 3), (std::vector<std::string_view>{
                                   "Pad One: left stick (+0.500, +0.000)",
                                   "Rodent: Left released, wheel -120",
                                   "Rodent: Left pressed",
                               }));
    CHECK_EQ(input.log_[0].count, 2);
    expect(input, R"({ "Mouse": { "Delta": [0, 0], "Wheel": [0, -120] } })");
}

TEST(readings_are_handled_in_the_order_they_came) {
    Input input = ::input();
    step(input, PadOne);
    step(input, Keys);
    input.update(true);
    step(input, R"({ "Keyboard": { "Id": 3, "Down": ["A"] } })");
    step(input, R"({ "Gamepad": { "Id": 1, "Buttons": ["A"] } })");
    step(input, R"({ "Keyboard": { "Id": 3, "Down": [] } })");
    input.update(true);
    CHECK_EQ(newest(input, 3), (std::vector<std::string_view>{"Keys: A up", "Pad One: A pressed", "Keys: A down"}));
    expect(input, R"({ "ReadingCount": 3, "Latest": ["A up"] })");
}

TEST(scripted_readings_are_whole_states) {
    ScriptedSource source;
    auto step = [](ScriptedSource& source, std::string_view step) {
        std::expected<void, std::string> applied = source.apply(json::parse(step).value());
        CHECK(applied.has_value(), "{}", applied.error());
    };
    step(source, PadOne);
    step(source, Rodent);
    // What a step leaves out is at rest.
    step(source, R"({ "Gamepad": { "Id": 1, "LeftStick": [0.5, 0], "Buttons": ["A"] } })");
    step(source, R"({ "Gamepad": { "Id": 1, "RightTrigger": 1 } })");
    GameInputGamepadState state = source.readings(GameInputKindGamepad)[1].gamepad;
    CHECK_EQ(std::tuple(state.leftThumbstickX, static_cast<std::int32_t>(state.buttons), state.rightTrigger), std::tuple(0.0f, 0, 1.0f));

    // A mouse's motion adds up.
    step(source, R"({ "Mouse": { "Id": 4, "Motion": [10, 0] } })");
    step(source, R"({ "Mouse": { "Id": 4, "Motion": [10, 0] } })");
    std::vector<Reading> readings = source.readings(GameInputKindMouse);
    std::vector<std::pair<std::optional<size_t>, std::int64_t>> totals;
    for (const Reading& reading : readings) {
        totals.emplace_back(reading.device, reading.mouse.positionX);
    }
    CHECK_EQ(totals, (std::vector<std::pair<std::optional<size_t>, std::int64_t>>{{4, 10}, {4, 20}}));
    // Later readings have later timestamps, even at the same time.
    CHECK(readings[1].timestamp > readings[0].timestamp);
    CHECK(source.readings(GameInputKindMouse).empty());

    std::vector<std::pair<game_input::GameInputDeviceEvent, Device>> events = source.device_events();
    CHECK_EQ(events.size(), 2);
    CHECK_EQ(events[0].second.label, "Pad One");
    CHECK_EQ(events[0].second.rumble_motors, GameInputRumbleLowFrequency | GameInputRumbleHighFrequency);
}

TEST(scripted_steps_say_what_is_wrong_with_them) {
    ScriptedSource source;
    auto problem = [&](std::string_view step) {
        std::expected<void, std::string> applied = source.apply(json::parse(step).value());
        CHECK(!applied.has_value(), "{} is done", step);
        return applied.error();
    };
    CHECK_EQ(problem(R"({ "Gamepad": { "Id": 9 } })"), "device 9 isn't connected");
    CHECK_EQ(problem(R"({ "Connect": { "Id": 1 } })"), "Connect needs a Name");
    CHECK_EQ(problem(R"({ "Connect": { "Id": 0, "Name": "Pad" } })"), "a device's Id must be a whole number from 1");
    CHECK_EQ(problem(R"({ "Connect": { "Id": 1, "Name": "Pad", "Input": ["Wheel"] } })"), "Input can't have Wheel");
    CHECK_EQ(problem(R"({ "Keyboard": { "Id": 1, "Down": ["Nope"] } })"), "Nope isn't a key");
    CHECK_EQ(problem(R"({ "Gamepad": { "Id": 1, "LeftStick": [1] } })"), "LeftStick must be [x, y], not a list of 1");
    CHECK(problem(R"({ "Mouse": { "Id": 1, "Buttons": "Left" } })").starts_with("Buttons must be a list of names, not "));
    CHECK(problem(R"({ "Disconnect": "Pad" })").starts_with("Disconnect needs a device's Id, not "));
    CHECK_EQ(problem(R"({ "Wave": 1 })"), "Wave isn't a step for the demo");
    CHECK(problem(R"({ "Disconnect": 1, "Wave": 1 })").starts_with("a step for the demo has one of "));
    CHECK(problem("[]").starts_with("a step for the demo must be an object, not "));
}
