// Tests that the tracker follows presents through Windows' events to when
// the display took them, as PresentMon does, with made-up events in the
// order that Windows sends them, for each mode: the display taking the
// buffer itself, with and without tearing, and on an overlay plane, DWM
// composing it into its own frame, fullscreen exclusive flips and copies,
// and copies that DWM composes, made on the GPU and on the CPU.

#include "PresentTiming/PresentTiming.h"
#include "Testing/Testing.h"

#include <algorithm>
#include <cstdint>
#include <initializer_list>
#include <optional>
#include <span>
#include <tuple>
#include <utility>
#include <vector>

using present_timing::Corrector;
using present_timing::Present;
using present_timing::PresentMode;
using present_timing::events::Confirmed;
using present_timing::events::Event;
using present_timing::events::InFrame;
using present_timing::events::Kind;
using present_timing::events::Runtime;
using present_timing::tracker::Tracker;

namespace {

constexpr std::uint32_t Process = 100;
constexpr std::uint32_t Thread = 7;
constexpr std::uint32_t DwmProcess = 50;
constexpr std::uint32_t DwmThread = 51;
constexpr std::uint64_t Window = 0x99;
constexpr std::uint64_t SwapChain = 0xABC;
constexpr std::uint32_t AllowTearing = 0x200;
constexpr std::uint32_t FlipWaitVsync = 5;
constexpr std::uint32_t FlipWaitComplete = 11;

Event event(std::int64_t time, std::uint32_t process, std::uint32_t thread, Kind kind) {
    return Event{.time = time, .process = process, .thread = thread, .kind = std::move(kind)};
}

// What the process's thread sends for a Present call that starts at a time,
// with the submit sequence of its queue packet.
void present(Tracker& tracker, std::int64_t start, std::uint32_t sequence, std::uint32_t flags) {
    std::tuple<std::uint64_t, std::uint64_t, std::uint64_t> key{1, sequence, 1};
    for (const auto& [offset, kind] : std::initializer_list<std::pair<std::int64_t, Kind>>{
             {0, Kind::PresentStart{.runtime = Runtime::Dxgi, .swap_chain = SwapChain, .flags = flags, .sync_interval = 0}},
             {1, Kind::TokenCompositionSurfaceObject{.key = key}},
             {2, Kind::PresentHistoryStart{.token = 1000 + std::uint64_t{sequence}, .model = 3, .token_data = 0}},
             {3, Kind::QueuePacketStart{.packet_type = 0, .submit_sequence = sequence, .context = 0x5, .present = true}},
             {4, Kind::PresentInfo{.window = Window}},
             {5, Kind::PresentStop{.runtime = Runtime::Dxgi, .result = 0}},
         }) {
        tracker.handle(event(start + offset, Process, Thread, kind));
    }
}

void in_frame(Tracker& tracker, std::int64_t time, std::uint32_t sequence, bool independent_flip) {
    std::tuple<std::uint64_t, std::uint64_t, std::uint64_t> key{1, sequence, 1};
    tracker.handle(event(time, 4, 4, Kind::TokenStateChanged{.key = key, .state = InFrame, .independent_flip = independent_flip}));
}

// The display's interrupt that says which flips are on the screen.
void sync_dpc(Tracker& tracker, std::int64_t time, std::span<const std::uint32_t> sequences) {
    std::vector<std::uint64_t> flip_sequences;
    for (std::uint32_t sequence : sequences) {
        flip_sequences.push_back(std::uint64_t{sequence} << 32);
    }
    tracker.handle(event(time, 0, 0, Kind::SyncDpcMultiPlane{.planes = {0x1000}, .flip_sequences = std::move(flip_sequences)}));
}

void sync_dpc(Tracker& tracker, std::int64_t time, std::initializer_list<std::uint32_t> sequences) {
    sync_dpc(tracker, time, std::span<const std::uint32_t>(sequences));
}

// A present that the display takes at once, mid-refresh: on the screen as
// the flip happens.
void torn(Tracker& tracker, std::int64_t start, std::uint32_t sequence) {
    present(tracker, start, sequence, AllowTearing);
    in_frame(tracker, start + 6, sequence, true);
    tracker.handle(event(start + 10, 0, 0, Kind::MmioFlipMultiPlaneOverlay{.submit_sequence = sequence, .status = FlipWaitComplete}));
    sync_dpc(tracker, start + 20, {sequence});
}

} // namespace

