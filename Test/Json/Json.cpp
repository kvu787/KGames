// Library/Json: parsing, where mistakes are, and writing that parses back.

#include "Json/Json.h"
#include "Testing/Testing.h"

#include <algorithm>
#include <limits>
#include <string>
#include <tuple>
#include <vector>

using json::object;
using json::parse;
using json::Value;

namespace {

std::string repeat(std::string_view text, size_t count) {
    std::string repeated;
    for (size_t index = 0; index < count; index++) {
        repeated += text;
    }
    return repeated;
}

} // namespace

TEST(parses_every_kind_of_value) {
    Value value = parse(R"( { "a": [1, -2.5e3, true, false, null], "b": "x\n\u00e9\ud83d\ude00" } )").value();
    CHECK_EQ(value, object({
                        {"a", Value::Array{{1.0, -2500.0, true, false, Value::Null{}}}},
                        {"b", "x\n\u00e9\U0001F600"},
                    }));
    CHECK(value.get("b") != nullptr);
    CHECK_EQ(*value.get("b"), Value("x\n\u00e9\U0001F600"));
    CHECK(value.get("c") == nullptr);
}

TEST(says_where_a_mistake_is) {
    struct Case {
        std::string_view text;
        size_t line;
        size_t column;
        std::string_view message;
    };
    for (const Case& expected : {
             Case{"{\n  \"a\": 1,\n}", 3, 1, "JSON doesn't allow a comma before '}'"},
             Case{"[01]", 1, 2, "numbers can't start with 0, like 01"},
             Case{"\"\\ud800\"", 1, 4, "unpaired UTF-16 surrogate"},
             Case{"\"\\udc00\"", 1, 4, "unpaired UTF-16 surrogate"},
             Case{"{} x", 1, 4, "expected the end of the file"},
             Case{"[1 2]", 1, 4, "expected ',' or ']'"},
             Case{"[1e400]", 1, 2, "the number is too large"},
             Case{"-1e400", 1, 1, "the number is too large"},
         }) {
        json::Error error = parse(expected.text).error();
        CHECK_EQ(std::tuple(error.line, error.column, std::string_view(error.message)), std::tuple(expected.line, expected.column, expected.message), "for {}",
                 testing::describe(expected.text));
    }
}

TEST(fails_on_deep_nesting_instead_of_overflowing_the_stack) {
    auto nested = [](size_t depth) {
        return repeat("[", depth) + repeat("]", depth);
    };
    CHECK(parse(nested(128)).has_value());
    json::Error error = parse(nested(129)).error();
    CHECK_EQ(std::tuple(error.line, error.column, std::string_view(error.message)), std::tuple(1, 129, std::string_view("arrays and objects are nested too deeply")));
    // Deep enough to overflow the stack without the limit.
    CHECK(!parse(repeat("[{\"a\":", 100'000)).has_value());
}

TEST(writes_what_parses_back_the_same) {
    Value value = object({
        {"Numbers", std::array{0.1, 1.0 / 3.0, -0.0, 1e-300, 123456789.0}},
        {"Text", "quote \" backslash \\ tab \t bell \u0007 e \u00e9"},
        {"Nested", object({{"Empty", Value::Array{}}, {"Object", Value::Object{}}})},
        {"Rows", Value::Array{{object({{"Flag", true}}), Value::Null{}}}},
    });
    std::string text = value.to_json();
    CHECK_EQ(parse(text).value(), value, "{}", text);
}

TEST(writes_one_member_a_line) {
    Value value = object({{"Color", std::array{0.5, 0.25}}, {"Name", "Body"}});
    CHECK_EQ(value.to_json(), "{\n    \"Color\": [0.5, 0.25],\n    \"Name\": \"Body\"\n}");
}

// A long array of numbers or strings gets a line for each item, so that a
// change to one changes one line.
TEST(writes_a_long_flat_array_an_item_a_line) {
    std::vector<std::string> rows;
    for (int row = 0; row < 4; row++) {
        rows.push_back(repeat(std::to_string(row), 30));
    }
    Value value = object({{"Rows", rows}});
    std::string text = value.to_json();
    std::vector<std::string_view> lines = base::lines(text);
    CHECK_EQ(lines.size(), 8, "{}", text);
    CHECK_EQ(parse(text).value(), value);
    CHECK(std::ranges::all_of(lines, [](std::string_view line) { return line.size() <= json::LineLength; }));
}

TEST(writes_infinities_and_nan_as_null) {
    Value value = Value::Array{{std::numeric_limits<double>::infinity(), std::numeric_limits<double>::quiet_NaN()}};
    CHECK_EQ(value.to_json(), "[null, null]");
}

// Numbers are written as Rust's {:?} writes them, so that the files that
// either language's programs save read the same.
TEST(writes_numbers_as_rust_does) {
    CHECK_EQ(Value(1.0).to_json(), "1.0");
    CHECK_EQ(Value(-0.0).to_json(), "-0.0");
    CHECK_EQ(Value(0.1f).to_json(), "0.1");
    CHECK_EQ(Value(1e-7).to_json(), "1e-7");
    CHECK_EQ(Value(0.0001).to_json(), "0.0001");
    CHECK_EQ(Value(1.5e16).to_json(), "1.5e16");
    CHECK_EQ(Value(1e300).to_json(), "1e300");
    CHECK_EQ(Value(123456789012345.0).to_json(), "123456789012345.0");
    CHECK_EQ(Value(size_t{7}).to_json(), "7.0");
    CHECK_EQ(Value(1e21).describe(), "1000000000000000000000");
    CHECK_EQ(Value(0.5).describe(), "0.5");
}
