// What the mouse and the window do to GameInputDemo's cards: the combo box
// that picks which gamepad to listen to, what its list offers, the Devices
// card's list, its rows and their tooltips, focus and F11, through the
// cards' real layout, which Library/UserInterface's Headless draws without a
// window. And one frame drawn with the GPU, offscreen.

#include "GameInputDemo/Cards.h"
#include "GameInput/GameInput.h"
#include "GameInputDemo/Demo.h"
#include "GameInputDemo/DeviceTree.h"
#include "GameInputDemo/Input.h"
#include "GameInputDemo/ScriptedSource.h"
#include "Graphics/Graphics.h"
#include "Testing/Testing.h"
#include "UserInterface/Core/Interaction.h"
#include "UserInterface/Core/Interface.h"
#include "UserInterface/Design/Controls.h"
#include "UserInterface/Design/Theme.h"
#include "UserInterface/Design/Tokens.h"
#include "UserInterface/Drawing/Geometry.h"
#include "UserInterface/Input/Devices.h"
#include "UserInterface/Testing/Headless.h"
#include "Json/Json.h"

#include "Base/Files.h"
#include "Base/Text.h"
#include "Folders.h"

#include <array>
#include <cstddef>
#include <cstdint>
#include <cstdlib>
#include <expected>
#include <filesystem>
#include <format>
#include <optional>
#include <span>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

namespace controls = user_interface::controls;
namespace tokens = user_interface::tokens;
using game_input::GameInputRumbleParams;
using game_input_demo::Demo;
using game_input_demo::DeviceRows;
using game_input_demo::GamepadColumn;
using game_input_demo::GamepadComboBox;
using game_input_demo::Input;
using game_input_demo::Listen;
using game_input_demo::Node;
using game_input_demo::StickGraph;
using game_input_demo::Tree;
using user_interface::Headless;
using user_interface::Key;
using user_interface::Point;
using user_interface::Rect;
using user_interface::Size;
using user_interface::Target;
using user_interface::Theme;
using user_interface::ThemeKind;

