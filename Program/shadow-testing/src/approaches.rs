// The ways of drawing an omnidirectional shadow that are being tried, side
// by side: shadows that spread from the box the same way on every side, as
// if lit from straight above, without an offset or a direction. Each is a
// Kind, with a module of its own that has its settings, which sliders set,
// and how it draws, and a shader of its own in Shaders, which includes
// Common.hlsli. Each module also works out its shadow's opacity at a point on
// the CPU, as its shader does, so that tests can check every pixel that the
// GPU draws.
//
// Every way's defaults are matched, so that the ways differ only in the
// shape of their falloff: each starts about DEFAULT_EDGE_OPACITY at the box's
// edge, a subtle shadow on the light theme's background, and is half as dark
// about DEFAULT_HALF_DISTANCE out from it.

use user_interface::Rect;

use crate::{b_spline, bell, curved, exponential, gaussian, layered, linear, round_gaussian, smoothstep};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    Linear,
    Curved,
    Smoothstep,
    Exponential,
    Bell,
    Gaussian,
    RoundGaussian,
    BSpline,
    Layered,
}

// Every way, in the order the page shows them: those whose shadow goes by
// the distance from the box alone, then the Gaussian and its variants.
pub const KINDS: [Kind; 9] = [
    Kind::Linear,
    Kind::Curved,
    Kind::Smoothstep,
    Kind::Exponential,
    Kind::Bell,
    Kind::Gaussian,
    Kind::RoundGaussian,
    Kind::BSpline,
    Kind::Layered,
];

// The faintest opacity that changes a pixel: half of an 8-bit step. A
// shadow whose tail never ends stops drawing where it's fainter.
pub const FAINTEST: f32 = 0.5 / 255.0;

// What every way's defaults are matched to: the opacity at the box's edge,
// and how far out it's half that, in pixels.
pub const DEFAULT_EDGE_OPACITY: f32 = 0.25;
pub const DEFAULT_HALF_DISTANCE: f32 = 10.0;

// A setting of a way, and its slider.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Setting {
    // Its name in the state, and its slider's after the way's, like
    // "Linear/Length".
    pub name: &'static str,
    pub label: &'static str,
    pub minimum: f32,
    pub maximum: f32,
    pub step: f32,
    pub default: f32,
    // Decimals of the value that the slider shows, and what follows it.
    pub decimals: usize,
    pub suffix: &'static str,
}

impl Setting {
    // A length in whole pixels, from 0.
    pub const fn pixels(name: &'static str, label: &'static str, maximum: f32, default: f32) -> Self {
        Self::signed_pixels(name, label, 0.0, maximum, default)
    }

    // A length in whole pixels, from a minimum that may be less than 0.
    pub const fn signed_pixels(name: &'static str, label: &'static str, minimum: f32, maximum: f32, default: f32) -> Self {
        Self {
            name,
            label,
            minimum,
            maximum,
            step: 1.0,
            default,
            decimals: 0,
            suffix: " px",
        }
    }

    // An opacity, from 0 to 1, in hundredths.
    pub const fn opacity(name: &'static str, label: &'static str, default: f32) -> Self {
        Self {
            name,
            label,
            minimum: 0.0,
            maximum: 1.0,
            step: 0.01,
            default,
            decimals: 2,
            suffix: "",
        }
    }
}

