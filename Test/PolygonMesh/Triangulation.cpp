// Library/PolygonMesh's triangles for SimplePaint: every primitive at levels 0
// to 3 of subdivision passes SimplePaint's checks with flat and smooth normals,
// smooth normals split where they should, a smooth normal that would fail
// falls back to its triangle's flat one, concave faces are cut inside
// themselves, and faces without area are left out.

#include "Common.h"
#include "Math/Math.h"
#include "PolygonMesh/PolygonMesh.h"
#include "SimplePaint/Geometry.h"
#include "Testing/Testing.h"

#include <algorithm>
#include <array>
#include <cmath>
#include <format>
#include <numbers>
#include <set>
#include <span>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

using common::assert_close;
using common::assert_near;
using math::cross;
using math::dot;
using math::length;
using math::Matrix;
using math::normalize;
using math::subtract;
using math::Vector3;
using polygon_mesh::Edge;
using polygon_mesh::Mesh;
using polygon_mesh::Shading;
using polygon_mesh::Topology;
using polygon_mesh::Triangles;
using simple_paint::validate_mesh;
using simple_paint::Vertex;
namespace primitives = polygon_mesh::primitives;

namespace {

constexpr float Pi = std::numbers::pi_v<float>;
constexpr float Tau = 2.0f * std::numbers::pi_v<float>;

const std::array<Shading, 3> Shadings{Shading::Flat{}, Shading::Smooth{Pi / 6.0f}, Shading::Smooth{Pi}};

// Triangulates the mesh, and checks that SimplePaint takes the triangles, that
// each face with area is drawn in n - 2 of them, of its material, wound like
// it.
Triangles check(const Mesh& mesh, Shading shading, std::string_view what) {
    Triangles triangles = mesh.triangulate(shading);
    if (auto valid = validate_mesh(triangles.vertices, triangles.indices, mesh.material_count()); !valid) {
        testing::fail(std::format("{}, {}: {}", what, describe_value(shading), valid.error().message));
    }
    CHECK_EQ(triangles.indices.size(), 3 * triangles.faces.size(), "{}", what);
    size_t expected = 0;
    for (const polygon_mesh::Face& face : mesh.faces) {
        expected += face.vertices.size() - 2;
    }
    CHECK_EQ(triangles.faces.size(), expected, "{}'s triangles", what);
    for (size_t index = 0; index < std::min(triangles.indices.size() / 3, triangles.faces.size()); index++) {
        size_t face = triangles.faces[index];
        Vertex a = triangles.vertices[triangles.indices[index * 3]];
        Vertex b = triangles.vertices[triangles.indices[index * 3 + 1]];
        Vertex c = triangles.vertices[triangles.indices[index * 3 + 2]];
        Vector3 normal = cross(subtract(b.position, a.position), subtract(c.position, a.position));
        CHECK(dot(normal, mesh.face_normal(face)) > 0.0f, "{}: a triangle of face {} faces the other way", what, face);
        CHECK(std::ranges::all_of(std::array{a, b, c}, [&](const Vertex& vertex) { return vertex.material_index == mesh.faces[face].material; }));
    }
    return triangles;
}

// The polygon's points as one face, turned out of the XZ plane.
Mesh polygon(std::span<const std::array<float, 2>> points) {
    Matrix turn = Matrix::rotation_x(0.5f) * Matrix::rotation_z(0.3f);
    Mesh mesh;
    for (auto [x, z] : points) {
        mesh.add_vertex(turn.transform_point({x, 0.0f, z}));
    }
    std::vector<std::uint32_t> vertices;
    for (std::uint32_t vertex = 0; vertex < points.size(); vertex++) {
        vertices.push_back(vertex);
    }
    mesh.add_face(std::move(vertices), 0);
    return mesh;
}

// The polygon's area, by the shoelace formula.
float shoelace(std::span<const std::array<float, 2>> points) {
    size_t count = points.size();
    float twice = 0.0f;
    for (size_t index = 0; index < count; index++) {
        auto [x0, y0] = points[index];
        auto [x1, y1] = points[(index + 1) % count];
        twice += x0 * y1 - x1 * y0;
    }
    return std::abs(twice) * 0.5f;
}

} // namespace

