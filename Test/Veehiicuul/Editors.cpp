// The editors through their frames, as the main loop runs them, with
// presses and drags that the tests make up, in their cards' and views' real
// layout, which Library/UserInterface's Headless lays out without a window
// or a GPU: the tabs, what fits in the cards, which controls keep keyboard
// focus, color fields, painting the terrain, panning and zooming the view,
// building the track, the vehicles' footprints, arranging the scene, undo,
// and saving.

#include "Base/Files.h"
#include "Base/Numbers.h"
#include "Base/Text.h"
#include "Testing/Testing.h"
#include "UserInterface/UserInterface.h"
#include "Veehiicuul/Angles.h"
#include "Veehiicuul/Curve.h"
#include "Veehiicuul/Program.h"
#include "Veehiicuul/SceneEditor.h"
#include "Veehiicuul/VehicleEditor.h"
#include "Json/Json.h"

#include <algorithm>
#include <array>
#include <cmath>
#include <cstdint>
#include <filesystem>
#include <format>
#include <optional>
#include <string>
#include <string_view>
#include <system_error>
#include <utility>
#include <vector>

namespace curve = veehiicuul::curve;
namespace program = veehiicuul::program;
namespace tokens = user_interface::tokens;

using json::Value;
using user_interface::Headless;
using user_interface::Key;
using user_interface::Modifiers;
using user_interface::Point;
using user_interface::Rect;
using user_interface::Startup;
using user_interface::Target;
using veehiicuul::collision::Footprint;
using veehiicuul::program::Veehiicuul;
using veehiicuul::scene_editor::Picked;
using veehiicuul::terrain::Cell;
using veehiicuul::terrain::Terrain;
using veehiicuul::track::Pose;
using veehiicuul::track::Track;
using veehiicuul::vehicle_editor::ShownHeading;

namespace {

// The window's size, maximized, which the views fill.
constexpr std::array<float, 2> Size{tokens::WindowWidth, tokens::MaximizedHeight};
// The middle of the window, which is in the editors' view, right of the
// card and under the app bar.
constexpr Point Middle(Size[0] / 2.0f, Size[1] / 2.0f);

std::filesystem::path save_folder(std::string_view test) {
    return std::filesystem::temp_directory_path() / "VeehiicuulEditorTests" / base::path(test);
}

// The game in Play, after its first frames, saving to a folder of the
// test's own.
Headless<Veehiicuul> start(std::string_view test) {
    std::filesystem::path saves = save_folder(test);
    std::error_code ignored;
    std::filesystem::remove_all(saves, ignored);
    return Headless<Veehiicuul>::start(Startup::for_tests("{}", saves), Veehiicuul::start);
}

// Where a point on the ground is on the screen, in the shared view.
Point screen(const Headless<Veehiicuul>& app, std::array<double, 2> point) {
    std::array<float, 2> at = app.program.viewport.eye.ground_to_screen(point, Size);
    return Point(at[0], at[1]);
}

std::array<double, 2> ground(const Headless<Veehiicuul>& app, Point point) {
    return app.program.viewport.eye.to_ground({point.x, point.y}, Size);
}

// Has the shared view look at a point on the ground, which goes to the
// middle of the window.
void look_at(Headless<Veehiicuul>& app, std::array<double, 2> point) {
    app.program.viewport.eye.center = point;
    app.frame();
}

bool near_(std::array<double, 2> a, std::array<double, 2> b) {
    return curve::distance(a, b) < 1e-3;
}

Point offset(Point point, float dx, float dy) {
    return Point(point.x + dx, point.y + dy);
}

// What has keyboard focus, as the interface's state says: a name, or null.
Value focused(const Headless<Veehiicuul>& app) {
    Value state = app.ui.state();
    const Value* member = state.get("Focused");
    CHECK(member != nullptr);
    return *member;
}

} // namespace

