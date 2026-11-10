// The DLLs that this process has loaded, from a snapshot of its modules.

#pragma once

#include <filesystem>
#include <string>
#include <string_view>
#include <vector>

namespace modules {

// The full path of every DLL that the process has loaded, and of its exe.
std::vector<std::filesystem::path> loaded();

// The loaded DLLs with this file name, regardless of case.
std::vector<std::filesystem::path> named(std::string_view name);

// Whether two paths are the same, regardless of case, as Windows' are.
bool same(const std::filesystem::path& left, const std::filesystem::path& right);

// The paths as a failed check shows them, like ["C:\\Windows\\System32\\hid.dll"].
std::string debug(const std::vector<std::filesystem::path>& paths);

} // namespace modules
