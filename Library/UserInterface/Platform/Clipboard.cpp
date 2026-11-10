#include "UserInterface/Platform/Clipboard.h"

#include "Base/Text.h"

#include <algorithm>
#include <chrono>
#include <cstring>
#include <thread>

#pragma comment(lib, "user32.lib")

namespace user_interface {

namespace {

// Opens the clipboard, trying again for a few milliseconds while another
// program has it open, which it can only be for a moment.
bool open(HWND owner) {
    for (int attempt = 0; attempt < 5; attempt++) {
        if (attempt > 0) {
            std::this_thread::sleep_for(std::chrono::milliseconds(1));
        }
        if (OpenClipboard(owner)) {
            return true;
        }
    }
    return false;
}

std::optional<std::string> read_windows(HWND owner) {
    if (!IsClipboardFormatAvailable(CF_UNICODETEXT) || !open(owner)) {
        return std::nullopt;
    }
    std::optional<std::string> text;
    HANDLE handle = GetClipboardData(CF_UNICODETEXT);
    if (handle != nullptr) {
        const wchar_t* units = static_cast<const wchar_t*>(GlobalLock(handle));
        if (units != nullptr) {
            // The text ends at a null character, but no further than its
            // memory, in case another program left the null out.
            size_t most = GlobalSize(handle) / 2;
            std::wstring_view all(units, most);
            size_t length = std::min(all.find(L'\0'), most);
            text = base::narrow(all.substr(0, length));
            GlobalUnlock(handle);
        }
    }
    CloseClipboard();
    return text;
}

bool write_windows(HWND owner, std::string_view text) {
    if (!open(owner)) {
        return false;
    }
    bool written = false;
    if (EmptyClipboard()) {
        // With the null character that ends it.
        std::wstring units = base::widen(text);
        size_t count = units.size() + 1;
        HGLOBAL memory = GlobalAlloc(GMEM_MOVEABLE, count * 2);
        if (memory != nullptr) {
            wchar_t* destination = static_cast<wchar_t*>(GlobalLock(memory));
            if (destination != nullptr) {
                std::memcpy(destination, units.c_str(), count * sizeof(wchar_t));
                GlobalUnlock(memory);
                // Once the clipboard takes the memory, it's the clipboard's.
                written = SetClipboardData(CF_UNICODETEXT, memory) != nullptr;
            }
            if (!written) {
                GlobalFree(memory);
            }
        }
    }
    CloseClipboard();
    return written;
}

} // namespace

std::optional<std::string> Clipboard::text() const {
    return base::match(
        *this, [](const Own& own) -> std::optional<std::string> { return own.value; }, [](const Windows& windows) { return read_windows(windows.owner); },
        [](const Unavailable&) -> std::optional<std::string> { return std::nullopt; });
}

bool Clipboard::set_text(std::string_view text) {
    return base::match(
        *this,
        [&](Own& own) {
            own.value = std::string(text);
            return true;
        },
        [&](Windows& windows) { return write_windows(windows.owner, text); }, [](Unavailable&) { return false; });
}

} // namespace user_interface
