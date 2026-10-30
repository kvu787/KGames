// An independent binary64 evaluation of Specification.md, and the cases that
// the GPU test compares it with. Shared by the tests with #[path].

#![allow(dead_code)]

use simple_paint::{INTERIOR_MAXIMUM, MARGIN, Parameters};

pub type Normal = [f32; 3];

#[derive(Clone, Copy)]
pub struct Case {
    pub parameters: Parameters,
    pub normal: Normal,
}

// SplitMix64, so the cases are the same on every run without a crate.
pub struct Random(u64);

impl Random {
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }

    pub fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    // Uniform in [0, 1).
    pub fn unit(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1u64 << 53) as f64
    }

    // Uniform in [low, high).
    pub fn range(&mut self, low: f64, high: f64) -> f64 {
        low + (high - low) * self.unit()
    }
}

pub fn linear(srgb: f64) -> f64 {
    if srgb <= 0.04045 {
        srgb / 12.92
    } else {
        ((srgb + 0.055) / 1.055).powf(2.4)
    }
}

pub fn srgb(linear: f64) -> f64 {
    if linear <= 0.0031308 {
        12.92 * linear
    } else {
        1.055 * linear.powf(1.0 / 2.4) - 0.055
    }
}

pub fn curve(color: f64, brightness: f64, tone: f64) -> f64 {
    let a = linear(color) * brightness;
    let b = (1.0 - linear(color)) * (1.0 - brightness);
    a * tone / (a * tone + b * (1.0 - tone))
}

// The rational facing of the normalized, rotated normal, the explicit tone
// interpolation, and the uncomposed Schlick color function. Only roundoff
// beyond the mathematical [0, 1] facing range is removed here.
pub fn reference(case: &Case) -> [f64; 3] {
    let p = &case.parameters;
    let [nx, ny, nz] = case.normal.map(|n| n as f64);
    let angle = -p.rotation_degrees * std::f64::consts::PI / 180.0;
    let x = nx * angle.cos() - ny * angle.sin();
    let y = nx * angle.sin() + ny * angle.cos();
    let z = nz;
    let length = (x * x + y * y + z * z).sqrt();
    let mut facing = 0.0;
    if z > 0.0 {
        let r = x.hypot(z);
        facing = r * z * ((1.0 - p.shift) * (1.0 + p.shift)).sqrt() / (length * (r - p.shift * x));
        facing = facing.clamp(0.0, 1.0);
    }
    let tone = p.dark_point * (1.0 - facing) + p.light_point * facing;
    p.base_color_srgb
        .map(|color| curve(color, p.brightness, tone))
}

pub fn cases() -> Vec<Case> {
    let mut cases = Vec::new();
    // Cartesian extremes, full, inverted and constant tone ranges, exact
    // axis normals, both slice poles, both sides of the silhouette, and tiny
    // positive slices.
    for color in [MARGIN, 0.04045, 0.5, INTERIOR_MAXIMUM] {
        for brightness in [MARGIN, 0.126, 0.5, INTERIOR_MAXIMUM] {
            for shift in [0.0, 1.0e-12, 0.5, INTERIOR_MAXIMUM] {
                for rotation in [0.0, 90.0, 187.0, 359.999999] {
                    for [dark, light] in [[0.0, 1.0], [INTERIOR_MAXIMUM, MARGIN], [0.5, 0.5]] {
                        let parameters = Parameters {
                            base_color_srgb: [color, 0.223, INTERIOR_MAXIMUM],
                            brightness,
                            shift,
                            rotation_degrees: rotation,
                            dark_point: dark,
                            light_point: light,
                        };
                        for normal in [
                            [0.0, 0.0, 1.0],
                            [0.0, 0.0, -1.0],
                            [1.0, 0.0, 0.0],
                            [0.0, 1.0, 0.0],
                            [0.0, -1.0, 0.0],
                            [0.0, 1.0, 1.0e-30],
                            [1.0e-30, 1.0, 1.0e-30],
                            [1.0, 0.0, -1.0e-6],
                            [1.0, 0.0, 1.0e-6],
                        ] {
                            cases.push(Case { parameters, normal });
                        }
                        // Dense rings around the lobe maximum: these expose
                        // loss of 1 - facing much more effectively than
                        // uniformly random normals.
                        for offset in [
                            -0.01, -0.001, -0.0001, -0.00001, 0.0, 0.00001, 0.0001, 0.001, 0.01,
                        ] {
                            let a = shift.asin() + offset;
                            let rotation_angle = rotation * std::f64::consts::PI / 180.0;
                            cases.push(Case {
                                parameters,
                                normal: [
                                    (a.sin() * rotation_angle.cos()) as f32,
                                    (a.sin() * rotation_angle.sin()) as f32,
                                    a.cos() as f32,
                                ],
                            });
                        }
                    }
                }
            }
        }
    }
    let mut random = Random::new(0x5041_494E);
    for _ in 0..16384 {
        let mut interior = || MARGIN + (1.0 - 2.0 * MARGIN) * random.unit();
        let base_color_srgb = [interior(), interior(), interior()];
        let brightness = interior();
        let parameters = Parameters {
            base_color_srgb,
            brightness,
            shift: INTERIOR_MAXIMUM * random.unit(),
            rotation_degrees: 360.0 * random.unit(),
            dark_point: INTERIOR_MAXIMUM * random.unit(),
            light_point: MARGIN + INTERIOR_MAXIMUM * random.unit(),
        };
        let mut normal: Normal = std::array::from_fn(|_| random.range(-1.0, 1.0) as f32);
        let length = normal.iter().map(|n| n * n).sum::<f32>().sqrt();
        let scale = (-10.0 + 20.0 * random.unit() as f32).exp2();
        for component in &mut normal {
            *component *= scale / length;
        }
        cases.push(Case { parameters, normal });
    }
    cases
}
