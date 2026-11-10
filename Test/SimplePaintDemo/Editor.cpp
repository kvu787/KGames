// What input does to SimplePaintDemo's editor, through the cards' real
// layout, which user_interface::Headless builds without a window or a GPU,
// and where the cards go at both supported window heights.

#include "SimplePaintDemo/Editor.h"
#include "Base/Files.h"
#include "Base/Numbers.h"
#include "Base/Windows.h"
#include "PaintSettings/PaintSettings.h"
#include "SimplePaint/Material.h"
#include "SimplePaintDemo/Cards.h"
#include "SimplePaintDemo/Demo.h"
#include "SimplePaintDemo/Preview.h"
#include "Testing/Testing.h"
#include "UserInterface/Application/Program.h"
#include "UserInterface/Application/Settings.h"
#include "UserInterface/Core/Interaction.h"
#include "UserInterface/Design/Tokens.h"
#include "UserInterface/Drawing/Geometry.h"
#include "UserInterface/Input/Devices.h"
#include "UserInterface/Testing/Headless.h"
#include "UserInterface/Widgets/InfoBar.h"
#include "UserInterface/Widgets/Slider.h"

#include <algorithm>
#include <array>
#include <cstdint>
#include <d3d12.h>
#include <filesystem>
#include <format>
#include <optional>
#include <string>
#include <string_view>
#include <system_error>
#include <tuple>
#include <utility>
#include <vector>

using simple_paint::Material;
using simple_paint::Parameters;
using simple_paint_demo::cards::PreviewWidth;
using simple_paint_demo::demo::Demo;
using simple_paint_demo::editor::Command;
using simple_paint_demo::editor::Setting;
using simple_paint_demo::editor::Settings;
using simple_paint_demo::preview::copy_place;
using simple_paint_demo::preview::CopyPlace;
using simple_paint_demo::preview::ImageHeight;
using simple_paint_demo::preview::ImageWidth;
using user_interface::Headless;
using user_interface::Key;
using user_interface::Modifiers;
using user_interface::Notice;
using user_interface::Point;
using user_interface::Rect;
using user_interface::Severity;
using user_interface::slider_rail;
using user_interface::Startup;
using user_interface::Target;
using user_interface::tokens::FullscreenHeight;
using user_interface::tokens::MaximizedHeight;
using user_interface::tokens::WindowWidth;
namespace editor = simple_paint_demo::editor;

// A D3D12_BOX has no == of its own, and the checks look for one where the
// type is, which is here, outside every namespace.
static bool operator==(const D3D12_BOX& a, const D3D12_BOX& b) {
    return a.left == b.left && a.top == b.top && a.front == b.front && a.right == b.right && a.bottom == b.bottom && a.back == b.back;
}

// How a failed check shows a box.
static std::string describe_value(const D3D12_BOX& box) {
    return std::format("D3D12_BOX {{ left: {}, top: {}, front: {}, right: {}, bottom: {}, back: {} }}", box.left, box.top, box.front, box.right, box.bottom, box.back);
}

namespace {

Headless<Demo> demo(const Parameters& parameters, std::filesystem::path paint_path) {
    return Headless(Demo(parameters, std::move(paint_path)));
}

std::filesystem::path nowhere() {
    return std::filesystem::path(R"(C:\Nowhere\Paint.json)");
}

// A fresh folder for one test.
std::filesystem::path folder(std::string_view test) {
    std::filesystem::path made = std::filesystem::temp_directory_path() / "KGamesSimplePaintDemoTests" / test;
    std::error_code ignored;
    std::filesystem::remove_all(made, ignored);
    base::create_dir_all(made).value();
    return made;
}

// The point at `fraction` along a slider's rail.
Point along(const Headless<Demo>& app, std::string_view name, float fraction) {
    Rect area = app.target(name);
    auto [left, right] = slider_rail(area);
    return Point(left + (right - left) * fraction, area.center().y);
}

double value(const Headless<Demo>& app, std::string_view name) {
    const Setting& setting = *std::ranges::find_if(Settings, [&](const Setting& candidate) { return candidate.name == name; });
    return setting.value(app.program.editor.parameters);
}

std::string_view selected(const Headless<Demo>& app) {
    return Settings[app.program.editor.selected].name;
}

// The names of the frame's targets that scripts and tests see.
std::vector<std::string> names(const Headless<Demo>& app) {
    std::vector<std::string> listed;
    for (const Target& target : app.ui.targets()) {
        if (target.listed) {
            listed.push_back(target.name);
        }
    }
    return listed;
}

} // namespace

