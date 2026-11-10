#include "Veehiicuul/Track.h"

#include "Base/Sum.h"
#include "Base/Text.h"
#include "Veehiicuul/Angles.h"
#include "Veehiicuul/Curve.h"

#include <algorithm>
#include <cmath>
#include <format>
#include <optional>
#include <tuple>
#include <utility>

namespace veehiicuul::track {

using collision::Edges;
using collision::meet;
using json::Value;

namespace {

// How wide the start line is where it can't find the road's edges.
constexpr double StartLineWidth = 10.0;

constexpr std::array<std::string_view, 7> Members{"Start", "Colors", "BarrierWidth", "BarrierLength", "Road", "Islands", ""};

// How far along from `from` to `to` the segment a to b crosses it, in units,
// if it does.
std::optional<double> crossing(std::array<double, 2> from, std::array<double, 2> to, std::array<double, 2> a, std::array<double, 2> b) {
    std::array<double, 2> direction{to[0] - from[0], to[1] - from[1]};
    std::array<double, 2> edge{b[0] - a[0], b[1] - a[1]};
    double denominator = direction[0] * edge[1] - direction[1] * edge[0];
    if (std::abs(denominator) < 1e-12) {
        return std::nullopt;
    }
    std::array<double, 2> offset{a[0] - from[0], a[1] - from[1]};
    double t = (offset[0] * edge[1] - offset[1] * edge[0]) / denominator;
    double s = (offset[0] * direction[1] - offset[1] * direction[0]) / denominator;
    if (0.0 <= t && t <= 1.0 && 0.0 <= s && s <= 1.0) {
        return t * std::hypot(direction[0], direction[1]);
    }
    return std::nullopt;
}

// The start line goes across the road from edge to edge, StartLineAhead of
// the start: as far as the nearest edge on either side of its middle.
StartLine start_line(std::span<const std::span<const std::array<double, 2>>> outlines, const Pose& start) {
    std::array<double, 2> ahead = start.ahead();
    double sin = ahead[0];
    double cos = ahead[1];
    std::array<double, 2> center{
        start.position[0] + sin * StartLineAhead,
        start.position[1] + cos * StartLineAhead,
    };
    std::array<double, 2> across{cos, -sin};
    auto reach = [&](double direction) {
        std::array<double, 2> far_{center[0] + across[0] * direction * 1000.0, center[1] + across[1] * direction * 1000.0};
        std::optional<double> nearest;
        for (std::span<const std::array<double, 2>> outline : outlines) {
            for (size_t index = 0; index < outline.size(); index++) {
                std::array<double, 2> a = outline[index];
                std::array<double, 2> b = outline[(index + 1) % outline.size()];
                if (std::optional<double> distance = crossing(center, far_, a, b)) {
                    nearest = nearest ? std::min(*nearest, *distance) : *distance;
                }
            }
        }
        return nearest;
    };
    double width = StartLineWidth;
    if (std::optional<double> left = reach(1.0), right = reach(-1.0); left && right) {
        width = *left + *right;
    }
    // The middle moves to halfway between the edges, so the line spans the
    // road.
    double shift = 0.0;
    if (std::optional<double> left = reach(1.0), right = reach(-1.0); left && right) {
        shift = (*left - *right) / 2.0;
    }
    return StartLine{
        .center = {center[0] + across[0] * shift, center[1] + across[1] * shift},
        .width = width,
        .depth = StartLineDepth,
    };
}

std::pair<std::array<double, 2>, std::array<double, 2>> bounds(const std::array<std::array<double, 2>, 2>& segment) {
    const std::array<double, 2>& a = segment[0];
    const std::array<double, 2>& b = segment[1];
    return {{std::min(a[0], b[0]), std::min(a[1], b[1])}, {std::max(a[0], b[0]), std::max(a[1], b[1])}};
}

// Whether two outlines' edges cross, or, for one outline, whether edges that
// aren't next to each other do.
bool crosses(std::span<const std::array<double, 2>> a, std::span<const std::array<double, 2>> b, bool same) {
    auto segment = [](std::span<const std::array<double, 2>> outline, size_t index) {
        return std::array<std::array<double, 2>, 2>{outline[index], outline[(index + 1) % outline.size()]};
    };
    for (size_t i = 0; i < a.size(); i++) {
        std::array<std::array<double, 2>, 2> first = segment(a, i);
        auto [min_, max_] = bounds(first);
        for (size_t j = same ? i + 2 : 0; j < b.size(); j++) {
            if (same && i == 0 && j == b.size() - 1) {
                continue;
            }
            std::array<std::array<double, 2>, 2> second = segment(b, j);
            auto [other_min, other_max] = bounds(second);
            if (other_max[0] < min_[0] || other_min[0] > max_[0] || other_max[1] < min_[1] || other_min[1] > max_[1]) {
                continue;
            }
            if (meet(first, second)) {
                return true;
            }
        }
    }
    return false;
}

// What's wrong with a track's shape, for the track builder to say: edges
// that cross, islands outside the road or inside each other, and a start
// that isn't on the road.
std::vector<std::string> problems(std::span<const std::array<double, 2>> road, std::span<const std::vector<std::array<double, 2>>> islands, const Pose& start, const Edges& edges) {
    std::vector<std::string> found;
    // Each outline's kind, its number, and its points.
    using Outline = std::tuple<std::string_view, size_t, std::span<const std::array<double, 2>>>;
    std::vector<Outline> outlines{Outline{"The road", 0, road}};
    for (size_t index = 0; index < islands.size(); index++) {
        outlines.push_back(Outline{"Island", index + 1, islands[index]});
    }
    auto name = [](const Outline& outline) {
        std::string_view kind = std::get<0>(outline);
        size_t index = std::get<1>(outline);
        return index == 0 ? std::string(kind) : std::format("{} {}", kind, index);
    };
    for (size_t first = 0; first < outlines.size(); first++) {
        const Outline& a = outlines[first];
        for (size_t second = first; second < outlines.size(); second++) {
            const Outline& b = outlines[second];
            if (crosses(std::get<2>(a), std::get<2>(b), first == second)) {
                found.push_back(first == second ? std::format("{}'s edge crosses itself", name(a))
                                                : std::format("{}'s edge crosses {}'s", name(a), base::to_ascii_lowercase(name(b))));
            }
        }
    }
    std::span<const Outline> island_outlines = std::span<const Outline>(outlines).subspan(1);
    for (const Outline& island : island_outlines) {
        if (!inside(road, std::get<2>(island)[0])) {
            found.push_back(std::format("{} is outside the road", name(island)));
        }
    }
    // An island is inside another if its first point is, whichever of the
    // two comes first in the list. One point only decides it where their
    // edges don't cross, which is a problem of its own, above.
    for (const Outline& outer : island_outlines) {
        for (const Outline& inner : island_outlines) {
            if (std::get<1>(inner) != std::get<1>(outer) && inside(std::get<2>(outer), std::get<2>(inner)[0])) {
                found.push_back(std::format("{} is inside {}", name(inner), base::to_ascii_lowercase(name(outer))));
            }
        }
    }
    bool on_road = inside(road, start.position) && std::ranges::all_of(islands, [&](const std::vector<std::array<double, 2>>& island) { return !inside(island, start.position); });
    if (!on_road) {
        found.push_back("The start isn't on the road");
    }
    static_cast<void>(edges);
    return found;
}

} // namespace

double Pose::radians() const {
    return to_radians(heading);
}

std::array<double, 2> Pose::ahead() const {
    double angle = radians();
    double sin = std::sin(angle);
    double cos = std::cos(angle);
    return {sin, cos};
}

std::expected<Pose, std::string> Pose::parse(const Value& value, std::string_view name) {
    if (std::expected<void, std::string> known = files::only(value, {{"Position", "Heading"}}, name); !known) {
        return std::unexpected(std::move(known.error()));
    }
    std::expected<std::array<double, 2>, std::string> at =
        files::member(value, "Position", std::format("{}.", name)).and_then([&](const Value* found) { return files::point(*found, std::format("{}.Position", name)); });
    if (!at) {
        return std::unexpected(std::move(at.error()));
    }
    std::expected<double, std::string> facing =
        files::member(value, "Heading", std::format("{}.", name)).and_then([&](const Value* found) { return files::number(*found, std::format("{}.Heading", name)); });
    if (!facing) {
        return std::unexpected(std::move(facing.error()));
    }
    return Pose{
        .position = *at,
        .heading = *facing,
    };
}

Value Pose::to_value() const {
    return json::object({{"Position", files::point_value(position)}, {"Heading", files::rounded(heading)}});
}

std::string describe_value(const Pose& pose) {
    return std::format("Pose {{ position: [{}, {}], heading: {} }}", base::debug(pose.position[0]), base::debug(pose.position[1]), base::debug(pose.heading));
}

Shape::Shape(std::vector<std::array<double, 2>> road_outline, std::vector<std::vector<std::array<double, 2>>> island_outlines, Edges outline_edges, StartLine line,
             std::vector<std::string> shape_problems) :
    road(std::move(road_outline)), islands(std::move(island_outlines)), edges(std::move(outline_edges)), start_line(line), problems(std::move(shape_problems)) {}

Track Track::round() {
    return Track{
        .road = curve::circle({0.0, 0.0}, 40.0, 12),
        .islands = {curve::circle({0.0, 0.0}, 25.0, 10)},
        .start =
            Pose{
                .position = {32.5, 0.0},
                .heading = 180.0,
            },
        .road_color = {0x81, 0x85, 0x8B},
        .barrier_colors = {{{0xF6, 0x1E, 0x22}, {0xF6, 0xFC, 0xFF}}},
        .barrier_width = 1.5,
        .barrier_length = 15.0,
    };
}

std::expected<Track, std::string> Track::parse(const Value& value) {
    if (std::expected<void, std::string> known = files::only(value, std::span(Members).first(6), "the track"); !known) {
        return std::unexpected(std::move(known.error()));
    }
    auto member = [&](std::string_view name) {
        return files::member(value, name, "Track.");
    };
    std::expected<const Value*, std::string> colors = member("Colors");
    if (!colors) {
        return std::unexpected(std::move(colors.error()));
    }
    if (std::expected<void, std::string> known = files::only(**colors, {{"Road", "Barriers"}}, "Track.Colors"); !known) {
        return std::unexpected(std::move(known.error()));
    }
    std::expected<const Value*, std::string> barriers_member = files::member(**colors, "Barriers", "Track.Colors.");
    if (!barriers_member) {
        return std::unexpected(std::move(barriers_member.error()));
    }
    std::array<files::Color, 2> barriers{};
    if (const Value::Array* items = base::as<Value::Array>(**barriers_member); items != nullptr && items->values.size() == 2) {
        for (size_t index = 0; index < 2; index++) {
            std::expected<files::Color, std::string> color = files::color(items->values[index], std::format("Track.Colors.Barriers[{}]", index));
            if (!color) {
                return std::unexpected(std::move(color.error()));
            }
            barriers[index] = *color;
        }
    } else {
        return std::unexpected(std::string("Track.Colors.Barriers must be a list of 2 colors, which the barriers alternate between"));
    }
    std::expected<const Value*, std::string> islands_member = member("Islands");
    if (!islands_member) {
        return std::unexpected(std::move(islands_member.error()));
    }
    std::vector<std::vector<std::array<double, 2>>> island_curves;
    if (const Value::Array* items = base::as<Value::Array>(**islands_member)) {
        island_curves.reserve(items->values.size());
        for (size_t index = 0; index < items->values.size(); index++) {
            std::expected<std::vector<std::array<double, 2>>, std::string> island = files::points(items->values[index], std::format("Track.Islands[{}]", index), curve::Fewest);
            if (!island) {
                return std::unexpected(std::move(island.error()));
            }
            island_curves.push_back(std::move(*island));
        }
    } else {
        return std::unexpected(std::format("Track.Islands must be a list of curves, not {}", (*islands_member)->describe()));
    }
    std::expected<std::vector<std::array<double, 2>>, std::string> road_curve =
        member("Road").and_then([](const Value* found) { return files::points(*found, "Track.Road", curve::Fewest); });
    if (!road_curve) {
        return std::unexpected(std::move(road_curve.error()));
    }
    std::expected<Pose, std::string> start_pose = member("Start").and_then([](const Value* found) { return Pose::parse(*found, "Track.Start"); });
    if (!start_pose) {
        return std::unexpected(std::move(start_pose.error()));
    }
    std::expected<files::Color, std::string> road_paint =
        files::member(**colors, "Road", "Track.Colors.").and_then([](const Value* found) { return files::color(*found, "Track.Colors.Road"); });
    if (!road_paint) {
        return std::unexpected(std::move(road_paint.error()));
    }
    std::expected<double, std::string> width = member("BarrierWidth").and_then([](const Value* found) { return files::positive(*found, "Track.BarrierWidth"); });
    if (!width) {
        return std::unexpected(std::move(width.error()));
    }
    std::expected<double, std::string> length = member("BarrierLength").and_then([](const Value* found) { return files::positive(*found, "Track.BarrierLength"); });
    if (!length) {
        return std::unexpected(std::move(length.error()));
    }
    return Track{
        .road = std::move(*road_curve),
        .islands = std::move(island_curves),
        .start = *start_pose,
        .road_color = *road_paint,
        .barrier_colors = barriers,
        .barrier_width = *width,
        .barrier_length = *length,
    };
}

Value Track::to_value() const {
    Value::Array barriers;
    barriers.values.reserve(barrier_colors.size());
    for (files::Color color : barrier_colors) {
        barriers.values.emplace_back(files::color_text(color));
    }
    Value::Array island_curves;
    island_curves.values.reserve(islands.size());
    for (const std::vector<std::array<double, 2>>& island : islands) {
        island_curves.values.push_back(files::points_value(island));
    }
    return json::object({
        {"Start", start.to_value()},
        {"Colors", json::object({
                       {"Road", files::color_text(road_color)},
                       {"Barriers", std::move(barriers)},
                   })},
        {"BarrierWidth", files::rounded(barrier_width)},
        {"BarrierLength", files::rounded(barrier_length)},
        {"Road", files::points_value(road)},
        {"Islands", std::move(island_curves)},
    });
}

std::vector<std::array<double, 2>>* Track::curve_mut(size_t index) {
    if (index == 0) {
        return &road;
    }
    return index - 1 < islands.size() ? &islands[index - 1] : nullptr;
}

Shape Track::shape() const {
    std::vector<std::array<double, 2>> road_outline = curve::sample(road);
    std::vector<std::vector<std::array<double, 2>>> island_outlines;
    island_outlines.reserve(islands.size());
    for (const std::vector<std::array<double, 2>>& island : islands) {
        island_outlines.push_back(curve::sample(island));
    }
    std::vector<std::span<const std::array<double, 2>>> outlines{road_outline};
    outlines.insert(outlines.end(), island_outlines.begin(), island_outlines.end());
    Edges edges(outlines);
    StartLine line = start_line(outlines, start);
    std::vector<std::string> found = problems(road_outline, island_outlines, start, edges);
    return Shape(std::move(road_outline), std::move(island_outlines), std::move(edges), line, std::move(found));
}

bool inside(std::span<const std::array<double, 2>> outline, std::array<double, 2> point) {
    double x = point[0];
    double z = point[1];
    bool within = false;
    for (size_t index = 0; index < outline.size(); index++) {
        std::array<double, 2> a = outline[index];
        std::array<double, 2> b = outline[(index + 1) % outline.size()];
        if ((a[1] > z) != (b[1] > z) && x < a[0] + (z - a[1]) / (b[1] - a[1]) * (b[0] - a[0])) {
            within = !within;
        }
    }
    return within;
}

} // namespace veehiicuul::track
