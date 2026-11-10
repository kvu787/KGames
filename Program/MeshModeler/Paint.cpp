#include "MeshModeler/Paint.h"

#include "Base/Panic.h"
#include "Base/Sum.h"
#include "Base/Vec.h"
#include "PaintSettings/PaintSettings.h"

#include <algorithm>
#include <format>
#include <utility>

namespace mesh_modeler::paint {

using simple_paint::GpuMaterial;
using simple_paint::Material;
using simple_paint::Parameters;
using user_interface::Color;

Paint::Paint(std::string_view name, std::uint32_t color) : name(name), parameters() {
    set_color(Color::hex(color));
}

Paint::Paint(std::string name, const Parameters& parameters) : name(std::move(name)), parameters(parameters) {}

Color Paint::color() const {
    auto [r, g, b] = parameters.base_color_srgb;
    return Color::rgb(static_cast<float>(r), static_cast<float>(g), static_cast<float>(b));
}

void Paint::set_color(Color color) {
    auto channel = [](float value) {
        return std::clamp(static_cast<double>(value), simple_paint::Margin, simple_paint::InteriorMaximum);
    };
    parameters.base_color_srgb = {channel(color.r), channel(color.g), channel(color.b)};
}

GpuMaterial Paint::material() const {
    if (std::expected<Material, simple_paint::Error> compiled = Material::compile(parameters)) {
        return compiled->constants();
    }
    std::expected<Material, simple_paint::Error> fallback = Material::compile(Parameters{});
    if (!fallback) {
        base::panic("the default paint compiles");
    }
    return fallback->constants();
}

std::array<float, 4> Paint::linear_color() const {
    auto [r, g, b] = color().to_linear();
    return {r, g, b, 1.0f};
}

json::Value Paint::to_value() const {
    json::Value::Object object;
    object.members.reserve(2);
    object.members.emplace_back("Name", json::Value(std::string_view(name)));
    object.members.emplace_back("Paint", paint_settings::to_value(parameters));
    return object;
}

std::expected<Paint, std::string> Paint::from_value(const json::Value& value) {
    const json::Value* named = value.get("Name");
    if (named == nullptr) {
        return std::unexpected("A paint must have a Name.");
    }
    const json::Value::String* text = base::as<json::Value::String>(*named);
    if (text == nullptr) {
        return std::unexpected(std::format("A paint's Name must be text, not {}.", named->describe()));
    }
    std::string paint_name = text->value;
    Parameters settings;
    if (const json::Value* paint = value.get("Paint")) {
        std::expected<Parameters, std::string> parsed = paint_settings::parse(*paint, Parameters{});
        if (!parsed) {
            return std::unexpected(std::format("The paint {}'s {}.", paint_name, parsed.error()));
        }
        settings = *parsed;
    }
    return Paint(std::move(paint_name), settings);
}

std::vector<Paint> default_paints() {
    return base::vec(Paint("Blue", 0x1B3994), Paint("Coral", 0xE8624A), Paint("Sand", 0xD9C09A), Paint("Moss", 0x4E7A3A));
}

std::uint32_t next_color(size_t count) {
    constexpr std::array<std::uint32_t, 8> Colors{0x0078D4, 0xCA5010, 0x107C10, 0x8764B8, 0x038387, 0xD13438, 0xC19C00, 0xE3008C};
    return Colors[count % Colors.size()];
}

} // namespace mesh_modeler::paint
