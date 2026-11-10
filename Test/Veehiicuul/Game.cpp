// Play through its frames, as the main loop runs them, with input that the
// tests make up, without a window, with Library/UserInterface's Headless:
// driving with the gamepad and the keys, braking, crashing, the vehicles,
// the scenes, the camera, the card, settings that name what isn't there,
// and a window without focus. And the 3D view, drawn offscreen with the GPU.

#include "Veehiicuul/Game.h"
#include "Base/Numbers.h"
#include "Base/Panic.h"
#include "Base/Text.h"
#include "Graphics/Graphics.h"
#include "Testing/Testing.h"
#include "UserInterface/UserInterface.h"
#include "Veehiicuul/Camera.h"
#include "Veehiicuul/Controls.h"
#include "Veehiicuul/Program.h"
#include "Veehiicuul/Track.h"
#include "Json/Json.h"

#include <array>
#include <cmath>
#include <cstdint>
#include <filesystem>
#include <optional>
#include <string>
#include <string_view>
#include <tuple>
#include <utility>
#include <vector>

namespace camera = veehiicuul::camera;
namespace tokens = user_interface::tokens;

using json::Value;
using user_interface::Gamepad;
using user_interface::GamepadButtons;
using user_interface::Headless;
using user_interface::Key;
using user_interface::Point;
using user_interface::Rect;
using user_interface::Startup;
using veehiicuul::camera::to_ground;
using veehiicuul::controls::Controls;
using veehiicuul::driving::Car;
using veehiicuul::game::Game;
using veehiicuul::game::Pause;
using veehiicuul::program::Veehiicuul;
using veehiicuul::track::Pose;

namespace {

// The game as it starts, with Settings.json's text, after its first frames.
Headless<Veehiicuul> start_with(std::string_view settings) {
    // Where tests that don't save would save.
    std::filesystem::path folder = std::filesystem::temp_directory_path() / "VeehiicuulGameTests";
    return Headless<Veehiicuul>::start(Startup::for_tests(settings, folder), Veehiicuul::start);
}

Headless<Veehiicuul> start() {
    Headless<Veehiicuul> app = start_with("{}");
    CHECK(app.ui.notices().empty(), "{}", testing::describe(app.ui.notices()));
    return app;
}

Pose start_of(const Headless<Veehiicuul>& app) {
    return app.program.game.current_scene().track.start;
}

Gamepad gamepad(Gamepad reading) {
    reading.connected = true;
    return reading;
}

Gamepad buttons(std::uint32_t pressed) {
    return gamepad(Gamepad{.buttons = GamepadButtons(pressed)});
}

double dot(std::array<double, 2> a, std::array<double, 2> b) {
    return a[0] * b[0] + a[1] * b[1];
}

std::array<double, 2> moved(const Headless<Veehiicuul>& app) {
    std::array<double, 2> from = start_of(app).position;
    std::array<double, 2> position = app.program.game.car.position;
    return {position[0] - from[0], position[1] - from[1]};
}

double speed(const Headless<Veehiicuul>& app) {
    return app.program.game.car.speed();
}

// Where to push the right stick, on the screen, for the car to go the way
// that it faces.
Point ahead_on_the_screen(const Headless<Veehiicuul>& app) {
    double heading = app.program.game.car.heading;
    std::array<double, 2> ahead{std::sin(heading), std::cos(heading)};
    return Point(static_cast<float>(dot(ahead, to_ground({1.0, 0.0}))), static_cast<float>(dot(ahead, to_ground({0.0, 1.0}))));
}

// How many of the image's pixels are near the color.
size_t count(const graphics::Image& image, std::array<std::uint8_t, 3> color) {
    auto close = [](std::uint8_t a, std::uint8_t b) {
        return base::abs_diff(a, b) <= 2;
    };
    size_t found = 0;
    for (size_t first = 0; first + 4 <= image.pixels.size(); first += 4) {
        const std::uint8_t* bgra = &image.pixels[first];
        if (close(bgra[2], color[0]) && close(bgra[1], color[1]) && close(bgra[0], color[2])) {
            found += 1;
        }
    }
    return found;
}

} // namespace

