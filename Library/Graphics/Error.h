// A failed call of the Windows API with a message of the library's own, like
// "no GPU supports Direct3D 12", which is what windows_core::Error::new makes
// in Rust. It's a base::WindowsError, so whatever catches those catches it,
// and its what() reads the same way: the message, then the code.

#pragma once

#include "Base/Windows.h"

#include <string>

namespace graphics {

class Error : public base::WindowsError {
public:
    // With what the system says the code means.
    explicit Error(HRESULT code);
    // With the message, or what the system says if it's empty.
    Error(HRESULT code, std::string message);

    // The message, without the code.
    const std::string& message() const {
        return message_;
    }

    // Like "no GPU supports Direct3D 12 (0x80004005)".
    const char* what() const noexcept override {
        return text.c_str();
    }

private:
    std::string message_;
    std::string text;
};

} // namespace graphics
