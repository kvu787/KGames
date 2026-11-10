// MinimalSceneTest without a window: the sphere's motion, the card out of its
// path, and frames drawn offscreen with the GPU.

#include "Base/Files.h"
#include "Base/Numbers.h"
#include "MinimalSceneTest/MinimalSceneTest.h"
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
#include <string_view>
#include <tuple>
#include <utility>

using graphics::AdapterChoice;
using graphics::Gpu;
using graphics::write_png;
using minimal_scene_test::moving_sphere::MovingSphere;
using minimal_scene_test::scene::Background;
using minimal_scene_test::scene::Motion;
using minimal_scene_test::scene::sphere_center;
using minimal_scene_test::scene::SphereRadius;
using minimal_scene_test::scene::to_screen;
using minimal_scene_test::scene::TravelDistance;
using minimal_scene_test::scene::ViewHeight;
using minimal_scene_test::settings::SceneSettings;
using user_interface::Headless;
using user_interface::InfoCard;
using user_interface::Rect;
using user_interface::Theme;
using user_interface::tokens::FullscreenHeight;
using user_interface::tokens::MaximizedHeight;
using user_interface::tokens::PagePadding;
using user_interface::tokens::PerfBarHeight;
using user_interface::tokens::WindowWidth;

namespace {

Headless<MovingSphere> sphere() {
    return Headless<MovingSphere>(MovingSphere(SceneSettings{}));
}

// How far the circle is outside the rectangle: less than 0 if they overlap.
float clearance(std::array<float, 2> center, float radius, Rect rect) {
    float dx = std::max(std::max(rect.x - center[0], center[0] - rect.right()), 0.0f);
    float dy = std::max(std::max(rect.y - center[1], center[1] - rect.bottom()), 0.0f);
    return std::sqrt(dx * dx + dy * dy) - radius;
}

bool close(std::array<std::uint8_t, 4> a, std::array<std::uint8_t, 4> b) {
    for (size_t channel = 0; channel < 3; channel++) {
        if (std::abs(static_cast<std::int32_t>(a[channel]) - static_cast<std::int32_t>(b[channel])) > 2) {
            return false;
        }
    }
    return true;
}

} // namespace

TEST(the_sphere_starts_in_the_middle_moving_at_8_units_a_second) {
    Motion motion;
    CHECK_EQ(std::pair(motion.position, motion.direction), std::pair(0.0f, 1.0f));
    motion.advance(0.5f);
    CHECK_EQ(motion.position, 4.0f);
}

TEST(the_sphere_bounces_back_from_7_by_as_far_as_it_went_past) {
    Motion motion;
    for (int step = 0; step < 7; step++) {
        motion.advance(0.125f);
    }
    CHECK_EQ(std::pair(motion.position, motion.direction), std::pair(7.0f, 1.0f));
    motion.advance(0.125f);
    CHECK_EQ(std::pair(motion.position, motion.direction), std::pair(6.0f, -1.0f));
    for (int step = 0; step < 14; step++) {
        motion.advance(0.125f);
    }
    CHECK_EQ(std::pair(motion.position, motion.direction), std::pair(-6.0f, 1.0f));
    motion.advance(0.125f);
    CHECK_EQ(motion.position, -5.0f);
}

TEST(any_time_keeps_the_sphere_on_its_path) {
    // From 5, 16 units take it to 7 and back to -7.
    Motion motion{
        .position = 5.0f,
        .direction = 1.0f,
    };
    motion.advance(2.0f);
    CHECK_EQ(std::pair(motion.position, motion.direction), std::pair(-7.0f, -1.0f));
    // Many trips back and forth.
    motion.advance(1000.0f);
    CHECK(std::abs(motion.position) <= TravelDistance, "{}", base::display(motion.position));
}

TEST(small_frames_keep_the_sphere_on_its_path) {
    Motion motion;
    float frame = 1.0f / 3000.0f;
    std::uint32_t turns = 0;
    for (int step = 0; step < 3000 * 10; step++) {
        float direction = motion.direction;
        motion.advance_frame(frame);
        turns += static_cast<std::uint32_t>(motion.direction != direction);
        CHECK(std::abs(motion.position) <= TravelDistance, "{}", base::display(motion.position));
    }
    // 80 units in 10 seconds, 14 units between turns, after 7 to the first.
    CHECK_EQ(turns, 6);
}

