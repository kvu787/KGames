// Measures another process's frame times, like a game's while a build runs:
// reads the process's presents with Library/PresentTiming for a while, and
// prints the statistics of the times between its frames on the screen.
//
//     FrameTimes <process name or ID> [--seconds N]
//
// It has no window. See README.md.

#include "Base/Panic.h"
#include "Base/Text.h"
#include "FrameTimes/FrameTimes.h"
#include "PresentTiming/PresentTiming.h"

#include <fcntl.h>
#include <io.h>

#include <algorithm>
#include <chrono>
#include <cstdio>
#include <expected>
#include <format>
#include <optional>
#include <print>
#include <string>
#include <string_view>
#include <thread>
#include <vector>

using frame_times::find_process;
using frame_times::parse_arguments;
using frame_times::processes;
using frame_times::report;
using frame_times::summarize;
using frame_times::Usage;
using present_timing::PresentTiming;
using present_timing::StartError;

namespace {

constexpr std::string_view AccessDenied = "Couldn't start the ETW session that reads the presents: it needs the Performance Log Users group,\n"
                                          "or an elevated administrator. To join the group, once, from an elevated PowerShell, then sign out\n"
                                          "and back in:\n"
                                          "\n"
                                          "    Add-LocalGroupMember -Group 'Performance Log Users' -Member $env:USERNAME\n"
                                          "\n"
                                          "See Library/PresentTiming/README.md#privileges.";

// How often the presents whose fate became known are taken from the session.
constexpr std::chrono::milliseconds Poll{250};

std::expected<void, std::string> run() {
    std::vector<std::string> arguments = base::arguments();
    std::expected<frame_times::Options, std::string> options = parse_arguments(arguments);
    if (!options) {
        return std::unexpected(std::format("{}\nUsage: {}", options.error(), Usage));
    }
    std::expected<std::vector<frame_times::Process>, std::string> running = processes();
    if (!running) {
        return std::unexpected(std::move(running.error()));
    }
    std::expected<frame_times::Process, std::string> process = find_process(options->process, *running);
    if (!process) {
        return std::unexpected(std::move(process.error()));
    }
    std::expected<PresentTiming, StartError> started = PresentTiming::start("FrameTimes", process->id);
    if (!started) {
        return std::unexpected(base::match(
            started.error(), [](const StartError::AccessDenied&) { return std::string(AccessDenied); },
            [](const StartError::Failed& failed) { return std::format("Couldn't start the ETW session that reads the presents: {}", failed.message); }));
    }
    std::println("Measuring {}, process {}, for {} seconds.", process->name, process->id, base::display(options->seconds));
    // Rust hands on each line as it ends. C keeps what's written to a file
    // or a pipe until there's a lot of it, or the program ends, so the line
    // is handed on here, before the measuring.
    std::fflush(stdout);

    // What a Duration can't hold, Rust's Duration::from_secs_f64 panics on.
    if (options->seconds >= 18446744073709551616.0) {
        base::panic("cannot convert float seconds to Duration: value is either too big or NaN");
    }
    std::chrono::duration<double> duration(options->seconds);
    std::chrono::steady_clock::time_point start = std::chrono::steady_clock::now();
    std::vector<present_timing::Present> presents;
    {
        // The session stops at the end of this block, before the summary.
        PresentTiming timing = std::move(*started);
        while (true) {
            std::chrono::duration<double> left = duration - (std::chrono::steady_clock::now() - start);
            if (left <= std::chrono::duration<double>::zero()) {
                break;
            }
            std::this_thread::sleep_for(std::min<std::chrono::duration<double>>(left, Poll));
            std::vector<present_timing::Present> taken = timing.take();
            presents.insert(presents.end(), taken.begin(), taken.end());
        }
    }

    std::optional<frame_times::Summary> summary = summarize(presents, present_timing::frequency());
    if (!summary) {
        return std::unexpected(
            std::format("Fewer than two of its {} presents reached the screen, which leaves no frame time. Is it drawing, and is its window on a screen?", presents.size()));
    }
    std::println();
    std::print("{}", report(*summary));
    return {};
}

} // namespace

int main() {
    base::fail_quietly();
    // Text goes out as it's written, as Rust's does: without this, a line
    // that's written to a file or a pipe ends with a carriage return and a
    // line feed, not a line feed alone.
    _setmode(_fileno(stdout), _O_BINARY);
    _setmode(_fileno(stderr), _O_BINARY);
    std::expected<void, std::string> result = run();
    if (!result) {
        std::println(stderr, "FrameTimes: {}", result.error());
        return 1;
    }
    return 0;
}
