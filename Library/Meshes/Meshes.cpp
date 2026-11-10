#include "Meshes/Meshes.h"

#include "Base/Panic.h"

// The car, which the build made: Library/Meshes/Generator wrote it, as
// Meshes.vcxproj says.
#include "Car.h"

#include <cmath>
#include <cstddef>
#include <numbers>
#include <utility>

namespace meshes {

namespace car {

constexpr std::span<const simple_paint::Vertex> Vertices(generated::Vertices);
constexpr std::span<const std::uint32_t> Indices(generated::Indices);

} // namespace car

using simple_paint::Vertex;

namespace {

// A rectangle from center - u - v to center + u + v, facing u x v.
void add_quad(Mesh& mesh, std::array<float, 3> center, std::array<float, 3> u, std::array<float, 3> v, std::array<float, 3> normal, std::uint32_t material_index) {
    std::uint32_t first = static_cast<std::uint32_t>(mesh.vertices.size());
    for (auto [a, b] : {std::pair{-1.0f, -1.0f}, std::pair{1.0f, -1.0f}, std::pair{1.0f, 1.0f}, std::pair{-1.0f, 1.0f}}) {
        std::array<float, 3> position{};
        for (size_t i = 0; i < 3; i++) {
            position[i] = center[i] + a * u[i] + b * v[i];
        }
        mesh.vertices.push_back(Vertex{
            .position = position,
            .normal = normal,
            .material_index = material_index,
        });
    }
    mesh.indices.insert(mesh.indices.end(), {first, first + 1, first + 2, first, first + 2, first + 3});
}

} // namespace

Mesh::Mesh(std::vector<simple_paint::Vertex> mesh_vertices, std::vector<std::uint32_t> mesh_indices) : vertices(std::move(mesh_vertices)), indices(std::move(mesh_indices)) {}

Mesh uv_sphere(std::uint32_t longitudes, std::uint32_t latitudes, std::uint32_t material_index) {
    if (!(longitudes >= 4 && longitudes <= 512 && latitudes >= 2 && latitudes <= 512)) {
        base::panic("a UV sphere needs 4 to 512 longitudes and 2 to 512 latitudes");
    }
    auto point = [&](std::array<float, 3> position) {
        return Vertex{
            .position = position,
            .normal = position,
            .material_index = material_index,
        };
    };
    std::vector<Vertex> vertices;
    vertices.reserve(2 + longitudes * (latitudes - 1));
    std::vector<std::uint32_t> indices;
    indices.reserve(6 * longitudes * (latitudes - 1));
    vertices.push_back(point({0.0f, 1.0f, 0.0f}));
    for (std::uint32_t v = 1; v < latitudes; v++) {
        float latitude = std::numbers::pi_v<float> * static_cast<float>(v) / static_cast<float>(latitudes);
        for (std::uint32_t u = 0; u < longitudes; u++) {
            // Twice pi is tau, to the bit.
            float longitude = 2.0f * std::numbers::pi_v<float> * static_cast<float>(u) / static_cast<float>(longitudes);
            vertices.push_back(point({
                std::sin(latitude) * std::cos(longitude),
                std::cos(latitude),
                std::sin(latitude) * std::sin(longitude),
            }));
        }
    }
    std::uint32_t bottom = static_cast<std::uint32_t>(vertices.size());
    vertices.push_back(point({0.0f, -1.0f, 0.0f}));

    // Wrap longitude without duplicating the seam. Separate pole fans avoid
    // zero-area triangles, including at the minimum latitude count.
    for (std::uint32_t u = 0; u < longitudes; u++) {
        std::uint32_t next = (u + 1) % longitudes;
        indices.insert(indices.end(), {0, 1 + next, 1 + u});
        for (std::uint32_t v = 0; v < latitudes - 2; v++) {
            std::uint32_t a = 1 + v * longitudes + u;
            std::uint32_t b = 1 + v * longitudes + next;
            std::uint32_t c = a + longitudes;
            std::uint32_t d = b + longitudes;
            indices.insert(indices.end(), {a, b, c, b, d, c});
        }
        std::uint32_t last_ring = 1 + (latitudes - 2) * longitudes;
        indices.insert(indices.end(), {bottom, last_ring + u, last_ring + next});
    }
    return Mesh(std::move(vertices), std::move(indices));
}

Mesh cuboid(std::array<float, 3> size, std::uint32_t material_index) {
    std::array<float, 3> half{size[0] * 0.5f, size[1] * 0.5f, size[2] * 0.5f};
    Mesh mesh;
    mesh.vertices.reserve(24);
    mesh.indices.reserve(36);
    // Each face's normal, and two axes along it whose cross product is the
    // normal, so the corners go counterclockwise seen from outside.
    for (size_t axis = 0; axis < 3; axis++) {
        for (float sign : {1.0f, -1.0f}) {
            size_t u_axis = (axis + 1) % 3;
            size_t v_axis = (axis + 2) % 3;
            std::array<float, 3> normal{};
            normal[axis] = sign;
            std::array<float, 3> u{};
            u[u_axis] = half[u_axis] * sign;
            std::array<float, 3> v{};
            v[v_axis] = half[v_axis];
            std::array<float, 3> center{normal[0] * half[0], normal[1] * half[1], normal[2] * half[2]};
            add_quad(mesh, center, u, v, normal, material_index);
        }
    }
    return mesh;
}

Mesh plane(std::array<float, 2> size, std::uint32_t material_index) {
    Mesh mesh;
    mesh.vertices.reserve(4);
    mesh.indices.reserve(6);
    // Z cross X is +Y.
    std::array<float, 3> u{0.0f, 0.0f, size[1] * 0.5f};
    std::array<float, 3> v{size[0] * 0.5f, 0.0f, 0.0f};
    add_quad(mesh, {0.0f, 0.0f, 0.0f}, u, v, {0.0f, 1.0f, 0.0f}, material_index);
    return mesh;
}

} // namespace meshes
