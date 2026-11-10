// Every program's input devices, read with GameInput: gamepads, keyboards and
// mice, which are all the input there is. The main loop reads them once a
// frame, right before the frame, and turns what they read into the
// interface's events (Input/Reader.h). Programs that show the devices
// themselves, like GameInputDemo, read the same frame's readings and device
// events from here, through Startup::devices, and run gamepads' motors.
//
// GameInput's input is readings, not presses and releases: each reading is a
// snapshot of one device's whole state, with the time that it came, and
// GameInput keeps half a second of them. This polls that history once a
// frame and walks it with GetNextReading, so that no reading is missed
// between frames. Readings' times are GameInput's
// microseconds, which `qpc` turns into the performance counter's ticks, on
// the clock that frames and the display's flips are timed on.

#pragma once

#include "Base/Windows.h"
#include "GameInput/GameInput.h"

#include <wrl/client.h>

#include <array>
#include <cstddef>
#include <cstdint>
#include <expected>
#include <memory>
#include <optional>
#include <span>
#include <string>
#include <utility>
#include <vector>

namespace user_interface {

// A gamepad, keyboard or mouse, with everything that GameInput says about it.
struct Device {
    // Tells devices apart, and stays the same while a device is connected.
    size_t id = 0;
    // What it calls itself, like "Xbox Controller".
    std::string name;
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

    bool operator==(const Device&) const = default;
};

// A snapshot of one device's whole state, at a time.
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

// The kinds of input that every frame reads, each with its own history.
inline constexpr std::array<game_input::GameInputKind, 3> Kinds{game_input::GameInputKindGamepad, game_input::GameInputKindKeyboard, game_input::GameInputKindMouse};

namespace input_devices_detail {

// The device events that GameInput's callback queues for the main thread,
// behind a mutex, which InputDevices.cpp defines.
struct DeviceQueue;

} // namespace input_devices_detail

class Devices {
public:
    // Starts GameInput, with GameInput.lib's loader, which loads the installed
    // runtime (see Library/GameInput), and starts listening for gamepads,
    // keyboards and mice. The callback also reports the devices that are already
    // connected. Returns what's wrong if GameInput can't start.
    static std::expected<Devices, std::string> create();

    ~Devices();
    Devices(const Devices&) = delete;
    Devices& operator=(const Devices&) = delete;
    Devices(Devices&& other) noexcept;
    Devices& operator=(Devices&& other) noexcept;

    const game_input::Runtime& runtime() const;

    // The time now on GameInput's clock, in microseconds.
    std::uint64_t now() const;

    // A time on GameInput's clock as the counter's ticks.
    std::int64_t qpc(std::uint64_t timestamp) const;

    // Reads the frame's input: every reading since the last frame, of each
    // kind, and what happened to devices.
    void read();

    // How many frames have read input, which tells a program whether it has
    // already seen this frame's readings.
    std::uint64_t frame() const;

    // This frame's readings with the kind of input, oldest first.
    std::span<const Reading> readings(game_input::GameInputKind kind) const;

    // What happened to devices this frame, oldest first: they connected,
    // disconnected, or changed.
    std::span<const std::pair<game_input::GameInputDeviceEvent, Device>> device_events() const;

    // Runs a gamepad's motors, or stops them with null.
    void set_rumble(size_t device, const game_input::GameInputRumbleParams* params);

private:
    using Object = Microsoft::WRL::ComPtr<game_input::IGameInputDevice>;
    using ReadingObject = Microsoft::WRL::ComPtr<game_input::IGameInputReading>;

    Devices(Microsoft::WRL::ComPtr<game_input::IGameInput> game_input, game_input::Runtime runtime, game_input::GameInputCallbackToken callback_token,
            std::unique_ptr<input_devices_detail::DeviceQueue> queue);

    // Finds where GameInput's clock is from the counter's, from the closest
    // of a few readings of both.
    void calibrate();

    // The device's data, from what's known about it, or from GameInput.
    Device device(game_input::IGameInputDevice* device);

    Reading reading(game_input::IGameInputReading* reading);

    // Readings of one kind since the last call, oldest first.
    std::vector<ReadingObject> new_readings(game_input::GameInputKind kind);

    // GetCurrentReading and GetNextReading, with the HRESULT that GameInput
    // returns. They fail every frame for each kind of input with nothing new
    // to read, so a failure is a value here, not an exception.
    std::expected<ReadingObject, HRESULT> current_reading(game_input::GameInputKind kind) const;

    std::expected<ReadingObject, HRESULT> next_reading(game_input::IGameInputReading* last, game_input::GameInputKind kind) const;

    // Readings of one kind newer than a timestamp, oldest first. Walks back
    // from the newest reading with GetPreviousReading.
    std::vector<ReadingObject> readings_after(std::uint64_t timestamp, game_input::GameInputKind kind) const;

    Microsoft::WRL::ComPtr<game_input::IGameInput> game_input;
    game_input::Runtime runtime_;
    game_input::GameInputCallbackToken callback_token = 0;
    // The queue that the callback fills, which it has a pointer to. It's on
    // the heap, so that it stays where the callback's pointer says it is
    // when the Devices moves.
    std::unique_ptr<input_devices_detail::DeviceQueue> queue;
    // The last reading handed out, for each kind of input.
    std::vector<std::pair<game_input::GameInputKind, ReadingObject>> last_readings;
    // Every device seen, as GameInput's object and as data. GameInput keeps
    // one object per device, so the object's address tells devices apart.
    std::vector<std::pair<Object, Device>> known;
    // GameInput's microseconds at the counter's tick 0, which GameInput's
    // clock is a fixed distance from. Checked again every few seconds.
    std::int64_t epoch = 0;
    std::int64_t calibrated = 0;
    // This frame's readings by kind, oldest first, and device events.
    std::array<std::vector<Reading>, 3> readings_;
    std::vector<std::pair<game_input::GameInputDeviceEvent, Device>> device_events_;
    std::uint64_t frame_ = 0;
};

// What GetNextReading's failure says about the readings after the last one
// handled.
enum class NextReadingFailure : std::uint8_t {
    // There are none yet.
    NothingNewer,
    // The last reading can't be walked on from, so find the ones after it
    // from the newest end, by their times.
    FindFromNewest,
};

// How a failed check in a test shows a failure: its name.
std::string describe_value(NextReadingFailure failure);

// Only GAMEINPUT_E_READING_NOT_FOUND says that there's nothing newer. Any
// other failure, like GAMEINPUT_E_REFERENCE_READING_TOO_OLD after a pause in
// input, when the last reading handled has left GameInput's half second of
// history, starts over from the newest reading, as Microsoft's sample for
// GetNextReading does. Otherwise a failure that kept coming would stop that
// kind of input for good.
NextReadingFailure after_next_reading_failed(HRESULT code);

} // namespace user_interface
