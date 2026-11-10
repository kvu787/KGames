#include "PresentTiming/Tracker.h"

#include "PresentTiming/PresentTiming.h"

#include <algorithm>
#include <functional>
#include <ranges>

namespace present_timing::tracker {

using events::Confirmed;
using events::Discarded;
using events::Event;
using events::InFrame;
using events::Kind;
using events::Runtime;

namespace {

// How many presents can be followed at once, before the oldest is lost.
constexpr size_t Ring = 2048;

// DXGI's Present flags and results, and Direct3D 9's result for an
// occluded window.
constexpr std::uint32_t PresentTest = 0x1;
constexpr std::uint32_t PresentDoNotSequence = 0x2;
constexpr std::uint32_t StatusOccluded = 0x087A'0001;
constexpr std::uint32_t StatusNoDesktopAccess = 0x087A'0005;
constexpr std::uint32_t StatusModeChangeInProgress = 0x087A'0008;
constexpr std::uint32_t D3d9PresentOccluded = 0x0876'0878;

// The kernel's flip statuses, and present models.
constexpr std::uint32_t FlipWaitVsync = 5;
constexpr std::uint32_t FlipWaitHsync = 15;
constexpr std::uint32_t FlipImmediate = 0x2;
constexpr std::uint32_t ModelRedirectedGdi = 1;
constexpr std::uint32_t ModelRedirectedComposition = 7;
constexpr std::uint32_t PacketMmioFlip = 3;
constexpr std::uint32_t PacketSoftware = 7;

// The map's value for the key, if it has one.
template <class Map, class Key> std::optional<typename Map::mapped_type> lookup(const Map& map, const Key& key) {
    auto found = map.find(key);
    if (found == map.end()) {
        return std::nullopt;
    }
    return found->second;
}

} // namespace

bool Tracker::composed(Mode mode) {
    return mode == Mode::ComposedFlip || mode == Mode::ComposedCopyGpuGdi || mode == Mode::ComposedCopyCpuGdi;
}

size_t Tracker::KeyHash::operator()(const Key& key) const noexcept {
    size_t hash = 0;
    for (std::uint64_t part : {std::get<0>(key), std::get<1>(key), std::get<2>(key)}) {
        hash ^= std::hash<std::uint64_t>()(part) + 0x9e3779b97f4a7c15ull + (hash << 6) + (hash >> 2);
    }
    return hash;
}

Tracker::Tracker(std::uint32_t process) : process(process), ring(Ring) {}

std::vector<Present> Tracker::take() {
    return std::exchange(output, {});
}

bool Tracker::tracked(std::uint32_t process_id) const {
    return process_id == process || (dwm_process != 0 && process_id == dwm_process);
}

const Tracker::Tracked& Tracker::get(std::uint64_t id) const {
    return presents.at(id);
}

Tracker::Tracked& Tracker::get_mut(std::uint64_t id) {
    return presents.at(id);
}

void Tracker::handle(const Event& event) {
    base::match(
        event.kind,
        [&](const Kind::PresentStart& kind) {
            if (tracked(event.process) && (kind.flags & PresentTest) == 0) {
                std::uint64_t id = create(event, kind.runtime);
                Tracked& present = get_mut(id);
                present.swap_chain = kind.swap_chain;
                present.flags = kind.flags;
                track(id);
            }
        },
        [&](const Kind::PresentStop& kind) {
            if (tracked(event.process)) {
                present_stop(event, kind.runtime, kind.result);
            }
        },
        [&](const Kind::TokenCompositionSurfaceObject& kind) {
            while (std::optional<std::uint64_t> id = find_or_create(event)) {
                if (get(*id).seen_win32k) {
                    remove_lost(*id);
                    continue;
                }
                Tracked& present = get_mut(*id);
                present.mode = Mode::ComposedFlip;
                present.seen_win32k = true;
                present.win32k_key = kind.key;
                by_win32k_key.insert_or_assign(kind.key, *id);
                break;
            }
        },
        [&](const Kind::PresentHistoryStart& kind) {
            if (kind.model != ModelRedirectedGdi) {
                present_history_start(event, kind.token, kind.model, kind.token_data);
            }
        },
        [&](const Kind::PresentHistoryInfo& kind) {
            if (std::optional<std::uint64_t> id = lookup(by_token, kind.token)) {
                Tracked& present = get_mut(*id);
                if (present.mode == Mode::ComposedFlip && !present.seen_win32k && !present.waiting_for_dwm) {
                    present.waiting_for_dwm = true;
                    waiting_for_dwm.push_back(*id);
                }
                // A copy made on the GPU goes to DWM, which takes the
                // newest of its window's.
                const Tracked& copy = get(*id);
                if (copy.mode == Mode::ComposedCopyGpuGdi) {
                    std::uint64_t window = copy.window;
                    last_by_window.insert_or_assign(window, *id);
                }
                by_token.erase(kind.token);
            }
        },
        [&](const Kind::QueuePacketStart& kind) {
            if (kind.packet_type == PacketMmioFlip || kind.packet_type == PacketSoftware || kind.present) {
                std::optional<std::uint64_t> id = lookup(by_thread, event.thread);
                if (id && get(*id).submit_sequence == 0) {
                    get_mut(*id).submit_sequence = kind.submit_sequence;
                    by_submit_sequence[kind.submit_sequence].emplace_back(kind.context, *id);
                }
            }
        },
        [&](const Kind::QueuePacketStop& kind) {
            std::optional<std::uint64_t> found;
            if (auto entries = by_submit_sequence.find(kind.submit_sequence); entries != by_submit_sequence.end()) {
                for (const auto& [known, id] : entries->second) {
                    if (known == kind.context) {
                        found = id;
                        break;
                    }
                }
            }
            if (found) {
                std::uint64_t id = *found;
                const Tracked& present = get(id);
                bool copy = present.mode == Mode::LegacyCopyToFrontBuffer;
                if (copy || (present.mode == Mode::LegacyFlip && !present.wait_for_flip)) {
                    set_screen_time(id, event.time);
                    // A copy whose queue packet is done before its
                    // Present_Info could still turn out to be a copy for
                    // DWM, so it waits for that.
                    if (!copy || get(id).seen_dxgk_present) {
                        complete(id);
                    }
                }
            }
        },
        [&](const Kind::PresentInfo& kind) {
            if (std::optional<std::uint64_t> id = lookup(by_thread, event.thread)) {
                Tracked& present = get_mut(*id);
                present.seen_dxgk_present = true;
                if (present.window == 0) {
                    present.window = kind.window;
                }
                if (!present.runtime || present.thread != event.thread) {
                    by_thread.erase(event.thread);
                }
                const Tracked& copy = get(*id);
                if (copy.mode == Mode::LegacyCopyToFrontBuffer && copy.screen) {
                    complete(*id);
                }
            }
        },
        [&](const Kind::Blit& kind) {
            while (std::optional<std::uint64_t> id = find_or_create(event)) {
                if (get(*id).mode != Mode::Unknown) {
                    remove_lost(*id);
                    continue;
                }
                // A copy to the screen, until a present history token
                // says that it's to the window's surface for DWM.
                Tracked& present = get_mut(*id);
                present.window = kind.window;
                present.mode = kind.redirected ? Mode::ComposedCopyCpuGdi : Mode::LegacyCopyToFrontBuffer;
                present.tearing = !kind.redirected;
                break;
            }
        },
        [&](const Kind::BlitCancel&) {
            if (std::optional<std::uint64_t> id = lookup(by_thread, event.thread)) {
                get_mut(*id).final_state = Final::Discarded;
                complete(*id);
            }
        },
        [&](const Kind::Flip& kind) {
            if (std::optional<std::uint64_t> id = flip(event)) {
                Tracked& present = get_mut(*id);
                if (kind.mmio) {
                    present.wait_for_flip = true;
                } else if (kind.interval == 0) {
                    present.tearing = true;
                }
            }
        },
        [&](const Kind::FlipMultiPlaneOverlay& kind) {
            if (std::optional<std::uint64_t> id = flip(event)) {
                Tracked& present = get_mut(*id);
                present.wait_for_flip = true;
                present.wait_for_mpo_flip = true;
                if (present.swap_chain == 0) {
                    present.swap_chain = (static_cast<std::uint64_t>(kind.source) << 32) | static_cast<std::uint64_t>(kind.layer);
                }
            }
        },
        [&](const Kind::IndependentFlip& kind) {
            if (std::optional<std::uint64_t> id = by_submit(kind.submit_sequence)) {
                get_mut(*id).mode = Mode::IndependentFlip;
            }
        },
        [&](const Kind::MmioFlip& kind) {
            if (std::optional<std::uint64_t> id = by_submit(kind.submit_sequence)) {
                Tracked& present = get_mut(*id);
                if (present.mode == Mode::ComposedFlip) {
                    present.mode = Mode::IndependentFlip;
                }
                if ((kind.flags & FlipImmediate) != 0) {
                    set_screen_time(*id, event.time);
                    get_mut(*id).tearing = true;
                    if (get(*id).mode == Mode::LegacyFlip) {
                        complete(*id);
                    }
                }
            }
        },
        [&](const Kind::MmioFlipMultiPlaneOverlay& kind) {
            std::optional<std::uint64_t> found = by_submit(kind.submit_sequence);
            if (found && kind.status) {
                std::uint64_t id = *found;
                switch (*kind.status) {
                case FlipWaitVsync:
                    get_mut(id).wait_for_sync_dpc = true;
                    break;
                case FlipWaitHsync: {
                    Tracked& present = get_mut(id);
                    present.wait_for_sync_dpc = true;
                    present.tearing = true;
                    break;
                }
                default: {
                    get_mut(id).tearing = true;
                    // NVIDIA's driver may flip later than the kernel's
                    // event, by the delay that it asked for.
                    if (auto request = lookup(flip_requests, event.thread)) {
                        get_mut(id).flip_delay = request->first;
                        flip_requests.clear();
                    }
                    std::int64_t delay = get(id).flip_delay;
                    set_screen_time(id, event.time + delay);
                    if (get(id).mode == Mode::LegacyFlip) {
                        complete(id);
                    }
                }
                }
            }
        },
        [&](const Kind::VSyncDpc& kind) {
            if (kind.fence != 0) {
                if (std::optional<std::uint64_t> id = by_submit(static_cast<std::uint32_t>(kind.fence >> 32))) {
                    set_screen_time(*id, event.time);
                    const Tracked& present = get(*id);
                    if (present.mode == Mode::LegacyFlip && !present.wait_for_mpo_flip) {
                        complete(*id);
                    }
                }
            }
        },
        [&](const Kind::SyncDpcMultiPlane& kind) {
            if (!kind.flip_sequences.empty()) {
                bool multi_plane = std::ranges::count_if(kind.planes, [](std::uint64_t address) { return address != 0; }) > 1;
                for (std::uint64_t sequence : kind.flip_sequences) {
                    if (sequence == 0) {
                        continue;
                    }
                    if (std::optional<std::uint64_t> id = by_submit(static_cast<std::uint32_t>(sequence >> 32))) {
                        Tracked& present = get_mut(*id);
                        if (multi_plane && (present.mode == Mode::IndependentFlip || present.mode == Mode::ComposedFlip)) {
                            present.mode = Mode::ComposedIndependentFlip;
                        }
                        if (present.final_state != Final::Presented) {
                            set_screen_time(*id, event.time);
                        }
                        complete(*id);
                    }
                }
            }
        },
        [&](const Kind::TokenStateChanged& kind) {
            if (std::optional<std::uint64_t> id = lookup(by_win32k_key, kind.key)) {
                token_state_changed(*id, kind.key, kind.state, kind.independent_flip);
            }
        },
        [&](const Kind::DwmSchedulePresent&) {
            dwm_process = event.process;
            dwm_thread = event.thread;
        },
        [&](const Kind::DwmGetPresentHistory&) {
            // The newest copy of each window goes in DWM's next frame.
            std::vector<std::uint64_t> copies;
            for (std::uint64_t id : last_by_window | std::views::values) {
                Mode mode = get(id).mode;
                if (mode == Mode::ComposedCopyGpuGdi || mode == Mode::ComposedCopyCpuGdi) {
                    copies.push_back(id);
                }
            }
            std::ranges::stable_sort(copies, {}, [&](std::uint64_t id) { return get(id).start; });
            for (std::uint64_t id : copies) {
                get_mut(id).waiting_for_dwm = true;
                waiting_for_dwm.push_back(id);
            }
            last_by_window.clear();
        },
        [&](const Kind::DwmFlipChain& kind) {
            if (auto found = by_token_data.find(kind.token_data); found != by_token_data.end()) {
                std::uint64_t id = found->second;
                by_token_data.erase(found);
                get_mut(id).token_data = 0;
                last_by_window.insert_or_assign(kind.window, id);
            }
        },
        [&](const Kind::DwmScheduleSurfaceUpdate& kind) {
            if (std::optional<std::uint64_t> id = lookup(by_win32k_key, kind.key)) {
                Tracked& present = get_mut(*id);
                if (present.seen_in_frame && !present.waiting_for_dwm) {
                    present.waiting_for_dwm = true;
                    waiting_for_dwm.push_back(*id);
                }
            }
        },
        [&](const Kind::NvidiaFlipRequest& kind) {
            if (kind.token == last_flip_token) {
                return;
            }
            last_flip_token = kind.token;
            std::int64_t proposed = 0;
            std::int64_t delay = 0;
            if (kind.alloc == 0) {
                proposed = static_cast<std::int64_t>(kind.time);
                delay = std::max<std::int64_t>(proposed - event.time, 0);
                std::optional<std::int64_t> last = lookup(last_flip_time, kind.source);
                if (last && *last != 0 && proposed < *last) {
                    delay = *last - proposed;
                    proposed = *last;
                }
            }
            last_flip_time.insert_or_assign(kind.source, proposed);
            flip_requests.try_emplace(event.thread, delay, kind.token);
        });
}

std::uint64_t Tracker::create(const Event& event, std::optional<Runtime> runtime) {
    std::uint64_t id = next_id;
    next_id += 1;
    presents.insert_or_assign(id, Tracked{
                                      .start = event.time,
                                      .process = event.process,
                                      .thread = event.thread,
                                      .driver_thread = 0,
                                      .time_in_present = 0,
                                      .runtime = runtime,
                                      .swap_chain = 0,
                                      .flags = 0,
                                      .submit_sequence = 0,
                                      .win32k_key = std::nullopt,
                                      .token = 0,
                                      .token_data = 0,
                                      .window = 0,
                                      .ring = 0,
                                      .mode = Mode::Unknown,
                                      .final_state = Final::Unknown,
                                      .screen = std::nullopt,
                                      .tearing = false,
                                      .flip_delay = 0,
                                      .dependents = {},
                                      .wait_for_flip = false,
                                      .wait_for_mpo_flip = false,
                                      .wait_for_sync_dpc = false,
                                      .seen_dxgk_present = false,
                                      .seen_win32k = false,
                                      .seen_in_frame = false,
                                      .completed = false,
                                      .lost = false,
                                      .failed = false,
                                      .waiting_for_dwm = false,
                                      .waiting_for_present_stop = false,
                                  });
    return id;
}

void Tracker::track(std::uint64_t id) {
    if (std::optional<std::uint64_t> old = ring[next_ring]) {
        remove_lost(*old);
    }
    ring[next_ring] = id;
    get_mut(id).ring = next_ring;
    next_ring = (next_ring + 1) % Ring;
    const Tracked& present = get(id);
    std::uint32_t process_id = present.process;
    std::int64_t start = present.start;
    std::uint32_t thread = present.thread;
    ordered[process_id].try_emplace(start, id);
    set_thread_present(thread, id);
}

void Tracker::set_thread_present(std::uint32_t thread, std::uint64_t id) {
    if (std::optional<std::uint64_t> old = lookup(by_thread, thread)) {
        remove_lost(*old);
    }
    by_thread.try_emplace(thread, id);
}

std::optional<std::uint64_t> Tracker::find_or_create(const Event& event) {
    if (std::optional<std::uint64_t> id = lookup(by_thread, event.thread)) {
        return id;
    }
    std::optional<std::uint64_t> deferred;
    if (auto found = ordered.find(event.process); found != ordered.end()) {
        for (std::uint64_t id : found->second | std::views::values) {
            const Tracked& present = presents.at(id);
            if (present.driver_thread == 0 && !present.seen_dxgk_present && !present.seen_win32k && present.mode == Mode::Unknown) {
                deferred = id;
                break;
            }
        }
    }
    if (deferred) {
        get_mut(*deferred).driver_thread = event.thread;
        set_thread_present(event.thread, *deferred);
        return deferred;
    }
    if (tracked(event.process)) {
        std::uint64_t id = create(event, std::nullopt);
        track(id);
        return id;
    }
    return std::nullopt;
}

std::optional<std::uint64_t> Tracker::by_submit(std::uint32_t sequence) const {
    auto entries = by_submit_sequence.find(sequence);
    if (entries == by_submit_sequence.end()) {
        return std::nullopt;
    }
    std::optional<std::uint64_t> oldest;
    std::int64_t oldest_start = 0;
    for (std::uint64_t id : entries->second | std::views::values) {
        std::int64_t start = presents.at(id).start;
        if (!oldest || start < oldest_start) {
            oldest = id;
            oldest_start = start;
        }
    }
    return oldest;
}

void Tracker::remove_from_submit_sequence(std::uint64_t id) {
    std::uint32_t sequence = get(id).submit_sequence;
    if (sequence != 0) {
        if (auto found = by_submit_sequence.find(sequence); found != by_submit_sequence.end()) {
            std::vector<std::pair<std::uint64_t, std::uint64_t>>& entries = found->second;
            if (entries.size() == 1) {
                if (entries[0].second == id) {
                    by_submit_sequence.erase(found);
                }
            } else if (auto entry = std::ranges::find(entries, id, &std::pair<std::uint64_t, std::uint64_t>::second); entry != entries.end()) {
                entries.erase(entry);
            }
        }
    }
    get_mut(id).submit_sequence = 0;
}

void Tracker::set_screen_time(std::uint64_t id, std::int64_t time) {
    Tracked& present = get_mut(id);
    if (!present.screen) {
        present.final_state = Final::Presented;
    }
    present.screen = time;
}

void Tracker::present_stop(const Event& event, Runtime runtime, std::uint32_t result) {
    std::optional<std::uint64_t> found = lookup(by_thread, event.thread);
    if (!found) {
        return;
    }
    std::uint64_t id = *found;
    Tracked& present = get_mut(id);
    present.runtime = runtime;
    present.time_in_present = event.time - present.start;
    if (present.waiting_for_present_stop) {
        present.waiting_for_present_stop = false;
        by_thread.erase(event.thread);
        emit_ready();
        return;
    }
    if (static_cast<std::int32_t>(result) < 0) {
        present.failed = true;
        complete(id);
        return;
    }
    bool occluded = false;
    switch (runtime) {
    case Runtime::Dxgi:
        occluded = result == StatusOccluded || result == StatusModeChangeInProgress || result == StatusNoDesktopAccess;
        break;
    case Runtime::D3d9:
        occluded = result == D3d9PresentOccluded;
        break;
    }
    if (occluded) {
        present.final_state = Final::Discarded;
        complete(id);
        return;
    }
    by_thread.erase(event.thread);
}

void Tracker::present_history_start(const Event& event, std::uint64_t token, std::uint32_t model, std::uint64_t token_data) {
    std::uint64_t id = 0;
    while (true) {
        std::optional<std::uint64_t> found = find_or_create(event);
        if (!found) {
            return;
        }
        if (get(*found).token != 0) {
            remove_lost(*found);
            continue;
        }
        id = *found;
        break;
    }
    Tracked& present = get_mut(id);
    present.tearing = false;
    present.token = token;
    if (present.screen) {
        present.screen = std::nullopt;
        present.final_state = Final::Unknown;
    }
    if (std::optional<std::uint64_t> other = lookup(by_token, token); other && *other != id) {
        remove_lost(*other);
    }
    by_token.insert_or_assign(token, id);
    switch (get(id).mode) {
    // The copy was to the window's surface, for DWM.
    case Mode::LegacyCopyToFrontBuffer:
        get_mut(id).mode = Mode::ComposedCopyGpuGdi;
        break;
    case Mode::Unknown:
        if (model == ModelRedirectedComposition) {
            remove_lost(id);
            return;
        }
        get_mut(id).mode = Mode::ComposedFlip;
        break;
    // DWM takes a copy made on the CPU by its token data, which its
    // flip chain events name, or else, without that, in its next
    // frame.
    case Mode::ComposedCopyCpuGdi:
        if (token_data == 0) {
            get_mut(id).waiting_for_dwm = true;
            waiting_for_dwm.push_back(id);
        } else {
            by_token_data.insert_or_assign(token_data, id);
            get_mut(id).token_data = token_data;
        }
        break;
    default:
        break;
    }
}

std::optional<std::uint64_t> Tracker::flip(const Event& event) {
    std::optional<std::uint64_t> usable;
    while (std::optional<std::uint64_t> id = lookup(by_thread, event.thread)) {
        const Tracked& present = get(*id);
        if (present.seen_dxgk_present || (present.submit_sequence != 0 && present.time_in_present != 0 && present.runtime)) {
            remove_lost(*id);
            continue;
        }
        if (present.runtime) {
            if (present.mode != Mode::Unknown) {
                return std::nullopt;
            }
            usable = id;
        } else {
            by_thread.erase(event.thread);
        }
        break;
    }
    std::uint64_t id = 0;
    if (usable) {
        id = *usable;
    } else if (tracked(event.process)) {
        id = create(event, std::nullopt);
        track(id);
    } else {
        return std::nullopt;
    }
    get_mut(id).mode = Mode::LegacyFlip;
    if (event.thread == dwm_thread) {
        std::vector<std::uint64_t> waiting = std::exchange(waiting_for_dwm, {});
        for (std::uint64_t dependent : waiting) {
            get_mut(dependent).waiting_for_dwm = false;
        }
        Tracked& present = get_mut(id);
        waiting_for_dwm = std::exchange(present.dependents, std::move(waiting));
        for (std::uint64_t dependent : std::vector<std::uint64_t>(waiting_for_dwm)) {
            get_mut(dependent).waiting_for_dwm = true;
        }
    }
    return id;
}

void Tracker::token_state_changed(std::uint64_t id, Key key, std::uint32_t state, bool independent_flip) {
    switch (state) {
    case InFrame: {
        Tracked& present = get_mut(id);
        present.seen_in_frame = true;
        if (independent_flip && present.mode == Mode::ComposedFlip) {
            present.mode = Mode::IndependentFlip;
        }
        std::uint64_t window = present.window;
        if (window != 0) {
            std::optional<std::uint64_t> previous = lookup(last_by_window, window);
            if (!previous) {
                last_by_window.insert_or_assign(window, id);
            } else if (*previous != id) {
                last_by_window.insert_or_assign(window, id);
                if (!get(*previous).wait_for_sync_dpc) {
                    if (!get(*previous).screen) {
                        get_mut(*previous).final_state = Final::Discarded;
                    }
                    remove_from_submit_sequence(*previous);
                }
            }
        }
        break;
    }
    case Confirmed: {
        const Tracked& present = get(id);
        if (present.final_state == Final::Unknown && (present.flags & PresentDoNotSequence) != 0) {
            get_mut(id).final_state = Final::Discarded;
            remove_from_submit_sequence(id);
        }
        std::uint64_t window = get(id).window;
        if (window != 0) {
            last_by_window.erase(window);
        }
        break;
    }
    case Discarded: {
        by_win32k_key.erase(key);
        Tracked& present = get_mut(id);
        present.win32k_key = std::nullopt;
        if (!present.seen_in_frame && present.final_state == Final::Unknown) {
            present.final_state = Final::Discarded;
            complete(id);
        } else if (present.mode != Mode::ComposedFlip) {
            complete(id);
        }
        break;
    }
    default:
        break;
    }
}

void Tracker::remove_lost(std::uint64_t id) {
    Tracked& present = get_mut(id);
    present.lost = true;
    if (present.completed) {
        if (present.waiting_for_present_stop) {
            present.waiting_for_present_stop = false;
            std::uint32_t thread = present.thread;
            std::uint32_t driver_thread = present.driver_thread;
            unmap_thread(thread, id);
            unmap_thread(driver_thread, id);
            emit_ready();
        }
    } else {
        complete(id);
    }
}

void Tracker::unmap_thread(std::uint32_t thread, std::uint64_t id) {
    if (auto found = by_thread.find(thread); found != by_thread.end() && found->second == id) {
        by_thread.erase(found);
    }
}

void Tracker::complete(std::uint64_t id) {
    // The first present that completes, and every one under way then,
    // are taken as lost, as their earlier events may not have been seen.
    if (!completed_one && !get(id).lost) {
        while (true) {
            std::optional<std::uint64_t> first;
            for (const std::map<std::int64_t, std::uint64_t>& starts : ordered | std::views::values) {
                if (!starts.empty()) {
                    first = starts.begin()->second;
                    break;
                }
            }
            if (!first) {
                break;
            }
            if (get(*first).completed) {
                // Already removed from the maps' view; take it out.
                stop_tracking(*first);
                continue;
            }
            remove_lost(*first);
        }
        completed_one = true;
        return;
    }
    if (get(id).completed) {
        return;
    }
    {
        Tracked& present = get_mut(id);
        present.completed = true;
        if (!present.lost && present.runtime && present.time_in_present == 0) {
            present.waiting_for_present_stop = true;
        }
    }

    // DWM's present: what it showed shows the composed presents with it,
    // only the newest of each window's.
    std::vector<std::uint64_t> dependents = std::exchange(get_mut(id).dependents, {});
    if (!dependents.empty()) {
        const Tracked& frame = get(id);
        bool lost = frame.lost;
        Final final_state = frame.final_state;
        std::optional<std::int64_t> screen = frame.screen;
        if (lost) {
            for (std::uint64_t dependent : dependents) {
                if (auto found = presents.find(dependent); found != presents.end()) {
                    found->second.lost = true;
                }
            }
        } else {
            std::vector<std::uint64_t> windows;
            for (std::uint64_t dependent : dependents | std::views::reverse) {
                // One that completed already, and was handed on, is gone.
                auto found = presents.find(dependent);
                if (found == presents.end()) {
                    continue;
                }
                Tracked& present = found->second;
                if (present.completed) {
                    continue;
                }
                bool is_composed = composed(present.mode);
                if (is_composed && std::ranges::find(windows, present.window) != windows.end()) {
                    present.final_state = Final::Discarded;
                    present.screen = std::nullopt;
                } else if (present.final_state != Final::Discarded) {
                    present.final_state = final_state;
                    if (screen) {
                        present.screen = screen;
                    }
                }
                if (is_composed) {
                    windows.push_back(present.window);
                }
            }
        }
        for (std::uint64_t dependent : dependents | std::views::reverse) {
            if (auto found = presents.find(dependent); found != presents.end() && !found->second.completed) {
                complete(dependent);
            }
        }
    }

    // A present on the screen completes the swap chain's earlier ones.
    if (get(id).final_state == Final::Presented) {
        const Tracked& present = get(id);
        std::uint32_t process_id = present.process;
        std::int64_t start = present.start;
        std::uint64_t swap_chain = present.swap_chain;
        bool lost = present.lost;
        std::vector<std::uint64_t> earlier;
        if (auto found = ordered.find(process_id); found != ordered.end()) {
            for (auto entry = found->second.begin(), end = found->second.lower_bound(start); entry != end; ++entry) {
                earlier.push_back(entry->second);
            }
        }
        for (std::uint64_t other : earlier) {
            bool still_ordered = false;
            if (auto found = ordered.find(process_id); found != ordered.end()) {
                still_ordered = std::ranges::any_of(found->second, [&](const auto& entry) { return entry.second == other; });
            }
            if (!still_ordered || get(other).swap_chain != swap_chain) {
                continue;
            }
            if (lost) {
                get_mut(other).lost = true;
            }
            complete(other);
        }
    }

    std::int64_t start = get(id).start;
    stop_tracking(id);
    completed.push_back(id);
    emit_ready();

    // A present that waits too long for its Present_Stop stops waiting.
    if (!completed.empty()) {
        std::uint64_t oldest = *completed.front();
        if (get(oldest).waiting_for_present_stop && start - get(oldest).start > 2 * frequency()) {
            Tracked& present = get_mut(oldest);
            present.waiting_for_present_stop = false;
            present.lost = true;
            stop_tracking(oldest);
            emit_ready();
        }
    }
}

void Tracker::stop_tracking(std::uint64_t id) {
    const Tracked& present = get(id);
    size_t ring_index = present.ring;
    std::uint32_t thread = present.thread;
    std::uint32_t driver_thread = present.driver_thread;
    std::uint32_t process_id = present.process;
    std::int64_t start = present.start;
    std::optional<Key> key = present.win32k_key;
    std::uint64_t token = present.token;
    std::uint64_t token_data = present.token_data;
    std::uint64_t window = present.window;
    bool waiting_for_stop = present.waiting_for_present_stop;
    bool waiting = present.waiting_for_dwm;
    if (ring[ring_index] == id) {
        ring[ring_index] = std::nullopt;
    }
    if (!waiting_for_stop) {
        unmap_thread(thread, id);
    }
    unmap_thread(driver_thread, id);
    if (auto found = ordered.find(process_id); found != ordered.end()) {
        if (auto entry = found->second.find(start); entry != found->second.end() && entry->second == id) {
            found->second.erase(entry);
        }
    }
    remove_from_submit_sequence(id);
    if (key) {
        if (auto found = by_win32k_key.find(*key); found != by_win32k_key.end() && found->second == id) {
            by_win32k_key.erase(found);
        }
    }
    if (token != 0) {
        if (auto found = by_token.find(token); found != by_token.end() && found->second == id) {
            by_token.erase(found);
        }
    }
    if (token_data != 0) {
        if (auto found = by_token_data.find(token_data); found != by_token_data.end() && found->second == id) {
            by_token_data.erase(found);
        }
    }
    if (window != 0) {
        if (auto found = last_by_window.find(window); found != last_by_window.end() && found->second == id) {
            last_by_window.erase(found);
        }
    }
    if (waiting) {
        std::erase(waiting_for_dwm, id);
    }
    Tracked& stopped = get_mut(id);
    stopped.win32k_key = std::nullopt;
    stopped.token = 0;
    stopped.token_data = 0;
    stopped.window = 0;
    stopped.waiting_for_dwm = false;
}

void Tracker::emit_ready() {
    while (!completed.empty()) {
        std::uint64_t id = *completed.front();
        if (get(id).waiting_for_present_stop) {
            break;
        }
        completed.pop_front();
        auto node = presents.extract(id);
        const Tracked& present = node.mapped();
        if (present.process == process && !present.lost && !present.failed) {
            PresentMode mode = PresentMode::Unknown;
            switch (present.mode) {
            case Mode::Unknown:
                mode = PresentMode::Unknown;
                break;
            case Mode::LegacyFlip:
                mode = PresentMode::LegacyFlip;
                break;
            case Mode::LegacyCopyToFrontBuffer:
                mode = PresentMode::LegacyCopy;
                break;
            case Mode::IndependentFlip:
                mode = PresentMode::IndependentFlip;
                break;
            case Mode::ComposedIndependentFlip:
                mode = PresentMode::OverlayPlane;
                break;
            case Mode::ComposedFlip:
                mode = PresentMode::ComposedFlip;
                break;
            case Mode::ComposedCopyGpuGdi:
                mode = PresentMode::ComposedGpuCopy;
                break;
            case Mode::ComposedCopyCpuGdi:
                mode = PresentMode::ComposedCpuCopy;
                break;
            }
            output.push_back(Present{
                .start = present.start,
                .thread = present.thread,
                .mode = mode,
                .tearing = present.tearing,
                .screen = present.final_state == Final::Presented ? present.screen : std::nullopt,
                .flip_delay = present.flip_delay,
            });
        }
    }
}

} // namespace present_timing::tracker
