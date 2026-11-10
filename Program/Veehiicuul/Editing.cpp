#include "Veehiicuul/Editing.h"

#include "Veehiicuul/Angles.h"
#include "Veehiicuul/Program.h"

#include <algorithm>
#include <cmath>
#include <compare>
#include <format>
#include <numbers>
#include <utility>
#include <vector>

namespace veehiicuul::editing {

namespace tokens = user_interface::tokens;

using camera::Eye;
using program::Message;
using simple_paint::Parameters;
using user_interface::Color;
using user_interface::ColorField;
using user_interface::ComboBox;
using user_interface::Cursor;
using user_interface::Dispatch;
using user_interface::Form;
using user_interface::Hit;
using user_interface::Interaction;
using user_interface::Key;
using user_interface::MouseButton;
using user_interface::Painter;
using user_interface::Point;
using user_interface::Rect;
using user_interface::Sense;
using user_interface::Size;
using user_interface::Slider;
using user_interface::Swatch;
using user_interface::Text;
using user_interface::Tree;
using user_interface::Ui;
using user_interface::ViewInput;

Rect left_card(Rect page) {
    return page.left_strip(CardWidth);
}

Rect right_card(Rect page) {
    return page.right_strip(CardWidth);
}

Rect view_area(const Ui& ui) {
    Size size = ui.size();
    float top = ui.content_top() + tokens::AppBarHeight;
    return Rect(0.0f, top, size.width, size.height - top - tokens::StatusBarHeight);
}

Rect visible(Rect view) {
    return Rect(view.x + tokens::PagePadding + CardWidth, view.y, view.width - tokens::PagePadding - CardWidth, view.height);
}

std::array<float, 2> window_size(const Ui& ui) {
    return {ui.size().width, ui.size().height};
}

GroundView::GroundView(bool view_panning) : panning(view_panning) {}

std::string_view GroundView::kind() const {
    return "View";
}

std::optional<Sense> GroundView::sense() const {
    return Sense::View;
}

bool GroundView::always_interacts() const {
    return true;
}

void GroundView::interact(const Interaction& interaction, Dispatch<Message>& ctx) const {
    Hit hit = interaction.whole();
    if (hit.pressed || hit.secondary_pressed || hit.middle_pressed) {
        ctx.clear_focus();
    }
    bool space = ctx.key_down(Key::Space);
    if ((space || panning) && (hit.hovered || hit.held)) {
        ctx.set_cursor(Cursor::Move);
    }
    ViewInput input{
        .hovered = hit.hovered,
        .pointer = interaction.pointer,
        .pressed = hit.pressed,
        .secondary_pressed = hit.secondary_pressed,
        .middle_pressed = hit.middle_pressed,
        .double_clicked = hit.double_clicked,
        .held = hit.held,
        .held_button = hit.held_button,
        .drag_delta = interaction.drag_delta,
        .released = hit.released,
        .wheel = hit.wheel,
        .modifiers = ctx.modifiers(),
    };
    ctx.send(Message::View{input, space});
}

Viewport::Viewport(std::array<double, 2> center, double height) : eye(Eye::top_down(center, height)) {}

bool Viewport::game_angle() const {
    return eye.pitch < std::numbers::pi / 2.0 - 1e-6;
}

void Viewport::set_game_angle(bool game) {
    eye.pitch = game ? camera::Pitch : std::numbers::pi / 2.0;
}

bool Viewport::panning() const {
    return grab.has_value();
}

GroundInput Viewport::apply(const ViewInput& input, bool space, std::array<float, 2> size) {
    bool held = input.holding(MouseButton::Left);
    std::optional<std::array<float, 2>> cursor;
    if (input.pointer) {
        cursor = std::array<float, 2>{input.pointer->x, input.pointer->y};
    }
    if (input.wheel.y != 0.0f && cursor) {
        eye.zoom_at(std::pow(camera::Notch, -static_cast<double>(input.wheel.y)), *cursor, size);
    }
    if (input.pressed && space) {
        grab = cursor ? std::optional(eye.to_ground(*cursor, size)) : std::nullopt;
    }
    if (!held) {
        grab = std::nullopt;
    }
    if (grab && cursor) {
        std::array<double, 2> under = eye.to_ground(*cursor, size);
        eye.center[0] += (*grab)[0] - under[0];
        eye.center[1] += (*grab)[1] - under[1];
    }
    return GroundInput{
        .pressed = input.pressed,
        .held = held,
        .ground = cursor ? std::optional(eye.to_ground(*cursor, size)) : std::nullopt,
        .cursor = cursor,
        .panning = space || grab.has_value(),
    };
}

std::optional<size_t> nearest(const Eye& eye, std::array<float, 2> size, std::array<float, 2> cursor, std::span<const std::array<double, 2>> points, float reach) {
    // The first of the nearest, in the order of every float, as min_by with
    // total_cmp gives.
    std::optional<std::pair<size_t, float>> best;
    for (size_t index = 0; index < points.size(); index++) {
        std::array<float, 2> screen = eye.ground_to_screen(points[index], size);
        float distance = std::hypot(screen[0] - cursor[0], screen[1] - cursor[1]);
        if (!(distance <= reach)) {
            continue;
        }
        if (!best || std::strong_order(best->second, distance) > 0) {
            best = std::pair(index, distance);
        }
    }
    return best ? std::optional(best->first) : std::nullopt;
}

double heading_toward(std::array<double, 2> from, std::array<double, 2> to) {
    return to_degrees(std::atan2(to[0] - from[0], to[1] - from[1]));
}

Color interface_color(files::Color color) {
    return Color::from_bytes({color[0], color[1], color[2], 255});
}

Projection::Projection(Eye view_eye, std::array<float, 2> window) : eye(view_eye), size(window) {}

Point Projection::point(std::array<double, 2> ground) const {
    std::array<float, 2> screen = eye.ground_to_screen(ground, size);
    return Point(screen[0], screen[1]);
}

void Projection::outline(Painter& painter, std::span<const std::array<double, 2>> points, float width, Color color) const {
    std::vector<Point> screen;
    screen.reserve(points.size());
    for (std::array<double, 2> ground : points) {
        screen.push_back(point(ground));
    }
    if (!screen.empty()) {
        Point first = screen.front();
        screen.push_back(first);
    }
    painter.polyline(screen, width, color);
}

double SliderRange::nearest(double value) const {
    return std::clamp(std::round(value * steps) / steps, minimum, maximum);
}

float slider_height() {
    return Slider<Message>::height(true);
}

Slider<Message> slider(std::string_view label, double value, SliderRange range, std::function<Message(double)> to) {
    return Slider<Message>(value, range.start, range.end)
        .step(1.0 / range.steps)
        .allowed(range.minimum, range.maximum)
        .header(std::string(label))
        .show_value(range.decimals)
        .suffix(std::string(range.suffix_))
        .on_change([range, to = std::move(to)](double moved) { return to(range.nearest(moved)); });
}

void slider_row(Form& form, Tree<Message>& tree, std::string_view name, std::string_view label, double value, SliderRange range, std::function<Message(double)> to) {
    Rect rect = form.take(slider_height());
    tree.add(name, rect, slider(label, value, range, std::move(to)));
}

Rect header(Tree<Message>& tree, Rect column, std::string_view label) {
    tree.draw(column.top_strip(tokens::Body.line_height), Text::label(std::string(label)));
    return column.bottom_strip(tokens::ControlHeight);
}

void color_field(Tree<Message>& tree, std::string_view name, Rect rect, files::Color color, std::function<Message(files::Color)> to) {
    ColorField<Message> field = ColorField<Message>(interface_color(color)).on_change([to = std::move(to)](Color changed) {
        std::array<std::uint8_t, 4> bytes = changed.to_bytes();
        return to({bytes[0], bytes[1], bytes[2]});
    });
    tree.add(name, rect, std::move(field)).set_focusable(true);
}

void paint(Form& form, Tree<Message>& tree, std::string_view name, std::span<const std::string> slots, size_t slot, const Parameters& parameters,
           std::function<Message(size_t)> to_slot, std::function<Message(Parameters)> to_paint) {
    Rect row = form.control(tree, "Slot");
    Rect swatch_area = row.cut_right(tokens::ControlHeight, tokens::SpaceS);
    tree.add(std::format("{}/Slot", name), row, ComboBox<Message>(slot, slots).on_select(std::move(to_slot)));
    float red = static_cast<float>(parameters.base_color_srgb[0]);
    float green = static_cast<float>(parameters.base_color_srgb[1]);
    float blue = static_cast<float>(parameters.base_color_srgb[2]);
    tree.draw(swatch_area, Swatch(Color::rgb(red, green, blue)));
    SliderRange unit(0.0, 1.0, 1000.0, 3);
    double lowest = simple_paint::Margin;
    double highest = simple_paint::InteriorMaximum;
    SliderRange degrees = SliderRange(0.0, 360.0, 10.0, 1).between(0.0, 359.9).suffix("°");
    using Set = void (*)(Parameters&, double);
    struct Field {
        std::string_view part;
        std::string_view label;
        double value;
        SliderRange range;
        Set set;
    };
    std::array<std::array<Field, 2>, 4> rows{{
        {{
            {"Red", "Red", parameters.base_color_srgb[0], unit.between(lowest, highest), [](Parameters& p, double v) { p.base_color_srgb[0] = v; }},
            {"Brightness", "Brightness", parameters.brightness, unit.between(lowest, highest), [](Parameters& p, double v) { p.brightness = v; }},
        }},
        {{
            {"Green", "Green", parameters.base_color_srgb[1], unit.between(lowest, highest), [](Parameters& p, double v) { p.base_color_srgb[1] = v; }},
            {"Shift", "Shift", parameters.shift, unit.between(0.0, highest), [](Parameters& p, double v) { p.shift = v; }},
        }},
        {{
            {"Blue", "Blue", parameters.base_color_srgb[2], unit.between(lowest, highest), [](Parameters& p, double v) { p.base_color_srgb[2] = v; }},
            {"Rotation", "Rotation", parameters.rotation_degrees, degrees, [](Parameters& p, double v) { p.rotation_degrees = v; }},
        }},
        {{
            {"DarkPoint", "Dark point", parameters.dark_point, unit.between(0.0, highest), [](Parameters& p, double v) { p.dark_point = v; }},
            {"LightPoint", "Light point", parameters.light_point, unit.between(lowest, 1.0), [](Parameters& p, double v) { p.light_point = v; }},
        }},
    }};
    for (const std::array<Field, 2>& sliders : rows) {
        std::array<Rect, 2> columns = form.columns<2>(slider_height());
        for (size_t index = 0; index < 2; index++) {
            const Field& field = sliders[index];
            Set set = field.set;
            Slider<Message> widget = slider(field.label, field.value, field.range, [parameters, set, to = to_paint](double value) {
                Parameters edited = parameters;
                set(edited, value);
                return to(edited);
            });
            tree.add(std::format("{}/{}", name, field.part), columns[index], std::move(widget));
        }
    }
}

} // namespace veehiicuul::editing