TEST(a_press_on_a_slider_sets_it_and_drags_it) {
    Headless<Demo> app = demo(Parameters{}, nowhere());
    Point half = along(app, "R", 0.5f);
    app.press_point(half);
    CHECK_EQ(value(app, "R"), 0.5);
    CHECK_EQ(app.label("R"), std::optional<std::string>("0.500"));
    CHECK_EQ(std::pair(selected(app), app.program.editor.dragging), std::pair(std::string_view("R"), std::optional<size_t>(0)));
    // Dragging follows the pointer along the rail, and holds the slider even
    // when the pointer leaves the row.
    Point quarter = along(app, "R", 0.25f);
    app.move_point(Point(quarter.x, 900.0f));
    CHECK_EQ(value(app, "R"), 0.25);
    app.release();
    CHECK_EQ(app.program.editor.dragging, std::nullopt);
    // After the release, moving the pointer changes nothing.
    Point three_quarters = along(app, "R", 0.75f);
    app.move_point(three_quarters);
    CHECK_EQ(value(app, "R"), 0.25);
}

TEST(sliders_stop_at_the_last_step_simple_paint_accepts) {
    Headless<Demo> app = demo(Parameters{}, nowhere());
    for (auto [name, low, high] : std::array<std::tuple<std::string_view, double, double>, 6>{{
             {"R", 0.001, 0.999},
             {"Brightness", 0.001, 0.999},
             {"Shift", 0.0, 0.999},
             {"Rotation", 0.0, 359.9},
             {"Dark Point", 0.0, 0.999},
             {"Light Point", 0.001, 1.0},
         }}) {
        for (auto [x, expected] : std::array<std::pair<float, double>, 2>{{{-500.0f, low}, {5000.0f, high}}}) {
            // Pressed on the rail, then dragged far past its end.
            Point half = along(app, name, 0.5f);
            app.press_point(half).move_point(Point(x, 0.0f)).release();
            CHECK_EQ(value(app, name), expected, "{}", name);
        }
    }
    (void)Material::compile(app.program.editor.parameters).value();
}

TEST(a_press_on_a_setting_s_row_selects_it) {
    Headless<Demo> app = demo(Parameters{}, nowhere());
    Rect row = app.target("Setting/Brightness");
    app.click_point(Point(row.x + 40.0f, row.center().y));
    CHECK_EQ(selected(app), "Brightness");
    CHECK_EQ(app.program.editor.parameters, Parameters{}, "a press on the name changes nothing");
}

TEST(losing_focus_ends_a_drag) {
    Headless<Demo> app = demo(Parameters{}, nowhere());
    Point half = along(app, "Shift", 0.5f);
    app.press_point(half);
    CHECK_EQ(app.program.editor.dragging, std::optional<size_t>(4));
    app.focus(false);
    CHECK_EQ(app.program.editor.dragging, std::nullopt);
    app.focus(true);
    Point far_ = along(app, "Shift", 0.9f);
    app.move_point(far_);
    CHECK_EQ(app.program.editor.parameters.shift, 0.5);
}

TEST(the_wheel_moves_a_slider_by_its_smallest_step) {
    Headless<Demo> app = demo(Parameters{}, nowhere());
    Point rotation = along(app, "Rotation", 0.0f);
    app.scroll_point(rotation, 3.0f);
    CHECK_EQ(app.program.editor.parameters.rotation_degrees, 0.3);
    CHECK_EQ(selected(app), "Rotation");
    app.scroll_point(rotation, -5.0f);
    CHECK_EQ(app.program.editor.parameters.rotation_degrees, 0.0);
    // Off the sliders, the wheel does nothing.
    app.scroll_point(Point(100.0f, 100.0f), 1.0f);
    Point label = app.target("Setting/Shift").origin() + Point(20.0f, 10.0f);
    app.scroll_point(label, 1.0f);
    CHECK_EQ(app.program.editor.parameters, Parameters{});
}

