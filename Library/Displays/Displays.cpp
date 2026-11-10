#include "Displays/Displays.h"

#include "Base/Text.h"

#include <ShellScalingApi.h>
#include <algorithm>
#include <array>
#include <expected>
#include <format>
#include <tuple>
#include <utility>

#pragma comment(lib, "shcore.lib")
#pragma comment(lib, "user32.lib")

namespace displays {

namespace {

// Crockford's Base32 alphabet. It leaves out I, L, O and U, so an identifier
// read off the screen has no characters that look alike.
constexpr std::string_view Alphabet = "0123456789ABCDEFGHJKMNPQRSTVWXYZ";
// 30 bits: the chance that two of five displays share an identifier is about
// one in a hundred million.
constexpr size_t IdentifierLength = 6;

// Makes the thread per-monitor DPI aware while it lives.
class PerMonitorAware {
public:
    PerMonitorAware() : before(SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2)) {}

    ~PerMonitorAware() {
        // Null if it couldn't be changed, so there's nothing to put back.
        if (before != nullptr) {
            SetThreadDpiAwarenessContext(before);
        }
    }

    PerMonitorAware(const PerMonitorAware&) = delete;
    PerMonitorAware& operator=(const PerMonitorAware&) = delete;

private:
    DPI_AWARENESS_CONTEXT before;
};

// Whether the two are the same but for the case of their ASCII letters:
// str::eq_ignore_ascii_case in Rust.
bool eq_ignore_ascii_case(std::string_view a, std::string_view b) {
    auto lower = [](char character) {
        return character >= 'A' && character <= 'Z' ? static_cast<char>(character + ('a' - 'A')) : character;
    };
    return std::ranges::equal(a, b, [&](char left, char right) { return lower(left) == lower(right); });
}

std::string from_wide(std::span<const wchar_t> text) {
    size_t length = static_cast<size_t>(std::ranges::find(text, L'\0') - text.begin());
    return base::narrow(std::wstring_view(text.data(), length));
}

// DisplayConfig functions return Win32 error codes.
std::expected<void, HRESULT> check(LONG error) {
    if (error == ERROR_SUCCESS) {
        return {};
    }
    return std::unexpected(HRESULT_FROM_WIN32(static_cast<DWORD>(error)));
}

std::vector<HMONITOR> monitors() {
    auto add = [](HMONITOR monitor, HDC, LPRECT, LPARAM monitors) -> BOOL {
        reinterpret_cast<std::vector<HMONITOR>*>(monitors)->push_back(monitor);
        return TRUE;
    };
    std::vector<HMONITOR> monitors;
    base::check_bool(EnumDisplayMonitors(nullptr, nullptr, add, reinterpret_cast<LPARAM>(&monitors)));
    return monitors;
}

// A monitor showing part of the desktop, as QueryDisplayConfig reports it.
struct Target {
    // The GDI device name of the part of the desktop it shows, which
    // GetMonitorInfoW also reports, like \\.\DISPLAY1.
    std::string gdi_name;
    std::string name;
    std::string device_path;
    DISPLAYCONFIG_VIDEO_OUTPUT_TECHNOLOGY output_technology;
    std::optional<double> refresh_rate;
};

std::expected<std::vector<Target>, HRESULT> targets() {
    UINT32 flags = QDC_ONLY_ACTIVE_PATHS;
    std::vector<DISPLAYCONFIG_PATH_INFO> paths;
    while (true) {
        UINT32 path_count = 0;
        UINT32 mode_count = 0;
        if (auto checked = check(GetDisplayConfigBufferSizes(flags, &path_count, &mode_count)); !checked) {
            return std::unexpected(checked.error());
        }
        paths = std::vector<DISPLAYCONFIG_PATH_INFO>(path_count);
        std::vector<DISPLAYCONFIG_MODE_INFO> modes(mode_count);
        LONG error = QueryDisplayConfig(flags, &path_count, paths.data(), &mode_count, modes.data(), nullptr);
        // A display came on between the two calls, so ask again.
        if (error == ERROR_INSUFFICIENT_BUFFER) {
            continue;
        }
        if (auto checked = check(error); !checked) {
            return std::unexpected(checked.error());
        }
        paths.resize(path_count);
        break;
    }

    std::vector<Target> targets;
    for (const DISPLAYCONFIG_PATH_INFO& path : paths) {
        DISPLAYCONFIG_SOURCE_DEVICE_NAME source{
            .header =
                {
                    .type = DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME,
                    .size = sizeof(DISPLAYCONFIG_SOURCE_DEVICE_NAME),
                    .adapterId = path.sourceInfo.adapterId,
                    .id = path.sourceInfo.id,
                },
        };
        // A monitor without its names gets an identifier from its GDI device name.
        if (DisplayConfigGetDeviceInfo(&source.header) != ERROR_SUCCESS) {
            continue;
        }
        DISPLAYCONFIG_TARGET_DEVICE_NAME target{
            .header =
                {
                    .type = DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME,
                    .size = sizeof(DISPLAYCONFIG_TARGET_DEVICE_NAME),
                    .adapterId = path.targetInfo.adapterId,
                    .id = path.targetInfo.id,
                },
        };
        if (DisplayConfigGetDeviceInfo(&target.header) != ERROR_SUCCESS) {
            continue;
        }

        DISPLAYCONFIG_RATIONAL rate = path.targetInfo.refreshRate;
        targets.push_back(Target{
            .gdi_name = from_wide(source.viewGdiDeviceName),
            .name = from_wide(target.monitorFriendlyDeviceName),
            .device_path = from_wide(target.monitorDevicePath),
            .output_technology = path.targetInfo.outputTechnology,
            .refresh_rate =
                rate.Numerator != 0 && rate.Denominator != 0 ? std::optional(static_cast<double>(rate.Numerator) / static_cast<double>(rate.Denominator)) : std::nullopt,
        });
    }
    return targets;
}

std::string_view connection(DISPLAYCONFIG_VIDEO_OUTPUT_TECHNOLOGY technology) {
    switch (technology) {
    case DISPLAYCONFIG_OUTPUT_TECHNOLOGY_HD15:
        return "VGA";
    case DISPLAYCONFIG_OUTPUT_TECHNOLOGY_DVI:
        return "DVI";
    case DISPLAYCONFIG_OUTPUT_TECHNOLOGY_HDMI:
        return "HDMI";
    case DISPLAYCONFIG_OUTPUT_TECHNOLOGY_DISPLAYPORT_EXTERNAL:
        return "DisplayPort";
    case DISPLAYCONFIG_OUTPUT_TECHNOLOGY_DISPLAYPORT_USB_TUNNEL:
        return "DisplayPort over USB";
    case DISPLAYCONFIG_OUTPUT_TECHNOLOGY_LVDS:
    case DISPLAYCONFIG_OUTPUT_TECHNOLOGY_DISPLAYPORT_EMBEDDED:
    case DISPLAYCONFIG_OUTPUT_TECHNOLOGY_UDI_EMBEDDED:
    case DISPLAYCONFIG_OUTPUT_TECHNOLOGY_INTERNAL:
        return "Built in";
    case DISPLAYCONFIG_OUTPUT_TECHNOLOGY_MIRACAST:
        return "Miracast";
    case DISPLAYCONFIG_OUTPUT_TECHNOLOGY_INDIRECT_WIRED:
        return "Indirect display";
    case DISPLAYCONFIG_OUTPUT_TECHNOLOGY_INDIRECT_VIRTUAL:
        return "Virtual display";
    default:
        return "Other";
    }
}

// FNV-1a, whose output never changes, unlike the standard library's hash,
// which may change between its versions. Its top bits depend on every bit of
// the input, so the identifier is taken from those. Device paths are
// case-insensitive.
std::string identifier(std::string_view device_path) {
    std::uint64_t hash = 0xCBF2'9CE4'8422'2325;
    for (char byte : base::to_ascii_lowercase(device_path)) {
        hash ^= static_cast<std::uint8_t>(byte);
        hash *= 0x0100'0000'01B3;
    }
    std::string text;
    text.reserve(IdentifierLength);
    for (size_t index = 1; index <= IdentifierLength; index++) {
        text.push_back(Alphabet[static_cast<size_t>(hash >> (64 - 5 * index)) & 31]);
    }
    return text;
}

} // namespace

