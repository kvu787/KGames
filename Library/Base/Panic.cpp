#include "Base/Panic.h"

#include "Base/Windows.h"

#include <crtdbg.h>
#include <cstdio>
#include <cstdlib>
#include <exception>

namespace base {

void panic(std::string_view message) {
    throw Panic(std::string(message));
}

void unreachable(std::string_view message) {
    throw Panic(std::string(message));
}

namespace {

// What std::terminate calls: for an exception that nothing caught.
[[noreturn]] void stop_with_what_was_thrown() {
    const char* what = "something that isn't a std::exception was thrown, and nothing caught it";
    std::string kept;
    if (std::exception_ptr thrown = std::current_exception()) {
        try {
            std::rethrow_exception(thrown);
        } catch (const std::exception& exception) {
            kept = exception.what();
            what = kept.c_str();
        } catch (...) {}
    } else {
        what = "std::terminate was called";
    }
    std::fputs("The program stopped: ", stderr);
    std::fputs(what, stderr);
    std::fputs("\n", stderr);
    std::fflush(stderr);
    std::abort();
}

} // namespace

void fail_quietly() {
    std::set_terminate(stop_with_what_was_thrown);
    SetErrorMode(SEM_FAILCRITICALERRORS | SEM_NOGPFAULTERRORBOX | SEM_NOOPENFILEERRORBOX);
    _set_abort_behavior(0, _WRITE_ABORT_MSG | _CALL_REPORTFAULT);
#ifdef _DEBUG
    // Only the debug C runtime reports these.
    for (int kind : {_CRT_WARN, _CRT_ERROR, _CRT_ASSERT}) {
        _CrtSetReportMode(kind, _CRTDBG_MODE_FILE | _CRTDBG_MODE_DEBUG);
        _CrtSetReportFile(kind, _CRTDBG_FILE_STDERR);
    }
#endif
}

} // namespace base
