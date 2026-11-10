// What a widget is to the input: its identity, what it senses, its targets,
// which the pointer can be on, and the interaction that says what the input
// did to it in a frame. These are the types between the interface's frame
// machinery, in Interface.h, and the widgets.

#pragma once

#include "Base/Sum.h"
#include "UserInterface/Drawing/Geometry.h"
#include "UserInterface/Drawing/Painter.h"
#include "UserInterface/Input/Devices.h"

#include <cstddef>
#include <cstdint>
#include <optional>
#include <string>
#include <string_view>
#include <utility>
#include <variant>
#include <vector>

namespace user_interface {

using Id = std::uint64_t;

// A widget's identity, from its name: FNV-1a, which is fast for short names.
constexpr Id id(std::string_view name) {
    std::uint64_t hash = 0xcbf29ce484222325;
    for (char byte : name) {
        hash ^= static_cast<std::uint8_t>(byte);
        hash *= 0x0100000001b3;
    }
    return hash;
}

// The keys that a widget takes while it has keyboard focus, which the
// program's shortcuts then don't see.
enum class Keys : std::uint8_t {
    None,
    // Space and Enter, like a button's.
    Activate,
    // The arrows, Home, End, Page Up and Page Down, and Space and Enter,
    // like a slider's or a list's.
    Navigate,
    // Everything typed, and the keys that edit text.
    Text,
};

// What a widget does with the wheel turned over it, which decides whether
// what it's in may scroll instead.
enum class Wheel : std::uint8_t {
    // Leaves the wheel to what it's in, like a list or a scroll area.
    Ignored,
    // Takes the wheel, like a slider or a 3D view.
    Taken,
    // Takes the wheel only while it has keyboard focus, as WinUI's
    // NumberBox does.
    WhileFocused,
};

// What a widget wants from the pointer and the keyboard.
struct Sense {
    // Presses of each mouse button on it.
    bool left = false;
    bool right = false;
    bool middle = false;
    // Whether it takes keyboard focus when pressed, and with Tab.
    bool focusable = false;
    Keys keys = Keys::None;
    Wheel wheel = Wheel::Ignored;

    // Covers what's under it, and nothing else, like a card.
    static const Sense None;
    // Left presses, without taking focus.
    static const Sense Click;
    // Like a button: left presses, focus, and Space and Enter.
    static const Sense Button;
    // Like a slider or a list: focus, and the keys that move.
    static const Sense Navigate;
    static const Sense Text;
    // Every button and the wheel, without focus, like a 3D view's.
    static const Sense View;

    constexpr Sense() = default;
    constexpr Sense(bool left, bool right, bool middle, bool focusable, Keys keys) : left(left), right(right), middle(middle), focusable(focusable), keys(keys) {}

    // The widget takes the wheel, so what it's in won't scroll under it.
    constexpr Sense takes_wheel() const {
        Sense sense = *this;
        sense.wheel = Wheel::Taken;
        return sense;
    }

    constexpr Sense takes_wheel_while_focused() const {
        Sense sense = *this;
        sense.wheel = Wheel::WhileFocused;
        return sense;
    }

    bool takes(MouseButton button) const;

    bool operator==(const Sense&) const = default;
};

inline constexpr Sense Sense::None = Sense(false, false, false, false, Keys::None);
inline constexpr Sense Sense::Click = Sense(true, false, false, false, Keys::None);
inline constexpr Sense Sense::Button = Sense(true, false, false, true, Keys::Activate);
inline constexpr Sense Sense::Navigate = Sense(true, false, false, true, Keys::Navigate);
inline constexpr Sense Sense::Text = Sense(true, false, false, true, Keys::Text);
inline constexpr Sense Sense::View = Sense(true, true, true, false, Keys::None).takes_wheel();

// A part of a widget that's a target of its own, like a list's row or a
// text field's clear button. The whole widget is `Whole`. A part's target
// is named "<widget>/<suffix>", and keeps the part, so that the widget knows
// which of its targets the input was on.
struct Part {
    struct Whole {
        bool operator==(const Whole&) const = default;
    };
    // By its index, like a row, an item or a swatch.
    struct Index {
        size_t index;
        bool operator==(const Index&) const = default;
    };
    // By a name, like "Clear" or "Up", or a menu item's path.
    struct Named {
        std::string name;
        bool operator==(const Named&) const = default;
    };
    std::variant<Whole, Index, Named> value;

