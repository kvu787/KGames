// SpinningCarTest's settings: the materials, the sphere's resolution, the
// file that it writes, and what the card says about a setting that's wrong.

#include "Base/Files.h"
#include "SimplePaint/SimplePaint.h"
#include "SpinningCarTest/SpinningCarTest.h"
#include "Testing/Testing.h"
#include "UserInterface/UserInterface.h"

// Where the repository is, for the program's settings file.
#include "Folders.h"

#include <array>
#include <cstdint>
#include <filesystem>
#include <format>
#include <optional>
#include <string>
#include <string_view>
#include <system_error>
#include <tuple>
#include <vector>

using simple_paint::Material;
using simple_paint::Parameters;
using spinning_car_test::settings::SceneSettings;
using spinning_car_test::spinning_car::SpinningCar;
using user_interface::Headless;
using user_interface::InfoCard;
using user_interface::Rect;
using user_interface::Settings;
using user_interface::Startup;

namespace {

// What's wrong with the settings, as a list that a check compares.
std::vector<std::string> problems(const Settings& settings) {
    return std::vector<std::string>(settings.problems().begin(), settings.problems().end());
}

} // namespace

TEST(the_defaults_are_materials_that_simple_paint_accepts) {
    SceneSettings scene;
    CHECK_EQ(scene.sphere_resolution, (std::array<std::uint32_t, 2>{64, 32}));
    // The body's.
    CHECK_EQ(scene.materials[1].base_color_srgb, (std::array<double, 3>{0.103999995, 0.3592001, 0.8}));
    CHECK_EQ(scene.materials[1].rotation_degrees, 35.600586);
    for (const Parameters& material : scene.materials) {
        CHECK(Material::compile(material).has_value());
    }
}

TEST(the_file_it_writes_reads_back_as_the_defaults) {
    std::filesystem::path folder = std::filesystem::temp_directory_path() / "SpinningCarSettings";
    std::error_code ignored;
    std::filesystem::remove_all(folder, ignored);
    CHECK(base::create_dir_all(folder).has_value());
    std::filesystem::path path = folder / "Settings.json";
    Settings settings = Settings::load(path);
    CHECK_EQ(SceneSettings::read(settings), SceneSettings{});
    CHECK(settings.write_if_missing().has_value());
    Settings written = Settings::load(path);
    CHECK_EQ(SceneSettings::read(written), SceneSettings{});
    CHECK(written.problems().empty(), "{}", testing::describe(written.problems()));
}

TEST(the_program_s_settings_file_has_the_defaults) {
    std::filesystem::path path = std::filesystem::path(folders::Repository) / "Program" / "SpinningCarTest" / "Settings.json";
    Settings settings = Settings::load(path);
    CHECK_EQ(SceneSettings::read(settings), SceneSettings{});
    CHECK(settings.problems().empty(), "{}", testing::describe(settings.problems()));
}

TEST(the_settings_check_the_sphere_s_resolution) {
    using Case = std::tuple<std::string_view, std::array<std::uint32_t, 2>, std::optional<std::string_view>>;
    for (const auto& [sphere, resolution, problem] : {
             Case(R"({ "UResolution": 4, "VResolution": 512 })", {4, 512}, std::nullopt),
             Case(R"({ "VResolution": 2 })", {64, 2}, std::nullopt),
             Case(R"({ "UResolution": 3 })", {64, 32}, "UResolution must be a whole number from 4 to 512, not 3"),
             Case(R"({ "VResolution": 513 })", {64, 32}, "VResolution must be a whole number from 2 to 512, not 513"),
             Case(R"({ "UResolution": 64.5 })", {64, 32}, "UResolution must be a whole number from 4 to 512, not 64.5"),
             Case(R"({ "Radius": 1 })", {64, 32}, "has Radius, which isn't one of UResolution and VResolution"),
             Case(R"({ "UResolution": 128, "VResolution": 600 })", {64, 32}, "VResolution must be a whole number from 2 to 512, not 600"),
             Case("64", {64, 32}, "must be an object with UResolution and VResolution, not 64"),
         }) {
        Settings settings = Settings::from_text(std::format(R"({{ "Sphere": {} }})", sphere));
        SceneSettings read = SceneSettings::read(settings);
        CHECK_EQ(read.sphere_resolution, resolution, "for {}", sphere);
        std::vector<std::string> expected;
        if (problem) {
            expected.push_back(std::format("Sphere {}, so it's the default.", *problem));
        }
        CHECK_EQ(problems(settings), expected, "for {}", sphere);
    }
}

TEST(a_material_that_is_wrong_keeps_its_default) {
    Settings settings = Settings::from_text(R"({ "SimplePaintShader_Body": { "Brightness": 0.25 }, "SimplePaintShader_Wheels": { "Shift": 2 } })");
    SceneSettings read = SceneSettings::read(settings);
    SceneSettings defaults;
    CHECK_EQ(read.materials[1].brightness, 0.25);
    CHECK_EQ(read.materials[4], defaults.materials[4], "a material SimplePaint rejects keeps its default");
    CHECK_EQ(problems(settings), std::vector<std::string>{"SimplePaintShader_Wheels SimplePaint Shift must be finite and in [0, 0.9990234375], so it's the default."});
}

TEST(the_card_says_what_is_wrong_with_the_settings) {
    std::filesystem::path folder = std::filesystem::temp_directory_path() / "SpinningCarNotices";
    CHECK(base::create_dir_all(folder).has_value());
    Startup startup = Startup::for_tests(R"({ "Sphere": { "UResolution": 1000 } })", folder);
    Headless<SpinningCar> app = Headless<SpinningCar>::start(std::move(startup), SpinningCar::start);
    CHECK_EQ(app.program.settings, SceneSettings{});
    std::optional<std::string> label = app.label("Notice/0");
    CHECK(label.has_value());
    CHECK(label->starts_with("Warning: Settings.json Sphere "), "{}", *label);
    Rect card = app.target(InfoCard::Name);
    CHECK(card.contains(app.target("Notice/0").center()));
}
