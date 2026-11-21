// The ways of drawing a shadow that are being tried, side by side: each is a
// Kind, with a module of its own that has its settings, which sliders set,
// and how it draws, and a shader of its own in Shaders, which includes
// Common.hlsli. Each module also works out its shadow's opacity at a point on
// the CPU, as its shader does, so that tests can check every pixel that the
// GPU draws. Fluent, Library/user-interface's own shadow, is there for
// reference, and the interface draws it.

use user_interface::Rect;

use crate::{curved, fluent, gaussian, linear, long, offset};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    Linear,
    Curved,
    Gaussian,
    Offset,
    Long,
    // Library/user-interface's own, for reference, which the interface draws.
    Fluent,
}

// Every way, in the order the page shows them.
pub const KINDS: [Kind; 6] = [Kind::Linear, Kind::Curved, Kind::Gaussian, Kind::Offset, Kind::Long, Kind::Fluent];

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
        Self::offset(name, label, 0.0, maximum, default)
    }

    // A distance in whole pixels, from a minimum that may be less than 0.
    pub const fn offset(name: &'static str, label: &'static str, minimum: f32, maximum: f32, default: f32) -> Self {
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
    // Its name in the state, and before its sliders' names.
    pub fn name(self) -> &'static str {
        match self {
            Kind::Linear => "Linear",
            Kind::Curved => "Curved",
            Kind::Gaussian => "Gaussian",
            Kind::Offset => "Offset",
            Kind::Long => "Long",
            Kind::Fluent => "Fluent",
        }
    }

    // Its card's title.
    pub fn title(self) -> &'static str {
        match self {
            Kind::Fluent => "Fluent, the library's",
            _ => self.name(),
        }
    }

    // What its card says it draws.
    pub fn description(self) -> &'static str {
        match self {
            Kind::Linear => linear::DESCRIPTION,
            Kind::Curved => curved::DESCRIPTION,
            Kind::Gaussian => gaussian::DESCRIPTION,
            Kind::Offset => offset::DESCRIPTION,
            Kind::Long => long::DESCRIPTION,
            Kind::Fluent => fluent::DESCRIPTION,
        }
    }

    pub fn settings(self) -> &'static [Setting] {
        match self {
            Kind::Linear => linear::SETTINGS,
            Kind::Curved => curved::SETTINGS,
            Kind::Gaussian => gaussian::SETTINGS,
            Kind::Offset => offset::SETTINGS,
            Kind::Long => long::SETTINGS,
            Kind::Fluent => fluent::SETTINGS,
        }
    }

    // Its compiled vertex and pixel shaders, if the program draws it.
    pub fn shaders(self) -> Option<(&'static [u8], &'static [u8])> {
        match self {
            Kind::Linear => Some((linear::VERTEX_SHADER, linear::PIXEL_SHADER)),
            Kind::Curved => Some((curved::VERTEX_SHADER, curved::PIXEL_SHADER)),
            Kind::Gaussian => Some((gaussian::VERTEX_SHADER, gaussian::PIXEL_SHADER)),
            Kind::Offset => Some((offset::VERTEX_SHADER, offset::PIXEL_SHADER)),
            Kind::Long => Some((long::VERTEX_SHADER, long::PIXEL_SHADER)),
            Kind::Fluent => None,
        }
    }

    // The settings as its shader takes them.
    pub fn params(self, values: &[f32]) -> [f32; 8] {
        match self {
            Kind::Linear => linear::params(values),
            Kind::Curved => curved::params(values),
            Kind::Gaussian => gaussian::params(values),
            Kind::Offset => offset::params(values),
            Kind::Long => long::params(values),
            Kind::Fluent => [0.0; 8],
        }
    }

    // Where its quad goes, around the box and all of the shadow that it can
    // draw at these settings.
    pub fn area(self, values: &[f32], rect: Rect) -> Rect {
        match self {
            Kind::Linear => linear::area(values, rect),
            Kind::Curved => curved::area(values, rect),
            Kind::Gaussian => gaussian::area(values, rect),
            Kind::Offset => offset::area(values, rect),
            Kind::Long => long::area(values, rect),
            Kind::Fluent => rect,
        }
    }

    // Its shadow's opacity at a point, as its shader works it out, if the
    // program draws it.
    pub fn opacity(self, values: &[f32], rect: Rect, x: f32, y: f32) -> Option<f32> {
        match self {
            Kind::Linear => Some(linear::opacity(values, rect, x, y)),
            Kind::Curved => Some(curved::opacity(values, rect, x, y)),
            Kind::Gaussian => Some(gaussian::opacity(values, rect, x, y)),
            Kind::Offset => Some(offset::opacity(values, rect, x, y)),
            Kind::Long => Some(long::opacity(values, rect, x, y)),
            Kind::Fluent => None,
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