namespace {

// The demo, drawn once, with gamepads of these names connected, so that the
// next frame's input has targets.
Headless<Demo> demo(std::span<const std::string_view> gamepads) {
    Demo program = Demo::scripted(0.1f);
    for (size_t index = 0; index < gamepads.size(); index++) {
        std::string step = std::format(R"({{ "Connect": {{ "Id": {}, "Name": "{}", "Input": ["Gamepad"] }} }})", index + 1, gamepads[index]);
        std::expected<void, std::string> done = program.script(json::parse(step).value());
        CHECK(done.has_value(), "{}", done.error());
    }
    return Headless<Demo>(std::move(program));
}

// Does what a script's Program step says, and runs a frame with it.
void step(Headless<Demo>& app, std::string_view step) {
    std::expected<void, std::string> done = app.program.script(json::parse(step).value());
    CHECK(done.has_value(), "{}", done.error());
    app.frame();
}

const Input& input(const Headless<Demo>& app) {
    CHECK(app.program.input_.has_value());
    return *app.program.input_;
}

std::string listen(const Headless<Demo>& app) {
    return std::string(input(app).listen.label());
}

// The names of the last frame's widgets.
std::vector<std::string> names(const Headless<Demo>& app) {
    std::vector<std::string> names;
    for (const Target& target : app.ui.targets()) {
        if (target.listed) {
            names.push_back(target.name);
        }
    }
    return names;
}

// The labels of the combo box's choices, in the list's order.
std::vector<std::string> choices(const Headless<Demo>& app) {
    std::vector<std::string> labels;
    for (size_t index = 0;; index++) {
        std::string name = std::format("{}/{}", GamepadComboBox, index);
        if (!app.has_target(name)) {
            break;
        }
        labels.push_back(app.label(name).value());
    }
    return labels;
}

// Opens the combo box's list.
void open(Headless<Demo>& app) {
    app.click(GamepadComboBox);
    CHECK_EQ(app.ui.popup_name(), std::optional(GamepadComboBox));
}

// The Devices card's rows, in its order.
std::vector<Rect> rows(const Headless<Demo>& app) {
    std::vector<Rect> rects;
    for (size_t index = 0;; index++) {
        std::string name = std::format("{}/{}", DeviceRows, index);
        if (!app.has_target(name)) {
            break;
        }
        rects.push_back(app.target(name));
    }
    return rects;
}

// The widget whose tooltip shows, and the tooltip's area.
std::optional<std::pair<std::string, Rect>> tooltip(const Headless<Demo>& app) {
    json::Value state = app.ui.state();
    const json::Value* shown = state.get("Tooltip");
    if (shown == nullptr) {
        return std::nullopt;
    }
    const json::Value* named = shown->get("Name");
    const json::Value::String* name = named != nullptr ? base::as<json::Value::String>(*named) : nullptr;
    if (name == nullptr) {
        return std::nullopt;
    }
    const json::Value* given = shown->get("Area");
    const json::Value::Array* area = given != nullptr ? base::as<json::Value::Array>(*given) : nullptr;
    if (area == nullptr) {
        testing::fail("a tooltip without an area");
    }
    auto number = [&](size_t index) {
        const json::Value::Number* found = base::as<json::Value::Number>(area->values[index]);
        if (found == nullptr) {
            testing::fail("an area that isn't numbers");
        }
        return static_cast<float>(found->value);
    };
    return std::pair(name->value, Rect(number(0), number(1), number(2), number(3)));
}

// The tooltip once the pointer has rested at the point.
std::optional<std::pair<std::string, Rect>> rest(Headless<Demo>& app, Point point) {
    app.move_point(point).wait(0.6);
    return tooltip(app);
}

// A device with as much in its tooltip as a real one, whose node in
// Windows' device tree has `nodes` - 1 nodes above it.
Headless<Demo> real_device(size_t nodes) {
    Headless<Demo> app = demo({{"Pad One", "Pad Two"}});
    Node node{
        .instance_id = R"(USB\VID_045E&PID_0B12\3032434D30303237303135323037)",
        .name = "Xbox Controller",
        .bus_name = "Controller",
        .class_ = "XboxComposite",
        .location = "Port_#0003.Hub_#0004",
        .hardware_ids = std::vector<std::string>(6, R"(USB\VID_045E&PID_0B12&REV_0509)"),
    };
    Input& read = app.program.input_.value();
    read.devices[1].tree = Tree::known(std::vector<Node>(nodes, node));
    std::string_view path = R"(\\?\HID#VID_045E&PID_0B12&IG_00#3&2A8D2A0C&0&0000#{4D1E55B2-F16F-11CF-88CB-001111000030}\KBD)";
    read.devices[1].pnp_path = std::format("{}{}", path, path);
    app.frame();
    return app;
}

bool close(std::array<std::uint8_t, 4> a, std::array<std::uint8_t, 4> b) {
    for (size_t index = 0; index < 3; index++) {
        if (std::abs(static_cast<int>(a[index]) - static_cast<int>(b[index])) > 2) {
            return false;
        }
    }
    return true;
}

} // namespace

TEST(the_combo_box_and_the_device_list_are_the_only_widgets) {
    Headless<Demo> app = demo({{"Pad One", "Pad Two"}});
    CHECK_EQ(names(app), (std::vector<std::string>{std::string(GamepadComboBox), std::string(DeviceRows), "Devices/0", "Devices/1"}));
    CHECK_EQ(app.label(GamepadComboBox), std::optional<std::string>("Pad One"));
    // In the Gamepad card's header, after its title.
    Rect combo_box = app.target(GamepadComboBox);
    float header_y = tokens::PerfBarHeight + tokens::AppBarHeight + tokens::PagePadding + tokens::CardPadding;
    CHECK_EQ(combo_box.y, header_y);
    CHECK(combo_box.x > tokens::PagePadding + tokens::CardPadding + 40.0f, "{}", testing::describe(combo_box));
    CHECK(combo_box.right() < tokens::PagePadding + GamepadColumn, "{}", testing::describe(combo_box));
    CHECK_EQ(app.label("Devices/1"), std::optional<std::string>("Pad Two"));
}

