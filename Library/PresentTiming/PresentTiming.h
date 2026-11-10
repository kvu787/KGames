// When each of a process's presents reached the screen, from Windows' own
// events about them, which a real-time ETW session reads as they happen, by
// the rules that PresentMon follows, so that the times are the same as its.
// See README.md.

#pragma once

#include "PresentTiming/Events.h"
#include "PresentTiming/Present.h"
#include "PresentTiming/Session.h"
#include "PresentTiming/Tracker.h"

#include <cstdint>
#include <expected>
#include <filesystem>
#include <memory>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

namespace present_timing {

using session::file_clock;

// What PresentMon's console does to the presents that it reads, before it
// times them: in the order they started, a present that NVIDIA's flip delay
// would put on the screen before the one before it goes on the screen with
// that one.
class Corrector {
public:
    Corrector() = default;
    Corrector(Corrector&&) = default;
    Corrector& operator=(Corrector&&) = default;
    Corrector(const Corrector&) = delete;
    Corrector& operator=(const Corrector&) = delete;

    std::vector<Present> correct(std::vector<Present> presents);

private:
    std::int64_t last_screen = 0;
    std::int64_t last_flip_delay = 0;
    std::int64_t last_start = 0;
};

// The presents that a session's thread has found, with the lock that the
// thread and the one that takes them share them under. In PresentTiming.cpp.
struct SharedPresents;

// Reads the presents of one process, until it's destroyed.
class PresentTiming {
public:
    // Starts reading the process's presents, in an ETW session named after
    // the program and this process, after stopping sessions of programs
    // whose processes are gone.
    static std::expected<PresentTiming, StartError> start(std::string_view name, std::uint32_t process);

    PresentTiming(PresentTiming&&) = default;
    PresentTiming& operator=(PresentTiming&&) = default;
    PresentTiming(const PresentTiming&) = delete;
    PresentTiming& operator=(const PresentTiming&) = delete;

    // The presents whose fate became known since the last call, in the order
    // they started.
    std::vector<Present> take();

private:
    PresentTiming(session::Session session, std::shared_ptr<SharedPresents> presents);

    Corrector corrector;
    std::shared_ptr<SharedPresents> presents;
    // Last, so that it's the first to go, as it is in Rust: the session
    // stops, and its thread ends, before the rest does.
    session::Session session_;
};

// The process's presents in a trace file that has these providers' events,
// like one that xperf or WPR saved, with the performance counter's
// frequency on the computer that saved it, to compare with what PresentMon
// says of the same file.
std::expected<std::pair<std::vector<Present>, std::int64_t>, std::string> read_file(const std::filesystem::path& path, std::uint32_t process);

// The performance counter's ticks a second.
std::int64_t frequency();

} // namespace present_timing
