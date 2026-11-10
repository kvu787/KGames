// The hexel terrain: where cells are, which cell a point is in, brushes and
// fills, its palette, and its rows of characters in a scene's file.

#include "Veehiicuul/Terrain.h"
#include "Testing/Testing.h"
#include "Json/Json.h"

#include <algorithm>
#include <array>
#include <cmath>
#include <cstdint>
#include <expected>
#include <format>
#include <optional>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

using json::Value;
using veehiicuul::terrain::Cell;
using veehiicuul::terrain::disk;
using veehiicuul::terrain::distance;
using veehiicuul::terrain::FarthestCell;
using veehiicuul::terrain::neighbors;
using veehiicuul::terrain::Terrain;

namespace {

Terrain terrain_with(bool flat_top) {
    Terrain terrain;
    terrain.flat_top = flat_top;
    return terrain;
}

} // namespace

// A cell's middle is in it, and so is each point a little inside each of its
// corners, with pointy tops or flat ones.
TEST(a_point_is_in_the_cell_whose_middle_is_nearest) {
    for (bool flat_top : {false, true}) {
        Terrain terrain = terrain_with(flat_top);
        for (Cell cell : {Cell{0, 0}, Cell{3, -2}, Cell{-7, 11}, Cell{12, 5}}) {
            std::array<double, 2> center = terrain.center(cell);
            CHECK_EQ(terrain.cell_at(center), cell);
            for (std::array<double, 2> corner : terrain.corners(cell)) {
                std::array<double, 2> inside{center[0] + (corner[0] - center[0]) * 0.95, center[1] + (corner[1] - center[1]) * 0.95};
                CHECK_EQ(terrain.cell_at(inside), cell, "{} {}", flat_top, testing::describe(cell));
            }
            // Neighbors are as far apart as a cell is wide.
            for (Cell neighbor : neighbors(cell)) {
                std::array<double, 2> other = terrain.center(neighbor);
                double apart = std::hypot(other[0] - center[0], other[1] - center[1]);
                CHECK(std::abs(apart - terrain.size * std::sqrt(3.0)) < 1e-9);
            }
        }
    }
    // Pointy tops: a corner straight up and down along Z.
    std::array<std::array<double, 2>, 6> corners = terrain_with(false).corners({0, 0});
    CHECK(std::ranges::any_of(corners, [](std::array<double, 2> corner) { return std::abs(corner[0]) < 1e-9 && std::abs(corner[1] - 2.0) < 1e-9; }));
}

TEST(a_brush_is_the_cells_within_its_radius) {
    for (auto [radius, count] : {std::pair{0, 1}, std::pair{1, 7}, std::pair{2, 19}, std::pair{3, 37}}) {
        std::vector<Cell> cells = disk({4, -1}, radius);
        CHECK_EQ(cells.size(), count);
        CHECK(std::ranges::all_of(cells, [&](Cell cell) { return distance(cell, {4, -1}) <= radius; }));
    }
}

// A fill takes the cells joined to the first through cells of its color,
// and stops at cells of other colors, or its reach.
TEST(a_fill_stops_at_other_colors_and_its_reach) {
    Terrain terrain = terrain_with(false);
    for (Cell cell : disk({0, 0}, 3)) {
        if (distance(cell, {0, 0}) == 3) {
            terrain.set(cell, std::uint8_t{2});
        }
    }
    std::vector<Cell> inside = terrain.region({0, 0}, 60);
    CHECK_EQ(inside.size(), 19);
    std::vector<Cell> ring = terrain.region({3, 0}, 60);
    CHECK_EQ(ring.size(), 18);
    // Outside the ring, unpainted cells go on, as far as the reach.
    CHECK_EQ(terrain.region({10, 0}, 2).size(), 19);
}

TEST(removing_a_color_takes_it_off_its_cells) {
    Terrain terrain = terrain_with(false);
    terrain.set({0, 0}, std::uint8_t{0});
    terrain.set({1, 0}, std::uint8_t{1});
    terrain.set({2, 0}, std::uint8_t{2});
    terrain.remove_color(1);
    CHECK_EQ(terrain.palette.size(), 2);
    CHECK_EQ(terrain.get({0, 0}), std::optional<std::uint8_t>(std::uint8_t{0}));
    CHECK_EQ(terrain.get({1, 0}), std::nullopt);
    CHECK_EQ(terrain.get({2, 0}), std::optional<std::uint8_t>(std::uint8_t{1}), "the colors after it keep their colors");
}

