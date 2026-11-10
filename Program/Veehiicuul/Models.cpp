#include "Veehiicuul/Models.h"

#include "Base/Files.h"
#include "Base/Sum.h"
#include "Base/Text.h"
#include "PaintSettings/PaintSettings.h"
#include "Veehiicuul/Files.h"

#include <algorithm>
#include <cmath>
#include <format>
#include <limits>
#include <system_error>

namespace veehiicuul::models {

using collision::Footprint;
using json::Value;
using simple_paint::Parameters;
using simple_paint::Vertex;

namespace {

// The smallest and largest of SimplePaint's color channels.
constexpr double Lowest = 1.0 / 1024.0;
constexpr double Highest = 1.0 - 1.0 / 1024.0;

} // namespace

Parameters paint_from_color(std::array<float, 4> color) {
    auto srgb = [](float linear) {
        double channel = static_cast<double>(std::clamp(linear, 0.0f, 1.0f));
        double encoded = channel <= 0.003'130'8 ? channel * 12.92 : 1.055 * std::pow(channel, 1.0 / 2.4) - 0.055;
        return std::clamp(std::round(encoded * 10'000.0) / 10'000.0, Lowest, Highest);
    };
    return Parameters{
        .base_color_srgb = {srgb(color[0]), srgb(color[1]), srgb(color[2])},
        .brightness = 0.5,
        .shift = 0.25,
        .rotation_degrees = 35.0,
        .dark_point = 0.0,
        .light_point = 1.0,
    };
}

Model::Model(std::string model_name, std::vector<std::string> model_slots, std::vector<Parameters> paints, std::vector<Vertex> model_vertices,
             std::vector<std::uint32_t> model_indices, Footprint model_footprint) :
    name(std::move(model_name)),
    slots(std::move(model_slots)),
    blender_paints(std::move(paints)),
    vertices(std::move(model_vertices)),
    indices(std::move(model_indices)),
    footprint(model_footprint) {}

std::expected<Model, std::string> Model::read(std::string_view model_name, const gltf::Model& model) {
    if (model.material_list.size() > MostSlots) {
        return std::unexpected(std::format("has {} materials, and a model has {} at most", model.material_list.size(), MostSlots));
    }
    std::vector<Parameters> paints;
    for (const gltf::Material& material : model.material_list) {
        Parameters from_color = paint_from_color(material.base_color);
        const Value* paint = material.extras ? material.extras->get("SimplePaint") : nullptr;
        if (paint != nullptr) {
            std::expected<Parameters, std::string> parsed = paint_settings::parse(*paint, from_color);
            if (!parsed) {
                return std::unexpected(std::format("material {}'s SimplePaint {}", material.name, parsed.error()));
            }
            paints.push_back(*parsed);
        } else {
            paints.push_back(from_color);
        }
    }
    std::vector<Vertex> mesh_vertices;
    mesh_vertices.reserve(model.positions.size());
    for (size_t index = 0; index < model.positions.size(); index++) {
        mesh_vertices.push_back(Vertex{
            .position = model.positions[index],
            .normal = model.normals[index],
            .material_index = model.materials[index],
        });
    }
    if (std::expected<void, simple_paint::Error> valid = simple_paint::validate_mesh(mesh_vertices, model.indices, static_cast<std::uint32_t>(model.material_list.size()));
        !valid) {
        return std::unexpected(std::move(valid.error().message));
    }
    std::vector<std::string> names;
    names.reserve(model.material_list.size());
    for (const gltf::Material& material : model.material_list) {
        names.push_back(material.name);
    }
    return Model(std::string(model_name), std::move(names), std::move(paints), std::move(mesh_vertices), model.indices, Footprint::of_model(model.positions));
}

std::vector<Parameters> Model::materials(std::span<const Paint* const> paints) const {
    std::vector<Parameters> painted = blender_paints;
    for (const Paint* paint : paints) {
        for (const std::pair<std::string, Parameters>& entry : *paint) {
            auto found = std::ranges::find(slots, entry.first);
            if (found != slots.end()) {
                painted[static_cast<size_t>(found - slots.begin())] = entry.second;
            }
        }
    }
    return painted;
}

Models::Models(std::vector<Model> models) : list(std::move(models)) {}

std::expected<Models, std::string> Models::load(const std::filesystem::path& folder) {
    std::error_code error;
    std::filesystem::directory_iterator entries(folder, error);
    if (error) {
        return std::unexpected(std::format("Couldn't read {}: {}", base::display(folder), base::os_error(static_cast<unsigned long>(error.value()))));
    }
    std::vector<std::filesystem::path> paths;
    for (; entries != std::filesystem::directory_iterator(); entries.increment(error)) {
        std::filesystem::path path = entries->path();
        if (base::eq_ignore_ascii_case(base::display(path.extension()), ".glb")) {
            paths.push_back(std::move(path));
        }
    }
    // By their names' bytes, as UTF-8.
    std::ranges::stable_sort(paths, {}, [](const std::filesystem::path& path) { return base::display(path); });
    std::vector<Model> models;
    for (const std::filesystem::path& path : paths) {
        std::string name = base::display(path.stem());
        std::expected<gltf::Model, std::string> model = gltf::load(path);
        if (!model) {
            return std::unexpected(std::move(model.error()));
        }
        std::expected<Model, std::string> read = Model::read(name, *model);
        if (!read) {
            return std::unexpected(std::format("{}: {}", base::display(path), read.error()));
        }
        models.push_back(std::move(*read));
    }
    if (models.empty()) {
        return std::unexpected(std::format("{} has no models", base::display(folder)));
    }
    return Models(std::move(models));
}

std::optional<size_t> Models::find(std::string_view name) const {
    for (size_t index = 0; index < list.size(); index++) {
        if (list[index].name == name) {
            return index;
        }
    }
    return std::nullopt;
}

std::vector<std::string_view> Models::names() const {
    std::vector<std::string_view> all;
    all.reserve(list.size());
    for (const Model& model : list) {
        all.push_back(model.name);
    }
    return all;
}

std::expected<Paint, std::string> parse_paint(const Value& value, std::string_view name) {
    const Value::Object* object = base::as<Value::Object>(value);
    if (object == nullptr) {
        return std::unexpected(std::format("{} must be an object of materials by their slots' names, not {}", name, value.describe()));
    }
    Paint paint;
    paint.reserve(object->members.size());
    for (const std::pair<std::string, Value>& entry : object->members) {
        std::expected<Parameters, std::string> parameters = paint_settings::parse(entry.second, paint_from_color({0.5f, 0.5f, 0.5f, 1.0f}));
        if (!parameters) {
            return std::unexpected(std::format("{}.{} {}", name, entry.first, parameters.error()));
        }
        paint.emplace_back(entry.first, *parameters);
    }
    return paint;
}

Value paint_value(const Paint& paint) {
    Value::Object object;
    object.members.reserve(paint.size());
    for (const std::pair<std::string, Parameters>& entry : paint) {
        object.members.emplace_back(entry.first, paint_settings::to_value(entry.second));
    }
    return object;
}

std::string paint_file(const Paint& paint) {
    return json::object({{"Paint", paint_value(paint)}}).to_json() + "\n";
}

std::expected<Paint, std::string> parse_paint_file(std::string_view text) {
    std::expected<Value, std::string> value = files::parse(text);
    if (!value) {
        return std::unexpected(std::move(value.error()));
    }
    if (std::expected<void, std::string> known = files::only(*value, {{"Paint"}}, "a model's paint file"); !known) {
        return std::unexpected(std::move(known.error()));
    }
    std::expected<const Value*, std::string> paint = files::member(*value, "Paint", "");
    if (!paint) {
        return std::unexpected(std::move(paint.error()));
    }
    return parse_paint(**paint, "Paint");
}

void set_slot(Paint& paint, std::span<const std::string> slots, std::string_view slot, const Parameters& parameters) {
    auto existing = std::ranges::find_if(paint, [&](const std::pair<std::string, Parameters>& entry) { return entry.first == slot; });
    if (existing != paint.end()) {
        existing->second = parameters;
        return;
    }
    paint.emplace_back(std::string(slot), parameters);
    std::ranges::stable_sort(paint, {}, [&](const std::pair<std::string, Parameters>& entry) {
        auto found = std::ranges::find(slots, entry.first);
        return found == slots.end() ? std::numeric_limits<size_t>::max() : static_cast<size_t>(found - slots.begin());
    });
}

} // namespace veehiicuul::models
