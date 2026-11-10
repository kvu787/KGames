// What the game's files are: the models from Blender, and the document of
// scenes, vehicles and paints, which read and write back the same, and the
// game's own settings in Settings.json.

#include "Veehiicuul/Document.h"
#include "Base/Files.h"
#include "Base/Panic.h"
#include "Base/Text.h"
#include "SimplePaint/Material.h"
#include "Testing/Testing.h"
#include "UserInterface/Application/Settings.h"
#include "Veehiicuul/Collision.h"
#include "Veehiicuul/Driving.h"
#include "Veehiicuul/Files.h"
#include "Veehiicuul/Models.h"
#include "Veehiicuul/Scene.h"
#include "Veehiicuul/Settings.h"
#include "Veehiicuul/Vehicles.h"

#include "Folders.h"

#include <algorithm>
#include <array>
#include <cmath>
#include <cstddef>
#include <expected>
#include <filesystem>
#include <format>
#include <optional>
#include <string>
#include <string_view>
#include <tuple>
#include <utility>
#include <vector>

namespace files = veehiicuul::files;

using simple_paint::Material;
using simple_paint::Parameters;
using user_interface::Settings;
using veehiicuul::collision::Footprint;
using veehiicuul::document::Document;
using veehiicuul::driving::Acceleration;
using veehiicuul::driving::DeadZone;
using veehiicuul::models::Model;
using veehiicuul::models::Models;
using veehiicuul::models::paint_file;
using veehiicuul::models::paint_from_color;
using veehiicuul::scene::Scene;
using veehiicuul::settings::GameSettings;
using veehiicuul::vehicles::Vehicle;
using veehiicuul::vehicles::Vehicles;

namespace {

std::filesystem::path folder() {
    return std::filesystem::path(folders::Project) / "../../Program/Veehiicuul";
}

std::pair<Models, Document> load() {
    std::expected<Models, std::string> models = Models::load(folder() / "Models");
    if (!models) {
        base::panic(models.error());
    }
    std::expected<Document, std::string> document = Document::load(folder(), *models);
    if (!document) {
        base::panic(document.error());
    }
    return {std::move(*models), std::move(*document)};
}

// A file's text, which must be there.
std::string text_of(const std::filesystem::path& path) {
    std::expected<std::string, std::string> text = files::read(path);
    if (!text) {
        base::panic(text.error());
    }
    return std::move(*text);
}

} // namespace

// The models are the .glb files from Blender, each with its slots, which
// are its materials, painted as Blender says until the game paints them.
TEST(models_come_from_blender_with_their_slots) {
    auto [models, document] = load();
    CHECK_EQ(models.names(), (std::vector<std::string_view>{"BigCone", "SlopeCar"}));
    const Model& car = models.list[1];
    CHECK_EQ(car.slots, (std::vector<std::string>{"SlopeCarAxles", "SlopeCarBodyBlue", "SlopeCarCabinBlue", "SlopeCarHeadlights", "SlopeCarWheels"}));
    CHECK(std::abs(car.footprint.min_[0] + 1.5) < 1e-5 && std::abs(car.footprint.min_[1] + 3.0) < 1e-5);
    // Blender's paint is from each material's color.
    const Model& cone = models.list[0];
    auto found = std::ranges::find(cone.slots, "BigConeOrange");
    CHECK(found != cone.slots.end());
    size_t orange = static_cast<size_t>(found - cone.slots.begin());
    CHECK_EQ(cone.blender_paints[orange], paint_from_color({1.0f, 0.268'714'8f, 0.0f, 1.0f}));
    // The car's paint file paints its slots, and the cone has none.
    std::vector<Parameters> materials = document.model_materials(models, 1);
    CHECK_EQ(materials[0].base_color_srgb, (std::array{0.2159, 0.2159, 0.2159}));
    CHECK_EQ(document.model_materials(models, 0), cone.blender_paints);
    for (const Parameters& parameters : materials) {
        std::expected<Material, simple_paint::Error> compiled = Material::compile(parameters);
        CHECK(compiled.has_value(), "{}", compiled.error().message);
    }
}