TEST(the_first_present_to_complete_and_those_under_way_are_lost) {
    Tracker tracker(Process);
    torn(tracker, 100, 1);
    CHECK_EQ(tracker.take(), std::vector<Present>{});
    torn(tracker, 200, 2);
    CHECK_EQ(tracker.take().size(), 1);
}

TEST(a_torn_flip_is_on_the_screen_when_it_flips) {
    Tracker tracker(Process);
    torn(tracker, 100, 1);
    torn(tracker, 200, 2);
    CHECK_EQ(tracker.take(), (std::vector<Present>{Present{
                                 .start = 200,
                                 .thread = Thread,
                                 .mode = PresentMode::IndependentFlip,
                                 .tearing = true,
                                 .screen = 210,
                                 .flip_delay = 0,
                             }}));
}

TEST(a_flip_that_waits_for_vertical_sync_is_on_the_screen_at_the_sync) {
    Tracker tracker(Process);
    torn(tracker, 100, 1);
    tracker.take();
    present(tracker, 200, 2, 0);
    in_frame(tracker, 206, 2, true);
    tracker.handle(event(210, 0, 0, Kind::MmioFlipMultiPlaneOverlay{.submit_sequence = 2, .status = FlipWaitVsync}));
    CHECK_EQ(tracker.take(), std::vector<Present>{});
    tracker.handle(event(250, 0, 0, Kind::VSyncDpc{.fence = std::uint64_t{2} << 32}));
    sync_dpc(tracker, 251, {2});
    std::vector<Present> presents = tracker.take();
    CHECK_EQ(presents.size(), 1);
    CHECK_EQ(presents[0].screen, std::optional<std::int64_t>(250));
    CHECK_EQ(presents[0].mode, PresentMode::IndependentFlip);
    CHECK(!presents[0].tearing);
}

TEST(a_flip_with_other_planes_on_the_screen_is_on_an_overlay_plane) {
    Tracker tracker(Process);
    torn(tracker, 100, 1);
    tracker.take();
    present(tracker, 200, 2, AllowTearing);
    in_frame(tracker, 206, 2, true);
    tracker.handle(event(210, 0, 0, Kind::MmioFlipMultiPlaneOverlay{.submit_sequence = 2, .status = FlipWaitComplete}));
    tracker.handle(event(220, 0, 0, Kind::SyncDpcMultiPlane{.planes = {0x1000, 0x2000}, .flip_sequences = {std::uint64_t{2} << 32}}));
    CHECK_EQ(tracker.take()[0].mode, PresentMode::OverlayPlane);
}

namespace {

// DWM's frame: it schedules its present, flips on an overlay plane's path,
// queues the flip with a submit sequence, waits for vertical sync, and the
// display takes it at the sync.
void dwm_frame(Tracker& tracker, std::int64_t time, std::uint32_t sequence) {
    tracker.handle(event(time, DwmProcess, DwmThread, Kind::DwmSchedulePresent{}));
    tracker.handle(event(time + 1, DwmProcess, DwmThread, Kind::FlipMultiPlaneOverlay{.source = 0, .layer = 0}));
    tracker.handle(event(time + 2, DwmProcess, DwmThread, Kind::QueuePacketStart{.packet_type = 3, .submit_sequence = sequence, .context = 0x9, .present = false}));
    tracker.handle(event(time + 3, 0, 0, Kind::MmioFlipMultiPlaneOverlay{.submit_sequence = sequence, .status = FlipWaitVsync}));
    tracker.handle(event(time + 10, 0, 0, Kind::VSyncDpc{.fence = std::uint64_t{sequence} << 32}));
    sync_dpc(tracker, time + 11, {sequence});
}

// A present that DWM will compose: in its next frame, and scheduled with it.
void composed(Tracker& tracker, std::int64_t start, std::uint32_t sequence) {
    present(tracker, start, sequence, 0);
    in_frame(tracker, start + 6, sequence, false);
    tracker.handle(event(start + 7, DwmProcess, DwmThread, Kind::DwmScheduleSurfaceUpdate{.key = {1, sequence, 1}}));
}

} // namespace

