#include "Veehiicuul/Document.h"

#include "Base/Files.h"
#include "Veehiicuul/Files.h"

#include <algorithm>
#include <format>
#include <system_error>

namespace veehiicuul::document {

using models::Model;
using models::Models;
using models::Paint;
using scene::Scene;
using simple_paint::Parameters;
using vehicles::Vehicle;
using vehicles::Vehicles;

std::expected<Document, std::string> Document::load(const std::filesystem::path& folder, const Models& models) {
    std::expected<std::vector<Scene>, std::string> loaded_scenes = Scene::load_all(folder / "Scenes");
    if (!loaded_scenes) {
        return std::unexpected(std::move(loaded_scenes.error()));
    }
    std::filesystem::path vehicles_path = folder / "Vehicles.json";
    std::expected<std::string, std::string> vehicles_text = files::read(vehicles_path);
    if (!vehicles_text) {
        return std::unexpected(std::move(vehicles_text.error()));
    }
    std::expected<Vehicles, std::string> loaded_vehicles = Vehicles::parse(*vehicles_text);
    if (!loaded_vehicles) {
        return std::unexpected(std::format("{}: {}", base::display(vehicles_path), loaded_vehicles.error()));
    }
    std::vector<std::pair<std::string, Paint>> loaded_paints;
    for (const Model& model : models.list) {
        std::filesystem::path path = folder / "Models" / base::path(std::format("{}.json", model.name));
        Paint model_paint;
        std::error_code error;
        if (std::filesystem::is_regular_file(path, error)) {
            std::expected<std::string, std::string> text = files::read(path);
            if (!text) {
                return std::unexpected(std::move(text.error()));
            }
            std::expected<Paint, std::string> parsed = models::parse_paint_file(*text);
            if (!parsed) {
                return std::unexpected(std::format("{}: {}", base::display(path), parsed.error()));
            }
            model_paint = std::move(*parsed);
        }
        loaded_paints.emplace_back(model.name, std::move(model_paint));
    }
    Document document{
        .scenes = std::move(*loaded_scenes),
        .vehicles = std::move(*loaded_vehicles),
        .paints = std::move(loaded_paints),
    };
    std::vector<std::string> problems = document.missing(models);
    if (!problems.empty()) {
        return std::unexpected(std::move(problems.front()));
    }
    return document;
}

std::vector<std::string> Document::missing(const Models& models) const {
    std::vector<std::string> problems;
    for (const Vehicle& vehicle : vehicles.list) {
        if (!models.find(vehicle.model)) {
            problems.push_back(std::format("Vehicle {}'s model, {}, isn't in Models", vehicle.name, vehicle.model));
        }
    }
    for (const Scene& scene : scenes) {
        for (const scene::Decoration& decoration : scene.decorations) {
            if (!models.find(decoration.model)) {
                problems.push_back(std::format("Scene {} has a {}, which isn't in Models", scene.name, decoration.model));
            }
        }
        for (const scene::Parked& parked : scene.parked) {
            if (!vehicles.find(parked.vehicle)) {
                problems.push_back(std::format("Scene {} parks a {}, which isn't in Vehicles.json", scene.name, parked.vehicle));
            }
        }
    }
    return problems;
}

const Paint& Document::paint(std::string_view model) const {
    static const Paint None;
    for (const std::pair<std::string, Paint>& entry : paints) {
        if (entry.first == model) {
            return entry.second;
        }
    }
    return None;
}

Paint& Document::paint_mut(std::string_view model) {
    for (std::pair<std::string, Paint>& entry : paints) {
        if (entry.first == model) {
            return entry.second;
        }
    }
    paints.emplace_back(std::string(model), Paint());
    return paints.back().second;
}

std::vector<Parameters> Document::model_materials(const Models& models, size_t model) const {
    const Model& found = models.list[model];
    return found.materials({{&paint(found.name)}});
}

std::optional<std::pair<size_t, std::vector<Parameters>>> Document::vehicle_materials(const Models& models, size_t vehicle) const {
    if (vehicle >= vehicles.list.size()) {
        return std::nullopt;
    }
    const Vehicle& found = vehicles.list[vehicle];
    std::optional<size_t> index = models.find(found.model);
    if (!index) {
        return std::nullopt;
    }
    const Model& model = models.list[*index];
    return std::pair(*index, model.materials({{&paint(model.name), &found.paint}}));
}

std::vector<std::pair<std::string, std::string>> Document::changed_files(const Document& saved) const {
    std::vector<std::pair<std::string, std::string>> files;
    for (const Scene& scene : scenes) {
        auto other = std::ranges::find_if(saved.scenes, [&](const Scene& candidate) { return candidate.name == scene.name; });
        if (other == saved.scenes.end() || !(*other == scene)) {
            files.emplace_back(std::format("Scenes/{}.json", scene.name), scene.to_json());
        }
    }
    if (vehicles != saved.vehicles) {
        files.emplace_back("Vehicles.json", vehicles.to_json());
    }
    for (const std::pair<std::string, Paint>& entry : paints) {
        if (saved.paint(entry.first) != entry.second) {
            files.emplace_back(std::format("Models/{}.json", entry.first), models::paint_file(entry.second));
        }
    }
    return files;
}

std::vector<std::string> Document::removed_files(const Document& saved) const {
    std::vector<std::string> removed;
    for (const Scene& scene : saved.scenes) {
        if (!std::ranges::any_of(scenes, [&](const Scene& kept) { return kept.name == scene.name; })) {
            removed.push_back(std::format("Scenes/{}.json", scene.name));
        }
    }
    return removed;
}

} // namespace veehiicuul::document
