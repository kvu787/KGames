// Library/Base's fail_quietly: a program that an uncaught exception stops
// says why on standard error, as a panic does in Rust, whichever thread it's
// on, so that a program that RunInBackground runs leaves the reason in its
// log. And in Debug, an error that the debug C runtime reports stops the
// program, so that a test that corrupts the heap fails.

#include "Base/Files.h"
#include "Base/Panic.h"
#include "Base/Text.h"
#include "Base/Windows.h"
#include "Testing/Testing.h"

#include <string>
#include <thread>

namespace {

// Set, to what it should do, for the copy of this program that stops on
// purpose.
constexpr const char* StopHere = "KGAMES_BASE_TESTS_STOP_HERE";

// Throws where nothing can catch it, so the program stops.
void stop() noexcept {
    base::panic("the cascade didn't settle");
}

struct Output {
    DWORD exit_code;
    std::string errors;
};

// Runs this program again, with the variable set to `how` and only the test
// named, and returns its exit code and what it wrote to standard error.
Output run_again(const char* test, const char* how) {
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
    SetEnvironmentVariableA(StopHere, how);
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
    Output output = run_again("an_uncaught_exception_stops_the_program_and_says_why", "here");
    CHECK_NE(output.exit_code, 0);
    CHECK(output.errors.contains("The program stopped: the cascade didn't settle"), "{}", output.errors);
}

// A handler set with std::set_terminate is the calling thread's alone, with
// MSVC. So a panic on another thread, like the one that flushes the present
// timing's session, stopped the program with nothing said.
TEST(an_uncaught_exception_on_another_thread_says_why_too) {
    if (base::environment_variable(StopHere)) {
        // The copy: the thread's exception has nothing to catch it.
        std::thread other([] { base::panic("the other thread's reason"); });
        other.join();
        return;
    }
    Output output = run_again("an_uncaught_exception_on_another_thread_says_why_too", "thread");
    CHECK_NE(output.exit_code, 0);
    CHECK(output.errors.contains("The program stopped: the other thread's reason"), "{}", output.errors);
}

#ifdef _DEBUG
// The debug C runtime finds a write past the end of a block of the heap when
// the block is freed, and reports it. With its reports going to a file, the
// program went on, and a test that corrupted the heap passed.
TEST(a_write_past_a_block_of_the_heap_stops_a_debug_program) {
    if (base::environment_variable(StopHere)) {
        // The copy: one byte past the 16 that were asked for.
        char* block = new char[16];
        char* volatile past = block + 16;
        *past = 1;
        delete[] block;
        return;
    }
    Output output = run_again("a_write_past_a_block_of_the_heap_stops_a_debug_program", "heap");
    CHECK_NE(output.exit_code, 0);
    CHECK(output.errors.contains("The debug C runtime reported"), "{}", output.errors);
}
#endif
