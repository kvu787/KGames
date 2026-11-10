// MinimalSceneTest's settings: each object's material, the file that it
// writes, and what the card says about a setting that's wrong.

#include "Base/Files.h"
#include "Base/Numbers.h"
#include "MinimalSceneTest/MinimalSceneTest.h"
#include "SimplePaint/SimplePaint.h"
#include "Testing/Testing.h"
#include "UserInterface/UserInterface.h"

// Where the repository is, for the program's settings file, and the frames
// that the tests save.
#include "Folders.h"

#include <array>
#include <cstdint>
#include <cstdlib>
#include <filesystem>
#include <optional>
#include <string>
#include <system_error>
#include <vector>

using graphics::AdapterChoice;
using graphics::Gpu;
using graphics::write_png;
using minimal_scene_test::moving_sphere::MovingSphere;
using minimal_scene_test::settings::SceneSettings;
using simple_paint::Material;
using simple_paint::Parameters;
using user_interface::Headless;
using user_interface::InfoCard;
using user_interface::Point;
using user_interface::Rect;
using user_interface::Settings;
using user_interface::Startup;
using user_interface::Theme;

TEST(the_defaults_are_a_constant_color_sphere_and_materials_that_simple_paint_accepts) {
    SceneSettings scene;
    // Equal Dark and Light Points paint a constant color, which is the base
    // color at tone 1 - Brightness.
    CHECK_EQ(scene.sphere.dark_point, scene.sphere.light_point);
    CHECK_EQ(scene.sphere.dark_point, 1.0 - scene.sphere.brightness);
    for (const Parameters* material : {&scene.sphere, &scene.ground, &scene.cube}) {
        CHECK(Material::compile(*material).has_value());
    }
}

TEST(the_file_it_writes_reads_back_as_the_defaults) {
    std::filesystem::path folder = std::filesystem::temp_directory_path() / "MinimalSceneSettings";
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
    std::filesystem::path path = std::filesystem::path(folders::Repository) / "Program" / "MinimalSceneTest" / "Settings.json";
    Settings settings = Settings::load(path);
    CHECK_EQ(settings.choice("WindowMode", "Maximized", {{"Maximized", "BorderlessFullscreen"}}), "Maximized");
    CHECK_EQ(SceneSettings::read(settings), SceneSettings{});
    CHECK(settings.problems().empty(), "{}", testing::describe(settings.problems()));
}

TEST(a_material_that_is_wrong_keeps_its_default) {
    Settings settings = Settings::from_text(R"({ "Ground": { "Shift": 0.5, "RotationDegrees": 90 }, "Cube": { "Shift": 2 } })");
    SceneSettings read = SceneSettings::read(settings);
    SceneSettings defaults;
    Parameters ground = defaults.ground;
    ground.shift = 0.5;
    ground.rotation_degrees = 90.0;
    CHECK_EQ(read.ground, ground);
    CHECK_EQ(read.cube, defaults.cube, "a material SimplePaint rejects keeps its default");
    CHECK_EQ(read.sphere, defaults.sphere);
    CHECK_EQ(std::vector<std::string>(settings.problems().begin(), settings.problems().end()),
             std::vector<std::string>{"Cube SimplePaint Shift must be finite and in [0, 0.9990234375], so it's the default."});
}

TEST(the_card_says_what_is_wrong_with_the_settings) {
    std::filesystem::path folder = std::filesystem::temp_directory_path() / "MinimalSceneNotices";
    CHECK(base::create_dir_all(folder).has_value());
    Startup startup = Startup::for_tests(R"({ "Sphere": { "Brightness": "Bright" } })", folder);
    Headless<MovingSphere> app = Headless<MovingSphere>::start(std::move(startup), MovingSphere::start);
    CHECK_EQ(app.program.settings, SceneSettings{});
    std::optional<std::string> label = app.label("Notice/0");
    CHECK(label.has_value());
    CHECK(label->starts_with("Warning: Settings.json Sphere "), "{}", *label);
    Rect card = app.target(InfoCard::Name);
    Rect notice = app.target("Notice/0");
    CHECK(card.contains(notice.origin()) && card.contains(Point(notice.right(), notice.bottom())));

    // It's a warning, in its colors.
    Gpu gpu(AdapterChoice::HighPerformance{});
    graphics::Image image = app.render(gpu);
    std::filesystem::path output = std::filesystem::path(folders::Repository) / "target" / "TestOutput";
    CHECK(base::create_dir_all(output).has_value());
    write_png(output / "MinimalSceneTestNotice.png", image);
    Theme theme = Theme::light();
    std::array<std::uint8_t, 4> fill = image.pixel(base::saturating_cast<std::uint32_t>(notice.right()) - 8, base::saturating_cast<std::uint32_t>(notice.bottom()) - 8);
    std::array<std::uint8_t, 4> expected = theme.caution_background.over(theme.acrylic).to_bytes();
    bool matches = true;
    for (size_t channel = 0; channel < 3; channel++) {
        matches = matches && std::abs(static_cast<std::int32_t>(fill[channel]) - static_cast<std::int32_t>(expected[channel])) <= 2;
    }
    CHECK(matches, "{} isn't {}", testing::describe(fill), testing::describe(expected));
}