// Six vehicles, each faster than the one before, each painted its own way
// over the car, with the footprint that touches the track's edges.
TEST(the_vehicles_each_look_and_drive_their_own_way) {
    auto [models, document] = load();
    const Vehicles& vehicles = document.vehicles;
    std::vector<std::string_view> names;
    for (const Vehicle& vehicle : vehicles.list) {
        names.push_back(vehicle.name);
    }
    CHECK_EQ(names, (std::vector<std::string_view>{"Green", "Blue", "Red", "Yellow", "Cyan", "Magenta"}));
    CHECK_EQ(vehicles.list[3].acceleration, (Acceleration{.forward = 140.0, .reverse = 280.0, .left = 140.0, .right = 140.0}));
    for (size_t index = 0; index + 1 < vehicles.list.size(); index++) {
        CHECK(vehicles.list[index].acceleration.forward < vehicles.list[index + 1].acceleration.forward);
    }
    std::vector<std::array<double, 3>> bodies;
    for (size_t index = 0; index < vehicles.list.size(); index++) {
        std::optional<std::pair<size_t, std::vector<Parameters>>> painted = document.vehicle_materials(models, index);
        CHECK(painted.has_value());
        size_t model = painted->first;
        const std::vector<Parameters>& materials = painted->second;
        CHECK_EQ(models.list[model].name, "SlopeCar");
        CHECK(!std::ranges::contains(bodies, materials[1].base_color_srgb), "{} has another's body", vehicles.list[index].name);
        bodies.push_back(materials[1].base_color_srgb);
        // The vehicle paints its body and cabin, and the car paints the rest.
        CHECK_EQ(materials[0], document.model_materials(models, model)[0]);
    }
    const Footprint& yellow = vehicles.list[3].footprint;
    CHECK_EQ(std::pair(yellow.min_, yellow.max_), std::pair(std::array{-1.5, -3.0}, std::array{1.5, 2.9925}));
}

// The files read and write back the same, so that saving what didn't change
// changes nothing, and a change writes only the file that it's in.
TEST(the_files_write_back_as_they_were_and_only_what_changed) {
    auto [models, document] = load();
    for (const Scene& scene : document.scenes) {
        std::string text = text_of(folder() / base::path(std::format("Scenes/{}.json", scene.name)));
        CHECK_EQ(scene.to_json(), text, "Scenes/{}.json", scene.name);
    }
    CHECK_EQ(document.vehicles.to_json(), text_of(folder() / "Vehicles.json"));
    CHECK_EQ(paint_file(document.paint("SlopeCar")), text_of(folder() / "Models/SlopeCar.json"));
    CHECK(document.changed_files(document).empty());
    Document changed = document;
    changed.scenes[0].track.barrier_width = 2.0;
    changed.vehicles.list[0].acceleration.forward = 11.0;
    std::vector<std::string> paths;
    for (std::pair<std::string, std::string>& file : changed.changed_files(document)) {
        paths.push_back(std::move(file.first));
    }
    CHECK_EQ(paths, (std::vector<std::string>{"Scenes/Ribeye.json", "Vehicles.json"}));
}

TEST(says_what_a_document_names_that_is_not_there) {
    auto [models, document] = load();
    CHECK(document.missing(models).empty());
    document.vehicles.list[0].model = "Boat";
    document.scenes[0].parked[1].vehicle = "Truck";
    CHECK_EQ(document.missing(models), (std::vector<std::string>{
                                           "Vehicle Green's model, Boat, isn't in Models",
                                           "Scene Ribeye parks a Truck, which isn't in Vehicles.json",
                                       }));
}

