// The settings editor's state: the material being edited, its settings, and
// what the keys and the wheel do to them. Cards.cpp draws a slider for each
// setting with Library/UserInterface, which handles what dragging it does.
//
// Settings move in steps of 0.001, or 0.1 degrees for Rotation, and stop at
// the last step inside SimplePaint's accepted range, so the material they
// make is always valid. Values from Paint.json can be between steps.

#pragma once

#include "Base/Sum.h"
#include "SimplePaint/Material.h"
#include "UserInterface/Input/Devices.h"
#include "UserInterface/Widgets/Slider.h"
#include "Json/Json.h"

#include <array>
#include <cstddef>
#include <cstdint>
#include <filesystem>
#include <optional>
#include <string>
#include <string_view>
#include <utility>
#include <variant>
#include <vector>

namespace simple_paint_demo::editor {

struct Editor;

struct Setting {
    std::string_view name;
    // The slider's rail runs from `start` to `end`.
    double start;
    double end;
    // The first and last steps inside SimplePaint's accepted range.
    double minimum;
    double maximum;
    // Steps per unit, so a value on a step is a whole number of them.
    double steps;
    size_t decimals;
    std::string_view unit;

    constexpr Setting(std::string_view name, double start, double end, double minimum, double maximum, double steps, size_t decimals, std::string_view unit,
                      double (*get)(const simple_paint::Parameters&), void (*set)(simple_paint::Parameters&, double)) :
        name(name), start(start), end(end), minimum(minimum), maximum(maximum), steps(steps), decimals(decimals), unit(unit), get(get), set(set) {}

    double value(const simple_paint::Parameters& parameters) const;

    // The slider that shows `value`, over its rail, a step at a time, inside
    // the range. Its label, which scripts see, is the value as it's shown,
    // and its wheel accumulates parts of notches by itself.
    template <class M> user_interface::Slider<M> slider(double value) const {
        return user_interface::Slider<M>(value, start, end).step(1.0 / steps).allowed(minimum, maximum).decimals(decimals).suffix(std::string(unit));
    }

    // The step nearest `value`, inside the range. Dividing whole numbers,
    // like 107 / 1000, gives the double nearest the decimal, so values save
    // as written, like 0.107.
    double nearest(double value) const;

    // The value `count` steps from `value`. A value from Paint.json can be
    // past the range, like 0.9990234375 past 0.999, where the nearest step
    // back in it is the other way. Then it stays put.
    double nudge(double value, double count) const;

    std::string format(double value) const;

    // Like "0.001 to 0.999".
    std::string range() const;

private:
    // The editor sets a setting's value.
    friend struct Editor;

    double (*get)(const simple_paint::Parameters&);
    void (*set)(simple_paint::Parameters&, double);
};

// A setting from 0 to 1, in steps of 0.001, which is a member of the
// parameters, or one of a member's items.
#define SIMPLE_PAINT_DEMO_UNIT_SETTING(name, minimum, maximum, field)                                                                                                              \
    Setting(                                                                                                                                                                       \
        name, 0.0, 1.0, minimum, maximum, 1000.0, 3, "", [](const simple_paint::Parameters& p) { return p.field; },                                                                \
        [](simple_paint::Parameters& p, double value) { p.field = value; })

inline constexpr std::array<Setting, 8> Settings{
    SIMPLE_PAINT_DEMO_UNIT_SETTING("R", 0.001, 0.999, base_color_srgb[0]),
    SIMPLE_PAINT_DEMO_UNIT_SETTING("G", 0.001, 0.999, base_color_srgb[1]),
    SIMPLE_PAINT_DEMO_UNIT_SETTING("B", 0.001, 0.999, base_color_srgb[2]),
    SIMPLE_PAINT_DEMO_UNIT_SETTING("Brightness", 0.001, 0.999, brightness),
    SIMPLE_PAINT_DEMO_UNIT_SETTING("Shift", 0.0, 0.999, shift),
    Setting(
        "Rotation", 0.0, 360.0, 0.0, 359.9, 10.0, 1, "\u00B0", [](const simple_paint::Parameters& p) { return p.rotation_degrees; },
        [](simple_paint::Parameters& p, double value) { p.rotation_degrees = value; }),
    SIMPLE_PAINT_DEMO_UNIT_SETTING("Dark Point", 0.0, 0.999, dark_point),
    SIMPLE_PAINT_DEMO_UNIT_SETTING("Light Point", 0.001, 1.0, light_point),
};

#undef SIMPLE_PAINT_DEMO_UNIT_SETTING

// What a key does to the editor.
struct Command {
    // Selects the setting this many below the selected one, or above it if
    // it's less than 0.
    struct Select {
        std::int32_t offset;
        bool operator==(const Select&) const = default;
    };
    // Moves the selected setting by this many steps.
    struct Nudge {
        double steps;
        bool operator==(const Nudge&) const = default;
    };
    struct Save {
        bool operator==(const Save&) const = default;
    };
    std::variant<Select, Nudge, Save> value;

    template <base::AlternativeOf<Command> T> Command(T&& alternative) : value(std::forward<T>(alternative)) {}

    bool operator==(const Command&) const = default;
};

// How a failed check in a test shows a command.
std::string describe_value(const Command& command);

// What a key does, pressed with the modifiers held. Up and Down select a
// setting, and Left and Right move it a step, or 10 with Shift. Holding an
// arrow key repeats it. Ctrl+S saves, and holding it does nothing more.
std::optional<Command> command(user_interface::Key key, bool repeat, user_interface::Modifiers modifiers);

struct Editor {
    simple_paint::Parameters parameters;
    // Compiled from parameters whenever they change.
    simple_paint::GpuMaterial material;
    // The setting that the arrow keys move, which is highlighted.
    size_t selected = 0;
    // The setting whose slider the pointer drags, while the button is down.
    std::optional<size_t> dragging;
    // What happened at the last save, and whether it failed.
    std::optional<std::pair<std::string, bool>> status;

    explicit Editor(const simple_paint::Parameters& parameters);
    Editor(const Editor&) = delete;
    Editor& operator=(const Editor&) = delete;
    Editor(Editor&&) = default;
    Editor& operator=(Editor&&) = default;

    // Sets a setting to a value that's one of its steps, if it isn't already
    // that.
    void set(size_t setting, double value);

    // Moves a setting by a number of its steps.
    void nudge(size_t setting, double steps);

    void apply(Command command, const std::filesystem::path& paint_path);

    // Goes back to SimplePaint's default material, keeping the selection.
    void reset();

    // What the editor holds, for scripted runs to save and check. The
    // interface's state, which the main loop adds, says where the sliders
    // and buttons are.
    json::Value state() const;

    void save(const std::filesystem::path& path);
};

// The material in Paint.json, if there is one. Problems go in `notices`,
// and then the default material is used.
simple_paint::Parameters load(const std::filesystem::path& path, std::vector<std::string>& notices);

} // namespace simple_paint_demo::editor
