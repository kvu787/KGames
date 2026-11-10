// Validated transforms for SimplePaint.hlsl's vertex stage. Projection is
// always orthographic, and object and view transforms are affine. Only clip
// XYZ is stored; the vertex shader supplies the invariant clip W = 1.
//
// Matrices follow Library/Math's conventions, which are DirectXMath's.

#pragma once

#include "Math/Math.h"
#include "SimplePaint/Error.h"

#include <array>
#include <expected>

namespace simple_paint::orthographic {

// What SimplePaint.hlsl binds at b0: three columns of the world-to-clip
// transform, and three of the normal-to-view one.
struct ObjectTransforms {
    std::array<std::array<float, 4>, 3> world_to_clip{};
    std::array<std::array<float, 4>, 3> normal_to_view{};

    bool operator==(const ObjectTransforms&) const = default;
};

static_assert(sizeof(ObjectTransforms) == 96);

struct Projection {
    std::array<float, 3> scale{};
    float depth_offset = 0.0f;

    bool operator==(const Projection&) const = default;
};

// A right-handed orthographic projection of a `width` x `height` view, from
// `near_` to `far_` in front of the camera, to depths 0 to 1.
std::expected<Projection, Error> make_projection(float width, float height, float near_, float far_);

// Packs an object's world-view transform with the projection. Only rotations,
// reflections and uniform scale in [1/1024, 1024] are supported; shear and
// nonuniform scale are rejected instead of shading wrong normals.
std::expected<ObjectTransforms, Error> build_object_transforms(const math::Matrix& world_view, const Projection& projection);

} // namespace simple_paint::orthographic
