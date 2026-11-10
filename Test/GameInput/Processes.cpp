#include "Processes.h"

#include "Base/Files.h"
#include "Base/Text.h"
#include "Base/Windows.h"
#include "Testing/Testing.h"

#include <format>
#include <vector>

namespace processes {

namespace {

// How long a program that a test runs may take, before the test ends it and
// fails.
constexpr DWORD TimeoutInMilliseconds = 120'000;

// Closes the handle when it goes out of scope.
struct Handle {
    HANDLE handle;

    ~Handle() {
        if (handle != nullptr && handle != INVALID_HANDLE_VALUE) {
            CloseHandle(handle);
        }
    }
};

} // namespace

std::filesystem::path current_exe() {
    std::vector<wchar_t> name(32768);
    size_t length = GetModuleFileNameW(nullptr, name.data(), static_cast<DWORD>(name.size()));
    CHECK(length != 0 && length != name.size(), "Couldn't find this program's exe: {}", base::os_error(GetLastError()));
    return std::filesystem::path(std::wstring_view(name.data(), length));
}

bool has_variable(std::string_view name) {
    SetLastError(ERROR_SUCCESS);
    return GetEnvironmentVariableW(base::widen(name).c_str(), nullptr, 0) != 0 || GetLastError() != ERROR_ENVVAR_NOT_FOUND;
}

Output run(const std::filesystem::path& exe, std::initializer_list<std::string_view> arguments, std::string_view variable) {
    std::filesystem::path printed_path = exe;
    printed_path.replace_extension(".txt");
    PROCESS_INFORMATION process{};
    {
        // The program inherits these two handles, as what it reads and what
        // it prints to.
        SECURITY_ATTRIBUTES inherited{sizeof(SECURITY_ATTRIBUTES), nullptr, TRUE};
        Handle nothing{CreateFileW(L"NUL", GENERIC_READ, FILE_SHARE_READ | FILE_SHARE_WRITE, &inherited, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, nullptr)};
        CHECK(nothing.handle != INVALID_HANDLE_VALUE, "Couldn't open NUL: {}", base::os_error(GetLastError()));
        Handle printed{CreateFileW(printed_path.c_str(), GENERIC_WRITE, FILE_SHARE_READ, &inherited, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, nullptr)};
        CHECK(printed.handle != INVALID_HANDLE_VALUE, "Couldn't write {}: {}", base::display(printed_path), base::os_error(GetLastError()));

        std::wstring command = L"\"" + exe.native() + L"\"";
        for (std::string_view argument : arguments) {
            CHECK(!argument.contains('"'), "{} has a quote in it", argument);
            command += L" \"" + base::widen(argument) + L"\"";
        }
        STARTUPINFOW startup{};
        startup.cb = sizeof(STARTUPINFOW);
        startup.dwFlags = STARTF_USESTDHANDLES;
        startup.hStdInput = nothing.handle;
        startup.hStdOutput = printed.handle;
        startup.hStdError = printed.handle;

        // The program inherits this process's environment, so the variable
        // is in it while the program starts.
        std::wstring name = base::widen(variable);
        SetEnvironmentVariableW(name.c_str(), L"1");
        BOOL started = CreateProcessW(exe.c_str(), command.data(), nullptr, nullptr, TRUE, CREATE_NO_WINDOW, nullptr, nullptr, &startup, &process);
        DWORD error = GetLastError();
        SetEnvironmentVariableW(name.c_str(), nullptr);
        CHECK(started, "Couldn't run {}: {}", base::display(exe), base::os_error(error));
    }
    Handle ended{process.hProcess};
    Handle thread{process.hThread};
    if (WaitForSingleObject(process.hProcess, TimeoutInMilliseconds) != WAIT_OBJECT_0) {
        TerminateProcess(process.hProcess, 1);
        WaitForSingleObject(process.hProcess, INFINITE);
        testing::fail(std::format("{} didn't end in {} seconds", base::display(exe), TimeoutInMilliseconds / 1000));
    }
    DWORD exit_code = 1;
    CHECK(GetExitCodeProcess(process.hProcess, &exit_code), "Couldn't get what {} exited with: {}", base::display(exe), base::os_error(GetLastError()));
    std::expected<std::vector<std::uint8_t>, std::string> printed = base::read(printed_path);
    CHECK(printed.has_value(), "Couldn't read {}: {}", base::display(printed_path), printed.error());
    return Output{exit_code, std::string(reinterpret_cast<const char*>(printed->data()), printed->size())};
}

} // namespace processes
