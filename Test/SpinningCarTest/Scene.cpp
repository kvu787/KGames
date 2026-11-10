// SpinningCarTest without a window: where the car is over time, the card out
// of its path, and frames drawn offscreen with the GPU. Test/Meshes tests the
// car mesh.

#include "Base/Files.h"
#include "Base/Numbers.h"
#include "SpinningCarTest/SpinningCarTest.h"
#include "Testing/Testing.h"
#include "UserInterface/UserInterface.h"

// Where the repository is, for the frames that the tests save.
#include "Folders.h"

#include <algorithm>
#include <array>
#include <cmath>
#include <cstdint>
#include <cstdlib>
#include <filesystem>
#include <format>
#include <limits>
#include <numbers>
#include <string_view>
#include <tuple>
#include <utility>

using graphics::AdapterChoice;
using graphics::Gpu;
using graphics::write_png;
using spinning_car_test::scene::Background;
using spinning_car_test::scene::car_on_screen;
using spinning_car_test::scene::car_placement;
using spinning_car_test::scene::SphereCenter;
using spinning_car_test::scene::SphereRadius;
using spinning_car_test::scene::to_screen;
using spinning_car_test::scene::ViewHeight;
using spinning_car_test::settings::SceneSettings;
using spinning_car_test::spinning_car::SpinningCar;
using user_interface::Headless;
using user_interface::InfoCard;
using user_interface::Point;
using user_interface::Rect;
using user_interface::Theme;
using user_interface::tokens::FullscreenHeight;
using user_interface::tokens::MaximizedHeight;
using user_interface::tokens::PagePadding;
using user_interface::tokens::PerfBarHeight;
using user_interface::tokens::WindowWidth;

namespace {

constexpr float FracPi2 = std::numbers::pi_v<float> / 2.0f;
constexpr float FracPi4 = std::numbers::pi_v<float> / 4.0f;

Headless<SpinningCar> car() {
    return Headless<SpinningCar>(SpinningCar(SceneSettings{}));
}

bool near_(std::pair<float, float> actual, std::pair<float, float> expected) {
    return std::abs(actual.first - expected.first) < 1e-5f && std::abs(actual.second - expected.second) < 1e-5f;
}

// How far apart the rectangles are: less than 0 if they overlap.
float gap(Rect a, Rect b) {
    return std::max(std::max(std::max(b.x - a.right(), a.x - b.right()), b.y - a.bottom()), a.y - b.bottom());
}

bool close(std::array<std::uint8_t, 4> a, std::array<std::uint8_t, 4> b) {
    for (size_t channel = 0; channel < 3; channel++) {
        if (std::abs(static_cast<std::int32_t>(a[channel]) - static_cast<std::int32_t>(b[channel])) > 2) {
            return false;
        }
    }
    return true;
}

// The pixel's red, green and blue.
std::array<std::uint8_t, 3> color(const graphics::Image& image, float x, float y) {
    std::array<std::uint8_t, 4> pixel = image.pixel(base::saturating_cast<std::uint32_t>(x), base::saturating_cast<std::uint32_t>(y));
    return {pixel[0], pixel[1], pixel[2]};
}

} // namespace

TEST(the_car_goes_from_the_middle_to_7_then_to_minus_7_and_back) {
    for (auto [seconds, expected] : {
             std::pair(0.0, std::pair(0.0f, 0.0f)),
             std::pair(0.5, std::pair(4.0f, FracPi4)),
             std::pair(0.875, std::pair(7.0f, 0.875f * FracPi2)),
             std::pair(1.75, std::pair(0.0f, 1.75f * FracPi2)),
             std::pair(2.625, std::pair(-7.0f, 2.625f * FracPi2)),
             // A cycle after 0.5 seconds, the car is back at 4, and after a whole
             // turn, it starts over.
             std::pair(4.0, std::pair(4.0f, 0.0f)),
         }) {
        std::pair<float, float> placement = car_placement(seconds, false);
        CHECK(near_(placement, expected), "at {} s, {} isn't {}", base::display(seconds), testing::describe(placement), testing::describe(expected));
    }
    // 3.5 seconds is a whole cycle of 28 units at 8 units a second.
    for (double seconds : {0.3, 1.1, 2.9}) {
        CHECK(std::abs(car_placement(seconds, false).first - car_placement(seconds + 3.5, false).first) < 1e-5f);
    }
}

TEST(a_stationary_car_stays_in_the_middle_facing_plus_x) {
    for (double seconds : {0.0, 1.0, 123.4}) {
        CHECK_EQ(car_placement(seconds, true), std::pair(0.0f, FracPi2));
    }
}

TEST(the_car_keeps_the_frames_clock) {
    Headless<SpinningCar> app = car();
    app.wait(0.5);
    CHECK_EQ(app.program.placement, car_placement(app.time(), false));
    json::Value state = app.state();
    CHECK(state.get("Stationary") != nullptr);
    CHECK_EQ(*state.get("Stationary"), json::Value(false));
    CHECK(state.get("TextShown") != nullptr);
    CHECK_EQ(*state.get("TextShown"), json::Value(true));
}

