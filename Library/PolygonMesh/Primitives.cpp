#include "PolygonMesh/Primitives.h"

#include "Base/Panic.h"

#include <algorithm>
#include <cmath>
#include <initializer_list>
#include <numbers>
#include <span>
#include <vector>

namespace polygon_mesh::primitives {

namespace {

constexpr float Pi = std::numbers::pi_v<float>;
constexpr float Tau = 2.0f * std::numbers::pi_v<float>;

// The radius of each ring from the bottom to the top.
std::vector<float> ring_radii(float bottom, float top, std::uint32_t segments) {
    std::vector<float> radii;
    radii.reserve(segments + 1);
    for (std::uint32_t ring = 0; ring <= segments; ring++) {
        radii.push_back(bottom + (top - bottom) * static_cast<float>(ring) / static_cast<float>(segments));
    }
    return radii;
}

// Rings of `sides` vertices from the bottom to the top, evenly up `height`,
// with quads between them, and either a cap or a tip at each end: a tip where
// a ring's radius is 0.
Mesh frustum(std::span<const float> radii, float height, std::uint32_t sides) {
    Mesh mesh;
    std::uint32_t rings = static_cast<std::uint32_t>(radii.size());
    // A ring of radius 0 at an end is a single vertex, its tip.
    std::vector<std::uint32_t> ring_start;
    ring_start.reserve(radii.size());
    for (size_t ring = 0; ring < radii.size(); ring++) {
        float radius = radii[ring];
        float y = height * (static_cast<float>(ring) / static_cast<float>(rings - 1) - 0.5f);
        ring_start.push_back(static_cast<std::uint32_t>(mesh.positions.size()));
        if (radius == 0.0f) {
            mesh.add_vertex({0.0f, y, 0.0f});
            continue;
        }
        for (std::uint32_t side = 0; side < sides; side++) {
            float angle = Tau * static_cast<float>(side) / static_cast<float>(sides);
            mesh.add_vertex({radius * std::cos(angle), y, radius * std::sin(angle)});
        }
    }
    auto vertex = [&](size_t ring, std::uint32_t side) {
        return radii[ring] == 0.0f ? ring_start[ring] : ring_start[ring] + side % sides;
    };
    for (size_t ring = 0; ring < radii.size() - 1; ring++) {
        for (std::uint32_t side = 0; side < sides; side++) {
            // Around from +X toward +Z, the bottom, then the top: outward.
            std::vector<std::uint32_t> face{vertex(ring, side), vertex(ring + 1, side), vertex(ring + 1, side + 1), vertex(ring, side + 1)};
            face.erase(std::unique(face.begin(), face.end()), face.end());
            if (face.front() == face.back()) {
                face.pop_back();
            }
            mesh.add_face(std::move(face), 0);
        }
    }
    // From +X toward +Z faces -Y, which is out of the bottom cap.
    if (radii[0] > 0.0f) {
        std::vector<std::uint32_t> cap;
        cap.reserve(sides);
        for (std::uint32_t side = 0; side < sides; side++) {
            cap.push_back(vertex(0, side));
        }
        mesh.add_face(std::move(cap), 0);
    }
    size_t top = radii.size() - 1;
    if (radii[top] > 0.0f) {
        std::vector<std::uint32_t> cap;
        cap.reserve(sides);
        for (std::uint32_t side = sides; side-- > 0;) {
            cap.push_back(vertex(top, side));
        }
        mesh.add_face(std::move(cap), 0);
    }
    return mesh;
}

} // namespace

Mesh cube(float size) {
    float h = size * 0.5f;
    Mesh mesh;
    // Around the -Z side, then around the +Z side.
    for (math::Vector3 position : std::initializer_list<math::Vector3>{{-h, -h, -h}, {h, -h, -h}, {h, h, -h}, {-h, h, -h}, {-h, -h, h}, {h, -h, h}, {h, h, h}, {-h, h, h}}) {
        mesh.add_vertex(position);
    }
    // -Z, +Z, -Y, +Y, -X and +X.
    for (std::array<std::uint32_t, 4> face :
         std::initializer_list<std::array<std::uint32_t, 4>>{{0, 3, 2, 1}, {4, 5, 6, 7}, {0, 1, 5, 4}, {3, 7, 6, 2}, {0, 4, 7, 3}, {1, 2, 6, 5}}) {
        mesh.add_face(std::vector<std::uint32_t>(face.begin(), face.end()), 0);
    }
    return mesh;
}

Mesh plane(std::array<float, 2> size, std::array<std::uint32_t, 2> segments) {
    if (!(segments[0] >= 1 && segments[1] >= 1)) {
        base::panic("a plane needs at least 1 segment along X and Z");
    }
    auto [columns, rows] = segments;
    Mesh mesh;
    for (std::uint32_t row = 0; row <= rows; row++) {
        for (std::uint32_t column = 0; column <= columns; column++) {
            mesh.add_vertex({
                size[0] * (static_cast<float>(column) / static_cast<float>(columns) - 0.5f),
                0.0f,
                size[1] * (static_cast<float>(row) / static_cast<float>(rows) - 0.5f),
            });
        }
    }
    auto vertex = [&](std::uint32_t column, std::uint32_t row) {
        return row * (columns + 1) + column;
    };
    for (std::uint32_t row = 0; row < rows; row++) {
        for (std::uint32_t column = 0; column < columns; column++) {
            // From -X -Z toward +Z first, which is counterclockwise from +Y.
            mesh.add_face(
                {
                    vertex(column, row),
                    vertex(column, row + 1),
                    vertex(column + 1, row + 1),
                    vertex(column + 1, row),
                },
                0);
        }
    }
    return mesh;
}

Mesh cylinder(float radius, float height, std::uint32_t sides, std::uint32_t segments) {
    if (!(sides >= 3 && segments >= 1)) {
        base::panic("a cylinder needs at least 3 sides and 1 segment");
    }
    return frustum(ring_radii(radius, radius, segments), height, sides);
}

Mesh cone(float bottom_radius, float top_radius, float height, std::uint32_t sides) {
    if (!(sides >= 3)) {
        base::panic("a cone needs at least 3 sides");
    }
    if (!(bottom_radius >= 0.0f && top_radius >= 0.0f && (bottom_radius > 0.0f || top_radius > 0.0f))) {
        base::panic("a cone needs radii of at least 0, and one more than 0");
    }
    return frustum(ring_radii(bottom_radius, top_radius, 1), height, sides);
}

Mesh uv_sphere(float radius, std::uint32_t segments, std::uint32_t rings) {
    if (!(segments >= 3 && rings >= 2)) {
        base::panic("a UV sphere needs at least 3 segments and 2 rings");
    }
    Mesh mesh;
    std::uint32_t top = mesh.add_vertex({0.0f, radius, 0.0f});
    for (std::uint32_t ring = 1; ring < rings; ring++) {
        float latitude = Pi * static_cast<float>(ring) / static_cast<float>(rings);
        for (std::uint32_t segment = 0; segment < segments; segment++) {
            float longitude = Tau * static_cast<float>(segment) / static_cast<float>(segments);
            mesh.add_vertex({
                radius * std::sin(latitude) * std::cos(longitude),
                radius * std::cos(latitude),
                radius * std::sin(latitude) * std::sin(longitude),
            });
        }
    }
    std::uint32_t bottom = mesh.add_vertex({0.0f, -radius, 0.0f});
    auto vertex = [&](std::uint32_t ring, std::uint32_t segment) {
        return 1 + (ring - 1) * segments + segment % segments;
    };
    for (std::uint32_t segment = 0; segment < segments; segment++) {
        // Around from +X toward +Z, the lower ring, then the upper: outward.
        mesh.add_face({vertex(1, segment), top, vertex(1, segment + 1)}, 0);
        for (std::uint32_t ring = 1; ring < rings - 1; ring++) {
            mesh.add_face({vertex(ring + 1, segment), vertex(ring, segment), vertex(ring, segment + 1), vertex(ring + 1, segment + 1)}, 0);
        }
        mesh.add_face({bottom, vertex(rings - 1, segment), vertex(rings - 1, segment + 1)}, 0);
    }
    return mesh;
}

Mesh torus(float major_radius, float minor_radius, std::uint32_t major_segments, std::uint32_t minor_segments) {
    if (!(major_segments >= 3 && minor_segments >= 3)) {
        base::panic("a torus needs at least 3 segments each way");
    }
    Mesh mesh;
    for (std::uint32_t major = 0; major < major_segments; major++) {
        float turn = Tau * static_cast<float>(major) / static_cast<float>(major_segments);
        float sin = std::sin(turn);
        float cos = std::cos(turn);
        for (std::uint32_t minor = 0; minor < minor_segments; minor++) {
            float tube_turn = Tau * static_cast<float>(minor) / static_cast<float>(minor_segments);
            float tube_sin = std::sin(tube_turn);
            float tube_cos = std::cos(tube_turn);
            float distance = major_radius + minor_radius * tube_cos;
            mesh.add_vertex({distance * cos, minor_radius * tube_sin, distance * sin});
        }
    }
    auto vertex = [&](std::uint32_t major, std::uint32_t minor) {
        return (major % major_segments) * minor_segments + minor % minor_segments;
    };
    for (std::uint32_t major = 0; major < major_segments; major++) {
        for (std::uint32_t minor = 0; minor < minor_segments; minor++) {
            // Around the tube first, then around the axis: outward.
            mesh.add_face({vertex(major, minor), vertex(major, minor + 1), vertex(major + 1, minor + 1), vertex(major + 1, minor)}, 0);
        }
    }
    return mesh;
}

} // namespace polygon_mesh::primitives