TEST(a_composed_present_is_on_the_screen_with_dwm_s_frame) {
    Tracker tracker(Process);
    tracker.handle(event(10, DwmProcess, DwmThread, Kind::DwmSchedulePresent{}));
    torn(tracker, 100, 1);
    tracker.take();
    composed(tracker, 200, 2);
    dwm_frame(tracker, 300, 900);
    CHECK_EQ(tracker.take(), (std::vector<Present>{Present{
                                 .start = 200,
                                 .thread = Thread,
                                 .mode = PresentMode::ComposedFlip,
                                 .tearing = false,
                                 .screen = 310,
                                 .flip_delay = 0,
                             }}));
}

TEST(of_two_presents_in_one_dwm_frame_only_the_newer_is_on_the_screen) {
    Tracker tracker(Process);
    tracker.handle(event(10, DwmProcess, DwmThread, Kind::DwmSchedulePresent{}));
    torn(tracker, 100, 1);
    tracker.take();
    composed(tracker, 200, 2);
    composed(tracker, 250, 3);
    dwm_frame(tracker, 300, 900);
    std::vector<Present> presents = tracker.take();
    std::ranges::stable_sort(presents, {}, &Present::start);
    std::vector<std::pair<std::int64_t, std::optional<std::int64_t>>> screens;
    for (const Present& present : presents) {
        screens.emplace_back(present.start, present.screen);
    }
    CHECK_EQ(screens, (std::vector<std::pair<std::int64_t, std::optional<std::int64_t>>>{{200, std::nullopt}, {250, 310}}));
}