TEST(the_card_is_at_the_bottom_left_out_of_the_car_s_path_at_both_heights) {
    Headless<SpinningCar> app = car();
    for (bool fullscreen : {false, true}) {
        app.set_fullscreen(fullscreen);
        float height = fullscreen ? FullscreenHeight : MaximizedHeight;
        Rect card = app.target(InfoCard::Name);
        CHECK_EQ(std::tuple(card.x, card.bottom(), card.width), std::tuple(PagePadding, height - PagePadding, InfoCard::Width));
        // Every place and turn of the car comes in 28 seconds, 8 of its 3.5-second
        // trips and 7 of its 4-second turns.
        float closest = std::numeric_limits<float>::max();
        for (int frame = 0; frame < 28 * 60; frame++) {
            std::pair<float, float> placement = car_placement(static_cast<double>(frame) / 60.0, false);
            closest = std::min(closest, gap(car_on_screen(placement, WindowWidth, height), card));
        }
        closest = std::min(closest, gap(car_on_screen(car_placement(0.0, true), WindowWidth, height), card));
        CHECK(closest > 50.0f, "the car comes within {} pixels of the card at {} pixels high", base::display(closest), base::display(height));
        // Nor does the card cover the sphere.
        std::array<float, 2> at = to_screen(SphereCenter, WindowWidth, height);
        float radius = SphereRadius * height / ViewHeight;
        CHECK(gap(Rect(at[0] - radius, at[1] - radius, radius * 2.0f, radius * 2.0f), card) > 50.0f);
    }
}

TEST(a_frame_draws_the_scene_under_the_card_at_both_heights) {
    Gpu gpu(AdapterChoice::HighPerformance{});
    Headless<SpinningCar> app = car();
    std::filesystem::path folder = std::filesystem::path(folders::Repository) / "target" / "TestOutput";
    CHECK(base::create_dir_all(folder).has_value());
    std::array<std::uint8_t, 4> background{};
    for (size_t channel = 0; channel < 4; channel++) {
        background[channel] = base::saturating_cast<std::uint8_t>(std::round(Background[channel] * 255.0f));
    }
    // Back to maximized, after the depth buffer was made for fullscreen.
    for (bool fullscreen : {false, true, false}) {
        app.set_fullscreen(fullscreen);
        graphics::Image image = app.render(gpu);
        std::string_view name = fullscreen ? "Fullscreen" : "Maximized";
        write_png(folder / std::format("SpinningCarTest{}.png", name), image);
        float height = fullscreen ? FullscreenHeight : MaximizedHeight;
        CHECK_EQ(std::pair(image.width, image.height), std::pair(base::saturating_cast<std::uint32_t>(WindowWidth), base::saturating_cast<std::uint32_t>(height)));

        // The background, at the top right corner under the PerfBar, which
        // nothing covers.
        std::pair<std::uint32_t, std::uint32_t> corner(2550, base::saturating_cast<std::uint32_t>(PerfBarHeight) + 10);
        CHECK(close(image.pixel(corner.first, corner.second), background), "{}", testing::describe(image.pixel(corner.first, corner.second)));
        // The sphere, in its dark blue, under the light at its middle.
        std::array<float, 2> sphere = to_screen(SphereCenter, WindowWidth, height);
        float below = sphere[1] + SphereRadius * height / ViewHeight * 0.6f;
        std::array<std::uint8_t, 3> sphere_color = color(image, sphere[0], below);
        CHECK(sphere_color[2] > sphere_color[1] && sphere_color[1] > sphere_color[0], "{}", testing::describe(sphere_color));
        // The cube, in red, flat on top.
        std::array<float, 2> cube = to_screen({0.0f, 1.0f, -1.5f}, WindowWidth, height);
        std::array<std::uint8_t, 3> cube_color = color(image, cube[0], cube[1]);
        CHECK(cube_color[0] > 150 && cube_color[1] < 80 && cube_color[2] < 80, "{}", testing::describe(cube_color));
        // The car, in its blue, in front of the ground's green.
        Point middle = car_on_screen(app.program.placement, WindowWidth, height).center();
        std::array<std::uint8_t, 3> car_color = color(image, middle.x, middle.y);
        CHECK(car_color[2] > car_color[1] && car_color[2] > car_color[0], "{}", testing::describe(car_color));
        // The card, opaque, right of its title.
        Rect card = app.target(InfoCard::Name);
        std::array<std::uint8_t, 4> fill = image.pixel(base::saturating_cast<std::uint32_t>(card.right()) - 24, base::saturating_cast<std::uint32_t>(card.y) + 24);
        CHECK(close(fill, Theme::light().acrylic.to_bytes()), "{}", testing::describe(fill));
    }
}
