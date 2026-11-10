// The packed orthographic transforms against full matrix products, and the
// projection's near and far depth endpoints.

#include "Base/Text.h"
#include "Math/Math.h"
#include "SimplePaint/Orthographic.h"
#include "Testing/Testing.h"

#include "Reference.h"

#include <algorithm>
#include <array>
#include <cmath>
#include <print>
#include <utility>

using math::dot;
using math::Matrix;
using reference::Random;
using simple_paint::orthographic::build_object_transforms;
using simple_paint::orthographic::make_projection;

namespace {

void require_near(float actual, float expected) {
    CHECK(std::isfinite(actual) && std::abs(actual - expected) <= 2.0e-5f * std::fmax(std::abs(expected), 1.0f),
          "Packed orthographic transform differs from the matrix product: {} isn't {}", base::display(actual), base::display(expected));
}

} // namespace

TEST(orthographic_transforms) {
    Random random(0x4F525448);
    auto position = [&] {
        return static_cast<float>(random.range(-10.0, 10.0));
    };
    size_t count = 0;
    for (float aspect : {0.5625f, 1.0f, 16.0f / 9.0f, 32.0f / 9.0f}) {
        simple_paint::orthographic::Projection projection = make_projection(5.0f * aspect, 5.0f, 1.0f, 20.0f).value();
        Matrix reference_projection = Matrix::orthographic_rh(5.0f * aspect, 5.0f, 1.0f, 20.0f);
        // Near and far must map correctly, for depth clipping and overlap.
        for (auto [z, depth] : {std::pair{-1.0f, 0.0f}, std::pair{-20.0f, 1.0f}}) {
            require_near(z * projection.scale[2] + projection.depth_offset, depth);
        }
        for (int sample = 0; sample < 1000; sample++) {
            // The random numbers, one after another, in the order that the
            // values they make are written in.
            std::array<float, 3> angles{};
            for (float& angle : angles) {
                angle = position() * 0.314f;
            }
            float x = position();
            float y = position();
            float z = position();
            Matrix world =
                Matrix::scaling(0.4f, 0.4f, 0.4f) * Matrix::rotation_z(angles[0]) * Matrix::rotation_x(angles[1]) * Matrix::rotation_y(angles[2]) * Matrix::translation(x, y, z);
            Matrix view = Matrix::look_at_rh({1.6889755f, 3.6863865f, 2.9253915f}, {0.0f, 0.0f, 0.0f}, {-0.36863866f, 0.6755902f, -0.63850087f});
            Matrix world_view = world * view;
            simple_paint::orthographic::ObjectTransforms compact = build_object_transforms(world_view, projection).value();
            math::Vector4 vertex{};
            for (size_t axis = 0; axis < 3; axis++) {
                vertex[axis] = position();
            }
            vertex[3] = 1.0f;
            math::Vector3 normal{};
            for (float& component : normal) {
                component = position();
            }
            math::Vector4 expected_position = (world_view * reference_projection).transform(vertex);
            math::Vector3 expected_normal = world_view.transform_direction(normal);
            for (size_t axis = 0; axis < 3; axis++) {
                // From -0.0, where Rust's sum of floats starts.
                float clip = -0.0f;
                for (size_t k = 0; k < 4; k++) {
                    clip += vertex[k] * compact.world_to_clip[axis][k];
                }
                std::array<float, 4> row = compact.normal_to_view[axis];
                float view_normal = dot(normal, {row[0], row[1], row[2]});
                require_near(clip, expected_position[axis]);
                require_near(view_normal, expected_normal[axis]);
            }
            require_near(expected_position[3], 1.0f);
            count += 1;
        }
    }
    std::println("{} packed transforms, depth endpoints passed.", count);
}
