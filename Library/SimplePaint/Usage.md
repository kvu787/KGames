# Using SimplePaint

SimplePaint makes it easy to color 3D models in a way that looks good from any angle.

SimplePaint is an unlit, opaque, orthographic-only surface shader. It colors a surface from its normal relative to the camera. Lights, shadows, distance from the camera, and world position do not enter the paint calculation. Rotating a model changes which surfaces face the camera; moving it sideways does not change its paint.

This guide describes the module in this folder. [Specification.md](Specification.md) defines its contract; [README.md](README.md) provides a short integration checklist. The supported development and target platform is Windows 11 x64.

## Controls

| Parameter   | Abstract domain  | Meaning                                                     |
| ----------- | ---------------- | ----------------------------------------------------------- |
| R, G, B     | Each in (0, 1)   | Base color in user-facing sRGB space                        |
| Brightness  | (0, 1)           | Positions the base-color anchor on the tone curve           |
| Shift       | [0, 1)           | Warps the facing lobe in the direction selected by Rotation |
| Rotation    | [0, 360) degrees | Circular orientation of the shift                           |
| Dark Point  | [0, 1)           | Tone selected when the warped facing value is zero          |
| Light Point | (0, 1]           | Tone selected when the warped facing value is one           |

Parentheses exclude an endpoint; brackets include it. These are the mathematical domains. This binary32 GPU implementation accepts the following numerical subset, with `m = 1/1024 = 0.0009765625` and `M = 1-m = 0.9990234375`:

| Input              | Accepted finite Rust value |
| ------------------ | -------------------------- |
| Each sRGB channel  | [m, M]                     |
| Brightness         | [m, M]                     |
| Shift              | [0, M]                     |
| Rotation (degrees) | [0, 360)                   |
| Dark Point         | [0, M]                     |
| Light Point        | [m, 1]                     |

The implementation **returns an error for out-of-range values, NaN, and infinity**. It does not clamp inputs, wrap rotation, or silently repair a material. Values inside the abstract domains but outside these numerical limits are also errors. GPU constants are rounded to binary32 after validation and precomputation in binary64. The [specification](Specification.md) explains the accuracy tradeoff.

Start with Shift = 0, Dark Point = 0, and Light Point = 1. The base color appears at **tone = 1 - Brightness**. Increasing Brightness moves that anchor toward the dark end, so more of the surface becomes bright. Brightness is not a multiplier on the RGB values.

Shift = 0 gives symmetric facing shading and makes Rotation irrelevant. Increasing Shift moves the maximum toward a sideways-facing normal. Rotation = 0 shifts toward screen right; 90 toward screen up; 180 toward screen left; 270 toward screen down. This convention uses view axes +X right, +Y up, +Z toward the camera. Rotation is counterclockwise as viewed on screen.

Dark Point and Light Point choose **tones**, not RGB intensities. Their resulting colors depend on the base color and Brightness. Dark Point may exceed Light Point, giving an inverted ramp. Equal endpoints give a constant color. Exact black and white are available as the tone-curve endpoints even though the base-color channels exclude 0 and 1.

There is no positive facing cutoff. Back-facing normals and exact silhouette normals select Dark Point. A front-facing surface approaches that tone continuously as its facing approaches zero. The shader does not flip back-face normals; an application may choose to cull back faces.

## Use from a Rust package

Depend on the crate with a path from your package's folder, like a program in `Program`:

```toml
[dependencies]
simple_paint = { path = "../../Library/SimplePaint" }
```

- `Source/Material.rs`: validated parameters and GPU constant compilation, with no DirectX types.
- `Source/Geometry.rs`: validation for indexed triangle meshes.
- `Source/Orthographic.rs`: the validated transforms that the adapter's vertex stage reads.
- `Source/Direct3D12.rs`: the adapter's pipeline and meshes in GPU memory, for hosts that draw
  with [Library/Graphics](../Graphics/README.md).
- `Shaders/SimplePaintCore.hlsli`: the binding-independent HLSL functions.
- `Shaders/SimplePaint.hlsl`: the configurable DX12 vertex and pixel shader adapter.

Compile each material once when its settings change:

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
let gpu = *material.constants(); // Copy it to an upload buffer.
```

Handle the `Error` at your UI or loading boundary and display its message. Keep the last valid material if an edit fails. Do not change GPU constants after compilation.

Each material occupies **80 bytes**, aligned to 16 bytes: five consecutive HLSL `float4` registers. A material array has an 80-byte stride, not a 256-byte stride. Align the start of the containing DX12 constant buffer to 256 bytes; round its allocation/CBV size up to a multiple of 256, as `direct3d12::materials_size` does. Six materials use 480 bytes in a 512-byte allocation. Respect GPU fence ownership when updating an in-flight buffer.

In your vertex shader, transform the normal to the view frame and call `SimplePaintRotateNormal`. Pass its result using `noperspective`, and pass the material index using `nointerpolation`. Call `SimplePaintShade` in the pixel shader:

```hlsl
#include "SimplePaintCore.hlsli"

