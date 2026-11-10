#include "Veehiicuul/VehicleEditor.h"

#include "Base/Text.h"
#include "Veehiicuul/Angles.h"
#include "Veehiicuul/Collision.h"
#include "Veehiicuul/Program.h"

#include <algorithm>
#include <cmath>
#include <format>
#include <vector>

namespace veehiicuul::vehicle_editor {

namespace tokens = user_interface::tokens;

using collision::Footprint;
using document::Document;
using editing::GroundInput;
using editing::HeadedHeight;
using editing::header;
using editing::Projection;
using editing::slider_height;
using editing::SliderRange;
using editing::window_size;
using json::Value;
using models::Models;
using program::Message;
using simple_paint::Parameters;
using user_interface::Button;
using user_interface::Canvas;
using user_interface::Card;
using user_interface::CardArea;
using user_interface::ComboBox;
using user_interface::Form;
using user_interface::Icon;
using user_interface::InfoBar;
using user_interface::ListRows;
using user_interface::ListView;
using user_interface::Painter;
using user_interface::Point;
using user_interface::Rect;
using user_interface::Severity;
using user_interface::Text;
using user_interface::TextBox;
using user_interface::Theme;
using user_interface::Tree;
using user_interface::Ui;
using user_interface::controls::ListItemHeight;
using vehicles::Vehicle;

namespace {

// How near the pointer a side counts as under it, in pixels.
constexpr float Reach = 10.0f;
// The most rows the list shows without scrolling.
constexpr size_t ListRowCount = 6;

// How far the footprint goes to each side, as Sides has them.
std::array<double, 4> sides(const Footprint& footprint) {
    return {footprint.max_[0], -footprint.min_[0], footprint.max_[1], -footprint.min_[1]};
}

// Each side's corners, as Sides has them, from the footprint's corners.
std::array<std::array<std::array<double, 2>, 2>, 4> side_corners(const std::array<std::array<double, 2>, 4>& corners) {
    return {{{corners[1], corners[2]}, {corners[3], corners[0]}, {corners[2], corners[3]}, {corners[0], corners[1]}}};
}

float distance_to_segment(std::array<float, 2> from, std::array<float, 2> start, std::array<float, 2> end) {
    Point point(from[0], from[1]);
    Point a(start[0], start[1]);
    Point b(end[0], end[1]);
    Point along = b - a;
    float length = along.x * along.x + along.y * along.y;
    float t = length > 0.0f ? std::clamp(((point.x - a.x) * along.x + (point.y - a.y) * along.y) / length, 0.0f, 1.0f) : 0.0f;
    return point.distance(a.lerp(b, t));
}

} // namespace

void VehicleEditor::sync_name(const Document& document, size_t vehicle) {
    name_text = document.vehicles.list[vehicle].name;
}

void VehicleEditor::view_input(const GroundInput& input, Document& document, size_t vehicle, std::array<float, 2> size) {
    camera::Eye eye = viewport.eye;
    double heading = to_radians(ShownHeading);
    Vehicle& current = document.vehicles.list[vehicle];
    std::array<double, 4> values = sides(current.footprint);
    std::array<std::array<double, 2>, 4> corners = current.footprint.corners({0.0, 0.0}, heading);
    under = std::nullopt;
    if (input.cursor && !input.panning) {
        std::array<std::array<std::array<double, 2>, 2>, 4> edges = side_corners(corners);
        for (size_t side = 0; side < 4; side++) {
            std::array<float, 2> a = eye.ground_to_screen(edges[side][0], size);
            std::array<float, 2> b = eye.ground_to_screen(edges[side][1], size);
            if (distance_to_segment(*input.cursor, a, b) <= Reach) {
                under = side;
                break;
            }
        }
    }
    if (input.pressed && !input.panning) {
        drag = under;
    }
    // The drag follows the pointer to where it's let go of.
    if (drag && input.ground) {
        std::array<double, 2> ground = *input.ground;
        double sin = std::sin(heading);
        double cos = std::cos(heading);
        double left = ground[0] * cos - ground[1] * sin;
        double ahead = ground[0] * sin + ground[1] * cos;
        std::array<double, 4> dragged{left, -left, ahead, -ahead};
        values[*drag] = std::clamp(dragged[*drag], 0.05, 10.0);
        current.footprint = Footprint::from_sides(values[0], values[1], values[2], values[3]);
    }
    if (!input.held) {
        drag = std::nullopt;
    }
}

void VehicleEditor::apply(const VehicleMsg& given, Document& document, const Models& models, size_t& vehicle) {
    base::match(
        given, [&](const VehicleMsg::Select& select) { vehicle = std::min(select.index, document.vehicles.list.size() - 1); },
        [&](const VehicleMsg::Add&) {
            Vehicle copy = document.vehicles.list[vehicle];
            copy.name = document.vehicles.new_name();
            document.vehicles.list.push_back(std::move(copy));
            vehicle = document.vehicles.list.size() - 1;
            message.clear();
        },
        [&](const VehicleMsg::Delete&) {
            std::string name = document.vehicles.list[vehicle].name;
            // A parked name matches without case, as Vehicles::find
            // reads it, so a hand-edited scene can't dodge the guard.
            auto parked = std::ranges::find_if(document.scenes, [&](const scene::Scene& scene) {
                return std::ranges::any_of(scene.parked, [&](const scene::Parked& placed) { return base::eq_ignore_ascii_case(placed.vehicle, name); });
            });
            if (document.vehicles.list.size() == 1) {
                message = "The game needs a vehicle, so the last can't go.";
            } else if (parked != document.scenes.end()) {
                message = std::format("{} is parked in {}, so it can't go.", name, parked->name);
            } else {
                document.vehicles.list.erase(document.vehicles.list.begin() + static_cast<std::ptrdiff_t>(vehicle));
                vehicle = std::min(vehicle, document.vehicles.list.size() - 1);
                message = std::string();
            }
        },
        [&](const VehicleMsg::Name& typed) {
            name_text = typed.text;
            // A rename also changes the name where it's parked.
            std::string name(base::trim(name_text));
            std::string old = document.vehicles.list[vehicle].name;
            if (!name.empty() && name != old && !document.vehicles.find(name)) {
                for (scene::Scene& scene : document.scenes) {
                    for (scene::Parked& placed : scene.parked) {
                        if (base::eq_ignore_ascii_case(placed.vehicle, old)) {
                            placed.vehicle = name;
                        }
                    }
                }
                document.vehicles.list[vehicle].name = std::move(name);
            }
        },
        [&](const VehicleMsg::Model& chosen) {
            std::vector<std::string_view> names = models.names();
            document.vehicles.list[vehicle].model = std::string(names[std::min(chosen.model, names.size() - 1)]);
        },
        [&](const VehicleMsg::Rate& rate) {
            driving::Acceleration& acceleration = document.vehicles.list[vehicle].acceleration;
            switch (rate.index) {
            case 0:
                acceleration.forward = rate.value;
                break;
            case 1:
                acceleration.reverse = rate.value;
                break;
            case 2:
                acceleration.left = rate.value;
                break;
            default:
                acceleration.right = rate.value;
            }
        },
        [&](const VehicleMsg::Side& moved) {
            Vehicle& current = document.vehicles.list[vehicle];
            std::array<double, 4> values = sides(current.footprint);
            values[moved.side] = moved.value;
            current.footprint = Footprint::from_sides(values[0], values[1], values[2], values[3]);
        },
        [&](const VehicleMsg::Fit&) {
            Vehicle& current = document.vehicles.list[vehicle];
            size_t model = models.find(current.model).value_or(0);
            current.footprint = models.list[model].footprint;
        },
        [&](const VehicleMsg::Slot& chosen) { slot = chosen.slot; },
        [&](const VehicleMsg::Paint& painted) {
            Vehicle& current = document.vehicles.list[vehicle];
            size_t model = models.find(current.model).value_or(0);
            const std::vector<std::string>& slots = models.list[model].slots;
            const std::string& name = slots[std::min(slot, slots.size() - 1)];
            models::set_slot(current.paint, slots, name, painted.parameters);
        },
        [&](const VehicleMsg::ResetPaint&) {
            Vehicle& current = document.vehicles.list[vehicle];
            size_t model = models.find(current.model).value_or(0);
            const std::vector<std::string>& slots = models.list[model].slots;
            const std::string& name = slots[std::min(slot, slots.size() - 1)];
            std::erase_if(current.paint, [&](const std::pair<std::string, Parameters>& entry) { return entry.first == name; });
        });
}

void VehicleEditor::view(Ui& ui, Tree<Message>& tree, Rect page, Rect view_rect, const Document& document, const Models& models, size_t vehicle) const {
    // The model's rectangle, thin, and the footprint, with a handle on
    // each side.
    double heading = to_radians(ShownHeading);
    const Vehicle& current = document.vehicles.list[vehicle];
    std::optional<std::array<std::array<double, 2>, 4>> model_corners;
    if (std::optional<size_t> model = models.find(current.model)) {
        model_corners = models.list[*model].footprint.corners({0.0, 0.0}, heading);
    }
    std::array<std::array<double, 2>, 4> corners = current.footprint.corners({0.0, 0.0}, heading);
    std::optional<size_t> dragged = drag;
    std::optional<size_t> pointed = under;
    Projection projection(viewport.eye, window_size(ui));
    tree.draw(view_rect, Canvas([model_corners, corners, dragged, pointed, projection](Painter& painter, Rect rect) {
                  const Theme& theme = painter.theme();
                  painter.push_clip(rect);
                  if (model_corners) {
                      projection.outline(painter, *model_corners, 1.0f, theme.text_secondary);
                  }
                  projection.outline(painter, corners, 2.0f, theme.accent);
                  std::array<std::array<std::array<double, 2>, 2>, 4> edges = side_corners(corners);
                  for (size_t side = 0; side < edges.size(); side++) {
                      std::array<double, 2> a = edges[side][0];
                      std::array<double, 2> b = edges[side][1];
                      Point middle = projection.point({(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0});
                      bool active = dragged == side || (!dragged && pointed == side);
                      painter.circle(middle, active ? 7.0f : 5.0f, theme.accent);
                      painter.circle(middle, active ? 3.0f : 2.0f, theme.acrylic);
                  }
                  painter.pop_clip();
              }));
    card(ui, tree, page, document, models, vehicle);
    paint_card(ui, tree, page, document, models, vehicle);
}

void VehicleEditor::card(Ui& ui, Tree<Message>& tree, Rect page, const Document& document, const Models& models, size_t vehicle) const {
    Card frame = Card("Vehicles").subtitle("How each looks and drives").floating();
    tree.sized_card(ui, std::nullopt, editing::left_card(page), std::move(frame), [&](Ui& inner, Tree<Message>& items, CardArea area) {
        Form form(area.content);
        std::vector<std::string> names;
        names.reserve(document.vehicles.list.size());
        for (const Vehicle& listed : document.vehicles.list) {
            names.push_back(listed.name);
        }
        float shown = static_cast<float>(std::clamp(names.size(), size_t{1}, ListRowCount));
        ListView<Message> list = ListView<Message>(names.size(), ListItemHeight)
                                     .selection(vehicle)
                                     .on_select([](size_t index) { return Message::Vehicle{VehicleMsg::Select{index}}; })
                                     .labels([labels = names](size_t row) { return labels[row]; });
        Rect list_rect = form.take(shown * ListItemHeight);
        ListRows rows = list.rows(inner, "Vehicles/List", list_rect);
        items.container("Vehicles/List", list_rect, std::move(list), [&](Tree<Message>& listed) {
            for (size_t row : rows.indices()) {
                listed.draw(rows.content(row), Text::label(names[row]));
            }
        });
        auto [add, remove] = form.columns<2>(tokens::ControlHeight);
        items.add("Vehicles/Add", add, Button<Message>("Add a vehicle").icon(Icon::Add).on_press(Message::Vehicle{VehicleMsg::Add{}}))
            .set_tooltip("Adds a copy of this vehicle, with a name of its own");
        items.add("Vehicles/Delete", remove, Button<Message>("Delete it").icon(Icon::Delete).on_press(Message::Vehicle{VehicleMsg::Delete{}}))
            .set_tooltip("Deletes this vehicle, unless it's parked in a scene");
        if (!message.empty()) {
            InfoBar<Message> bar(Severity::Warning, "", message);
            float height = bar.height(inner, form.rest().width);
            items.add("Vehicles/Message", form.take(height), std::move(bar));
        }

        form.heading(items, "Vehicle");
        // The name, which a rename also changes where it's parked, and
        // the model. The field holds what's typed while it has focus,
        // and the vehicle's name otherwise, which Prepare keeps it on.
        const Vehicle& current = document.vehicles.list[vehicle];
        std::string shown_name = inner.is_focused("Vehicles/Name") ? name_text : current.name;
        auto [name_column, model_column] = form.columns<2>(HeadedHeight);
        Rect field = header(items, name_column, "Name");
        items
            .add("Vehicles/Name", field, TextBox<Message>(std::move(shown_name)).placeholder("Name").max_length(40).on_change([](std::string name) {
                return Message::Vehicle{VehicleMsg::Name{std::move(name)}};
            }))
            .set_focusable(true);
        std::vector<std::string_view> model_names = models.names();
        size_t model = models.find(current.model).value_or(0);
        Rect model_area = header(items, model_column, "Model");
        items.add("Vehicles/Model", model_area, ComboBox<Message>(model, model_names).on_select([](size_t chosen) { return Message::Vehicle{VehicleMsg::Model{chosen}}; }));
        SliderRange rates(0.0, 2000.0, 1.0, 0);
        const driving::Acceleration& acceleration = current.acceleration;
        std::array<double, 4> rate_values{acceleration.forward, acceleration.reverse, acceleration.left, acceleration.right};
        for (std::array<size_t, 2> row : {std::array<size_t, 2>{0, 1}, std::array<size_t, 2>{2, 3}}) {
            std::array<Rect, 2> columns = form.columns<2>(slider_height());
            for (size_t column = 0; column < 2; column++) {
                size_t index = row[column];
                items.add(std::format("Vehicles/{}", Rates[index]), columns[column],
                          editing::slider(Rates[index], rate_values[index], rates, [index](double value) { return Message::Vehicle{VehicleMsg::Rate{index, value}}; }));
            }
        }
        form.caption(inner, items, "Acceleration each way, in units a second, a second.");

        form.heading(items, "Footprint");
        std::array<double, 4> side_values = sides(current.footprint);
        SliderRange range(0.05, 10.0, 200.0, 3);
        for (std::array<size_t, 2> row : {std::array<size_t, 2>{0, 1}, std::array<size_t, 2>{2, 3}}) {
            std::array<Rect, 2> columns = form.columns<2>(slider_height());
            for (size_t column = 0; column < 2; column++) {
                size_t side = row[column];
                items.add(std::format("Footprint/{}", Sides[side]), columns[column],
                          editing::slider(Sides[side], side_values[side], range, [side](double value) { return Message::Vehicle{VehicleMsg::Side{side, value}}; }));
            }
        }
        items.add("Footprint/Fit", form.take(tokens::ControlHeight), Button<Message>("Fit it to the model").on_press(Message::Vehicle{VehicleMsg::Fit{}}))
            .set_tooltip("Makes the footprint the rectangle around the model, seen from above");
        form.caption(inner, items, "Drag a side in the view to move it.");
        return form.bottom();
    });
}

void VehicleEditor::paint_card(Ui& ui, Tree<Message>& tree, Rect page, const Document& document, const Models& models, size_t vehicle) const {
    Card frame = Card("Paint").subtitle("This vehicle's, over its model's").floating();
    tree.sized_card(ui, std::nullopt, editing::right_card(page), std::move(frame), [&](Ui& inner, Tree<Message>& items, CardArea area) {
        Form form(area.content);
        const Vehicle& current = document.vehicles.list[vehicle];
        size_t model = models.find(current.model).value_or(0);
        const std::vector<std::string>& slots = models.list[model].slots;
        size_t shown_slot = std::min(slot, slots.size() - 1);
        const std::string& model_name = models.list[model].name;
        const models::Paint& model_paint = document.paint(model_name);
        std::vector<Parameters> materials = models.list[model].materials({{&model_paint, &current.paint}});
        editing::paint(
            form, items, "VehiclePaint", slots, shown_slot, materials[shown_slot], [](size_t chosen) { return Message::Vehicle{VehicleMsg::Slot{chosen}}; },
            [](Parameters parameters) { return Message::Vehicle{VehicleMsg::Paint{parameters}}; });
        bool painted = std::ranges::any_of(current.paint, [&](const std::pair<std::string, Parameters>& entry) { return entry.first == slots[shown_slot]; });
        Button<Message> reset = Button<Message>("Use the model's paint").icon(Icon::Refresh).on_press(Message::Vehicle{VehicleMsg::ResetPaint{}});
        items.add("VehiclePaint/Reset", form.take(tokens::ControlHeight), std::move(reset)).set_enabled(painted).set_tooltip("Paints this slot as the model is painted everywhere");
        form.caption(inner, items, painted ? "This vehicle paints this slot its own way." : "This slot has the model's paint.");
        return form.bottom();
    });
}

Value VehicleEditor::state(const Document& document, size_t vehicle) const {
    const Vehicle& current = document.vehicles.list[vehicle];
    std::array<double, 4> values = sides(current.footprint);
    Value::Array names;
    names.values.reserve(document.vehicles.list.size());
    for (const Vehicle& listed : document.vehicles.list) {
        names.values.emplace_back(listed.name);
    }
    Value::Array painted;
    painted.values.reserve(current.paint.size());
    for (const std::pair<std::string, Parameters>& entry : current.paint) {
        painted.values.emplace_back(entry.first);
    }
    return json::object({
        {"Selected", current.name},
        {"Model", current.model},
        {"Names", std::move(names)},
        {"Footprint", json::object({
                          {"Left", values[0]},
                          {"Right", values[1]},
                          {"Front", values[2]},
                          {"Back", values[3]},
                      })},
        {"PaintedSlots", std::move(painted)},
        {"Message", message},
    });
}

} // namespace veehiicuul::vehicle_editor
