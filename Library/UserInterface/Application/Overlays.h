// Windows without a frame that float over the others, like IdentifyDisplays'
// boxes, one on each display. Each has its own interface, drawn with one
// GPU, and they draw only until their contents show: they don't change, so
// the loop then only checks the input, every 4 ms or so, or as often as the
// system clock ticks, which is every 15.6 ms unless a process asked for a
// finer timer resolution, as waits time out on its ticks. A
// click on any of them, or Esc while one has focus, closes them all. The
// input comes from GameInput, like every program's.

#pragma once

#include "Base/Windows.h"
#include "UserInterface/Core/Interface.h"
#include "UserInterface/Core/Tree.h"
#include "UserInterface/Design/Theme.h"

#include <cstddef>
#include <expected>
#include <functional>
#include <span>
#include <string>
#include <variant>

namespace user_interface {

// An overlay's title, and where it is on the desktop, in physical pixels.
struct Overlay {
    std::string title;
    RECT rect;
};

// What run_overlays is made of, in Overlays.cpp.
namespace overlays_detail {

// An overlay's tree, given its interface and its index.
using ViewFunction = std::function<Tree<std::monostate>(Ui&, size_t)>;

// run_overlays, with its view behind a std::function, so that all of it is
// compiled once.
std::expected<void, std::string> run_overlays(std::span<const Overlay> overlays, ThemeKind theme, const ViewFunction& view);

} // namespace overlays_detail

// Shows the overlays, with focus, and draws each with the tree that `view`
// describes, given its index, until one closes. `view` is called with a Ui&
// and the overlay's index, a size_t, and returns a Tree<std::monostate>, as
// overlays send no messages.
template <class F> std::expected<void, std::string> run_overlays(std::span<const Overlay> overlays, ThemeKind theme, F&& view) {
    // The std::function holds only a reference to the view, so making it
    // allocates nothing.
    return overlays_detail::run_overlays(overlays, theme, [&view](Ui& ui, size_t index) -> Tree<std::monostate> { return view(ui, index); });
}

} // namespace user_interface