TEST(the_tabs_go_between_play_and_the_editors) {
    Headless<Veehiicuul> app = start("Tabs");
    CHECK_EQ(app.program.mode, program::Play);
    for (auto [label, mode] : {
             std::pair<std::string_view, size_t>{"Scene", program::Scene},
             std::pair<std::string_view, size_t>{"Terrain", program::Terrain},
             std::pair<std::string_view, size_t>{"Track", program::Track},
             std::pair<std::string_view, size_t>{"Vehicles", program::Vehicles},
             std::pair<std::string_view, size_t>{"Play", program::Play},
         }) {
        app.click(std::format("Mode/{}", label));
        CHECK_EQ(app.program.mode, mode, "{}", label);
    }
    // The tabs never take keyboard focus, so the arrow keys drive in Play
    // instead of going to them. Back in Play, the car starts over, and
    // waits a moment first.
    CHECK_EQ(focused(app), Value());
    app.hold(Key::Up).hold(Key::Left).frames(26);
    CHECK_EQ(app.program.mode, program::Play);
    CHECK(app.program.game.car.speed() > 0.0);
}

// Every editor's controls are in its cards, above the status bar, at either
// height, even the scene editor's with a decoration picked, which shows its
// model's paint.
TEST(the_editors_controls_fit_in_their_cards) {
    for (bool fullscreen : {false, true}) {
        Headless<Veehiicuul> app = start("Fit");
        app.set_fullscreen(fullscreen);
        float height = fullscreen ? tokens::FullscreenHeight : tokens::MaximizedHeight;
        float bottom = height - tokens::StatusBarHeight - tokens::PagePadding - tokens::CardPadding;
        for (std::string_view label : {"Scene", "Terrain", "Track", "Vehicles"}) {
            app.click(std::format("Mode/{}", label));
            if (label == "Scene") {
                app.click("Scene/Objects/0");
                CHECK_EQ(app.program.scene_editor.picked, std::optional<Picked>(Picked::Decoration{0}));
            }
            for (const Target& target : app.ui.targets()) {
                if (!target.listed) {
                    continue;
                }
                Rect area = target.rect;
                if (target.name.ends_with("/View") || area.y < tokens::PerfBarHeight + tokens::AppBarHeight) {
                    continue;
                }
                CHECK(area.x >= tokens::PagePadding + tokens::CardPadding && area.right() <= tokens::WindowWidth - tokens::PagePadding - tokens::CardPadding &&
                          area.bottom() <= bottom,
                      "{}: {} is at {}", label, target.name, testing::describe(area));
            }
        }
    }
}

// Only text fields keep keyboard focus, so that the arrows never go to a
// control, like the tools' segments or the list. A color field's color
// changes with Enter, and a press in the view takes its focus.
TEST(only_text_fields_keep_keyboard_focus) {
    Headless<Veehiicuul> app = start("Focus");
    app.click("Mode/Terrain");
    app.click("Terrain/Tool/Fill");
    CHECK_EQ(focused(app), Value());
    app.key(Key::Right).key(Key::Left);
    CHECK_EQ(app.program.terrain_editor.tool, 2);
    app.click("Mode/Scene").click("Scene/Objects/1");
    CHECK_EQ(focused(app), Value());
    app.key(Key::Down);
    CHECK_EQ(app.program.scene_editor.picked, std::optional<Picked>(Picked::Parked{0}));

    app.click("Mode/Terrain").click("Terrain/Color");
    CHECK_EQ(focused(app), Value("Terrain/Color"));
    app.key_with(Modifiers::Control, Key::A).type_text("#123456");
    CHECK_NE(app.program.game.current_scene().terrain.palette[0], (veehiicuul::files::Color{0x12, 0x34, 0x56}), "not before Enter");
    app.key(Key::Enter);
    CHECK_EQ(app.program.game.current_scene().terrain.palette[0], (veehiicuul::files::Color{0x12, 0x34, 0x56}));
    app.press_point(Middle);
    CHECK_EQ(focused(app), Value());
}

