// Veehiicuul's parts, as a library, so that Test/Veehiicuul can test them
// without a window: the scenes, with their terrain and tracks, the models
// and vehicles, how a car drives and what it touches, the editors that make
// them, and the game that plays them. Main.cpp is the program.

#pragma once

#include "Veehiicuul/Camera.h"
#include "Veehiicuul/Collision.h"
#include "Veehiicuul/Controls.h"
#include "Veehiicuul/Curve.h"
#include "Veehiicuul/Document.h"
#include "Veehiicuul/Drawing.h"
#include "Veehiicuul/Driving.h"
#include "Veehiicuul/Files.h"
#include "Veehiicuul/Game.h"
#include "Veehiicuul/Ground.h"
#include "Veehiicuul/Models.h"
#include "Veehiicuul/Scene.h"
#include "Veehiicuul/Settings.h"
#include "Veehiicuul/Terrain.h"
#include "Veehiicuul/Track.h"
#include "Veehiicuul/Vehicles.h"

#include <string_view>

namespace veehiicuul {

inline constexpr std::string_view Name = "Veehiicuul";
// The game's folder, which it reads its files from: its scenes, vehicles,
// models and settings.
extern const std::string_view Folder;

} // namespace veehiicuul
