// SimplePaint's portable material interface: the parameters, their
// validation, and the GPU constants compiled from them. No DirectX types or
// renderer state. Specification.md defines the input contract and the
// constants' layout.

use std::mem::offset_of;

use crate::Error;

// The practical binary32 domain, distinct from the open real-number domain.
// These limits reject inputs; they are never shader clamps. Exact powers of
// two make boundary tests unambiguous.
pub const MARGIN: f64 = 1.0 / 1024.0;
pub const INTERIOR_MAXIMUM: f64 = 1.0 - MARGIN;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Parameters {
    // User-facing sRGB channels, each in [MARGIN, INTERIOR_MAXIMUM].
    pub base_color_srgb: [f64; 3],
    // The base color anchors at tone 1 - brightness. The same interior
    // limits as RGB.
    pub brightness: f64,
    // In [0, INTERIOR_MAXIMUM]. Zero disables directional warping.
    pub shift: f64,
    // A counterclockwise screen angle in [0, 360). Zero points right.
    pub rotation_degrees: f64,
    // The tones at facing 0 and 1. Dark is in [0, INTERIOR_MAXIMUM], and
    // light in [MARGIN, 1]. Either order is valid, and so is equality, which
    // paints a constant color.
    pub dark_point: f64,
    pub light_point: f64,
}

impl Default for Parameters {
    fn default() -> Self {
        Self {
            base_color_srgb: [0.107, 0.223, 0.578],
            brightness: 0.5,
            shift: 0.0,
            rotation_degrees: 0.0,
            dark_point: 0.0,
            light_point: 1.0,
        }
    }
}

// Five HLSL float4 registers. The fourth color lanes are reserved and zero.
// Copy it verbatim into a constant buffer, whose start must be 256-byte
// aligned. Instances that a shader reads come only from Material::compile.
#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GpuMaterial {
    // cos(-rotation), sin(-rotation), sqrt((1 - shift) / (1 + shift)), and
    // a zero or one flag for whether the requested shift is nonzero.
    pub warp: [f32; 4],
    pub numerator_dark: [f32; 4],
    pub numerator_light: [f32; 4],
    pub complement_dark: [f32; 4],
    pub complement_light: [f32; 4],
}

const _: () = {
    assert!(size_of::<GpuMaterial>() == 80 && align_of::<GpuMaterial>() == 16);
    assert!(offset_of!(GpuMaterial, numerator_dark) == 16);
    assert!(offset_of!(GpuMaterial, numerator_light) == 32);
    assert!(offset_of!(GpuMaterial, complement_dark) == 48);
    assert!(offset_of!(GpuMaterial, complement_light) == 64);
};

pub struct Material {
    gpu: GpuMaterial,
}

impl Material {
    // Validates all parameters in binary64 before narrowing or computing
    // anything. Rejects them with the field and its accepted range. It never
    // clamps, wraps the rotation, or substitutes a different material.
    pub fn compile(parameters: &Parameters) -> Result<Material, Error> {
        validate_parameters(parameters)?;
        let p = parameters;
        let radians = -p.rotation_degrees * std::f64::consts::PI / 180.0;
        let mut gpu = GpuMaterial {
            warp: [
                radians.cos() as f32,
                radians.sin() as f32,
                ((1.0 - p.shift) / (1.0 + p.shift)).sqrt() as f32,
                if p.shift == 0.0 { 0.0 } else { 1.0 },
            ],
            ..Default::default()
        };
        for channel in 0..3 {
            let c = decode_srgb(p.base_color_srgb[channel]);
            let a = c * p.brightness;
            let b = (1.0 - c) * (1.0 - p.brightness);
            // Compose the tone remap with Schlick on the CPU, retaining
            // positive endpoint contributions. A common scale cancels from
            // the ratio, and ensures at least one coefficient per channel is
            // exactly one.
            let nd = a * p.dark_point;
            let nl = a * p.light_point;
            let cd = b * (1.0 - p.dark_point);
            let cl = b * (1.0 - p.light_point);
            let scale = nd.max(nl).max(cd).max(cl);
            gpu.numerator_dark[channel] = (nd / scale) as f32;
            gpu.numerator_light[channel] = (nl / scale) as f32;
            gpu.complement_dark[channel] = (cd / scale) as f32;
            gpu.complement_light[channel] = (cl / scale) as f32;
        }
        Ok(Material { gpu })
    }

    pub fn constants(&self) -> &GpuMaterial {
        &self.gpu
    }
}

// Checks the portable input without computing GPU constants.
pub fn validate_parameters(p: &Parameters) -> Result<(), Error> {
    require_range(p.base_color_srgb[0], MARGIN, INTERIOR_MAXIMUM, "R")?;
    require_range(p.base_color_srgb[1], MARGIN, INTERIOR_MAXIMUM, "G")?;
    require_range(p.base_color_srgb[2], MARGIN, INTERIOR_MAXIMUM, "B")?;
    require_range(p.brightness, MARGIN, INTERIOR_MAXIMUM, "Brightness")?;
    require_range(p.shift, 0.0, INTERIOR_MAXIMUM, "Shift")?;
    require_range(p.dark_point, 0.0, INTERIOR_MAXIMUM, "Dark Point")?;
    require_range(p.light_point, MARGIN, 1.0, "Light Point")?;
    let rotation = p.rotation_degrees;
    if !rotation.is_finite() || !(0.0..360.0).contains(&rotation) {
        return Err(Error(
            "SimplePaint Rotation must be finite and in [0, 360) degrees.".to_string(),
        ));
    }
    Ok(())
}

fn require_range(value: f64, minimum: f64, maximum: f64, name: &str) -> Result<(), Error> {
    // NaN fails every comparison, so it's rejected here too.
    if value.is_finite() && value >= minimum && value <= maximum {
        Ok(())
    } else {
        Err(Error(format!(
            "SimplePaint {name} must be finite and in [{minimum}, {maximum}]."
        )))
    }
}

fn decode_srgb(value: f64) -> f64 {
    if value <= 0.04045 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}
