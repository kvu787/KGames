#include "Gltf/Gltf.h"

#include "Base/Files.h"
#include "Base/Numbers.h"
#include "Base/Sum.h"
#include "Base/Text.h"

#include <algorithm>
#include <bit>
#include <cmath>
#include <format>
#include <limits>
#include <ranges>
#include <string_view>

namespace gltf {

namespace {

using json::Value;

constexpr std::uint32_t Magic = 0x46546C67;
constexpr std::uint32_t JsonChunk = 0x4E4F534A;
constexpr std::uint32_t BinaryChunk = 0x004E4942;

// What Rust's standard library has and C++'s doesn't: u32::from_le_bytes,
// usize::next_multiple_of, checked_add, checked_mul, and str::from_utf8's
// check.
std::uint32_t u32_from_le_bytes(std::span<const std::uint8_t> bytes) {
    return static_cast<std::uint32_t>(bytes[0]) | static_cast<std::uint32_t>(bytes[1]) << 8 | static_cast<std::uint32_t>(bytes[2]) << 16 |
           static_cast<std::uint32_t>(bytes[3]) << 24;
}

constexpr size_t next_multiple_of(size_t value, size_t multiple) {
    size_t remainder = value % multiple;
    return remainder == 0 ? value : value + (multiple - remainder);
}

constexpr std::optional<size_t> checked_add(size_t a, size_t b) {
    return a <= std::numeric_limits<size_t>::max() - b ? std::optional<size_t>(a + b) : std::nullopt;
}

constexpr std::optional<size_t> checked_mul(size_t a, size_t b) {
    return b == 0 || a <= std::numeric_limits<size_t>::max() / b ? std::optional<size_t>(a * b) : std::nullopt;
}

bool is_utf8(std::string_view text) {
    while (!text.empty()) {
        base::Decoded decoded = base::decode_char(text);
        // U+FFFD is what bytes that aren't UTF-8 decode to, one at a time,
        // and what the three bytes of a real U+FFFD decode to.
        if (decoded.character == U'�' && decoded.length == 1) {
            return false;
        }
        text.remove_prefix(decoded.length);
    }
    return true;
}

// The member `name` of `value`, if it's this kind of value, or null.
template <class T> const T* member(const Value& value, std::string_view name) {
    const Value* found = value.get(name);
    return found != nullptr ? base::as<T>(*found) : nullptr;
}

// The member `name` of `value`, as a list, or an empty one.
std::span<const Value> list(const Value& value, std::string_view name) {
    if (const Value::Array* items = member<Value::Array>(value, name)) {
        return items->values;
    }
    return {};
}

// The most elements an accessor may have: more than any model here, and
// few enough to allocate without a thought.
constexpr size_t MaxElements = size_t{1} << 24;

// A whole number that's 0 or more, at most u32::MAX, as glTF's indices and
// sizes are.
std::optional<size_t> index(const Value& value, std::string_view name) {
    if (const Value::Number* number = member<Value::Number>(value, name)) {
        double whole = number->value;
        if (whole >= 0.0 && whole <= static_cast<double>(std::numeric_limits<std::uint32_t>::max()) && whole - std::trunc(whole) == 0.0) {
            return static_cast<size_t>(whole);
        }
    }
    return std::nullopt;
}

template <size_t N> std::expected<std::array<double, N>, std::string> numbers(const Value* value, std::array<double, N> default_) {
    if (value == nullptr) {
        return default_;
    }
    if (const Value::Array* items = base::as<Value::Array>(*value); items != nullptr && items->values.size() == N) {
        std::array<double, N> result = default_;
        for (size_t slot = 0; slot < N; slot++) {
            const Value& item = items->values[slot];
            const Value::Number* number = base::as<Value::Number>(item);
            if (number == nullptr || !std::isfinite(number->value)) {
                return std::unexpected(std::format("has a list of {} numbers with {}", N, item.describe()));
            }
            result[slot] = number->value;
        }
        return result;
    }
    return std::unexpected(std::format("has {} where a list of {} numbers should be", value->describe(), N));
}

// A 4 by 4 matrix, as glTF stores them: column by column.
using Matrix = std::array<double, 16>;

constexpr Matrix Identity{1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0};

Matrix multiply(const Matrix& a, const Matrix& b) {
    Matrix result{};
    for (size_t column = 0; column < 4; column++) {
        for (size_t row = 0; row < 4; row++) {
            // From -0.0, where Rust's sum of floats starts, so that a sum of
            // negative zeros is one, as it is there.
            double sum = -0.0;
            for (size_t k = 0; k < 4; k++) {
                sum += a[k * 4 + row] * b[column * 4 + k];
            }
            result[column * 4 + row] = sum;
        }
    }
    return result;
}

// A node's own transform, from its matrix, or its translation, rotation and
// scale.
std::expected<Matrix, std::string> local_transform(const Value& node) {
    if (node.get("matrix") != nullptr) {
        return numbers<16>(node.get("matrix"), Identity);
    }
    auto translation = numbers<3>(node.get("translation"), {0.0, 0.0, 0.0});
    if (!translation) {
        return std::unexpected(translation.error());
    }
    auto [tx, ty, tz] = *translation;
    auto quaternion = numbers<4>(node.get("rotation"), {0.0, 0.0, 0.0, 1.0});
    if (!quaternion) {
        return std::unexpected(quaternion.error());
    }
    auto [x, y, z, w] = *quaternion;
    auto scales = numbers<3>(node.get("scale"), {1.0, 1.0, 1.0});
    if (!scales) {
        return std::unexpected(scales.error());
    }
    auto [sx, sy, sz] = *scales;
    // The rotation's columns, from its quaternion.
    std::array<std::array<double, 3>, 3> rotation{{
        {1.0 - 2.0 * (y * y + z * z), 2.0 * (x * y + z * w), 2.0 * (x * z - y * w)},
        {2.0 * (x * y - z * w), 1.0 - 2.0 * (x * x + z * z), 2.0 * (y * z + x * w)},
        {2.0 * (x * z + y * w), 2.0 * (y * z - x * w), 1.0 - 2.0 * (x * x + y * y)},
    }};
    std::array<double, 3> scale{sx, sy, sz};
    Matrix matrix = Identity;
    for (size_t column = 0; column < 3; column++) {
        for (size_t row = 0; row < 3; row++) {
            matrix[column * 4 + row] = rotation[column][row] * scale[column];
        }
    }
    matrix[12] = tx;
    matrix[13] = ty;
    matrix[14] = tz;
    return matrix;
}

std::expected<size_t, std::string> index_of(const Value& value, std::string_view name, size_t accessor) {
    std::optional<size_t> found = index(value, name);
    if (!found) {
        return std::unexpected(std::format("has accessor {} without a {}", accessor, name));
    }
    return *found;
}

// An accessor's elements, each `N` numbers, as doubles.
template <size_t N> std::expected<std::vector<std::array<double, N>>, std::string> accessor(const Value& document, std::span<const std::uint8_t> binary, size_t which) {
    std::span<const Value> accessors = list(document, "accessors");
    if (which >= accessors.size()) {
        return std::unexpected(std::format("has no accessor {}", which));
    }
    const Value& accessor = accessors[which];
    size_t components = 0;
    if (const Value::String* kind = member<Value::String>(accessor, "type")) {
        if (kind->value == "SCALAR") {
            components = 1;
        } else if (kind->value == "VEC2") {
            components = 2;
        } else if (kind->value == "VEC3") {
            components = 3;
        } else if (kind->value == "VEC4") {
            components = 4;
        } else if (kind->value == "MAT4") {
            components = 16;
        }
    }
    if (components != N) {
        return std::unexpected(std::format("has accessor {} with {} numbers an element, not {}", which, components, N));
    }
    if (accessor.get("sparse") != nullptr) {
        return std::unexpected(std::format("has accessor {}, which is sparse, which isn't supported", which));
    }
    std::expected<size_t, std::string> count_read = index_of(accessor, "count", which);
    if (!count_read) {
        return std::unexpected(count_read.error());
    }
    size_t count = *count_read;
    std::expected<size_t, std::string> component_type_read = index_of(accessor, "componentType", which);
    if (!component_type_read) {
        return std::unexpected(component_type_read.error());
    }
    size_t component_type = *component_type_read;
    const Value::Bool* normalized_member = member<Value::Bool>(accessor, "normalized");
    bool normalized = normalized_member != nullptr && normalized_member->value;
    size_t size = 0;
    switch (component_type) {
    case 5120:
    case 5121:
        size = 1;
        break;
    case 5122:
    case 5123:
        size = 2;
        break;
    case 5125:
    case 5126:
        size = 4;
        break;
    default:
        return std::unexpected(std::format("has accessor {} of component type {}", which, component_type));
    }
    if (count > MaxElements) {
        return std::unexpected(std::format("has accessor {} with {} elements, more than {}", which, count, MaxElements));
    }
    std::optional<size_t> view_index = index(accessor, "bufferView");
    if (!view_index) {
        // Without a buffer view, every element is zero.
        return std::vector<std::array<double, N>>(count);
    }
    std::span<const Value> views = list(document, "bufferViews");
    if (*view_index >= views.size()) {
        return std::unexpected(std::format("has no buffer view {}", *view_index));
    }
    const Value& view = views[*view_index];
    if (index(view, "buffer") != size_t{0}) {
        return std::unexpected(std::format("has buffer view {} outside the file's binary chunk", *view_index));
    }
    auto past_the_end = [&] {
        return std::unexpected(std::format("has accessor {} past the end of the binary chunk", which));
    };
    std::optional<size_t> start_checked = checked_add(index(view, "byteOffset").value_or(0), index(accessor, "byteOffset").value_or(0));
    if (!start_checked) {
        return past_the_end();
    }
    size_t start = *start_checked;
    size_t stride = index(view, "byteStride").value_or(size * N);
    if (stride < size * N) {
        return std::unexpected(std::format("has buffer view {} with a stride of {}, less than its elements", *view_index, stride));
    }
    // Where the last element ends, which must be in the chunk, so that the
    // reads below can't be past it.
    if (count > 0) {
        std::optional<size_t> end =
            checked_mul(count - 1, stride).and_then([&](size_t last) { return checked_add(last, size * N); }).and_then([&](size_t length) { return checked_add(start, length); });
        if (!end) {
            return past_the_end();
        }
        if (*end > binary.size()) {
            return past_the_end();
        }
    }
    std::vector<std::array<double, N>> elements;
    elements.reserve(count);
    for (size_t element = 0; element < count; element++) {
        std::array<double, N> values{};
        for (size_t component = 0; component < N; component++) {
            size_t at = start + element * stride + component * size;
            std::span<const std::uint8_t> bytes = binary.subspan(at, size);
            double value = 0.0;
            switch (component_type) {
            case 5120: {
                double raw = static_cast<double>(static_cast<std::int8_t>(bytes[0]));
                value = normalized ? std::max(raw / 127.0, -1.0) : raw;
                break;
            }
            case 5121: {
                double raw = static_cast<double>(bytes[0]);
                value = normalized ? raw / 255.0 : raw;
                break;
            }
            case 5122: {
                double raw = static_cast<double>(static_cast<std::int16_t>(static_cast<std::uint16_t>(bytes[0] | bytes[1] << 8)));
                value = normalized ? std::max(raw / 32767.0, -1.0) : raw;
                break;
            }
            case 5123: {
                double raw = static_cast<double>(static_cast<std::uint16_t>(bytes[0] | bytes[1] << 8));
                value = normalized ? raw / 65535.0 : raw;
                break;
            }
            case 5125:
                value = static_cast<double>(u32_from_le_bytes(bytes));
                break;
            default:
                value = static_cast<double>(std::bit_cast<float>(u32_from_le_bytes(bytes)));
                break;
            }
            values[component] = value;
        }
        elements.push_back(values);
    }
    return elements;
}

std::uint32_t default_material(Model& model) {
    for (size_t index = 0; index < model.material_list.size(); index++) {
        if (model.material_list[index].name == "Default") {
            return static_cast<std::uint32_t>(index);
        }
    }
    model.material_list.push_back(Material{
        .name = "Default",
        .base_color = {1.0f, 1.0f, 1.0f, 1.0f},
        .extras = std::nullopt,
    });
    return static_cast<std::uint32_t>(model.material_list.size() - 1);
}

double determinant(const std::array<std::array<double, 3>, 3>& m) {
    return m[0][0] * (m[1][1] * m[2][2] - m[2][1] * m[1][2]) - m[1][0] * (m[0][1] * m[2][2] - m[2][1] * m[0][2]) + m[2][0] * (m[0][1] * m[1][2] - m[1][1] * m[0][2]);
}

// The inverse transpose of a matrix stored column by column, as columns.
std::array<std::array<double, 3>, 3> inverse_transpose(const std::array<std::array<double, 3>, 3>& m, double determinant) {
    // The cofactor matrix is the inverse's transpose, times the determinant.
    auto cofactor = [&](size_t column, size_t row) {
        size_t c1 = (column + 1) % 3;
        size_t c2 = (column + 2) % 3;
        size_t r1 = (row + 1) % 3;
        size_t r2 = (row + 2) % 3;
        return m[c1][r1] * m[c2][r2] - m[c2][r1] * m[c1][r2];
    };
    std::array<std::array<double, 3>, 3> result{};
    for (size_t column = 0; column < 3; column++) {
        for (size_t row = 0; row < 3; row++) {
            result[column][row] = cofactor(column, row) / determinant;
        }
    }
    return result;
}

std::array<float, 3> face_normal(const std::array<std::array<float, 3>, 3>& corners) {
    const std::array<float, 3>& a = corners[0];
    const std::array<float, 3>& b = corners[1];
    const std::array<float, 3>& c = corners[2];
    std::array<float, 3> u{b[0] - a[0], b[1] - a[1], b[2] - a[2]};
    std::array<float, 3> v{c[0] - a[0], c[1] - a[1], c[2] - a[2]};
    std::array<float, 3> normal{u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0]};
    float length = std::sqrt(normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]);
    if (length > 0.0f) {
        return {normal[0] / length, normal[1] / length, normal[2] / length};
    }
    return {0.0f, 1.0f, 0.0f};
}

