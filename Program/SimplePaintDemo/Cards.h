// Describes the demo's cards from the editor's state, with
// Library/UserInterface's widgets, and what the input on each one means.
// Programs support two client areas, 2560 x 1369 maximized and 2560 x 1440
// in borderless fullscreen, so the cards have fixed places in pixels. Of the
// 71 pixels that borderless fullscreen adds to the height, the preview and
// the GPU constants card get all of them, as each is at the bottom of its
// column.
//
// The preview card has a view in it, which the demo's render fills with the
// sphere before the interface draws over the rest of the frame.

#pragma once

#include "SimplePaintDemo/Demo.h"
#include "SimplePaintDemo/Preview.h"
#include "UserInterface/Core/Interface.h"
#include "UserInterface/Core/Tree.h"
#include "UserInterface/Design/Tokens.h"

namespace simple_paint_demo::cards {

// The preview's card, whose view is as wide as the sphere's image.
inline constexpr float PreviewWidth = static_cast<float>(preview::ImageWidth) + user_interface::tokens::CardPadding * 2.0f;
// The height of the settings card, above the GPU constants.
inline constexpr float SettingsHeight = 468.0f;

user_interface::Tree<demo::Message> view(const demo::Demo& demo, user_interface::Ui& ui);

} // namespace simple_paint_demo::cards
