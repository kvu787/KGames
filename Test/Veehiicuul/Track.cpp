// The tracks: the curves that make their edges, Ribeye's shape, what's
// wrong with a track, reading one, and the ground that a track and a
// terrain make.

#include "Veehiicuul/Track.h"
#include "Base/Panic.h"
#include "Base/Text.h"
#include "Meshes/Meshes.h"
#include "SimplePaint/Geometry.h"
#include "Testing/Testing.h"
#include "Veehiicuul/Collision.h"
#include "Veehiicuul/Curve.h"
#include "Veehiicuul/Ground.h"
#include "Veehiicuul/Scene.h"
#include "Veehiicuul/Terrain.h"
#include "Json/Json.h"

#include "Folders.h"

#include <algorithm>
#include <array>
#include <cmath>
#include <cstdint>
#include <expected>
#include <filesystem>
#include <span>
#include <string>
#include <string_view>
#include <utility>
#include <variant>
#include <vector>

namespace curve = veehiicuul::curve;
namespace ground = veehiicuul::ground;

using json::Value;
using simple_paint::validate_mesh;
using veehiicuul::collision::Footprint;
using veehiicuul::curve::SamplesPerSpan;
using veehiicuul::ground::area;
using veehiicuul::ground::Barriers;
using veehiicuul::ground::ColorCount;
using veehiicuul::ground::Road;
using veehiicuul::ground::StartLine;
using veehiicuul::ground::triangulate;
using veehiicuul::ground::with_holes;
using veehiicuul::scene::Scene;
using veehiicuul::terrain::Terrain;
using veehiicuul::track::inside;
using veehiicuul::track::Pose;
using veehiicuul::track::Shape;
using veehiicuul::track::Track;

