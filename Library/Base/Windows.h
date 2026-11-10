// Windows.h, and what a failed call of the Windows API is: a WindowsError,
// thrown by base::check, which is what the ? after a call is in Rust with
// windows_core::Result.
//
//     base::check(device->CreateCommandQueue(&description, IID_PPV_ARGS(&queue)));
//
// KGames.props defines NOMINMAX, WIN32_LEAN_AND_MEAN and UNICODE for every
// file. Windows.h still defines names that no code here can use as its own:
// near, far, small and interface, which AGENTS.md lists with the rest.

#pragma once

#include <Windows.h>

#include <exception>
#include <string>

namespace base {

// A failed call, by its HRESULT.
class WindowsError : public std::exception {
public:
    explicit WindowsError(HRESULT code);

    HRESULT code() const {
        return code_;
    }

    // Like "The parameter is incorrect. (0x80070057)".
    const char* what() const noexcept override {
        return message_.c_str();
    }

private:
    HRESULT code_;
    std::string message_;
};

// Throws if the HRESULT is a failure.
inline void check(HRESULT result) {
    if (FAILED(result)) {
        throw WindowsError(result);
    }
}

// Throws GetLastError's error if a call that returns a BOOL failed.
inline void check_bool(BOOL succeeded) {
    if (!succeeded) {
        throw WindowsError(HRESULT_FROM_WIN32(GetLastError()));
    }
}

// The last error, as a WindowsError to throw.
WindowsError last_error();

} // namespace base
