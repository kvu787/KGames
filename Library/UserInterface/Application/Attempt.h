// The library's own: a call of what throws a base::WindowsError, for code
// that says what it was doing and stops, like the main loop when the GPU
// can't make a swap chain. It's what
//
//     .map_err(|error| format!("Couldn't open the window: {error}"))?
//
// after a call is in Rust:
//
//     std::expected<Window, std::string> opened = attempt("Couldn't open the window", [&] { return Window::create(title, monitor, mode); });
//     if (!opened) {
//         return std::unexpected(std::move(opened.error()));
//     }

#pragma once

#include "Base/Windows.h"

#include <expected>
#include <format>
#include <string>
#include <string_view>
#include <type_traits>

namespace user_interface {

// What `call` returns, or, if it threw a base::WindowsError, `what`, a
// colon, and what the error says. Any other exception goes on, as a panic
// does.
template <class Call> auto attempt(std::string_view what, Call&& call) -> std::expected<std::invoke_result_t<Call&>, std::string> {
    try {
        if constexpr (std::is_void_v<std::invoke_result_t<Call&>>) {
            call();
            return {};
        } else {
            return call();
        }
    } catch (const base::WindowsError& error) {
        return std::unexpected(std::format("{}: {}", what, error.what()));
    }
}

} // namespace user_interface
