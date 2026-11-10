// Sum types: a value that's one of several alternatives, each with its own
// data, which is what an enum is in Rust, and std::variant in C++. One is a
// struct with its alternatives as nested structs, a `value` that holds one of
// them, and a constructor from each:
//
//     struct Part {
//         struct Whole {
//             bool operator==(const Whole&) const = default;
//         };
//         struct Index {
//             size_t index;
//             bool operator==(const Index&) const = default;
//         };
//         struct Named {
//             std::string name;
//             bool operator==(const Named&) const = default;
//         };
//         std::variant<Whole, Index, Named> value;
//         template <base::AlternativeOf<Part> T> Part(T&& alternative) : value(std::forward<T>(alternative)) {}
//         bool operator==(const Part&) const = default;
//     };
//
// So Part::Index{3} is a Part, and so is Part::Whole{}. Code asks which it is
// with base::is and base::as, or handles every alternative with base::match,
// which doesn't compile if one is missing:
//
//     if (auto* index = base::as<Part::Index>(part)) { ... index->index ... }
//     base::match(part,
//         [&](const Part::Whole&) { ... },
//         [&](const Part::Index& index) { ... },
//         [&](const Part::Named& named) { ... });
//
// An arm is a lambda, so it can't return from the function it's in, or break
// out of a loop there. Code that needs to does it with base::as instead.
//
// base::make<Part::Index> is a function that makes that alternative from its
// members' values, for where a function is wanted, like a widget's reaction
// to its value, which makes the program's message from it:
//
//     Slider<Message>(volume, 0.0f, 1.0f).on_change(base::make<Message::Volume>)

#pragma once

#include <concepts>
#include <type_traits>
#include <utility>
#include <variant>

namespace base {

// The lambdas, as one callable with an overload for each.
template <class... Arms> struct Overloaded : Arms... {
    using Arms::operator()...;
};

namespace detail {

template <class T, class Variant> struct IsAlternative : std::false_type {};
template <class T, class... Ts> struct IsAlternative<T, std::variant<Ts...>> : std::bool_constant<(std::same_as<T, Ts> || ...)> {};

} // namespace detail

// One of the alternatives of a sum type's `value`, for the sum type's
// constructor. Nothing converts to an alternative on the way in: only an
// alternative itself makes the sum type.
template <class T, class Sum>
concept AlternativeOf = detail::IsAlternative<std::remove_cvref_t<T>, decltype(Sum::value)>::value;

// Something with a `value` that's a std::variant.
template <class S>
concept SumType = requires(S& sum) { std::variant_size<std::remove_cvref_t<decltype(sum.value)>>::value; };

// Whether the sum type holds the alternative.
template <class T, SumType S> bool is(const S& sum) {
    return std::holds_alternative<T>(sum.value);
}

// The alternative, if the sum type holds it, or null.
template <class T, SumType S> const T* as(const S& sum) {
    return std::get_if<T>(&sum.value);
}

template <class T, SumType S> T* as(S& sum) {
    return std::get_if<T>(&sum.value);
}

// Makes the alternative from its members' values, in their order.
template <class T>
inline constexpr auto make = [](auto&&... values) { return T{std::forward<decltype(values)>(values)...}; };

// Calls the arm for the alternative that the sum type holds.
template <SumType S, class... Arms> decltype(auto) match(S&& sum, Arms&&... arms) {
    return std::visit(Overloaded{std::forward<Arms>(arms)...}, std::forward<S>(sum).value);
}

// The same, for a plain std::variant.
template <class... Ts, class... Arms> decltype(auto) match(const std::variant<Ts...>& variant, Arms&&... arms) {
    return std::visit(Overloaded{std::forward<Arms>(arms)...}, variant);
}

template <class... Ts, class... Arms> decltype(auto) match(std::variant<Ts...>& variant, Arms&&... arms) {
    return std::visit(Overloaded{std::forward<Arms>(arms)...}, variant);
}

} // namespace base
