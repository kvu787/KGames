// What the Devices card's tooltip says about a device, from what GameInput
// and Windows' device tree say about it. A test gives the device's data, so
// it doesn't need the devices themselves.

#include "GameInputDemo/Devices.h"
#include "GameInput/GameInput.h"
#include "GameInputDemo/DeviceTree.h"
#include "GameInputDemo/Input.h"
#include "GameInputDemo/ScriptedSource.h"
#include "Testing/Testing.h"
#include "Json/Json.h"

#include <algorithm>
#include <chrono>
#include <expected>
#include <format>
#include <memory>
#include <span>
#include <string>
#include <string_view>
#include <thread>
#include <utility>
#include <vector>

using game_input::GameInputIsoKeyboard;
using game_input::GameInputKeyboardInfo;
using game_input::GameInputMouseInfo;
using game_input::GameInputMouseLeftButton;
using game_input::GameInputMouseRightButton;
using game_input::GameInputUsage;
using game_input::GameInputVersion;
using game_input_demo::Device;
using game_input_demo::Input;
using game_input_demo::Node;
using game_input_demo::ScriptedSource;
using game_input_demo::Section;
using game_input_demo::Tree;

namespace {

// A keyboard from a script, which has only the data that scripts give.
Device keyboard() {
    Input input(std::make_unique<ScriptedSource>(), 0.1f);
    std::string_view step = R"({ "Connect": { "Id": 1, "Name": "Keys", "Input": ["Keyboard"] } })";
    std::expected<void, std::string> applied = input.scripted()->apply(json::parse(step).value());
    CHECK(applied.has_value(), "{}", applied.error());
    input.update(true);
    return input.devices[0];
}

Node node(std::string_view instance_id, std::string_view name, std::string_view class_) {
    return Node{
        .instance_id = std::string(instance_id),
        .name = std::string(name),
        .class_ = std::string(class_),
    };
}

const Section& section(std::span<const Section> sections, std::string_view title) {
    auto found = std::ranges::find(sections, title, &Section::title);
    CHECK(found != sections.end(), "no {} section", title);
    return *found;
}

std::string_view value(const Section& section, std::string_view name) {
    auto found = std::ranges::find(section.rows, name, &std::pair<std::string, std::string>::first);
    CHECK(found != section.rows.end(), "{} has no {}", section.title, name);
    return found->second;
}

std::vector<std::string_view> titles(std::span<const Section> sections) {
    std::vector<std::string_view> titles;
    for (const Section& section : sections) {
        titles.push_back(section.title);
    }
    return titles;
}

// Whether the section has a row of the name.
bool has(const Section& section, std::string_view name) {
    return std::ranges::contains(section.rows, name, &std::pair<std::string, std::string>::first);
}

// Why a lookup found nothing, once its thread is done.
std::string problem(std::string_view pnp_path) {
    Tree tree = Tree::look_up(std::string(pnp_path));
    std::chrono::steady_clock::time_point start = std::chrono::steady_clock::now();
    while (tree.get() == nullptr) {
        CHECK(std::chrono::steady_clock::now() - start < std::chrono::seconds(10), "the lookup didn't finish");
        std::this_thread::yield();
    }
    std::expected<std::vector<Node>, std::string> result = *tree.get();
    CHECK(!result.has_value(), "the lookup found nodes");
    return result.error();
}

} // namespace

TEST(a_scripted_device_says_why_windows_has_nothing_on_it) {
    std::vector<Section> sections = keyboard().details();
    CHECK_EQ(titles(sections), (std::vector<std::string_view>{"GameInput", "Windows"}));
    const Section& game_input = section(sections, "GameInput");
    CHECK_EQ(value(game_input, "Family"), "virtual");
    CHECK_EQ(value(game_input, "Input"), "keyboard");
    CHECK_EQ(value(game_input, "IDs"), "vendor 0x0000, product 0x0000, revision 0x0000");
    CHECK_EQ(value(game_input, "Container"), "none");
    CHECK_EQ(value(game_input, "PnP path"), "none");
    const Section& windows = section(sections, "Windows");
    CHECK_EQ(windows.rows, (std::vector<std::pair<std::string, std::string>>{{"Not found", "the device comes from a script"}}));
}

TEST(names_what_game_input_says) {
    Device device = keyboard();
    device.vendor_id = 0x29EA;
    device.product_id = 0x0360;
    device.revision = 0x64;
    device.usage = GameInputUsage{.page = 0x01, .id = 0x06};
    device.firmware_version = GameInputVersion{.major = 1, .minor = 2, .build = 3, .revision = 4};
    device.container_id = GUID{0x00000000, 0x0000, 0x0000, {0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff}};
    device.keyboard = GameInputKeyboardInfo{
        .kind = GameInputIsoKeyboard,
        .layout = 0x409,
        .keyCount = 104,
        .functionKeyCount = 12,
        .maxSimultaneousKeys = 6,
        .platformType = 4,
        .platformSubtype = 0,
    };
    device.mouse = GameInputMouseInfo{
        .supportedButtons = GameInputMouseLeftButton | GameInputMouseRightButton,
        .sampleRate = 1000,
        .hasWheelX = false,
        .hasWheelY = true,
    };
    std::vector<Section> sections = device.details();
    const Section& game_input = section(sections, "GameInput");
    CHECK_EQ(value(game_input, "IDs"), "vendor 0x29EA, product 0x0360, revision 0x0064");
    CHECK_EQ(value(game_input, "HID usage"), "Generic Desktop: Keyboard (page 0x01, usage 0x06)");
    CHECK_EQ(value(game_input, "Versions"), "hardware 0.0.0.0, firmware 1.2.3.4");
    CHECK_EQ(value(game_input, "Container"), "{00000000-0000-0000-FFFF-FFFFFFFFFFFF}, built into this computer");
    CHECK_EQ(value(game_input, "Keyboard"), "ISO, layout 0x00000409, 104 keys, 12 function keys, 6 at once, type 4, subtype 0");
    CHECK_EQ(value(game_input, "Mouse"), "buttons Left, Right, wheel, 1000 Hz");
    // Only what the device has.
    for (std::string_view missing : {"Gamepad", "Rumble", "System buttons", "XInput index"}) {
        CHECK(!has(game_input, missing), "{} is shown", missing);
    }
}

