#include "SimplePaint/Material.h"

#include "Base/Text.h"

#include <algorithm>
#include <cmath>
#include <format>
#include <numbers>
#include <string_view>

namespace simple_paint {

namespace {

std::expected<void, Error> require_range(double value, double minimum, double maximum, std::string_view name) {
    // NaN fails every comparison, so it's rejected here too.
    if (std::isfinite(value) && value >= minimum && value <= maximum) {
        return {};
    }
    return std::unexpected(Error{std::format("SimplePaint {} must be finite and in [{}, {}].", name, base::display(minimum), base::display(maximum))});
}

double decode_srgb(double value) {
    if (value <= 0.04045) {
        return value / 12.92;
    }
    return std::pow((value + 0.055) / 1.055, 2.4);
}

} // namespace

std::string describe_value(const Parameters& parameters) {
    return std::format("Parameters {{ base_color_srgb: [{}, {}, {}], brightness: {}, shift: {}, rotation_degrees: {}, dark_point: {}, light_point: {} }}",
                       base::debug(parameters.base_color_srgb[0]), base::debug(parameters.base_color_srgb[1]), base::debug(parameters.base_color_srgb[2]),
                       base::debug(parameters.brightness), base::debug(parameters.shift), base::debug(parameters.rotation_degrees), base::debug(parameters.dark_point),
                       base::debug(parameters.light_point));
}

std::expected<Material, Error> Material::compile(const Parameters& parameters) {
    if (std::expected<void, Error> valid = validate_parameters(parameters); !valid) {
        return std::unexpected(valid.error());
    }
    const Parameters& p = parameters;
    double radians = -p.rotation_degrees * std::numbers::pi / 180.0;
    GpuMaterial gpu{
        .warp =
            {
                static_cast<float>(std::cos(radians)),
                static_cast<float>(std::sin(radians)),
                static_cast<float>(std::sqrt((1.0 - p.shift) / (1.0 + p.shift))),
                p.shift == 0.0 ? 0.0f : 1.0f,
            },
    };
    for (size_t channel = 0; channel < 3; channel++) {
        double c = decode_srgb(p.base_color_srgb[channel]);
        double a = c * p.brightness;
        double b = (1.0 - c) * (1.0 - p.brightness);
        // Compose the tone remap with Schlick on the CPU, retaining
        // positive endpoint contributions. A common scale cancels from
        // the ratio, and ensures at least one coefficient per channel is
        // exactly one.
        double nd = a * p.dark_point;
        double nl = a * p.light_point;
        double cd = b * (1.0 - p.dark_point);
        double cl = b * (1.0 - p.light_point);
        double scale = std::max(std::max(std::max(nd, nl), cd), cl);
        gpu.numerator_dark[channel] = static_cast<float>(nd / scale);
        gpu.numerator_light[channel] = static_cast<float>(nl / scale);
        gpu.complement_dark[channel] = static_cast<float>(cd / scale);
        gpu.complement_light[channel] = static_cast<float>(cl / scale);
    }
    return Material(gpu);
}

std::expected<void, Error> validate_parameters(const Parameters& p) {
    if (std::expected<void, Error> in_range = require_range(p.base_color_srgb[0], Margin, InteriorMaximum, "R"); !in_range) {
        return in_range;
    }
    if (std::expected<void, Error> in_range = require_range(p.base_color_srgb[1], Margin, InteriorMaximum, "G"); !in_range) {
        return in_range;
    }
    if (std::expected<void, Error> in_range = require_range(p.base_color_srgb[2], Margin, InteriorMaximum, "B"); !in_range) {
        return in_range;
    }
    if (std::expected<void, Error> in_range = require_range(p.brightness, Margin, InteriorMaximum, "Brightness"); !in_range) {
        return in_range;
    }
    if (std::expected<void, Error> in_range = require_range(p.shift, 0.0, InteriorMaximum, "Shift"); !in_range) {
        return in_range;
    }
    if (std::expected<void, Error> in_range = require_range(p.dark_point, 0.0, InteriorMaximum, "Dark Point"); !in_range) {
        return in_range;
    }
    if (std::expected<void, Error> in_range = require_range(p.light_point, Margin, 1.0, "Light Point"); !in_range) {
        return in_range;
    }
    double rotation = p.rotation_degrees;
    if (!std::isfinite(rotation) || !(rotation >= 0.0 && rotation < 360.0)) {
        return std::unexpected(Error{"SimplePaint Rotation must be finite and in [0, 360) degrees."});
    }
    return {};
}

} // namespace simple_paint
