// Draws what's flat on the ground, the terrain and the track: each triangle
// in its material's color, as linear RGB, which the host draws to the back
// buffer's sRGB view, without lights.
//
// It takes the vertices and constant buffers that SimplePaint.hlsl's adapter
// does, so SimplePaint's Pipeline and Mesh draw with it: an object's
// transforms at b0, and the materials' colors at b1. The root signature is
// embedded in the compiled vertex shader.
#define ROOT_SIGNATURE \
    "RootFlags(ALLOW_INPUT_ASSEMBLER_INPUT_LAYOUT), " \
    "CBV(b0), " \
    "CBV(b1)"

// The terrain's palette, then the road, two of barriers and two of the
// start line, as Ground.rs says.
static const uint MaterialCount = 72;

cbuffer ObjectConstants : register(b0)
{
    // Three columns of affine transforms; clip W is always one.
    float4 worldToClip[3];
    float4 normalToView[3];
};
cbuffer Colors : register(b1)
{
    float4 colors[MaterialCount];
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
    nointerpolation float3 color : COLOR;
};

[RootSignature(ROOT_SIGNATURE)]
PixelInput VSMain(VertexInput input)
{
    PixelInput output;
    const float4 p = float4(input.position, 1.0f);
    output.position = float4(dot(p, worldToClip[0]), dot(p, worldToClip[1]),
        dot(p, worldToClip[2]), 1.0f);
    output.color = colors[input.materialIndex].rgb;
    return output;
}

float4 PSMain(PixelInput input) : SV_TARGET
{
    return float4(input.color, 1.0f);
}
