#include "Veehiicuul/Scene.h"

#include "Base/Files.h"
#include "Base/Sum.h"
#include "Base/Text.h"

#include <algorithm>
#include <format>
#include <system_error>
#include <utility>

namespace veehiicuul::scene {

using json::Value;
using terrain::Terrain;
using track::Pose;
using track::Track;

namespace {

constexpr std::array<std::string_view, 7> Members{"Background", "Camera", "FollowHeight", "Terrain", "Track", "Decorations", "Vehicles"};

std::expected<Pose, std::string> pose(const Value& value, std::string_view name) {
    std::expected<std::array<double, 2>, std::string> position =
        files::member(value, "Position", std::format("{}.", name)).and_then([&](const Value* found) { return files::point(*found, std::format("{}.Position", name)); });
    if (!position) {
        return std::unexpected(std::move(position.error()));
    }
    std::expected<double, std::string> heading =
        files::member(value, "Heading", std::format("{}.", name)).and_then([&](const Value* found) { return files::number(*found, std::format("{}.Heading", name)); });
    if (!heading) {
        return std::unexpected(std::move(heading.error()));
    }
    return Pose{
        .position = *position,
        .heading = *heading,
    };
}

std::expected<Decoration, std::string> decoration(const Value& value, std::string_view name) {
    if (std::expected<void, std::string> known = files::only(value, {{"Model", "Position", "Heading", "Scale"}}, name); !known) {
        return std::unexpected(std::move(known.error()));
    }
    std::expected<double, std::string> scale =
        files::member(value, "Scale", std::format("{}.", name)).and_then([&](const Value* found) { return files::positive(*found, std::format("{}.Scale", name)); });
    if (!scale) {
        return std::unexpected(std::move(scale.error()));
    }
    if (!(SmallestScale <= *scale && *scale <= LargestScale)) {
        return std::unexpected(std::format("{}.Scale must be from {} to {}, not {}", name, base::display(SmallestScale), base::display(LargestScale), base::display(*scale)));
    }
    std::expected<std::string, std::string> model =
        files::member(value, "Model", std::format("{}.", name)).and_then([&](const Value* found) { return files::text(*found, std::format("{}.Model", name)); });
    if (!model) {
        return std::unexpected(std::move(model.error()));
    }
    std::expected<Pose, std::string> placed = pose(value, name);
    if (!placed) {
        return std::unexpected(std::move(placed.error()));
    }
    return Decoration{
        .model = std::move(*model),
        .pose = *placed,
        .scale = *scale,
    };
}

std::expected<Parked, std::string> parked(const Value& value, std::string_view name) {
    if (std::expected<void, std::string> known = files::only(value, {{"Vehicle", "Position", "Heading"}}, name); !known) {
        return std::unexpected(std::move(known.error()));
    }
    std::expected<std::string, std::string> vehicle =
        files::member(value, "Vehicle", std::format("{}.", name)).and_then([&](const Value* found) { return files::text(*found, std::format("{}.Vehicle", name)); });
    if (!vehicle) {
        return std::unexpected(std::move(vehicle.error()));
    }
    std::expected<Pose, std::string> placed = pose(value, name);
    if (!placed) {
        return std::unexpected(std::move(placed.error()));
    }
    return Parked{
        .vehicle = std::move(*vehicle),
        .pose = *placed,
    };
}

} // namespace

Scene::Scene(std::string_view scene_name) :
    name(scene_name), background{0x1C, 0xDA, 0x61}, camera_center{0.0, 0.0}, camera_height(100.0), follow_height(45.0), terrain(), track(Track::round()) {}

Scene::Scene(std::string scene_name, files::Color background_color, std::array<double, 2> center, double height, double following_height, Terrain scene_terrain, Track scene_track,
             std::vector<Decoration> scene_decorations, std::vector<Parked> parked_vehicles) :
    name(std::move(scene_name)),
    background(background_color),
    camera_center(center),
    camera_height(height),
    follow_height(following_height),
    terrain(std::move(scene_terrain)),
    track(std::move(scene_track)),
    decorations(std::move(scene_decorations)),
    parked(std::move(parked_vehicles)) {}

std::expected<Scene, std::string> Scene::parse(std::string_view scene_name, std::string_view text) {
    std::expected<Value, std::string> value = files::parse(text);
    if (!value) {
        return std::unexpected(std::move(value.error()));
    }
    if (std::expected<void, std::string> known = files::only(*value, Members, "a scene"); !known) {
        return std::unexpected(std::move(known.error()));
    }
    auto member = [&](std::string_view member_name) {
        return files::member(*value, member_name, "");
    };
    std::expected<const Value*, std::string> camera = member("Camera");
    if (!camera) {
        return std::unexpected(std::move(camera.error()));
    }
    if (std::expected<void, std::string> known = files::only(**camera, {{"Center", "Height"}}, "Camera"); !known) {
        return std::unexpected(std::move(known.error()));
    }
    std::expected<const Value*, std::string> decorations_member = member("Decorations");
    if (!decorations_member) {
        return std::unexpected(std::move(decorations_member.error()));
    }
    std::vector<Decoration> placed;
    if (const Value::Array* items = base::as<Value::Array>(**decorations_member)) {
        placed.reserve(items->values.size());
        for (size_t index = 0; index < items->values.size(); index++) {
            std::expected<Decoration, std::string> item = decoration(items->values[index], std::format("Decorations[{}]", index));
            if (!item) {
                return std::unexpected(std::move(item.error()));
            }
            placed.push_back(std::move(*item));
        }
    } else {
        return std::unexpected(std::format("Decorations must be a list, not {}", (*decorations_member)->describe()));
    }
    std::expected<const Value*, std::string> vehicles_member = member("Vehicles");
    if (!vehicles_member) {
        return std::unexpected(std::move(vehicles_member.error()));
    }
    std::vector<Parked> parked_vehicles;
    if (const Value::Array* items = base::as<Value::Array>(**vehicles_member)) {
        parked_vehicles.reserve(items->values.size());
        for (size_t index = 0; index < items->values.size(); index++) {
            // The function above, not the scene's parked vehicles.
            std::expected<Parked, std::string> item = scene::parked(items->values[index], std::format("Vehicles[{}]", index));
            if (!item) {
                return std::unexpected(std::move(item.error()));
            }
            parked_vehicles.push_back(std::move(*item));
        }
    } else {
        return std::unexpected(std::format("Vehicles must be a list, not {}", (*vehicles_member)->describe()));
    }
    std::expected<files::Color, std::string> background_color = member("Background").and_then([](const Value* found) { return files::color(*found, "Background"); });
    if (!background_color) {
        return std::unexpected(std::move(background_color.error()));
    }
    std::expected<std::array<double, 2>, std::string> center =
        files::member(**camera, "Center", "Camera.").and_then([](const Value* found) { return files::point(*found, "Camera.Center"); });
    if (!center) {
        return std::unexpected(std::move(center.error()));
    }
    std::expected<double, std::string> height = files::member(**camera, "Height", "Camera.").and_then([](const Value* found) { return files::positive(*found, "Camera.Height"); });
    if (!height) {
        return std::unexpected(std::move(height.error()));
    }
    std::expected<double, std::string> following_height = member("FollowHeight").and_then([](const Value* found) { return files::positive(*found, "FollowHeight"); });
    if (!following_height) {
        return std::unexpected(std::move(following_height.error()));
    }
    std::expected<Terrain, std::string> scene_terrain = member("Terrain").and_then([](const Value* found) { return Terrain::parse(*found); });
    if (!scene_terrain) {
        return std::unexpected(std::move(scene_terrain.error()));
    }
    std::expected<Track, std::string> scene_track = member("Track").and_then([](const Value* found) { return Track::parse(*found); });
    if (!scene_track) {
        return std::unexpected(std::move(scene_track.error()));
    }
    return Scene(std::string(scene_name), *background_color, *center, *height, *following_height, std::move(*scene_terrain), std::move(*scene_track), std::move(placed),
                 std::move(parked_vehicles));
}

std::string Scene::to_json() const {
    Value::Array placed;
    placed.values.reserve(decorations.size());
    for (const Decoration& decoration : decorations) {
        placed.values.push_back(json::object({
            {"Model", decoration.model},
            {"Position", files::point_value(decoration.pose.position)},
            {"Heading", files::rounded(decoration.pose.heading)},
            {"Scale", files::rounded(decoration.scale)},
        }));
    }
    Value::Array parked_vehicles;
    parked_vehicles.values.reserve(parked.size());
    for (const Parked& vehicle : parked) {
        parked_vehicles.values.push_back(json::object({
            {"Vehicle", vehicle.vehicle},
            {"Position", files::point_value(vehicle.pose.position)},
            {"Heading", files::rounded(vehicle.pose.heading)},
        }));
    }
    return json::object({
                            {"Background", files::color_text(background)},
                            {"Camera", json::object({
                                           {"Center", files::point_value(camera_center)},
                                           {"Height", files::rounded(camera_height)},
                                       })},
                            {"FollowHeight", files::rounded(follow_height)},
                            {"Terrain", terrain.to_value()},
                            {"Track", track.to_value()},
                            {"Decorations", std::move(placed)},
                            {"Vehicles", std::move(parked_vehicles)},
                        })
               .to_json() +
           "\n";
}

std::expected<Scene, std::string> Scene::load(const std::filesystem::path& path) {
    std::string scene_name = base::display(path.stem());
    std::expected<std::string, std::string> text = files::read(path);
    if (!text) {
        return std::unexpected(std::move(text.error()));
    }
    std::expected<Scene, std::string> scene = Scene::parse(scene_name, *text);
    if (!scene) {
        return std::unexpected(std::format("{}: {}", base::display(path), scene.error()));
    }
    return scene;
}

std::expected<std::vector<Scene>, std::string> Scene::load_all(const std::filesystem::path& folder) {
    std::error_code error;
    std::filesystem::directory_iterator entries(folder, error);
    if (error) {
        return std::unexpected(std::format("Couldn't read {}: {}", base::display(folder), base::os_error(static_cast<unsigned long>(error.value()))));
    }
    std::vector<std::filesystem::path> paths;
    for (; entries != std::filesystem::directory_iterator(); entries.increment(error)) {
        std::filesystem::path path = entries->path();
        if (base::eq_ignore_ascii_case(base::display(path.extension()), ".json")) {
            paths.push_back(std::move(path));
        }
    }
    // By their names' bytes, as UTF-8.
    std::ranges::stable_sort(paths, {}, [](const std::filesystem::path& path) { return base::display(path); });
    if (paths.empty()) {
        return std::unexpected(std::format("{} has no scenes", base::display(folder)));
    }
    std::vector<Scene> scenes;
    scenes.reserve(paths.size());
    for (const std::filesystem::path& path : paths) {
        std::expected<Scene, std::string> scene = Scene::load(path);
        if (!scene) {
            return std::unexpected(std::move(scene.error()));
        }
        scenes.push_back(std::move(*scene));
    }
    return scenes;
}

} // namespace veehiicuul::scene
