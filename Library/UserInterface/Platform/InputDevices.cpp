#include "UserInterface/Platform/InputDevices.h"

#include "Base/Panic.h"
#include "Base/Text.h"
#include "UserInterface/Platform/Clock.h"

#include <algorithm>
#include <format>
#include <intrin.h>
#include <iterator>
#include <limits>
#include <mutex>
#include <string_view>

namespace user_interface {

namespace input_devices_detail {

// The device events that GameInput's callback queued, with each one's
// device.
struct DeviceQueue {
    std::mutex mutex;
    std::vector<std::pair<game_input::GameInputDeviceEvent, Microsoft::WRL::ComPtr<game_input::IGameInputDevice>>> events;
};

} // namespace input_devices_detail

namespace {

using input_devices_detail::DeviceQueue;
using Microsoft::WRL::ComPtr;

// GameInput's device callback runs on one of its threads. It only queues the
// event, and the main thread takes it in `read`.
void CALLBACK on_device_event(game_input::GameInputCallbackToken, void* context, game_input::GameInputDeviceEvent event, game_input::IGameInputDevice* device) {
    DeviceQueue* queue = static_cast<DeviceQueue*>(context);
    if (device != nullptr) {
        std::lock_guard lock(queue->mutex);
        // The queue's pointer holds a reference of its own.
        queue->events.emplace_back(event, device);
    }
}

// How often GameInput's clock is checked against the counter's, in seconds.
constexpr double CalibrationSeconds = 10.0;

// What the system says a failed call's HRESULT means, or "" if it says
// nothing.
std::string message(HRESULT code) {
    wchar_t* buffer = nullptr;
    DWORD length = FormatMessageW(FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS, nullptr, static_cast<DWORD>(code), 0,
                                  reinterpret_cast<wchar_t*>(&buffer), 0, nullptr);
    std::string text;
    if (buffer != nullptr) {
        text = base::trim(base::narrow(std::wstring_view(buffer, length)));
        LocalFree(buffer);
    }
    return text;
}

// A span of the counter's ticks in microseconds: the product in 128 bits,
// which it can take more than 64 of, divided toward zero, and then the low
// 64 bits of that.
std::int64_t microseconds_from_ticks(std::int64_t ticks) {
    std::int64_t high = 0;
    std::uint64_t low = static_cast<std::uint64_t>(_mul128(ticks, 1'000'000, &high));
    bool negative = high < 0;
    std::uint64_t size_high = static_cast<std::uint64_t>(high);
    std::uint64_t size_low = low;
    if (negative) {
        size_low = ~low + 1;
        size_high = ~size_high + (size_low == 0 ? 1 : 0);
    }
    std::uint64_t frequency = static_cast<std::uint64_t>(clock::frequency());
    std::uint64_t remainder = 0;
    std::uint64_t quotient = _udiv128(size_high % frequency, size_low, frequency, &remainder);
    return static_cast<std::int64_t>(negative ? ~quotient + 1 : quotient);
}

// A string that a GameInput method writes as UTF-8 with a terminating zero,
// after saying how big it is when given an empty buffer.
template <class Get> std::string utf8(Get&& get) {
    std::string buffer(get(0, nullptr), '\0');
    get(static_cast<std::uint32_t>(buffer.size()), buffer.empty() ? nullptr : buffer.data());
    std::string text = base::from_utf8_lossy(buffer);
    while (!text.empty() && text.back() == '\0') {
        text.pop_back();
    }
    return text;
}

// A device's data, from GameInput.
Device describe(game_input::IGameInputDevice* device) {
    std::string name = utf8([&](std::uint32_t size, char* buffer) { return device->GetDisplayName(size, buffer); });
    if (name.empty()) {
        name = "Unnamed device";
    }
    std::uint16_t vendor_id = device->GetVendorId();
    std::uint16_t product_id = device->GetProductId();
    game_input::GameInputUsage usage{};
    device->GetUsage(&usage);
    game_input::GameInputVersion hardware_version{};
    device->GetHardwareVersion(&hardware_version);
    game_input::GameInputVersion firmware_version{};
    device->GetFirmwareVersion(&firmware_version);
    GUID container_id{};
    device->GetContainerId(&container_id);
    std::uint32_t xinput_index = 0;
    game_input::GameInputKeyboardInfo keyboard{};
    game_input::GameInputMouseInfo mouse{};
    game_input::GameInputGamepadInfo gamepad{};
    Device described;
    described.id = reinterpret_cast<size_t>(device);
    described.label = std::format("{} ({:04X}:{:04X})", name, vendor_id, product_id);
    described.name = std::move(name);
    described.vendor_id = vendor_id;
    described.product_id = product_id;
    described.revision = device->GetRevisionNumber();
    described.usage = usage;
    described.hardware_version = hardware_version;
    described.firmware_version = firmware_version;
    described.family = device->GetDeviceFamily();
    described.supported_input = device->GetSupportedInput();
    described.rumble_motors = device->GetSupportedRumbleMotors();
    described.system_buttons = device->GetSupportedSystemButtons();
    described.container_id = container_id;
    described.pnp_path = utf8([&](std::uint32_t size, char* buffer) { return device->GetPnpPath(size, buffer); });
    if (device->GetXInputDeviceIndex(&xinput_index)) {
        described.xinput_index = xinput_index;
    }
    if (device->GetKeyboardInfo(&keyboard)) {
        described.keyboard = keyboard;
    }
    if (device->GetMouseInfo(&mouse)) {
        described.mouse = mouse;
    }
    if (device->GetGamepadInfo(&gamepad)) {
        described.gamepad = gamepad;
    }
    return described;
}

} // namespace

std::expected<Devices, std::string> Devices::create() {
    auto created = game_input::create();
    if (!created) {
        return std::unexpected(std::move(created.error()));
    }
    auto& [input, runtime] = *created;
    std::unique_ptr<DeviceQueue> queue = std::make_unique<DeviceQueue>();
    game_input::GameInputCallbackToken callback_token = 0;
    HRESULT registered = input->RegisterDeviceCallback(nullptr, game_input::GameInputKindGamepad | game_input::GameInputKindKeyboard | game_input::GameInputKindMouse, queue.get(),
                                                       on_device_event, &callback_token);
    if (FAILED(registered)) {
        return std::unexpected(std::format("GameInput couldn't listen for devices: {} (0x{:08X})", message(registered), static_cast<std::uint32_t>(registered)));
    }
    // By default, GameInput gives a process input only while it has
    // focus, on GameInput's own timing. The reader follows the devices'
    // state without focus too, so that keys and buttons already held when
    // the window gets focus, like the click that gives it focus, count
    // only once they're let go of, and it drops what came without focus
    // itself, by the window's own focus changes (Input/Reader.h).
    input->EnableBackgroundInput(true);
    Devices devices(std::move(input), std::move(runtime), callback_token, std::move(queue));
    devices.calibrate();
    return devices;
}

Devices::Devices(Microsoft::WRL::ComPtr<game_input::IGameInput> game_input, game_input::Runtime runtime, game_input::GameInputCallbackToken callback_token,
                 std::unique_ptr<input_devices_detail::DeviceQueue> queue) :
    game_input(std::move(game_input)), runtime_(std::move(runtime)), callback_token(callback_token), queue(std::move(queue)) {}

Devices::~Devices() {
    if (!game_input) {
        // Moved from: another Devices has the callback and its queue.
        return;
    }
    // Once it returns, the callback doesn't run again, so its queue
    // can go.
    game_input->UnregisterCallback(callback_token);
    queue.reset();
    game_input.Reset();
}

Devices::Devices(Devices&& other) noexcept = default;

Devices& Devices::operator=(Devices&& other) noexcept {
    if (this != &other) {
        std::destroy_at(this);
        std::construct_at(this, std::move(other));
    }
    return *this;
}

const game_input::Runtime& Devices::runtime() const {
    return runtime_;
}

std::uint64_t Devices::now() const {
    return game_input->GetCurrentTimestamp();
}

std::int64_t Devices::qpc(std::uint64_t timestamp) const {
    return clock::ticks_from_microseconds(static_cast<std::int64_t>(timestamp) - epoch);
}

void Devices::calibrate() {
    std::int64_t best_span = std::numeric_limits<std::int64_t>::max();
    std::int64_t best_middle = 0;
    std::int64_t best_game_input = 0;
    for (int attempt = 0; attempt < 8; attempt++) {
        std::int64_t before = clock::now();
        std::int64_t game_input_now = static_cast<std::int64_t>(now());
        std::int64_t after = clock::now();
        if (after - before < best_span) {
            best_span = after - before;
            best_middle = (before + after) / 2;
            best_game_input = game_input_now;
        }
    }
    std::int64_t middle_microseconds = microseconds_from_ticks(best_middle);
    epoch = best_game_input - middle_microseconds;
    calibrated = best_middle;
}

void Devices::read() {
    frame_ += 1;
    if (clock::seconds(clock::now() - calibrated) >= CalibrationSeconds) {
        calibrate();
    }
    device_events_.clear();
    std::vector<std::pair<game_input::GameInputDeviceEvent, Object>> events;
    {
        std::lock_guard lock(queue->mutex);
        events.assign(std::make_move_iterator(queue->events.begin()), std::make_move_iterator(queue->events.end()));
        queue->events.clear();
    }
    for (const auto& [event, object] : events) {
        // A device that connects, or whose keyboard layout or XInput
        // index changes, is described again.
        if (event == game_input::GameInputDeviceEventConnected || event == game_input::GameInputDeviceEventKeyboardLayoutUpdated ||
            event == game_input::GameInputDeviceEventXInputIndexUpdated) {
            std::erase_if(known, [&](const std::pair<Object, Device>& entry) { return entry.first == object; });
        }
        Device described = device(object.Get());
        device_events_.emplace_back(event, std::move(described));
    }
    for (size_t index = 0; index < Kinds.size(); index++) {
        std::vector<ReadingObject> objects = new_readings(Kinds[index]);
        std::vector<Reading> readings;
        readings.reserve(objects.size());
        for (const ReadingObject& object : objects) {
            readings.push_back(reading(object.Get()));
        }
        readings_[index] = std::move(readings);
    }
}

std::uint64_t Devices::frame() const {
    return frame_;
}

std::span<const Reading> Devices::readings(game_input::GameInputKind kind) const {
    auto known_kind = std::ranges::find(Kinds, kind);
    if (known_kind == Kinds.end()) {
        return {};
    }
    return readings_[static_cast<size_t>(known_kind - Kinds.begin())];
}

std::span<const std::pair<game_input::GameInputDeviceEvent, Device>> Devices::device_events() const {
    return device_events_;
}

void Devices::set_rumble(size_t device, const game_input::GameInputRumbleParams* params) {
    for (const auto& [object, data] : known) {
        if (data.id == device) {
            object->SetRumbleState(params);
            break;
        }
    }
}

Device Devices::device(game_input::IGameInputDevice* device) {
    for (const auto& [object, data] : known) {
        if (object.Get() == device) {
            return data;
        }
    }
    Device described = describe(device);
    known.emplace_back(device, described);
    return described;
}

Reading Devices::reading(game_input::IGameInputReading* reading) {
    game_input::GameInputKind kinds = reading->GetInputKind();
    // The device's object is kept, even if its device event hasn't
    // come yet, so that no other device can get its address, and with
    // it its ID.
    Object object;
    reading->GetDevice(&object);
    std::optional<size_t> id;
    if (object) {
        id = device(object.Get()).id;
    }
    Reading data{
        .device = id,
        .timestamp = reading->GetTimestamp(),
        .kinds = kinds,
    };
    if ((kinds & game_input::GameInputKindGamepad) != 0) {
        reading->GetGamepadState(&data.gamepad);
    }
    if ((kinds & game_input::GameInputKindKeyboard) != 0) {
        std::vector<game_input::GameInputKeyState> keys(reading->GetKeyCount());
        std::uint32_t count = reading->GetKeyState(static_cast<std::uint32_t>(keys.size()), keys.data());
        if (count < keys.size()) {
            keys.resize(count);
        }
        data.keys = std::move(keys);
    }
    if ((kinds & game_input::GameInputKindMouse) != 0) {
        reading->GetMouseState(&data.mouse);
    }
    return data;
}

std::vector<Devices::ReadingObject> Devices::new_readings(game_input::GameInputKind kind) {
    std::vector<ReadingObject> readings;
    auto last_of_kind = std::ranges::find_if(last_readings, [&](const std::pair<game_input::GameInputKind, ReadingObject>& entry) { return entry.first == kind; });
    if (last_of_kind == last_readings.end()) {
        // The first time, start from the newest reading.
        if (std::expected<ReadingObject, HRESULT> current = current_reading(kind)) {
            last_readings.emplace_back(kind, *current);
            readings.push_back(std::move(*current));
        }
        return readings;
    }
    size_t index = static_cast<size_t>(last_of_kind - last_readings.begin());
    ReadingObject last = last_readings[index].second;
    while (true) {
        std::expected<ReadingObject, HRESULT> next = next_reading(last.Get(), kind);
        if (next) {
            readings.push_back(*next);
            last = std::move(*next);
        } else {
            if (after_next_reading_failed(next.error()) == NextReadingFailure::FindFromNewest) {
                std::vector<ReadingObject> after = readings_after(last->GetTimestamp(), kind);
                readings.insert(readings.end(), std::make_move_iterator(after.begin()), std::make_move_iterator(after.end()));
            }
            break;
        }
    }
    if (!readings.empty()) {
        last_readings[index].second = readings.back();
    }
    return readings;
}

std::expected<Devices::ReadingObject, HRESULT> Devices::current_reading(game_input::GameInputKind kind) const {
    ReadingObject reading;
    HRESULT code = game_input->GetCurrentReading(kind, nullptr, &reading);
    if (FAILED(code)) {
        return std::unexpected(code);
    }
    return reading;
}

std::expected<Devices::ReadingObject, HRESULT> Devices::next_reading(game_input::IGameInputReading* last, game_input::GameInputKind kind) const {
    ReadingObject reading;
    HRESULT code = game_input->GetNextReading(last, kind, nullptr, &reading);
    if (FAILED(code)) {
        return std::unexpected(code);
    }
    return reading;
}

std::vector<Devices::ReadingObject> Devices::readings_after(std::uint64_t timestamp, game_input::GameInputKind kind) const {
    std::vector<ReadingObject> readings;
    ReadingObject reading;
    if (std::expected<ReadingObject, HRESULT> current = current_reading(kind)) {
        reading = std::move(*current);
    }
    while (reading) {
        if (reading->GetTimestamp() <= timestamp) {
            break;
        }
        ReadingObject previous;
        if (FAILED(game_input->GetPreviousReading(reading.Get(), kind, nullptr, &previous))) {
            previous.Reset();
        }
        readings.push_back(std::move(reading));
        reading = std::move(previous);
    }
    std::ranges::reverse(readings);
    return readings;
}

std::string describe_value(NextReadingFailure failure) {
    switch (failure) {
    case NextReadingFailure::NothingNewer:
        return "NothingNewer";
    case NextReadingFailure::FindFromNewest:
        return "FindFromNewest";
    }
    base::unreachable();
}

NextReadingFailure after_next_reading_failed(HRESULT code) {
    if (code == game_input::GAMEINPUT_E_READING_NOT_FOUND) {
        return NextReadingFailure::NothingNewer;
    }
    return NextReadingFailure::FindFromNewest;
}

} // namespace user_interface
