// What the editors share: where their cards and views are, the view of the
// ground that they pan and zoom, where points on the ground are on the screen
// for what they draw over the view, and the controls that change the
// document: sliders of its numbers, fields of its colors, and the controls
// that paint a model's slots with SimplePaint. They're Library/UserInterface's
// widgets, laid out with its Form, with colors from the theme's roles and
// sizes from its tokens.
//
// Programs only support two window setups (see Library/UserInterface's
// README), so the editors' cards have fixed places in pixels: the app bar at
// the top, with the tabs, a card at the top of the page's left side, as tall
// as what's in it, and in the vehicle editor, another at the top of its right
// side, the status bar at the bottom, and the 3D view under all of them,
// which fills the window. The cards float over the view, so it takes input
// only where they don't cover it.

#pragma once

#include "SimplePaint/Material.h"
#include "UserInterface/UserInterface.h"
#include "Veehiicuul/Camera.h"
#include "Veehiicuul/Files.h"

#include <array>
#include <cstddef>
#include <functional>
#include <optional>
#include <span>
#include <string>
#include <string_view>

namespace veehiicuul::program {

// What a frame's input means to the game, which Program.h has whole.
struct Message;

} // namespace veehiicuul::program

namespace veehiicuul::editing {

// How wide the editors' cards are.
inline constexpr float CardWidth = 520.0f;
// How tall a control is with its header above it, as Form::row puts it.
inline constexpr float HeadedHeight = user_interface::tokens::Body.line_height + user_interface::controls::TextBoxHeaderGap + user_interface::tokens::ControlHeight;

// Where the editors' card down the left side of the page may go.
user_interface::Rect left_card(user_interface::Rect page);

// Where the vehicle editor's second card, down the right side, may go.
user_interface::Rect right_card(user_interface::Rect page);

// Where the 3D view takes input: the window between the app bar, under the
// PerfBar, and the status bar, all the way across. The cards, and
// the notices under the app bar, cover it.
user_interface::Rect view_area(const user_interface::Ui& ui);

// The part of the view that a card down the left side leaves, whose middle
// is where new things go.
user_interface::Rect visible(user_interface::Rect view);

std::array<float, 2> window_size(const user_interface::Ui& ui);

// The editors' and the game's view of the ground: a target that hears the
// pointer every frame, sends what it did with whether Space is down, and
// takes keyboard focus from a text field when pressed, so that the keys
// after it, like Space, are the view's.
class GroundView : public user_interface::Widget<program::Message> {
public:
    // Whether a pan drag was under way as of the last frame, for the cursor.
    bool panning;

    explicit GroundView(bool view_panning);

    std::string_view kind() const override;

    std::optional<user_interface::Sense> sense() const override;

    bool always_interacts() const override;

    void interact(const user_interface::Interaction& interaction, user_interface::Dispatch<program::Message>& ctx) const override;
};

// What happened in an editor's view in a frame, on the ground.
struct GroundInput {
    // The left button's press, and whether it's still down, which is what
    // edits and pans.
    bool pressed = false;
    bool held = false;
    // The point on the ground under the pointer, if it's on the view.
    std::optional<std::array<double, 2>> ground;
    std::optional<std::array<float, 2>> cursor;
    // Whether Space is down or a pan drag is under way, so that a drag pans
    // instead of editing.
    bool panning = false;
};

// A view of the ground for editing: straight down, or at the game's angle.
// Space and a drag pan it, and the wheel zooms it around the pointer. It
// fills the window, so its middle is the window's.
class Viewport {
public:
    camera::Eye eye;

    Viewport(std::array<double, 2> center, double height);

    bool game_angle() const;

    void set_game_angle(bool game);

    bool panning() const;

