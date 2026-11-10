// SimplePaint's CPU contract: parameter validation at every boundary, the
// packed color curve against the reference, the curve's anchors and
// endpoints, transform validation, mesh validation, and the specification's
// two facing equations against each other.

#include "Base/Text.h"
#include "Math/Math.h"
#include "SimplePaint/Geometry.h"
#include "SimplePaint/Material.h"
#include "SimplePaint/Orthographic.h"
#include "Testing/Testing.h"

#include "Reference.h"

#include <array>
#include <cmath>
#include <cstdint>
#include <limits>
#include <print>
#include <span>

using math::Matrix;
using reference::Random;
using simple_paint::InteriorMaximum;
using simple_paint::Margin;
using simple_paint::Material;
using simple_paint::Parameters;
using simple_paint::validate_mesh;
using simple_paint::Vertex;
using simple_paint::orthographic::build_object_transforms;
using simple_paint::orthographic::make_projection;

namespace {

constexpr double Infinity = std::numeric_limits<double>::infinity();

// Every scalar on its own, including the adjacent binary64 values at each
// boundary, so invalid input can't become valid by rounding to binary32.
void check_parameter_boundaries() {
    for (size_t field = 0; field < 8; field++) {
        double low = field < 4 || field == 7 ? Margin : 0.0;
        double high = field == 5 ? 360.0 : field == 7 ? 1.0 : InteriorMaximum;
        auto with = [&](double value) {
            Parameters p;
            switch (field) {
            case 0:
            case 1:
            case 2:
                p.base_color_srgb[field] = value;
                break;
            case 3:
                p.brightness = value;
                break;
            case 4:
                p.shift = value;
                break;
            case 5:
                p.rotation_degrees = value;
                break;
            case 6:
                p.dark_point = value;
                break;
            default:
                p.light_point = value;
                break;
            }
            return p;
        };
        for (double bad : {std::nextafter(low, -Infinity), std::nextafter(high, Infinity), -Infinity, Infinity, std::numeric_limits<double>::quiet_NaN()}) {
            CHECK(!Material::compile(with(bad)).has_value(), "field {} accepted {}", field, base::display(bad));
        }
        (void)Material::compile(with(low)).value();
        // Rotation excludes 360 itself.
        double top = field == 5 ? std::nextafter(high, -Infinity) : high;
        (void)Material::compile(with(top)).value();
        if (field == 5) {
            CHECK(!Material::compile(with(360.0)).has_value());
        }
    }
    // Negative zero is accepted where zero is.
    (void)Material::compile(Parameters{.shift = -0.0, .dark_point = -0.0}).value();
}

// The packed GPU endpoint coefficients against the reference over the whole
// tone interval, with the anchors, endpoints and monotonicity.
size_t check_curve() {
    size_t checks = 0;
    for (double c : {Margin, 0.04045, 0.223, 0.5, InteriorMaximum}) {
        for (double b : {Margin, 0.126, 0.5, InteriorMaximum}) {
            Parameters parameters{
                .base_color_srgb = {c, c, c},
                .brightness = b,
                .shift = 0.0,
                .rotation_degrees = 0.0,
                .dark_point = 0.0,
                .light_point = 1.0,
            };
            Material material = Material::compile(parameters).value();
            const simple_paint::GpuMaterial& m = material.constants();
            // The curve that the shader computes from the packed coefficients.
            auto packed = [&](double f) {
                double numerator = static_cast<double>(m.numerator_dark[0]) * (1.0 - f) + static_cast<double>(m.numerator_light[0]) * f;
                double complement = static_cast<double>(m.complement_dark[0]) * (1.0 - f) + static_cast<double>(m.complement_light[0]) * f;
                return numerator / (numerator + complement);
            };
            double previous = -1.0;
            for (int i = 0; i <= 4096; i++) {
                double f = static_cast<double>(i) / 4096.0;
                double value = packed(f);
                CHECK(std::abs(value - reference::curve(c, b, f)) < 1.0e-7, "Packed curve differs from reference");
                CHECK(value >= previous, "Schlick curve is not monotone");
                previous = value;
                checks += 1;
            }
            // The anchor and endpoints, in the reference, and in the packed
            // curve, which is the reference's within binary32 rounding.
            CHECK(std::abs(reference::curve(c, b, 1.0 - b) - reference::linear(c)) < 1.0e-13, "Base-color anchor failed");
            CHECK(reference::curve(c, b, 0.0) == 0.0 && reference::curve(c, b, 1.0) == 1.0, "Endpoints failed");
            CHECK(std::abs(packed(1.0 - b) - reference::linear(c)) < 1.0e-7, "Packed base-color anchor failed");
            CHECK(packed(0.0) == 0.0 && packed(1.0) == 1.0, "Packed endpoints failed");
        }
    }
    return checks;
}

void check_transform_validation() {
    simple_paint::orthographic::Projection projection = make_projection(5.0f, 5.0f, 1.0f, 20.0f).value();
    CHECK(!make_projection(0.0f, 5.0f, 1.0f, 20.0f).has_value());
    CHECK(!make_projection(5.0f, 5.0f, 20.0f, 1.0f).has_value());
    CHECK(!build_object_transforms(Matrix::scaling(1.0f, 2.0f, 1.0f), projection).has_value());
    CHECK(!build_object_transforms(Matrix::scaling(0.0f, 0.0f, 0.0f), projection).has_value());
    // Like XMMatrixPerspectiveFovRH(1, 1, 1, 20).
    float height = 1.0f / std::tan(0.5f);
    float range = 20.0f / (1.0f - 20.0f);
    Matrix perspective{{{
        {height, 0.0f, 0.0f, 0.0f},
        {0.0f, height, 0.0f, 0.0f},
        {0.0f, 0.0f, range, -1.0f},
        {0.0f, 0.0f, range, 0.0f},
    }}};
    CHECK(!build_object_transforms(perspective, projection).has_value());
    Matrix shear{{{
        {1.0f, 0.0f, 0.0f, 0.0f},
        {0.5f, 1.0f, 0.0f, 0.0f},
        {0.0f, 0.0f, 1.0f, 0.0f},
        {0.0f, 0.0f, 0.0f, 1.0f},
    }}};
    CHECK(!build_object_transforms(shear, projection).has_value());
    Matrix rotated = Matrix::scaling(0.5f, 0.5f, 0.5f) * Matrix::rotation_y(0.7f) * Matrix::translation(3.0f, 1.0f, -2.0f);
    (void)build_object_transforms(rotated, projection).value();
}

void check_mesh_validation() {
    auto vertex = [](std::array<float, 3> position, std::array<float, 3> normal, std::uint32_t material_index) {
        return Vertex{
            .position = position,
            .normal = normal,
            .material_index = material_index,
        };
    };
    std::array<float, 3> up{0.0f, 0.0f, 1.0f};
    std::array<Vertex, 3> triangle{
        vertex({0.0f, 0.0f, 0.0f}, up, 0),
        vertex({1.0f, 0.0f, 0.0f}, up, 0),
        vertex({0.0f, 1.0f, 0.0f}, up, 0),
    };
    using Indices = std::array<std::uint32_t, 3>;
    validate_mesh(triangle, Indices{0, 1, 2}, 1).value();
    CHECK(!validate_mesh(triangle, std::span<const std::uint32_t>(), 1).has_value(), "empty indices");
    CHECK(!validate_mesh(triangle, (std::array<std::uint32_t, 2>{0, 1}), 1).has_value(), "a partial triangle");
    CHECK(!validate_mesh(triangle, (Indices{0, 1, 3}), 1).has_value(), "an index out of bounds");
    CHECK(!validate_mesh(triangle, (Indices{0, 1, 2}), 0).has_value(), "no materials");
    std::array<Vertex, 3> mixed = triangle;
    mixed[2].material_index = 1;
    CHECK(!validate_mesh(mixed, (Indices{0, 1, 2}), 2).has_value(), "two materials in a triangle");
    CHECK(!validate_mesh(mixed, (Indices{0, 1, 2}), 1).has_value(), "a material out of range");
    std::array<Vertex, 3> short_ = triangle;
    short_[0].normal = {0.0f, 0.0f, 0.49f};
    CHECK(!validate_mesh(short_, (Indices{0, 1, 2}), 1).has_value(), "a short normal");
    std::array<Vertex, 3> far_ = triangle;
    far_[1].position = {1.0e7f, 0.0f, 0.0f};
    CHECK(!validate_mesh(far_, (Indices{0, 1, 2}), 1).has_value(), "a position out of bounds");
    std::array<Vertex, 3> opposed = triangle;
    opposed[1].normal = {0.0f, 0.0f, -1.0f};
    CHECK(!validate_mesh(opposed, (Indices{0, 1, 2}), 1).has_value(), "normals without a common cone");
}

// The specification's circular-slice and Schlick construction against its
// rational facing equation, away from ill-conditioned slice endpoints.
void check_facing_equations() {
    Random random(0x4B3132);
    for (int sample = 0; sample < 20000; sample++) {
        double y = 1.8 * random.unit() - 0.9;
        double r = std::sqrt(1.0 - y * y);
        double angle = 2.8 * random.unit() - 1.4;
        double x = r * std::sin(angle);
        double z = r * std::cos(angle);
        double s = InteriorMaximum * random.unit();
        double t = (x + r) / (2.0 * r);
        double u1 = (1.0 + s) / 2.0;
        double u2 = 1.0 - u1;
        double schlick = u2 * t / (u1 * (1.0 - t) + u2 * t);
        double warped_x = -r + 2.0 * r * schlick;
        double slice_facing = std::sqrt(std::fmax(1.0 - warped_x * warped_x - y * y, 0.0));
        double reduced = r * z * std::sqrt((1.0 - s) * (1.0 + s)) / (r - s * x);
        CHECK(std::abs(slice_facing - reduced) < 1.0e-11, "SimplePaint slice/Schlick and rational facing equations differ");
    }
}

} // namespace

TEST(contract) {
    check_parameter_boundaries();
    size_t checks = check_curve();
    check_transform_validation();
    check_mesh_validation();
    check_facing_equations();
    std::println("{} curve comparisons; parameter boundaries, anchors, transform and mesh validation passed.", checks);
    std::println("20,000 independent SimplePaint slice/Schlick facing comparisons passed.");
}