TEST(a_present_that_dxgi_says_is_occluded_never_reaches_the_screen) {
    Tracker tracker(Process);
    torn(tracker, 100, 1);
    tracker.take();
    tracker.handle(event(200, Process, Thread, Kind::PresentStart{.runtime = Runtime::Dxgi, .swap_chain = SwapChain, .flags = 0, .sync_interval = 0}));
    tracker.handle(event(205, Process, Thread, Kind::PresentStop{.runtime = Runtime::Dxgi, .result = 0x087A'0001}));
    std::vector<Present> presents = tracker.take();
    CHECK_EQ(presents.size(), 1);
    CHECK_EQ(presents[0].screen, std::nullopt);
}

TEST(another_process_s_presents_are_not_reported) {
    Tracker tracker(Process + 1);
    torn(tracker, 100, 1);
    torn(tracker, 200, 2);
    CHECK_EQ(tracker.take(), std::vector<Present>{});
}

TEST(a_confirmed_token_lets_the_window_s_next_present_in) {
    Tracker tracker(Process);
    tracker.handle(event(10, DwmProcess, DwmThread, Kind::DwmSchedulePresent{}));
    torn(tracker, 100, 1);
    tracker.take();
    composed(tracker, 200, 2);
    tracker.handle(event(260, 4, 4, Kind::TokenStateChanged{.key = {1, 2, 1}, .state = Confirmed, .independent_flip = false}));
    dwm_frame(tracker, 300, 900);
    CHECK_EQ(tracker.take()[0].screen, std::optional<std::int64_t>(310));
}

TEST(a_present_that_direct3d_9_says_is_occluded_never_reaches_the_screen) {
    Tracker tracker(Process);
    torn(tracker, 100, 1);
    tracker.take();
    tracker.handle(event(200, Process, Thread, Kind::PresentStart{.runtime = Runtime::D3d9, .swap_chain = SwapChain, .flags = 0, .sync_interval = -1}));
    tracker.handle(event(205, Process, Thread, Kind::PresentStop{.runtime = Runtime::D3d9, .result = 0x0876'0878}));
    std::vector<Present> presents = tracker.take();
    CHECK_EQ(presents.size(), 1);
    CHECK_EQ(presents[0].screen, std::nullopt);
}

TEST(a_present_of_an_api_without_events_of_its_own_is_reported) {
    // Like OpenGL's or Vulkan's, which only the kernel's and Win32k's
    // events tell of.
    Tracker tracker(Process);
    tracker.handle(event(10, DwmProcess, DwmThread, Kind::DwmSchedulePresent{}));
    torn(tracker, 100, 1);
    tracker.take();
    std::tuple<std::uint64_t, std::uint64_t, std::uint64_t> key{1, 2, 1};
    tracker.handle(event(200, Process, Thread, Kind::TokenCompositionSurfaceObject{.key = key}));
    tracker.handle(event(201, Process, Thread, Kind::PresentHistoryStart{.token = 1002, .model = 3, .token_data = 0}));
    tracker.handle(event(202, Process, Thread, Kind::QueuePacketStart{.packet_type = 0, .submit_sequence = 2, .context = 0x5, .present = true}));
    tracker.handle(event(203, Process, Thread, Kind::PresentInfo{.window = Window}));
    in_frame(tracker, 206, 2, false);
    tracker.handle(event(207, DwmProcess, DwmThread, Kind::DwmScheduleSurfaceUpdate{.key = key}));
    dwm_frame(tracker, 300, 900);
    CHECK_EQ(tracker.take(), (std::vector<Present>{Present{
                                 .start = 200,
                                 .thread = Thread,
                                 .mode = PresentMode::ComposedFlip,
                                 .tearing = false,
                                 .screen = 310,
                                 .flip_delay = 0,
                             }}));
}

namespace {

// A fullscreen exclusive flip: the kernel's flip on the present's thread,
// its queue packet, and the display taking it at the sync.
void legacy_flip(Tracker& tracker, std::int64_t start, std::uint32_t sequence) {
    for (const auto& [offset, kind] : std::initializer_list<std::pair<std::int64_t, Kind>>{
             {0, Kind::PresentStart{.runtime = Runtime::Dxgi, .swap_chain = SwapChain, .flags = 0, .sync_interval = 1}},
             {1, Kind::Flip{.interval = 1, .mmio = true}},
             {2, Kind::QueuePacketStart{.packet_type = 3, .submit_sequence = sequence, .context = 0x5, .present = false}},
             {3, Kind::PresentInfo{.window = Window}},
             {4, Kind::PresentStop{.runtime = Runtime::Dxgi, .result = 0}},
         }) {
        tracker.handle(event(start + offset, Process, Thread, kind));
    }
    tracker.handle(event(start + 10, 0, 0, Kind::MmioFlip{.submit_sequence = sequence, .flags = 0}));
    tracker.handle(event(start + 20, 0, 0, Kind::VSyncDpc{.fence = std::uint64_t{sequence} << 32}));
}

} // namespace

TEST(a_fullscreen_exclusive_flip_is_a_legacy_flip_on_the_screen_at_the_sync) {
    Tracker tracker(Process);
    legacy_flip(tracker, 100, 1);
    legacy_flip(tracker, 200, 2);
    CHECK_EQ(tracker.take(), (std::vector<Present>{Present{
                                 .start = 200,
                                 .thread = Thread,
                                 .mode = PresentMode::LegacyFlip,
                                 .tearing = false,
                                 .screen = 220,
                                 .flip_delay = 0,
                             }}));
}

namespace {

// What a bitblt model swap chain's Present call sends: its copy, and its
// queue packet, and, for a copy to the window's surface for DWM, a present
// history token, whose data is the flip chain and serial number that DWM's
// events name a copy made on the CPU by.
void blit(Tracker& tracker, std::int64_t start, std::uint32_t sequence, bool redirected, std::optional<std::uint64_t> history) {
    std::vector<Kind> kinds{
        Kind::PresentStart{.runtime = Runtime::Dxgi, .swap_chain = SwapChain, .flags = 0, .sync_interval = 0},
        Kind::Blit{.window = Window, .redirected = redirected},
    };
    if (history) {
        kinds.push_back(Kind::PresentHistoryStart{.token = 1000 + std::uint64_t{sequence}, .model = 3, .token_data = *history});
    }
    kinds.push_back(Kind::QueuePacketStart{.packet_type = 0, .submit_sequence = sequence, .context = 0x5, .present = true});
    kinds.push_back(Kind::PresentInfo{.window = Window});
    kinds.push_back(Kind::PresentStop{.runtime = Runtime::Dxgi, .result = 0});
    for (size_t offset = 0; offset < kinds.size(); offset++) {
        tracker.handle(event(start + static_cast<std::int64_t>(offset), Process, Thread, std::move(kinds[offset])));
    }
}

} // namespace

TEST(a_fullscreen_exclusive_copy_is_on_the_screen_when_its_queue_packet_is_done) {
    Tracker tracker(Process);
    torn(tracker, 100, 1);
    tracker.take();
    blit(tracker, 200, 2, false, std::nullopt);
    tracker.handle(event(230, 0, 0, Kind::QueuePacketStop{.context = 0x5, .submit_sequence = 2}));
    CHECK_EQ(tracker.take(), (std::vector<Present>{Present{
                                 .start = 200,
                                 .thread = Thread,
                                 .mode = PresentMode::LegacyCopy,
                                 .tearing = true,
                                 .screen = 230,
                                 .flip_delay = 0,
                             }}));
}

TEST(a_copy_whose_queue_packet_is_done_before_its_present_info_waits_for_it) {
    Tracker tracker(Process);
    torn(tracker, 100, 1);
    tracker.take();
    for (const auto& [time, kind] : std::initializer_list<std::pair<std::int64_t, Kind>>{
             {200, Kind::PresentStart{.runtime = Runtime::Dxgi, .swap_chain = SwapChain, .flags = 0, .sync_interval = 0}},
             {201, Kind::Blit{.window = Window, .redirected = false}},
             {202, Kind::QueuePacketStart{.packet_type = 0, .submit_sequence = 2, .context = 0x5, .present = true}},
         }) {
        tracker.handle(event(time, Process, Thread, kind));
    }
    tracker.handle(event(203, 0, 0, Kind::QueuePacketStop{.context = 0x5, .submit_sequence = 2}));
    tracker.handle(event(204, Process, Thread, Kind::PresentInfo{.window = Window}));
    tracker.handle(event(205, Process, Thread, Kind::PresentStop{.runtime = Runtime::Dxgi, .result = 0}));
    std::vector<Present> presents = tracker.take();
    CHECK_EQ(presents.size(), 1);
    CHECK_EQ(presents[0].mode, PresentMode::LegacyCopy);
    CHECK_EQ(presents[0].screen, std::optional<std::int64_t>(203));
}

TEST(a_copy_that_the_kernel_cancels_never_reaches_the_screen) {
    Tracker tracker(Process);
    torn(tracker, 100, 1);
    tracker.take();
    for (const auto& [time, kind] : std::initializer_list<std::pair<std::int64_t, Kind>>{
             {200, Kind::PresentStart{.runtime = Runtime::Dxgi, .swap_chain = SwapChain, .flags = 0, .sync_interval = 0}},
             {201, Kind::Blit{.window = Window, .redirected = false}},
             {202, Kind::BlitCancel{}},
             {203, Kind::PresentStop{.runtime = Runtime::Dxgi, .result = 0}},
         }) {
        tracker.handle(event(time, Process, Thread, kind));
    }
    std::vector<Present> presents = tracker.take();
    CHECK_EQ(presents.size(), 1);
    CHECK_EQ(presents[0].screen, std::nullopt);
}

TEST(a_copy_made_on_the_gpu_for_dwm_is_on_the_screen_with_dwm_s_frame) {
    Tracker tracker(Process);
    tracker.handle(event(10, DwmProcess, DwmThread, Kind::DwmSchedulePresent{}));
    torn(tracker, 100, 1);
    tracker.take();
    blit(tracker, 200, 2, false, 0);
    tracker.handle(event(210, 0, 0, Kind::PresentHistoryInfo{.token = 1002}));
    tracker.handle(event(290, DwmProcess, DwmThread, Kind::DwmGetPresentHistory{}));
    dwm_frame(tracker, 300, 900);
    CHECK_EQ(tracker.take(), (std::vector<Present>{Present{
                                 .start = 200,
                                 .thread = Thread,
                                 .mode = PresentMode::ComposedGpuCopy,
                                 .tearing = false,
                                 .screen = 310,
                                 .flip_delay = 0,
                             }}));
}

TEST(of_two_copies_in_one_dwm_frame_only_the_newer_is_on_the_screen) {
    Tracker tracker(Process);
    tracker.handle(event(10, DwmProcess, DwmThread, Kind::DwmSchedulePresent{}));
    torn(tracker, 100, 1);
    tracker.take();
    blit(tracker, 200, 2, false, 0);
    tracker.handle(event(210, 0, 0, Kind::PresentHistoryInfo{.token = 1002}));
    blit(tracker, 250, 3, false, 0);
    tracker.handle(event(260, 0, 0, Kind::PresentHistoryInfo{.token = 1003}));
    tracker.handle(event(290, DwmProcess, DwmThread, Kind::DwmGetPresentHistory{}));
    dwm_frame(tracker, 300, 900);
    std::vector<Present> presents = tracker.take();
    std::ranges::stable_sort(presents, {}, &Present::start);
    std::vector<std::pair<std::int64_t, std::optional<std::int64_t>>> screens;
    for (const Present& present : presents) {
        screens.emplace_back(present.start, present.screen);
    }
    CHECK_EQ(screens, (std::vector<std::pair<std::int64_t, std::optional<std::int64_t>>>{{200, std::nullopt}, {250, 310}}));
}

TEST(a_copy_made_on_the_cpu_for_dwm_is_on_the_screen_with_dwm_s_frame) {
    Tracker tracker(Process);
    tracker.handle(event(10, DwmProcess, DwmThread, Kind::DwmSchedulePresent{}));
    torn(tracker, 100, 1);
    tracker.take();
    std::uint64_t token_data = (std::uint64_t{7} << 32) | 2;
    blit(tracker, 200, 2, true, token_data);
    tracker.handle(event(210, 0, 0, Kind::PresentHistoryInfo{.token = 1002}));
    tracker.handle(event(280, DwmProcess, DwmThread, Kind::DwmFlipChain{.token_data = token_data, .window = Window}));
    tracker.handle(event(290, DwmProcess, DwmThread, Kind::DwmGetPresentHistory{}));
    dwm_frame(tracker, 300, 900);
    CHECK_EQ(tracker.take(), (std::vector<Present>{Present{
                                 .start = 200,
                                 .thread = Thread,
                                 .mode = PresentMode::ComposedCpuCopy,
                                 .tearing = false,
                                 .screen = 310,
                                 .flip_delay = 0,
                             }}));
}

TEST(nvidia_s_flip_delay_puts_a_flip_later) {
    Tracker tracker(Process);
    torn(tracker, 100, 1);
    tracker.take();
    present(tracker, 200, 2, AllowTearing);
    in_frame(tracker, 206, 2, true);
    // The driver means to flip at 230, 22 ticks after its request, which the
    // kernel's event on the same thread then comes 2 ticks after.
    tracker.handle(event(208, 0, 33, Kind::NvidiaFlipRequest{.alloc = 0, .source = 0, .time = 230, .token = 1}));
    tracker.handle(event(210, 0, 33, Kind::MmioFlipMultiPlaneOverlay{.submit_sequence = 2, .status = FlipWaitComplete}));
    sync_dpc(tracker, 240, {2});
    std::vector<Present> presents = tracker.take();
    CHECK_EQ(presents[0].flip_delay, 22);
    CHECK_EQ(presents[0].screen, std::optional<std::int64_t>(232));
}

TEST(a_flip_delayed_past_the_next_puts_the_next_on_the_screen_with_it) {
    auto present = [](std::int64_t start, std::int64_t screen, std::int64_t flip_delay) {
        return Present{
            .start = start,
            .thread = Thread,
            .mode = PresentMode::IndependentFlip,
            .tearing = true,
            .screen = screen,
            .flip_delay = flip_delay,
        };
    };
    // In the order they started, whatever order they came in.
    Corrector corrector;
    std::vector<Present> corrected = corrector.correct({present(200, 260, 50), present(100, 150, 0)});
    std::vector<std::int64_t> starts;
    for (const Present& one : corrected) {
        starts.push_back(one.start);
    }
    CHECK_EQ(starts, (std::vector<std::int64_t>{100, 200}));
    // The one after the delayed one goes on the screen with it.
    corrected = corrector.correct({present(300, 250, 0)});
    CHECK_EQ(corrected[0].screen, std::optional<std::int64_t>(260));
    CHECK_EQ(corrected[0].flip_delay, 10);
    // And so does the one after that, which now has a delay too.
    corrected = corrector.correct({present(400, 255, 0)});
    CHECK_EQ(corrected[0].screen, std::optional<std::int64_t>(260));
    // But not after one without a delay.
    Corrector another;
    another.correct({present(200, 260, 0)});
    corrected = another.correct({present(300, 250, 0)});
    CHECK_EQ(corrected[0].screen, std::optional<std::int64_t>(250));
}
