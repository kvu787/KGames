// Why a material, mesh or transform was rejected. SimplePaint never clamps,
// wraps or repairs its inputs instead.

#pragma once

#include <string>

namespace simple_paint {

struct Error {
    std::string message;

    bool operator==(const Error&) const = default;
};

// How a failed check in a test shows an error.
inline std::string describe_value(const Error& error) {
    return "Error(\"" + error.message + "\")";
}

} // namespace simple_paint