// A press paints the hexels under the brush, a drag paints along it, Erase
// takes the paint off, and undo and redo go back and forth.
TEST(the_terrain_editor_paints_and_erases_hexels) {
    Headless<Veehiicuul> app = start("Terrain");
    app.click("Mode/Terrain");
    look_at(app, {150.0, 100.0});
    Cell cell = app.program.game.current_scene().terrain.cell_at(ground(app, Middle));
    size_t before = app.program.game.current_scene().terrain.cells.size();
    app.click_point(Middle);
    const Terrain& terrain = app.program.game.current_scene().terrain;
    CHECK_EQ(terrain.get(cell), std::optional<std::uint8_t>(std::uint8_t{0}));
    size_t painted = terrain.cells.size();
    CHECK(painted > before && painted <= before + 7, "{} then {}", before, painted);
    // The fourth color, from its swatch.
    app.click("Palette/3");
    CHECK_EQ(app.program.terrain_editor.color, 3);
    app.drag(Point(1400.0f, 900.0f), Point(1700.0f, 900.0f), 6);
    const Terrain& dragged = app.program.game.current_scene().terrain;
    Cell along = dragged.cell_at(ground(app, Point(1550.0f, 900.0f)));
    CHECK_EQ(dragged.get(along), std::optional<std::uint8_t>(std::uint8_t{3}));
    app.click("Terrain/Tool/Erase");
    CHECK_EQ(app.program.terrain_editor.tool, 1);
    app.click_point(Middle);
    CHECK_EQ(app.program.game.current_scene().terrain.get(cell), std::nullopt);
    app.key_with(Modifiers::Control, Key::Z);
    CHECK_EQ(app.program.game.current_scene().terrain.get(cell), std::optional<std::uint8_t>(std::uint8_t{0}), "undo brings the paint back");
    app.key_with(Modifiers::Control, Key::Y);
    CHECK_EQ(app.program.game.current_scene().terrain.get(cell), std::nullopt, "redo takes it off again");
    CHECK(app.program.unsaved());
    // The buttons undo and redo too, and are off when there's nothing to do.
    CHECK(app.is_enabled("Undo") && !app.is_enabled("Redo"));
    app.click("Undo");
    CHECK_EQ(app.program.game.current_scene().terrain.get(cell), std::optional<std::uint8_t>(std::uint8_t{0}));
    app.click("Redo");
    CHECK_EQ(app.program.game.current_scene().terrain.get(cell), std::nullopt);
}

// Space and a drag pan the view, without painting, and the wheel zooms it
// around the pointer, which keeps the ground under it where it is.
TEST(the_view_pans_with_space_and_zooms_with_the_wheel) {
    Headless<Veehiicuul> app = start("View");
    app.click("Mode/Terrain");
    size_t painted = app.program.game.current_scene().terrain.cells.size();
    std::array<double, 2> center = app.program.viewport.eye.center;
    std::array<double, 2> grabbed = ground(app, Middle);
    app.hold(Key::Space).drag(Middle, offset(Middle, 200.0f, 100.0f), 4).release_key(Key::Space);
    CHECK_EQ(app.program.game.current_scene().terrain.cells.size(), painted);
    CHECK_NE(app.program.viewport.eye.center, center);
    CHECK(near_(ground(app, offset(Middle, 200.0f, 100.0f)), grabbed));
    double height = app.program.viewport.eye.height;
    Point at(1800.0f, 400.0f);
    std::array<double, 2> under = ground(app, at);
    app.scroll_point(at, 1.0f);
    CHECK(std::abs(app.program.viewport.eye.height - height / 1.25) < 1e-9);
    CHECK(near_(ground(app, at), under));
    CHECK(!app.program.unsaved());
    // The wheel over the card doesn't zoom.
    app.scroll_point(Point(200.0f, 600.0f), 1.0f);
    CHECK(std::abs(app.program.viewport.eye.height - height / 1.25) < 1e-9);
}