std::expected<void, std::string> add_primitive(const Value& document, std::span<const std::uint8_t> binary, const Value& primitive, const Matrix& transform, Model& model) {
    if (std::optional<size_t> mode = index(primitive, "mode"); mode && *mode != 4) {
        return std::unexpected("has a primitive that isn't triangles");
    }
    const Value* attributes = primitive.get("attributes");
    if (attributes == nullptr) {
        return std::unexpected("has a primitive without attributes");
    }
    std::optional<size_t> position_accessor = index(*attributes, "POSITION");
    if (!position_accessor) {
        return std::unexpected("has a primitive without positions");
    }
    std::expected<std::vector<std::array<double, 3>>, std::string> positions_read = accessor<3>(document, binary, *position_accessor);
    if (!positions_read) {
        return std::unexpected(positions_read.error());
    }
    const std::vector<std::array<double, 3>>& positions = *positions_read;
    std::optional<std::vector<std::array<double, 3>>> normals;
    if (std::optional<size_t> normal_accessor = index(*attributes, "NORMAL")) {
        std::expected<std::vector<std::array<double, 3>>, std::string> normals_read = accessor<3>(document, binary, *normal_accessor);
        if (!normals_read) {
            return std::unexpected(normals_read.error());
        }
        normals = std::move(*normals_read);
    }
    std::vector<size_t> indices;
    if (std::optional<size_t> index_accessor = index(primitive, "indices")) {
        std::expected<std::vector<std::array<double, 1>>, std::string> indices_read = accessor<1>(document, binary, *index_accessor);
        if (!indices_read) {
            return std::unexpected(indices_read.error());
        }
        indices.reserve(indices_read->size());
        for (const std::array<double, 1>& element : *indices_read) {
            indices.push_back(base::saturating_cast<size_t>(element[0]));
        }
    } else {
        indices.reserve(positions.size());
        for (size_t position = 0; position < positions.size(); position++) {
            indices.push_back(position);
        }
    }
    if (indices.size() % 3 != 0 || std::ranges::any_of(indices, [&](size_t index) { return index >= positions.size(); })) {
        return std::unexpected("has a primitive whose indices aren't whole triangles of its vertices");
    }
    std::uint32_t material = 0;
    if (std::optional<size_t> material_index = index(primitive, "material")) {
        if (*material_index >= model.material_list.size()) {
            return std::unexpected(std::format("has no material {}", *material_index));
        }
        material = static_cast<std::uint32_t>(*material_index);
    } else {
        material = default_material(model);
    }

    // Positions go through the transform, and normals through its inverse
    // transpose, which keeps them at right angles to the surface under any
    // scale. A transform that mirrors turns triangles inside out, so their
    // winding is reversed to keep their fronts out.
    std::array<std::array<double, 3>, 3> linear{{
        {transform[0], transform[1], transform[2]},
        {transform[4], transform[5], transform[6]},
        {transform[8], transform[9], transform[10]},
    }};
    auto place = [&](const std::array<double, 3>& position) -> std::array<float, 3> {
        auto [x, y, z] = position;
        std::array<double, 3> result{transform[12], transform[13], transform[14]};
        for (size_t row = 0; row < 3; row++) {
            result[row] += linear[0][row] * x + linear[1][row] * y + linear[2][row] * z;
        }
        return {static_cast<float>(result[0]), static_cast<float>(result[1]), static_cast<float>(result[2])};
    };
    double linear_determinant = determinant(linear);
    if (std::abs(linear_determinant) < 1e-12) {
        return std::unexpected("has a node whose transform flattens its mesh");
    }
    std::array<std::array<double, 3>, 3> normal_matrix = inverse_transpose(linear, linear_determinant);
    auto turn = [&](const std::array<double, 3>& normal) -> std::array<float, 3> {
        auto [x, y, z] = normal;
        std::array<double, 3> result{};
        for (size_t row = 0; row < 3; row++) {
            result[row] = normal_matrix[0][row] * x + normal_matrix[1][row] * y + normal_matrix[2][row] * z;
        }
        double length = std::sqrt(result[0] * result[0] + result[1] * result[1] + result[2] * result[2]);
        if (length > 0.0) {
            return {static_cast<float>(result[0] / length), static_cast<float>(result[1] / length), static_cast<float>(result[2] / length)};
        }
        return {0.0f, 1.0f, 0.0f};
    };
    bool mirrored = linear_determinant < 0.0;
    std::uint32_t first = static_cast<std::uint32_t>(model.positions.size());
    if (normals) {
        if (normals->size() != positions.size()) {
            return std::unexpected("has a primitive with a different number of normals and positions");
        }
        model.positions.append_range(positions | std::views::transform(place));
        model.normals.append_range(*normals | std::views::transform(turn));
        model.materials.insert(model.materials.end(), positions.size(), material);
        for (size_t triangle = 0; triangle < indices.size(); triangle += 3) {
            std::uint32_t a = first + static_cast<std::uint32_t>(indices[triangle]);
            std::uint32_t b = first + static_cast<std::uint32_t>(indices[triangle + 1]);
            std::uint32_t c = first + static_cast<std::uint32_t>(indices[triangle + 2]);
            if (mirrored) {
                model.indices.insert(model.indices.end(), {a, c, b});
            } else {
                model.indices.insert(model.indices.end(), {a, b, c});
            }
        }
    } else {
        // Without normals, each triangle is flat, with its own vertices.
        for (size_t triangle = 0; triangle < indices.size(); triangle += 3) {
            std::array<std::array<float, 3>, 3> corners{
                place(positions[indices[triangle]]),
                place(positions[indices[triangle + 1]]),
                place(positions[indices[triangle + 2]]),
            };
            if (mirrored) {
                std::swap(corners[1], corners[2]);
            }
            std::array<float, 3> normal = face_normal(corners);
            std::uint32_t base = static_cast<std::uint32_t>(model.positions.size());
            model.positions.append_range(corners);
            model.normals.insert(model.normals.end(), 3, normal);
            model.materials.insert(model.materials.end(), 3, material);
            model.indices.insert(model.indices.end(), {base, base + 1, base + 2});
        }
    }
    return {};
}