TEST(parts_of_a_notch_add_up_over_the_same_slider) {
    // Like a touchpad, which turns the wheel by a quarter of a notch at a time.
    Headless<Demo> app = demo(Parameters{}, nowhere());
    Point rotation = along(app, "Rotation", 0.0f);
    for (int turn = 0; turn < 3; turn++) {
        app.scroll_point(rotation, 0.25f);
    }
    CHECK_EQ(app.program.editor.parameters.rotation_degrees, 0.0);
    app.scroll_point(rotation, 0.25f);
    CHECK_EQ(app.program.editor.parameters.rotation_degrees, 0.1);
    for (int turn = 0; turn < 6; turn++) {
        app.scroll_point(rotation, 0.5f);
    }
    CHECK_EQ(app.program.editor.parameters.rotation_degrees, 0.4);
    // What's left over one slider doesn't carry over to another.
    app.scroll_point(rotation, 0.5f);
    CHECK_EQ(app.program.editor.parameters.rotation_degrees, 0.4);
    Point shift = along(app, "Shift", 0.0f);
    app.scroll_point(shift, 0.5f).scroll_point(shift, 0.25f);
    CHECK_EQ(app.program.editor.parameters.shift, 0.0);
}

TEST(arrow_keys_select_and_move_a_setting) {
    Headless<Demo> app = demo(Parameters{}, nowhere());
    app.key(Key::Up);
    CHECK_EQ(selected(app), "R", "Up stops at the first setting");
    app.key(Key::Right);
    // Values on a step are the double nearest their decimal, so they save as
    // written.
    CHECK_EQ(app.program.editor.parameters.base_color_srgb[0], 0.108);
    app.key_with(Modifiers::Shift, Key::Right);
    CHECK_EQ(app.program.editor.parameters.base_color_srgb[0], 0.118);
    app.key(Key::Left);
    CHECK_EQ(app.program.editor.parameters.base_color_srgb[0], 0.117);
    for (int press = 0; press < 20; press++) {
        app.key(Key::Down);
    }
    CHECK_EQ(selected(app), "Light Point", "Down stops at the last setting");
    app.key(Key::Right);
    CHECK_EQ(app.program.editor.parameters.light_point, 1.0);
    // With Ctrl, the arrows do nothing.
    app.key_with(Modifiers::Control, Key::Left).key_with(Modifiers::Control, Key::Up);
    CHECK_EQ(std::pair(selected(app), app.program.editor.parameters.light_point), std::pair(std::string_view("Light Point"), 1.0));
}

TEST(the_sliders_take_no_keyboard_focus_so_the_arrows_stay_the_editor_s) {
    Headless<Demo> app = demo(Parameters{}, nowhere());
    Point shift = along(app, "Shift", 0.5f);
    app.click_point(shift);
    CHECK_EQ(selected(app), "Shift");
    CHECK_EQ(app.ui.focused_name(), std::nullopt);
    // Tab goes to the buttons, which don't take the arrows.
    app.key(Key::Tab);
    CHECK_EQ(app.ui.focused_name(), std::optional<std::string_view>("Reset"));
    app.key(Key::Right);
    CHECK_EQ(app.program.editor.parameters.shift, 0.501);
}