    template <base::AlternativeOf<Part> T> Part(T&& alternative) : value(std::forward<T>(alternative)) {}

    std::optional<size_t> index() const;

    bool is_named(std::string_view name) const;

    bool operator==(const Part&) const = default;
};

// How a failed check in a test shows a part.
std::string describe_value(const Part& part);

// A widget's area, which the pointer can be on: the whole widget, or one
// of its parts.
struct Target {
    Id id;
    std::string name;
    // What it shows, like a row's text, for scripts and tests.
    std::optional<std::string> label;
    // Its area, inside the clip it was in.
    Rect rect;
    Layer layer;
    bool enabled;
    bool focusable;
    Sense sense;
    // Like "Button" or "Slider", as WinUI names its controls.
    std::string_view kind;
    // Whether scripts and tests see it. Surfaces that only cover what's
    // under them, like cards, aren't listed.
    bool listed;
    // The widget it belongs to, and which part of it.
    Id owner;
    Part part;
};

// WinUI's visual states of a control, which pick its brushes.
enum class ControlState : std::uint8_t {
    Normal,
    PointerOver,
    Pressed,
    Disabled,
};

// What the input did to a target this frame. Widgets act on presses, on the
// frame that the button goes down, and never wait for its release: then a
// button sends its message, and a menu or a context menu opens.
struct Hit {
    // The pointer is on it, and it's enabled.
    bool hovered = false;
    // The left button went down on it this frame.
    bool pressed = false;
    // It has a press: a button that went down on it is still down.
    bool held = false;
    // Its left press ended this frame, wherever the pointer was, which ends
    // a drag, like a slider's.
    bool released = false;
    // The left button went down on it, or Space or Enter went down while it
    // had focus: what a button acts on.
    bool activated = false;
    // Its press was the second of a double-click.
    bool double_clicked = false;
    // The right button went down on it, which opens its context menu.
    bool secondary_pressed = false;
    // The middle button went down on it.
    bool middle_pressed = false;
    // The button that its press is of, while it's held.
    std::optional<MouseButton> held_button;
    // Notches of the wheel turned over it.
    Point wheel;

    // Whether the left button holds it down.
    bool pressing() const;

    // WinUI's visual state, from what the input did to it.
    ControlState state(bool enabled) const;
};

// What the input did to a widget and its parts this frame, which the widget
// turns into messages and changes of its own state.
struct Interaction {
    bool enabled = false;
    // What happened to each of its targets that something happened to, by
    // part. A part that nothing happened to isn't here.
    std::vector<std::pair<Part, Hit>> hits;
    // It has keyboard focus, and the keys reach it: no popup or dialog over
    // it takes them. The part with focus, if the widget has focus at all,
    // whether or not the keys reach it.
    bool focused = false;
    std::optional<Part> focused_part;
    // The pointer, while it's on the widget or the widget is held.
    std::optional<Point> pointer;
    // How far the pointer moved this frame while the widget was held.
    Point drag_delta;
    // Notches of the wheel turned over the widget's area while what's under
    // the pointer doesn't take the wheel itself, for what scrolls.
    Point wheel_over;

    // The part's hit, or null if nothing happened to it.
    const Hit* hit(const Part& part) const;

    // The whole widget's hit, or nothing's.
    Hit whole() const;

    // The part that the pointer is on.
    const Part* hovered_part() const;

    // The part with a press.
    const Part* held_part() const;

    const Part* pressed_part() const;

    const Part* activated_part() const;

    bool activated(const Part& part) const;

    bool pressed(const Part& part) const;

    bool held(const Part& part) const;

    bool hovered(const Part& part) const;

    bool any_held() const;

    // The wheel over any of its targets.
    Point wheel() const;
};

enum class Cursor : std::uint8_t {
    Arrow,
    Text,
    Hand,
    Move,
    ResizeHorizontal,
    ResizeVertical,
    ResizeDiagonal,
    ResizeAntidiagonal,
    Crosshair,
    NotAllowed,
};

} // namespace user_interface