// Adds the roots' trees of nodes, each node's children after it, with a
// stack of its own, so that a long chain of nodes can't overflow the
// thread's.
std::expected<void, std::string> add_nodes(const Value& document, std::span<const std::uint8_t> binary, std::vector<size_t> roots, Model& model) {
    std::span<const Value> nodes = list(document, "nodes");
    std::vector<bool> visited(nodes.size(), false);
    std::vector<std::pair<size_t, Matrix>> pending;
    pending.reserve(roots.size());
    for (size_t root : roots | std::views::reverse) {
        pending.emplace_back(root, Identity);
    }
    while (!pending.empty()) {
        auto [index_in_file, parent] = pending.back();
        pending.pop_back();
        if (index_in_file >= nodes.size()) {
            return std::unexpected(std::format("has no node {}", index_in_file));
        }
        const Value& node = nodes[index_in_file];
        if (visited[index_in_file]) {
            return std::unexpected(std::format("has node {} in more than one place, or in a loop", index_in_file));
        }
        visited[index_in_file] = true;
        std::expected<Matrix, std::string> local = local_transform(node);
        if (!local) {
            return std::unexpected(std::format("node {} {}", index_in_file, local.error()));
        }
        Matrix transform = multiply(parent, *local);
        if (std::optional<size_t> mesh_index = index(node, "mesh")) {
            std::span<const Value> meshes = list(document, "meshes");
            if (*mesh_index >= meshes.size()) {
                return std::unexpected(std::format("has no mesh {}", *mesh_index));
            }
            for (const Value& primitive : list(meshes[*mesh_index], "primitives")) {
                std::expected<void, std::string> added = add_primitive(document, binary, primitive, transform, model);
                if (!added) {
                    return added;
                }
            }
        }
        for (const Value& child : list(node, "children") | std::views::reverse) {
            if (const Value::Number* number = base::as<Value::Number>(child)) {
                pending.emplace_back(base::saturating_cast<size_t>(number->value), transform);
            }
        }
    }
    return {};
}

