// Devices and readings from a test, or from a script's "Program" steps (see
// Library/UserInterface's Testing/Script.h), instead of GameInput, so that the
// demo can be tested without a gamepad, and without the user's keyboard and
// mouse. Its readings are snapshots of a device's whole state, like
// GameInput's, and it keeps the rumble that the demo sends, for checking.
//
// A script's steps, each one's "Program" member:
//
//     { "Connect": { "Id": 1, "Name": "Pad", "Input": ["Gamepad"],
//                    "Motors": ["LowFrequency", "HighFrequency"] } }
//     { "Gamepad": { "Id": 1, "LeftStick": [0.5, 0], "RightStick": [0, 0],
//                    "LeftTrigger": 0, "RightTrigger": 1, "Buttons": ["A"] } }
//     { "Keyboard": { "Id": 2, "Down": ["F1", "A"] } }
//     { "Mouse": { "Id": 3, "Motion": [10, -5], "Wheel": [0, 120],
//                  "Buttons": ["Left"] } }
//     { "Disconnect": 1 }
//
// "Gamepad", "Keyboard" and "Mouse" give that input's whole state: what's
// left out is at rest, except that "Motion" and "Wheel" are added to the
// mouse's running totals, as a mouse moves. "Input" can also have
// "Keyboard" and "Mouse", and "Motors" "LeftTrigger" and "RightTrigger".

#pragma once

#include "GameInput/GameInput.h"
#include "GameInputDemo/Devices.h"
#include "Json/Json.h"

#include <algorithm>
#include <cstddef>
#include <cstdint>
#include <expected>
#include <format>
#include <optional>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

namespace game_input_demo {

class ScriptedSource : public InputSource {
public:
    // The time now, in microseconds. The demo sets it from the frame's.
    std::uint64_t now_ = 0;

    void connect(Device device);

    void disconnect(size_t id);

    // A new reading of a connected device's state: its last state, changed
    // by `change`, which is called with the reading.
    template <class Change> std::expected<void, std::string> read(size_t id, Change&& change) {
        auto device = std::ranges::find(connected, id, &Device::id);
        if (device == connected.end()) {
            return std::unexpected(std::format("device {} isn't connected", id));
        }
        game_input::GameInputKind supported_input = device->supported_input;
        Reading reading;
        if (auto known = std::ranges::find(latest, std::optional(id), &Reading::device); known != latest.end()) {
            reading = *known;
        }
        // Later than every earlier reading, so the order stays.
        std::optional<std::uint64_t> newest;
        for (const Reading& earlier : latest) {
            newest = newest ? std::max(*newest, earlier.timestamp) : earlier.timestamp;
        }
        reading.timestamp = newest ? std::max(now_, *newest + 1) : now_;
        reading.kinds = supported_input;
        reading.device = id;
        std::forward<Change>(change)(reading);
        std::erase_if(latest, [&](const Reading& known) { return known.device == id; });
        latest.push_back(reading);
        for (game_input::GameInputKind kind : {game_input::GameInputKindGamepad, game_input::GameInputKindKeyboard, game_input::GameInputKindMouse}) {
            if ((reading.kinds & kind) == 0) {
                continue;
            }
            auto queue = std::ranges::find(queues, kind, &std::pair<game_input::GameInputKind, std::vector<Reading>>::first);
            if (queue != queues.end()) {
                queue->second.push_back(reading);
            } else {
                queues.emplace_back(kind, std::vector<Reading>{reading});
            }
        }
        return {};
    }

    // The gamepad's rumble, as the demo last set it: nothing once it's stopped.
    std::optional<game_input::GameInputRumbleParams> rumble(size_t id) const;

    // Does what a script's "Program" step says.
    std::expected<void, std::string> apply(const json::Value& step);

    std::uint64_t now() override;

    std::vector<std::pair<game_input::GameInputDeviceEvent, Device>> device_events() override;

    std::vector<Reading> readings(game_input::GameInputKind kind) override;

    void set_rumble(size_t device, const game_input::GameInputRumbleParams* params) override;

    ScriptedSource* as_scripted() override;

private:
    std::vector<Device> connected;
    std::vector<std::pair<game_input::GameInputDeviceEvent, Device>> events;
    // Readings not handed out yet, by kind of input, oldest first.
    std::vector<std::pair<game_input::GameInputKind, std::vector<Reading>>> queues;
    // Each device's newest reading, which the next one starts from.
    std::vector<Reading> latest;
    // Each gamepad's rumble, as the demo last set it.
    std::vector<std::pair<size_t, std::optional<game_input::GameInputRumbleParams>>> rumble_;
};

// The key, as GameInput reports it held, from its name, like "F1" or "A".
// Its scan code comes from the keyboard layout, and has the 0xE0 prefix for
// extended keys, like GameInput's.
std::optional<game_input::GameInputKeyState> key_state(std::string_view name);

} // namespace game_input_demo
