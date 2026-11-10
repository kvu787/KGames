// What IdentifyDisplays shows on each display: a box with the display's
// identifier, its model and its mode, as a library that
// Test/IdentifyDisplays tests without a window.

#pragma once

#include "Displays/Displays.h"
#include "UserInterface/Core/Tree.h"
#include "UserInterface/Drawing/Text.h"

#include <span>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

namespace identify_displays {

// One box: for one monitor, which shows one display, or more if monitors
// clone each other. It's a widget, drawn over its overlay's whole window.
class Identity : public user_interface::Widget<> {
public:
    explicit Identity(std::span<const displays::Display* const> displays);

    // The box's size, in physical pixels.
    std::pair<float, float> size(user_interface::Fonts& fonts) const;

    std::string_view kind() const override;

    void draw(user_interface::Draw& ctx) const override;

private:
    enum class Kind {
        Identifier,
        Name,
        Details,
        Hint,
        Gap,
    };

    // A line of a box, at 100% scale.
    struct Line {
        std::string text;
        user_interface::TextStyle style;
        Kind kind;
    };

    user_interface::TextStyle style(user_interface::TextStyle style) const;

    float padding() const;

    std::vector<Line> lines;
    // Displays at other scales get a box as big as it would look at 100%.
    float scale;
};

} // namespace identify_displays
