// The view's camera: orthographic, as SimplePaint paints only orthographic
// views, orbiting a target point. It turns around the target, pans across
// the view, and zooms by how much of the world the view's height shows. It
// also maps between the world and the view's pixels, for drawing overlays and
// picking.

#pragma once

#include "Math/Math.h"
#include "MeshModeler/Angles.h"
#include "SimplePaint/Orthographic.h"
#include "UserInterface/Drawing/Geometry.h"

#include <utility>

namespace mesh_modeler::camera {

inline constexpr float MinimumHeight = 0.05f;
inline constexpr float MaximumHeight = 200.0f;

// The views that View's menu has.
enum class Viewpoint {
    Front,
    Back,
    Right,
    Left,
    Top,
    Bottom,
};

struct Camera {
    math::Vector3 target{0.0f, 0.0f, 0.0f};
    // Around the up axis from looking down -Z, and up from the horizon, in
    // radians.
    float yaw = to_radians(35.0f);
    float pitch = to_radians(25.0f);
    // How much of the world the view's height shows.
    float height = 6.0f;

    // From the target toward the eye.
    math::Vector3 backward() const;

    math::Vector3 right() const;

    math::Vector3 up() const;

    math::Vector3 eye() const;

    math::Matrix view() const;

    // SimplePaint's packed projection for the view's size.
    simple_paint::orthographic::Projection projection(user_interface::Rect view) const;

    // From the world to the view's clip space, v * M.
    math::Matrix world_to_clip(user_interface::Rect view) const;

    // How much of the world a pixel is.
    float pixel_size(user_interface::Rect view) const;

    // Where a point of the world is in the window, in pixels, and its depth
    // toward the camera: more is nearer.
    std::pair<user_interface::Point, float> to_screen(math::Vector3 point, user_interface::Rect view) const;

    // The ray through a pixel of the window: where it starts, far toward the
    // camera, and its direction into the view.
    std::pair<math::Vector3, math::Vector3> ray(user_interface::Point pixel, user_interface::Rect view) const;

    // Turns around the target by pixels of a drag.
    void orbit(user_interface::Point delta);

    // Moves across the view so that the world follows the pointer.
    void pan(user_interface::Point delta, user_interface::Rect view);

    // Zooms in for notches up, keeping the point of the world under the
    // pointer where it is.
    void zoom(float notches, user_interface::Point at, user_interface::Rect view);

    // Shows the box from `minimum` to `maximum` whole, from where the camera
    // looks now.
    void frame(math::Vector3 minimum, math::Vector3 maximum);

    void look_from(Viewpoint viewpoint);

    bool operator==(const Camera&) const = default;

private:
    float width(user_interface::Rect view) const;
};

} // namespace mesh_modeler::camera
