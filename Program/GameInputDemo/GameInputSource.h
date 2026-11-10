// Devices and readings from GameInput, as Library/UserInterface reads them
// for every program: once a frame, right before the frame, with every reading
// since the last one. The demo shows the same readings that the interface
// makes its events from, and runs gamepads' motors through the library.
//
// Input.h keeps each device's last reading itself, to say what changed.

#pragma once

#include "GameInput/GameInput.h"
#include "GameInputDemo/DeviceTree.h"
#include "GameInputDemo/Devices.h"
#include "UserInterface/Platform/InputDevices.h"

#include <cstddef>
#include <cstdint>
#include <memory>
#include <utility>
#include <vector>

namespace game_input_demo {

class GameInputSource : public InputSource {
public:
    explicit GameInputSource(std::shared_ptr<user_interface::Devices> devices);

    const game_input::Runtime* runtime() const override;

    std::uint64_t now() override;

    std::vector<std::pair<game_input::GameInputDeviceEvent, Device>> device_events() override;

    std::vector<Reading> readings(game_input::GameInputKind kind) override;

    void set_rumble(size_t device, const game_input::GameInputRumbleParams* params) override;

private:
    // The demo's data for a device, with its node in the device tree.
    Device describe(const user_interface::Device& device, bool changed);

    std::shared_ptr<user_interface::Devices> devices;
    game_input::Runtime runtime_;
    // The last frame whose device events, and whose readings of each kind,
    // were handed out, so that each is handed out once.
    std::uint64_t events_frame = 0;
    std::vector<std::pair<game_input::GameInputKind, std::uint64_t>> readings_frames;
    // Every device seen, with its node in Windows' device tree, which is
    // looked up once.
    std::vector<Device> described;
};

} // namespace game_input_demo
