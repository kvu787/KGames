// A present, once it's known whether it reached the screen, how, and when,
// and why presents' times aren't known. PresentTiming.h includes this, and
// has the rest of the library: these are in a header of their own so that
// Tracker.h and Session.h, which PresentTiming.h includes, can use them.

#pragma once

#include "Base/Panic.h"
#include "Base/Sum.h"

#include <array>
#include <cstdint>
#include <optional>
#include <string>
#include <string_view>
#include <utility>
#include <variant>

namespace present_timing {

// How a present got to the screen: each of the ways that PresentMon tells
// apart, which are every way that Windows' events tell apart.
enum class PresentMode {
    // Its events didn't say.
    Unknown,
    // A fullscreen exclusive flip: the process owns the display, which
    // shows its buffer. PresentMon's "Hardware: Legacy Flip". DWM's own
    // frames go to the screen this way too.
    LegacyFlip,
    // A fullscreen exclusive bitblt: the process copies its buffer into the
    // one that the display shows. PresentMon's "Hardware: Legacy Copy to
    // front buffer".
    LegacyCopy,
    // The display took the swap chain's buffer itself, without DWM, like in
    // borderless fullscreen. PresentMon's "Hardware: Independent Flip".
    IndependentFlip,
    // The same, on one of the display's overlay planes, over the desktop's.
    // PresentMon's "Hardware Composed: Independent Flip".
    OverlayPlane,
    // DWM drew the swap chain's buffer into its own frame. PresentMon's
    // "Composed: Flip".
    ComposedFlip,
    // A bitblt model swap chain's buffer was copied into the window's
    // surface on the GPU, which DWM drew into its own frame. PresentMon's
    // "Composed: Copy with GPU GDI".
    ComposedGpuCopy,
    // The same, copied on the CPU. PresentMon's "Composed: Copy with CPU
    // GDI".
    ComposedCpuCopy,
};

// Every mode: PresentMode::ALL in Rust, which an enum class has no place for.
inline constexpr std::array<PresentMode, 8> AllPresentModes = {
    PresentMode::Unknown,      PresentMode::LegacyFlip,   PresentMode::LegacyCopy,      PresentMode::IndependentFlip,
    PresentMode::OverlayPlane, PresentMode::ComposedFlip, PresentMode::ComposedGpuCopy, PresentMode::ComposedCpuCopy,
};

// The mode's name: PresentMode::name in Rust.
constexpr std::string_view name(PresentMode mode) {
    switch (mode) {
    case PresentMode::Unknown:
        return "Unknown";
    case PresentMode::LegacyFlip:
        return "Legacy flip";
    case PresentMode::LegacyCopy:
        return "Legacy copy";
    case PresentMode::IndependentFlip:
        return "Independent flip";
    case PresentMode::OverlayPlane:
        return "Overlay plane";
    case PresentMode::ComposedFlip:
        return "Composed flip";
    case PresentMode::ComposedGpuCopy:
        return "Composed GPU copy";
    case PresentMode::ComposedCpuCopy:
        return "Composed CPU copy";
    }
    base::unreachable();
}

// How a failed check in a test shows a mode: as its name in the code.
std::string describe_value(PresentMode mode);

// A present, once it's known whether it reached the screen, and when.
struct Present {
    // When the Present call started, as the performance counter's ticks.
    std::int64_t start;
    // The thread that called Present.
    std::uint32_t thread;
    PresentMode mode;
    // Whether its flip could tear, reaching the screen mid-refresh.
    bool tearing;
    // When the display took it, as the performance counter's ticks, or
    // nothing if it never reached the screen, like when a newer one replaced
    // it.
    std::optional<std::int64_t> screen;
    // How much later than the kernel's flip NVIDIA's driver said it would
    // flip, which the screen time has in it, in ticks.
    std::int64_t flip_delay;

    bool operator==(const Present&) const = default;
};

// How a failed check in a test shows a present.
std::string describe_value(const Present& present);

// Why presents' times aren't known.
struct StartError {
    // Reading ETW's events needs the Performance Log Users group, or an
    // elevated administrator.
    struct AccessDenied {
        bool operator==(const AccessDenied&) const = default;
    };
    struct Failed {
        std::string message;
        bool operator==(const Failed&) const = default;
    };
    std::variant<AccessDenied, Failed> value;

    template <base::AlternativeOf<StartError> T> StartError(T&& alternative) : value(std::forward<T>(alternative)) {}

    // The error as a message says it: what Display writes in Rust.
    std::string to_string() const;

    bool operator==(const StartError&) const = default;
};

} // namespace present_timing
