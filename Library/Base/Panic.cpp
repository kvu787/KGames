#include "Base/Panic.h"

#include "Base/Windows.h"

#include <crtdbg.h>
#include <cstdlib>

namespace base {

void panic(std::string_view message) {
    throw Panic(std::string(message));
}

void unreachable(std::string_view message) {
    throw Panic(std::string(message));
}

void fail_quietly() {
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
