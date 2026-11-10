// The test harness: what #[test], assert! and assert_eq! are in Rust. A test
// is a function that TEST names, in any .cpp of a project in Test, and the
// project's program, whose main is here, runs every one:
//
//     #include "Testing/Testing.h"
//
//     TEST(transforms_compose_from_left_to_right) {
//         CHECK_EQ(Matrix::Identity * combined, combined);
//         CHECK(near(actual, expected), "{} isn't {}", actual, expected);
//     }
//
// A check that fails ends its test, which fails, with where the check is,
// what it compared, and its message, which is std::format's. So does an
// exception that the test doesn't catch, like a panic. The program then runs
// the rest, and exits with 1 if any failed.
//
//     target\Debug\MathTests.exe                 Runs every test
//     target\Debug\MathTests.exe compose         Runs the tests with "compose" in their names
//     target\Debug\MathTests.exe --list          Lists them
//
// A test's full name is its file's name, without .cpp, then its own, like
// Matrices::transforms_compose_from_left_to_right.

#pragma once

#include "Base/Text.h"

#include <concepts>
#include <format>
#include <optional>
#include <ranges>
#include <source_location>
#include <string>
#include <string_view>
#include <tuple>
#include <type_traits>
#include <utility>
#include <variant>

namespace testing {

// What a failed check throws, to end its test.
struct Failure {
    std::string message;
    std::source_location where;
};

[[noreturn]] void fail(std::string message, std::source_location where = std::source_location::current());

// Registers a test, when the program starts.
struct Registration {
    Registration(const char* file, const char* name, void (*body)());
};

// A value as a failed check shows it: what {:?} is in Rust, as far as C++
// can tell without the type's help. A type says more with a describe of its
// own, beside it in its namespace, or a std::formatter.
template <class T> std::string describe(const T& value);

namespace detail {

template <class T>
concept HasDescribe = requires(const T& value) {
    { describe_value(value) } -> std::convertible_to<std::string>;
};

template <class T>
concept StringLike = std::convertible_to<const T&, std::string_view>;

template <class T>
concept TupleLike = requires { std::tuple_size<T>::value; } && !std::ranges::range<T>;

template <class T> std::string describe_one(const T& value) {
    if constexpr (HasDescribe<T>) {
        return describe_value(value);
    } else if constexpr (std::same_as<T, bool>) {
        return value ? "true" : "false";
    } else if constexpr (std::same_as<T, std::nullptr_t> || std::same_as<T, std::nullopt_t>) {
        return "None";
    } else if constexpr (std::same_as<T, char>) {
        return std::format("'{}'", value);
    } else if constexpr (std::same_as<T, char32_t>) {
        std::string text;
        base::push_char(text, value);
        return "'" + text + "'";
    } else if constexpr (StringLike<T>) {
        return base::debug(std::string_view(value));
    } else if constexpr (std::same_as<T, float> || std::same_as<T, double>) {
        return base::debug(value);
    } else if constexpr (std::integral<T>) {
        return std::format("{}", value);
    } else if constexpr (std::is_enum_v<T>) {
        return std::format("{}", static_cast<std::underlying_type_t<T>>(value));
    } else if constexpr (std::is_pointer_v<T>) {
        return value == nullptr ? std::string("null") : std::format("{}", static_cast<const void*>(value));
    } else if constexpr (std::ranges::range<const T&>) {
        std::string text = "[";
        bool first = true;
        for (const auto& item : value) {
            text += first ? "" : ", ";
            text += testing::describe(item);
            first = false;
        }
        return text + "]";
    } else if constexpr (TupleLike<T>) {
        std::string text = "(";
        std::apply(
            [&](const auto&... items) {
                bool first = true;
                ((text += first ? "" : ", ", text += testing::describe(items), first = false), ...);
            },
            value);
        return text + ")";
    } else if constexpr (std::formattable<T, char>) {
        return std::format("{}", value);
    } else {
        return "<a value that can't be shown>";
    }
}

template <class T> std::string describe_one(const std::optional<T>& value) {
    return value ? "Some(" + testing::describe(*value) + ")" : std::string("None");
}

template <class... Ts> std::string describe_one(const std::variant<Ts...>& value) {
    return std::visit([](const auto& alternative) { return testing::describe(alternative); }, value);
}

// Whole numbers compare by value, whatever their signs and sizes, as the
// compiler would warn about otherwise.
template <class A, class B> bool equal(const A& a, const B& b) {
    if constexpr (std::integral<A> && std::integral<B> && !std::same_as<A, bool> && !std::same_as<B, bool> && !std::same_as<A, char32_t> && !std::same_as<B, char32_t>) {
        return std::cmp_equal(a, b);
    } else {
        return a == b;
    }
}

[[noreturn]] void fail_check(std::string_view what, std::string message, std::source_location where);
[[noreturn]] void fail_compare(std::string_view comparison, std::string_view left_text, std::string_view right_text, std::string left, std::string right, std::string message,
                               std::source_location where);

template <class A, class B>
void check_eq(const A& a, const B& b, std::string_view a_text, std::string_view b_text, std::string message, std::source_location where = std::source_location::current()) {
    if (!equal(a, b)) {
        fail_compare("==", a_text, b_text, testing::describe(a), testing::describe(b), std::move(message), where);
    }
}

template <class A, class B>
void check_ne(const A& a, const B& b, std::string_view a_text, std::string_view b_text, std::string message, std::source_location where = std::source_location::current()) {
    if (equal(a, b)) {
        fail_compare("!=", a_text, b_text, testing::describe(a), testing::describe(b), std::move(message), where);
    }
}

inline std::string message() {
    return {};
}

template <class... Arguments> std::string message(std::format_string<Arguments...> format, Arguments&&... arguments) {
    return std::format(format, std::forward<Arguments>(arguments)...);
}

// Runs the tests that the arguments ask for, and returns the exit code.
int run();

} // namespace detail

template <class T> std::string describe(const T& value) {
    return detail::describe_one(value);
}

} // namespace testing

