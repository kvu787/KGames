#include "Base/Windows.h"

#include "Base/Text.h"

#include <format>

namespace base {

namespace {

std::string describe(HRESULT code) {
    wchar_t* buffer = nullptr;
    DWORD length = FormatMessageW(FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS, nullptr, static_cast<DWORD>(code), 0,
                                  reinterpret_cast<wchar_t*>(&buffer), 0, nullptr);
    std::string text;
    if (buffer != nullptr) {
        text = trim(narrow(std::wstring_view(buffer, length)));
        LocalFree(buffer);
    }
    if (text.empty()) {
        return std::format("0x{:08X}", static_cast<unsigned long>(code));
    }
    return std::format("{} (0x{:08X})", text, static_cast<unsigned long>(code));
}

} // namespace

WindowsError::WindowsError(HRESULT code) : code_(code), message_(describe(code)) {}

WindowsError last_error() {
    return WindowsError(HRESULT_FROM_WIN32(GetLastError()));
}

} // namespace base
