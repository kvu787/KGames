// What the showcase's widgets show, what its shortcuts and menus do, its
// log, and its state for scripts: the model, the messages, and `update`,
// which applies them. `cards::view` describes the page from the model.

#pragma once

#include "Base/Sum.h"
#include "Base/VecDeque.h"
#include "UserInterface/UserInterface.h"
#include "Json/Json.h"

#include <array>
#include <cstdint>
#include <optional>
#include <string>
#include <string_view>
#include <variant>
#include <vector>

namespace ui_showcase::showcase {

inline constexpr std::array<std::string_view, 3> Sizes{"Small", "Medium", "Large"};
inline constexpr std::array<std::string_view, 3> Alignments{"Left", "Center", "Right"};
inline constexpr std::array<std::string_view, 4> Tools{"Brush", "Fill", "Erase", "Pick"};
inline constexpr std::array<std::string_view, 4> Tabs{"Type ramp", "Icons", "Colors", "Shapes"};
inline constexpr std::array<std::string_view, 4> Options{"Snap to grid", "Show rulers", "Autosave", "Spell check"};
inline constexpr std::array<std::string_view, 4> Buttons{"Standard", "Accent", "Subtle", "Save"};

// The list's 120 items.
std::vector<std::string> items();

// What the input on the showcase's widgets means, and what its keys and
// menus do.
struct Message {
    // A press of one of Buttons, by index.
    struct Press {
        size_t index;
        bool operator==(const Press&) const = default;
    };
    struct Link {
        bool operator==(const Link&) const = default;
    };
    struct Pin {
        bool pinned;
        bool operator==(const Pin&) const = default;
    };
    struct Add {
        bool operator==(const Add&) const = default;
    };
    struct Remove {
        bool operator==(const Remove&) const = default;
    };
    // A menu's command, by its label.
    struct Command {
        std::string label;
        bool operator==(const Command&) const = default;
    };
    struct ToggleTheme {
        bool operator==(const ToggleTheme&) const = default;
    };
    struct ToggleTooltips {
        bool operator==(const ToggleTooltips&) const = default;
    };
    struct Tooltips {
        bool on;
        bool operator==(const Tooltips&) const = default;
    };
    struct OptionsEnabled {
        bool enabled;
        bool operator==(const OptionsEnabled&) const = default;
    };
    struct Option {
        size_t index;
        bool on;
        bool operator==(const Option&) const = default;
    };
    struct Filling {
        bool filling;
        bool operator==(const Filling&) const = default;
    };
    struct Size {
        size_t size;
        bool operator==(const Size&) const = default;
    };
    struct Alignment {
        size_t alignment;
        bool operator==(const Alignment&) const = default;
    };
    struct Tool {
        size_t tool;
        bool operator==(const Tool&) const = default;
    };
    struct Volume {
        double volume;
        bool operator==(const Volume&) const = default;
    };
    struct Opacity {
        double opacity;
        bool operator==(const Opacity&) const = default;
    };
    struct Angle {
        double angle;
        bool operator==(const Angle&) const = default;
    };
    struct Role {
        size_t role;
        bool operator==(const Role&) const = default;
    };
    struct Name {
        std::string name;
        bool operator==(const Name&) const = default;
    };
    struct Greet {
        bool operator==(const Greet&) const = default;
    };
    struct CloseGreeting {
        bool operator==(const CloseGreeting&) const = default;
    };
    struct Search {
        std::string search;
        bool operator==(const Search&) const = default;
    };
    struct Quantity {
        double quantity;
        bool operator==(const Quantity&) const = default;
    };
    struct Scale {
        double scale;
        bool operator==(const Scale&) const = default;
    };
    struct Color {
        user_interface::Color color;
        bool operator==(const Color&) const = default;
    };
    struct Swatch {
        size_t index;
        bool operator==(const Swatch&) const = default;
    };
    struct Tab {
        size_t tab;
        bool operator==(const Tab&) const = default;
    };
    // A row of the list, among the items the search matches.
    struct Select {
        size_t item;
        bool operator==(const Select&) const = default;
    };
    struct Activate {
        size_t item;
        bool operator==(const Activate&) const = default;
    };
    struct CloseCaution {
        bool operator==(const CloseCaution&) const = default;
    };
    struct BringBackCaution {
        bool operator==(const BringBackCaution&) const = default;
    };
    struct Expand {
        bool expanded;
        bool operator==(const Expand&) const = default;
    };
    struct Ask {
        bool operator==(const Ask&) const = default;
    };
    struct Answer {
        user_interface::DialogResult result;
        bool operator==(const Answer&) const = default;
    };
    struct Reset {
        bool operator==(const Reset&) const = default;
    };
    // What the interface is on changed: the theme, if it did, and the names
    // of what has focus, what's pressed, and the open popup.
    struct Watch {
        std::array<std::optional<std::string>, 3> names;
        bool operator==(const Watch&) const = default;
    };