TEST(keys_and_their_modifiers_make_commands) {
    auto command = editor::command;
    CHECK_EQ(command(Key::Up, false, Modifiers::None), std::optional<Command>(Command::Select{-1}));
    CHECK_EQ(command(Key::Down, true, Modifiers::Shift), std::optional<Command>(Command::Select{1}));
    // Holding an arrow repeats it.
    CHECK_EQ(command(Key::Right, true, Modifiers::None), std::optional<Command>(Command::Nudge{1.0}));
    CHECK_EQ(command(Key::Left, false, Modifiers::Shift), std::optional<Command>(Command::Nudge{-10.0}));
    CHECK_EQ(command(Key::Left, false, Modifiers::Control), std::nullopt);
    // Ctrl+S saves, but holding it doesn't save again.
    CHECK_EQ(command(Key::S, false, Modifiers::Control), std::optional<Command>(Command::Save{}));
    CHECK_EQ(command(Key::S, true, Modifiers::Control), std::nullopt);
    CHECK_EQ(command(Key::S, false, Modifiers::None), std::nullopt);
    CHECK_EQ(command(Key::S, false, Modifiers::ControlShift), std::nullopt);
}

TEST(an_arrow_or_the_wheel_never_moves_a_value_the_other_way) {
    // SimplePaint accepts these, like MinimalSceneTest's sphere does, but
    // they're past the sliders' ranges, which end at 0.001 and 0.999.
    Parameters parameters;
    parameters.base_color_srgb = {0.0009765625, 0.5, 0.9990234375};
    (void)Material::compile(parameters).value();
    Headless<Demo> app = demo(parameters, nowhere());
    app.key(Key::Left);
    CHECK_EQ(app.program.editor.parameters.base_color_srgb[0], 0.0009765625, "Left left R as it was");
    app.key(Key::Right);
    CHECK_EQ(app.program.editor.parameters.base_color_srgb[0], 0.002);
    Point b = along(app, "B", 0.5f);
    app.scroll_point(b, 1.0f);
    CHECK_EQ(app.program.editor.parameters.base_color_srgb[2], 0.9990234375, "the wheel left B as it was");
    app.key(Key::Right);
    CHECK_EQ(app.program.editor.parameters.base_color_srgb[2], 0.9990234375, "Right left B as it was");
    app.key(Key::Left);
    CHECK_EQ(app.program.editor.parameters.base_color_srgb[2], 0.998);
}

TEST(ctrl_s_and_save_write_paint_json_and_it_loads_back) {
    std::filesystem::path path = folder("Save") / "Paint.json";
    Headless<Demo> app = demo(Parameters{}, path);
    Point shift = along(app, "Shift", 0.6f);
    app.click_point(shift);
    app.key_with(Modifiers::Control, Key::S);
    CHECK_EQ(app.program.editor.status, std::optional(std::pair(std::format("Saved to {}", base::display(path)), false)));
    std::vector<std::string> notices;
    CHECK_EQ(editor::load(path, notices), app.program.editor.parameters);
    CHECK(notices.empty(), "{}", testing::describe(notices));

    // The Save button does the same, and says when it couldn't.
    app.program.paint_path = nowhere();
    app.click("Save");
    const std::optional<std::pair<std::string, bool>>& status = app.program.editor.status;
    CHECK(status && status->second && status->first.starts_with(R"(Couldn't save to C:\Nowhere\Paint.json)"), "{}", testing::describe(status));
}

TEST(reset_goes_back_to_simple_paint_s_defaults) {
    Parameters shifted;
    shifted.shift = 0.5;
    Headless<Demo> app = demo(shifted, nowhere());
    app.program.editor.selected = 3;
    app.click("Reset");
    CHECK_EQ(app.program.editor.parameters, Parameters{});
    CHECK_EQ(selected(app), "Brightness", "the selection stays");
}

TEST(a_bad_paint_json_falls_back_to_the_defaults_with_a_notice) {
    std::filesystem::path missing = folder("Missing") / "Paint.json";
    std::vector<std::string> none;
    CHECK_EQ(editor::load(missing, none), Parameters{});
    CHECK(none.empty(), "a missing file is fine: {}", testing::describe(none));
    for (auto [text, problem] : std::array<std::pair<std::string_view, std::string_view>, 3>{{
             {"{ \"Shift\": }", "line 1, column 12: expected a value"},
             {"{ \"Shift\": 1.5 }", "SimplePaint Shift must be finite and in [0, 0.9990234375]."},
             {"{ \"Glow\": 1 }", "Glow isn't a SimplePaint setting."},
         }}) {
        std::filesystem::path path = folder("Bad") / "Paint.json";
        base::write(path, text).value();
        std::vector<std::string> notices;
        CHECK_EQ(editor::load(path, notices), Parameters{});
        CHECK_EQ(notices.size(), 1);
        CHECK(notices[0].contains(problem) && notices[0].ends_with("The default material is used."), "{}", notices[0]);
    }
}

