#include "GameInputDemo/GameInputSource.h"

#include <algorithm>
#include <span>

namespace game_input_demo {

using namespace game_input;

GameInputSource::GameInputSource(std::shared_ptr<user_interface::Devices> input_devices) : devices(std::move(input_devices)), runtime_(devices->runtime()) {}

Device GameInputSource::describe(const user_interface::Device& device, bool changed) {
    if (changed) {
        std::erase_if(described, [&](const Device& known) { return known.id == device.id; });
    }
    if (auto known = std::ranges::find(described, device.id, &Device::id); known != described.end()) {
        return *known;
    }
    Device made = Device::from_game_input(device, Tree::look_up(device.pnp_path));
    described.push_back(made);
    return made;
}

const Runtime* GameInputSource::runtime() const {
    return &runtime_;
}

std::uint64_t GameInputSource::now() {
    return devices->now();
}

std::vector<std::pair<GameInputDeviceEvent, Device>> GameInputSource::device_events() {
    if (devices->frame() == events_frame) {
        return {};
    }
    events_frame = devices->frame();
    std::span<const std::pair<GameInputDeviceEvent, user_interface::Device>> events = devices->device_events();
    std::vector<std::pair<GameInputDeviceEvent, Device>> handed;
    handed.reserve(events.size());
    for (const auto& [event, device] : events) {
        // A device that connects, or whose keyboard layout or XInput
        // index changes, is described again.
        bool changed = event == GameInputDeviceEventConnected || event == GameInputDeviceEventKeyboardLayoutUpdated || event == GameInputDeviceEventXInputIndexUpdated;
        handed.emplace_back(event, describe(device, changed));
    }
    return handed;
}

std::vector<Reading> GameInputSource::readings(GameInputKind kind) {
    std::uint64_t frame = devices->frame();
    auto taken = std::ranges::find(readings_frames, kind, &std::pair<GameInputKind, std::uint64_t>::first);
    if (taken != readings_frames.end()) {
        if (taken->second == frame) {
            return {};
        }
        taken->second = frame;
    } else {
        readings_frames.emplace_back(kind, frame);
    }
    std::span<const Reading> read = devices->readings(kind);
    return std::vector<Reading>(read.begin(), read.end());
}

void GameInputSource::set_rumble(size_t device, const GameInputRumbleParams* params) {
    devices->set_rumble(device, params);
}

} // namespace game_input_demo
