// Draws the mesh flat-lit: each material's color, lit by a light from the
// camera's upper left, as a diffuse term over an ambient one. It takes the
// vertices and the constant buffers that SimplePaint.hlsl's adapter does, so
// SimplePaint's Pipeline draws with it: the object's transforms at b0, and the
// light and the materials' colors at b1. The colors are linear, and the host
// draws to an sRGB view, which encodes them once, as it does SimplePaint's.
#define ROOT_SIGNATURE \
    "RootFlags(ALLOW_INPUT_ASSEMBLER_INPUT_LAYOUT), " \
    "CBV(b0), " \
    "CBV(b1)"

#ifndef MATERIAL_COUNT
#define MATERIAL_COUNT 1
#endif

cbuffer ObjectConstants : register(b0)
{
    // Three columns of affine transforms; clip W is always one.
    float4 worldToClip[3];
    float4 normalToView[3];
};
cbuffer ShadingConstants : register(b1)
{
    // Toward the light, in view space, and how much of the color is ambient.
    float4 light;
    float4 colors[MATERIAL_COUNT];
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
    float3 normal : NORMAL;
    nointerpolation uint materialIndex : MATERIAL;
};

[RootSignature(ROOT_SIGNATURE)]
PixelInput VSMain(VertexInput input)
{
    PixelInput output;
    const float4 p = float4(input.position, 1.0f);
    output.position = float4(dot(p, worldToClip[0]), dot(p, worldToClip[1]), dot(p, worldToClip[2]), 1.0f);
    output.normal = float3(dot(input.normal, normalToView[0].xyz), dot(input.normal, normalToView[1].xyz), dot(input.normal, normalToView[2].xyz));
    output.materialIndex = input.materialIndex;
    return output;
}

float4 PSMain(PixelInput input) : SV_TARGET
{
    const float3 normal = normalize(input.normal);
    const float ambient = light.w;
    const float brightness = ambient + (1.0f - ambient) * saturate(dot(normal, light.xyz));
    return float4(colors[input.materialIndex].rgb * brightness, 1.0f);
}
