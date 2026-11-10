// What the demo reads, as plain data: the devices, and readings of their
// state. Input.h works on this data only, from an InputSource: GameInput, as
// Library/UserInterface reads it (GameInputSource.h), or a script or a test
// (ScriptedSource.h), so that tests can give the demo any readings, like a
// gamepad's, without one.

#pragma once

#include "Base/Windows.h"
#include "GameInput/GameInput.h"
#include "GameInputDemo/DeviceTree.h"

#include <cstddef>
#include <cstdint>
#include <optional>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

namespace game_input_demo {

class ScriptedSource;
struct Section;

// A gamepad, keyboard or mouse that GameInput reports, with everything that
// GameInput and Windows say about it.
struct Device {
    // Tells devices apart, and stays the same while a device is connected.
    size_t id = 0;
    // Its name, then its vendor and product IDs, like "Xbox Controller
    // (045E:0B13)".
    std::string label;
    std::uint16_t vendor_id = 0;
    std::uint16_t product_id = 0;
    std::uint16_t revision = 0;
    // Its HID usage page and usage, like 0x01 and 0x06 for a keyboard.
    game_input::GameInputUsage usage{};
    game_input::GameInputVersion hardware_version{};
    game_input::GameInputVersion firmware_version{};
    game_input::GameInputDeviceFamily family{};
    game_input::GameInputKind supported_input{};
    game_input::GameInputRumbleMotors rumble_motors{};
    game_input::GameInputSystemButtons system_buttons{};
    // Windows' container of the physical device, which the devices it makes
    // up share, like a mouse's keyboard for its macros.
    GUID container_id{};
    // The path of its device interface, or "" if it has none.
    std::string pnp_path;
    std::optional<std::uint32_t> xinput_index;
    std::optional<game_input::GameInputKeyboardInfo> keyboard;
    std::optional<game_input::GameInputMouseInfo> mouse;
    std::optional<game_input::GameInputGamepadInfo> gamepad;
    // Its node in Windows' device tree, then the nodes above it, or why
    // there are none, once they're looked up.
    Tree tree;

    // from_game_input, which makes a device from the library's, as it reads
    // it from GameInput, with its node in Windows' device tree, comes with
    // the parts of the demo that use Library/UserInterface.

    std::string_view family_name() const;

    std::string kinds() const;

    bool is_gamepad() const;

    // How often it says it sends mouse input, if it does.
    std::optional<std::uint32_t> mouse_sample_rate() const;

    // What the Devices card's tooltip says about it: what GameInput says,
    // then Windows' node for it, and what that's connected through.
    std::vector<Section> details() const;

    bool operator==(const Device&) const = default;
};

// Part of what the Devices card's tooltip says about a device: a title,
// and values with their names. A value can have more than one line.
struct Section {
    std::string title;
    std::vector<std::pair<std::string, std::string>> rows;

    bool operator==(const Section&) const = default;
};

std::string rumble_motor_names(game_input::GameInputRumbleMotors motors);

// A snapshot of one device's whole state, at a time: the library's, which
// it reads from GameInput, and which a script makes up too.
//
// Until the parts of the demo that use Library/UserInterface are ported, it's
// defined here, member for member as "UserInterface/Platform/InputDevices.h"
// defines user_interface::Reading. With them, it becomes that type, by
// `using user_interface::Reading;`.
struct Reading {
    // The ID of the device it's from (see Device), or nothing when GameInput
    // doesn't say.
    std::optional<size_t> device;
    // On GameInput's clock, in microseconds.
    std::uint64_t timestamp = 0;
    // Which kinds of input the reading has, like a gamepad's, or both a
    // keyboard's and a mouse's.
    game_input::GameInputKind kinds{};
    game_input::GameInputGamepadState gamepad{};
    // The keys held down.
    std::vector<game_input::GameInputKeyState> keys;
    // The buttons held, and running totals of the motion and the wheels.
    game_input::GameInputMouseState mouse{};

    bool operator==(const Reading&) const = default;
};

// Where devices and readings come from.
class InputSource {
public:
    virtual ~InputSource() = default;

    // The GameInput runtime that was loaded, for the status bar, if any.
    virtual const game_input::Runtime* runtime() const {
        return nullptr;
    }

    // The time now, on the readings' clock, in microseconds.
    virtual std::uint64_t now() = 0;

    // What happened to devices since the last call, oldest first: they
    // connected, disconnected, or changed.
    virtual std::vector<std::pair<game_input::GameInputDeviceEvent, Device>> device_events() = 0;

    // The readings with this kind of input since the last call, oldest
    // first. Input.h compares each with the same device's reading before
    // it, which it keeps itself.
    virtual std::vector<Reading> readings(game_input::GameInputKind kind) = 0;

    // Runs a gamepad's motors, or stops them with null.
    virtual void set_rumble(size_t device, const game_input::GameInputRumbleParams* params) = 0;

    // The source, if it's a script's or a test's.
    virtual ScriptedSource* as_scripted() {
        return nullptr;
    }
};

} // namespace game_input_demo