TEST(an_unnamed_usage_shows_its_numbers) {
    Device device = keyboard();
    device.usage = GameInputUsage{.page = 0xFF00, .id = 0x01};
    std::vector<Section> sections = device.details();
    CHECK_EQ(value(section(sections, "GameInput"), "HID usage"), "page 0xFF00, usage 0x01");
}

TEST(shows_the_device_node_then_what_it_is_connected_through) {
    Device device = keyboard();
    Node keyboard_node = node(R"(HID\VID_29EA&PID_0360&MI_01\7&39A24A11&0&0000)", "HID Keyboard Device", "Keyboard");
    keyboard_node.manufacturer = "(Standard keyboards)";
    keyboard_node.service = "kbdhid";
    keyboard_node.inf = "keyboard.inf";
    keyboard_node.hardware_ids = {R"(HID\VID_29EA&PID_0360&MI_01)", "HID_DEVICE"};
    Node interface_node = node(R"(USB\VID_29EA&PID_0360&MI_01\6&1952A169&0&0001)", "USB Input Device", "HIDClass");
    interface_node.bus_name = "Kinesis Adv360";
    Node composite = node(R"(USB\VID_29EA&PID_0360\360555127546)", "USB Composite Device", "USB");
    composite.bus_name = "Kinesis Adv360";
    composite.location = "Port_#0001.Hub_#0002";
    device.tree = Tree::known(std::vector<Node>{keyboard_node, interface_node, composite});

    std::vector<Section> sections = device.details();
    CHECK_EQ(titles(sections), (std::vector<std::string_view>{"GameInput", "Windows", "Connected through, nearest first"}));
    const Section& windows = section(sections, "Windows");
    CHECK_EQ(value(windows, "Instance ID"), R"(HID\VID_29EA&PID_0360&MI_01\7&39A24A11&0&0000)");
    CHECK_EQ(value(windows, "Name"), "HID Keyboard Device");
    CHECK_EQ(value(windows, "Manufacturer"), "(Standard keyboards)");
    CHECK_EQ(value(windows, "Driver"), "kbdhid");
    CHECK_EQ(value(windows, "INF file"), "keyboard.inf");
    // One line each.
    CHECK_EQ(value(windows, "Hardware IDs"), "HID\\VID_29EA&PID_0360&MI_01\nHID_DEVICE");
    // It doesn't report a name on its bus, or say where it is.
    CHECK(!has(windows, "Bus name") && !has(windows, "Location"));

    const Section& through = section(sections, "Connected through, nearest first");
    CHECK_EQ(through.rows, (std::vector<std::pair<std::string, std::string>>{
                               {"HIDClass", "USB Input Device, \"Kinesis Adv360\"\nUSB\\VID_29EA&PID_0360&MI_01\\6&1952A169&0&0001"},
                               {"USB", "USB Composite Device, \"Kinesis Adv360\", at Port_#0001.Hub_#0002\nUSB\\VID_29EA&PID_0360\\360555127546"},
                           }));
}

TEST(a_device_node_without_a_driver_says_so) {
    Device device = keyboard();
    device.tree = Tree::known(std::vector<Node>{node(R"(HID\INTC816&COL01\3&125F27AF&0&0000)", "", "")});
    std::vector<Section> sections = device.details();
    CHECK_EQ(titles(sections), (std::vector<std::string_view>{"GameInput", "Windows"}));
    const Section& windows = section(sections, "Windows");
    for (std::string_view name : {"Name", "Manufacturer", "Class", "Driver", "Hardware IDs"}) {
        CHECK_EQ(value(windows, name), "none", "{}", name);
    }
    CHECK(!has(windows, "INF file"));
}

TEST(says_while_windows_is_looked_up) {
    Device device = keyboard();
    device.tree = Tree();
    std::vector<Section> sections = device.details();
    const Section& windows = section(sections, "Windows");
    CHECK_EQ(windows.rows, (std::vector<std::pair<std::string, std::string>>{{"Device tree", "looking it up"}}));
}

TEST(a_lookup_says_why_it_found_nothing) {
    CHECK_EQ(problem(""), "GameInput gives no PnP path");
    std::string missing = problem(R"(\\?\HID#NoSuchDevice#0#{884b96c3-56ef-11d1-bc8c-00a0c91405dd})");
    CHECK(missing.starts_with("Windows has no device interface at the PnP path"), "{}", missing);
}