TEST(the_car_starts_on_the_start_line_at_rest) {
    Headless<Veehiicuul> app = start();
    const Game& game = app.program.game;
    const veehiicuul::scene::Scene& scene = game.current_scene();
    CHECK_EQ(std::pair(std::string_view(scene.name), std::string_view(game.current_vehicle().name)), std::pair(std::string_view("Ribeye"), std::string_view("Yellow")));
    CHECK_EQ(game.car.position, scene.track.start.position);
    CHECK_EQ(game.car.heading, scene.track.start.radians());
    CHECK_EQ(game.car.speed(), 0.0);
    CHECK(!game.paused());
    CHECK_EQ(game.camera_center(), scene.camera_center);
    Value state = app.state();
    CHECK(state.get("Mode") != nullptr);
    CHECK_EQ(*state.get("Mode"), Value("Play"));
}

// The right stick, pushed the way the car faces on the screen, speeds it up
// at its vehicle's forward rate, 140 for Yellow, without turning it, and the
// left trigger brakes it to a stop, at its reverse rate, 280.
TEST(the_stick_drives_the_car_and_the_trigger_stops_it) {
    Headless<Veehiicuul> app = start();
    double heading = app.program.game.car.heading;
    // 30 frames, half a second, of pushing: set_gamepad's own frame, then 29.
    Point push = ahead_on_the_screen(app);
    app.set_gamepad(gamepad(Gamepad{.right_stick = push}));
    app.frames(29);
    CHECK(std::abs(speed(app) - 70.0) < 1e-6, "{}", base::display(speed(app)));
    CHECK(std::abs(app.program.game.car.heading - heading) < 1e-9);
    app.set_gamepad(gamepad(Gamepad{
        .right_stick = Point(-1.0f, 0.0f),
        .left_trigger = 1.0f,
    }));
    app.frames(12);
    CHECK(speed(app) > 0.0);
    app.frames(2);
    CHECK_EQ(app.program.game.car.velocity, (std::array{0.0, 0.0}));
    // 0.5 seconds speeding up to 70, and 0.25 slowing down: about 26 units.
    double distance = std::hypot(moved(app)[0], moved(app)[1]);
    CHECK(25.0 <= distance && distance < 28.0, "{}", base::display(distance));
    CHECK_EQ(app.program.game.crashes, 0);
}

// The arrow keys push the car too, toward where they point on the screen,
// where the road goes up and to the left from the start, and Space brakes.
TEST(the_keys_drive_the_car_up_the_road_and_space_stops_it) {
    Headless<Veehiicuul> app = start();
    app.hold(Key::Up).hold(Key::Left).frames(16);
    CHECK(speed(app) > 10.0, "{}", base::display(speed(app)));
    std::array<double, 2> up_and_left = to_ground({-1.0, 1.0});
    CHECK(dot(moved(app), up_and_left) > 1.0, "{}", testing::describe(moved(app)));
    app.hold(Key::Space).frames(60);
    CHECK_EQ(speed(app), 0.0);
    CHECK_EQ(app.program.game.crashes, 0);
}

// Pushing the car into the road's edge puts it back at the start, at rest,
// and the push doesn't move it again for Pause seconds.
TEST(touching_an_edge_puts_the_car_back_at_the_start) {
    Headless<Veehiicuul> app = start();
    app.hold(Key::Right);
    size_t frames = 0;
    while (app.program.game.crashes == 0) {
        app.frame();
        frames += 1;
        CHECK(frames < 120, "the car didn't reach the edge");
    }
    Pose from = start_of(app);
    CHECK_EQ(std::pair(app.program.game.car.position, app.program.game.car.heading), std::pair(from.position, from.radians()));
    CHECK(app.program.game.paused());
    // Still pushing, it stays put until the pause is over.
    size_t paused_frames = base::saturating_cast<size_t>(Pause * 60.0);
    app.frames(paused_frames - 1);
    CHECK_EQ(app.program.game.car.position, from.position);
    app.frames(2);
    CHECK(!app.program.game.paused());
    CHECK_NE(app.program.game.car.position, from.position);
    Value state = app.state();
    CHECK(state.get("Crashes") != nullptr);
    CHECK_EQ(*state.get("Crashes"), Value(std::uint32_t{1}));
}

