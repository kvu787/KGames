// A scene's track: the curves of its edges, where the car starts, and its
// colors. The road's curve is its outer edge, and each island's curve is
// the edge of a patch of ground inside the road, which isn't road. Their
// shape, from the curves, is what the car mustn't touch, and Ground.h makes
// the road, its barriers and the start line from it.
//
//     "Track": {
//         "Start": { "Position": [94.5, -48.5], "Heading": -104.3 },
//         "Colors": { "Road": "#81858B", "Barriers": ["#F61E22", "#F6FCFF"] },
//         "BarrierWidth": 1.5,
//         "BarrierLength": 15,
//         "Road": [[-94.6, 36.1], ...],
//         "Islands": [[[-79.7, 21.4], ...], ...]
//     }

#pragma once

#include "Veehiicuul/Collision.h"
#include "Veehiicuul/Files.h"
#include "Json/Json.h"

#include <array>
#include <cstddef>
#include <expected>
#include <ranges>
#include <span>
#include <string>
#include <string_view>
#include <vector>

namespace veehiicuul::track {

// Where a car starts, or a thing is put: its place on the ground, and which
// way it faces, in degrees from +Z toward +X.
struct Pose {
    std::array<double, 2> position{};
    double heading = 0.0;

    double radians() const;

    // The way it faces, on the ground.
    std::array<double, 2> ahead() const;

    static std::expected<Pose, std::string> parse(const json::Value& value, std::string_view name);

    json::Value to_value() const;

    bool operator==(const Pose&) const = default;
};

// How a failed check in a test shows a pose.
std::string describe_value(const Pose& pose);

struct Shape;

struct Track {
    // The control points of the road's outer edge, and of each island's.
    std::vector<std::array<double, 2>> road;
    std::vector<std::vector<std::array<double, 2>>> islands;
    Pose start;
    files::Color road_color{};
    // The barriers alternate between these.
    std::array<files::Color, 2> barrier_colors{};
    // How wide the barriers are, on the ground beside each edge, and about
    // how long each one is.
    double barrier_width = 0.0;
    double barrier_length = 0.0;

    // A track with a round road and no islands, around the origin.
    static Track round();

    static std::expected<Track, std::string> parse(const json::Value& value);

    json::Value to_value() const;

    // Every curve, the road's first, then the islands', by index.
    auto curves() const {
        return std::views::iota(size_t{0}, islands.size() + 1) |
               std::views::transform([this](size_t index) -> const std::vector<std::array<double, 2>>& { return index == 0 ? road : islands[index - 1]; });
    }

    // The curve at the index, or null if there's none.
    std::vector<std::array<double, 2>>* curve_mut(size_t index);

    Shape shape() const;

    bool operator==(const Track&) const = default;
};

// How far ahead of the car's start the start line is, and how deep it is
// along the road.
inline constexpr double StartLineAhead = 4.25;
inline constexpr double StartLineDepth = 1.5;

// The start line, across the road ahead of the start.
struct StartLine {
    std::array<double, 2> center{};
    // Across the road, and along it.
    double width = 0.0;
    double depth = 0.0;

    bool operator==(const StartLine&) const = default;
};

// What a track's curves make: the outlines of its road and its islands,
// their edges, the start line, and what's wrong with them, if anything.
struct Shape {
    std::vector<std::array<double, 2>> road;
    std::vector<std::vector<std::array<double, 2>>> islands;
    collision::Edges edges;
    StartLine start_line;
    std::vector<std::string> problems;

    Shape(std::vector<std::array<double, 2>> road_outline, std::vector<std::vector<std::array<double, 2>>> island_outlines, collision::Edges outline_edges, StartLine line,
          std::vector<std::string> shape_problems);
    // A shape is made from a track, and never copied.
    Shape(const Shape&) = delete;
    Shape& operator=(const Shape&) = delete;
    Shape(Shape&&) = default;
    Shape& operator=(Shape&&) = default;
};

// Whether a point is inside an outline, by how many of its edges a line
// from it toward +X crosses.
bool inside(std::span<const std::array<double, 2>> outline, std::array<double, 2> point);

} // namespace veehiicuul::track
