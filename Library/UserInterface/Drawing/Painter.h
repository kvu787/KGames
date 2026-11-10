// What a frame draws: shapes and text, as instances of one quad that the GPU
// draws in one call.
//
// So far, this has only Layer, which a widget's targets are on too. The
// rest, which is the brushes, the draw list and the Painter that widgets and
// programs draw with, is still to come.

#pragma once

#include <array>
#include <cstdint>

namespace user_interface {

// Layers draw in order: a dialog covers the page, a popup like a combo box's
// list covers both, and a tooltip covers everything.
enum class Layer : std::uint8_t {
    Base,
    Dialog,
    Popup,
    Tooltip,
};

inline constexpr std::array<Layer, 4> AllLayers{Layer::Base, Layer::Dialog, Layer::Popup, Layer::Tooltip};

} // namespace user_interface
