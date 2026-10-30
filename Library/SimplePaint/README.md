# SimplePaint shader

This folder is the canonical home of SimplePaint, the shader that KGames' programs color 3D
models with. SimplePaint makes it easy to color 3D models in a way that looks good from any
angle: it's an unlit, opaque, orthographic-only surface shader that paints a surface from its
normal relative to the camera.

[Specification.md](Specification.md) defines the mathematics, supported inputs, GPU layout,
and verification criteria. The Rust/HLSL sources implement that contract, and
[Test/SimplePaint](../../Test/SimplePaint) verifies it. Keep the specification, implementation,
and tests aligned when changing the shader. [Usage.md](Usage.md) explains the controls and
integration, and [SimplePaintDemo](../../Program/SimplePaintDemo/README.md) edits a material
live on a sphere.

It's a port of the C++20 module in Veehiicuul's `3dTestScene_CppDx12`, with the same shaders,
except that the DX12 adapter now embeds its root signature.

## Compile a material

```rust
use simple_paint::{Material, Parameters};

let parameters = Parameters {
    base_color_srgb: [0.107, 0.223, 0.578],
    brightness: 0.5,
    shift: 0.6,
    rotation_degrees: 45.0,
    dark_point: 0.05,
    light_point: 0.95,
};
let material = Material::compile(&parameters)?;
let gpu = *material.constants(); // Copy before material is dropped.
```

All values must be finite. With `m = 1/1024` and `M = 1-m`, RGB and Brightness accept [m,M],
Shift and Dark Point accept [0,M], Light Point accepts [m,1], and Rotation accepts [0,360)
degrees. Invalid inputs return an `Error`; no values are clamped or wrapped. Reversed and equal
tone endpoints are valid. Base RGB is sRGB; the shader output is linear RGB. The base color
appears at tone `1-Brightness`. `validate_parameters` checks the material domain without
computing GPU constants. SimplePaint has no JSON dependency or application settings.

## Compile and bind the shaders

`Shaders/SimplePaint.hlsl` contains complete `VSMain` and `PSMain` entry points. A host
compiles them in its `Build.rs`, with its own material count, from the folder that this crate
names in `DEP_SIMPLE_PAINT_SHADERS`. [Usage.md](Usage.md#use-from-a-rust-package) shows how.

- `b0`: `orthographic::ObjectTransforms`, 96 bytes, built with `make_projection` and
  `build_object_transforms`.
- `b1`: a tightly packed array of `GpuMaterial`, 80 bytes per material. Align each buffer's
  start to 256 bytes and round CBV allocation sizes up to 256 bytes; do not add padding between
  array elements.
- The root signature is embedded in the vertex stage: root CBVs at `b0` and `b1`.
- Vertex input: POSITION (`float3`, offset 0), NORMAL (`float3`, offset 12), MATERIAL (`uint`,
  offset 24); stride 28 bytes, as `simple_paint::Vertex`. Every triangle has one material
  index, in range.
- Use orthographic projection, with +X right, +Y up, and +Z toward the camera. Rotation 0
  shifts right, 90 up. Normals interpolate with `noperspective`; the material index uses
  `nointerpolation`.
- Output is opaque linear RGB. Encode to sRGB exactly once for display, such as through
  `Renderer::srgb_render_target` from Library/Graphics. No lights, textures, or samplers are
  required.

`direct3d12::Pipeline` creates the adapter's pipeline for a render target, and
`direct3d12::Mesh` validates a mesh, uploads it, and draws it with its constant buffers.

## Validate geometry and transforms

Call `validate_mesh` before upload, or let `direct3d12::Mesh::new` call it. Positions must be
finite with component magnitudes <= 1e6; normal lengths must be in [0.5,2]. Indices must
describe a nonempty triangle list, with one valid material per triangle. Every triangle normal
must project at least 0.125 onto the normalized sum of its three normals, ensuring
interpolation never reaches zero. Validation fails instead of repairing a mesh.

The transform helper accepts finite orthographic dimensions in [1e-4,1e6],
`0 <= near < far <= 1e6`, and a depth span >= 1e-4. Object/view transforms must be affine,
bounded, and use uniform scale in [1/1024,1024]. Perspective, shear, singular scale, and
nonuniform scale are rejected. Back-facing and exact silhouette normals select Dark Point;
there is no positive facing cutoff.

## Files

| File                            | Purpose                                                        |
| ------------------------------- | -------------------------------------------------------------- |
| `Source/Material.rs`            | Parameters, validation, coefficients, GPU ABI                  |
| `Source/Geometry.rs`            | Host-independent mesh validation, and the adapter's vertex     |
| `Source/Orthographic.rs`        | Validated transforms for the adapter's vertex stage            |
| `Source/Direct3D12.rs`          | The adapter's pipeline, and meshes in GPU memory               |
| `Shaders/SimplePaintCore.hlsli` | Reusable paint math without bindings                           |
| `Shaders/SimplePaint.hlsl`      | Configurable DX12 VS/PS adapter, with its root signature       |
| `Build.rs`                      | Tells dependents' build scripts where the shaders are          |

The facing-lobe and anchored color-curve mathematical construction is credited to Kevin Vu.
