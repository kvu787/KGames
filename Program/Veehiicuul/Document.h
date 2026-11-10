// Everything that the editors change, and Save writes: the scenes, the
// vehicles, and the models' paints. The models' meshes come from Blender,
// so they're not part of it.

#pragma once

#include "SimplePaint/Material.h"
#include "Veehiicuul/Models.h"
#include "Veehiicuul/Scene.h"
#include "Veehiicuul/Vehicles.h"

#include <cstddef>
#include <expected>
#include <filesystem>
#include <optional>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

namespace veehiicuul::document {

struct Document {
    std::vector<scene::Scene> scenes;
    vehicles::Vehicles vehicles;
    // Each model's paint, by its name, in the models' order.
    std::vector<std::pair<std::string, models::Paint>> paints;

    // The document in the game's folder: Scenes, Vehicles.json, and each
    // model's paint file in Models, if it has one.
    static std::expected<Document, std::string> load(const std::filesystem::path& folder, const models::Models& models);

    // What the document names that isn't there: models of vehicles and
    // decorations, and parked vehicles.
    std::vector<std::string> missing(const models::Models& models) const;

    const models::Paint& paint(std::string_view model) const;

    models::Paint& paint_mut(std::string_view model);

    // A model's slots' materials, with its paint.
    std::vector<simple_paint::Parameters> model_materials(const models::Models& models, size_t model) const;

    // A vehicle's model, and its slots' materials, with its model's paint
    // and its own.
    std::optional<std::pair<size_t, std::vector<simple_paint::Parameters>>> vehicle_materials(const models::Models& models, size_t vehicle) const;

    // The files that differ from what's in `saved`, each as its path in the
    // game's folder and its text.
    std::vector<std::pair<std::string, std::string>> changed_files(const Document& saved) const;

    // The files that save should delete: scenes in `saved` that the document
    // no longer has, like after undoing a new scene.
    std::vector<std::string> removed_files(const Document& saved) const;

    bool operator==(const Document&) const = default;
};

} // namespace veehiicuul::document
