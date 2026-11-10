#include "Common.h"

#include "Math/Math.h"

#include <algorithm>
#include <cmath>
#include <optional>

using math::cross;
using math::dot;
using math::subtract;
using polygon_mesh::Face;
using polygon_mesh::Mesh;

namespace common {

float signed_volume(const Mesh& mesh) {
    double volume = 0.0;
    for (const Face& face : mesh.faces) {
        math::Vector3 first = mesh.position(face.vertices[0]);
        for (size_t index = 1; index + 1 < face.vertices.size(); index++) {
            math::Vector3 b = mesh.position(face.vertices[index]);
            math::Vector3 c = mesh.position(face.vertices[index + 1]);
            volume += static_cast<double>(dot(first, cross(b, c))) / 6.0;
        }
    }
    return static_cast<float>(volume);
}

void assert_close(float actual, float expected, float tolerance, std::string_view what) {
    CHECK(std::abs(actual - expected) <= tolerance, "{}: {} isn't {}", what, base::display(actual), base::display(expected));
}

void assert_near(std::array<float, 3> actual, std::array<float, 3> expected, float tolerance, std::string_view what) {
    math::Vector3 difference = subtract(actual, expected);
    CHECK(std::sqrt(dot(difference, difference)) <= tolerance, "{}: {} isn't {}", what, testing::describe(actual), testing::describe(expected));
}

void assert_valid(const Mesh& mesh, std::string_view what) {
    if (std::expected<void, std::string> valid = mesh.validate(); !valid) {
        testing::fail(std::format("{}: {}", what, valid.error()));
    }
    CHECK(mesh.topology().is_consistently_wound(), "{} isn't consistently wound", what);
}

std::vector<size_t> sides_histogram(const Mesh& mesh) {
    size_t most = 3;
    if (!mesh.faces.empty()) {
        most = 0;
        for (const Face& face : mesh.faces) {
            most = std::max(most, face.vertices.size());
        }
    }
    std::vector<size_t> counts(most - 2, 0);
    for (const Face& face : mesh.faces) {
        counts[face.vertices.size() - 3] += 1;
    }
    return counts;
}

} // namespace common
