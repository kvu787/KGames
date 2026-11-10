#include "SimplePaint/Orthographic.h"

#include <cmath>
#include <cstddef>

namespace simple_paint::orthographic {

std::expected<Projection, Error> make_projection(float width, float height, float near_, float far_) {
    if (!std::isfinite(width) || !std::isfinite(height) || !std::isfinite(near_) || !std::isfinite(far_) || width < 1.0e-4f || height < 1.0e-4f || width > 1.0e6f ||
        height > 1.0e6f || near_ < 0.0f || far_ > 1.0e6f || far_ - near_ < 1.0e-4f) {
        return std::unexpected(Error{"Orthographic projection requires bounded positive dimensions and 0 <= near < far."});
    }
    float depth_scale = 1.0f / (near_ - far_);
    return Projection{
        .scale = {2.0f / width, 2.0f / height, depth_scale},
        .depth_offset = depth_scale * near_,
    };
}

std::expected<ObjectTransforms, Error> build_object_transforms(const math::Matrix& world_view, const Projection& projection) {
    auto error = [](const char* message) {
        return std::unexpected(Error{message});
    };
    float scale_x = projection.scale[0];
    float scale_y = projection.scale[1];
    float scale_z = projection.scale[2];
    float offset = projection.depth_offset;
    if (!std::isfinite(scale_x) || !std::isfinite(scale_y) || !std::isfinite(scale_z) || !std::isfinite(offset) || scale_x <= 0.0f || scale_y <= 0.0f || scale_z >= 0.0f ||
        scale_x > 2.0e4f || scale_y > 2.0e4f || scale_z < -1.0e4f || std::abs(offset) > 1.0e10f) {
        return error("Invalid packed orthographic projection.");
    }
    const std::array<std::array<float, 4>, 4>& m = world_view.rows;
    for (const std::array<float, 4>& row : m) {
        for (float value : row) {
            if (!std::isfinite(value) || std::abs(value) > 1.0e6f) {
                return error("SimplePaint transforms must be finite and bounded.");
            }
        }
    }
    if (m[0][3] != 0.0f || m[1][3] != 0.0f || m[2][3] != 0.0f || m[3][3] != 1.0f) {
        return error("SimplePaint supports affine object/view transforms and orthographic projection only.");
    }
    // The rows' squared lengths: how each basis vector is scaled.
    std::array<double, 3> lengths{};
    for (size_t i = 0; i < 3; i++) {
        for (size_t k = 0; k < 3; k++) {
            lengths[i] += static_cast<double>(m[i][k]) * static_cast<double>(m[i][k]);
        }
    }
    for (size_t i = 0; i < 3; i++) {
        if (lengths[i] < 1.0 / (1024.0 * 1024.0) || lengths[i] > 1024.0 * 1024.0 || std::abs(lengths[i] - lengths[0]) > 1.0e-5 * lengths[0]) {
            return error("SimplePaint requires uniform scale in [1/1024,1024].");
        }
        for (size_t j = 0; j < i; j++) {
            double dot = 0.0;
            for (size_t k = 0; k < 3; k++) {
                dot += static_cast<double>(m[i][k]) * static_cast<double>(m[j][k]);
            }
            if (std::abs(dot) > 1.0e-5 * lengths[0]) {
                return error("SimplePaint does not support sheared object/view transforms.");
            }
        }
    }
    std::array<std::array<float, 4>, 4> columns = world_view.transpose().rows;
    auto scaled = [](std::array<float, 4> column, float factor) {
        return std::array<float, 4>{column[0] * factor, column[1] * factor, column[2] * factor, column[3] * factor};
    };
    std::array<float, 4> depth = scaled(columns[2], scale_z);
    depth[3] += offset;
    return ObjectTransforms{
        .world_to_clip = {scaled(columns[0], scale_x), scaled(columns[1], scale_y), depth},
        // Normals use XYZ only; the validated object and view scale is uniform.
        .normal_to_view = {columns[0], columns[1], columns[2]},
    };
}

} // namespace simple_paint::orthographic
