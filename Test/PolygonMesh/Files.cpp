// Library/PolygonMesh's files: JSON that reads back exactly, its text, what
// it says about files it can't read, saving and loading, and Wavefront OBJ.

#include "Base/Files.h"
#include "Base/Text.h"
#include "Common.h"
#include "PolygonMesh/PolygonMesh.h"
#include "Testing/Testing.h"

#include <algorithm>
#include <filesystem>
#include <format>
#include <ranges>
#include <string>
#include <string_view>
#include <tuple>
#include <vector>

using common::unwrap;
using common::unwrap_err;
using math::Vector3;
using polygon_mesh::Edge;
using polygon_mesh::Mesh;
namespace primitives = polygon_mesh::primitives;

namespace {

// A triangle of material 2 with one creased edge.
Mesh triangle() {
    Mesh mesh;
    for (Vector3 position : {Vector3{0.0f, 0.0f, 0.0f}, Vector3{1.0f, 0.0f, 0.0f}, Vector3{0.0f, 1.0f, 0.5f}}) {
        mesh.add_vertex(position);
    }
    mesh.add_face({0, 1, 2}, 2);
    mesh.set_crease(Edge(0, 1), 2.5f);
    return mesh;
}

// How many of the text's lines start with the prefix.
size_t lines_that_start_with(std::string_view text, std::string_view prefix) {
    return static_cast<size_t>(std::ranges::count_if(base::lines(text), [&](std::string_view line) { return line.starts_with(prefix); }));
}

} // namespace

TEST(json_reads_back_exactly) {
    Mesh cube = primitives::cube(2.0f);
    cube.assign_material({1, 4}, 3);
    cube.crease_edges({Edge(0, 1), Edge(2, 6)}, 1.25f);
    cube.set_crease(Edge(4, 5), 10.0f);
    CHECK_EQ(unwrap(Mesh::from_json(cube.to_json())), cube);
    // Positions with every digit that a float has.
    Mesh torus = primitives::torus(1.3f, 0.37f, 12, 7).catmull_clark(1).mesh;
    torus.rotate(std::views::iota(std::uint32_t{0}, static_cast<std::uint32_t>(torus.vertex_count())), {0.1f, 0.2f, 0.3f}, {1.0f, 2.0f, 3.0f}, 0.7f);
    Edge edge = *torus.edges().begin();
    torus.set_crease(edge, 0.3f);
    CHECK_EQ(unwrap(Mesh::from_json(torus.to_json())), torus);
    CHECK_EQ(unwrap(Mesh::from_json(Mesh().to_json())), Mesh());
}

TEST(json_has_a_line_for_each_position_face_and_crease) {
    std::string_view expected = R"({
    "Positions": [
        [0, 0, 0],
        [1, 0, 0],
        [0, 1, 0.5]
    ],
    "Faces": [
        [0, 1, 2]
    ],
    "Materials": [2],
    "Creases": [
        [0, 1, 2.5]
    ]
}
)";
    CHECK_EQ(triangle().to_json(), expected);
    CHECK_EQ(Mesh().to_json(), "{\n    \"Positions\": [],\n    \"Faces\": [],\n    \"Materials\": [],\n    \"Creases\": []\n}\n");
    // Materials that don't fit on a line wrap.
    Mesh grid = primitives::plane({10.0f, 10.0f}, {10, 10});
    grid.assign_material(std::views::iota(50, 100), 7);
    std::string text = grid.to_json();
    std::vector<std::string_view> materials;
    bool started = false;
    for (std::string_view line : base::lines(text)) {
        started = started || line.contains("Materials");
        if (!started) {
            continue;
        }
        if (line.contains("Creases")) {
            break;
        }
        materials.push_back(line);
    }
    CHECK(materials.size() > 3);
    CHECK(std::ranges::all_of(materials, [](std::string_view line) { return line.size() <= 100; }));
    std::string zeros = "        ";
    for (size_t index = 0; index < 31; index++) {
        zeros += index > 0 ? ", 0" : "0";
    }
    zeros += ",";
    CHECK_EQ(materials[1], zeros);
    CHECK_EQ(unwrap(Mesh::from_json(text)), grid);
}

TEST(json_can_leave_out_materials_and_creases) {
    std::string_view text = R"({"Positions": [[0, 0, 0], [1, 0, 0], [0, 1, 0]], "Faces": [[0, 1, 2]]})";
    Mesh mesh = unwrap(Mesh::from_json(text));
    CHECK_EQ(std::tuple(mesh.vertex_count(), mesh.face_count(), mesh.faces[0].material), (std::tuple<size_t, size_t, std::uint32_t>(3, 1, 0)));
    CHECK(mesh.creases.empty());
}

