// The card that a test program floats over its scene's bottom left corner,
// which the scene never crosses: the program's name and what it shows, the
// frame rate, the GPU, the display, whether the window has focus, the keys,
// and what's wrong with Settings.json, if anything. MinimalSceneTest and
// SpinningCarTest show one, and F1 shows and hides it.
//
// This defines Tree::info_card, which Core/Tree.h declares.

#pragma once

#include "Base/Text.h"
#include "UserInterface/Application/Program.h"
#include "UserInterface/Core/Interface.h"
#include "UserInterface/Core/Tree.h"
#include "UserInterface/Design/Tokens.h"
#include "UserInterface/Drawing/Geometry.h"
#include "UserInterface/Input/Devices.h"
#include "UserInterface/Input/Events.h"
#include "UserInterface/Widgets/Card.h"
#include "UserInterface/Widgets/InfoBar.h"
#include "UserInterface/Widgets/Text.h"

#include <algorithm>
#include <array>
#include <cstddef>
#include <format>
#include <span>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

namespace user_interface {

// What the card says about the program. It only refers to its texts, which
// last until Tree::info_card has added the card.
struct InfoCard {
    // A label and its value, or a key and what it does.
    using Row = std::pair<std::string_view, std::string_view>;

    std::string_view title;
    // What the scene shows, like "A sphere moving at 8 units per second".
    std::string_view description;
    // Fields after the display's, each a label and its value.
    std::span<const Row> rows;
    // The program's own keys, each with what it does, on a row under the
    // library's: F1's, F9's, F10's and F11's.
    std::span<const Row> keys;

    // The card's name, which tests and scripts find it by.
    static constexpr std::string_view Name = "InfoCard";
    static constexpr float Width = 480.0f;

