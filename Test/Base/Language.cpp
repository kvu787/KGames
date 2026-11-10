// What the repository's code relies on of C++ and its standard library,
// where a newer one could change what the same text means without the
// compiler saying a word. A test here fails on the day that one does, and
// says what to do then.

#include "Testing/Testing.h"

#include <cstddef>
#include <span>
#include <string_view>
#include <vector>

namespace {

// As the functions that take a list of texts take it, like files::only,
// Settings::choice and the widgets that take their items in place.
std::vector<std::string_view> listed(std::span<const std::string_view> texts) {
    return std::vector<std::string_view>(texts.begin(), texts.end());
}

size_t count(std::span<const std::string_view> texts) {
    return texts.size();
}

} // namespace

// A function that takes a list of texts takes a std::span<const
// std::string_view>, and code lists the texts in place in double braces,
// like Segmented<Message>(shader, {{"SimplePaint", "Flat"}}): the inner
// braces make an array of std::string_views, and the span is of the array.
//
// What would change it: a std::span constructor from a
// std::initializer_list, which C++26 has (P2447, "std::span over an
// initializer list"). The outer braces would then be that list, of one
// std::string_view, and the inner braces would make it from two pointers:
// "A" as where its text starts and "B" as where it ends, which is whatever
// lies between the two literals in memory. It compiles the same, without a
// warning. Only a list of exactly two texts changes: one text makes the one
// std::string_view, and nothing makes a std::string_view of three.
//
// What to do then: take a std::initializer_list<std::string_view> in the
// functions that take lists of texts, and write their lists in single
// braces, like {"SimplePaint", "Flat"}.
TEST(texts_listed_in_double_braces_are_that_many_texts) {
    CHECK_EQ(count({{"A", "B"}}), 2);
    CHECK_EQ(count({{"A"}}), 1);
    CHECK_EQ(count({{"A", "B", "C"}}), 3);
    CHECK_EQ(listed({{"A", "B"}}), (std::vector<std::string_view>{"A", "B"}));
    CHECK_EQ(listed({{"A"}}), (std::vector<std::string_view>{"A"}));
    CHECK_EQ(listed({{"A", "B", "C"}}), (std::vector<std::string_view>{"A", "B", "C"}));
}