TEST(the_open_list_offers_each_gamepad_and_none) {
    Headless<Demo> app = demo({{"Pad One", "Pad Two"}});
    open(app);
    CHECK_EQ(choices(app), (std::vector<std::string>{"Pad One", "Pad Two", "None"}));

    // A click on a choice picks it, and closes the list.
    app.click("Gamepad/1");
    CHECK_EQ(listen(app), "Pad Two");
    CHECK(input(app).listen_picked);
    CHECK_EQ(app.ui.popup_name(), std::nullopt);
    CHECK_EQ(app.label(GamepadComboBox), std::optional<std::string>("Pad Two"));
    open(app);
    app.click("Gamepad/2");
    CHECK_EQ(input(app).listen, Listen(Listen::NoGamepad{}));
    CHECK_EQ(app.label(GamepadComboBox), std::optional<std::string>("None"));
}

TEST(a_picked_gamepad_stays_in_the_list_while_it_is_disconnected) {
    Headless<Demo> app = demo({{"Pad One", "Pad Two"}});
    open(app);
    app.click("Gamepad/1");
    step(app, R"({ "Disconnect": 2 })");
    CHECK_EQ(listen(app), "Pad Two");
    CHECK_EQ(rows(app).size(), 1);
    open(app);
    CHECK_EQ(choices(app), (std::vector<std::string>{"Pad One", "Pad Two", "None"}));
}

TEST(without_gamepads_the_list_offers_none) {
    Headless<Demo> app = demo({});
    CHECK_EQ(app.label(GamepadComboBox), std::optional<std::string>("None"));
    open(app);
    CHECK_EQ(choices(app), (std::vector<std::string>{"None"}));
}

TEST(esc_or_a_press_off_the_list_closes_it_and_keys_leave_the_gamepad_alone) {
    Headless<Demo> app = demo({{"Pad One", "Pad Two"}});
    open(app);
    app.key(Key::Escape);
    CHECK_EQ(app.ui.popup_name(), std::nullopt);
    open(app);
    app.click_point(Point(1300.0f, 300.0f));
    CHECK_EQ(app.ui.popup_name(), std::nullopt);
    CHECK_EQ(listen(app), "Pad One");

    // Neither a press on the box nor one on a choice gives it keyboard
    // focus, so once the list is closed, the keys that the user presses to
    // try the Keyboard card leave the gamepad alone.
    open(app);
    app.click("Gamepad/0");
    for (Key key : {Key::Down, Key::Up, Key::End, Key::Tab, Key::Space, Key::Down}) {
        app.key(key);
        CHECK_EQ(app.ui.focused_name(), std::nullopt);
        CHECK_EQ(app.ui.popup_name(), std::nullopt);
    }
    CHECK_EQ(listen(app), "Pad One");
}

TEST(while_the_list_is_open_the_arrows_move_to_a_choice_and_enter_picks_it) {
    Headless<Demo> app = demo({{"Pad One", "Pad Two"}});
    open(app);
    CHECK_EQ(app.ui.popup_name(), std::optional(GamepadComboBox));
    app.key(Key::Down);
    CHECK_EQ(listen(app), "Pad One");
    app.key(Key::End).key(Key::Enter);
    CHECK_EQ(input(app).listen, Listen(Listen::NoGamepad{}));
    CHECK_EQ(app.ui.popup_name(), std::nullopt);
}

TEST(without_game_input_there_is_nothing_to_point_at) {
    Headless app(Demo::with_input(std::unexpected(std::string("GameInput isn't installed"))));
    CHECK(names(app).empty());
    CHECK_EQ(app.state(), json::Value(json::Value::Object{{{"Interface", app.ui.state()}}}));
}

TEST(f11_switches_fullscreen_from_the_window_and_not_through_game_input) {
    Headless<Demo> app = demo({});
    step(app, R"({ "Connect": { "Id": 9, "Name": "Keys", "Input": ["Keyboard"] } })");
    app.key(Key::F11);
    CHECK(app.fullscreen());
    // GameInput reads the same key, which mustn't switch it back.
    step(app, R"({ "Keyboard": { "Id": 9, "Down": ["F11"] } })");
    step(app, R"({ "Keyboard": { "Id": 9, "Down": [] } })");
    CHECK(app.fullscreen());
    CHECK_EQ(input(app).log_[1].text, "Keys: F11 down");
}