// The cells are rows of characters from the origin, which read back the
// same.
TEST(writes_its_cells_as_rows_and_reads_them_back) {
    Terrain terrain = terrain_with(true);
    terrain.set({-2, 5}, std::uint8_t{0});
    terrain.set({1, 5}, std::uint8_t{2});
    terrain.set({0, 7}, std::uint8_t{1});
    Value value = terrain.to_value();
    const Value* origin = value.get("Origin");
    CHECK(origin != nullptr);
    CHECK_EQ(*origin, Value(std::vector<double>{-2.0, 5.0}));
    const Value* rows = value.get("Rows");
    CHECK(rows != nullptr);
    CHECK_EQ(*rows, json::parse(R"(["0..2", "", "..1"])").value());
    const Value* orientation = value.get("Orientation");
    CHECK(orientation != nullptr);
    CHECK_EQ(*orientation, Value("FlatTop"));
    CHECK_EQ(Terrain::parse(value).value(), terrain);
    std::string_view text = R"({ "Size": 2, "Orientation": "PointyTop", "Palette": ["#000000"], "Origin": [0, 0], "Rows": ["01"] })";
    std::expected<Terrain, std::string> wrong = Terrain::parse(json::parse(text).value());
    CHECK(!wrong.has_value());
    CHECK_EQ(wrong.error(), "Terrain.Rows[0] has '1', which isn't one of the palette's 1 colors");
}

// A cell is FarthestCell at most from cell [0, 0], along each axis, so that
// the cells around it, a brush's and a fill's, and the steps between it and
// them, are numbers that an int32 holds: a point far away, or any point of
// a terrain of tiny cells, is in the farthest cell, and a file whose cells
// are past it is wrong.
TEST(cells_are_no_farther_than_the_farthest_cell) {
    Terrain tiny = terrain_with(false);
    tiny.size = 1e-300;
    Cell far_ = tiny.cell_at({1.0, 0.0});
    CHECK_EQ(far_, Cell(FarthestCell, 0));
    CHECK_EQ(tiny.cell_at({-1.0, 0.0}), Cell(-FarthestCell, 0));
    CHECK_EQ(tiny.cell_at({1.0, 1.0}), Cell(FarthestCell, FarthestCell));
    CHECK_EQ(terrain_with(true).cell_at({-1e300, 1e300}), Cell(FarthestCell, -FarthestCell));
    // A brush and a fill there are the cells that they are anywhere.
    for (Cell cell : {far_, Cell(-FarthestCell, FarthestCell)}) {
        for (Cell neighbor : neighbors(cell)) {
            CHECK_EQ(distance(neighbor, cell), 1);
        }
        std::vector<Cell> brush = disk(cell, 8);
        CHECK_EQ(brush.size(), 217);
        CHECK(std::ranges::all_of(brush, [&](Cell under) { return distance(under, cell) <= 8; }));
        std::vector<Cell> filled = tiny.region(cell, 60);
        CHECK_EQ(filled.size(), 10981);
        CHECK(std::ranges::all_of(filled, [&](Cell under) { return distance(under, cell) <= 60; }));
    }
    // Painting with a brush there leaves the cells past the farthest
    // unpainted, so that the terrain reads back from what it writes.
    Terrain painted = terrain_with(false);
    for (Cell under : disk({FarthestCell, -FarthestCell}, 2)) {
        painted.set(under, std::uint8_t{1});
    }
    CHECK_EQ(painted.cells.size(), 9);
    CHECK_EQ(painted.get({FarthestCell - 2, -FarthestCell + 2}), std::optional<std::uint8_t>(std::uint8_t{1}));
    CHECK_EQ(painted.get({FarthestCell + 1, -FarthestCell}), std::nullopt);
    CHECK_EQ(Terrain::parse(painted.to_value()).value(), painted);

    auto problem = [](std::string_view origin, std::string_view rows) {
        std::string text = std::format(R"({{ "Size": 2, "Orientation": "PointyTop", "Palette": ["#000000"], "Origin": {}, "Rows": {} }})", origin, rows);
        std::expected<Terrain, std::string> terrain = Terrain::parse(json::parse(text).value());
        return terrain ? std::string("It reads") : terrain.error();
    };
    CHECK_EQ(problem("[2147483647, 2147483647]", R"(["00", "00"])"), "Terrain.Origin must be within 1073741824 cells of [0, 0], not [2147483647, 2147483647]");
    CHECK_EQ(problem("[0, -1073741825]", R"(["0"])"), "Terrain.Origin must be within 1073741824 cells of [0, 0], not [0, -1073741825]");
    CHECK_EQ(problem("[1e300, 0]", R"([])"), std::format("Terrain.Origin must be within 1073741824 cells of [0, 0], not [1{}, 0]", std::string(300, '0')));
    CHECK_EQ(problem("[1073741824, 0]", R"(["00"])"), "Terrain.Rows[0] goes past 1073741824 cells from [0, 0]");
    CHECK_EQ(problem("[0, 1073741824]", R"(["0", "", "0"])"), "Terrain.Rows[2] goes past 1073741824 cells from [0, 0]");
    // Up to it, they read.
    CHECK_EQ(problem("[1073741823, -1073741824]", R"(["00", ".."])"), "It reads");
}
