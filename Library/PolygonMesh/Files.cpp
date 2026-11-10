#include "PolygonMesh/Files.h"

#include "Base/Files.h"
#include "Base/Numbers.h"
#include "Base/Sum.h"
#include "Base/Text.h"
#include "Json/Json.h"

#include <algorithm>
#include <cmath>
#include <format>
#include <limits>

namespace polygon_mesh {

using json::Value;

namespace {

constexpr std::array<std::string_view, 4> Names{"Positions", "Faces", "Materials", "Creases"};
constexpr std::string_view Members = "Positions, Faces, Materials and Creases";

// The longest line that the materials take before they wrap.
constexpr size_t LineLength = 100;

// The float's shortest decimal that reads back as the same float, or null,
// which JSON has instead of infinities and NaN.
std::string number(float value) {
    return std::isfinite(value) ? base::display(value) : "null";
}

// Like "1 face" or "2 faces".
std::string plural(size_t count, std::string_view noun) {
    return count == 1 ? std::format("1 {}", noun) : std::format("{} {}s", count, noun);
}

std::string join(std::span<const std::string> items, std::string_view separator) {
    std::string text;
    for (size_t index = 0; index < items.size(); index++) {
        if (index > 0) {
            text += separator;
        }
        text += items[index];
    }
    return text;
}

std::string join(std::span<const std::uint32_t> numbers, std::string_view separator) {
    std::vector<std::string> items;
    items.reserve(numbers.size());
    for (std::uint32_t item : numbers) {
        items.push_back(std::to_string(item));
    }
    return join(items, separator);
}

// A member that's a list with an item on each line.
std::string nested(std::string_view name, std::span<const std::string> items) {
    if (items.empty()) {
        return std::format("    \"{}\": []", name);
    }
    return std::format("    \"{}\": [\n        {}\n    ]", name, join(items, ",\n        "));
}

// A member that's a list of numbers, on one line if it fits, and wrapped
// otherwise.
std::string wrapped(std::string_view name, std::span<const std::uint32_t> numbers) {
    std::string line = std::format("    \"{}\": [{}]", name, join(numbers, ", "));
    if (line.size() <= LineLength) {
        return line;
    }
    std::vector<std::string> lines;
    for (std::uint32_t item_number : numbers) {
        std::string item = std::to_string(item_number);
        // The indent, the line so far, a comma and a space, and the number,
        // with room for the comma after it.
        if (!lines.empty() && 8 + lines.back().size() + 2 + item.size() < LineLength) {
            lines.back() += ", ";
            lines.back() += item;
        } else {
            lines.push_back(std::move(item));
        }
    }
    return std::format("    \"{}\": [\n        {}\n    ]", name, join(lines, ",\n        "));
}

// A coordinate, if it's a number that a float can hold.
std::optional<float> coordinate(const Value& value) {
    const Value::Number* found = base::as<Value::Number>(value);
    if (found != nullptr && std::abs(found->value) <= static_cast<double>(std::numeric_limits<float>::max())) {
        return static_cast<float>(found->value);
    }
    return std::nullopt;
}

std::optional<std::uint32_t> whole_number(const Value& value) {
    const Value::Number* found = base::as<Value::Number>(value);
    if (found != nullptr && found->value - std::trunc(found->value) == 0.0 && found->value >= 0.0 &&
        found->value <= static_cast<double>(std::numeric_limits<std::uint32_t>::max())) {
        return base::saturating_cast<std::uint32_t>(found->value);
    }
    return std::nullopt;
}

// A vertex's number, or what's wrong with it, to follow what it's in.
std::expected<std::uint32_t, std::string> vertex_number(const Value& value, size_t count) {
    std::optional<std::uint32_t> vertex = whole_number(value);
    if (!vertex) {
        return std::unexpected(std::format("has {}, which isn't a vertex's number.", value.describe()));
    }
    if (*vertex >= count) {
        return std::unexpected(std::format("uses vertex {}, but there are only {} positions.", *vertex, count));
    }
    return *vertex;
}

} // namespace

std::string Mesh::to_json() const {
    std::vector<std::string> position_items;
    position_items.reserve(positions.size());
    for (const math::Vector3& position : positions) {
        position_items.push_back(std::format("[{}, {}, {}]", number(position[0]), number(position[1]), number(position[2])));
    }
    std::vector<std::string> face_items;
    face_items.reserve(faces.size());
    for (const Face& face : faces) {
        face_items.push_back(std::format("[{}]", join(face.vertices, ", ")));
    }
    std::vector<std::string> crease_items;
    crease_items.reserve(creases.size());
    for (const auto& [edge, sharpness] : creases) {
        auto [a, b] = edge.vertices();
        crease_items.push_back(std::format("[{}, {}, {}]", a, b, number(sharpness)));
    }
    std::vector<std::uint32_t> materials;
    materials.reserve(faces.size());
    for (const Face& face : faces) {
        materials.push_back(face.material);
    }
    std::string text = "{\n";
    text += nested("Positions", position_items);
    text += ",\n";
    text += nested("Faces", face_items);
    text += ",\n";
    text += wrapped("Materials", materials);
    text += ",\n";
    text += nested("Creases", crease_items);
    text += "\n}\n";
    return text;
}

std::expected<Mesh, std::string> Mesh::from_json(std::string_view text) {
    std::expected<Value, json::Error> parsed = json::parse(text);
    if (!parsed) {
        const json::Error& error = parsed.error();
        return std::unexpected(std::format("It isn't valid JSON, at line {}, column {}: {}.", error.line, error.column, error.message));
    }
    const Value& value = *parsed;
    const Value::Object* object = base::as<Value::Object>(value);
    if (object == nullptr) {
        return std::unexpected(std::format("A mesh must be an object with {}, not {}.", Members, value.describe()));
    }
    for (const auto& [name, member] : object->members) {
        if (!std::ranges::contains(Names, std::string_view(name))) {
            return std::unexpected(std::format("{} isn't part of a mesh, which has {}.", name, Members));
        }
    }
    auto list = [&](std::string_view name, bool required) -> std::expected<std::span<const Value>, std::string> {
        const Value* member = value.get(name);
        if (member == nullptr) {
            if (!required) {
                return std::span<const Value>();
            }
            return std::unexpected(std::format("The mesh has no {}.", name));
        }
        if (const Value::Array* items = base::as<Value::Array>(*member)) {
            return std::span<const Value>(items->values);
        }
        return std::unexpected(std::format("{} must be a list, not {}.", name, member->describe()));
    };

    Mesh mesh;
    std::expected<std::span<const Value>, std::string> position_items = list("Positions", true);
    if (!position_items) {
        return std::unexpected(position_items.error());
    }
    for (size_t index = 0; index < position_items->size(); index++) {
        const Value& item = (*position_items)[index];
        std::optional<std::vector<float>> position;
        const Value::Array* coordinates = base::as<Value::Array>(item);
        if (coordinates != nullptr && coordinates->values.size() == 3) {
            position.emplace();
            position->reserve(3);
            for (const Value& axis : coordinates->values) {
                std::optional<float> held = coordinate(axis);
                if (!held) {
                    position.reset();
                    break;
                }
                position->push_back(*held);
            }
        }
        if (!position || position->size() != 3) {
            return std::unexpected(std::format("Positions[{}] must be a list of 3 numbers that a float can hold, X, Y and Z, not {}.", index, item.describe()));
        }
        mesh.add_vertex({(*position)[0], (*position)[1], (*position)[2]});
    }
    size_t count = mesh.positions.size();
    std::expected<std::span<const Value>, std::string> face_items = list("Faces", true);
    if (!face_items) {
        return std::unexpected(face_items.error());
    }
    for (size_t index = 0; index < face_items->size(); index++) {
        const Value& item = (*face_items)[index];
        const Value::Array* items = base::as<Value::Array>(item);
        if (items == nullptr) {
            return std::unexpected(std::format("Faces[{}] must be a list of its vertices' numbers, not {}.", index, item.describe()));
        }
        if (items->values.size() < 3) {
            return std::unexpected(std::format("Faces[{}] has {} vertices, but a face needs at least 3.", index, items->values.size()));
        }
        std::vector<std::uint32_t> vertices;
        vertices.reserve(items->values.size());
        for (const Value& entry : items->values) {
            std::expected<std::uint32_t, std::string> vertex = vertex_number(entry, count);
            if (!vertex) {
                return std::unexpected(std::format("Faces[{}] {}", index, vertex.error()));
            }
            if (std::ranges::contains(vertices, *vertex)) {
                return std::unexpected(std::format("Faces[{}] uses vertex {} twice.", index, *vertex));
            }
            vertices.push_back(*vertex);
        }
        mesh.add_face(std::move(vertices), 0);
    }
    std::expected<std::span<const Value>, std::string> materials = list("Materials", false);
    if (!materials) {
        return std::unexpected(materials.error());
    }
    if (value.get("Materials") != nullptr && materials->size() != mesh.faces.size()) {
        return std::unexpected(std::format("Materials has {}, but the mesh has {}, and each needs one.", plural(materials->size(), "material"), plural(mesh.faces.size(), "face")));
    }
    for (size_t index = 0; index < materials->size(); index++) {
        const Value& item = (*materials)[index];
        std::optional<std::uint32_t> material = whole_number(item);
        if (!material) {
            return std::unexpected(std::format("Materials[{}] must be a whole number from 0 to {}, not {}.", index, std::numeric_limits<std::uint32_t>::max(), item.describe()));
        }
        mesh.faces[index].material = *material;
    }
    std::set<Edge> edges = mesh.edges();
    std::expected<std::span<const Value>, std::string> crease_items = list("Creases", false);
    if (!crease_items) {
        return std::unexpected(crease_items.error());
    }
    for (size_t index = 0; index < crease_items->size(); index++) {
        const Value& item = (*crease_items)[index];
        auto shape = [&] {
            return std::format("Creases[{}] must be a list of an edge's two vertices and its sharpness, like [0, 1, 2.5], not {}.", index, item.describe());
        };
        const Value::Array* items = base::as<Value::Array>(item);
        if (items == nullptr) {
            return std::unexpected(shape());
        }
        const Value::Number* sharpness = items->values.size() == 3 ? base::as<Value::Number>(items->values[2]) : nullptr;
        if (sharpness == nullptr) {
            return std::unexpected(shape());
        }
        std::expected<std::uint32_t, std::string> first = vertex_number(items->values[0], count);
        std::expected<std::uint32_t, std::string> second = vertex_number(items->values[1], count);
        if (!first) {
            return std::unexpected(std::format("Creases[{}] {}", index, first.error()));
        }
        if (!second) {
            return std::unexpected(std::format("Creases[{}] {}", index, second.error()));
        }
        Edge edge(*first, *second);
        auto [a, b] = edge.vertices();
        if (a == b || !edges.contains(edge)) {
            return std::unexpected(std::format("Creases[{}] is from vertex {} to {}, but no face has that edge.", index, a, b));
        }
        float fine = static_cast<float>(sharpness->value);
        if (!(fine > 0.0f && fine <= InfinitelySharp)) {
            return std::unexpected(
                std::format("Creases[{}]'s sharpness must be more than 0 and at most {}, not {}.", index, base::display(InfinitelySharp), base::display(sharpness->value)));
        }
        if (!mesh.creases.emplace(edge, fine).second) {
            return std::unexpected(std::format("Creases[{}] creases the edge from {} to {} again.", index, a, b));
        }
    }
    if (std::expected<void, std::string> valid = mesh.validate(); !valid) {
        return std::unexpected(valid.error());
    }
    return mesh;
}

std::expected<void, std::string> Mesh::save(const std::filesystem::path& path) const {
    std::expected<void, std::string> written = base::write(path, std::string_view(to_json()));
    if (!written) {
        return std::unexpected(std::format("Couldn't save {}: {}", base::display(path), written.error()));
    }
    return {};
}

std::expected<Mesh, std::string> Mesh::load(const std::filesystem::path& path) {
    std::expected<std::string, std::string> text = base::read_to_string(path);
    if (!text) {
        return std::unexpected(std::format("Couldn't read {}: {}", base::display(path), text.error()));
    }
    std::expected<Mesh, std::string> mesh = from_json(*text);
    if (!mesh) {
        return std::unexpected(std::format("{}: {}", base::display(path), mesh.error()));
    }
    return mesh;
}

std::string Mesh::to_obj() const {
    std::string text = std::format("# {} vertices and {} faces, from KGames' PolygonMesh\n", positions.size(), faces.size());
    for (const math::Vector3& position : positions) {
        text += std::format("v {} {} {}\n", number(position[0]), number(position[1]), number(position[2]));
    }
    std::optional<std::uint32_t> material;
    for (const Face& face : faces) {
        if (material != face.material) {
            text += std::format("usemtl Material{}\n", face.material);
            material = face.material;
        }
        std::vector<std::uint32_t> vertices;
        vertices.reserve(face.vertices.size());
        for (std::uint32_t vertex : face.vertices) {
            vertices.push_back(vertex + 1);
        }
        text += std::format("f {}\n", join(vertices, " "));
    }
    return text;
}

} // namespace polygon_mesh
