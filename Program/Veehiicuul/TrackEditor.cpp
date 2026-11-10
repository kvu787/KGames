#include "Veehiicuul/TrackEditor.h"

#include "Veehiicuul/Curve.h"
#include "Veehiicuul/Program.h"

#include <algorithm>
#include <format>
#include <vector>

namespace veehiicuul::track_editor {

namespace tokens = user_interface::tokens;

using camera::Eye;
using editing::GroundInput;
using editing::HeadedHeight;
using editing::header;
using editing::nearest;
using editing::Projection;
using editing::slider_height;
using editing::SliderRange;
using editing::Viewport;
using json::Value;
using program::Message;
using track::Track;
using user_interface::Button;
using user_interface::Canvas;
using user_interface::Card;
using user_interface::CardArea;
using user_interface::ComboBox;
using user_interface::Form;
using user_interface::Icon;
using user_interface::InfoBar;
using user_interface::Painter;
using user_interface::Rect;
using user_interface::Segmented;
using user_interface::Severity;
using user_interface::Theme;
using user_interface::Tree;
using user_interface::Ui;

namespace {

constexpr std::array<Icon, 4> ToolIcons{Icon::Pin, Icon::Add, Icon::Delete, Icon::Move};
// How near the pointer a point counts as under it, in pixels.
constexpr float Reach = 12.0f;
// How far ahead of the start its handle is, in units.
constexpr double Handle = 8.0;
// The most problems the card lists.
constexpr size_t MostProblems = 4;

// Where the start's handle is, which turns it.
std::array<double, 2> handle(const Track& track) {
    std::array<double, 2> ahead = track.start.ahead();
    return {track.start.position[0] + ahead[0] * Handle, track.start.position[1] + ahead[1] * Handle};
}

// The first control point near the pointer, as its curve and its index.
std::optional<std::pair<size_t, size_t>> point_under(const Track& track, const Eye& eye, std::array<float, 2> size, std::array<float, 2> cursor) {
    size_t index = 0;
    for (const std::vector<std::array<double, 2>>& points : track.curves()) {
        if (std::optional<size_t> found = nearest(eye, size, cursor, points, Reach)) {
            return std::pair(index, *found);
        }
        index += 1;
    }
    return std::nullopt;
}

} // namespace

std::string curve_name(size_t index) {
    return index == 0 ? std::string("Road") : std::format("Island {}", index);
}

void TrackEditor::view_input(const GroundInput& input, Track& track, Eye eye, std::array<float, 2> size) {
    if (input.pressed && !input.panning && input.cursor && input.ground) {
        std::array<float, 2> cursor = *input.cursor;
        std::array<double, 2> ground = *input.ground;
        if (tool == 0) {
            std::array<std::array<double, 2>, 1> turner{handle(track)};
            std::array<std::array<double, 2>, 1> start{track.start.position};
            if (nearest(eye, size, cursor, turner, Reach)) {
                drag = Drag::Heading{};
            } else if (nearest(eye, size, cursor, start, Reach * 2.0f)) {
                drag = Drag::Start{};
            } else if (std::optional<std::pair<size_t, size_t>> found = point_under(track, eye, size, cursor)) {
                drag = Drag::Point{found->first, found->second};
            } else {
                drag = std::nullopt;
            }
            if (drag) {
                if (const Drag::Point* point = base::as<Drag::Point>(*drag)) {
                    curve = point->curve;
                }
            }
        } else if (tool == 1) {
            size_t chosen = curve;
            if (std::vector<std::array<double, 2>>* points = track.curve_mut(chosen)) {
                size_t index = std::min(curve::insertion(*points, ground), points->size());
                points->insert(points->begin() + static_cast<std::ptrdiff_t>(index), ground);
                drag = Drag::Point{chosen, index};
            }
        } else if (tool == 2) {
            if (std::optional<std::pair<size_t, size_t>> found = point_under(track, eye, size, cursor)) {
                std::vector<std::array<double, 2>>* points = track.curve_mut(found->first);
                if (points != nullptr && points->size() > curve::Fewest) {
                    points->erase(points->begin() + static_cast<std::ptrdiff_t>(found->second));
                    curve = found->first;
                }
            }
        } else {
            drag = Drag::Track{ground};
        }
    }
    if (drag && input.ground) {
        std::array<double, 2> ground = *input.ground;
        // A copy, as moving the track makes the drag anew.
        Drag moved = *drag;
        base::match(
            moved,
            [&](const Drag::Point& point) {
                std::vector<std::array<double, 2>>* points = track.curve_mut(point.curve);
                if (points != nullptr && point.index < points->size()) {
                    (*points)[point.index] = ground;
                }
            },
            [&](const Drag::Start&) { track.start.position = ground; }, [&](const Drag::Heading&) { track.start.heading = editing::heading_toward(track.start.position, ground); },
            [&](const Drag::Track& grabbed) {
                std::array<double, 2> shift{ground[0] - grabbed.grabbed[0], ground[1] - grabbed.grabbed[1]};
                for (size_t index = 0; index <= track.islands.size(); index++) {
                    for (std::array<double, 2>& point : *track.curve_mut(index)) {
                        point[0] += shift[0];
                        point[1] += shift[1];
                    }
                }
                track.start.position[0] += shift[0];
                track.start.position[1] += shift[1];
                drag = Drag::Track{ground};
            });
    }
    // The drag followed the pointer to where it was let go of.
    if (!input.held) {
        drag = std::nullopt;
    }
    under = std::nullopt;
    if (input.cursor && !input.panning) {
        if (std::optional<std::pair<size_t, size_t>> found = point_under(track, eye, size, *input.cursor)) {
            auto curves = track.curves();
            if (found->first < curves.size()) {
                under = curves[found->first][found->second];
            }
        }
    }
}

void TrackEditor::apply(const TrackMsg& message, Track& track) {
    base::match(
        message, [&](const TrackMsg::Tool& chosen) { tool = chosen.tool; }, [&](const TrackMsg::Curve& chosen) { curve = chosen.curve; },
        [&](const TrackMsg::AddIsland& island) {
            track.islands.push_back(curve::circle(island.center, island.radius, 8));
            curve = track.islands.size();
        },
        [&](const TrackMsg::DeleteIsland&) {
            // Only while the choice is still an island: undo can take
            // one away between the frame's messages.
            if (curve > 0 && curve <= track.islands.size()) {
                track.islands.erase(track.islands.begin() + static_cast<std::ptrdiff_t>(curve - 1));
                curve -= 1;
            }
        },
        [&](const TrackMsg::Heading& turned) { track.start.heading = turned.heading; }, [&](const TrackMsg::BarrierWidth& barrier) { track.barrier_width = barrier.width; },
        [&](const TrackMsg::BarrierLength& barrier) { track.barrier_length = barrier.length; }, [&](const TrackMsg::RoadColor& road) { track.road_color = road.color; },
        [&](const TrackMsg::BarrierColor& barrier) { track.barrier_colors[barrier.index] = barrier.color; });
    curve = std::min(curve, track.islands.size());
}

void TrackEditor::view(Ui& ui, Tree<Message>& tree, Rect page, Rect view_rect, const Viewport& viewport, const Track& track, std::span<const std::string> problems) const {
    overlay(ui, tree, view_rect, viewport, track);
    card(ui, tree, page, view_rect, viewport, track, problems);
}

void TrackEditor::overlay(Ui& ui, Tree<Message>& tree, Rect view_rect, const Viewport& viewport, const Track& track) const {
    Projection projection(viewport.eye, editing::window_size(ui));
    std::vector<std::vector<std::array<double, 2>>> curves;
    curves.reserve(track.islands.size() + 1);
    for (const std::vector<std::array<double, 2>>& points : track.curves()) {
        curves.push_back(points);
    }
    size_t selected = curve;
    std::array<double, 2> start = track.start.position;
    std::array<double, 2> turner = handle(track);
    std::optional<std::array<double, 2>> ringed = under;
    tree.draw(view_rect, Canvas([curves = std::move(curves), projection, selected, start, turner, ringed](Painter& painter, Rect rect) {
                  const Theme& theme = painter.theme();
                  painter.push_clip(rect);
                  for (size_t index = 0; index < curves.size(); index++) {
                      const std::vector<std::array<double, 2>>& points = curves[index];
                      bool picked = index == selected;
                      user_interface::Color color = picked ? theme.accent : theme.text_primary;
                      projection.outline(painter, points, picked ? 1.5f : 1.0f, color);
                      for (std::array<double, 2> point : points) {
                          user_interface::Point center = projection.point(point);
                          painter.circle(center, picked ? 5.0f : 4.0f, color);
                          painter.circle(center, 2.0f, theme.acrylic);
                      }
                  }
                  user_interface::Point from = projection.point(start);
                  user_interface::Point end = projection.point(turner);
                  painter.line(from, end, 2.0f, theme.text_primary);
                  painter.circle(end, 7.0f, theme.text_primary);
                  painter.circle(end, 4.0f, theme.acrylic);
                  if (ringed) {
                      painter.ring(projection.point(*ringed), 9.0f, 2.0f, theme.accent);
                  }
                  painter.pop_clip();
              }));
}

void TrackEditor::card(Ui& ui, Tree<Message>& tree, Rect page, Rect view_rect, const Viewport& viewport, const Track& track, std::span<const std::string> problems) const {
    Card frame = Card("Track").subtitle("Curves that make the road and its barriers").floating();
    user_interface::Point middle = editing::visible(view_rect).center();
    std::array<double, 2> island_center = viewport.eye.to_ground({middle.x, middle.y}, editing::window_size(ui));
    double island_radius = viewport.eye.height / 12.0;
    tree.sized_card(ui, std::nullopt, editing::left_card(page), std::move(frame), [&](Ui& inner, Tree<Message>& items, CardArea area) {
        Form form(area.content);
        std::array<std::pair<Icon, std::string_view>, 4> tools;
        for (size_t index = 0; index < tools.size(); index++) {
            tools[index] = {ToolIcons[index], Tools[index]};
        }
        items.add("Track/Tool", form.take(tokens::ControlHeight), Segmented<Message>(tool, tools).on_select([](size_t chosen) { return Message::Track{TrackMsg::Tool{chosen}}; }));

        form.heading(items, "Curves");
        size_t shown = std::min(curve, track.islands.size());
        std::vector<std::string> names;
        names.reserve(track.islands.size() + 1);
        for (size_t index = 0; index <= track.islands.size(); index++) {
            names.push_back(curve_name(index));
        }
        Rect curve_area = form.control(items, "Curve");
        items.add("Track/Curve", curve_area, ComboBox<Message>(shown, names).on_select([](size_t chosen) { return Message::Track{TrackMsg::Curve{chosen}}; }));
        auto [add, remove] = form.columns<2>(tokens::ControlHeight);
        TrackMsg::AddIsland island{
            .center = island_center,
            .radius = island_radius,
        };
        items.add("Track/AddIsland", add, Button<Message>("Add an island").icon(Icon::Add).on_press(Message::Track{island}))
            .set_tooltip("Adds a circle of 8 points in the middle of the view");
        items.add("Track/DeleteIsland", remove, Button<Message>("Delete the island").icon(Icon::Delete).on_press(Message::Track{TrackMsg::DeleteIsland{}}))
            .set_enabled(shown > 0)
            .set_tooltip("Deletes the chosen island");
        auto curves = track.curves();
        size_t count = shown < curves.size() ? curves[shown].size() : 0;
        form.caption(inner, items, std::format("{} has {} control points.", curve_name(shown), count));

        form.heading(items, "Start");
        SliderRange degrees = SliderRange(-180.0, 180.0, 10.0, 1).suffix("°");
        editing::slider_row(form, items, "Track/Heading", "Heading", track.start.heading, degrees, [](double value) { return Message::Track{TrackMsg::Heading{value}}; });
        form.caption(inner, items, "Drag the car, or its handle, to move or turn it.");

        form.heading(items, "Barriers");
        auto [width, length] = form.columns<2>(slider_height());
        SliderRange widths(0.1, 5.0, 100.0, 2);
        items.add("Track/BarrierWidth", width, editing::slider("Width", track.barrier_width, widths, [](double value) { return Message::Track{TrackMsg::BarrierWidth{value}}; }));
        SliderRange lengths(1.0, 50.0, 10.0, 1);
        items.add("Track/BarrierLength", length,
                  editing::slider("Length", track.barrier_length, lengths, [](double value) { return Message::Track{TrackMsg::BarrierLength{value}}; }));

        form.heading(items, "Colors");
        auto [road_column, first_column, second] = form.columns<3>(HeadedHeight);
        Rect road = header(items, road_column, "Road");
        Rect first = header(items, first_column, "Barriers");
        editing::color_field(items, "Track/RoadColor", road, track.road_color, [](files::Color color) { return Message::Track{TrackMsg::RoadColor{color}}; });
        editing::color_field(items, "Track/BarrierColor0", first, track.barrier_colors[0], [](files::Color color) { return Message::Track{TrackMsg::BarrierColor{0, color}}; });
        editing::color_field(items, "Track/BarrierColor1", second.bottom_strip(tokens::ControlHeight), track.barrier_colors[1],
                             [](files::Color color) { return Message::Track{TrackMsg::BarrierColor{1, color}}; });

        form.heading(items, "Problems");
        if (problems.empty()) {
            InfoBar<Message> bar(Severity::Success, "", "None. The track can be driven.");
            float height = bar.height(inner, form.rest().width);
            items.add("Track/Drivable", form.take(height), std::move(bar));
        }
        for (size_t index = 0; index < std::min(problems.size(), MostProblems); index++) {
            InfoBar<Message> bar(Severity::Warning, "", problems[index]);
            float height = bar.height(inner, form.rest().width);
            items.add(std::format("Track/Problem/{}", index), form.take(height), std::move(bar));
        }
        if (problems.size() > MostProblems) {
            form.caption(inner, items, std::format("And {} more.", problems.size() - MostProblems));
        }
        return form.bottom();
    });
}

Value TrackEditor::state(const Track& track, std::span<const std::string> problems) const {
    Value::Array counts;
    counts.values.reserve(track.islands.size() + 1);
    for (const std::vector<std::array<double, 2>>& points : track.curves()) {
        counts.values.emplace_back(points.size());
    }
    return json::object({
        {"Tool", Tools[tool]},
        {"Curve", curve_name(curve)},
        {"ControlPoints", std::move(counts)},
        {"Start", track.start.to_value()},
        {"Problems", std::vector<std::string>(problems.begin(), problems.end())},
    });
}

} // namespace veehiicuul::track_editor
