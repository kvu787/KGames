// Panics: what code does when something that can't happen has happened, like
// a cascade of messages that never settles. A panic is an exception that
// nothing catches but tests, so it stops the program, which is what Rust's
// panic! does.
//
// What Rust checks and C++ doesn't, a panic stands in for where the code
// relies on the check: base::expect on an optional that must hold a value,
// instead of unwrap.

#pragma once

#include <optional>
#include <stdexcept>
#include <string>
#include <string_view>
#include <utility>

namespace base {

struct Panic : std::logic_error {
    using std::logic_error::logic_error;
};

[[noreturn]] void panic(std::string_view message);

// For a place that the code never reaches.
[[noreturn]] void unreachable(std::string_view message = "entered unreachable code");

// The optional's value, or a panic with the message.
template <class T> T& expect(std::optional<T>& optional, std::string_view message) {
    if (!optional) {
        panic(message);
    }
    return *optional;
}

template <class T> const T& expect(const std::optional<T>& optional, std::string_view message) {
    if (!optional) {
        panic(message);
    }
    return *optional;
}

template <class T> T expect(std::optional<T>&& optional, std::string_view message) {
    if (!optional) {
        panic(message);
    }
    return std::move(*optional);
}

// Stops a failure from opening a dialog, which would take focus and wait for
// someone to close it: the C runtime's assertions and errors go to stderr,
// and a crash ends the program at once. And has an exception that nothing
// catches, like a panic, write what it says to stderr before it stops the
// program, as a panic does in Rust, so that a program that stopped says why.
// Every program and test calls it first.
void fail_quietly();

} // namespace base