TEST(a_frame_counts_a_tenth_of_a_second_at_most) {
    // Like the first frame after the window was minimized for a minute.
    Motion motion;
    motion.advance_frame(60.0f);
    CHECK_EQ(std::pair(motion.position, motion.direction), std::pair(0.8f, 1.0f));
    motion.advance_frame(0.05f);
    CHECK(std::abs(motion.position - 1.2f) < 1e-6f, "{}", base::display(motion.position));
}

TEST(the_sphere_moves_with_the_frames_clock) {
    Headless<MovingSphere> app = sphere();
    app.wait(0.5);
    float expected = 8.0f * static_cast<float>(app.time());
    CHECK(std::abs(app.program.motion.position - expected) < 1e-4f, "{} isn't {}", base::display(app.program.motion.position), base::display(expected));
    json::Value state = app.state();
    CHECK(state.get("SphereDirection") != nullptr);
    CHECK_EQ(*state.get("SphereDirection"), json::Value(1.0f));
    CHECK(state.get("TextShown") != nullptr);
    CHECK_EQ(*state.get("TextShown"), json::Value(true));
}

TEST(the_card_is_at_the_bottom_left_out_of_the_sphere_s_path_at_both_heights) {
    Headless<MovingSphere> app = sphere();
    for (bool fullscreen : {false, true}) {
        app.set_fullscreen(fullscreen);
        float height = fullscreen ? FullscreenHeight : MaximizedHeight;
        Rect card = app.target(InfoCard::Name);
        CHECK_EQ(std::tuple(card.x, card.bottom(), card.width), std::tuple(PagePadding, height - PagePadding, InfoCard::Width));
        float radius = SphereRadius * height / ViewHeight;
        float closest = std::numeric_limits<float>::max();
        for (int step = -700; step <= 700; step++) {
            std::array<float, 2> center = to_screen(sphere_center(static_cast<float>(step) / 100.0f), WindowWidth, height);
            closest = std::min(closest, clearance(center, radius, card));
        }
        CHECK(closest > 50.0f, "the sphere comes within {} pixels of the card at {} pixels high", base::display(closest), base::display(height));
    }
}

TEST(a_frame_draws_the_scene_under_the_card_at_both_heights) {
    Gpu gpu(AdapterChoice::HighPerformance{});
    Headless<MovingSphere> app = sphere();
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
        write_png(folder / std::format("MinimalSceneTest{}.png", name), image);
        float height = fullscreen ? FullscreenHeight : MaximizedHeight;
        CHECK_EQ(std::pair(image.width, image.height), std::pair(base::saturating_cast<std::uint32_t>(WindowWidth), base::saturating_cast<std::uint32_t>(height)));

        // The background, at the top right corner under the PerfBar, which
        // nothing covers.
        std::pair<std::uint32_t, std::uint32_t> corner(2550, base::saturating_cast<std::uint32_t>(PerfBarHeight) + 10);
        CHECK(close(image.pixel(corner.first, corner.second), background), "{}", testing::describe(image.pixel(corner.first, corner.second)));
        // The sphere, in front of the cube, in its blue.
        std::array<float, 2> at = to_screen(sphere_center(app.program.motion.position), WindowWidth, height);
        std::array<std::uint8_t, 4> seen = image.pixel(base::saturating_cast<std::uint32_t>(at[0]), base::saturating_cast<std::uint32_t>(at[1]));
        std::uint8_t red = seen[0];
        std::uint8_t green = seen[1];
        std::uint8_t blue = seen[2];
        CHECK(blue > 200 && blue > green && green > red, "{}", testing::describe(std::array{red, green, blue}));
        // The card, opaque, right of its title.
        Rect card = app.target(InfoCard::Name);
        std::array<std::uint8_t, 4> fill = image.pixel(base::saturating_cast<std::uint32_t>(card.right()) - 24, base::saturating_cast<std::uint32_t>(card.y) + 24);
        CHECK(close(fill, Theme::light().acrylic.to_bytes()), "{}", testing::describe(fill));
    }
}