TEST(input_without_window_focus_is_dropped_and_stops_the_motors) {
    Headless<Demo> app = demo({});
    step(app, R"({ "Connect": { "Id": 1, "Name": "Pad", "Input": ["Gamepad"], "Motors": ["LowFrequency"] } })");
    step(app, R"({ "Connect": { "Id": 2, "Name": "Keys", "Input": ["Keyboard"] } })");
    step(app, R"({ "Keyboard": { "Id": 2, "Down": ["F1"] } })");
    step(app, R"({ "Gamepad": { "Id": 1, "LeftStick": [1, 0] } })");
    CHECK(input(app).rumble_enabled);
    auto rumble = [](Headless<Demo>& of) {
        return of.program.input_.value().scripted()->rumble(1);
    };
    CHECK_EQ(rumble(app).value().lowFrequency, 1.0f);

    app.focus(false);
    CHECK(!rumble(app).has_value());
    std::uint64_t count = input(app).reading_count;
    step(app, R"({ "Gamepad": { "Id": 1, "LeftStick": [0, 1], "Buttons": ["A"] } })");
    CHECK_EQ(input(app).reading_count, count);
    CHECK_EQ(static_cast<std::int32_t>(input(app).gamepad.state.buttons), 0);

    // With focus again, the button held from before doesn't count until
    // it's let go of, but the stick does.
    app.focus(true);
    step(app, R"({ "Gamepad": { "Id": 1, "LeftStick": [0, 0.5], "Buttons": ["A"] } })");
    CHECK_EQ(input(app).gamepad.state.leftThumbstickY, 0.5f);
    CHECK_EQ(static_cast<std::int32_t>(input(app).gamepad.state.buttons), 0);
}

TEST(each_device_has_a_row_under_the_one_before) {
    Headless<Demo> app = demo({{"Pad One", "Pad Two", "Pad Three"}});
    std::vector<Rect> rows = ::rows(app);
    CHECK_EQ(rows.size(), 3);
    for (size_t index = 0; index + 1 < rows.size(); index++) {
        Rect above = rows[index];
        Rect below = rows[index + 1];
        CHECK_EQ(std::pair(below.x, below.width), std::pair(above.x, above.width));
        CHECK_EQ(below.y, above.bottom());
    }
    // In the second column, under the Latest input event card.
    CHECK(rows[0].x > tokens::PagePadding + GamepadColumn, "{}", testing::describe(rows[0]));
}

TEST(rows_that_do_not_fit_scroll) {
    Headless<Demo> app = demo({});
    for (int id = 1; id <= 40; id++) {
        step(app, std::format(R"({{ "Connect": {{ "Id": {}, "Name": "Keys {}", "Input": ["Keyboard"] }} }})", id, id));
    }
    for (bool fullscreen : {false, true}) {
        app.set_fullscreen(fullscreen);
        // The list ends above the card's padding and the status bar, and so
        // do the rows in sight, with a scroll bar for the rest.
        Rect list = app.target(DeviceRows);
        CHECK(list.bottom() <= app.size().height - tokens::StatusBarHeight - tokens::PagePadding - tokens::CardPadding, "{}", testing::describe(list));
        std::vector<Rect> rows = ::rows(app);
        CHECK(rows.size() > 5 && rows.size() < 40, "{}", rows.size());
        for (Rect row : rows) {
            CHECK(row.bottom() <= list.bottom(), "{}", testing::describe(rows));
        }
        CHECK(app.has_target("Devices/ScrollBar"));
        // The wheel scrolls to the last row, and back.
        app.scroll(DeviceRows, -40.0f);
        CHECK(app.has_target("Devices/39") && !app.has_target("Devices/0"));
        CHECK_EQ(app.label("Devices/39"), std::optional<std::string>("Keys 40"));
        app.scroll(DeviceRows, 40.0f);
        CHECK(app.has_target("Devices/0"));
    }
}

