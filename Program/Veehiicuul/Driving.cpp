#include "Veehiicuul/Driving.h"

#include "Base/Text.h"

#include <cmath>
#include <format>

namespace veehiicuul::driving {

namespace {

// Below this speed, in units per second, the brake stops the car at once,
// instead of slowing it by less and less.
constexpr double Stopped = 0.01;

} // namespace

double Car::speed() const {
    return std::hypot(velocity[0], velocity[1]);
}

double DeadZone::apply(double value) const {
    double magnitude = std::abs(value);
    if (magnitude < inner) {
        return 0.0;
    }
    double scaled = magnitude > outer ? 1.0 : (magnitude - inner) / (outer - inner);
    return std::copysign(scaled, value);
}

std::string describe_value(const Acceleration& acceleration) {
    return std::format("Acceleration {{ forward: {}, reverse: {}, left: {}, right: {} }}", base::debug(acceleration.forward), base::debug(acceleration.reverse),
                       base::debug(acceleration.left), base::debug(acceleration.right));
}

std::string describe_value(const Car& car) {
    return std::format("Car {{ position: [{}, {}], heading: {}, velocity: [{}, {}] }}", base::debug(car.position[0]), base::debug(car.position[1]), base::debug(car.heading),
                       base::debug(car.velocity[0]), base::debug(car.velocity[1]));
}

std::string describe_value(const DeadZone& dead_zone) {
    return std::format("DeadZone {{ inner: {}, outer: {} }}", base::debug(dead_zone.inner), base::debug(dead_zone.outer));
}

std::string describe_value(const DeadZones& dead_zones) {
    return std::format("DeadZones {{ sideways: {}, forward_and_back: {} }}", describe_value(dead_zones.sideways), describe_value(dead_zones.forward_and_back));
}

void drive(Car& car, std::array<double, 2> push, double brake, const Acceleration& acceleration, const DeadZones& dead_zones, double seconds) {
    double sin = std::sin(car.heading);
    double cos = std::cos(car.heading);
    if (brake == 0.0) {
        // The push along the car's own directions: +X to its left and +Z
        // ahead. The dead zones apply to each, so a push that's nearly ahead
        // is exactly ahead.
        double x = push[0];
        double z = push[1];
        double left = dead_zones.sideways.apply(x * cos - z * sin);
        double ahead = dead_zones.forward_and_back.apply(x * sin + z * cos);
        double length = std::hypot(left, ahead);
        if (length > 1.0) {
            left /= length;
            ahead /= length;
        }
        left *= left < 0.0 ? acceleration.right : acceleration.left;
        ahead *= ahead < 0.0 ? acceleration.reverse : acceleration.forward;
        // Back along X and Z.
        car.velocity[0] += seconds * (left * cos + ahead * sin);
        car.velocity[1] += seconds * (ahead * cos - left * sin);
    } else {
        double speed = car.speed();
        double slowing = acceleration.reverse * brake * seconds;
        if (speed < Stopped || slowing >= speed) {
            car.velocity = {0.0, 0.0};
        } else {
            for (double& component : car.velocity) {
                component = component * (speed - slowing) / speed;
            }
        }
    }
    if (car.velocity != std::array<double, 2>{0.0, 0.0}) {
        car.heading = std::atan2(car.velocity[0], car.velocity[1]);
    }
    car.position[0] += car.velocity[0] * seconds;
    car.position[1] += car.velocity[1] * seconds;
}

} // namespace veehiicuul::driving