TEST(json_that_isn_t_a_mesh_says_what_s_wrong) {
    std::string_view positions = R"("Positions": [[0, 0, 0], [1, 0, 0], [0, 1, 0], [5, 5, 5]])";
    auto with = [&](std::string_view rest) {
        return std::format("{{{}, {}}}", positions, rest);
    };
    struct Case {
        std::string text;
        std::string_view expected;
    };
    std::vector<Case> cases{
        {"nonsense", "It isn't valid JSON, at line 1, column 1: expected a value."},
        {"[]", "A mesh must be an object with Positions, Faces, Materials and Creases, not a list of 0."},
        {std::format("{{{}}}", positions), "The mesh has no Faces."},
        {R"({"Faces": []})", "The mesh has no Positions."},
        {
            with(R"("Faces": [], "Normals": [])"),
            "Normals isn't part of a mesh, which has Positions, Faces, Materials and Creases.",
        },
        {R"({"Positions": {}, "Faces": []})", "Positions must be a list, not an object."},
        {
            R"({"Positions": [[0, 0]], "Faces": []})",
            "Positions[0] must be a list of 3 numbers that a float can hold, X, Y and Z, not a list of 2.",
        },
        {
            R"({"Positions": [[0, 0, 1e39]], "Faces": []})",
            "Positions[0] must be a list of 3 numbers that a float can hold, X, Y and Z, not a list of 3.",
        },
        {with(R"("Faces": [3])"), "Faces[0] must be a list of its vertices' numbers, not 3."},
        {with(R"("Faces": [[0, 1]])"), "Faces[0] has 2 vertices, but a face needs at least 3."},
        {with(R"("Faces": [[0, 1, 2], [0, 1, 4]])"), "Faces[1] uses vertex 4, but there are only 4 positions."},
        {with(R"("Faces": [[0, 1, 1.5]])"), "Faces[0] has 1.5, which isn't a vertex's number."},
        {with(R"("Faces": [[0, -1, 2]])"), "Faces[0] has -1, which isn't a vertex's number."},
        {with(R"("Faces": [[0, 1, 0]])"), "Faces[0] uses vertex 0 twice."},
        {
            with(R"("Faces": [[0, 1, 2]], "Materials": [0, 1])"),
            "Materials has 2 materials, but the mesh has 1 face, and each needs one.",
        },
        {
            with(R"("Faces": [[0, 1, 2]], "Materials": [-1])"),
            "Materials[0] must be a whole number from 0 to 4294967295, not -1.",
        },
        {
            with(R"("Faces": [[0, 1, 2]], "Creases": [[0, 1]])"),
            "Creases[0] must be a list of an edge's two vertices and its sharpness, like [0, 1, 2.5], not a list of 2.",
        },
        {
            with(R"("Faces": [[0, 1, 2]], "Creases": [[0, 9, 1]])"),
            "Creases[0] uses vertex 9, but there are only 4 positions.",
        },
        {
            with(R"("Faces": [[0, 1, 2]], "Creases": [[3, 0, 1]])"),
            "Creases[0] is from vertex 0 to 3, but no face has that edge.",
        },
        {
            with(R"("Faces": [[0, 1, 2]], "Creases": [[1, 1, 1]])"),
            "Creases[0] is from vertex 1 to 1, but no face has that edge.",
        },
        {
            with(R"("Faces": [[0, 1, 2]], "Creases": [[0, 1, 0]])"),
            "Creases[0]'s sharpness must be more than 0 and at most 10, not 0.",
        },
        {
            with(R"("Faces": [[0, 1, 2]], "Creases": [[0, 1, 10.5]])"),
            "Creases[0]'s sharpness must be more than 0 and at most 10, not 10.5.",
        },
        {
            with(R"("Faces": [[0, 1, 2]], "Creases": [[0, 1, 1], [1, 0, 2]])"),
            "Creases[1] creases the edge from 0 to 1 again.",
        },
    };
    for (const Case& wrong : cases) {
        CHECK_EQ(unwrap_err(Mesh::from_json(wrong.text)), wrong.expected, "for {}", wrong.text);
    }
}

TEST(saving_and_loading) {
    std::filesystem::path folder = std::filesystem::temp_directory_path() / "KGamesPolygonMeshTests";
    unwrap(base::create_dir_all(folder));
    std::filesystem::path path = folder / "Cube.json";
    Mesh cube = primitives::cube(1.0f);
    cube.set_crease(Edge(0, 1), 3.0f);
    unwrap(cube.save(path));
    CHECK_EQ(unwrap(Mesh::load(path)), cube);

    unwrap(base::write(path, std::string_view("{}")));
    CHECK_EQ(unwrap_err(Mesh::load(path)), std::format("{}: The mesh has no Positions.", base::display(path)));
    std::filesystem::path missing = folder / "Missing.json";
    CHECK(unwrap_err(Mesh::load(missing)).starts_with(std::format("Couldn't read {}: ", base::display(missing))));
}

TEST(obj_has_the_positions_and_the_faces_by_material) {
    Mesh mesh;
    for (Vector3 position : {Vector3{0.0f, 0.0f, 0.0f}, Vector3{1.0f, 0.0f, 0.0f}, Vector3{1.0f, 1.0f, 0.0f}, Vector3{0.0f, 1.0f, -0.25f}}) {
        mesh.add_vertex(position);
    }
    mesh.add_face({0, 1, 2}, 0);
    mesh.add_face({0, 2, 3}, 1);
    mesh.add_face({3, 2, 1, 0}, 1);
    mesh.add_face({2, 1, 0}, 0);
    std::string_view expected = "# 4 vertices and 4 faces, from KGames' PolygonMesh\n"
                                "v 0 0 0\n"
                                "v 1 0 0\n"
                                "v 1 1 0\n"
                                "v 0 1 -0.25\n"
                                "usemtl Material0\n"
                                "f 1 2 3\n"
                                "usemtl Material1\n"
                                "f 1 3 4\n"
                                "f 4 3 2 1\n"
                                "usemtl Material0\n"
                                "f 3 2 1\n";
    CHECK_EQ(mesh.to_obj(), expected);
    std::string cube = primitives::cube(2.0f).to_obj();
    CHECK_EQ(lines_that_start_with(cube, "usemtl"), 1);
    CHECK_EQ(lines_that_start_with(cube, "f "), 6);
}
