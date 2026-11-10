// What std::env::current_exe, std::env::var_os and std::process::Command are
// in Rust, for the tests that run a copy of this program: where its exe is,
// and running one and reading what it printed.

#pragma once

#include <cstdint>
#include <filesystem>
#include <initializer_list>
#include <string>
#include <string_view>

namespace processes {

// The full path of this program's exe.
std::filesystem::path current_exe();

// Whether the environment has the variable.
bool has_variable(std::string_view name);

// What a program printed, to its output and its errors, and what it exited
// with.
struct Output {
    std::uint32_t exit_code;
    std::string printed;
};

// Runs the exe with the arguments, which mustn't have quotes in them, and
// with the environment variable set to 1 for it, and waits for it to end. It
// has no window, and takes no input. What it prints goes to a file beside it,
// which is read once it has ended, so nothing waits on what the program
// starts.
Output run(const std::filesystem::path& exe, std::initializer_list<std::string_view> arguments, std::string_view variable);

} // namespace processes
