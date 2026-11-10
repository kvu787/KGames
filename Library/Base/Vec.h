// A vector of the items, each moved in: what vec![a, b, c] is in Rust. A
// std::vector's own braces copy their items, which costs an allocation for
// each string in them, and which an item that's never copied, like a menu's,
// can't be:
//
//     std::vector<MenuItem<Message>> items = base::vec(
//         MenuItem<Message>("Open").on_choose(Message::Open{}),
//         MenuItem<Message>::separator());
//
// The vector is of the first item's type, which the rest convert to.

#pragma once

#include <type_traits>
#include <utility>
#include <vector>

namespace base {

template <class First, class... Rest> std::vector<std::remove_cvref_t<First>> vec(First&& first, Rest&&... rest) {
    std::vector<std::remove_cvref_t<First>> items;
    items.reserve(1 + sizeof...(rest));
    items.emplace_back(std::forward<First>(first));
    (items.emplace_back(std::forward<Rest>(rest)), ...);
    return items;
}

} // namespace base
