// Checks Library/Math against values that DirectXMath agrees on.

#include "Math/Math.h"
#include "Testing/Testing.h"

#include <algorithm>
#include <cmath>
#include <limits>
#include <numbers>
#include <span>
#include <vector>

using math::cross;
using math::dot;
using math::length;
using math::Matrix;
using math::normalize;
using math::Vector3;
using math::Vector4;

namespace {

bool near_to(float actual, float expected) {
    return std::abs(actual - expected) <= 1.0e-5f * std::max(std::abs(expected), 1.0f);
}

void check_near(std::span<const float> actual, std::span<const float> expected) {
    CHECK(std::ranges::equal(actual, expected, near_to), "{} isn't {}", testing::describe(actual), testing::describe(expected));
}

// MinimalSceneTest's camera, 5 units along Backward from the origin: 47.5
// degrees above the horizon, and 30 degrees from +Z toward +X.
constexpr Vector3 Right{0.8660254f, 0.0f, -0.5f};
constexpr Vector3 Up{-0.36863866f, 0.6755902f, -0.63850087f};
constexpr Vector3 Backward{0.3377951f, 0.7372773f, 0.5850783f};

constexpr float Quarter = std::numbers::pi_v<float> / 2.0f;

} // namespace

TEST(look_at_puts_the_camera_basis_in_the_columns) {
    Vector3 eye = math::scale(Backward, 5.0f);
    Matrix view = Matrix::look_at_rh(eye, {}, Up);
    size_t column = 0;
    for (Vector3 axis : {Right, Up, Backward}) {
        std::vector<float> actual;
        for (size_t row = 0; row < 3; row++) {
            actual.push_back(view.rows[row][column]);
        }
        check_near(actual, axis);
        column++;
    }
    // The eye goes to the origin, and the focus straight ahead of it.
    check_near(view.transform_point(eye), Vector3{});
    check_near(view.transform_point({}), Vector3{0.0f, 0.0f, -5.0f});
    // A view matrix is a rotation: the basis stays right-handed and unit length.
    std::vector<Vector3> rows;
    for (size_t row = 0; row < 3; row++) {
        rows.push_back({view.rows[row][0], view.rows[row][1], view.rows[row][2]});
    }
    CHECK(near_to(length(rows[0]), 1.0f) && near_to(dot(rows[0], rows[1]), 0.0f));
    check_near(cross(rows[0], rows[1]), rows[2]);
}

TEST(rotations_are_counterclockwise_about_their_axes) {
    check_near(Matrix::rotation_x(Quarter).transform_direction({0.0f, 1.0f, 0.0f}), Vector3{0.0f, 0.0f, 1.0f});
    check_near(Matrix::rotation_y(Quarter).transform_direction({0.0f, 0.0f, 1.0f}), Vector3{1.0f, 0.0f, 0.0f});
    check_near(Matrix::rotation_z(Quarter).transform_direction({1.0f, 0.0f, 0.0f}), Vector3{0.0f, 1.0f, 0.0f});
}

TEST(transforms_compose_from_left_to_right) {
    Matrix scale = Matrix::scaling(2.0f, 2.0f, 2.0f);
    Matrix rotate = Matrix::rotation_z(Quarter);
    Matrix translate = Matrix::translation(10.0f, 0.0f, 0.0f);
    Matrix combined = scale * rotate * translate;
    // Scaled to (2, 0, 0), rotated to (0, 2, 0), then moved.
    check_near(combined.transform_point({1.0f, 0.0f, 0.0f}), Vector3{10.0f, 2.0f, 0.0f});
    CHECK(Matrix::Identity * combined == combined);
    CHECK(combined.transpose().transpose() == combined);
}

TEST(orthographic_projection_maps_near_and_far_to_depths_0_and_1) {
    Matrix projection = Matrix::orthographic_rh(8.0f, 4.0f, 1.0f, 20.0f);
    check_near(projection.transform({4.0f, -2.0f, -1.0f, 1.0f}), Vector4{1.0f, -1.0f, 0.0f, 1.0f});
    check_near(projection.transform({-4.0f, 2.0f, -20.0f, 1.0f}), Vector4{-1.0f, 1.0f, 1.0f, 1.0f});
}

TEST(normalizing_gives_length_1_or_zero_for_nothing) {
    check_near(normalize({3.0f, 0.0f, 4.0f}), Vector3{0.6f, 0.0f, 0.8f});
    // XMVector3Normalize gives zero for a vector without length, not NaN.
    CHECK_EQ(normalize({0.0f, 0.0f, 0.0f}), (Vector3{0.0f, 0.0f, 0.0f}));
    CHECK_EQ(normalize({std::numeric_limits<float>::infinity(), 0.0f, 0.0f}), (Vector3{0.0f, 0.0f, 0.0f}));
}

// Rust's sum of floats starts from -0.0, so a sum of products that are all
// negative zero is negative zero, and the C++ gives the same bits.
TEST(a_sum_of_negative_zeros_is_negative_zero) {
    Matrix negative;
    for (auto& row : negative.rows) {
        row.fill(-1.0f);
    }
    for (float component : negative.transform({0.0f, 0.0f, 0.0f, 0.0f})) {
        CHECK(component == 0.0f && std::signbit(component));
    }
    Matrix zero{};
    Matrix product = zero * negative;
    for (const auto& row : product.rows) {
        for (float entry : row) {
            CHECK(entry == 0.0f && std::signbit(entry));
        }
    }
    // A positive zero among them makes the sum positive.
    CHECK(!std::signbit(negative.transform({0.0f, 0.0f, 0.0f, -0.0f})[0]));
}