// Defines a test. Its name is a sentence in snake_case that says what holds.
#define TEST(name)                                                                                                                                                                 \
    static void name();                                                                                                                                                            \
    static const ::testing::Registration name##_registration{__FILE__, #name, &name};                                                                                              \
    static void name()

// assert!(condition) and assert!(condition, "message {}", argument).
#define CHECK(condition, ...)                                                                                                                                                      \
    do {                                                                                                                                                                           \
        if (!(condition)) {                                                                                                                                                        \
            ::testing::detail::fail_check(#condition, ::testing::detail::message(__VA_ARGS__), std::source_location::current());                                                   \
        }                                                                                                                                                                          \
    } while (false)

// assert_eq!(a, b) and assert_ne!(a, b), with a message like CHECK's.
#define CHECK_EQ(a, b, ...) ::testing::detail::check_eq((a), (b), #a, #b, ::testing::detail::message(__VA_ARGS__))
#define CHECK_NE(a, b, ...) ::testing::detail::check_ne((a), (b), #a, #b, ::testing::detail::message(__VA_ARGS__))

// What #[should_panic] is: the expression must throw, as a panic does.
#define CHECK_THROWS(expression)                                                                                                                                                   \
    do {                                                                                                                                                                           \
        bool threw = false;                                                                                                                                                        \
        try {                                                                                                                                                                      \
            (void)(expression);                                                                                                                                                    \
        } catch (const ::testing::Failure&) {                                                                                                                                      \
            throw;                                                                                                                                                                 \
        } catch (...) {                                                                                                                                                            \
            threw = true;                                                                                                                                                          \
        }                                                                                                                                                                          \
        if (!threw) {                                                                                                                                                              \
            ::testing::detail::fail_check(#expression " throws", std::string(), std::source_location::current());                                                                  \
        }                                                                                                                                                                          \
    } while (false)