// Q and E, and the d-pad's left and right, go through the vehicles, around
// from the last to the first, and each puts the car back at the start. The
// d-pad's up and down go through the scenes.
TEST(the_vehicles_go_around_and_each_starts_over) {
    Headless<Veehiicuul> app = start();
    app.hold(Key::Up).frames(8).release_key(Key::Up);
    std::vector<std::string> names;
    for (Key key : {Key::E, Key::E, Key::E, Key::Q}) {
        app.key(key);
        names.push_back(app.program.game.current_vehicle().name);
    }
    CHECK_EQ(names, (std::vector<std::string>{"Cyan", "Magenta", "Green", "Magenta"}));
    CHECK_EQ(app.program.game.car.position, start_of(app).position);
    CHECK(app.program.game.paused());
    app.set_gamepad(buttons(GamepadButtons::DpadLeft)).frames(3);
    CHECK_EQ(app.program.game.current_vehicle().name, "Cyan", "holding the button switches once");
    // With one scene, the next scene is Ribeye again, from the start.
    app.set_gamepad(buttons(0)).set_gamepad(buttons(GamepadButtons::DpadUp));
    CHECK_EQ(app.program.game.current_scene().name, "Ribeye");
}

// C, or Menu, switches between the fixed camera and one that follows the
// car, each at its scene's height. The wheel over the view and the gamepad
// zoom, and Home, or Y, zooms back.
TEST(the_camera_follows_the_car_and_zooms) {
    Headless<Veehiicuul> app = start();
    CHECK_EQ(app.program.game.camera.height, 150.0);
    app.key(Key::C);
    CHECK(app.program.game.camera.follow);
    CHECK_EQ(app.program.game.camera.height, 45.0);
    CHECK_EQ(app.program.game.camera_center(), app.program.game.car.position);
    app.scroll_point(Point(1280.0f, 500.0f), 1.0f);
    CHECK(std::abs(app.program.game.camera.height - 36.0) < 1e-9);
    // The wheel over the app bar doesn't zoom.
    app.scroll_point(Point(1280.0f, 20.0f), 1.0f);
    CHECK(std::abs(app.program.game.camera.height - 36.0) < 1e-9);
    app.key(Key::Home);
    CHECK_EQ(app.program.game.camera.height, 45.0);
    // RB and the left stick up zoom in at ZoomSpeed units a second, for
    // half a second: set_gamepad's own frame, then 29.
    app.set_gamepad(gamepad(Gamepad{
        .left_stick = Point(0.0f, 1.0f),
        .buttons = GamepadButtons(GamepadButtons::RightShoulder),
    }));
    app.frames(29);
    CHECK(std::abs(app.program.game.camera.height - (45.0 - camera::ZoomSpeed * 0.5)) < 1e-9);
    app.frames(60);
    CHECK_EQ(app.program.game.camera.height, camera::Lowest);
    app.set_gamepad(buttons(GamepadButtons::Menu));
    CHECK(!app.program.game.camera.follow);
    CHECK_EQ(app.program.game.camera.height, 150.0);
}

// F1 hides the card and shows it again, and the card fits in the page's
// bottom left corner, under the notices, at either height.
TEST(f1_hides_the_card_which_fits_the_page) {
    Headless<Veehiicuul> app = start();
    app.key(Key::F1);
    CHECK(!app.program.game.show_card);
    CHECK(!app.has_target("Play/Card"));
    app.key(Key::F1);
    CHECK(app.program.game.show_card);
    Headless<Veehiicuul> noticed = start_with(R"({ "Vehicle": "Purple" })");
    for (bool fullscreen : {false, true}) {
        noticed.set_fullscreen(fullscreen);
        float height = fullscreen ? tokens::FullscreenHeight : tokens::MaximizedHeight;
        Rect card = noticed.target("Play/Card");
        CHECK(card.width < tokens::WindowWidth / 2.0f && card.height < height / 2.0f, "{}", testing::describe(card));
        CHECK_EQ(std::pair(card.x, card.bottom()), std::pair(tokens::PagePadding, height - tokens::StatusBarHeight - tokens::PagePadding));
        CHECK(card.y > noticed.target("Notice/0").bottom(), "{}", testing::describe(card));
    }
}

