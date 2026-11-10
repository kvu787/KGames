// A real-time ETW session with the providers that Events.h lists, whose
// events a thread of its own reads as they come, and hands to a callback.
// The session is the system's: it outlives the process if the process ends
// without stopping it, like when the process is killed. So a session's name
// has its process's ID, and a session whose process is gone is stopped
// before a new one starts, while another process's live session is left
// alone.
//
// ETW hands a real-time session's events to its reader when a buffer fills,
// or when the session's flush timer goes off, at most once a second. So a
// thread of its own flushes the session's buffers every FlushPeriod, as
// Intel's PresentMon service does for its overlay, so that the events come
// 7 to 52 ms after they happen, 13 in the median, as measured, instead of 1
// to 2 seconds later.

#pragma once

#include "Base/Windows.h"
#include "PresentTiming/Present.h"

#include <evntcons.h>
#include <evntrace.h>

#include <atomic>
#include <cstdint>
#include <expected>
#include <filesystem>
#include <functional>
#include <memory>
#include <optional>
#include <string>
#include <string_view>
#include <thread>

namespace present_timing::session {

// What the reading thread hands each event to.
using Callback = std::move_only_function<void(const EVENT_RECORD&)>;

class Session {
public:
    // Starts the session, named after the program and this process, enables
    // the providers, and starts reading.
    static std::expected<Session, StartError> start(std::string_view program, Callback callback);

    // The session goes with the one that it's moved to, which stops it.
    Session(Session&& other) noexcept;
    Session& operator=(Session&& other) noexcept;
    Session(const Session&) = delete;
    Session& operator=(const Session&) = delete;

    ~Session();

private:
    // The thread that flushes the session's buffers, and what tells it to stop.
    struct Flusher {
        std::shared_ptr<std::atomic<bool>> stop_;
        std::thread thread;

        // Flushes the session every FlushPeriod, on a fixed cadence, until
        // it's stopped. Without the thread, if it can't start, the events
        // still come, with the flush timer.
        static std::optional<Flusher> start(CONTROLTRACE_ID control);

        void stop() &&;
    };

    Session(CONTROLTRACE_ID control, std::wstring name, PROCESSTRACE_HANDLE trace, std::thread thread, std::optional<Flusher> flusher);

    // Stops the session, and waits for its threads: what the destructor does.
    void drop() noexcept;

    CONTROLTRACE_ID control;
    // In UTF-16, for the Windows API.
    std::wstring name;
    PROCESSTRACE_HANDLE trace;
    // Nothing in a session that was moved from, which has nothing to stop.
    std::optional<std::thread> thread;
    std::optional<Flusher> flusher;
};

// Reads the events in a trace file, like one that xperf or WPR saved, in
// order, and gives the performance counter's frequency on the computer that
// saved it.
std::expected<std::int64_t, std::string> read_file(const std::filesystem::path& path, Callback callback);

// The performance counter's frequency on the computer that saved a trace
// file, from its header, if its events are timed on the performance counter.
// The header's ReservedFlags is the session's ClientContext, which says the
// clock: 1 for the performance counter, 2 for the system time, and 3 for the
// CPU's cycles, which would give wrong times here.
std::expected<std::int64_t, std::string> file_clock(std::uint32_t reserved_flags, std::int64_t performance_frequency);

} // namespace present_timing::session
