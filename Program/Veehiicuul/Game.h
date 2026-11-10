// The game: the document, which has the scenes and the vehicles, the models,
// the car in the current scene, the camera, and what each frame's controls
// do to them in Play. Drawing.h draws it, and the editors change the
// document.
//
// The car starts behind the start line, and the player drives it around the
// track. If it touches an edge of the road, it goes back to the start, at
// rest, and so does switching to another vehicle or scene.

#pragma once

#include "Veehiicuul/Camera.h"
#include "Veehiicuul/Collision.h"
#include "Veehiicuul/Controls.h"
#include "Veehiicuul/Document.h"
#include "Veehiicuul/Driving.h"
#include "Veehiicuul/Models.h"
#include "Veehiicuul/Scene.h"
#include "Veehiicuul/Settings.h"
#include "Veehiicuul/Track.h"
#include "Veehiicuul/Vehicles.h"
#include "Json/Json.h"

#include <array>
#include <cstddef>
#include <cstdint>
#include <expected>
#include <filesystem>
#include <optional>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

namespace veehiicuul::game {

// After the car is put back at the start, the controls don't move it for
// this long, so that the player can let go of the stick first.
inline constexpr double Pause = 0.35;
// The longest that the car moves in one frame, in seconds, so that a frame
// that comes late, like after the window was minimized, doesn't carry it
// far.
inline constexpr double LongestStep = 0.1;
// The car drives in steps of this long, as many as a frame's time holds,
// so that it drives the same at any frame rate: the same push for the same
// time puts it in the same place. What's left of a frame's time waits for
// the next.
inline constexpr double DriveStep = 1.0 / 240.0;

class Game {
public:
    document::Document document;
    models::Models models;
    driving::DeadZones dead_zones;
    // The current scene's and the vehicle's indices.
    size_t scene;
    size_t vehicle;
    driving::Car car;
    camera::Camera camera;
    // How many times the car touched an edge, since the game started.
    std::uint32_t crashes;
    bool show_card;

    // The game, in the scene and with the vehicle that the settings name,
    // or the first of each, with a notice about Settings.json, if they name
    // none.
    Game(document::Document game_document, models::Models game_models, const settings::GameSettings& settings, std::vector<std::string>& notices);
    Game(const Game&) = delete;
    Game& operator=(const Game&) = delete;
    Game(Game&&) = default;
    Game& operator=(Game&&) = default;

    // The game with the scenes, vehicles and models in `folder`.
    static std::expected<Game, std::string> load(const std::filesystem::path& folder, const settings::GameSettings& settings, std::vector<std::string>& notices);

    const scene::Scene& current_scene() const;

    const vehicles::Vehicle& current_vehicle() const;

    collision::Footprint footprint() const;

    // The current scene's track's shape, made again if the track changed.
    const track::Shape& shape();

    // The shape as of the last call to shape.
    const track::Shape& last_shape() const;

    // The height of the camera's view before the player zooms it.
    double default_height() const;

    // Where the camera looks, on the ground.
    std::array<double, 2> camera_center() const;

    // Whether the controls can't move the car yet, as it was just put back
    // at the start.
    bool paused() const;

    // Puts the car on the start, at rest, at `seconds`.
    void put_at_start(double seconds);

    // Goes to a scene, from the start.
    void go_to_scene(size_t index, double seconds);

    // Keeps the indices in the document, after an editor changed it.
    void keep_in_document();

    // Moves the game to a frame's time, `seconds`, with its controls.
    void update(const controls::Controls& controls, double seconds);

    // What scripts save and check about Play.
    std::vector<std::pair<std::string_view, json::Value>> state() const;

private:
    // When the car was last put at the start, and the last frame's time, in
    // seconds.
    double restarted;
    std::optional<double> last;
    // Time that the last frame's drive steps left over.
    double pending;
    // The current scene's track's shape, and the track it's of.
    track::Shape shape_;
    track::Track shape_of;
};

} // namespace veehiicuul::game
