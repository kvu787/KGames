// Unlit: each triangle in its material's color, as the host gives it. The
// pixel shader returns one value per material, so no light, normal, position
// or view changes a pixel's color. Every 3D model that isn't painted with
// SimplePaint is drawn with this, as the AGENTS.md at the root says.
//
// It takes the vertices and the constant buffers that SimplePaint.hlsl's
// adapter does, so SimplePaint's Pipeline and Mesh draw with it: an object's
// transforms at b0, and the materials' colors at b1, as float4s, 16 bytes
// apart, whose alpha isn't used. The host gives the colors as its view takes
// them: linear for an sRGB view, which encodes them, or sRGB for a view that
// stores them as they are. Define the count when compiling both entry points;
// the host uploads that many colors.
#ifndef UNLIT_COLOR_COUNT
#define UNLIT_COLOR_COUNT 1
#endif
#if UNLIT_COLOR_COUNT < 1
#error UNLIT_COLOR_COUNT must be positive
#endif
static const uint ColorCount = UNLIT_COLOR_COUNT;

// Embedded in the compiled vertex shader, as SimplePaint.hlsl's is.
#define UNLIT_ROOT_SIGNATURE \
    "RootFlags(ALLOW_INPUT_ASSEMBLER_INPUT_LAYOUT), " \
    "CBV(b0), " \
    "CBV(b1)"

cbuffer ObjectConstants : register(b0)
{
    // Three columns of affine transforms; clip W is always one.
    float4 worldToClip[3];
    float4 normalToView[3];
};
cbuffer Colors : register(b1)
{
    float4 colors[ColorCount];
};

struct VertexInput
{
    float3 position : POSITION;
    float3 normal : NORMAL;
    uint materialIndex : MATERIAL;
};
struct PixelInput
{
    float4 position : SV_POSITION;
    nointerpolation uint materialIndex : MATERIAL;
};

[RootSignature(UNLIT_ROOT_SIGNATURE)]
PixelInput VSMain(VertexInput input)
{
    PixelInput output;
    const float4 p = float4(input.position, 1.0f);
    output.position = float4(dot(p, worldToClip[0]), dot(p, worldToClip[1]),
        dot(p, worldToClip[2]), 1.0f);
    output.materialIndex = input.materialIndex;
    return output;
}

float4 PSMain(PixelInput input) : SV_TARGET
{
    return float4(colors[input.materialIndex].rgb, 1.0f);
}