std::expected<Model, std::string> read(const Value& document, std::span<const std::uint8_t> binary) {
    Model model;
    for (const Value& material : list(document, "materials")) {
        const Value::String* name_member = member<Value::String>(material, "name");
        std::string name = name_member != nullptr ? name_member->value : std::format("Material {}", model.material_list.size());
        const Value* pbr = material.get("pbrMetallicRoughness");
        std::expected<std::array<double, 4>, std::string> color = numbers<4>(pbr != nullptr ? pbr->get("baseColorFactor") : nullptr, {1.0, 1.0, 1.0, 1.0});
        if (!color) {
            return std::unexpected(std::format("material {} {}", name, color.error()));
        }
        const Value* extras = material.get("extras");
        model.material_list.push_back(Material{
            .name = std::move(name),
            .base_color = {static_cast<float>((*color)[0]), static_cast<float>((*color)[1]), static_cast<float>((*color)[2]), static_cast<float>((*color)[3])},
            .extras = extras != nullptr ? std::optional<Value>(*extras) : std::nullopt,
        });
    }
    std::span<const Value> scenes = list(document, "scenes");
    size_t scene = index(document, "scene").value_or(0);
    std::vector<size_t> roots;
    if (scene < scenes.size()) {
        for (const Value& node : list(scenes[scene], "nodes")) {
            if (const Value::Number* number = base::as<Value::Number>(node)) {
                roots.push_back(base::saturating_cast<size_t>(number->value));
            }
        }
    } else {
        // Without scenes, every node that's no other's child is a root.
        std::span<const Value> nodes = list(document, "nodes");
        std::vector<size_t> children;
        for (const Value& node : nodes) {
            for (const Value& child : list(node, "children")) {
                if (const Value::Number* number = base::as<Value::Number>(child)) {
                    children.push_back(base::saturating_cast<size_t>(number->value));
                }
            }
        }
        for (size_t node = 0; node < nodes.size(); node++) {
            if (!std::ranges::contains(children, node)) {
                roots.push_back(node);
            }
        }
    }
    std::expected<void, std::string> added = add_nodes(document, binary, std::move(roots), model);
    if (!added) {
        return std::unexpected(added.error());
    }
    if (model.indices.empty()) {
        return std::unexpected("has no triangles");
    }
    return model;
}

} // namespace