TEST(every_primitive_passes_simple_paint_s_checks_at_levels_0_to_3) {
    Mesh cylinder = primitives::cylinder(1.0f, 2.0f, 8, 2);
    cylinder.assign_material({16}, 1);
    Mesh creased = primitives::cube(2.0f);
    Topology topology = creased.topology();
    creased.crease_edges(topology.edges().first(6), 1.5f);
    std::vector<std::pair<Mesh, std::string_view>> meshes;
    meshes.emplace_back(primitives::cube(2.0f), "a cube");
    meshes.emplace_back(std::move(creased), "a creased cube");
    meshes.emplace_back(primitives::plane({2.0f, 3.0f}, {3, 2}), "a plane");
    meshes.emplace_back(std::move(cylinder), "a cylinder");
    meshes.emplace_back(primitives::cone(1.0f, 0.0f, 2.0f, 8), "a cone");
    meshes.emplace_back(primitives::cone(1.0f, 0.5f, 2.0f, 6), "a truncated cone");
    meshes.emplace_back(primitives::cone(1.0f, 0.0f, 0.1f, 3), "a flat pyramid");
    meshes.emplace_back(primitives::uv_sphere(1.0f, 8, 4), "a UV sphere");
    meshes.emplace_back(primitives::uv_sphere(1.0f, 3, 4), "a UV sphere of 3 segments");
    meshes.emplace_back(primitives::torus(1.0f, 0.25f, 12, 6), "a torus");
    for (const auto& [mesh, what] : meshes) {
        for (std::uint32_t level = 0; level <= 3; level++) {
            Mesh surface = mesh.catmull_clark(level).mesh;
            for (Shading shading : Shadings) {
                check(surface, shading, std::format("{} at level {}", what, level));
            }
        }
    }
}

TEST(flat_normals_are_the_faces) {
    Mesh cube = primitives::cube(2.0f);
    Triangles triangles = check(cube, Shading::Flat{}, "a flat cube");
    CHECK_EQ(std::pair(triangles.vertices.size(), triangles.indices.size()), (std::pair<size_t, size_t>(24, 36)));
    CHECK_EQ(triangles.faces, (std::vector<size_t>{0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5}));
    for (size_t index = 0; index < std::min(triangles.indices.size() / 3, triangles.faces.size()); index++) {
        size_t face = triangles.faces[index];
        for (size_t corner = 0; corner < 3; corner++) {
            CHECK_EQ(triangles.vertices[triangles.indices[index * 3 + corner]].normal, cube.face_normal(face));
        }
    }
}

TEST(smooth_normals_split_at_sharp_edges_creases_and_materials) {
    // Every edge of a cube is 90 degrees, more than 30.
    Mesh cube = primitives::cube(2.0f);
    Triangles split = check(cube, Shading::Smooth{Pi / 6.0f}, "a cube smooth to 30 degrees");
    CHECK_EQ(split, cube.triangulate(Shading::Flat{}));
    // But not more than 180, so each corner has one normal, out of it.
    Triangles round = check(cube, Shading::Smooth{Pi}, "a smooth cube");
    CHECK_EQ(round.vertices.size(), 8);
    for (const Vertex& vertex : round.vertices) {
        assert_near(vertex.normal, normalize(vertex.position), 1.0e-6f, "a smooth cube's normal");
    }
    // A crease of any sharpness splits them.
    Mesh creased = primitives::cube(2.0f);
    std::set<Edge> edges = creased.edges();
    creased.crease_edges(edges, 0.25f);
    CHECK_EQ(creased.triangulate(Shading::Smooth{Pi}), cube.triangulate(Shading::Flat{}));
    // So does a material, which only changes which vertices there are.
    Mesh painted = primitives::cube(2.0f);
    painted.assign_material({3}, 1);
    Triangles two = check(painted, Shading::Smooth{Pi}, "a cube of two materials");
    CHECK_EQ(two.vertices.size(), 12);
    // A cylinder's caps are 90 degrees from its sides, which are 45 from
    // each other.
    Mesh cylinder = primitives::cylinder(1.0f, 2.0f, 8, 1);
    Triangles triangles = check(cylinder, Shading::Smooth{Pi / 3.0f}, "a cylinder");
    CHECK_EQ(triangles.vertices.size(), 32);
    // A sphere's normals point out of it.
    Mesh sphere = primitives::uv_sphere(1.0f, 16, 8);
    triangles = check(sphere, Shading::Smooth{Pi / 3.0f}, "a sphere");
    CHECK_EQ(triangles.vertices.size(), sphere.vertex_count());
    for (const Vertex& vertex : triangles.vertices) {
        CHECK(dot(vertex.normal, vertex.position) > 0.98f, "a sphere's normal at {}", testing::describe(vertex.position));
        assert_close(length(vertex.normal), 1.0f, 1.0e-6f, "a normal's length");
    }
}

