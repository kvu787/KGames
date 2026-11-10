#include "Base/Panic.h"

#include "Base/Windows.h"

#include <crtdbg.h>
#include <csignal>
#include <cstdio>
#include <cstdlib>
#include <exception>
#include <string>

namespace base {

void panic(std::string_view message) {
    throw Panic(std::string(message));
}

void unreachable(std::string_view message) {
    throw Panic(std::string(message));
}

namespace {

// What abort raises, on whichever thread called it: std::terminate calls
// abort for an exception that nothing caught, on every thread, where a
// handler set with std::set_terminate would be the calling thread's alone.
// The exception, if there's one, is still the current one here.
void on_abort(int) {
    std::string what = "abort was called";
    if (std::exception_ptr thrown = std::current_exception()) {
        try {
            std::rethrow_exception(thrown);
        } catch (const std::exception& exception) {
            what = exception.what();
        } catch (...) {
            what = "something that isn't a std::exception was thrown, and nothing caught it";
        }
    }
    std::fputs("The program stopped: ", stderr);
    std::fputs(what.c_str(), stderr);
    std::fputs("\n", stderr);
    std::fflush(stderr);
    // What abort exits with.
    _exit(3);
}

#ifdef _DEBUG
// What the debug C runtime calls with each of its reports. An error or a
// failed assertion, like a write past the end of a block of the heap, which
// it finds when the block is freed, stops the program: written to a file,
// as fail_quietly has them, it would go on, and a test would pass.
int on_report(int kind, char* message, int*) {
    if (kind == _CRT_WARN) {
        return FALSE;
    }
    std::fputs("The debug C runtime reported: ", stderr);
    std::fputs(message != nullptr ? message : "an error", stderr);
    std::fputs("\n", stderr);
    std::fflush(stderr);
    std::abort();
}
#endif

} // namespace

void fail_quietly() {
    std::signal(SIGABRT, on_abort);
    SetErrorMode(SEM_FAILCRITICALERRORS | SEM_NOGPFAULTERRORBOX | SEM_NOOPENFILEERRORBOX);
    _set_abort_behavior(0, _WRITE_ABORT_MSG | _CALL_REPORTFAULT);
#ifdef _DEBUG
    // Only the debug C runtime reports these.
    for (int kind : {_CRT_WARN, _CRT_ERROR, _CRT_ASSERT}) {
        _CrtSetReportMode(kind, _CRTDBG_MODE_FILE | _CRTDBG_MODE_DEBUG);
        _CrtSetReportFile(kind, _CRTDBG_FILE_STDERR);
    }
    _CrtSetReportHook2(_CRT_RPTHOOK_INSTALL, on_report);
#endif
}

} // namespace base