namespace {

Scene ribeye() {
    std::filesystem::path path = std::filesystem::path(folders::Project) / "../../Program/Veehiicuul/Scenes/Ribeye.json";
    std::expected<Scene, std::string> scene = Scene::load(path);
    if (!scene) {
        base::panic(scene.error());
    }
    return std::move(*scene);
}

bool near_(std::array<double, 2> a, std::array<double, 2> b, double tolerance) {
    return curve::distance(a, b) < tolerance;
}

// Twice a triangle's signed area: below 0 if it's wound counterclockwise
// seen from above, where X is right and -Z up.
double turn(std::array<double, 2> a, std::array<double, 2> b, std::array<double, 2> c) {
    return (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
}

// The triangles that cover a polygon, which they cover once: their areas
// add up to its.
double covered(std::span<const std::array<double, 2>> outline) {
    double total = 0.0;
    for (std::array<size_t, 3> triangle : triangulate(outline)) {
        double doubled = turn(outline[triangle[0]], outline[triangle[1]], outline[triangle[2]]);
        CHECK(doubled <= 1e-9, "{} isn't counterclockwise", testing::describe(triangle));
        total -= doubled / 2.0;
    }
    return total;
}

// What's wrong with the track, which must be something.
std::string problem_of(const std::expected<Track, std::string>& track) {
    CHECK(!track.has_value());
    return track.error();
}

} // namespace

// A curve is smooth, near its control points, and the same all the way
// around: a square's is a rounded square inside it, and a circle's control
// points make a curve of about the circle's radius.
TEST(a_curve_goes_around_near_its_control_points) {
    std::array<std::array<double, 2>, 4> square{{{0.0, 0.0}, {6.0, 0.0}, {6.0, 6.0}, {0.0, 6.0}}};
    std::vector<std::array<double, 2>> outline = curve::sample(square);
    CHECK_EQ(outline.size(), 4 * SamplesPerSpan);
    // At a control point's parameter, the curve is a sixth of the way to
    // each neighbor from it, and two thirds of it.
    CHECK(near_(curve::point(square, 1.0), {5.0, 1.0}, 1e-12));
    CHECK(near_(curve::point(square, 5.0), {5.0, 1.0}, 1e-12), "it goes around");
    // Inside its control points, though past where it is at their
    // parameters, between them.
    for (std::array<double, 2> point : outline) {
        CHECK(0.0 <= point[0] && point[0] <= 6.0 && 0.0 <= point[1] && point[1] <= 6.0);
    }
    CHECK(std::ranges::any_of(outline, [](std::array<double, 2> point) { return point[0] > 5.5; }));
    for (std::array<double, 2> point : curve::sample(curve::circle({10.0, -5.0}, 20.0, 12))) {
        double radius = curve::distance(point, {10.0, -5.0});
        CHECK(std::abs(radius - 20.0) < 0.05, "{}", base::display(radius));
    }
}

// A new control point goes after the one that starts the span nearest it,
// so that it bends the curve where it's put.
TEST(a_new_control_point_goes_into_the_nearest_span) {
    std::array<std::array<double, 2>, 4> square{{{0.0, 0.0}, {6.0, 0.0}, {6.0, 6.0}, {0.0, 6.0}}};
    // The span from parameter 1 to 2 is pulled most by points 1 and 2,
    // along X = 5.
    CHECK_EQ(curve::insertion(square, {7.0, 3.0}), 2);
    CHECK_EQ(curve::insertion(square, {3.0, -1.0}), 1);
}

// Ribeye's curves make a road with two islands, whose shape the car at the
// start doesn't touch, with the start line from edge to edge, and nothing
// wrong.
TEST(ribeye_s_curves_make_its_road_and_start_line) {
    Scene scene = ribeye();
    const Track& track = scene.track;
    std::vector<size_t> counts;
    for (const std::vector<std::array<double, 2>>& points : track.curves()) {
        counts.push_back(points.size());
    }
    CHECK_EQ(counts, (std::vector<size_t>{64, 28, 16}));
    Shape shape = track.shape();
    CHECK_EQ(shape.road.size(), 64 * SamplesPerSpan);
    CHECK_EQ(shape.edges.len(), (64 + 28 + 16) * SamplesPerSpan);
    CHECK(shape.problems.empty(), "{}", testing::describe(shape.problems));
    Footprint footprint = Footprint::from_sides(1.5, 1.5, 2.9925, 3.0);
    CHECK(!shape.edges.touch(footprint, track.start.position, track.start.radians()));
    veehiicuul::track::StartLine line = shape.start_line;
    CHECK(12.0 <= line.width && line.width < 13.5, "{}", base::display(line.width));
    CHECK(inside(shape.road, line.center) && !inside(shape.islands[0], line.center));
}

TEST(says_what_is_wrong_with_a_track_s_shape) {
    Track track = Track::round();
    CHECK(track.shape().problems.empty(), "{}", testing::describe(track.shape().problems));
    track.start.position = {0.0, 0.0};
    CHECK_EQ(track.shape().problems, (std::vector<std::string>{"The start isn't on the road"}));
    track.start.position = {32.5, 0.0};
    track.islands.push_back(curve::circle({200.0, 0.0}, 5.0, 6));
    CHECK_EQ(track.shape().problems, (std::vector<std::string>{"Island 2 is outside the road"}));
    track.islands.pop_back();
    // An island as big as the road crosses its edge.
    track.islands[0] = curve::circle({20.0, 0.0}, 30.0, 10);
    std::vector<std::string> crossing = track.shape().problems;
    CHECK(std::ranges::contains(crossing, "The road's edge crosses island 1's"), "{}", testing::describe(crossing));
    // An island inside an island, whichever of the two comes first.
    track.islands[0] = curve::circle({20.0, 0.0}, 6.0, 8);
    track.islands.push_back(curve::circle({20.0, 0.0}, 2.0, 6));
    CHECK_EQ(track.shape().problems, (std::vector<std::string>{"Island 2 is inside island 1"}));
    std::swap(track.islands[0], track.islands[1]);
    CHECK_EQ(track.shape().problems, (std::vector<std::string>{"Island 1 is inside island 2"}));
    track.islands.pop_back();
    // A curve that crosses itself, like a figure eight.
    track.islands[0] = {{-5.0, -5.0}, {5.0, 5.0}, {5.0, -5.0}, {-5.0, 5.0}};
    std::vector<std::string> itself = track.shape().problems;
    CHECK(std::ranges::contains(itself, "Island 1's edge crosses itself"), "{}", testing::describe(itself));
}

TEST(reads_a_track_and_writes_it_back) {
    Track track = Track::round();
    Value value = track.to_value();
    Track read = Track::parse(value).value();
    CHECK_EQ(read.road.size(), track.road.size());
    CHECK_EQ(read.start, (Pose{.position = {32.5, 0.0}, .heading = 180.0}));
    CHECK_EQ(read.to_value(), value);
    auto with = [&](std::string_view name, Value member) {
        Value::Object members = std::get<Value::Object>(value.value);
        std::erase_if(members.members, [&](const std::pair<std::string, Value>& other) { return other.first == name; });
        members.members.emplace_back(std::string(name), std::move(member));
        return Track::parse(Value(std::move(members)));
    };
    CHECK_EQ(problem_of(with("Road", json::parse("[[0, 0], [1, 0]]").value())), "Track.Road needs at least 3 points, not 2");
    CHECK_EQ(problem_of(with("BarrierWidth", 0.0)), "Track.BarrierWidth must be above 0, not 0");
    CHECK_EQ(problem_of(with("Hills", 1.0)), "Hills isn't part of the track. They're Start, Colors, BarrierWidth, BarrierLength, Road, Islands");
}

TEST(triangles_cover_a_polygon_even_where_it_turns_in) {
    for (const std::vector<std::array<double, 2>>& outline : {
             std::vector<std::array<double, 2>>{{0.0, 0.0}, {1.0, 0.0}, {1.0, 1.0}, {0.0, 1.0}},
             std::vector<std::array<double, 2>>{{0.0, 1.0}, {1.0, 1.0}, {1.0, 0.0}, {0.0, 0.0}},
             std::vector<std::array<double, 2>>{{0.0, 0.0}, {2.0, 0.0}, {2.0, 1.0}, {1.0, 1.0}, {1.0, 2.0}, {0.0, 2.0}},
         }) {
        CHECK_EQ(triangulate(outline).size(), outline.size() - 2);
        CHECK(std::abs(covered(outline) - std::abs(area(outline)) / 2.0) < 1e-9);
    }
}

// The road is its outline with its islands cut out: its triangles cover the
// outline's area less the islands'.
TEST(the_road_has_its_islands_cut_out) {
    auto square = [](std::array<double, 2> center, double size) {
        return std::vector<std::array<double, 2>>{
            {center[0] - size, center[1] - size},
            {center[0] + size, center[1] - size},
            {center[0] + size, center[1] + size},
            {center[0] - size, center[1] + size},
        };
    };
    std::vector<std::array<double, 2>> outline = square({0.0, 0.0}, 10.0);
    std::vector<std::vector<std::array<double, 2>>> holes{square({-4.0, 0.0}, 2.0), square({5.0, 3.0}, 1.0)};
    std::vector<std::array<double, 2>> joined = with_holes(outline, holes);
    CHECK_EQ(joined.size(), 4 + 2 * 6);
    CHECK(std::abs(covered(joined) - (400.0 - 16.0 - 4.0)) < 1e-6, "{}", base::display(covered(joined)));
    Shape shape = ribeye().track.shape();
    std::vector<std::array<double, 2>> road = with_holes(shape.road, shape.islands);
    // Added up from -0, as Rust's sum of floats is.
    double islands = -0.0;
    for (const std::vector<std::array<double, 2>>& island : shape.islands) {
        islands += std::abs(area(island));
    }
    double expected = (std::abs(area(shape.road)) - islands) / 2.0;
    CHECK(std::abs(covered(road) - expected) < 1e-3 * expected, "{} of {}", base::display(covered(road)), base::display(expected));
}

// The track's mesh is the road, then an even number of barriers along each
// outline, then the start line, and SimplePaint's pipeline takes it.
TEST(the_track_s_mesh_has_every_part_in_order) {
    Scene scene = ribeye();
    Shape shape = scene.track.shape();
    meshes::Mesh mesh = ground::track_mesh(scene.track, shape);
    std::expected<void, simple_paint::Error> valid = validate_mesh(mesh.vertices, mesh.indices, static_cast<std::uint32_t>(ColorCount));
    CHECK(valid.has_value(), "{}", valid.error().message);
    std::vector<std::uint32_t> materials;
    for (size_t first = 0; first < mesh.indices.size(); first += 3) {
        materials.push_back(mesh.vertices[mesh.indices[first]].material_index);
    }
    size_t road = static_cast<size_t>(std::ranges::find_if(materials, [](std::uint32_t material) { return material != Road; }) - materials.begin());
    // A polygon with two holes has two more triangles than corners less 2.
    size_t corners = shape.road.size();
    for (const std::vector<std::array<double, 2>>& island : shape.islands) {
        corners += island.size();
    }
    CHECK_EQ(road, corners + 2 * 2 - 2);
    size_t barriers = shape.edges.len() * 2;
    std::span<const std::uint32_t> rest(materials);
    CHECK(std::ranges::all_of(rest.subspan(road, barriers), [](std::uint32_t material) { return std::ranges::contains(Barriers, material); }));
    CHECK(std::ranges::all_of(rest.subspan(road + barriers), [](std::uint32_t material) { return std::ranges::contains(StartLine, material); }));
    CHECK_EQ(ground::colors(scene.terrain, scene.track)[Road],
             (std::array{static_cast<float>(0x81) / 255.0f, static_cast<float>(0x85) / 255.0f, static_cast<float>(0x8B) / 255.0f}));
}

// Each painted hexel is four triangles of its palette's color.
TEST(the_terrain_s_mesh_is_its_painted_hexels) {
    Terrain terrain;
    terrain.set({0, 0}, std::uint8_t{1});
    terrain.set({3, -2}, std::uint8_t{2});
    meshes::Mesh mesh = ground::terrain_mesh(terrain);
    CHECK_EQ(std::pair(mesh.vertices.size(), mesh.indices.size()), std::pair(size_t{12}, size_t{24}));
    CHECK_EQ(mesh.vertices[0].material_index, 1);
    std::expected<void, simple_paint::Error> valid = validate_mesh(mesh.vertices, mesh.indices, static_cast<std::uint32_t>(ColorCount));
    CHECK(valid.has_value(), "{}", valid.error().message);
    veehiicuul::files::Color color = terrain.palette[2];
    CHECK_EQ(ground::colors(terrain, Track::round())[2],
             (std::array{static_cast<float>(color[0]) / 255.0f, static_cast<float>(color[1]) / 255.0f, static_cast<float>(color[2]) / 255.0f}));
}
