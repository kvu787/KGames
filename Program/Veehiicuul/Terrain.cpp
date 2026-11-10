#include "Veehiicuul/Terrain.h"

#include "Base/Numbers.h"
#include "Base/Sum.h"
#include "Base/Text.h"
#include "Veehiicuul/Angles.h"

#include <algorithm>
#include <cmath>
#include <cstdlib>
#include <format>
#include <functional>
#include <unordered_set>

namespace veehiicuul::terrain {

using json::Value;

namespace {

constexpr double Sqrt3 = 1.732'050'807'568'877'2;

// The cell nearest fractional axial coordinates, by rounding them as the
// cube coordinates (q, r, -q - r).
Cell round_axial(double q, double r) {
    double s = -q - r;
    double rq = std::round(q);
    double rr = std::round(r);
    double rs = std::round(s);
    double dq = std::abs(rq - q);
    double dr = std::abs(rr - r);
    double ds = std::abs(rs - s);
    if (dq > dr && dq > ds) {
        rq = -rr - rs;
    } else if (dr > ds) {
        rr = -rq - rs;
    }
    return {base::saturating_cast<std::int32_t>(rq), base::saturating_cast<std::int32_t>(rr)};
}

// For the set of cells that a fill has seen.
struct CellHash {
    size_t operator()(Cell cell) const {
        std::uint64_t both = (static_cast<std::uint64_t>(static_cast<std::uint32_t>(cell.first)) << 32) | static_cast<std::uint32_t>(cell.second);
        return std::hash<std::uint64_t>()(both);
    }
};

// A character in single quotes, as {:?} writes one.
std::string debug_character(char character) {
    if (character == '\'') {
        return "'\\''";
    }
    if (character == '"') {
        return "'\"'";
    }
    // A byte as the character of its number, as `as char` gives.
    std::string text;
    base::push_char(text, static_cast<char32_t>(static_cast<unsigned char>(character)));
    std::string quoted = base::debug(text);
    return "'" + quoted.substr(1, quoted.size() - 2) + "'";
}

} // namespace

std::array<double, 2> Terrain::center(Cell cell) const {
    double q = static_cast<double>(cell.first);
    double r = static_cast<double>(cell.second);
    double along = size * Sqrt3 * (q + r / 2.0);
    double across = size * 1.5 * r;
    return flat_top ? std::array<double, 2>{across, along} : std::array<double, 2>{along, across};
}

Cell Terrain::cell_at(std::array<double, 2> point) const {
    double x = point[0];
    double z = point[1];
    double along = flat_top ? z : x;
    double across = flat_top ? x : z;
    double r = across / (size * 1.5);
    double q = along / (size * Sqrt3) - r / 2.0;
    return round_axial(q, r);
}

std::array<std::array<double, 2>, 6> Terrain::corners(Cell cell) const {
    std::array<double, 2> middle = center(cell);
    double x = middle[0];
    double z = middle[1];
    double start = flat_top ? 0.0 : 30.0;
    std::array<std::array<double, 2>, 6> around{};
    for (size_t index = 0; index < around.size(); index++) {
        double angle = to_radians(start + 60.0 * static_cast<double>(index));
        around[index] = {x + size * std::cos(angle), z + size * std::sin(angle)};
    }
    return around;
}

std::optional<std::uint8_t> Terrain::get(Cell cell) const {
    auto found = cells.find(cell);
    return found == cells.end() ? std::nullopt : std::optional<std::uint8_t>(found->second);
}

void Terrain::set(Cell cell, std::optional<std::uint8_t> color) {
    if (color) {
        cells.insert_or_assign(cell, *color);
    } else {
        cells.erase(cell);
    }
}

std::vector<Cell> Terrain::region(Cell start, std::int32_t reach) const {
    std::optional<std::uint8_t> color = get(start);
    std::vector<Cell> found{start};
    std::unordered_set<Cell, CellHash> seen{start};
    size_t index = 0;
    while (index < found.size()) {
        Cell cell = found[index];
        index += 1;
        for (Cell neighbor : neighbors(cell)) {
            if (distance(neighbor, start) <= reach && get(neighbor) == color && seen.insert(neighbor).second) {
                found.push_back(neighbor);
            }
        }
    }
    return found;
}

void Terrain::remove_color(std::uint8_t index) {
    if (static_cast<size_t>(index) >= palette.size()) {
        return;
    }
    palette.erase(palette.begin() + index);
    std::erase_if(cells, [&](const std::pair<const Cell, std::uint8_t>& entry) { return entry.second == index; });
    for (auto& [cell, color] : cells) {
        if (color > index) {
            color -= 1;
        }
    }
}

std::expected<Terrain, std::string> Terrain::parse(const Value& value) {
    if (std::expected<void, std::string> known = files::only(value, {{"Size", "Orientation", "Palette", "Origin", "Rows"}}, "the terrain"); !known) {
        return std::unexpected(std::move(known.error()));
    }
    std::expected<const Value*, std::string> orientation = files::member(value, "Orientation", "Terrain.");
    if (!orientation) {
        return std::unexpected(std::move(orientation.error()));
    }
    bool flat = false;
    if (const Value::String* text = base::as<Value::String>(**orientation); text != nullptr && base::eq_ignore_ascii_case(text->value, "PointyTop")) {
        flat = false;
    } else if (text != nullptr && base::eq_ignore_ascii_case(text->value, "FlatTop")) {
        flat = true;
    } else {
        return std::unexpected(std::format("Terrain.Orientation must be \"PointyTop\" or \"FlatTop\", not {}", (*orientation)->describe()));
    }
    std::expected<const Value*, std::string> palette_member = files::member(value, "Palette", "Terrain.");
    if (!palette_member) {
        return std::unexpected(std::move(palette_member.error()));
    }
    std::vector<files::Color> colors;
    if (const Value::Array* items = base::as<Value::Array>(**palette_member); items != nullptr && items->values.size() <= PaletteSize) {
        colors.reserve(items->values.size());
        for (size_t index = 0; index < items->values.size(); index++) {
            std::expected<files::Color, std::string> color = files::color(items->values[index], std::format("Terrain.Palette[{}]", index));
            if (!color) {
                return std::unexpected(std::move(color.error()));
            }
            colors.push_back(*color);
        }
    } else {
        return std::unexpected(std::format("Terrain.Palette must be a list of {} colors at most, not {}", PaletteSize, (*palette_member)->describe()));
    }
    std::expected<std::array<double, 2>, std::string> origin =
        files::member(value, "Origin", "Terrain.").and_then([](const Value* found) { return files::point(*found, "Terrain.Origin"); });
    if (!origin) {
        return std::unexpected(std::move(origin.error()));
    }
    if (std::ranges::any_of(*origin, [](double coordinate) { return coordinate - std::trunc(coordinate) != 0.0; })) {
        return std::unexpected(std::string("Terrain.Origin must be a cell's coordinates, two whole numbers"));
    }
    std::map<Cell, std::uint8_t> painted;
    std::expected<const Value*, std::string> rows_member = files::member(value, "Rows", "Terrain.");
    if (!rows_member) {
        return std::unexpected(std::move(rows_member.error()));
    }
    const Value::Array* rows = base::as<Value::Array>(**rows_member);
    if (rows == nullptr) {
        return std::unexpected(std::string("Terrain.Rows must be a list of rows of characters"));
    }
    for (size_t row_index = 0; row_index < rows->values.size(); row_index++) {
        const Value::String* row = base::as<Value::String>(rows->values[row_index]);
        if (row == nullptr) {
            return std::unexpected(std::format("Terrain.Rows[{}] must be text, not {}", row_index, rows->values[row_index].describe()));
        }
        for (size_t column = 0; column < row->value.size(); column++) {
            char character = row->value[column];
            if (character == '.') {
                continue;
            }
            size_t color = Alphabet.find(character);
            if (color == std::string_view::npos || color >= colors.size()) {
                return std::unexpected(std::format("Terrain.Rows[{}] has {}, which isn't one of the palette's {} colors", row_index, debug_character(character), colors.size()));
            }
            Cell cell{
                base::saturating_cast<std::int32_t>((*origin)[0]) + static_cast<std::int32_t>(column),
                base::saturating_cast<std::int32_t>((*origin)[1]) + static_cast<std::int32_t>(row_index),
            };
            painted.insert_or_assign(cell, static_cast<std::uint8_t>(color));
        }
    }
    std::expected<double, std::string> cell_size = files::member(value, "Size", "Terrain.").and_then([](const Value* found) { return files::positive(*found, "Terrain.Size"); });
    if (!cell_size) {
        return std::unexpected(std::move(cell_size.error()));
    }
    return Terrain{
        .size = *cell_size,
        .flat_top = flat,
        .palette = std::move(colors),
        .cells = std::move(painted),
    };
}

Value Terrain::to_value() const {
    std::int32_t q0 = 0;
    std::int32_t r0 = 0;
    std::int32_t q1 = -1;
    std::int32_t r1 = -1;
    if (!cells.empty()) {
        Cell first = cells.begin()->first;
        q0 = first.first;
        r0 = first.second;
        q1 = first.first;
        r1 = first.second;
    }
    for (const auto& [cell, color] : cells) {
        q0 = std::min(q0, cell.first);
        r0 = std::min(r0, cell.second);
        q1 = std::max(q1, cell.first);
        r1 = std::max(r1, cell.second);
    }
    Value::Array rows;
    rows.values.reserve(r1 >= r0 ? static_cast<size_t>(static_cast<std::int64_t>(r1) - r0 + 1) : 0);
    for (std::int64_t r = r0; r <= r1; r++) {
        std::string row;
        for (std::int64_t q = q0; q <= q1; q++) {
            std::optional<std::uint8_t> color = get({static_cast<std::int32_t>(q), static_cast<std::int32_t>(r)});
            row += color ? Alphabet[*color] : '.';
        }
        row.erase(row.find_last_not_of('.') + 1);
        rows.values.emplace_back(std::move(row));
    }
    Value::Array colors;
    colors.values.reserve(palette.size());
    for (files::Color color : palette) {
        colors.values.emplace_back(files::color_text(color));
    }
    Value::Array origin;
    origin.values.reserve(2);
    origin.values.emplace_back(static_cast<double>(q0));
    origin.values.emplace_back(static_cast<double>(r0));
    return json::object({
        {"Size", size},
        {"Orientation", flat_top ? "FlatTop" : "PointyTop"},
        {"Palette", std::move(colors)},
        {"Origin", std::move(origin)},
        {"Rows", std::move(rows)},
    });
}

std::string describe_value(const Terrain& terrain) {
    std::string colors;
    for (files::Color color : terrain.palette) {
        colors += std::format("{}[{}, {}, {}]", colors.empty() ? "" : ", ", static_cast<unsigned>(color[0]), static_cast<unsigned>(color[1]), static_cast<unsigned>(color[2]));
    }
    std::string painted;
    for (const auto& [cell, color] : terrain.cells) {
        painted += std::format("{}({}, {}): {}", painted.empty() ? "" : ", ", cell.first, cell.second, static_cast<unsigned>(color));
    }
    return std::format("Terrain {{ size: {}, flat_top: {}, palette: [{}], cells: {{{}}} }}", base::debug(terrain.size), terrain.flat_top, colors, painted);
}

std::array<Cell, 6> neighbors(Cell cell) {
    std::int32_t q = cell.first;
    std::int32_t r = cell.second;
    return {{{q + 1, r}, {q - 1, r}, {q, r + 1}, {q, r - 1}, {q + 1, r - 1}, {q - 1, r + 1}}};
}

std::int32_t distance(Cell from, Cell to) {
    std::int32_t dq = to.first - from.first;
    std::int32_t dr = to.second - from.second;
    return (std::abs(dq) + std::abs(dr) + std::abs(dq + dr)) / 2;
}

std::vector<Cell> disk(Cell center, std::int32_t radius) {
    std::vector<Cell> cells;
    for (std::int32_t dq = -radius; dq <= radius; dq++) {
        for (std::int32_t dr = std::max(-radius, -dq - radius); dr <= std::min(radius, -dq + radius); dr++) {
            cells.push_back({center.first + dq, center.second + dr});
        }
    }
    return cells;
}

} // namespace veehiicuul::terrain