TEST(the_demo_starts_with_paint_json_and_says_what_s_wrong_with_it) {
    std::filesystem::path saved = folder("Start");
    Parameters parameters;
    parameters.shift = 0.25;
    base::write(saved / "Paint.json", std::string_view(paint_settings::format(parameters, 0))).value();
    {
        Headless<Demo> app = Headless<Demo>::start(Startup(user_interface::Settings::from_text("{}"), saved, false), [](Startup& startup) { return Demo::start(startup); });
        CHECK_EQ(app.program.editor.parameters, parameters);
        CHECK(app.ui.notices().empty());
    }

    std::filesystem::path bad = folder("StartBad");
    base::write(bad / "Paint.json", std::string_view("{ \"Glow\": 1 }")).value();
    {
        Headless<Demo> app = Headless<Demo>::start(Startup(user_interface::Settings::from_text("{}"), bad, false), [](Startup& startup) { return Demo::start(startup); });
        CHECK_EQ(app.program.editor.parameters, Parameters{});
        const Notice& notice = app.ui.notices()[0];
        CHECK_EQ(std::pair(notice.severity, std::string_view(notice.title)), std::pair(Severity::Warning, std::string_view("Paint.json")));
        CHECK(notice.message.contains("Glow isn't a SimplePaint setting."), "{}", notice.message);
        CHECK(app.has_target("Notice/0"));
    }

    // A scripted run starts from the default material, whatever's there.
    Headless<Demo> app = Headless<Demo>::start(Startup::for_tests("{}", saved), [](Startup& startup) { return Demo::start(startup); });
    CHECK_EQ(app.program.editor.parameters, Parameters{});
}

TEST(any_input_leaves_a_valid_material) {
    Headless<Demo> app = demo(Parameters{}, nowhere());
    // A fixed sequence of presses, drags, wheels and keys all over the sliders.
    std::uint64_t seed = 0x5EED;
    auto next = [&] {
        seed = seed * 6364136223846793005 + 1442695040888963407;
        return static_cast<float>(seed >> 33) / static_cast<float>(std::uint64_t{1} << 31);
    };
    for (int round = 0; round < 2000; round++) {
        std::string_view name = Settings[base::saturating_cast<size_t>(next() * 8.0f) % 8].name;
        Point point = along(app, name, next() * 1.4f - 0.2f);
        Key key = std::array<Key, 4>{Key::Left, Key::Right, Key::Up, Key::Down}[base::saturating_cast<size_t>(next() * 4.0f) % 4];
        // The editor compiles every change, and would panic on an invalid one.
        switch (base::saturating_cast<std::uint32_t>(next() * 6.0f)) {
        case 0:
            app.press_point(point);
            break;
        case 1:
            app.move_point(point);
            break;
        case 2:
            app.release();
            break;
        case 3:
            app.scroll_point(point, next() * 20.0f - 10.0f);
            break;
        case 4:
            app.key(key);
            break;
        default:
            app.key_with(Modifiers::Shift, key);
            break;
        }
        (void)Material::compile(app.program.editor.parameters).value();
    }
}