// In VS, after constructing your view-space normal:
// output.paintNormal = SimplePaintRotateNormal(viewNormal, material);
// In PS, after fetching the same material:
// float3 linearRgb = SimplePaintShade(input.paintNormal, material);
// return float4(linearRgb, 1.0f);
```

The included DX12 adapter uses `b0` for the 96-byte packed object transforms, `b1` for the material array, and vertex attributes POSITION (`float3`), NORMAL (`float3`), MATERIAL (`uint`). Its input layout uses byte offsets 0, 12, and 24, for a 28-byte vertex: `simple_paint::Vertex`. It embeds its root signature, two root CBVs, in the compiled vertex stage, so `direct3d12::Pipeline` creates the root signature from the shader's bytes. Define `SIMPLE_PAINT_MATERIAL_COUNT` when compiling both shader stages and allocate the same number of material records. The shader defaults to one material. The host must check that its material count agrees with the shader count.

Use an orthographic projection with affine object/view transforms and clip W = 1. `orthographic::make_projection` constructs a right-handed orthographic projection, and `orthographic::build_object_transforms` constructs the columns consumed by the adapter's vertex stage, from a [Library/Math](../Math/README.md) matrix. The helper rejects perspective, shear, nonuniform scale, singular scale, nonfinite values, and out-of-range transforms. Its supported uniform scale is [1/1024, 1024]. A different normal-transform adapter is your project's responsibility; the provided adapter only supports those validated transforms.

Before uploading geometry, call `validate_mesh(vertices, indices, material_count)`. It accepts slices of vertices and 32-bit indices. Vertices implement `PaintVertex`, which gives their position, normal, and material index; `simple_paint::Vertex` does. Use normal lengths in [0.5, 2], positions within [-1,000,000, 1,000,000], valid indices, one material per triangle, and the common normal cone described in the specification. The validation returns an error if interpolation could approach a zero normal under that contract. `direct3d12::Mesh::new` validates the mesh before uploading it. The GPU core assumes these checks have succeeded.

Compile the HLSL entry points as `vs_6_0` and `ps_6_0` in your package's `Build.rs`, with DXC `-O3 -Ges -WX`, adding `-D SIMPLE_PAINT_MATERIAL_COUNT=N` for your material count. The GPU tests additionally use `-all_resources_bound -Qstrip_debug -Qstrip_reflect`. The crate's build script tells your build script where the shaders are, in the `DEP_SIMPLE_PAINT_SHADERS` environment variable, and [Library/ShaderCompiler](../ShaderCompiler/README.md) runs DXC:

```rust
let shaders = PathBuf::from(std::env::var("DEP_SIMPLE_PAINT_SHADERS").unwrap());
let source = shaders.join("SimplePaint.hlsl");
// Also covers SimplePaintCore.hlsli, which is in the same folder.
shader_compiler::rerun_if_changed(&source);
let output_directory = PathBuf::from(std::env::var("OUT_DIR").unwrap());
for (entry_point, profile, output) in [
    ("VSMain", "vs_6_0", "SimplePaintVertex.cso"),
    ("PSMain", "ps_6_0", "SimplePaintPixel.cso"),
] {
    let arguments = ["-O3", "-Ges", "-WX", "-D", "SIMPLE_PAINT_MATERIAL_COUNT=3"];
    shader_compiler::compile(&source, entry_point, profile, &arguments, &output_directory.join(output))
        .unwrap_or_else(|error| panic!("{error}"));
}
```

Embed the DXIL with `include_bytes!(concat!(env!("OUT_DIR"), "/SimplePaintVertex.cso"))`. Bind both constant buffers before drawing; `direct3d12::Mesh::draw` does.

The shader returns **linear RGB**, with alpha 1 supplied by the adapter. Encode it to sRGB once for display, for example with an `R8G8B8A8_UNORM_SRGB` render-target view. Do not apply a second gamma function in the pixel shader. For HDR or a later composite pass, retain linear values and let that pipeline perform the final display conversion.

## Build and verify

The tests are in [Test/SimplePaint](../../Test/SimplePaint), apart from the module, like every test in KGames. From the root of the repository:

```powershell
cargo test -p simple_paint_tests
```

They require Windows 11 x64, DXC from the Windows SDK, and Shader Model 6.0 runtimes for the preferred hardware adapter and WARP. They check D3D12 debug-layer warnings and errors when the debug layer, the "Graphics Tools" optional Windows feature, is installed.

The tests include 81,940 curve comparisons, 20,000 comparisons between the specification's slice/Schlick and rational facing equations, parameter, transform and mesh validation checks, and 30,208 production VS/PS samples on each of the preferred adapter and WARP. The GPU tests include 4,096 interpolated-normal cases and check both linear and sRGB error against an independent binary64 evaluation of the specification. [Specification.md](Specification.md#verification) defines the coverage and acceptance criteria.

The `ShaderCompilation` test compiles both shader stages with the default count of one and an explicit count of three. A zero material count must fail shader compilation.