TEST(keys_and_details_that_do_not_fit_scroll) {
    Headless<Demo> app = demo({{"Pad"}});
    step(app, R"({ "Connect": { "Id": 9, "Name": "Keys", "Input": ["Keyboard"] } })");
    CHECK(!app.has_target("Keys/ScrollBar") && !app.has_target("Latest/ScrollBar"));
    std::vector<std::string> keys;
    for (char key = 'A'; key <= 'Z'; key++) {
        keys.push_back(std::format("\"{}\"", key));
    }
    for (char key = '0'; key <= '9'; key++) {
        keys.push_back(std::format("\"{}\"", key));
    }
    step(app, std::format(R"({{ "Keyboard": {{ "Id": 9, "Down": [{}] }} }})", base::join(keys, ", ")));
    CHECK_EQ(input(app).keyboard.keys.size(), 36);
    CHECK(app.has_target("Keys/ScrollBar"));
    std::vector<std::string> buttons;
    for (const auto& [button, name] : game_input_demo::GamepadButtons) {
        buttons.push_back(std::format("\"{}\"", name));
    }
    step(app, std::format(R"({{ "Gamepad": {{ "Id": 1, "LeftStick": [1, 0], "Buttons": [{}] }} }})", base::join(buttons, ", ")));
    CHECK(input(app).latest.value().changes.size() > 20);
    CHECK(app.has_target("Latest/ScrollBar"));
}

TEST(the_pointer_resting_on_a_row_shows_its_devices_tooltip) {
    Headless<Demo> app = demo({{"Pad One", "Pad Two"}});
    std::vector<Rect> rows = ::rows(app);
    for (size_t index = 0; index < rows.size(); index++) {
        std::string name = rest(app, rows[index].center()).value().first;
        CHECK_EQ(name, std::format("Devices/{}", index));
    }
    CHECK_EQ(app.label("Devices/0"), std::optional<std::string>("Pad One"));
    // Not before half a second.
    app.move_point(rows[0].center()).wait(0.3);
    CHECK(!tooltip(app).has_value());
    // Not under the last row.
    Rect last = rows[1];
    CHECK(!rest(app, Point(last.center().x, last.bottom() + last.height)).has_value());
}

TEST(a_tooltip_stays_in_the_window_off_the_pointer) {
    for (size_t nodes : {1, 5, 9}) {
        for (bool fullscreen : {false, true}) {
            Headless<Demo> app = real_device(nodes);
            app.set_fullscreen(fullscreen);
            Size size = app.size();
            for (Rect row : rows(app)) {
                for (Point pointer : {Point(row.x + 1.0f, row.center().y), Point(row.right() - 1.0f, row.center().y)}) {
                    Rect area = rest(app, pointer).value().second;
                    std::string where = std::format("{} nodes, {}: {}", nodes, testing::describe(pointer), testing::describe(area));
                    // The program's part of the window, under the
                    // PerfBar.
                    float top = tokens::PerfBarHeight;
                    CHECK(area.x >= 0.0f && area.right() <= size.width && area.y >= top, "{}", where);
                    // A deep device's tooltip, though wider than WinUI's, can
                    // be taller than the maximized window. Then it starts at
                    // the top of the program's part of it.
                    if (area.height <= size.height - top - tokens::SpaceS * 2.0f) {
                        CHECK(area.bottom() <= size.height, "{}", where);
                    } else {
                        CHECK_EQ(area.y, top + tokens::SpaceS, "{}", where);
                    }
                    // Above the pointer, or under it if there's no room
                    // above. A tooltip with room on neither side covers it.
                    auto room = [&](float space) {
                        return area.height + controls::TooltipOffset + tokens::SpaceS <= space;
                    };
                    if (room(pointer.y - top) || room(size.height - pointer.y)) {
                        CHECK(pointer.y < area.y || pointer.y > area.bottom(), "{}", where);
                    }
                    if (room(pointer.y - top)) {
                        CHECK(area.bottom() < pointer.y, "{}", where);
                    }
                }
            }
        }
    }
}

