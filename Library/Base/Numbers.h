// Numbers: what Rust's casts and integer methods do that C++'s don't. A
// float that's out of an integer's range is undefined behavior to cast in
// C++, and saturates in Rust, and an unsigned subtraction below zero wraps
// in C++, where the code here means it to stop at zero. A signed sum or
// difference past what its type holds is undefined behavior in C++ too, and
// goes around in Rust where the code asks it to.

#pragma once

#include <cmath>
#include <concepts>
#include <limits>
#include <optional>
#include <type_traits>

namespace base {

// A float as a whole number, as Rust's `as` gives it: toward zero, the
// nearest that the type holds when it's out of range, and 0 for NaN.
template <std::integral To, std::floating_point From> constexpr To saturating_cast(From value) {
    if (value != value) {
        return 0;
    }
    // The limits as floats, which round to a power of two, so the upper one
    // is past what the type holds.
    constexpr From Lowest = static_cast<From>(std::numeric_limits<To>::lowest());
    constexpr From Highest = static_cast<From>(std::numeric_limits<To>::max());
    if (value <= Lowest) {
        return std::numeric_limits<To>::lowest();
    }
    if (value >= Highest) {
        return std::numeric_limits<To>::max();
    }
    return static_cast<To>(value);
}

// a - b, or 0 if b is more: usize::saturating_sub.
template <std::unsigned_integral T> constexpr T saturating_sub(T a, T b) {
    return a > b ? a - b : T{0};
}

// a - b, or nothing if b is more: usize::checked_sub.
template <std::unsigned_integral T> constexpr std::optional<T> checked_sub(T a, T b) {
    return a >= b ? std::optional<T>(a - b) : std::nullopt;
}

// The distance between two numbers: usize::abs_diff.
template <std::integral T> constexpr auto abs_diff(T a, T b) {
    using Unsigned = std::make_unsigned_t<T>;
    return a > b ? static_cast<Unsigned>(static_cast<Unsigned>(a) - static_cast<Unsigned>(b)) : static_cast<Unsigned>(static_cast<Unsigned>(b) - static_cast<Unsigned>(a));
}

// a + b and a - b, around from one end of what the type holds to the other
// when the sum or the difference is past it, as unsigned numbers go:
// i64::wrapping_add and wrapping_sub. They're for running totals and the
// differences of two of them, like a mouse's position: + and - of signed
// numbers are undefined there in C++.
template <std::signed_integral T> constexpr T wrapping_add(T a, T b) {
    using Unsigned = std::make_unsigned_t<T>;
    return static_cast<T>(static_cast<Unsigned>(static_cast<Unsigned>(a) + static_cast<Unsigned>(b)));
}

template <std::signed_integral T> constexpr T wrapping_sub(T a, T b) {
    using Unsigned = std::make_unsigned_t<T>;
    return static_cast<T>(static_cast<Unsigned>(static_cast<Unsigned>(a) - static_cast<Unsigned>(b)));
}

// The remainder that's never negative: rem_euclid.
template <std::floating_point T> T rem_euclid(T value, T divisor) {
    T remainder = std::fmod(value, divisor);
    return remainder < 0 ? remainder + std::abs(divisor) : remainder;
}

template <std::signed_integral T> constexpr T rem_euclid(T value, T divisor) {
    T remainder = value % divisor;
    return remainder < 0 ? (divisor < 0 ? remainder - divisor : remainder + divisor) : remainder;
}

} // namespace base