    // What the view's input did to the viewport: the pan and the zoom, and
    // what's left for the editor's tool, on the ground.
    GroundInput apply(const user_interface::ViewInput& input, bool space, std::array<float, 2> size);

private:
    // The point on the ground that a pan holds under the pointer.
    std::optional<std::array<double, 2>> grab;
};

// The nearest of some points on the ground to the pointer, within `reach`
// pixels of it on the screen.
std::optional<size_t> nearest(const camera::Eye& eye, std::array<float, 2> size, std::array<float, 2> cursor, std::span<const std::array<double, 2>> points, float reach);

// The heading, in degrees from +Z toward +X, from one point toward another.
double heading_toward(std::array<double, 2> from, std::array<double, 2> to);

// A file's color as the interface takes it.
user_interface::Color interface_color(files::Color color);

// Where points on the ground are on the screen, in a view that fills the
// window, for what an editor draws over its view. It draws before its cards,
// clipped to the view, so that the cards and the bars go over it.
class Projection {
public:
    Projection(camera::Eye view_eye, std::array<float, 2> window);

    user_interface::Point point(std::array<double, 2> ground) const;

    // A closed outline through points on the ground.
    void outline(user_interface::Painter& painter, std::span<const std::array<double, 2>> points, float width, user_interface::Color color) const;

private:
    camera::Eye eye;
    std::array<float, 2> size;
};

// The values a slider makes: from `start` at its left end to `end` at its
// right, a whole number of steps, between `minimum` and `maximum`, which can
// be inside its ends, like for a value that must be above 0, shown with
// `decimals` and the suffix. A value from a file can be between steps, and
// stays as it is until the slider moves.
struct SliderRange {
    double start;
    double end;
    double minimum;
    double maximum;
    // Steps per unit, like 1000 for steps of 0.001.
    double steps;
    size_t decimals;
    std::string_view suffix_;

    constexpr SliderRange(double from, double to, double steps_per_unit, size_t shown_decimals) :
        start(from), end(to), minimum(from), maximum(to), steps(steps_per_unit), decimals(shown_decimals), suffix_("") {}

    constexpr SliderRange between(double lowest, double highest) const {
        SliderRange range = *this;
        range.minimum = lowest;
        range.maximum = highest;
        return range;
    }

    // What the value shows with after it, like "°".
    constexpr SliderRange suffix(std::string_view text) const {
        SliderRange range = *this;
        range.suffix_ = text;
        return range;
    }

    // The step nearest `value`, within the range. Dividing whole numbers,
    // like 107 / 1000, gives the double nearest the decimal, so values save
    // as they show, like 0.107.
    double nearest(double value) const;

    bool operator==(const SliderRange&) const = default;
};

// How tall a slider is with its header.
float slider_height();

// A slider of one of the document's numbers, with its label and its value
// above it, slider_height() tall. Only input moves the value, so that one
// from a file stays as it is until then; what moving it sends carries the
// nearest step.
user_interface::Slider<program::Message> slider(std::string_view label, double value, SliderRange range, std::function<program::Message(double)> to);

// A slider on its own row of a form.
void slider_row(user_interface::Form& form, user_interface::Tree<program::Message>& tree, std::string_view name, std::string_view label, double value, SliderRange range,
                std::function<program::Message(double)> to);

// A control's header above it, in a column of a form's row that's
// HeadedHeight tall, as Form::row puts it in a row of its own. Returns the
// control's rectangle.
user_interface::Rect header(user_interface::Tree<program::Message>& tree, user_interface::Rect column, std::string_view label);

// A field of one of the document's colors, like "#1CDA61", with a swatch of
// it. What's typed takes effect with Enter. It takes keyboard focus, unlike
// the editors' other controls, and what a change sends carries the color.
void color_field(user_interface::Tree<program::Message>& tree, std::string_view name, user_interface::Rect rect, files::Color color,
                 std::function<program::Message(files::Color)> to);

// The controls of a SimplePaint material, for one of a model's slots, which
// the combo box picks, with a swatch of its base color: the base color's
// channels and the dark point down the left, and the brightness, the shift,
// the rotation and the light point down the right. A slider's change sends
// the whole material, with its field on the nearest step.
void paint(user_interface::Form& form, user_interface::Tree<program::Message>& tree, std::string_view name, std::span<const std::string> slots, size_t slot,
           const simple_paint::Parameters& parameters, std::function<program::Message(size_t)> to_slot, std::function<program::Message(simple_paint::Parameters)> to_paint);

} // namespace veehiicuul::editing