TEST(says_what_is_wrong_with_the_vehicles_file) {
    auto vehicle = [](std::string_view name, std::string_view footprint) {
        return std::format(R"({{ "Name": "{}", "Model": "SlopeCar", "Acceleration": {{ "Forward": 1, "Reverse": 2, "Left": 3, "Right": 4 }}, "Footprint": {} }})", name, footprint);
    };
    std::string_view footprint = R"({ "Left": 1, "Right": 1, "Front": 2, "Back": 2 })";
    auto file = [](std::string_view vehicles) {
        return std::format(R"({{ "Vehicles": [{}] }})", vehicles);
    };
    Vehicles parsed = Vehicles::parse(file(vehicle("Gray", footprint))).value();
    CHECK(parsed.list[0].paint.empty());
    struct Case {
        std::string text;
        std::string_view problem;
    };
    for (const Case& expected : {
             Case{file(""), "Vehicles needs at least one vehicle"},
             Case{file(std::format("{}, {}", vehicle("Gray", footprint), vehicle("Gray", footprint))), "Vehicles[1]: another vehicle is called Gray"},
             Case{file(vehicle("Gray", R"({ "Left": 1, "Right": 1, "Front": 2 })")), "Vehicles[0]: needs Gray's Footprint.Back"},
             Case{file(vehicle("Gray", R"({ "Left": 0, "Right": 1, "Front": 2, "Back": 2 })")), "Vehicles[0]: Gray's Footprint.Left must be above 0, not 0"},
         }) {
        std::expected<Vehicles, std::string> result = Vehicles::parse(expected.text);
        CHECK(!result.has_value(), "{} parsed", expected.text);
        CHECK_EQ(result.error(), expected.problem);
    }
}

namespace {

// Reads the game's settings from Settings.json's text, with what's wrong.
std::pair<GameSettings, std::vector<std::string>> read(std::string_view text) {
    Settings settings = Settings::from_text(text);
    GameSettings game = GameSettings::read(settings);
    return {game, std::vector<std::string>(settings.problems().begin(), settings.problems().end())};
}

} // namespace

TEST(settings_name_the_scene_vehicle_camera_and_dead_zones) {
    auto [defaults, no_problems] = read("{}");
    CHECK(no_problems.empty(), "{}", testing::describe(no_problems));
    CHECK_EQ(defaults, GameSettings());
    CHECK_EQ(std::tuple(std::string_view(defaults.scene), std::string_view(defaults.vehicle), defaults.follow),
             std::tuple(std::string_view("Ribeye"), std::string_view("Yellow"), false));
    auto [following, none] = read(R"({ "Camera": "follow", "DeadZones": { "Sideways": [0.2, 0.9] } })");
    CHECK(none.empty(), "{}", testing::describe(none));
    CHECK(following.follow);
    CHECK_EQ(following.dead_zones.sideways, (DeadZone{.inner = 0.2, .outer = 0.9}));
    CHECK_EQ(following.dead_zones.forward_and_back, (DeadZone{.inner = 0.05, .outer = 0.95}));
    // A setting that's wrong keeps its default, and says why.
    struct Case {
        std::string_view text;
        std::string_view problem;
    };
    for (const Case& expected : {
             Case{R"({ "Camera": "Above" })", "Camera must be \"Fixed\" or \"Follow\", not \"Above\", so it's the default."},
             Case{R"({ "Scene": 3 })", "Scene must be a name, not 3, so it's the default."},
             Case{R"({ "DeadZones": { "Sideways": [0.9, 0.2] } })",
                  "DeadZones has Sideways, which must be [inner, outer], from 0 to 1 with inner below outer, not a list of 2, so it's the default."},
         }) {
        auto [settings, problems] = read(expected.text);
        CHECK_EQ(problems, (std::vector<std::string>{std::string(expected.problem)}));
        CHECK_EQ(settings, GameSettings());
    }
    // The game's own Settings.json is right.
    Settings settings = Settings::load(folder() / "Settings.json");
    GameSettings game = GameSettings::read(settings);
    CHECK(settings.problems().empty(), "{}", testing::describe(settings.problems()));
    CHECK_EQ(std::tuple(std::string_view(game.scene), std::string_view(game.vehicle), game.follow), std::tuple(std::string_view("Ribeye"), std::string_view("Yellow"), false));
}

// A color's channels are two hex digits each, with no sign or space.
TEST(colors_are_hex_digits_only) {
    CHECK_EQ(files::parse_color("#1CDA61"), std::optional(files::Color{0x1C, 0xDA, 0x61}));
    CHECK_EQ(files::parse_color("#1cda61"), std::optional(files::Color{0x1C, 0xDA, 0x61}));
    for (std::string_view bad : {"#+1+2+3", "#1CDA6", "#1CDA611", "1CDA61 ", "#1CDA6G", "# 1DA61"}) {
        CHECK_EQ(files::parse_color(bad), std::nullopt, "{}", base::debug(bad));
    }
    CHECK_EQ(files::color_text({0x1C, 0xDA, 0x61}), "#1CDA61");
}
