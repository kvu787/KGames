// Shows each display's identifier on that display, and writes the
// identifiers to the console. Set "Display" in a program's settings file to
// an identifier, or run the program with --display and the identifier, to
// open it on that display.
//
// Each display gets a box in its middle, drawn with Library/UserInterface:
// its identifier, its model and its mode. The boxes close when one is
// clicked, or when Esc is pressed while one has focus. With --list, it only
// writes to the console.

#include "Base/Numbers.h"
#include "Base/Panic.h"
#include "Base/Text.h"
#include "Base/Windows.h"
#include "Displays/Displays.h"
#include "IdentifyDisplays/IdentifyDisplays.h"
#include "UserInterface/Application/Attempt.h"
#include "UserInterface/Application/Overlays.h"
#include "UserInterface/Core/Interface.h"
#include "UserInterface/Core/Tree.h"
#include "UserInterface/Design/Theme.h"
#include "UserInterface/Drawing/Text.h"

#include <fcntl.h>
#include <io.h>

#include <cstddef>
#include <cstdint>
#include <cstdio>
#include <expected>
#include <format>
#include <print>
#include <span>
#include <string>
#include <string_view>
#include <utility>
#include <variant>
#include <vector>

using displays::Display;
using identify_displays::Identity;
using user_interface::Fonts;
using user_interface::Overlay;
using user_interface::run_overlays;
using user_interface::ThemeKind;
using user_interface::Tree;
using user_interface::Ui;

namespace {

void print(std::span<const Display> displays, bool boxes) {
    size_t count = displays.size();
    std::println("{} {}, from left to right:", count, count == 1 ? "display" : "displays");
    for (const Display& display : displays) {
        std::string_view primary = display.primary ? ", the primary display" : "";
        std::string indent(display.identifier.size() + 4, ' ');
        std::println();
        std::println("  {}  {}{}", display.identifier, display.name, primary);
        std::println("{}{}", indent, display.details());
        std::println("{}At {}, {} on the desktop", indent, display.area.left, display.area.top);
        std::println("{}{}", indent, display.device_path);
    }
    std::println();
    std::println("To open a program on a display, set \"Display\" in the Settings.json in its");
    std::println("folder to the display's identifier, or run it with --display and the identifier.");
    if (boxes) {
        std::println();
        std::println("Click an identifier on screen, or press Esc, to close them.");
    }
}

std::expected<void, std::string> run() {
    bool boxes = true;
    for (const std::string& argument : base::arguments()) {
        if (argument == "--list") {
            boxes = false;
        } else {
            return std::unexpected(std::format("Unknown command-line argument: {}. The only one is --list.", argument));
        }
    }
    std::expected<std::vector<Display>, std::string> found = user_interface::attempt("Couldn't find the displays", [] { return displays::enumerate(); });
    if (!found) {
        return std::unexpected(std::move(found.error()));
    }
    const std::vector<Display>& displays = *found;
    print(displays, boxes);
    if (displays.empty() || !boxes) {
        return {};
    }

    // Clones share a monitor handle, and enumerate keeps them next to each
    // other.
    std::vector<std::vector<const Display*>> groups;
    for (const Display& display : displays) {
        if (!groups.empty() && groups.back()[0]->monitor == display.monitor) {
            groups.back().push_back(&display);
        } else {
            groups.push_back({&display});
        }
    }
    std::expected<Fonts, std::string> fonts = user_interface::attempt("Couldn't start DirectWrite", [] { return Fonts(); });
    if (!fonts) {
        return std::unexpected(std::move(fonts.error()));
    }
    std::vector<Identity> contents;
    std::vector<Overlay> overlays;
    for (const std::vector<const Display*>& group : groups) {
        Identity content(group);
        std::pair<float, float> size = content.size(*fonts);
        std::int32_t width = base::saturating_cast<std::int32_t>(size.first);
        std::int32_t height = base::saturating_cast<std::int32_t>(size.second);
        RECT area = group[0]->area;
        std::int32_t left = area.left + (area.right - area.left - width) / 2;
        std::int32_t top = area.top + (area.bottom - area.top - height) / 2;
        overlays.push_back(Overlay{
            .title = "Display identifier",
            .rect =
                RECT{
                    .left = left,
                    .top = top,
                    .right = left + width,
                    .bottom = top + height,
                },
        });
        contents.push_back(std::move(content));
    }
    return run_overlays(overlays, ThemeKind::Light, [&](Ui& ui, size_t index) {
        Tree<std::monostate> tree;
        tree.draw(ui.bounds(), Identity(contents[index]));
        return tree;
    });
}

} // namespace

int main() {
    base::fail_quietly();
    // Text goes out as it's written, as Rust's does: without this, a line
    // that's written to a file or a pipe ends with a carriage return and a
    // line feed, not a line feed alone.
    _setmode(_fileno(stdout), _O_BINARY);
    _setmode(_fileno(stderr), _O_BINARY);
    std::expected<void, std::string> result = run();
    if (!result) {
        std::println(stderr, "IdentifyDisplays: {}", result.error());
        return 1;
    }
    return 0;
}
