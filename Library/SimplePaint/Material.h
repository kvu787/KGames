// SimplePaint's portable material interface: the parameters, their
// validation, and the GPU constants compiled from them. No DirectX types or
// renderer state. Specification.md defines the input contract and the
// constants' layout.

#pragma once

#include "SimplePaint/Error.h"

#include <array>
#include <cstddef>
#include <expected>
#include <string>

namespace simple_paint {

// The practical binary32 domain, distinct from the open real-number domain.
// These limits reject inputs; they are never shader clamps. Exact powers of
// two make boundary tests unambiguous.
inline constexpr double Margin = 1.0 / 1024.0;
inline constexpr double InteriorMaximum = 1.0 - Margin;

struct Parameters {
    // User-facing sRGB channels, each in [Margin, InteriorMaximum].
    std::array<double, 3> base_color_srgb{0.107, 0.223, 0.578};
    // The base color anchors at tone 1 - brightness. The same interior
    // limits as RGB.
    double brightness = 0.5;
    // In [0, InteriorMaximum]. Zero disables directional warping.
    double shift = 0.0;
    // A counterclockwise screen angle in [0, 360). Zero points right.
    double rotation_degrees = 0.0;
    // The tones at facing 0 and 1. Dark is in [0, InteriorMaximum], and
    // light in [Margin, 1]. Either order is valid, and so is equality, which
    // paints a constant color.
    double dark_point = 0.0;
    double light_point = 1.0;

    bool operator==(const Parameters&) const = default;
};

// How a failed check in a test shows the parameters.
std::string describe_value(const Parameters& parameters);

// Five HLSL float4 registers. The fourth color lanes are reserved and zero.
// Copy it verbatim into a constant buffer, whose start must be 256-byte
// aligned. Instances that a shader reads come only from Material::compile.
struct alignas(16) GpuMaterial {
    // cos(-rotation), sin(-rotation), sqrt((1 - shift) / (1 + shift)), and
    // a zero or one flag for whether the requested shift is nonzero.
    std::array<float, 4> warp{};
    std::array<float, 4> numerator_dark{};
    std::array<float, 4> numerator_light{};
    std::array<float, 4> complement_dark{};
    std::array<float, 4> complement_light{};

    bool operator==(const GpuMaterial&) const = default;
};

static_assert(sizeof(GpuMaterial) == 80 && alignof(GpuMaterial) == 16);
static_assert(offsetof(GpuMaterial, numerator_dark) == 16);
static_assert(offsetof(GpuMaterial, numerator_light) == 32);
static_assert(offsetof(GpuMaterial, complement_dark) == 48);
static_assert(offsetof(GpuMaterial, complement_light) == 64);

class Material {
public:
    Material(const Material&) = delete;
    Material& operator=(const Material&) = delete;
    Material(Material&&) = default;
    Material& operator=(Material&&) = default;

    // Validates all parameters in binary64 before narrowing or computing
    // anything. Rejects them with the field and its accepted range. It never
    // clamps, wraps the rotation, or substitutes a different material.
    static std::expected<Material, Error> compile(const Parameters& parameters);

    const GpuMaterial& constants() const {
        return gpu;
    }

private:
    explicit Material(const GpuMaterial& compiled) : gpu(compiled) {}

    GpuMaterial gpu;
};

// Checks the portable input without computing GPU constants.
std::expected<void, Error> validate_parameters(const Parameters& p);

} // namespace simple_paint
