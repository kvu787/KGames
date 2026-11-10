// WinUI's InfoBar: a message of one of four severities, in its background
// color, with its status icon.
//
// So far, this has only Severity, which the theme has a color for. The rest,
// which is the InfoBar itself, is still to come.

#pragma once

#include "Base/Panic.h"

#include <cstdint>
#include <string>
#include <string_view>

namespace user_interface {

enum class Severity : std::uint8_t {
    Informational,
    Success,
    Warning,
    Error,
};

constexpr std::string_view name(Severity severity) {
    switch (severity) {
    case Severity::Informational:
        return "Informational";
    case Severity::Success:
        return "Success";
    case Severity::Warning:
        return "Warning";
    case Severity::Error:
        return "Error";
    }
    base::unreachable();
}

// How a failed check in a test shows a severity.
inline std::string describe_value(Severity severity) {
    return std::string(name(severity));
}

} // namespace user_interface