// Moving the road's first control point, adding one and deleting it, adding
// an island, and moving the start.
TEST(the_track_builder_moves_adds_and_deletes_points) {
    Headless<Veehiicuul> app = start("Track");
    app.click("Mode/Track");
    std::array<double, 2> first = app.program.game.current_scene().track.road[0];
    look_at(app, first);
    Point from = screen(app, first);
    Point to = offset(from, 40.0f, 30.0f);
    app.drag(from, to, 3);
    std::array<double, 2> moved = app.program.game.current_scene().track.road[0];
    CHECK(near_(moved, ground(app, to)), "{}", testing::describe(moved));
    CHECK_EQ(app.program.track_editor.curve, 0);

    app.click("Track/Tool/Add points");
    std::vector<std::array<double, 2>> road = curve::sample(app.program.game.current_scene().track.road);
    look_at(app, road[100]);
    Point on_the_road = screen(app, road[100]);
    app.click_point(on_the_road);
    const Track& track = app.program.game.current_scene().track;
    CHECK_EQ(track.road.size(), 65);
    CHECK(std::ranges::any_of(track.road, [&](std::array<double, 2> point) { return near_(point, ground(app, on_the_road)); }));

    app.click("Track/Tool/Delete points");
    app.click_point(on_the_road);
    CHECK_EQ(app.program.game.current_scene().track.road.size(), 64);

    app.click("Track/AddIsland");
    CHECK_EQ(app.program.game.current_scene().track.islands.size(), 3);
    CHECK_EQ(app.program.track_editor.curve, 3);
    app.click("Track/DeleteIsland");
    CHECK_EQ(app.program.game.current_scene().track.islands.size(), 2);

    app.click("Track/Tool/Move points");
    Pose start_pose = app.program.game.current_scene().track.start;
    look_at(app, start_pose.position);
    Point start_from = screen(app, start_pose.position);
    Point start_to = offset(start_from, -20.0f, 0.0f);
    app.drag(start_from, start_to, 2);
    Pose moved_pose = app.program.game.current_scene().track.start;
    CHECK(near_(moved_pose.position, ground(app, start_to)));
    CHECK_EQ(moved_pose.heading, start_pose.heading);
}

// A track with a problem says so, in the Track tab's card, and in Play's,
// which still fits the page.
TEST(a_track_with_a_problem_says_so) {
    Headless<Veehiicuul> app = start("Problem");
    app.program.game.document.scenes[0].track.start.position = {1000.0, 1000.0};
    app.frame();
    std::vector<std::string> problems = app.program.game.last_shape().problems;
    CHECK_EQ(problems.size(), 1, "{}", testing::describe(problems));
    CHECK(app.has_target("Play/Problem"));
    Rect card = app.target("Play/Card");
    CHECK(card.y > tokens::PerfBarHeight + tokens::AppBarHeight && card.bottom() == tokens::MaximizedHeight - tokens::StatusBarHeight - tokens::PagePadding, "{}",
          testing::describe(card));
    app.click("Mode/Track");
    CHECK(app.has_target("Track/Problem/0") && !app.has_target("Track/Problem/1"));
    Value state = app.state();
    const Value* editor = state.get("TrackEditor");
    const Value* listed = editor != nullptr ? editor->get("Problems") : nullptr;
    CHECK(listed != nullptr);
    CHECK_EQ(*listed, Value(problems));
}