TEST(smooth_normals_too_far_apart_fall_back_to_flat) {
    // Around a sphere of 3 segments, the normals at a quad's corners near the
    // equator are 120 degrees apart, too far for SimplePaint.
    Mesh sphere = primitives::uv_sphere(1.0f, 3, 4);
    Triangles triangles = check(sphere, Shading::Smooth{Pi}, "a sphere of 3 segments");
    CHECK(triangles.vertices.size() > sphere.vertex_count(), "no triangle fell back to flat");
    int flat = 0;
    for (size_t index = 0; index < triangles.indices.size() / 3; index++) {
        Vertex a = triangles.vertices[triangles.indices[index * 3]];
        Vertex b = triangles.vertices[triangles.indices[index * 3 + 1]];
        Vertex c = triangles.vertices[triangles.indices[index * 3 + 2]];
        if (a.normal == b.normal && b.normal == c.normal) {
            Vector3 normal = normalize(cross(subtract(b.position, a.position), subtract(c.position, a.position)));
            assert_near(a.normal, normal, 1.0e-6f, "a fallen-back normal");
            flat += 1;
        }
    }
    CHECK(flat > 0);
}

TEST(concave_faces_are_cut_inside_themselves) {
    std::vector<std::array<float, 2>> l_shape{{0.0f, 0.0f}, {2.0f, 0.0f}, {2.0f, 1.0f}, {1.0f, 1.0f}, {1.0f, 2.0f}, {0.0f, 2.0f}};
    std::vector<std::array<float, 2>> star;
    for (int point = 0; point < 10; point++) {
        float angle = Tau * static_cast<float>(point) / 10.0f;
        float radius = point % 2 == 0 ? 2.0f : 0.6f;
        star.push_back({radius * std::cos(angle), radius * std::sin(angle)});
    }
    std::vector<std::array<float, 2>> comb{
        {0.0f, 0.0f}, {5.0f, 0.0f}, {5.0f, 3.0f}, {4.0f, 3.0f}, {4.0f, 1.0f}, {3.0f, 1.0f}, {3.0f, 3.0f}, {2.0f, 3.0f}, {2.0f, 1.0f}, {1.0f, 1.0f}, {1.0f, 3.0f}, {0.0f, 3.0f},
    };
    // A square with a vertex in the middle of each side, like the faces next
    // to subdivided ones.
    std::vector<std::array<float, 2>> sides{{0.0f, 0.0f}, {1.0f, 0.0f}, {2.0f, 0.0f}, {2.0f, 1.0f}, {2.0f, 2.0f}, {1.0f, 2.0f}, {0.0f, 2.0f}, {0.0f, 1.0f}};
    std::vector<std::pair<std::span<const std::array<float, 2>>, std::string_view>> polygons{
        {l_shape, "an L"},
        {star, "a star"},
        {comb, "a comb"},
        {sides, "a square with midpoints"},
    };
    for (const auto& [points, what] : polygons) {
        Mesh mesh = polygon(points);
        std::vector<std::array<size_t, 3>> triangles = mesh.face_triangles(0);
        CHECK_EQ(triangles.size(), points.size() - 2, "{}", what);
        const std::vector<std::uint32_t>& face = mesh.faces[0].vertices;
        Vector3 normal = mesh.face_normal(0);
        float area = 0.0f;
        for (const std::array<size_t, 3>& triangle : triangles) {
            Vector3 a = mesh.position(face[triangle[0]]);
            Vector3 b = mesh.position(face[triangle[1]]);
            Vector3 c = mesh.position(face[triangle[2]]);
            Vector3 twice = cross(subtract(b, a), subtract(c, a));
            CHECK(dot(twice, normal) > 1.0e-6f, "{} has a triangle that's backward or without area", what);
            area += length(twice) * 0.5f;
        }
        assert_close(area, shoelace(points), 1.0e-4f, std::format("{}'s area", what));
        assert_close(mesh.face_area(0), area, 1.0e-4f, std::format("{}'s face area", what));
        for (Shading shading : Shadings) {
            check(mesh, shading, what);
        }
    }
}

TEST(faces_without_area_are_left_out) {
    // An extrusion that hasn't moved has walls without area.
    Mesh cube = primitives::cube(2.0f);
    cube.extrude_faces({3});
    for (Shading shading : Shadings) {
        Triangles triangles = cube.triangulate(shading);
        auto valid = validate_mesh(triangles.vertices, triangles.indices, 1);
        CHECK(valid.has_value(), "{}", valid.error().message);
        CHECK_EQ(triangles.faces.size(), 12);
        CHECK(std::ranges::all_of(triangles.faces, [](size_t face) { return face < 6; }));
    }
}
