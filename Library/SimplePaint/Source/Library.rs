// SimplePaint colors 3D models in a way that looks good from any angle. It's
// an unlit, opaque, orthographic-only surface shader that paints a surface
// from its normal relative to the camera. Specification.md defines it, and
// Usage.md explains its controls and how to use it.
//
// - material: the parameters, their validation, and the GPU constants.
// - geometry: validation of meshes before they're uploaded.
// - orthographic: validated transforms for the shader's vertex stage.
// - direct3d12: the shader's pipeline and meshes on the GPU, for hosts that
//   draw with Library/Graphics.
// - Shaders: the HLSL, which hosts compile with their own material count.
//
// The facing-lobe and anchored color-curve mathematical construction is
// credited to Kevin Vu.

#[path = "Direct3D12.rs"]
pub mod direct3d12;
#[path = "Geometry.rs"]
mod geometry;
#[path = "Material.rs"]
mod material;
#[path = "Orthographic.rs"]
pub mod orthographic;

pub use geometry::{PaintVertex, Vertex, validate_mesh};
pub use material::{GpuMaterial, INTERIOR_MAXIMUM, MARGIN, Material, Parameters, validate_parameters};

// Why a material, mesh or transform was rejected. SimplePaint never clamps,
// wraps or repairs its inputs instead.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error(pub String);

impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for Error {}

// The folder of SimplePaint.hlsl and SimplePaintCore.hlsli, for compiling
// them at run time, like the tests do. Build scripts get it from
// DEP_SIMPLE_PAINT_SHADERS instead.
pub const SHADER_DIRECTORY: &str = concat!(env!("CARGO_MANIFEST_DIR"), r"\Shaders");
