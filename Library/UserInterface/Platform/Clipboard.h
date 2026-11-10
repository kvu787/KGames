// Copy, cut and paste: Windows' clipboard for the user, and one of the
// interface's own for scripts and tests, which must never read or change
// what the user copied.

#pragma once

#include "Base/Sum.h"
#include "Base/Windows.h"

#include <optional>
#include <string>
#include <string_view>
#include <utility>
#include <variant>

namespace user_interface {

struct Clipboard {
    // Windows' clipboard, opened as the window that owns what it writes:
    // EmptyClipboard makes the window that has the clipboard open its owner,
    // and with no window, SetClipboardData fails.
    struct Windows {
        HWND owner;
    };
    // The interface's own, which holds the text.
    struct Own {
        std::string value;
    };
    // A clipboard that can't be read or written, like Windows' while another
    // program keeps it open, for tests.
    struct Unavailable {};
    std::variant<Windows, Own, Unavailable> value;

    template <base::AlternativeOf<Clipboard> T> Clipboard(T&& alternative) : value(std::forward<T>(alternative)) {}
    Clipboard(const Clipboard&) = delete;
    Clipboard& operator=(const Clipboard&) = delete;
    Clipboard(Clipboard&&) = default;
    Clipboard& operator=(Clipboard&&) = default;

    std::optional<std::string> text() const;

    // Puts the text on the clipboard, and says whether it's there, so that
    // a cut takes the text away only once it's on the clipboard.
    bool set_text(std::string_view text);
};

} // namespace user_interface