// Settings that name a scene or a vehicle that isn't there start the game
// in the first, with a notice under the app bar that says so.
TEST(settings_that_name_what_is_not_there_make_notices) {
    Headless<Veehiicuul> app = start_with(R"({ "Scene": "Sirloin", "Vehicle": "Purple", "Camera": "Above" })");
    const Game& game = app.program.game;
    CHECK_EQ(std::pair(std::string_view(game.current_scene().name), std::string_view(game.current_vehicle().name)),
             std::pair(std::string_view("Ribeye"), std::string_view("Green")));
    std::vector<std::string_view> messages;
    for (const user_interface::Notice& notice : app.ui.notices()) {
        messages.push_back(notice.message);
    }
    CHECK_EQ(messages, (std::vector<std::string_view>{
                           "Camera must be \"Fixed\" or \"Follow\", not \"Above\", so it's the default.",
                           "Scene is Sirloin, which isn't in Scenes, so the game starts in Ribeye.",
                           "Vehicle is Purple, which isn't in Vehicles.json, so the game starts with Green.",
                       }));
    CHECK(app.has_target("Notice/2"));
    CHECK(!app.program.game.camera.follow);
}

// Without the window's focus, the keys and the gamepad do nothing.
TEST(without_focus_nothing_moves_the_car) {
    Headless<Veehiicuul> app = start();
    app.focus(false).hold(Key::Up);
    app.set_gamepad(gamepad(Gamepad{
        .right_stick = Point(0.0f, 1.0f),
        .buttons = GamepadButtons(GamepadButtons::Menu),
    }));
    app.frames(30);
    CHECK_EQ(speed(app), 0.0);
    CHECK(!app.program.game.camera.follow);
}

// The 3D view fills the window under the bars and the card: the scene's
// background, the road, and the car at the start; and in the Vehicles tab,
// the vehicle on the theme's background.
TEST(the_view_draws_the_scene_under_the_interface) {
    graphics::Gpu gpu(graphics::AdapterChoice::HighPerformance{});
    Headless<Veehiicuul> app = start();
    graphics::Image image = app.render(gpu);
    size_t total = static_cast<size_t>(image.width * image.height);
    const veehiicuul::scene::Scene& scene = app.program.game.current_scene();
    size_t background = count(image, scene.background);
    size_t road = count(image, scene.track.road_color);
    CHECK(background > total / 5 && road > total / 20, "{} and {} of {}", background, road, total);
    // The app bar, under the PerfBar, is opaque over it.
    std::array<std::uint8_t, 4> bar = image.pixel(1000, base::saturating_cast<std::uint32_t>(tokens::PerfBarHeight + tokens::AppBarHeight / 2.0f));
    CHECK(bar[0] > 240 && bar[1] > 240 && bar[2] > 240, "{}", testing::describe(bar));
    // The Vehicles tab shows the yellow car, big, in the middle of the
    // window, on the theme's background.
    app.click("Mode/Vehicles");
    graphics::Image vehicles = app.render(gpu);
    size_t yellow = 0;
    for (size_t first = 0; first + 4 <= vehicles.pixels.size(); first += 4) {
        const std::uint8_t* bgra = &vehicles.pixels[first];
        if (bgra[2] > 180 && bgra[1] > 150 && bgra[0] < 80) {
            yellow += 1;
        }
    }
    CHECK(yellow > total / 50, "{} of {}", yellow, total);
    CHECK_NE(vehicles.pixel(vehicles.width / 2, vehicles.height / 2), vehicles.pixel(vehicles.width / 2, 100));
    CHECK(count(vehicles, {0xF3, 0xF3, 0xF3}) > total / 4);
}

