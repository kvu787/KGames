// What the game's JSON files are made of: numbers, points on the ground,
// colors and headings, read with what's wrong with them, and written back
// the same way, so that a file that's read and written again is unchanged.
//
// Points are [X, Z] on the ground, in units, headings are in degrees from +Z
// toward +X, and colors are sRGB, like "#1CDA61".

#pragma once

#include "Json/Json.h"

#include <array>
#include <cstddef>
#include <cstdint>
#include <expected>
#include <filesystem>
#include <optional>
#include <span>
#include <string>
#include <string_view>
#include <vector>

namespace veehiicuul::files {

// An sRGB color, a byte a channel, as files write it.
using Color = std::array<std::uint8_t, 3>;

// Reads a file's text as JSON, or says what's wrong with it.
std::expected<json::Value, std::string> parse(std::string_view text);

std::expected<std::string, std::string> read(const std::filesystem::path& path);

// Writes a file, and the folders it's in.
std::expected<void, std::string> write(const std::filesystem::path& path, std::string_view text);

// Fails if `value` isn't an object, or has a member that isn't one of
// `names`, which says what it's part of, like "a track".
std::expected<void, std::string> only(const json::Value& value, std::span<const std::string_view> names, std::string_view what);

// The member, which is never null.
std::expected<const json::Value*, std::string> member(const json::Value& value, std::string_view name, std::string_view path);

std::expected<double, std::string> number(const json::Value& value, std::string_view name);

std::expected<double, std::string> positive(const json::Value& value, std::string_view name);

std::expected<std::string, std::string> text(const json::Value& value, std::string_view name);

std::expected<std::array<double, 2>, std::string> point(const json::Value& value, std::string_view name);

// A list of at least `fewest` points.
std::expected<std::vector<std::array<double, 2>>, std::string> points(const json::Value& value, std::string_view name, size_t fewest);

std::expected<Color, std::string> color(const json::Value& value, std::string_view name);

// A color from text like "#1CDA61".
std::optional<Color> parse_color(std::string_view text);

std::string color_text(Color color);

// As sRGB from 0 to 1.
std::array<float, 3> srgb(Color color);

// A number as files write it: to 4 decimal places, a ten-thousandth of a
// unit, so that dragging doesn't write noise.
json::Value rounded(double value);

json::Value point_value(std::array<double, 2> point);

json::Value points_value(std::span<const std::array<double, 2>> points);

} // namespace veehiicuul::files
