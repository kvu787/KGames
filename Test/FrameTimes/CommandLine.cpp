// FrameTimes' command line, and finding the process that it names, by its
// exe's name or its ID.

#include "Base/Text.h"
#include "Base/Windows.h"
#include "FrameTimes/FrameTimes.h"
#include "Testing/Testing.h"

#include <algorithm>
#include <cstdint>
#include <expected>
#include <filesystem>
#include <initializer_list>
#include <string>
#include <string_view>
#include <vector>

using frame_times::DefaultSeconds;
using frame_times::find_process;
using frame_times::Options;
using frame_times::parse_arguments;
using frame_times::Process;
using frame_times::processes;

namespace {

std::expected<Options, std::string> parse(std::initializer_list<std::string_view> arguments) {
    std::vector<std::string> owned;
    for (std::string_view argument : arguments) {
        owned.emplace_back(argument);
    }
    return parse_arguments(owned);
}

Process process(std::uint32_t id, std::string_view name) {
    return Process{
        .id = id,
        .name = std::string(name),
    };
}

} // namespace

TEST(the_process_and_how_long_to_measure) {
    CHECK_EQ(parse({"AceCombat8"}).value(), (Options{
                                                .process = "AceCombat8",
                                                .seconds = DefaultSeconds,
                                            }));
    CHECK_EQ(DefaultSeconds, 10.0);
    CHECK_EQ(parse({"--seconds", "2.5", "1234"}).value(), (Options{
                                                              .process = "1234",
                                                              .seconds = 2.5,
                                                          }));
    CHECK_EQ(parse({"AceCombat8.exe", "--seconds", "30"}).value().seconds, 30.0);
}

TEST(wrong_command_lines_say_whats_wrong) {
    CHECK(parse({}).error().contains("Name the process"));
    CHECK(parse({"--seconds", "5"}).error().contains("Name the process"));
    CHECK(parse({"Game", "--seconds"}).error().contains("needs a number"));
    for (std::string_view seconds : {"0", "-1", "ten", "inf", "NaN"}) {
        CHECK(parse({"Game", "--seconds", seconds}).error().contains("above 0"), "{}", seconds);
    }
    CHECK(parse({"Game", "--frames", "5"}).error().contains("Unknown option: --frames"));
    CHECK(parse({"Game", "Other"}).error().contains("not both Game and Other"));
}

TEST(a_process_is_found_by_its_name_without_regard_to_case_or_exe) {
    std::vector<Process> processes{process(4, "System"), process(1200, "AceCombat8.exe"), process(1300, "explorer.exe")};
    for (std::string_view query : {"AceCombat8.exe", "acecombat8.EXE", "AceCombat8", "ACECOMBAT8"}) {
        CHECK_EQ(find_process(query, processes).value(), process(1200, "AceCombat8.exe"), "{}", query);
    }
    CHECK_EQ(find_process("system", processes).value(), process(4, "System"));
    CHECK(find_process("AceCombat", processes).error().contains("No process is named AceCombat"));
    CHECK(!find_process("AceCombat8.ex", processes).has_value());
}

TEST(a_process_is_found_by_its_id) {
    std::vector<Process> processes{process(1200, "AceCombat8.exe"), process(1300, "explorer.exe")};
    CHECK_EQ(find_process("1300", processes).value(), process(1300, "explorer.exe"));
    CHECK(find_process("1400", processes).error().contains("No process has the ID 1400"));
}

TEST(processes_of_the_same_name_need_an_id) {
    std::vector<Process> processes{process(1200, "Game.exe"), process(1300, "explorer.exe"), process(1400, "game.exe")};
    std::string error = find_process("Game", processes).error();
    CHECK(error.contains("2 processes are named Game.exe: 1200, 1400"), "{}", error);
    CHECK_EQ(find_process("1400", processes).value(), process(1400, "game.exe"));
}

TEST(the_running_processes_include_this_one) {
    std::vector<Process> running = processes().value();
    auto found = std::ranges::find(running, GetCurrentProcessId(), &Process::id);
    CHECK(found != running.end());
    const Process& this_ = *found;
    std::wstring path(MAX_PATH, L'\0');
    path.resize(GetModuleFileNameW(nullptr, path.data(), static_cast<DWORD>(path.size())));
    std::string exe = base::narrow(std::filesystem::path(path).filename().native());
    CHECK(base::to_ascii_lowercase(this_.name) == base::to_ascii_lowercase(exe), "{}", this_.name);
    CHECK_EQ(find_process(std::to_string(GetCurrentProcessId()), running).value(), this_);
}
