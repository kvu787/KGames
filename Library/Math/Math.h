// Vectors and matrices for 3D, in the conventions of DirectXMath:
//
// - Vectors are rows, and a matrix transforms one as v * M. So transforms
//   compose from left to right, like scaling * rotation * translation, and
//   the translation is in the last row.
// - Matrices are stored row by row, like XMFLOAT4X4.
// - Spaces are right-handed: in view space, +X is right, +Y is up, and the
//   camera looks along -Z.
// - Angles are in radians.

#pragma once

#include <array>

namespace math {

using Vector3 = std::array<float, 3>;
using Vector4 = std::array<float, 4>;

constexpr Vector3 add(Vector3 a, Vector3 b) {
    return {a[0] + b[0], a[1] + b[1], a[2] + b[2]};
}

constexpr Vector3 subtract(Vector3 a, Vector3 b) {
    return {a[0] - b[0], a[1] - b[1], a[2] - b[2]};
}

constexpr Vector3 scale(Vector3 v, float factor) {
    return {v[0] * factor, v[1] * factor, v[2] * factor};
}

constexpr float dot(Vector3 a, Vector3 b) {
    return a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
}

constexpr Vector3 cross(Vector3 a, Vector3 b) {
    return {
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    };
}

float length(Vector3 v);

// v at length 1, or zero if it has no length, as XMVector3Normalize gives,
// or none that a float can divide by.
Vector3 normalize(Vector3 v);

struct Matrix {
    std::array<std::array<float, 4>, 4> rows;

    static const Matrix Identity;

    static Matrix scaling(float x, float y, float z);
    static Matrix translation(float x, float y, float z);

    // Counterclockwise when looking from +X toward the origin, like
    // XMMatrixRotationX. So are the other two about their axes.
    static Matrix rotation_x(float angle);
    static Matrix rotation_y(float angle);
    static Matrix rotation_z(float angle);

    // The view matrix of a camera at `eye` that looks at `focus`, with `up`
    // pointing up on the screen, like XMMatrixLookAtRH. Its rows' first three
    // columns are the camera's right, up and backward directions, as columns.
    static Matrix look_at_rh(Vector3 eye, Vector3 focus, Vector3 up);

    // An orthographic projection of a `width` x `height` view, from
    // `near_plane` to `far_plane` in front of the camera, to depths 0 to 1,
    // like XMMatrixOrthographicRH.
    static Matrix orthographic_rh(float width, float height, float near_plane, float far_plane);

    Matrix transpose() const;

    // v * M.
    Vector4 transform(Vector4 v) const;

    // A point, with w = 1, so the translation applies.
    Vector3 transform_point(Vector3 point) const;

    // A direction, with w = 0, so the translation doesn't apply, like
    // XMVector3TransformNormal.
    Vector3 transform_direction(Vector3 direction) const;

    bool operator==(const Matrix&) const = default;
};

inline constexpr Matrix Matrix::Identity{{{
    {1.0f, 0.0f, 0.0f, 0.0f},
    {0.0f, 1.0f, 0.0f, 0.0f},
    {0.0f, 0.0f, 1.0f, 0.0f},
    {0.0f, 0.0f, 0.0f, 1.0f},
}}};

// a * b transforms by a, then by b.
Matrix operator*(const Matrix& a, const Matrix& b);

} // namespace math