TEST(the_cards_fit_both_window_heights) {
    Headless<Demo> maximized = demo(Parameters{}, nowhere());
    // The buttons, then each setting's row and slider.
    std::vector<std::string> expected{"Reset", "Save"};
    for (const Setting& setting : Settings) {
        expected.push_back(std::format("Setting/{}", setting.name));
        expected.push_back(std::string(setting.name));
    }
    CHECK_EQ(names(maximized), expected);
    for (bool fullscreen : {false, true}) {
        Headless<Demo> app = demo(Parameters{}, nowhere());
        app.set_fullscreen(fullscreen).frame();
        float height = fullscreen ? FullscreenHeight : MaximizedHeight;
        for (const std::string& name : names(app)) {
            Rect area = app.target(name);
            CHECK(area.x >= PreviewWidth && area.right() <= WindowWidth && area.bottom() <= height, "{}: {}", name, testing::describe(area));
            // The layout doesn't depend on the height above the status bar.
            CHECK_EQ(area, maximized.target(name), "{}", name);
        }
        // The view fills the preview card, and the sphere's image fills the
        // view: the whole image in fullscreen, and its middle maximized.
        Rect view = app.program.view_.value();
        CHECK_EQ(view.width, static_cast<float>(ImageWidth));
        CHECK_EQ(view.height, static_cast<float>(ImageHeight) - (fullscreen ? 0.0f : FullscreenHeight - MaximizedHeight));
        CopyPlace place = copy_place(view, {base::saturating_cast<std::uint32_t>(WindowWidth), base::saturating_cast<std::uint32_t>(height)}).value();
        std::array<std::uint32_t, 2> box_size{place.source.right - place.source.left, place.source.bottom - place.source.top};
        CHECK_EQ(place.destination, (std::array<std::uint32_t, 2>{base::saturating_cast<std::uint32_t>(view.x), base::saturating_cast<std::uint32_t>(view.y)}));
        CHECK_EQ(box_size, (std::array<std::uint32_t, 2>{base::saturating_cast<std::uint32_t>(view.width), base::saturating_cast<std::uint32_t>(view.height)}));
    }
    // Every card's text fits, with a long message from Save.
    for (bool fullscreen : {false, true}) {
        Headless<Demo> app = demo(Parameters{}, folder("Fit") / "A folder with a long name to save in" / "Paint.json");
        app.key_with(Modifiers::Control, Key::S).set_fullscreen(fullscreen);
        CHECK(app.program.editor.status && app.program.editor.status->second);
        CHECK_EQ(app.program.overflow, std::vector<std::string_view>(), "cards whose content didn't fit, fullscreen {}", fullscreen);
    }
}

TEST(the_image_s_middle_goes_in_the_view_inside_the_frame) {
    auto size = [](const CopyPlace& place) {
        return std::array<std::uint32_t, 2>{place.source.right - place.source.left, place.source.bottom - place.source.top};
    };
    // A view as big as the image takes all of it.
    CopyPlace whole = copy_place(Rect(32.0f, 120.0f, 1152.0f, 1239.0f), {2560, 1440}).value();
    CHECK_EQ(whole.source, (D3D12_BOX{.left = 0, .top = 0, .front = 0, .right = 1152, .bottom = 1239, .back = 1}));
    // A shorter one takes its middle.
    CopyPlace middle = copy_place(Rect(32.0f, 120.0f, 1152.0f, 1139.0f), {2560, 1369}).value();
    CHECK_EQ(std::tuple(middle.destination, middle.source.top, size(middle)), std::tuple(std::array<std::uint32_t, 2>{32, 120}, 50u, std::array<std::uint32_t, 2>{1152, 1139}));
    // A bigger one has the image in its middle.
    CopyPlace centered = copy_place(Rect(0.0f, 0.0f, 1352.0f, 1439.0f), {2560, 1440}).value();
    CHECK_EQ(std::pair(centered.destination, size(centered)), std::pair(std::array<std::uint32_t, 2>{100, 100}, std::array<std::uint32_t, 2>{1152, 1239}));
    // Past the frame's edges, it's cut there, and outside it, there's none.
    CopyPlace cut = copy_place(Rect(-100.0f, 1000.0f, 1152.0f, 1239.0f), {2560, 1440}).value();
    CHECK_EQ(std::tuple(cut.destination, cut.source.left, cut.source.top, size(cut)),
             std::tuple(std::array<std::uint32_t, 2>{0, 1000}, 100u, 0u, std::array<std::uint32_t, 2>{1052, 440}));
    CHECK_EQ(copy_place(Rect(3000.0f, 0.0f, 1152.0f, 1239.0f), {2560, 1440}), std::nullopt);
}