    // Whether F1 went down this frame, which shows or hides the card.
    // Holding it down repeats it, which does nothing more.
    static bool toggled(const Input& input);
};

// What only the info card's own code uses.
namespace info_card_detail {

// The column of the fields' labels.
inline constexpr float LabelWidth = 96.0f;

template <class M> InfoBar<M> info_bar(const Notice& notice) {
    return InfoBar<M>(notice.severity, notice.title, notice.message);
}

} // namespace info_card_detail

// Adds the card at the bottom left of the window, as tall as what it
// says.
template <class M> void Tree<M>::info_card(Ui& ui, const InfoCard& contents) {
    using Row = InfoCard::Row;
    FrameStats stats = ui.stats();
    const Environment& environment = ui.environment();
    std::string rate = stats.frames_per_second > 0.0f ? std::format("{:.0f} frames a second, {:.2f} ms each", stats.frames_per_second, stats.frame_time) : std::string("Measuring");
    std::string display = std::format("{} {}", environment.display, environment.display_name);
    std::vector<Row> rows{Row("Frame rate", rate), Row("GPU", environment.gpu), Row("Display", base::trim(display))};
    rows.insert(rows.end(), contents.rows.begin(), contents.rows.end());
    static constexpr std::array<Row, 4> library_keys{{{"F1", "Hide this"}, {"F9", "Theme"}, {"F10", "VSync"}, {"F11", "Fullscreen"}}};
    std::vector<std::span<const Row>> key_rows;
    for (std::span<const Row> keys : {std::span<const Row>(library_keys), contents.keys}) {
        if (!keys.empty()) {
            key_rows.push_back(keys);
        }
    }
    // Each key's keycaps' width, and its action's.
    std::vector<std::vector<std::pair<float, float>>> key_widths;
    key_widths.reserve(key_rows.size());
    for (std::span<const Row> keys : key_rows) {
        std::vector<std::pair<float, float>> widths;
        widths.reserve(keys.size());
        for (const auto& [key, action] : keys) {
            float key_width = Keycaps::width(ui.fonts(), key);
            float action_width = ui.measure(action, tokens::Caption).width;
            widths.emplace_back(key_width, action_width);
        }
        key_widths.push_back(std::move(widths));
    }
    std::span<const Notice> notices = ui.notices();
    float width = InfoCard::Width;
    std::vector<float> notice_heights;
    notice_heights.reserve(notices.size());
    for (const Notice& notice : notices) {
        notice_heights.push_back(info_card_detail::info_bar<M>(notice).height(ui, width - tokens::CardPadding * 2.0f));
    }
    bool window_focused = ui.window_focused();

    // The card sizes itself to its content, which goes from its top, so
    // its top is as far above the window's bottom as the card is tall:
    // under its header, the description, the fields, the window's focus
    // and the rows of keys, SpaceM apart, then each notice, SpaceM under
    // what's above it.
    float line = tokens::Body.line_height;
    float fields = static_cast<float>(rows.size()) * (line + tokens::SpaceXs) - tokens::SpaceXs;
    float keys_height = static_cast<float>(key_rows.size()) * (tokens::SpaceM + KeycapHeight);
    // From -0, as Rust's sum of floats is.
    float notices_height = -0.0f;
    for (float notice_height : notice_heights) {
        notices_height += tokens::SpaceM + notice_height;
    }
    float content = line + tokens::SpaceM + fields + tokens::SpaceM + line + keys_height + notices_height;
    float height = tokens::CardPadding * 2.0f + tokens::CardHeaderHeight + tokens::CardHeaderGap + content;
    Rect rect(tokens::PagePadding, ui.size().height - tokens::PagePadding - height, width, height);
    sized_card(ui, InfoCard::Name, rect, Card(std::string(contents.title)).floating(), [&](Ui&, Tree<M>& card, CardArea card_area) {
        Rect area = card_area.content;
        card.draw(area.cut_top(line, tokens::SpaceM), Text::label(std::string(contents.description)).secondary());
        for (const auto& [label, value] : rows) {
            card.draw(area.cut_top(line, tokens::SpaceXs), Field(std::string(label), info_card_detail::LabelWidth, std::string(value)));
        }
        area.cut_top(tokens::SpaceM - tokens::SpaceXs, 0.0f);

        // Whether the window has focus, with an info bar's status icon.
        Rect focus = area.cut_top(line, 0.0f);
        Severity severity = window_focused ? Severity::Success : Severity::Warning;
        std::string_view says = window_focused ? "The window has focus" : "No focus, so input is ignored";
        card.draw(Rect(focus.x, focus.y, tokens::Icon, focus.height), StatusIcon(severity, tokens::Icon));
        card.draw(focus.pad(Insets(tokens::Icon + tokens::SpaceS, 0.0f, 0.0f, 0.0f)), Text::label(std::string(says)));

        // Each key on its keycap, and what it does.
        for (size_t row_index = 0; row_index < key_rows.size(); row_index++) {
            std::span<const Row> keys = key_rows[row_index];
            const std::vector<std::pair<float, float>>& widths = key_widths[row_index];
            area.cut_top(tokens::SpaceM, 0.0f);
            Rect row = area.cut_top(KeycapHeight, 0.0f);
            float x = row.x;
            for (size_t index = 0; index < keys.size(); index++) {
                const auto& [key, action] = keys[index];
                auto [key_width, action_width] = widths[index];
                card.draw(Rect(x, row.y, key_width, row.height), Keycaps(std::string(key)));
                x += key_width + tokens::SpaceS;
                float room = std::max(row.right() - x, 0.0f);
                card.draw(Rect(x, row.y, std::min(action_width, room), row.height), Text::caption(std::string(action)));
                x += std::min(action_width, room) + tokens::SpaceL;
            }
        }

        for (size_t index = 0; index < notices.size(); index++) {
            area.cut_top(tokens::SpaceM, 0.0f);
            card.add(std::format("Notice/{}", index), area.cut_top(notice_heights[index], 0.0f), info_card_detail::info_bar<M>(notices[index]));
        }
        return area.y;
    });
    if (!elements.empty()) {
        elements.back().set_label(std::string(contents.title));
    }
}

} // namespace user_interface
