// SimplePaint's CPU contract: parameter validation at every boundary, the
// packed color curve against the reference, the curve's anchors and
// endpoints, transform validation, mesh validation, and the specification's
// two facing equations against each other.

#[path = "Reference.rs"]
mod reference;

use math::Matrix;
use reference::Random;
use simple_paint::orthographic::{build_object_transforms, make_projection};
use simple_paint::{INTERIOR_MAXIMUM, MARGIN, Material, Parameters, Vertex, validate_mesh};

fn main() {
    check_parameter_boundaries();
    let checks = check_curve();
    check_transform_validation();
    check_mesh_validation();
    check_facing_equations();
    println!("{checks} curve comparisons; parameter boundaries, anchors, transform and mesh validation passed.");
    println!("20,000 independent SimplePaint slice/Schlick facing comparisons passed.");
}

// Every scalar on its own, including the adjacent binary64 values at each
// boundary, so invalid input can't become valid by rounding to binary32.
fn check_parameter_boundaries() {
    for field in 0..8 {
        let low = if field < 4 || field == 7 { MARGIN } else { 0.0 };
        let high = match field {
            5 => 360.0,
            7 => 1.0,
            _ => INTERIOR_MAXIMUM,
        };
        let with = |value: f64| {
            let mut p = Parameters::default();
            match field {
                0..=2 => p.base_color_srgb[field] = value,
                3 => p.brightness = value,
                4 => p.shift = value,
                5 => p.rotation_degrees = value,
                6 => p.dark_point = value,
                _ => p.light_point = value,
            }
            p
        };
        for bad in [low.next_down(), high.next_up(), f64::NEG_INFINITY, f64::INFINITY, f64::NAN] {
            assert!(
                Material::compile(&with(bad)).is_err(),
                "field {field} accepted {bad}"
            );
        }
        Material::compile(&with(low)).unwrap();
        // Rotation excludes 360 itself.
        let top = if field == 5 { high.next_down() } else { high };
        Material::compile(&with(top)).unwrap();
        if field == 5 {
            assert!(Material::compile(&with(360.0)).is_err());
        }
    }
    // Negative zero is accepted where zero is.
    Material::compile(&Parameters {
        shift: -0.0,
        dark_point: -0.0,
        ..Default::default()
    })
    .unwrap();
}

// The packed GPU endpoint coefficients against the reference over the whole
// tone interval, with the anchors, endpoints and monotonicity.
fn check_curve() -> usize {
    let mut checks = 0;
    for c in [MARGIN, 0.04045, 0.223, 0.5, INTERIOR_MAXIMUM] {
        for b in [MARGIN, 0.126, 0.5, INTERIOR_MAXIMUM] {
            let parameters = Parameters {
                base_color_srgb: [c, c, c],
                brightness: b,
                shift: 0.0,
                rotation_degrees: 0.0,
                dark_point: 0.0,
                light_point: 1.0,
            };
            let material = Material::compile(&parameters).unwrap();
            let m = material.constants();
            let mut previous = -1.0;
            for i in 0..=4096 {
                let f = i as f64 / 4096.0;
                let numerator = m.numerator_dark[0] as f64 * (1.0 - f) + m.numerator_light[0] as f64 * f;
                let complement =
                    m.complement_dark[0] as f64 * (1.0 - f) + m.complement_light[0] as f64 * f;
                let value = numerator / (numerator + complement);
                assert!(
                    (value - reference::curve(c, b, f)).abs() < 1.0e-7,
                    "Packed curve differs from reference"
                );
                assert!(value >= previous, "Schlick curve is not monotone");
                previous = value;
                checks += 1;
            }
            assert!(
                (reference::curve(c, b, 1.0 - b) - reference::linear(c)).abs() < 1.0e-13,
                "Base-color anchor failed"
            );
            assert!(
                reference::curve(c, b, 0.0) == 0.0 && reference::curve(c, b, 1.0) == 1.0,
                "Endpoints failed"
            );
        }
    }
    checks
}