impl Kind {
    // Its card's title.
    pub fn title(self) -> &'static str {
        match self {
            Kind::RoundGaussian => "Round Gaussian",
            Kind::BSpline => "B-spline blur",
            Kind::Layered => "Layered Gaussians",
            _ => self.name(),
        }
    }

    // Its name in the state, and before its sliders' names.
    pub fn name(self) -> &'static str {
        match self {
            Kind::Linear => "Linear",
            Kind::Curved => "Curved",
            Kind::Smoothstep => "Smoothstep",
            Kind::Exponential => "Exponential",
            Kind::Bell => "Bell",
            Kind::Gaussian => "Gaussian",
            Kind::RoundGaussian => "RoundGaussian",
            Kind::BSpline => "BSpline",
            Kind::Layered => "Layered",
        }
    }

    // What its card says it draws.
    pub fn description(self) -> &'static str {
        match self {
            Kind::Linear => linear::DESCRIPTION,
            Kind::Curved => curved::DESCRIPTION,
            Kind::Smoothstep => smoothstep::DESCRIPTION,
            Kind::Exponential => exponential::DESCRIPTION,
            Kind::Bell => bell::DESCRIPTION,
            Kind::Gaussian => gaussian::DESCRIPTION,
            Kind::RoundGaussian => round_gaussian::DESCRIPTION,
            Kind::BSpline => b_spline::DESCRIPTION,
            Kind::Layered => layered::DESCRIPTION,
        }
    }

    // The Gaussian's variants have its settings, so that they compare at the
    // same ones.
    pub fn settings(self) -> &'static [Setting] {
        match self {
            Kind::Linear => linear::SETTINGS,
            Kind::Curved => curved::SETTINGS,
            Kind::Smoothstep => smoothstep::SETTINGS,
            Kind::Exponential => exponential::SETTINGS,
            Kind::Bell => bell::SETTINGS,
            Kind::Gaussian | Kind::RoundGaussian | Kind::BSpline => gaussian::SETTINGS,
            Kind::Layered => layered::SETTINGS,
        }
    }

    // Its compiled vertex and pixel shaders.
    pub fn shaders(self) -> (&'static [u8], &'static [u8]) {
        match self {
            Kind::Linear => (linear::VERTEX_SHADER, linear::PIXEL_SHADER),
            Kind::Curved => (curved::VERTEX_SHADER, curved::PIXEL_SHADER),
            Kind::Smoothstep => (smoothstep::VERTEX_SHADER, smoothstep::PIXEL_SHADER),
            Kind::Exponential => (exponential::VERTEX_SHADER, exponential::PIXEL_SHADER),
            Kind::Bell => (bell::VERTEX_SHADER, bell::PIXEL_SHADER),
            Kind::Gaussian => (gaussian::VERTEX_SHADER, gaussian::PIXEL_SHADER),
            Kind::RoundGaussian => (round_gaussian::VERTEX_SHADER, round_gaussian::PIXEL_SHADER),
            Kind::BSpline => (b_spline::VERTEX_SHADER, b_spline::PIXEL_SHADER),
            Kind::Layered => (layered::VERTEX_SHADER, layered::PIXEL_SHADER),
        }
    }

    // The settings as its shader takes them.
    pub fn params(self, values: &[f32]) -> [f32; 8] {
        match self {
            Kind::Linear => linear::params(values),
            Kind::Curved => curved::params(values),
            Kind::Smoothstep => smoothstep::params(values),
            Kind::Exponential => exponential::params(values),
            Kind::Bell => bell::params(values),
            Kind::Gaussian | Kind::RoundGaussian | Kind::BSpline => gaussian::params(values),
            Kind::Layered => layered::params(values),
        }
    }

    // Where its quad goes, around the box and all of the shadow that it can
    // draw at these settings.
    pub fn area(self, values: &[f32], rect: Rect) -> Rect {
        match self {
            Kind::Linear => linear::area(values, rect),
            Kind::Curved => curved::area(values, rect),
            Kind::Smoothstep => smoothstep::area(values, rect),
            Kind::Exponential => exponential::area(values, rect),
            Kind::Bell => bell::area(values, rect),
            Kind::Gaussian => gaussian::area(values, rect),
            Kind::RoundGaussian => round_gaussian::area(values, rect),
            Kind::BSpline => b_spline::area(values, rect),
            Kind::Layered => layered::area(values, rect),
        }
    }

    // Its shadow's opacity at a point, as its shader works it out.
    pub fn opacity(self, values: &[f32], rect: Rect, x: f32, y: f32) -> f32 {
        match self {
            Kind::Linear => linear::opacity(values, rect, x, y),
            Kind::Curved => curved::opacity(values, rect, x, y),
            Kind::Smoothstep => smoothstep::opacity(values, rect, x, y),
            Kind::Exponential => exponential::opacity(values, rect, x, y),
            Kind::Bell => bell::opacity(values, rect, x, y),
            Kind::Gaussian => gaussian::opacity(values, rect, x, y),
            Kind::RoundGaussian => round_gaussian::opacity(values, rect, x, y),
            Kind::BSpline => b_spline::opacity(values, rect, x, y),
            Kind::Layered => layered::opacity(values, rect, x, y),
        }
    }
}

// A way, and its settings' values, in the order of its settings.
#[derive(Clone, Debug, PartialEq)]
pub struct Approach {
    pub kind: Kind,
    pub values: Vec<f32>,
}

impl Approach {
    // The way with its settings at their defaults.
    pub fn new(kind: Kind) -> Self {
        Self {
            kind,
            values: kind.settings().iter().map(|setting| setting.default).collect(),
        }
    }

    // A setting's value, by its name.
    pub fn value(&self, name: &str) -> f32 {
        let index = self.kind.settings().iter().position(|setting| setting.name == name);
        self.values[index.unwrap_or_else(|| panic!("{} has no setting {name}", self.kind.name()))]
    }

    // Sets a setting to the value, inside what it allows.
    pub fn set(&mut self, index: usize, value: f64) {
        let setting = self.kind.settings()[index];
        self.values[index] = (value as f32).clamp(setting.minimum, setting.maximum);
    }
}

// How much of a pixel is on the side of an edge where the distance is less
// than 0, as Common.hlsli has it.
pub fn coverage(distance: f32) -> f32 {
    (0.5 - distance).clamp(0.0, 1.0)
}

// The signed distance from a point to the rectangle, as Common.hlsli works
// it out: 0 on its edge, and less than 0 inside it.
pub fn box_distance(rect: Rect, x: f32, y: f32) -> f32 {
    let center = rect.center();
    let qx = (x - center.x).abs() - rect.width / 2.0;
    let qy = (y - center.y).abs() - rect.height / 2.0;
    qx.max(qy).min(0.0) + qx.max(0.0).hypot(qy.max(0.0))
}
