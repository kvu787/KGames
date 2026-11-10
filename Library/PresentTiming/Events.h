// The ETW events that tell when presents reach the screen, from DXGI and
// Direct3D 9, the graphics kernel (DxgKrnl), Win32k and the desktop window
// manager (DWM), and NVIDIA's display driver, as plain data. The session
// enables these providers with these events, the levels and the keywords
// that PresentMon uses, so that the times are the same as PresentMon's.

#pragma once

#include "Base/Sum.h"
#include "Base/Windows.h"
#include "PresentTiming/Decoder.h"

#include <array>
#include <cstdint>
#include <optional>
#include <span>
#include <tuple>
#include <utility>
#include <variant>
#include <vector>

namespace present_timing::events {

inline constexpr GUID Dxgkrnl{0x802ec45a, 0x1e99, 0x4b83, {0x99, 0x20, 0x87, 0xc9, 0x82, 0x77, 0xba, 0x9d}};
inline constexpr GUID Win32k{0x8c416c79, 0xd49b, 0x4f01, {0xa4, 0x67, 0xe5, 0x6d, 0x3a, 0xa8, 0x23, 0x4c}};
inline constexpr GUID Dwm{0x9e9bba3c, 0x2e38, 0x40cb, {0x99, 0xf4, 0x9e, 0x82, 0x81, 0x42, 0x51, 0x64}};
inline constexpr GUID Dxgi{0xca11c036, 0x0102, 0x4a2d, {0xa6, 0xad, 0xf0, 0x3c, 0xfe, 0xd5, 0xd3, 0xc9}};
inline constexpr GUID D3d9{0x783aca0a, 0x790e, 0x4d7f, {0x84, 0x51, 0xaa, 0x85, 0x05, 0x11, 0xc6, 0xb9}};
inline constexpr GUID Nvidia{0xae4f8626, 0x8265, 0x40d1, {0xa7, 0x0b, 0x11, 0xb6, 0x42, 0x40, 0xe8, 0xe9}};

// A provider to enable: its level, keywords, and the events to let through.
struct Provider {
    GUID guid;
    std::uint8_t level;
    std::uint64_t any;
    std::uint64_t all;
    std::span<const std::uint16_t> events;
};

// In the order that PresentMon enables them: the kernel's first. EnableTraceEx2
// with no timeout configures each provider without waiting for it, so the
// order isn't a promise that the kernel's events come first: the tracker
// drops the presents that were under way when it started.
extern const std::array<Provider, 6> Providers;

// The API whose Present call a present came from.
enum class Runtime {
    Dxgi,
    D3d9,
};

struct Kind {
    // DXGI's and Direct3D 9's Present calls. Direct3D 9's flags are DXGI's
    // that mean the same.
    struct PresentStart {
        Runtime runtime;
        std::uint64_t swap_chain;
        std::uint32_t flags;
        std::int32_t sync_interval;
        bool operator==(const PresentStart&) const = default;
    };
    struct PresentStop {
        Runtime runtime;
        std::uint32_t result;
        bool operator==(const PresentStop&) const = default;
    };
    // Win32k: a present's token for composition, and its state changes.
    struct TokenCompositionSurfaceObject {
        std::tuple<std::uint64_t, std::uint64_t, std::uint64_t> key;
        bool operator==(const TokenCompositionSurfaceObject&) const = default;
    };
    struct TokenStateChanged {
        std::tuple<std::uint64_t, std::uint64_t, std::uint64_t> key;
        std::uint32_t state;
        bool independent_flip;
        bool operator==(const TokenStateChanged&) const = default;
    };
    // DxgKrnl.
    struct PresentHistoryStart {
        std::uint64_t token;
        std::uint32_t model;
        std::uint64_t token_data;
        bool operator==(const PresentHistoryStart&) const = default;
    };
    struct PresentHistoryInfo {
        std::uint64_t token;
        bool operator==(const PresentHistoryInfo&) const = default;
    };
    struct QueuePacketStart {
        std::uint32_t packet_type;
        std::uint32_t submit_sequence;
        std::uint64_t context;
        bool present;
        bool operator==(const QueuePacketStart&) const = default;
    };
    struct QueuePacketStop {
        std::uint64_t context;
        std::uint32_t submit_sequence;
        bool operator==(const QueuePacketStop&) const = default;
    };
    struct PresentInfo {
        std::uint64_t window;
        bool operator==(const PresentInfo&) const = default;
    };
    // A bitblt model present's copy, to the screen or else to the window's
    // surface for DWM, and the kernel finding the copy unneeded.
    struct Blit {
        std::uint64_t window;
        bool redirected;
        bool operator==(const Blit&) const = default;
    };
    struct BlitCancel {
        bool operator==(const BlitCancel&) const = default;
    };
    struct Flip {
        std::uint32_t interval;
        bool mmio;
        bool operator==(const Flip&) const = default;
    };
    struct FlipMultiPlaneOverlay {
        std::uint32_t source;
        std::uint32_t layer;
        bool operator==(const FlipMultiPlaneOverlay&) const = default;
    };
    struct IndependentFlip {
        std::uint32_t submit_sequence;
        std::uint32_t interval;
        bool operator==(const IndependentFlip&) const = default;
    };
    struct MmioFlip {
        std::uint32_t submit_sequence;
        std::uint32_t flags;
        bool operator==(const MmioFlip&) const = default;
    };
    // The status after the flip is nothing before the event's version 2.
    struct MmioFlipMultiPlaneOverlay {
        std::uint32_t submit_sequence;
        std::optional<std::uint32_t> status;
        bool operator==(const MmioFlipMultiPlaneOverlay&) const = default;
    };
    struct VSyncDpc {
        std::uint64_t fence;
        bool operator==(const VSyncDpc&) const = default;
    };
    struct SyncDpcMultiPlane {
        std::vector<std::uint64_t> planes;
        std::vector<std::uint64_t> flip_sequences;
        bool operator==(const SyncDpcMultiPlane&) const = default;
    };
    // DWM.
    struct DwmSchedulePresent {
        bool operator==(const DwmSchedulePresent&) const = default;
    };
    struct DwmGetPresentHistory {
        bool operator==(const DwmGetPresentHistory&) const = default;
    };
    struct DwmScheduleSurfaceUpdate {
        std::tuple<std::uint64_t, std::uint64_t, std::uint64_t> key;
        bool operator==(const DwmScheduleSurfaceUpdate&) const = default;
    };
    // DWM taking a present that was copied on the CPU, by its present
    // history token's data.
    struct DwmFlipChain {
        std::uint64_t token_data;
        std::uint64_t window;
        bool operator==(const DwmFlipChain&) const = default;
    };
    // NVIDIA's display driver: when it means to flip.
    struct NvidiaFlipRequest {
        std::uint64_t alloc;
        std::uint32_t source;
        std::uint64_t time;
        std::uint32_t token;
        bool operator==(const NvidiaFlipRequest&) const = default;
    };
    std::variant<PresentStart, PresentStop, TokenCompositionSurfaceObject, TokenStateChanged, PresentHistoryStart, PresentHistoryInfo, QueuePacketStart, QueuePacketStop,
                 PresentInfo, Blit, BlitCancel, Flip, FlipMultiPlaneOverlay, IndependentFlip, MmioFlip, MmioFlipMultiPlaneOverlay, VSyncDpc, SyncDpcMultiPlane, DwmSchedulePresent,
                 DwmGetPresentHistory, DwmScheduleSurfaceUpdate, DwmFlipChain, NvidiaFlipRequest>
        value;

    template <base::AlternativeOf<Kind> T> Kind(T&& alternative) : value(std::forward<T>(alternative)) {}

    bool operator==(const Kind&) const = default;
};

// An event, with what's in its header.
struct Event {
    // As the performance counter's ticks.
    std::int64_t time;
    std::uint32_t process;
    std::uint32_t thread;
    Kind kind;

    bool operator==(const Event&) const = default;
};

// Win32k's token states.
inline constexpr std::uint32_t InFrame = 3;
inline constexpr std::uint32_t Confirmed = 4;
inline constexpr std::uint32_t Discarded = 6;

// The event as plain data, or nothing if it's not one of these, or its
// fields aren't there.
std::optional<Event> decode(decoder::Decoder& decoder, const EVENT_RECORD& record);

} // namespace present_timing::events