// Ctrl's chords are the editors' shortcuts, like Ctrl+S, and don't push
// the car.
TEST(keys_with_ctrl_do_not_drive_the_car) {
    Headless<Veehiicuul> app = start();
    app.hold(Key::Control).hold(Key::S).frames(30);
    CHECK_EQ(speed(app), 0.0);
    CHECK_EQ(moved(app), (std::array{0.0, 0.0}));
    app.release_key(Key::Control).frames(16);
    CHECK(speed(app) > 0.0, "without Ctrl, S drives");
}

// A start that touches an edge counts one crash, and the car waits out the
// pause at the start, instead of a crash every frame.
TEST(a_crash_counts_once) {
    Headless<Veehiicuul> app = start();
    Pose from = start_of(app);
    std::array<double, 2> right = to_ground({1.0, 0.0});
    std::optional<std::array<double, 2>> touching;
    {
        Game& game = app.program.game;
        veehiicuul::collision::Footprint footprint = game.footprint();
        const veehiicuul::track::Shape& shape = game.shape();
        for (int step = 1; step < 400; step++) {
            std::array<double, 2> position{from.position[0] + right[0] * static_cast<double>(step) * 0.25, from.position[1] + right[1] * static_cast<double>(step) * 0.25};
            if (shape.edges.touch(footprint, position, from.radians())) {
                touching = position;
                break;
            }
        }
    }
    CHECK(touching.has_value(), "an edge to the right of the start");
    size_t scene = app.program.game.scene;
    app.program.game.document.scenes[scene].track.start.position = *touching;
    app.program.game.car.position = *touching;
    app.frames(10);
    CHECK_EQ(app.program.game.crashes, 1);
    CHECK(app.program.game.paused());
}

// A crash too is in fixed steps: the step that touches the edge crashes,
// and the pause runs on the drive's clock, so a run through a crash and out
// of its pause is the same at 60 frames a second as at 240.
TEST(a_crash_and_its_pause_are_the_same_at_any_frame_rate) {
    auto run = [](std::uint32_t rate) {
        Headless<Veehiicuul> app = start();
        // Pushing right reaches an edge in under two seconds; four seconds
        // crosses the crash, the pause, and the drive after it.
        Controls controls{.push = {1.0, 0.0}};
        app.program.game.update(controls, 0.0);
        for (std::uint32_t frame = 1; frame <= 4 * rate; frame++) {
            app.program.game.update(controls, static_cast<double>(frame) / static_cast<double>(rate));
        }
        return std::pair(app.program.game.car, app.program.game.crashes);
    };
    std::pair<Car, std::uint32_t> slow = run(60);
    std::pair<Car, std::uint32_t> fast = run(240);
    CHECK(slow.second >= 1, "the run crashed, {} times", slow.second);
    CHECK_EQ(slow.second, fast.second, "the same crashes");
    CHECK_EQ(slow.first, fast.first, "the same car after them");
}

// The car drives in fixed steps, so the same push for the same time puts it
// in the same place at 60 frames a second as at 240.
TEST(the_car_drives_the_same_at_any_frame_rate) {
    Headless<Veehiicuul> slow = start();
    Headless<Veehiicuul> fast = start();
    // Up the road, as the keys test drives, for half a second.
    Controls controls{.push = {-0.7, 0.7}};
    slow.program.game.update(controls, 0.0);
    fast.program.game.update(controls, 0.0);
    for (int frame = 1; frame <= 30; frame++) {
        slow.program.game.update(controls, static_cast<double>(frame) / 60.0);
    }
    for (int frame = 1; frame <= 120; frame++) {
        fast.program.game.update(controls, static_cast<double>(frame) / 240.0);
    }
    CHECK_EQ(slow.program.game.car, fast.program.game.car);
    CHECK_EQ(slow.program.game.crashes, 0);
    CHECK(slow.program.game.car.speed() > 1.0, "{}", base::display(slow.program.game.car.speed()));
}
