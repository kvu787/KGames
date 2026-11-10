// A scene: what a level of the game is, from its file in Scenes. It has the
// terrain, the track, the decorations, which are models, and vehicles parked
// as decorations, all on the ground, with the background's color where
// there's nothing, and the camera's view of it all.
//
//     {
//         "Background": "#1CDA61",
//         "Camera": { "Center": [-3.5, -15.3], "Height": 150 },
//         "FollowHeight": 45,
//         "Terrain": { ... },
//         "Track": { ... },
//         "Decorations": [{ "Model": "BigCone", "Position": [-53.4, -62.2], "Heading": 28, "Scale": 4.9 }],
//         "Vehicles": [{ "Vehicle": "Green", "Position": [37.6, -75.5], "Heading": -32.1 }]
//     }
//
// See Terrain.h and Track.h for theirs.

#pragma once

#include "Veehiicuul/Files.h"
#include "Veehiicuul/Terrain.h"
#include "Veehiicuul/Track.h"

#include <array>
#include <expected>
#include <filesystem>
#include <string>
#include <string_view>
#include <vector>

namespace veehiicuul::scene {

// A model put somewhere, turned, and scaled the same along every axis.
struct Decoration {
    std::string model;
    track::Pose pose;
    double scale = 0.0;

    bool operator==(const Decoration&) const = default;
};

// A vehicle put somewhere, as a decoration, which doesn't move.
struct Parked {
    std::string vehicle;
    track::Pose pose;

    bool operator==(const Parked&) const = default;
};

// The scale that decorations stay within, for SimplePaint.
inline constexpr double SmallestScale = 0.01;
inline constexpr double LargestScale = 100.0;

struct Scene {
    // Its file's name, without ".json".
    std::string name;
    files::Color background;
    // The fixed camera looks at `camera_center`, at a view `camera_height`
    // units tall, and the camera that follows the car at one
    // `follow_height` tall.
    std::array<double, 2> camera_center;
    double camera_height;
    double follow_height;
    terrain::Terrain terrain;
    track::Track track;
    std::vector<Decoration> decorations;
    std::vector<Parked> parked;

    // A new scene, with a round track, called `scene_name`.
    explicit Scene(std::string_view scene_name);

    // A scene of these parts.
    Scene(std::string scene_name, files::Color background_color, std::array<double, 2> center, double height, double following_height, terrain::Terrain scene_terrain,
          track::Track scene_track, std::vector<Decoration> scene_decorations, std::vector<Parked> parked_vehicles);

    static std::expected<Scene, std::string> parse(std::string_view scene_name, std::string_view text);

    std::string to_json() const;

    static std::expected<Scene, std::string> load(const std::filesystem::path& path);

    // Every scene in a folder, from its .json files, by name.
    static std::expected<std::vector<Scene>, std::string> load_all(const std::filesystem::path& folder);

    bool operator==(const Scene&) const = default;
};

} // namespace veehiicuul::scene
