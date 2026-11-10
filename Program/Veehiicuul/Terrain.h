// A scene's terrain: a grid of hexels, hexagonal cells on the ground, each
// painted a color of the terrain's palette, or not painted, where the
// scene's background shows. It's flat, under the track, like the rest of the
// ground.
//
// Cells have axial coordinates, (q, r). With pointy tops, a cell's middle is
// at X = size * sqrt(3) * (q + r / 2) and Z = size * 3 / 2 * r, where `size`
// is the distance from a cell's middle to its corners. With flat tops, X and
// Z swap roles.
//
// In a scene's file, the terrain's cells are rows of characters, one a cell,
// from the cell at `Origin` on: a palette index as a digit or a letter, 0 to
// 9, then A to Z, then a to z, or "." for a cell that isn't painted.
//
//     "Terrain": {
//         "Size": 2,
//         "Orientation": "PointyTop",
//         "Palette": ["#17B856", "#2ACB6B"],
//         "Origin": [-85, -46],
//         "Rows": ["..0..1", "..11"]
//     }

#pragma once

#include "Veehiicuul/Files.h"
#include "Json/Json.h"

#include <array>
#include <cstddef>
#include <cstdint>
#include <expected>
#include <map>
#include <optional>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

namespace veehiicuul::terrain {

// A cell's axial coordinates, (q, r).
using Cell = std::pair<std::int32_t, std::int32_t>;

inline constexpr std::string_view Alphabet = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
// The most colors a palette has, one a character.
inline constexpr size_t PaletteSize = 62;

struct Terrain {
    // From a cell's middle to its corners, in units.
    double size = 2.0;
    // Whether cells have flat tops and bottoms along Z, or points.
    bool flat_top = false;
    std::vector<files::Color> palette{{0x17, 0xB8, 0x56}, {0xD8, 0xC4, 0x8A}, {0x4A, 0x9B, 0xE0}};
    // Each painted cell's palette index, by its axial coordinates.
    std::map<Cell, std::uint8_t> cells;

    // A cell's middle on the ground.
    std::array<double, 2> center(Cell cell) const;

    // The cell that a point on the ground is in.
    Cell cell_at(std::array<double, 2> point) const;

    // A cell's six corners, in order around it.
    std::array<std::array<double, 2>, 6> corners(Cell cell) const;

    std::optional<std::uint8_t> get(Cell cell) const;

    // Paints a cell with a palette index, or clears it with nothing.
    void set(Cell cell, std::optional<std::uint8_t> color);

    // Every cell connected to `start` through cells of its color, or
    // unpainted ones, which can go on forever, so only those within
    // `reach` cells of it count.
    std::vector<Cell> region(Cell start, std::int32_t reach) const;

    // Takes a color out of the palette, and with it the cells painted with
    // it. The cells of the colors after it keep their colors.
    void remove_color(std::uint8_t index);

    static std::expected<Terrain, std::string> parse(const json::Value& value);

    json::Value to_value() const;

    bool operator==(const Terrain&) const = default;
};

// How a failed check in a test shows a terrain.
std::string describe_value(const Terrain& terrain);

std::array<Cell, 6> neighbors(Cell cell);

// How many steps between two cells.
std::int32_t distance(Cell from, Cell to);

// Every cell within `radius` steps of `center`.
std::vector<Cell> disk(Cell center, std::int32_t radius);

} // namespace veehiicuul::terrain