std::int32_t Display::width() const {
    return area.right - area.left;
}

std::int32_t Display::height() const {
    return area.bottom - area.top;
}

std::string Display::details() const {
    std::string details = std::format("{} x {}", width(), height());
    if (refresh_rate) {
        std::string rate = std::format("{:.2f}", *refresh_rate);
        std::string_view trimmed = rate;
        while (trimmed.ends_with('0')) {
            trimmed.remove_suffix(1);
        }
        while (trimmed.ends_with('.')) {
            trimmed.remove_suffix(1);
        }
        details += std::format(", {} Hz", trimmed);
    }
    std::uint32_t scaled = dpi * 100;
    std::uint32_t scale = scaled / USER_DEFAULT_SCREEN_DPI + (scaled % USER_DEFAULT_SCREEN_DPI != 0 ? 1 : 0);
    details += std::format(", {}% scale, {}", scale, connection);
    return details;
}

std::vector<Display> enumerate() {
    PerMonitorAware aware;
    // Without QueryDisplayConfig's monitors, identifiers come from GDI device names.
    std::vector<Target> found = targets().value_or(std::vector<Target>());
    std::vector<Display> displays;
    for (HMONITOR monitor : monitors()) {
        MONITORINFOEXW info{};
        info.cbSize = sizeof(MONITORINFOEXW);
        base::check_bool(GetMonitorInfoW(monitor, &info));
        std::string gdi_name = from_wide(info.szDevice);
        std::array<UINT, 2> dpi{USER_DEFAULT_SCREEN_DPI, USER_DEFAULT_SCREEN_DPI};
        base::check(GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &dpi[0], &dpi[1]));

        auto display = [&](std::string name, std::string_view connection, std::string device_path, std::optional<double> refresh_rate) {
            return Display{
                .identifier = identifier(device_path),
                .name = std::move(name),
                .connection = connection,
                .device_path = std::move(device_path),
                .monitor = monitor,
                .area = info.rcMonitor,
                .work_area = info.rcWork,
                .primary = (info.dwFlags & MONITORINFOF_PRIMARY) != 0,
                .dpi = dpi[0],
                .refresh_rate = refresh_rate,
            };
        };
        // A monitor that clones another shows the same part of the desktop,
        // so both are on one HMONITOR.
        size_t count = displays.size();
        for (const Target& target : found) {
            if (target.gdi_name != gdi_name) {
                continue;
            }
            std::string_view connected = connection(target.output_technology);
            std::string name = !target.name.empty() ? target.name : connected == "Built in" ? std::string("Built-in display") : std::string("Unnamed display");
            std::string device_path = target.device_path.empty() ? gdi_name : target.device_path;
            displays.push_back(display(std::move(name), connected, std::move(device_path), target.refresh_rate));
        }
        if (displays.size() == count) {
            displays.push_back(display(gdi_name, "Other", gdi_name, std::nullopt));
        }
    }
    std::ranges::stable_sort(displays,
                             [](const Display& a, const Display& b) { return std::tie(a.area.left, a.area.top, a.identifier) < std::tie(b.area.left, b.area.top, b.identifier); });
    return displays;
}

const Display* find(std::span<const Display> displays, std::string_view identifier) {
    std::string_view trimmed = base::trim(identifier);
    for (const Display& display : displays) {
        if (eq_ignore_ascii_case(display.identifier, trimmed)) {
            return &display;
        }
    }
    return nullptr;
}

const Display* primary(std::span<const Display> displays) {
    for (const Display& display : displays) {
        if (display.primary) {
            return &display;
        }
    }
    return displays.empty() ? nullptr : &displays.front();
}

} // namespace displays
