// Draws IdentifyDisplays' box for each display offscreen, without putting
// windows on the displays, and checks it: its size fits its text, and its
// accent bar and identifier draw.

#include "Base/Files.h"
#include "Base/Numbers.h"
#include "Displays/Displays.h"
#include "IdentifyDisplays/IdentifyDisplays.h"
#include "Testing/Testing.h"
#include "UserInterface/UserInterface.h"

// Where the repository is, for the boxes that the test saves.
#include "Folders.h"

#include <array>
#include <cstdint>
#include <cstdlib>
#include <filesystem>
#include <format>
#include <string>
#include <utility>
#include <variant>
#include <vector>

using displays::Display;
using graphics::AdapterChoice;
using graphics::Gpu;
using graphics::write_png;
using identify_displays::Identity;
using user_interface::Context;
using user_interface::Fonts;
using user_interface::Headless;
using user_interface::Program;
using user_interface::Size;
using user_interface::Theme;
using user_interface::ThemeKind;
using user_interface::Tree;
using user_interface::Ui;

namespace {

class Shown : public Program<std::monostate> {
public:
    explicit Shown(Identity identity) : identity(std::move(identity)) {}

    std::string title() const {
        return "Display identifier";
    }

    void update(std::monostate, Context<std::monostate>&) {}

    Tree<std::monostate> view(Ui& ui) const {
        Tree<std::monostate> tree;
        tree.draw(ui.bounds(), Identity(identity));
        return tree;
    }

private:
    Identity identity;
};

} // namespace

TEST(each_display_gets_a_box_that_fits_its_identifier) {
    std::vector<Display> displays = displays::enumerate();
    CHECK(!displays.empty());
    Fonts fonts;
    Gpu gpu(AdapterChoice::HighPerformance{});
    std::filesystem::path folder = std::filesystem::path(folders::Repository) / "target" / "TestOutput";
    CHECK(base::create_dir_all(folder).has_value());
    for (const Display& display : displays) {
        Identity identity({{&display}});
        auto [width, height] = identity.size(fonts);
        float scale = static_cast<float>(display.dpi) / 96.0f;
        CHECK(width >= 360.0f * scale && height > 200.0f * scale, "{} x {}", base::display(width), base::display(height));
        Headless<Shown> app = Headless<Shown>::with_size(Shown(std::move(identity)), ThemeKind::Light, Size(width, height));
        graphics::Image image = app.render(gpu);
        write_png(folder / std::format("IdentifyDisplays{}.png", display.identifier), image);
        Theme theme = Theme::light();
        std::array<std::uint8_t, 4> accent = theme.accent.to_bytes();
        std::array<std::uint8_t, 4> top = image.pixel(image.width / 2, 1);
        bool matches = true;
        for (size_t channel = 0; channel < 3; channel++) {
            matches = matches && std::abs(static_cast<std::int32_t>(top[channel]) - static_cast<std::int32_t>(accent[channel])) <= 2;
        }
        CHECK(matches, "{}", testing::describe(top));
        // The identifier's dark strokes are somewhere in its line.
        std::uint32_t line = base::saturating_cast<std::uint32_t>(4.0f * scale + 32.0f * scale);
        bool dark = false;
        for (std::uint32_t y = line; y < line + base::saturating_cast<std::uint32_t>(132.0f * scale) && !dark; y++) {
            for (std::uint32_t x = 0; x < image.width && !dark; x++) {
                dark = image.pixel(x, y)[0] < 64;
            }
        }
        CHECK(dark, "the identifier didn't draw");
    }
}