TEST(a_long_value_wraps_in_its_tooltip) {
    Headless<Demo> app = real_device(5);
    Rect row = rows(app)[1];
    Rect area = rest(app, row.center()).value().second;
    // The device's path, twice as long as a real one, wraps instead of
    // making the tooltip as wide, which is wider than WinUI's, so that every
    // detail fits in the window's height.
    CHECK(area.width > controls::TooltipMaxWidth && area.width <= game_input_demo::DeviceTooltipWidth, "{}", testing::describe(area));
    CHECK(area.bottom() <= tokens::MaximizedHeight, "{}", testing::describe(area));
}

TEST(every_widget_fits_in_the_window_at_both_heights_and_in_both_themes) {
    for (ThemeKind theme : {ThemeKind::Light, ThemeKind::Dark}) {
        for (bool fullscreen : {false, true}) {
            Headless<Demo> app = Headless<Demo>::with_theme(Demo::scripted(0.1f), theme);
            for (int id = 1; id <= 3; id++) {
                std::string step_text = std::format(R"({{ "Connect": {{ "Id": {}, "Name": "Pad {}", "Input": ["Gamepad"] }} }})", id, id);
                step(app, step_text);
            }
            app.set_fullscreen(fullscreen);
            open(app);
            float height = fullscreen ? tokens::FullscreenHeight : tokens::MaximizedHeight;
            for (const Target& target : app.ui.targets()) {
                if (!target.listed) {
                    continue;
                }
                Rect rect = target.rect;
                CHECK(rect.x >= 0.0f && rect.y >= 0.0f && rect.right() <= tokens::WindowWidth && rect.bottom() <= height, "{} is at {}, outside the window", target.name,
                      testing::describe(rect));
            }
        }
    }
}

TEST(the_stick_graphs_draw_where_the_sticks_are) {
    graphics::Gpu gpu(graphics::AdapterChoice::HighPerformance{});
    Headless<Demo> app = demo({{"Pad"}});
    step(app, R"({ "Gamepad": { "Id": 1, "LeftStick": [1, 0], "RightStick": [0, 1], "Buttons": ["A"] } })");
    graphics::Image image = app.render(gpu);
    std::filesystem::path folder = std::filesystem::path(folders::Repository) / "target" / "TestOutput";
    std::expected<void, std::string> made = base::create_dir_all(folder);
    CHECK(made.has_value(), "{}", made.error());
    graphics::write_png(folder / "GameInputDemo.png", image);

    // The graphs are under the Gamepad card's header and the captions over
    // them, with a ring at the stick's edge, SpaceS inside the graph.
    Theme theme(ThemeKind::Light);
    float top = tokens::PerfBarHeight + tokens::AppBarHeight + tokens::PagePadding + tokens::CardPadding + tokens::CardHeaderHeight + tokens::CardHeaderGap +
                tokens::Body.line_height + tokens::SpaceS;
    float left = tokens::PagePadding + tokens::CardPadding;
    float radius = StickGraph / 2.0f - tokens::SpaceS;
    Point center(left + StickGraph / 2.0f, top + StickGraph / 2.0f);
    // Left stick right, in the left stick's color.
    std::array<std::uint8_t, 4> dot = image.pixel(static_cast<std::uint32_t>(center.x + radius), static_cast<std::uint32_t>(center.y));
    CHECK(close(dot, theme.data.blue.to_bytes()), "{}", testing::describe(dot));
    // Right stick up: stick y is up, and screen y down.
    Point right(center.x + StickGraph + 24.0f, center.y - radius);
    dot = image.pixel(static_cast<std::uint32_t>(right.x), static_cast<std::uint32_t>(right.y));
    CHECK(close(dot, theme.data.orange.to_bytes()), "{}", testing::describe(dot));
    // Nothing at the left graph's center but its axes and its dead zone.
    std::array<std::uint8_t, 4> middle = image.pixel(static_cast<std::uint32_t>(center.x) + 3, static_cast<std::uint32_t>(center.y) + 3);
    CHECK(!close(middle, theme.data.blue.to_bytes()), "{}", testing::describe(middle));
}
