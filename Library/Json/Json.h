// A JSON parser and writer, for settings files and test scripts, since KGames
// depends on no libraries besides the Windows SDK's and C++'s own. The parser
// follows RFC 8259, and says where in the text a mistake is.

#pragma once

#include "Base/Sum.h"

#include <array>
#include <concepts>
#include <cstddef>
#include <expected>
#include <initializer_list>
#include <optional>
#include <string>
#include <string_view>
#include <utility>
#include <variant>
#include <vector>

namespace json {

struct Value {
    struct Null {
        bool operator==(const Null&) const = default;
    };
    struct Bool {
        bool value;
        bool operator==(const Bool&) const = default;
    };
    struct Number {
        double value;
        bool operator==(const Number&) const = default;
    };
    struct String {
        std::string value;
        bool operator==(const String&) const = default;
    };
    struct Array {
        std::vector<Value> values;
        bool operator==(const Array&) const = default;
    };
    struct Object {
        // Members in the file's order.
        std::vector<std::pair<std::string, Value>> members;
        bool operator==(const Object&) const = default;
    };
    std::variant<Null, Bool, Number, String, Array, Object> value;

    template <base::AlternativeOf<Value> T> Value(T&& alternative) : value(std::forward<T>(alternative)) {}

    // Null.
    Value() = default;

    // What a value of another type is as JSON, so that code can write
    // json::object({{"Shift", 0.5}, {"Name", "Body"}}).
    template <std::same_as<bool> T> Value(T boolean) : value(Bool{boolean}) {}
    Value(double number) : value(Number{number}) {}
    // The float's shortest decimal, like 0.1, rather than its exact binary
    // value, like 0.10000000149011612, which reads worse and is no more
    // useful.
    Value(float number);
    template <std::integral T>
        requires(!std::same_as<T, bool>)
    Value(T number) : value(Number{static_cast<double>(number)}) {}
    Value(const char* text) : value(String{text}) {}
    Value(std::string_view text) : value(String{std::string(text)}) {}
    Value(std::string text) : value(String{std::move(text)}) {}
    // Null for nothing.
    template <class T>
        requires std::constructible_from<Value, T>
    Value(std::optional<T> optional) {
        if (optional) {
            *this = Value(std::move(*optional));
        }
    }
    template <class T>
        requires std::constructible_from<Value, T>
    Value(std::vector<T> items) : value(Array{}) {
        Array& array = std::get<Array>(value);
        array.values.reserve(items.size());
        for (T& item : items) {
            array.values.emplace_back(std::move(item));
        }
    }
    template <class T, size_t N>
        requires std::constructible_from<Value, T>
    Value(std::array<T, N> items) : value(Array{}) {
        Array& array = std::get<Array>(value);
        array.values.reserve(N);
        for (T& item : items) {
            array.values.emplace_back(std::move(item));
        }
    }

    // For messages like `must be a number, not "1280"`.
    std::string describe() const;

    // A member of an object, if this is one and it has the member, or null.
    const Value* get(std::string_view name) const;

    // As JSON text, with each object member on its own line, indented by
    // four spaces a level. Arrays of numbers, strings and the like stay on
    // one line, unless it would be longer than LineLength, when each item
    // gets its own. Numbers are written so that they parse back exactly. JSON
    // has no infinities or NaN, so those are written as null.
    std::string to_json() const;

    bool operator==(const Value&) const = default;
};

// How a failed check in a test shows a value: as its JSON.
inline std::string describe_value(const Value& value) {
    return value.to_json();
}

// The longest line that an array of numbers, strings and the like stays on.
inline constexpr size_t LineLength = 100;

// An object with these members, in this order, like
// object({{"Shift", 0.5}, {"Name", "Body"}}).
Value object(std::initializer_list<std::pair<std::string_view, Value>> members);

struct Error {
    size_t line;
    size_t column;
    std::string message;
};

std::expected<Value, Error> parse(std::string_view text);

} // namespace json
