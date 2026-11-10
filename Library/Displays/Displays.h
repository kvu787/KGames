// Finds the displays that show part of the desktop, and gives each one an
// identifier that stays the same from one run to the next, so a setting can
// name the display to open on. IdentifyDisplays shows each display's
// identifier on that display.
//
// The identifier is a hash of the monitor's device path, like
// \\?\DISPLAY#DEL40F2#5&2d3b6e3c&0&UID4352#{e6f07b5f-...}. Windows makes the
// path from the monitor's model, from its EDID, and from the GPU output it's
// plugged into. So two monitors of the same model get different identifiers,
// and a monitor keeps its identifier across restarts, but gets a new one if
// it's plugged into another output.

#pragma once

#include "Base/Windows.h"

#include <cstdint>
#include <optional>
#include <span>
#include <string>
#include <string_view>
#include <vector>

namespace displays {

struct Display {
    std::string identifier;
    // The monitor's friendly name, which Windows usually makes from its
    // EDID, like "DELL U2720Q".
    std::string name;
    // What the monitor is plugged into, like "HDMI" or "Built in".
    std::string_view connection;
    // What the identifier is a hash of: the monitor's device path, or the
    // GDI device name, like \\.\DISPLAY1, if Windows doesn't report a monitor,
    // like over Remote Desktop. GDI device names can change between runs.
    std::string device_path;
    HMONITOR monitor = nullptr;
    // The display's part of the desktop, and that part without the taskbar,
    // in physical pixels.
    RECT area{};
    RECT work_area{};
    bool primary = false;
    std::uint32_t dpi = 0;
    // In hertz.
    std::optional<double> refresh_rate;

    std::int32_t width() const;

    std::int32_t height() const;

    // Like "3840 x 2160, 144 Hz, 150% scale, DisplayPort".
    std::string details() const;
};

// The displays that show part of the desktop, from left to right, in
// physical pixels and with their own DPI, whatever DPI awareness the calling
// thread has. Windows tells a thread that isn't per-monitor aware that every
// display is at 96 DPI, and scales its areas to match, so this asks as a
// per-monitor aware thread, and then puts the thread's awareness back.
std::vector<Display> enumerate();

// The display with this identifier, ignoring case, or null.
const Display* find(std::span<const Display> displays, std::string_view identifier);

const Display* primary(std::span<const Display> displays);

} // namespace displays
