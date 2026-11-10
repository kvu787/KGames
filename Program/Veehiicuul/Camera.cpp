#include "Veehiicuul/Camera.h"

#include <algorithm>
#include <cmath>

namespace veehiicuul::camera {

namespace {

// How far the camera is from what it looks at, in units.
constexpr float Distance = 500.0f;

} // namespace

Eye Eye::game(std::array<double, 2> eye_center, double eye_height) {
    return Eye{
        .center = eye_center,
        .height = eye_height,
        .pitch = Pitch,
    };
}

Eye Eye::top_down(std::array<double, 2> eye_center, double eye_height) {
    return Eye{
        .center = eye_center,
        .height = eye_height,
        .pitch = std::numbers::pi / 2.0,
    };
}

std::array<std::array<double, 3>, 3> Eye::axes() const {
    double pitch_sin = std::sin(pitch);
    double pitch_cos = std::cos(pitch);
    double yaw_sin = std::sin(Yaw);
    double yaw_cos = std::cos(Yaw);
    std::array<double, 3> forward{-pitch_cos * yaw_sin, -pitch_sin, -pitch_cos * yaw_cos};
    std::array<double, 3> right{yaw_cos, 0.0, -yaw_sin};
    // Right, crossed with forward.
    std::array<double, 3> up{
        right[1] * forward[2] - right[2] * forward[1],
        right[2] * forward[0] - right[0] * forward[2],
        right[0] * forward[1] - right[1] * forward[0],
    };
    return {forward, right, up};
}

math::Matrix Eye::view() const {
    std::array<double, 3> forward = axes()[0];
    double yaw_sin = std::sin(Yaw);
    double yaw_cos = std::cos(Yaw);
    std::array<float, 3> target{static_cast<float>(center[0]), 0.0f, static_cast<float>(center[1])};
    std::array<float, 3> eye{
        target[0] - Distance * static_cast<float>(forward[0]),
        target[1] - Distance * static_cast<float>(forward[1]),
        target[2] - Distance * static_cast<float>(forward[2]),
    };
    // Up is away from the camera along the ground, which is the screen's
    // up at any pitch, even straight down.
    return math::Matrix::look_at_rh(eye, target, {static_cast<float>(-yaw_sin), 0.0f, static_cast<float>(-yaw_cos)});
}

std::array<float, 2> Eye::to_screen(std::array<double, 3> point, std::array<float, 2> size) const {
    std::array<std::array<double, 3>, 3> directions = axes();
    const std::array<double, 3>& right = directions[1];
    const std::array<double, 3>& up = directions[2];
    std::array<double, 3> offset{point[0] - center[0], point[1], point[2] - center[1]};
    auto dot = [&](const std::array<double, 3>& axis) {
        return offset[0] * axis[0] + offset[1] * axis[1] + offset[2] * axis[2];
    };
    double scale = static_cast<double>(size[1]) / height;
    return {
        static_cast<float>(static_cast<double>(size[0]) / 2.0 + dot(right) * scale),
        static_cast<float>(static_cast<double>(size[1]) / 2.0 - dot(up) * scale),
    };
}

std::array<float, 2> Eye::ground_to_screen(std::array<double, 2> point, std::array<float, 2> size) const {
    return to_screen({point[0], 0.0, point[1]}, size);
}

std::array<double, 2> Eye::to_ground(std::array<float, 2> point, std::array<float, 2> size) const {
    std::array<std::array<double, 3>, 3> directions = axes();
    const std::array<double, 3>& forward = directions[0];
    const std::array<double, 3>& right = directions[1];
    const std::array<double, 3>& up = directions[2];
    double scale = height / static_cast<double>(size[1]);
    double across = (static_cast<double>(point[0]) - static_cast<double>(size[0]) / 2.0) * scale;
    double along = -(static_cast<double>(point[1]) - static_cast<double>(size[1]) / 2.0) * scale;
    std::array<double, 3> start{
        center[0] + across * right[0] + along * up[0],
        across * right[1] + along * up[1],
        center[1] + across * right[2] + along * up[2],
    };
    // Along the camera's line of sight to the ground.
    double t = -start[1] / forward[1];
    return {start[0] + t * forward[0], start[2] + t * forward[2]};
}

void Eye::zoom_at(double factor, std::array<float, 2> anchor, std::array<float, 2> size) {
    std::array<double, 2> before = to_ground(anchor, size);
    height = std::clamp(height * factor, Lowest / 4.0, Highest * 2.0);
    std::array<double, 2> after = to_ground(anchor, size);
    center[0] += before[0] - after[0];
    center[1] += before[1] - after[1];
}

std::array<double, 2> to_ground(std::array<double, 2> direction) {
    double right = direction[0];
    double up = direction[1];
    double sin = std::sin(Yaw);
    double cos = std::cos(Yaw);
    return {right * cos - up * sin, -right * sin - up * cos};
}

} // namespace veehiicuul::camera
