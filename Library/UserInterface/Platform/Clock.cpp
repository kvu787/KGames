#include "UserInterface/Platform/Clock.h"

#include "Base/Windows.h"

#include <algorithm>
#include <intrin.h>

namespace user_interface::clock {

std::int64_t now() {
    LARGE_INTEGER value{};
    QueryPerformanceCounter(&value);
    return value.QuadPart;
}

std::int64_t frequency() {
    static const std::int64_t Frequency = [] {
        LARGE_INTEGER value{};
        QueryPerformanceFrequency(&value);
        return std::max<std::int64_t>(value.QuadPart, 1);
    }();
    return Frequency;
}

double milliseconds(std::int64_t ticks) {
    return static_cast<double>(ticks) * 1000.0 / static_cast<double>(frequency());
}

double seconds(std::int64_t ticks) {
    return static_cast<double>(ticks) / static_cast<double>(frequency());
}

std::int64_t ticks_from_microseconds(std::int64_t microseconds) {
    // The product in 128 bits, which it can take more than 64 of, divided
    // toward zero, and then the low 64 bits of that.
    std::int64_t high = 0;
    std::uint64_t low = static_cast<std::uint64_t>(_mul128(microseconds, frequency(), &high));
    bool negative = high < 0;
    std::uint64_t size_high = static_cast<std::uint64_t>(high);
    std::uint64_t size_low = low;
    if (negative) {
        size_low = ~low + 1;
        size_high = ~size_high + (size_low == 0 ? 1 : 0);
    }
    std::uint64_t remainder = 0;
    std::uint64_t quotient = _udiv128(size_high % 1'000'000, size_low, 1'000'000, &remainder);
    return static_cast<std::int64_t>(negative ? ~quotient + 1 : quotient);
}

} // namespace user_interface::clock