std::pair<std::array<float, 3>, std::array<float, 3>> Model::bounds() const {
    constexpr float Infinity = std::numeric_limits<float>::infinity();
    std::array<float, 3> min_{Infinity, Infinity, Infinity};
    std::array<float, 3> max_{-Infinity, -Infinity, -Infinity};
    for (const std::array<float, 3>& position : positions) {
        for (size_t axis = 0; axis < 3; axis++) {
            // A position that's NaN leaves the box as it is, as f32::min and
            // f32::max leave it in Rust.
            min_[axis] = std::fmin(min_[axis], position[axis]);
            max_[axis] = std::fmax(max_[axis], position[axis]);
        }
    }
    return {min_, max_};
}

std::expected<Model, std::string> load(const std::filesystem::path& path) {
    std::expected<std::vector<std::uint8_t>, std::string> bytes = base::read(path);
    if (!bytes) {
        return std::unexpected(std::format("Couldn't read {}: {}", base::display(path), bytes.error()));
    }
    return parse(*bytes).transform_error([&](const std::string& problem) { return std::format("{}: {}", base::display(path), problem); });
}

std::expected<Model, std::string> parse(std::span<const std::uint8_t> bytes) {
    auto word = [&](size_t offset) -> std::expected<std::uint32_t, std::string> {
        if (offset + 4 > bytes.size()) {
            return std::unexpected("is cut short");
        }
        return u32_from_le_bytes(bytes.subspan(offset, 4));
    };
    std::expected<std::uint32_t, std::string> magic = word(0);
    if (!magic) {
        return std::unexpected(magic.error());
    }
    if (*magic != Magic) {
        return std::unexpected("isn't a binary glTF file");
    }
    std::expected<std::uint32_t, std::string> version = word(4);
    if (!version) {
        return std::unexpected(version.error());
    }
    if (*version != 2) {
        return std::unexpected(std::format("is glTF version {}, not 2", *version));
    }
    std::expected<std::uint32_t, std::string> file_length = word(8);
    if (!file_length) {
        return std::unexpected(file_length.error());
    }
    size_t length = std::min(static_cast<size_t>(*file_length), bytes.size());
    size_t offset = 12;
    std::optional<std::string_view> text;
    std::span<const std::uint8_t> binary;
    while (offset + 8 <= length) {
        std::expected<std::uint32_t, std::string> chunk_word = word(offset);
        if (!chunk_word) {
            return std::unexpected(chunk_word.error());
        }
        size_t chunk_length = *chunk_word;
        std::expected<std::uint32_t, std::string> kind = word(offset + 4);
        if (!kind) {
            return std::unexpected(kind.error());
        }
        if (offset + 8 + chunk_length > bytes.size()) {
            return std::unexpected("has a chunk that's cut short");
        }
        std::span<const std::uint8_t> data = bytes.subspan(offset + 8, chunk_length);
        if (*kind == JsonChunk && !text) {
            std::string_view chunk_text(reinterpret_cast<const char*>(data.data()), data.size());
            if (!is_utf8(chunk_text)) {
                return std::unexpected("has JSON that isn't UTF-8");
            }
            text = chunk_text;
        } else if (*kind == BinaryChunk && binary.empty()) {
            binary = data;
        }
        offset += 8 + next_multiple_of(chunk_length, 4);
    }
    if (!text) {
        return std::unexpected("has no JSON chunk");
    }
    std::expected<Value, json::Error> document = json::parse(*text);
    if (!document) {
        const json::Error& error = document.error();
        return std::unexpected(std::format("has JSON with a mistake at line {}, column {}: {}", error.line, error.column, error.message));
    }
    return read(*document, binary);
}

} // namespace gltf
