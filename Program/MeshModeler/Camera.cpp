#include "MeshModeler/Camera.h"

#include "Base/Panic.h"

#include <algorithm>
#include <cmath>
#include <expected>
#include <numbers>

namespace mesh_modeler::camera {

using user_interface::Point;
using user_interface::Rect;

namespace {

// How far the eye is from the target. Orthographic views don't change with
// it, but it puts the near and far planes around the scene.
constexpr float Distance = 100.0f;
constexpr float LimitPitch = 89.9f * std::numbers::pi_v<float> / 180.0f;

} // namespace

math::Vector3 Camera::backward() const {
    float sin_yaw = std::sin(yaw);
    float cos_yaw = std::cos(yaw);
    float sin_pitch = std::sin(pitch);
    float cos_pitch = std::cos(pitch);
    return {cos_pitch * sin_yaw, sin_pitch, cos_pitch * cos_yaw};
}

math::Vector3 Camera::right() const {
    return math::normalize(math::cross({0.0f, 1.0f, 0.0f}, backward()));
}

math::Vector3 Camera::up() const {
    return math::cross(backward(), right());
}

math::Vector3 Camera::eye() const {
    return math::add(target, math::scale(backward(), Distance));
}

math::Matrix Camera::view() const {
    return math::Matrix::look_at_rh(eye(), target, {0.0f, 1.0f, 0.0f});
}

float Camera::width(Rect view) const {
    return height * view.width / std::max(view.height, 1.0f);
}

simple_paint::orthographic::Projection Camera::projection(Rect view) const {
    std::expected<simple_paint::orthographic::Projection, simple_paint::Error> projection = simple_paint::orthographic::make_projection(width(view), height, 0.0f, Distance * 2.0f);
    if (!projection) {
        base::panic("the camera's projection is in range");
    }
    return *projection;
}

math::Matrix Camera::world_to_clip(Rect view) const {
    return this->view() * math::Matrix::orthographic_rh(width(view), height, 0.0f, Distance * 2.0f);
}

float Camera::pixel_size(Rect view) const {
    return height / std::max(view.height, 1.0f);
}

std::pair<Point, float> Camera::to_screen(math::Vector3 point, Rect view) const {
    math::Vector3 relative = math::subtract(point, target);
    float pixel = pixel_size(view);
    Point center = view.center();
    float x = center.x + math::dot(relative, right()) / pixel;
    float y = center.y - math::dot(relative, up()) / pixel;
    return {Point(x, y), math::dot(relative, backward())};
}

std::pair<math::Vector3, math::Vector3> Camera::ray(Point pixel, Rect view) const {
    float size = pixel_size(view);
    Point center = view.center();
    math::Vector3 across = math::scale(right(), (pixel.x - center.x) * size);
    math::Vector3 upward = math::scale(up(), (center.y - pixel.y) * size);
    math::Vector3 origin = math::add(math::add(eye(), across), upward);
    return {origin, math::scale(backward(), -1.0f)};
}

void Camera::orbit(Point delta) {
    yaw -= delta.x * 0.008f;
    pitch = std::clamp(pitch + delta.y * 0.008f, -LimitPitch, LimitPitch);
}

void Camera::pan(Point delta, Rect view) {
    float size = pixel_size(view);
    math::Vector3 across = math::scale(right(), -delta.x * size);
    math::Vector3 upward = math::scale(up(), delta.y * size);
    target = math::add(math::add(target, across), upward);
}

void Camera::zoom(float notches, Point at, Rect view) {
    math::Vector3 before = ray(at, view).first;
    height = std::clamp(height * std::pow(0.85f, notches), MinimumHeight, MaximumHeight);
    math::Vector3 after = ray(at, view).first;
    target = math::add(target, math::subtract(before, after));
}

void Camera::frame(math::Vector3 minimum, math::Vector3 maximum) {
    target = math::scale(math::add(minimum, maximum), 0.5f);
    float extent = math::length(math::subtract(maximum, minimum));
    height = std::clamp(extent * 1.25f, MinimumHeight * 4.0f, MaximumHeight);
}

void Camera::look_from(Viewpoint viewpoint) {
    float yaw_degrees = 0.0f;
    float pitch_degrees = 0.0f;
    switch (viewpoint) {
    case Viewpoint::Front:
        yaw_degrees = 0.0f;
        pitch_degrees = 0.0f;
        break;
    case Viewpoint::Back:
        yaw_degrees = 180.0f;
        pitch_degrees = 0.0f;
        break;
    case Viewpoint::Right:
        yaw_degrees = 90.0f;
        pitch_degrees = 0.0f;
        break;
    case Viewpoint::Left:
        yaw_degrees = -90.0f;
        pitch_degrees = 0.0f;
        break;
    case Viewpoint::Top:
        yaw_degrees = 0.0f;
        pitch_degrees = 89.9f;
        break;
    case Viewpoint::Bottom:
        yaw_degrees = 0.0f;
        pitch_degrees = -89.9f;
        break;
    }
    yaw = to_radians(yaw_degrees);
    pitch = to_radians(pitch_degrees);
}

} // namespace mesh_modeler::camera
