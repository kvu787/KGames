#include "Veehiicuul/SceneEditor.h"

#include "Base/Numbers.h"
#include "Veehiicuul/Collision.h"
#include "Veehiicuul/Program.h"
#include "Veehiicuul/Scene.h"
#include "Veehiicuul/Track.h"

#include <algorithm>
#include <cmath>
#include <format>
#include <string_view>
#include <vector>

namespace veehiicuul::scene_editor {

namespace tokens = user_interface::tokens;

using collision::Footprint;
using document::Document;
using editing::GroundInput;
using editing::HeadedHeight;
using editing::header;
using editing::Projection;
using editing::slider_height;
using editing::SliderRange;
using editing::Viewport;
using editing::window_size;
using json::Value;
using models::Models;
using program::Message;
using scene::Decoration;
using scene::Parked;
using scene::Scene;
using simple_paint::Parameters;
using track::Pose;
using user_interface::Button;
using user_interface::Canvas;
using user_interface::Card;
using user_interface::CardArea;
using user_interface::ComboBox;
using user_interface::Form;
using user_interface::Icon;
using user_interface::ListRows;
using user_interface::ListView;
using user_interface::Painter;
using user_interface::Point;
using user_interface::Rect;
using user_interface::Text;
using user_interface::Theme;
using user_interface::Tree;
using user_interface::Ui;
using user_interface::controls::ListItemHeight;

namespace {

// The most rows the list shows without scrolling.
constexpr size_t ListRowCount = 6;

// A thing's footprint, where it is, and its scale.
struct Thing {
    Footprint footprint;
    Pose pose;
    double scale;
};

// The rectangle that a thing takes on the ground, as its corners.
std::array<std::array<double, 2>, 4> corners(Footprint footprint, const Pose& pose, double scale) {
    Footprint scaled{
        .min_ = {footprint.min_[0] * scale, footprint.min_[1] * scale},
        .max_ = {footprint.max_[0] * scale, footprint.max_[1] * scale},
    };
    return scaled.corners(pose.position, pose.radians());
}

// Whether a point is in a thing's rectangle.
bool under(Footprint footprint, const Pose& pose, double scale, std::array<double, 2> point) {
    double angle = pose.radians();
    double sin = std::sin(angle);
    double cos = std::cos(angle);
    double dx = point[0] - pose.position[0];
    double dz = point[1] - pose.position[1];
    double left = (dx * cos - dz * sin) / scale;
    double ahead = (dx * sin + dz * cos) / scale;
    return footprint.min_[0] <= left && left <= footprint.max_[0] && footprint.min_[1] <= ahead && ahead <= footprint.max_[1];
}

// The decorations, then the parked vehicles, as rows, with their footprints.
std::vector<Thing> footprints(const Document& document, const Models& models, size_t scene) {
    const Scene& current = document.scenes[scene];
    Footprint unknown = Footprint::from_sides(1.0, 1.0, 1.0, 1.0);
    std::vector<Thing> things;
    things.reserve(current.decorations.size() + current.parked.size());
    for (const Decoration& decoration : current.decorations) {
        std::optional<size_t> found = models.find(decoration.model);
        things.push_back(Thing{found ? models.list[*found].footprint : unknown, decoration.pose, decoration.scale});
    }
    for (const Parked& parked : current.parked) {
        std::optional<size_t> found = document.vehicles.find(parked.vehicle);
        things.push_back(Thing{found ? document.vehicles.list[*found].footprint : unknown, parked.pose, 1.0});
    }
    return things;
}

} // namespace

std::string describe_value(const Picked& picked) {
    return base::match(
        picked, [](const Picked::Decoration& decoration) { return std::format("Decoration({})", decoration.index); },
        [](const Picked::Parked& parked) { return std::format("Parked({})", parked.index); });
}

void SceneEditor::keep_picked(const Document& document, size_t scene) {
    const Scene& current = document.scenes[scene];
    if (picked) {
        bool there = base::match(
            *picked, [&](const Picked::Decoration& decoration) { return decoration.index < current.decorations.size(); },
            [&](const Picked::Parked& parked) { return parked.index < current.parked.size(); });
        if (!there) {
            picked = std::nullopt;
        }
    }
    if (!picked) {
        grab = std::nullopt;
    }
}

Picked SceneEditor::row_of(const Document& document, size_t scene, size_t row) const {
    size_t decorations = document.scenes[scene].decorations.size();
    if (row < decorations) {
        return Picked::Decoration{row};
    }
    return Picked::Parked{row - decorations};
}

void SceneEditor::view_input(const GroundInput& input, Document& document, const Models& models, size_t scene) {
    keep_picked(document, scene);
    std::vector<Thing> things = footprints(document, models, scene);
    if (input.pressed && !input.panning && input.ground) {
        std::array<double, 2> ground = *input.ground;
        std::optional<size_t> row;
        for (size_t candidate = things.size(); candidate-- > 0;) {
            const Thing& thing = things[candidate];
            if (under(thing.footprint, thing.pose, thing.scale, ground)) {
                row = candidate;
                break;
            }
        }
        picked = row ? std::optional(row_of(document, scene, *row)) : std::nullopt;
        if (row) {
            std::array<double, 2> position = things[*row].pose.position;
            grab = std::array<double, 2>{position[0] - ground[0], position[1] - ground[1]};
        } else {
            grab = std::nullopt;
        }
    }
    // The drag follows the pointer to where it's let go of.
    if (grab && input.ground && picked) {
        std::array<double, 2> position{(*input.ground)[0] + (*grab)[0], (*input.ground)[1] + (*grab)[1]};
        Scene& current = document.scenes[scene];
        base::match(
            *picked, [&](const Picked::Decoration& decoration) { current.decorations[decoration.index].pose.position = position; },
            [&](const Picked::Parked& parked) { current.parked[parked.index].pose.position = position; });
    }
    if (!input.held) {
        grab = std::nullopt;
    }
}

void SceneEditor::apply(const SceneMsg& message, Document& document, const Models& models, size_t scene) {
    keep_picked(document, scene);
    base::match(
        message, [&](const SceneMsg::Pick& pick) { picked = row_of(document, scene, pick.row); }, [&](const SceneMsg::Model& chosen) { model = chosen.model; },
        [&](const SceneMsg::Vehicle& chosen) { vehicle = chosen.vehicle; },
        [&](const SceneMsg::Add& add) {
            std::vector<std::string_view> model_names = models.names();
            size_t index = std::min(model, base::saturating_sub(model_names.size(), size_t{1}));
            Scene& current = document.scenes[scene];
            current.decorations.push_back(Decoration{
                .model = std::string(model_names[index]),
                .pose =
                    Pose{
                        .position = add.center,
                        .heading = 0.0,
                    },
                .scale = 1.0,
            });
            picked = Picked::Decoration{current.decorations.size() - 1};
        },
        [&](const SceneMsg::Park& park) {
            size_t index = std::min(vehicle, document.vehicles.list.size() - 1);
            std::string name = document.vehicles.list[index].name;
            Scene& current = document.scenes[scene];
            current.parked.push_back(Parked{
                .vehicle = std::move(name),
                .pose =
                    Pose{
                        .position = park.center,
                        .heading = 0.0,
                    },
            });
            picked = Picked::Parked{current.parked.size() - 1};
        },
        [&](const SceneMsg::Delete&) {
            Scene& current = document.scenes[scene];
            if (picked) {
                if (const Picked::Decoration* decoration = base::as<Picked::Decoration>(*picked); decoration != nullptr && decoration->index < current.decorations.size()) {
                    current.decorations.erase(current.decorations.begin() + static_cast<std::ptrdiff_t>(decoration->index));
                } else if (const Picked::Parked* parked = base::as<Picked::Parked>(*picked); parked != nullptr && parked->index < current.parked.size()) {
                    current.parked.erase(current.parked.begin() + static_cast<std::ptrdiff_t>(parked->index));
                }
            }
            picked = std::nullopt;
        },
        [&](const SceneMsg::Camera& camera) {
            Scene& current = document.scenes[scene];
            current.camera_center = camera.center;
            current.camera_height = camera.height;
        },
        [&](const SceneMsg::Background& background) { document.scenes[scene].background = background.color; },
        [&](const SceneMsg::FollowHeight& follow) { document.scenes[scene].follow_height = follow.height; },
        [&](const SceneMsg::Heading& turned) {
            Scene& current = document.scenes[scene];
            if (picked) {
                base::match(
                    *picked, [&](const Picked::Decoration& decoration) { current.decorations[decoration.index].pose.heading = turned.heading; },
                    [&](const Picked::Parked& parked) { current.parked[parked.index].pose.heading = turned.heading; });
            }
        },
        [&](const SceneMsg::Scale& scaled) {
            if (picked) {
                if (const Picked::Decoration* decoration = base::as<Picked::Decoration>(*picked)) {
                    document.scenes[scene].decorations[decoration->index].scale = scaled.scale;
                }
            }
        },
        [&](const SceneMsg::Slot& chosen) { slot = chosen.slot; },
        [&](const SceneMsg::Paint& paint) {
            if (!picked) {
                return;
            }
            const Picked::Decoration* decoration = base::as<Picked::Decoration>(*picked);
            if (decoration == nullptr) {
                return;
            }
            std::optional<size_t> found = models.find(document.scenes[scene].decorations[decoration->index].model);
            if (!found) {
                return;
            }
            const std::string& name = models.list[*found].name;
            const std::vector<std::string>& slots = models.list[*found].slots;
            const std::string& painted = slots[std::min(slot, slots.size() - 1)];
            models::set_slot(document.paint_mut(name), slots, painted, paint.parameters);
        });
}

void SceneEditor::view(Ui& ui, Tree<Message>& tree, Rect page, Rect view_rect, const Viewport& viewport, const Document& document, const Models& models, size_t scene) const {
    // Each thing's place, and the picked one's rectangle.
    std::vector<Thing> things = footprints(document, models, scene);
    struct Mark {
        std::array<std::array<double, 2>, 4> corners;
        std::array<double, 2> position;
        bool chosen;
    };
    std::vector<Mark> marks;
    marks.reserve(things.size());
    for (size_t row = 0; row < things.size(); row++) {
        const Thing& thing = things[row];
        marks.push_back(Mark{corners(thing.footprint, thing.pose, thing.scale), thing.pose.position, picked && *picked == row_of(document, scene, row)});
    }
    Projection projection(viewport.eye, window_size(ui));
    tree.draw(view_rect, Canvas([marks = std::move(marks), projection](Painter& painter, Rect rect) {
                  const Theme& theme = painter.theme();
                  painter.push_clip(rect);
                  for (const Mark& mark : marks) {
                      if (mark.chosen) {
                          projection.outline(painter, mark.corners, 2.0f, theme.accent);
                      }
                      Point center = projection.point(mark.position);
                      painter.circle(center, 5.0f, mark.chosen ? theme.accent : theme.text_primary);
                      painter.circle(center, 2.0f, theme.acrylic);
                  }
                  painter.pop_clip();
              }));
    card(ui, tree, page, view_rect, viewport, document, models, scene);
}

void SceneEditor::card(Ui& ui, Tree<Message>& tree, Rect page, Rect view_rect, const Viewport& viewport, const Document& document, const Models& models, size_t scene) const {
    const std::string& name = document.scenes[scene].name;
    Point middle = editing::visible(view_rect).center();
    std::array<double, 2> center = viewport.eye.to_ground({middle.x, middle.y}, window_size(ui));
    SceneMsg::Camera camera{
        .center = center,
        .height = viewport.eye.height,
    };
    tree.sized_card(ui, std::nullopt, editing::left_card(page), Card("Scene").subtitle(name).floating(), [&](Ui& inner, Tree<Message>& items, CardArea area) {
        Form form(area.content);
        std::vector<std::string_view> vehicle_names;
        vehicle_names.reserve(document.vehicles.list.size());
        for (const vehicles::Vehicle& listed : document.vehicles.list) {
            vehicle_names.push_back(listed.name);
        }
        std::vector<std::string_view> model_names = models.names();
        const Scene& current = document.scenes[scene];
        auto [background_column, follow_column] = form.columns<2>(HeadedHeight);
        Rect background = header(items, background_column, "Background");
        editing::color_field(items, "Scene/Background", background, current.background, [](files::Color color) { return Message::Scene{SceneMsg::Background{color}}; });
        SliderRange heights = SliderRange(5.0, 200.0, 1.0, 0).suffix(" tall");
        // Its header lines up with the field's.
        Rect follow = follow_column.top_strip(slider_height());
        items.add("Scene/FollowHeight", follow,
                  editing::slider("Following view", current.follow_height, heights, [](double value) { return Message::Scene{SceneMsg::FollowHeight{value}}; }));
        Button<Message> button = Button<Message>("Make this view the fixed camera's").icon(Icon::Camera).on_press(Message::Scene{camera});
        items.add("Scene/Camera", form.take(tokens::ControlHeight), std::move(button)).set_tooltip("The fixed camera looks where the middle of the view right of this card is");

        form.heading(items, "Decorations and parked vehicles");
        std::vector<std::string> rows;
        rows.reserve(current.decorations.size() + current.parked.size());
        for (const Decoration& decoration : current.decorations) {
            rows.push_back(decoration.model);
        }
        for (const Parked& parked : current.parked) {
            rows.push_back(std::format("{}, parked", parked.vehicle));
        }
        std::optional<size_t> picked_row;
        if (picked) {
            picked_row = base::match(
                *picked, [](const Picked::Decoration& decoration) { return decoration.index; },
                [&](const Picked::Parked& parked) { return current.decorations.size() + parked.index; });
        }
        float shown = static_cast<float>(std::clamp(rows.size(), size_t{1}, ListRowCount));
        ListView<Message> list = ListView<Message>(rows.size(), ListItemHeight)
                                     .selection(picked_row)
                                     .on_select([](size_t row) { return Message::Scene{SceneMsg::Pick{row}}; })
                                     .labels([labels = rows](size_t row) { return labels[row]; });
        Rect list_rect = form.take(shown * ListItemHeight);
        ListRows shown_rows = list.rows(inner, "Scene/Objects", list_rect);
        items.container("Scene/Objects", list_rect, std::move(list), [&](Tree<Message>& listed) {
            for (size_t row : shown_rows.indices()) {
                listed.draw(shown_rows.content(row), Text::label(rows[row]));
            }
        });
        auto [model_column, vehicle_column] = form.columns<2>(HeadedHeight);
        Rect model_area = header(items, model_column, "Model");
        Rect vehicle_area = header(items, vehicle_column, "Vehicle");
        items.add("Scene/Model", model_area, ComboBox<Message>(std::min(model, base::saturating_sub(model_names.size(), size_t{1})), model_names).on_select([](size_t chosen) {
            return Message::Scene{SceneMsg::Model{chosen}};
        }));
        items.add("Scene/Vehicle", vehicle_area,
                  ComboBox<Message>(std::min(vehicle, vehicle_names.size() - 1), vehicle_names).on_select([](size_t chosen) { return Message::Scene{SceneMsg::Vehicle{chosen}}; }));
        auto [add, park, remove] = form.columns<3>(tokens::ControlHeight);
        items.add("Scene/AddDecoration", add, Button<Message>("Add the model").icon(Icon::Add).on_press(Message::Scene{SceneMsg::Add{center}}))
            .set_tooltip("Puts the model in the middle of the view");
        items.add("Scene/Park", park, Button<Message>("Park the vehicle").icon(Icon::Car).on_press(Message::Scene{SceneMsg::Park{center}}))
            .set_tooltip("Parks the vehicle in the middle of the view");
        items.add("Scene/Delete", remove, Button<Message>("Delete").icon(Icon::Delete).on_press(Message::Scene{SceneMsg::Delete{}})).set_tooltip("Deletes the one that's picked");

        SliderRange degrees = SliderRange(-180.0, 180.0, 10.0, 1).suffix("°");
        std::optional<size_t> paint_model;
        if (!picked) {
            form.caption(inner, items, "Press one in the view, or in the list, to pick it.");
        } else if (const Picked::Decoration* chosen = base::as<Picked::Decoration>(*picked)) {
            const Decoration& decoration = current.decorations[chosen->index];
            form.heading(items, std::format("The {}", decoration.model));
            auto [heading, scale] = form.columns<2>(slider_height());
            items.add("Scene/Heading", heading,
                      editing::slider("Heading", decoration.pose.heading, degrees, [](double value) { return Message::Scene{SceneMsg::Heading{value}}; }));
            SliderRange scales(scene::SmallestScale, scene::LargestScale, 100.0, 2);
            items.add("Scene/Scale", scale, editing::slider("Scale", decoration.scale, scales, [](double value) { return Message::Scene{SceneMsg::Scale{value}}; }));
            paint_model = models.find(decoration.model);
        } else if (const Picked::Parked* parked = base::as<Picked::Parked>(*picked)) {
            const Parked& placed = current.parked[parked->index];
            form.heading(items, std::format("The parked {}", placed.vehicle));
            editing::slider_row(form, items, "Scene/Heading", "Heading", placed.pose.heading, degrees, [](double value) { return Message::Scene{SceneMsg::Heading{value}}; });
            form.caption(inner, items, "The Vehicles tab paints vehicles.");
        }
        if (paint_model) {
            const std::string& model_name = models.list[*paint_model].name;
            const std::vector<std::string>& slots = models.list[*paint_model].slots;
            size_t shown_slot = std::min(slot, slots.size() - 1);
            Parameters parameters = document.model_materials(models, *paint_model)[shown_slot];
            form.heading(items, std::format("The {}'s paint, everywhere it is", model_name));
            editing::paint(
                form, items, "ModelPaint", slots, shown_slot, parameters, [](size_t chosen) { return Message::Scene{SceneMsg::Slot{chosen}}; },
                [](Parameters painted) { return Message::Scene{SceneMsg::Paint{painted}}; });
        }
        return form.bottom();
    });
}

Value SceneEditor::state(const Document& document, size_t scene) const {
    const Scene& current = document.scenes[scene];
    Value chosen;
    if (picked) {
        chosen = base::match(
            *picked, [](const Picked::Decoration& decoration) { return Value(std::format("Decoration {}", decoration.index)); },
            [](const Picked::Parked& parked) { return Value(std::format("Parked {}", parked.index)); });
    }
    Value::Array decorations;
    decorations.values.reserve(current.decorations.size());
    for (const Decoration& decoration : current.decorations) {
        decorations.values.push_back(json::object({
            {"Model", decoration.model},
            {"Pose", decoration.pose.to_value()},
            {"Scale", decoration.scale},
        }));
    }
    Value::Array parked_vehicles;
    parked_vehicles.values.reserve(current.parked.size());
    for (const Parked& parked : current.parked) {
        parked_vehicles.values.push_back(json::object({{"Vehicle", parked.vehicle}, {"Pose", parked.pose.to_value()}}));
    }
    return json::object({
        {"Picked", std::move(chosen)},
        {"Decorations", std::move(decorations)},
        {"Parked", std::move(parked_vehicles)},
        {"Background", files::color_text(current.background)},
        {"CameraHeight", current.camera_height},
    });
}

} // namespace veehiicuul::scene_editor
