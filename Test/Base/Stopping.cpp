// Library/Base's fail_quietly: a program that an uncaught exception stops
// says why on standard error, as a panic does in Rust, so that a program
// that RunInBackground runs leaves the reason in its log.

#include "Base/Files.h"
#include "Base/Panic.h"
#include "Base/Text.h"
#include "Base/Windows.h"
#include "Testing/Testing.h"

#include <string>

namespace {

// Set for the copy of this program that stops on purpose.
constexpr const char* StopHere = "KGAMES_BASE_TESTS_STOP_HERE";

// Throws where nothing can catch it, so the program stops.
void stop() noexcept {
    base::panic("the cascade didn't settle");
}

struct Output {
    DWORD exit_code;
    std::string errors;
};

// Runs this program again, with the variable set and only the test named,
// and returns its exit code and what it wrote to standard error.
Output run_again(const char* test) {
    SECURITY_ATTRIBUTES inherit{.nLength = sizeof(SECURITY_ATTRIBUTES), .bInheritHandle = TRUE};
    HANDLE read = nullptr;
    HANDLE write = nullptr;
    base::check_bool(CreatePipe(&read, &write, &inherit, 0));
    base::check_bool(SetHandleInformation(read, HANDLE_FLAG_INHERIT, 0));
    HANDLE nothing = CreateFileW(L"NUL", GENERIC_READ | GENERIC_WRITE, FILE_SHARE_READ | FILE_SHARE_WRITE, &inherit, OPEN_EXISTING, 0, nullptr);

    STARTUPINFOW startup{.cb = sizeof(STARTUPINFOW)};
    startup.dwFlags = STARTF_USESTDHANDLES;
    startup.hStdInput = nothing;
    startup.hStdOutput = nothing;
    startup.hStdError = write;
    std::wstring command = L"\"" + base::current_exe().native() + L"\" " + base::widen(test);
    SetEnvironmentVariableA(StopHere, "1");
    PROCESS_INFORMATION process{};
    BOOL started = CreateProcessW(nullptr, command.data(), nullptr, nullptr, TRUE, CREATE_NO_WINDOW, nullptr, nullptr, &startup, &process);
    SetEnvironmentVariableA(StopHere, nullptr);
    CloseHandle(write);
    CloseHandle(nothing);
    base::check_bool(started);

    Output output{};
    char buffer[4096];
    DWORD got = 0;
    while (ReadFile(read, buffer, sizeof(buffer), &got, nullptr) && got > 0) {
        output.errors.append(buffer, got);
    }
    CloseHandle(read);
    WaitForSingleObject(process.hProcess, INFINITE);
    GetExitCodeProcess(process.hProcess, &output.exit_code);
    CloseHandle(process.hProcess);
    CloseHandle(process.hThread);
    return output;
}

} // namespace

TEST(an_uncaught_exception_stops_the_program_and_says_why) {
    if (base::environment_variable(StopHere)) {
        // The copy: this never returns.
        stop();
    }
    Output output = run_again("an_uncaught_exception_stops_the_program_and_says_why");
    CHECK_NE(output.exit_code, 0);
    CHECK(output.errors.contains("The program stopped: the cascade didn't settle"), "{}", output.errors);
}