// A side of the footprint follows a drag from it, Fit makes it the model's
// rectangle, and a new vehicle is a copy, with a name of its own.
TEST(the_footprint_creator_moves_a_side_and_fits_the_model) {
    Headless<Veehiicuul> app = start("Footprint");
    app.click("Mode/Vehicles");
    double front = app.program.game.current_vehicle().footprint.max_[1];
    double heading = veehiicuul::to_radians(ShownHeading);
    std::array<double, 2> ahead{std::sin(heading), std::cos(heading)};
    veehiicuul::camera::Eye eye = app.program.vehicle_editor.viewport.eye;
    std::array<float, 2> at = eye.ground_to_screen({ahead[0] * front, ahead[1] * front}, Size);
    std::array<float, 2> farther = eye.ground_to_screen({ahead[0] * (front + 1.0), ahead[1] * (front + 1.0)}, Size);
    app.drag(Point(at[0], at[1]), Point(farther[0], farther[1]), 3);
    double moved = app.program.game.current_vehicle().footprint.max_[1];
    CHECK(std::abs(moved - (front + 1.0)) < 1e-3, "{} then {}", base::display(front), base::display(moved));
    app.click("Footprint/Fit");
    Footprint footprint = app.program.game.current_vehicle().footprint;
    CHECK(std::abs(footprint.max_[1] - 3.157522) < 1e-5 && std::abs(footprint.min_[0] + 1.5) < 1e-5, "{} {}", testing::describe(footprint.min_), testing::describe(footprint.max_));

    app.click("Vehicles/Add");
    CHECK_EQ(app.program.game.document.vehicles.list.size(), 7);
    CHECK_EQ(app.program.game.current_vehicle().name, "Vehicle 7");
    // Renaming Green renames it where it's parked too.
    app.click("Vehicles/List/0");
    CHECK_EQ(app.program.game.current_vehicle().name, "Green");
    Rect field = app.target("Vehicles/Name");
    app.click_point(Point(field.right() - 4.0f, field.center().y));
    app.type_text("ish");
    CHECK_EQ(app.program.game.current_vehicle().name, "Greenish");
    CHECK_EQ(app.program.game.current_scene().parked[0].vehicle, "Greenish");
    // A parked vehicle can't be deleted.
    app.click("Vehicles/Delete");
    CHECK_EQ(app.program.game.document.vehicles.list.size(), 7);
    CHECK_EQ(app.program.vehicle_editor.message, "Greenish is parked in Ribeye, so it can't go.");
    CHECK(app.has_target("Vehicles/Message"));
}

// A press on the cone picks it, and a drag moves it. A new decoration goes
// where the view looks, and Delete takes the picked one away.
TEST(the_scene_editor_picks_and_moves_decorations) {
    Headless<Veehiicuul> app = start("Scene");
    app.click("Mode/Scene");
    std::array<double, 2> cone = app.program.game.current_scene().decorations[0].pose.position;
    look_at(app, cone);
    Point from = screen(app, cone);
    Point to = offset(from, 50.0f, 20.0f);
    app.drag(from, to, 3);
    CHECK_EQ(app.program.scene_editor.picked, std::optional<Picked>(Picked::Decoration{0}));
    std::array<double, 2> moved = app.program.game.current_scene().decorations[0].pose.position;
    std::array<double, 2> dropped = ground(app, to);
    std::array<double, 2> grabbed = ground(app, from);
    CHECK(near_(moved, {cone[0] + dropped[0] - grabbed[0], cone[1] + dropped[1] - grabbed[1]}), "{}", testing::describe(moved));
    // A press on nothing picks nothing.
    app.click_point(Point(2400.0f, 1300.0f));
    CHECK_EQ(app.program.scene_editor.picked, std::optional<Picked>());
    app.click("Scene/AddDecoration");
    CHECK_EQ(app.program.game.current_scene().decorations.size(), 2);
    CHECK_EQ(app.program.scene_editor.picked, std::optional<Picked>(Picked::Decoration{1}));
    app.click("Scene/Delete");
    CHECK_EQ(app.program.game.current_scene().decorations.size(), 1);
    app.click("Scene/Park");
    CHECK_EQ(app.program.game.current_scene().parked.size(), 7);
    CHECK_EQ(app.program.scene_editor.picked, std::optional<Picked>(Picked::Parked{6}));
}

