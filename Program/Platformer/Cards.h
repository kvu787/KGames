// Describes the platformer's interface from the game's state: the world,
// drawn into a canvas that fills the window between the bars, the Run card
// and the Events card floating over it, and a banner when the run ends. The
// world's look is the level's own colors; everything else colors with the
// theme's roles, as the design system says.

#pragma once

#include "Platformer/Game.h"
#include "Platformer/Program.h"
#include "UserInterface/Core/Interface.h"
#include "UserInterface/Core/Tree.h"
#include "UserInterface/Drawing/Geometry.h"

namespace platformer::cards {

// Pixels per world unit.
inline constexpr float Scale = 48.0f;

// Where the world's canvas is: the window between the app bar and the
// status bar.
user_interface::Rect view_area(const user_interface::Ui& ui);

user_interface::Tree<program::Message> view(const game::Game& game, user_interface::Ui& ui);

} // namespace platformer::cards
