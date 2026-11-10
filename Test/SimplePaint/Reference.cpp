#include "Reference.h"

#include <algorithm>
#include <cmath>
#include <numbers>

namespace reference {

using simple_paint::InteriorMaximum;
using simple_paint::Margin;
using simple_paint::Parameters;

std::uint64_t Random::next() {
    state = state + 0x9E3779B97F4A7C15;
    std::uint64_t z = state;
    z = (z ^ (z >> 30)) * 0xBF58476D1CE4E5B9;
    z = (z ^ (z >> 27)) * 0x94D049BB133111EB;
    return z ^ (z >> 31);
}

double Random::unit() {
    return static_cast<double>(next() >> 11) / static_cast<double>(std::uint64_t{1} << 53);
}

double Random::range(double low, double high) {
    return low + (high - low) * unit();
}

double linear(double srgb) {
    if (srgb <= 0.04045) {
        return srgb / 12.92;
    }
    return std::pow((srgb + 0.055) / 1.055, 2.4);
}

double srgb(double linear) {
    if (linear <= 0.0031308) {
        return 12.92 * linear;
    }
    return 1.055 * std::pow(linear, 1.0 / 2.4) - 0.055;
}

double curve(double color, double brightness, double tone) {
    double a = linear(color) * brightness;
    double b = (1.0 - linear(color)) * (1.0 - brightness);
    return a * tone / (a * tone + b * (1.0 - tone));
}

std::array<double, 3> reference(const Case& case_) {
    const Parameters& p = case_.parameters;
    double nx = static_cast<double>(case_.normal[0]);
    double ny = static_cast<double>(case_.normal[1]);
    double nz = static_cast<double>(case_.normal[2]);
    double angle = -p.rotation_degrees * std::numbers::pi / 180.0;
    double x = nx * std::cos(angle) - ny * std::sin(angle);
    double y = nx * std::sin(angle) + ny * std::cos(angle);
    double z = nz;
    double length = std::sqrt(x * x + y * y + z * z);
    double facing = 0.0;
    if (z > 0.0) {
        double r = std::hypot(x, z);
        facing = r * z * std::sqrt((1.0 - p.shift) * (1.0 + p.shift)) / (length * (r - p.shift * x));
        facing = std::clamp(facing, 0.0, 1.0);
    }
    double tone = p.dark_point * (1.0 - facing) + p.light_point * facing;
    return {
        curve(p.base_color_srgb[0], p.brightness, tone),
        curve(p.base_color_srgb[1], p.brightness, tone),
        curve(p.base_color_srgb[2], p.brightness, tone),
    };
}

std::vector<Case> cases() {
    std::vector<Case> cases;
    // Cartesian extremes, full, inverted and constant tone ranges, exact
    // axis normals, both slice poles, both sides of the silhouette, and tiny
    // positive slices.
    for (double color : {Margin, 0.04045, 0.5, InteriorMaximum}) {
        for (double brightness : {Margin, 0.126, 0.5, InteriorMaximum}) {
            for (double shift : {0.0, 1.0e-12, 0.5, InteriorMaximum}) {
                for (double rotation : {0.0, 90.0, 187.0, 359.999999}) {
                    for (std::array<double, 2> tones : {std::array{0.0, 1.0}, std::array{InteriorMaximum, Margin}, std::array{0.5, 0.5}}) {
                        double dark = tones[0];
                        double light = tones[1];
                        Parameters parameters{
                            .base_color_srgb = {color, 0.223, InteriorMaximum},
                            .brightness = brightness,
                            .shift = shift,
                            .rotation_degrees = rotation,
                            .dark_point = dark,
                            .light_point = light,
                        };
                        for (Normal normal : {
                                 Normal{0.0f, 0.0f, 1.0f},
                                 Normal{0.0f, 0.0f, -1.0f},
                                 Normal{1.0f, 0.0f, 0.0f},
                                 Normal{0.0f, 1.0f, 0.0f},
                                 Normal{0.0f, -1.0f, 0.0f},
                                 Normal{0.0f, 1.0f, 1.0e-30f},
                                 Normal{1.0e-30f, 1.0f, 1.0e-30f},
                                 Normal{1.0f, 0.0f, -1.0e-6f},
                                 Normal{1.0f, 0.0f, 1.0e-6f},
                             }) {
                            cases.push_back(Case{parameters, normal});
                        }
                        // Dense rings around the lobe maximum: these expose
                        // loss of 1 - facing much more effectively than
                        // uniformly random normals.
                        for (double offset : {-0.01, -0.001, -0.0001, -0.00001, 0.0, 0.00001, 0.0001, 0.001, 0.01}) {
                            double a = std::asin(shift) + offset;
                            double rotation_angle = rotation * std::numbers::pi / 180.0;
                            cases.push_back(Case{
                                parameters,
                                Normal{
                                    static_cast<float>(std::sin(a) * std::cos(rotation_angle)),
                                    static_cast<float>(std::sin(a) * std::sin(rotation_angle)),
                                    static_cast<float>(std::cos(a)),
                                },
                            });
                        }
                    }
                }
            }
        }
    }
    Random random(0x5041494E);
    for (int sample = 0; sample < 16384; sample++) {
        auto interior = [&] {
            return Margin + (1.0 - 2.0 * Margin) * random.unit();
        };
        // One after another, in this order, as the numbers come.
        double red = interior();
        double green = interior();
        double blue = interior();
        double brightness = interior();
        double shift = InteriorMaximum * random.unit();
        double rotation_degrees = 360.0 * random.unit();
        double dark_point = InteriorMaximum * random.unit();
        double light_point = Margin + InteriorMaximum * random.unit();
        Parameters parameters{
            .base_color_srgb = {red, green, blue},
            .brightness = brightness,
            .shift = shift,
            .rotation_degrees = rotation_degrees,
            .dark_point = dark_point,
            .light_point = light_point,
        };
        Normal normal{};
        for (float& component : normal) {
            component = static_cast<float>(random.range(-1.0, 1.0));
        }
        float length = std::sqrt(normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]);
        float scale = std::exp2(-10.0f + 20.0f * static_cast<float>(random.unit()));
        for (float& component : normal) {
            component *= scale / length;
        }
        cases.push_back(Case{parameters, normal});
    }
    return cases;
}

} // namespace reference