// Save writes the files that changed, and only those, in the save folder.
TEST(save_writes_only_what_changed) {
    Headless<Veehiicuul> app = start("Save");
    app.key_with(Modifiers::Control, Key::S);
    CHECK_EQ(app.program.status, "Nothing to save");
    app.program.game.document.vehicles.list[0].acceleration.forward = 12.0;
    app.click("NewScene");
    CHECK_EQ(app.program.game.current_scene().name, "Scene 2");
    CHECK(app.program.unsaved());
    app.click("Save");
    CHECK_EQ(app.program.status, "Saved Scenes/Scene 2.json, Vehicles.json");
    CHECK(!app.program.unsaved());
    std::filesystem::path saves = save_folder("Save");
    CHECK(std::filesystem::is_regular_file(saves / "Scenes/Scene 2.json"));
    CHECK(!std::filesystem::exists(saves / "Scenes/Ribeye.json"));
    std::string vehicles = base::read_to_string(saves / "Vehicles.json").value();
    CHECK(vehicles.contains("\"Forward\": 12.0"), "{}", vehicles);
}

// An island added and then undone can't be deleted again: the frame's last
// message clamps the chosen curve, so the button deletes a real island or
// nothing.
TEST(an_undone_island_cannot_be_deleted_again) {
    Headless<Veehiicuul> app = start("UndoIsland");
    app.click("Mode/Track");
    size_t islands = app.program.game.current_scene().track.islands.size();
    app.click("Track/AddIsland");
    CHECK_EQ(app.program.track_editor.curve, islands + 1);
    app.key_with(Modifiers::Control, Key::Z);
    CHECK_EQ(app.program.game.current_scene().track.islands.size(), islands);
    CHECK(app.program.track_editor.curve <= islands, "the choice was clamped with the undo");
    app.click("Track/DeleteIsland");
    CHECK_EQ(app.program.game.current_scene().track.islands.size(), base::saturating_sub(islands, size_t{1}), "deleted a real island, not past the end");
}

// Saving after undoing a new scene removes its file, and the document then
// counts as saved, instead of "Unsaved changes" forever.
TEST(saving_after_undoing_a_new_scene_removes_its_file) {
    Headless<Veehiicuul> app = start("RemoveScene");
    app.click("NewScene");
    app.click("Save");
    std::filesystem::path saves = save_folder("RemoveScene");
    CHECK(std::filesystem::is_regular_file(saves / "Scenes/Scene 2.json"));
    app.key_with(Modifiers::Control, Key::Z);
    CHECK(app.program.unsaved());
    app.click("Save");
    CHECK_EQ(app.program.status, "Removed Scenes/Scene 2.json");
    CHECK(!std::filesystem::exists(saves / "Scenes/Scene 2.json"));
    CHECK(!app.program.unsaved(), "the save reconciled the deletion");
}

// Undo during a held drag takes the picked decoration away, and the pick
// goes with it, instead of indexing past the decorations that are left.
TEST(undo_during_a_drag_drops_the_pick) {
    Headless<Veehiicuul> app = start("UndoDrag");
    app.click("Mode/Scene");
    app.click("Scene/AddDecoration");
    CHECK_EQ(app.program.game.current_scene().decorations.size(), 2);
    std::array<double, 2> added = app.program.game.current_scene().decorations[1].pose.position;
    look_at(app, added);
    Point at = screen(app, added);
    app.press_point(at);
    CHECK_EQ(app.program.scene_editor.picked, std::optional<Picked>(Picked::Decoration{1}));
    app.key_with(Modifiers::Control, Key::Z);
    CHECK_EQ(app.program.game.current_scene().decorations.size(), 1);
    CHECK_EQ(app.program.scene_editor.picked, std::optional<Picked>());
    app.move_point(offset(at, 30.0f, 0.0f)).release();
    CHECK_EQ(app.program.game.current_scene().decorations.size(), 1);
}
