// Windows' performance counter (QPC), the one clock that input, frames and
// the display's flips are all timed on, so that the time between any two of
// them is a subtraction. ETW's events carry QPC values, and GameInput's
// readings carry microseconds that InputDevices.cpp turns into QPC values.

#pragma once

#include <cstdint>

namespace user_interface::clock {

// The counter now.
std::int64_t now();

// Ticks a second, which never changes while Windows runs.
std::int64_t frequency();

// A span of ticks in milliseconds.
double milliseconds(std::int64_t ticks);

// A span of ticks in seconds.
double seconds(std::int64_t ticks);

// Ticks in a span of microseconds.
std::int64_t ticks_from_microseconds(std::int64_t microseconds);

} // namespace user_interface::clock
