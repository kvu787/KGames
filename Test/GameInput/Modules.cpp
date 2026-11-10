#include "Modules.h"

#include "Base/Files.h"
#include "Base/Text.h"
#include "Base/Windows.h"
#include "Testing/Testing.h"

#include <TlHelp32.h>

#include <cwchar>
#include <iterator>

namespace modules {

namespace {

bool eq_ignore_ascii_case(std::string_view left, std::string_view right) {
    return base::to_ascii_lowercase(left) == base::to_ascii_lowercase(right);
}

} // namespace

std::vector<std::filesystem::path> loaded() {
    HANDLE snapshot = INVALID_HANDLE_VALUE;
    for (;;) {
        snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPMODULE, 0);
        if (snapshot != INVALID_HANDLE_VALUE) {
            break;
        }
        // Windows' ERROR_BAD_LENGTH, which a snapshot of the modules fails
        // with while a DLL loads or unloads. Then it's taken again.
        DWORD error = GetLastError();
        CHECK_EQ(error, ERROR_BAD_LENGTH, "Couldn't list the modules: {}", base::os_error(error));
    }
    MODULEENTRY32W entry{};
    entry.dwSize = sizeof(MODULEENTRY32W);
    std::vector<std::filesystem::path> paths;
    BOOL more = Module32FirstW(snapshot, &entry);
    while (more) {
        const wchar_t* path = entry.szExePath;
        size_t length = wcsnlen(path, std::size(entry.szExePath));
        paths.emplace_back(std::wstring_view(path, length));
        more = Module32NextW(snapshot, &entry);
    }
    CloseHandle(snapshot);
    return paths;
}

std::vector<std::filesystem::path> named(std::string_view name) {
    std::vector<std::filesystem::path> paths;
    for (std::filesystem::path& path : loaded()) {
        if (path.has_filename() && eq_ignore_ascii_case(base::display(path.filename()), name)) {
            paths.push_back(std::move(path));
        }
    }
    return paths;
}

bool same(const std::filesystem::path& left, const std::filesystem::path& right) {
    return eq_ignore_ascii_case(base::display(left), base::display(right));
}

std::string debug(const std::vector<std::filesystem::path>& paths) {
    std::string text = "[";
    for (size_t index = 0; index < paths.size(); index++) {
        text += index > 0 ? ", " : "";
        text += base::debug(base::display(paths[index]));
    }
    return text + "]";
}

} // namespace modules