    std::variant<Press, Link, Pin, Add, Remove, Command, ToggleTheme, ToggleTooltips, Tooltips, OptionsEnabled, Option, Filling, Size, Alignment, Tool, Volume, Opacity, Angle,
                 Role, Name, Greet, CloseGreeting, Search, Quantity, Scale, Color, Swatch, Tab, Select, Activate, CloseCaution, BringBackCaution, Expand, Ask, Answer, Reset, Watch>
        value;

    template <base::AlternativeOf<Message> T> Message(T&& alternative) : value(std::forward<T>(alternative)) {}

    bool operator==(const Message&) const = default;
};

class Showcase : public user_interface::Program<Message> {
public:
    std::array<std::uint32_t, 4> presses{0, 0, 0, 0};
    std::int32_t count = 0;
    bool pinned = false;
    std::uint32_t link_clicks = 0;
    bool options_enabled = true;
    std::array<bool, 4> options{true, false, true, false};
    bool tooltips = true;
    bool filling = true;
    double fill_started = 0.0;
    size_t size = 1;
    size_t alignment = 1;
    size_t tool = 0;
    float volume = 50.0f;
    double opacity = 0.75;
    float angle = 180.0f;
    size_t role = 0;
    std::string name;
    std::string search;
    std::optional<std::string> greeting;
    std::int32_t quantity = 3;
    double scale = 1.0;
    user_interface::Color color = user_interface::Color::hex(0x1E90FF);
    size_t tab = 0;
    std::vector<std::string> items = showcase::items();
    std::optional<size_t> selected;
    std::optional<size_t> opened;
    bool caution_shown = true;
    bool expanded = true;
    bool dialog_open = false;
    std::optional<std::string> dialog_result;
    std::optional<std::string> menu_choice;
    base::VecDeque<std::string> log_;

    Showcase() = default;

    explicit Showcase(user_interface::Startup& startup);

    void log(std::string line);

    // The items that the search matches, by their index in the list.
    std::vector<size_t> matches() const;

    void reset(double time);

    std::string title() const;

    // The shortcuts, and what the interface is on, which the log tells of
    // when it changes.
    std::vector<Message> input(const user_interface::Ui& ui) const;

    void update(Message message, user_interface::Context<Message>& ui);

    user_interface::Tree<Message> view(user_interface::Ui& ui) const;

    json::Value state() const;

private:
    void toggle_theme(user_interface::Context<Message>& ui);

    void toggle_tooltips();

    // What a menu's command does, by its label.
    void command(user_interface::Context<Message>& ui, std::string_view label);

    // What the interface was on last frame, and its theme, to log what
    // changes.
    std::array<std::optional<std::string>, 3> seen;
    user_interface::ThemeKind seen_theme = user_interface::ThemeKind::Light;
};

} // namespace ui_showcase::showcase
