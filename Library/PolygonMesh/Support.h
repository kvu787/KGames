// What PolygonMesh's sources use that Rust's standard library has, and that
// neither C++'s nor Library/Base has: the total order of floats, the last of
// the greatest items, where an item is in a list, hashes of pairs, arrays and
// vectors, and lists as {:?} writes them, for describe_value. It's for the
// library's .cpp files: none of its headers includes it.

#pragma once

#include "Base/Text.h"
#include "Math/Math.h"

#include <algorithm>
#include <array>
#include <bit>
#include <compare>
#include <cstddef>
#include <cstdint>
#include <functional>
#include <optional>
#include <ranges>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

namespace polygon_mesh {

// The order of two floats as f32::total_cmp gives it, which orders every
// float: -0 is before +0, and a NaN is past the infinity of its sign.
inline std::strong_ordering total_cmp(float a, float b) {
    std::int32_t left = std::bit_cast<std::int32_t>(a);
    std::int32_t right = std::bit_cast<std::int32_t>(b);
    left ^= static_cast<std::int32_t>(static_cast<std::uint32_t>(left >> 31) >> 1);
    right ^= static_cast<std::int32_t>(static_cast<std::uint32_t>(right >> 31) >> 1);
    return left <=> right;
}

inline std::strong_ordering total_cmp(double a, double b) {
    std::int64_t left = std::bit_cast<std::int64_t>(a);
    std::int64_t right = std::bit_cast<std::int64_t>(b);
    left ^= static_cast<std::int64_t>(static_cast<std::uint64_t>(left >> 63) >> 1);
    right ^= static_cast<std::int64_t>(static_cast<std::uint64_t>(right >> 63) >> 1);
    return left <=> right;
}

// The greatest of the items by `compare`, and the last of those as great, as
// Iterator::max_by gives, where std::ranges::max_element gives the first.
// There must be an item.
template <std::ranges::input_range Range, class Compare> auto max_by(Range&& items, Compare&& compare) {
    auto iterator = std::ranges::begin(items);
    auto end = std::ranges::end(items);
    auto greatest = *iterator;
    for (++iterator; iterator != end; ++iterator) {
        auto item = *iterator;
        if (compare(greatest, item) != std::strong_ordering::greater) {
            greatest = item;
        }
    }
    return greatest;
}

// Where the item first is in the list, or nothing if it isn't there:
// iter().position(). With a projection, where the first is whose projection
// is the item.
template <std::ranges::random_access_range Range, class T, class Projection = std::identity>
std::optional<size_t> position_of(const Range& items, const T& item, Projection projection = {}) {
    auto found = std::ranges::find(items, item, projection);
    if (found == std::ranges::end(items)) {
        return std::nullopt;
    }
    return static_cast<size_t>(found - std::ranges::begin(items));
}

// A hash of a pair, an array or a vector of numbers, which the standard
// library has none for, for the keys of a std::unordered_map or set.
struct Hash {
    template <class A, class B> size_t operator()(const std::pair<A, B>& pair) const noexcept {
        return combine(combine(0, pair.first), pair.second);
    }

    template <class T, size_t N> size_t operator()(const std::array<T, N>& items) const noexcept {
        size_t hash = 0;
        for (const T& item : items) {
            hash = combine(hash, item);
        }
        return hash;
    }

    template <class T> size_t operator()(const std::vector<T>& items) const noexcept {
        size_t hash = items.size();
        for (const T& item : items) {
            hash = combine(hash, item);
        }
        return hash;
    }

private:
    template <class T> static size_t combine(size_t hash, const T& item) noexcept {
        return hash ^ (std::hash<T>{}(item) + 0x9e3779b97f4a7c15ull + (hash << 6) + (hash >> 2));
    }
};

// The items between brackets, with ", " between them, each as `describe`
// writes it: a list as {:?} writes one, or a set, between "{}".
template <std::ranges::input_range Range, class Describe> std::string describe_list(Range&& items, Describe&& describe, std::string_view brackets = "[]") {
    std::string text(1, brackets[0]);
    bool first = true;
    for (const auto& item : items) {
        text += first ? "" : ", ";
        text += describe(item);
        first = false;
    }
    text += brackets[1];
    return text;
}

// Whole numbers, like [0, 3, 2, 1].
template <std::ranges::input_range Range> std::string describe_numbers(Range&& numbers, std::string_view brackets = "[]") {
    return describe_list(numbers, [](const auto& number) { return std::to_string(number); }, brackets);
}

// A position or a direction, like [0.0, 1.0, -0.5].
inline std::string describe_vector(math::Vector3 vector) {
    return describe_list(vector, [](float component) { return base::debug(component); });
}

} // namespace polygon_mesh