fn check_transform_validation() {
    let projection = make_projection(5.0, 5.0, 1.0, 20.0).unwrap();
    assert!(make_projection(0.0, 5.0, 1.0, 20.0).is_err());
    assert!(make_projection(5.0, 5.0, 20.0, 1.0).is_err());
    assert!(build_object_transforms(&Matrix::scaling(1.0, 2.0, 1.0), &projection).is_err());
    assert!(build_object_transforms(&Matrix::scaling(0.0, 0.0, 0.0), &projection).is_err());
    // Like XMMatrixPerspectiveFovRH(1, 1, 1, 20).
    let height = 1.0 / 0.5f32.tan();
    let range = 20.0 / (1.0 - 20.0);
    let perspective = Matrix([
        [height, 0.0, 0.0, 0.0],
        [0.0, height, 0.0, 0.0],
        [0.0, 0.0, range, -1.0],
        [0.0, 0.0, range, 0.0],
    ]);
    assert!(build_object_transforms(&perspective, &projection).is_err());
    let shear = Matrix([
        [1.0, 0.0, 0.0, 0.0],
        [0.5, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]);
    assert!(build_object_transforms(&shear, &projection).is_err());
    let rotated = Matrix::scaling(0.5, 0.5, 0.5) * Matrix::rotation_y(0.7) * Matrix::translation(3.0, 1.0, -2.0);
    build_object_transforms(&rotated, &projection).unwrap();
}

fn check_mesh_validation() {
    let vertex = |position: [f32; 3], normal: [f32; 3], material_index| Vertex {
        position,
        normal,
        material_index,
    };
    let up = [0.0, 0.0, 1.0];
    let triangle = [
        vertex([0.0, 0.0, 0.0], up, 0),
        vertex([1.0, 0.0, 0.0], up, 0),
        vertex([0.0, 1.0, 0.0], up, 0),
    ];
    validate_mesh(&triangle, &[0, 1, 2], 1).unwrap();
    assert!(validate_mesh(&triangle, &[], 1).is_err(), "empty indices");
    assert!(validate_mesh(&triangle, &[0, 1], 1).is_err(), "a partial triangle");
    assert!(validate_mesh(&triangle, &[0, 1, 3], 1).is_err(), "an index out of bounds");
    assert!(validate_mesh(&triangle, &[0, 1, 2], 0).is_err(), "no materials");
    let mut mixed = triangle;
    mixed[2].material_index = 1;
    assert!(validate_mesh(&mixed, &[0, 1, 2], 2).is_err(), "two materials in a triangle");
    assert!(validate_mesh(&mixed, &[0, 1, 2], 1).is_err(), "a material out of range");
    let mut short = triangle;
    short[0].normal = [0.0, 0.0, 0.49];
    assert!(validate_mesh(&short, &[0, 1, 2], 1).is_err(), "a short normal");
    let mut far = triangle;
    far[1].position = [1.0e7, 0.0, 0.0];
    assert!(validate_mesh(&far, &[0, 1, 2], 1).is_err(), "a position out of bounds");
    let mut opposed = triangle;
    opposed[1].normal = [0.0, 0.0, -1.0];
    assert!(validate_mesh(&opposed, &[0, 1, 2], 1).is_err(), "normals without a common cone");
}

// The specification's circular-slice and Schlick construction against its
// rational facing equation, away from ill-conditioned slice endpoints.
fn check_facing_equations() {
    let mut random = Random::new(0x4B_3132);
    for _ in 0..20000 {
        let y = 1.8 * random.unit() - 0.9;
        let r = (1.0 - y * y).sqrt();
        let angle = 2.8 * random.unit() - 1.4;
        let (x, z) = (r * angle.sin(), r * angle.cos());
        let s = INTERIOR_MAXIMUM * random.unit();
        let t = (x + r) / (2.0 * r);
        let u1 = (1.0 + s) / 2.0;
        let u2 = 1.0 - u1;
        let schlick = u2 * t / (u1 * (1.0 - t) + u2 * t);
        let warped_x = -r + 2.0 * r * schlick;
        let slice_facing = (1.0 - warped_x * warped_x - y * y).max(0.0).sqrt();
        let reduced = r * z * ((1.0 - s) * (1.0 + s)).sqrt() / (r - s * x);
        assert!(
            (slice_facing - reduced).abs() < 1.0e-11,
            "SimplePaint slice/Schlick and rational facing equations differ"
        );
    }
}
