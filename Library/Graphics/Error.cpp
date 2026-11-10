#include "Graphics/Error.h"

#include <format>
#include <string_view>
#include <utility>

namespace graphics {

namespace {

// What the system says a code means, from a base::WindowsError's what(),
// which is that and then the code, like
// "The parameter is incorrect. (0x80070057)", or the code alone when the
// system says nothing.
std::string system_message(std::string_view what) {
    // " (0x80070057)".
    constexpr size_t CodeLength = 13;
    if (what.size() >= CodeLength && what.ends_with(')') && what.substr(what.size() - CodeLength).starts_with(" (0x")) {
        return std::string(what.substr(0, what.size() - CodeLength));
    }
    return {};
}

} // namespace

Error::Error(HRESULT code) : base::WindowsError(code), message_(system_message(base::WindowsError::what())), text(base::WindowsError::what()) {}

Error::Error(HRESULT code, std::string message) : base::WindowsError(code), message_(std::move(message)) {
    if (message_.empty()) {
        message_ = system_message(base::WindowsError::what());
        text = base::WindowsError::what();
    } else {
        text = std::format("{} (0x{:08X})", message_, static_cast<unsigned long>(code));
    }
}

} // namespace graphics
