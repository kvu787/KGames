#include "Math/Math.h"

#include <cmath>

namespace math {

float length(Vector3 v) {
    return std::sqrt(dot(v, v));
}

Vector3 normalize(Vector3 v) {
    float size = length(v);
    return size > 1.0e-30f && std::isfinite(size) ? scale(v, 1.0f / size) : Vector3{};
}

Matrix Matrix::scaling(float x, float y, float z) {
    return {{{
        {x, 0.0f, 0.0f, 0.0f},
        {0.0f, y, 0.0f, 0.0f},
        {0.0f, 0.0f, z, 0.0f},
        {0.0f, 0.0f, 0.0f, 1.0f},
    }}};
}

Matrix Matrix::translation(float x, float y, float z) {
    return {{{
        {1.0f, 0.0f, 0.0f, 0.0f},
        {0.0f, 1.0f, 0.0f, 0.0f},
        {0.0f, 0.0f, 1.0f, 0.0f},
        {x, y, z, 1.0f},
    }}};
}

Matrix Matrix::rotation_x(float angle) {
    float sin = std::sin(angle);
    float cos = std::cos(angle);
    return {{{
        {1.0f, 0.0f, 0.0f, 0.0f},
        {0.0f, cos, sin, 0.0f},
        {0.0f, -sin, cos, 0.0f},
        {0.0f, 0.0f, 0.0f, 1.0f},
    }}};
}

Matrix Matrix::rotation_y(float angle) {
    float sin = std::sin(angle);
    float cos = std::cos(angle);
    return {{{
        {cos, 0.0f, -sin, 0.0f},
        {0.0f, 1.0f, 0.0f, 0.0f},
        {sin, 0.0f, cos, 0.0f},
        {0.0f, 0.0f, 0.0f, 1.0f},
    }}};
}

Matrix Matrix::rotation_z(float angle) {
    float sin = std::sin(angle);
    float cos = std::cos(angle);
    return {{{
        {cos, sin, 0.0f, 0.0f},
        {-sin, cos, 0.0f, 0.0f},
        {0.0f, 0.0f, 1.0f, 0.0f},
        {0.0f, 0.0f, 0.0f, 1.0f},
    }}};
}

Matrix Matrix::look_at_rh(Vector3 eye, Vector3 focus, Vector3 up) {
    Vector3 backward = normalize(subtract(eye, focus));
    Vector3 right = normalize(cross(up, backward));
    up = cross(backward, right);
    return {{{
        {right[0], up[0], backward[0], 0.0f},
        {right[1], up[1], backward[1], 0.0f},
        {right[2], up[2], backward[2], 0.0f},
        {-dot(right, eye), -dot(up, eye), -dot(backward, eye), 1.0f},
    }}};
}

Matrix Matrix::orthographic_rh(float width, float height, float near_plane, float far_plane) {
    float range = 1.0f / (near_plane - far_plane);
    return {{{
        {2.0f / width, 0.0f, 0.0f, 0.0f},
        {0.0f, 2.0f / height, 0.0f, 0.0f},
        {0.0f, 0.0f, range, 0.0f},
        {0.0f, 0.0f, range * near_plane, 1.0f},
    }}};
}

Matrix Matrix::transpose() const {
    Matrix transposed;
    for (size_t row = 0; row < 4; row++) {
        for (size_t column = 0; column < 4; column++) {
            transposed.rows[row][column] = rows[column][row];
        }
    }
    return transposed;
}

Vector4 Matrix::transform(Vector4 v) const {
    Vector4 transformed;
    for (size_t column = 0; column < 4; column++) {
        // From 0, in the rows' order, as Rust's sum adds them, so that the
        // floats round the same way.
        float sum = 0.0f;
        for (size_t row = 0; row < 4; row++) {
            sum += v[row] * rows[row][column];
        }
        transformed[column] = sum;
    }
    return transformed;
}

Vector3 Matrix::transform_point(Vector3 point) const {
    Vector4 transformed = transform({point[0], point[1], point[2], 1.0f});
    return {transformed[0], transformed[1], transformed[2]};
}

Vector3 Matrix::transform_direction(Vector3 direction) const {
    Vector4 transformed = transform({direction[0], direction[1], direction[2], 0.0f});
    return {transformed[0], transformed[1], transformed[2]};
}

Matrix operator*(const Matrix& a, const Matrix& b) {
    Matrix product;
    for (size_t row = 0; row < 4; row++) {
        for (size_t column = 0; column < 4; column++) {
            float sum = 0.0f;
            for (size_t k = 0; k < 4; k++) {
                sum += a.rows[row][k] * b.rows[k][column];
            }
            product.rows[row][column] = sum;
        }
    }
    return product;
}

} // namespace math
