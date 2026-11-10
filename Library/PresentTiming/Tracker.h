// Follows presents through the events about them, to when the display took
// each, by the rules that PresentMon 2.6.1's consumer follows (its
// PresentMonTraceConsumer.cpp), so that the times and the modes are the same
// as its, for every mode that it tells apart: fullscreen exclusive flips and
// copies, the display taking a flip model swap chain's buffer itself
// (independent flip), doing that on an overlay plane, DWM composing the
// buffer into its own frame, and DWM composing a bitblt model swap chain's
// copy, made on the GPU or on the CPU. It follows the process's presents,
// and DWM's, whose frames show the composed ones. Like PresentMon, it leaves
// out DirectComposition's presents to composition swap chains, which it
// can't follow to DWM's frames.
//
// A present is found by different keys as it goes: its thread while its
// Present call runs, Win32k's token for it, the kernel's present history
// token and that token's data, its window, and the submit sequence of its
// queue packet, which the flips and the display's sync interrupts (DPCs)
// name.

#pragma once

#include "PresentTiming/Events.h"
#include "PresentTiming/Present.h"

#include <cstddef>
#include <cstdint>
#include <deque>
#include <map>
#include <optional>
#include <tuple>
#include <unordered_map>
#include <utility>
#include <vector>

namespace present_timing::tracker {

class Tracker {
public:
    explicit Tracker(std::uint32_t process);

    Tracker(Tracker&&) = default;
    Tracker& operator=(Tracker&&) = default;
    Tracker(const Tracker&) = delete;
    Tracker& operator=(const Tracker&) = delete;

    // The process's presents whose fate became known since the last call.
    std::vector<Present> take();

    void handle(const events::Event& event);

private:
    // PresentMon's names for the modes.
    enum class Mode {
        Unknown,
        LegacyFlip,
        LegacyCopyToFrontBuffer,
        IndependentFlip,
        ComposedIndependentFlip,
        ComposedFlip,
        ComposedCopyGpuGdi,
        ComposedCopyCpuGdi,
    };

    // Whether DWM shows only the newest of a window's presents.
    static bool composed(Mode mode);

    enum class Final {
        Unknown,
        Presented,
        Discarded,
    };

    // Win32k's token for a present.
    using Key = std::tuple<std::uint64_t, std::uint64_t, std::uint64_t>;

    struct KeyHash {
        size_t operator()(const Key& key) const noexcept;
    };

    struct Tracked {
        std::int64_t start;
        std::uint32_t process;
        std::uint32_t thread;
        std::uint32_t driver_thread;
        std::int64_t time_in_present;
        // The API whose Present_Start made it, or nothing if a kernel event
        // did, like for an API without events of its own, like OpenGL or
        // Vulkan.
        std::optional<events::Runtime> runtime;
        std::uint64_t swap_chain;
        std::uint32_t flags;
        std::uint32_t submit_sequence;
        std::optional<Key> win32k_key;
        std::uint64_t token;
        // A copy made on the CPU's present history token data, which DWM's
        // flip chain events name.
        std::uint64_t token_data;
        std::uint64_t window;
        size_t ring;
        Mode mode;
        Final final_state;
        std::optional<std::int64_t> screen;
        bool tearing;
        std::int64_t flip_delay;
        std::vector<std::uint64_t> dependents;
        bool wait_for_flip;
        bool wait_for_mpo_flip;
        bool wait_for_sync_dpc;
        bool seen_dxgk_present;
        bool seen_win32k;
        bool seen_in_frame;
        bool completed;
        bool lost;
        bool failed;
        bool waiting_for_dwm;
        bool waiting_for_present_stop;
    };

    bool tracked(std::uint32_t process_id) const;
    const Tracked& get(std::uint64_t id) const;
    Tracked& get_mut(std::uint64_t id);
    std::uint64_t create(const events::Event& event, std::optional<events::Runtime> runtime);
    void track(std::uint64_t id);
    void set_thread_present(std::uint32_t thread, std::uint64_t id);
    // The present that a kernel or Win32k event on a thread is about: the
    // thread's present, or the oldest that hasn't been seen by the kernel
    // yet, which a driver's thread presents later, or else a new one.
    std::optional<std::uint64_t> find_or_create(const events::Event& event);
    // The oldest present with the submit sequence, of any queue.
    std::optional<std::uint64_t> by_submit(std::uint32_t sequence) const;
    void remove_from_submit_sequence(std::uint64_t id);
    void set_screen_time(std::uint64_t id, std::int64_t time);
    void present_stop(const events::Event& event, events::Runtime runtime, std::uint32_t result);
    void present_history_start(const events::Event& event, std::uint64_t token, std::uint32_t model, std::uint64_t token_data);
    // A flip that DWM, or a legacy fullscreen present, made: the present
    // that it's of.
    std::optional<std::uint64_t> flip(const events::Event& event);
    void token_state_changed(std::uint64_t id, Key key, std::uint32_t state, bool independent_flip);
    void remove_lost(std::uint64_t id);
    void unmap_thread(std::uint32_t thread, std::uint64_t id);
    void complete(std::uint64_t id);
    void stop_tracking(std::uint64_t id);
    // Hands on the completed presents at the front that no longer wait for
    // their Present_Stop: the process's own, of any API, as Presents.
    void emit_ready();

    // The process whose presents are reported.
    std::uint32_t process;
    std::unordered_map<std::uint64_t, Tracked> presents;
    std::uint64_t next_id = 1;
    std::vector<std::optional<std::uint64_t>> ring;
    size_t next_ring = 0;
    std::unordered_map<std::uint32_t, std::uint64_t> by_thread;
    std::unordered_map<std::uint32_t, std::map<std::int64_t, std::uint64_t>> ordered;
    // By submit sequence, then by the queue's context, in the order added.
    std::unordered_map<std::uint32_t, std::vector<std::pair<std::uint64_t, std::uint64_t>>> by_submit_sequence;
    std::unordered_map<Key, std::uint64_t, KeyHash> by_win32k_key;
    std::unordered_map<std::uint64_t, std::uint64_t> by_token;
    std::unordered_map<std::uint64_t, std::uint64_t> by_token_data;
    std::unordered_map<std::uint64_t, std::uint64_t> last_by_window;
    std::vector<std::uint64_t> waiting_for_dwm;
    std::uint32_t dwm_process = 0;
    std::uint32_t dwm_thread = 0;
    bool completed_one = false;
    // Completed presents, in the order they completed.
    std::deque<std::uint64_t> completed;
    // NVIDIA's flip requests.
    std::uint32_t last_flip_token = 0;
    std::unordered_map<std::uint32_t, std::int64_t> last_flip_time;
    std::unordered_map<std::uint32_t, std::pair<std::int64_t, std::uint32_t>> flip_requests;
    // The process's presents that are known, in the order they completed.
    std::vector<Present> output;
};

} // namespace present_timing::tracker
