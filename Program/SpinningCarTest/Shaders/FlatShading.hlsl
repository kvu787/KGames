// Draws the ground and the cube in flat colors, lit like the C++ scene that
// SpinningCarTest is a port of: each face is its material's color times
// 0.28, plus 0.72 times how much it faces the light. The colors are already
// sRGB, so the host draws them to the back buffer as it's stored.
//
// It takes the vertices and constant buffers that SimplePaint.hlsl's adapter
// does, so SimplePaint's Pipeline and Mesh draw with it: an object's
// transforms at b0, and the light and the materials' colors at b1. The root
// signature is embedded in the compiled vertex shader.
#define ROOT_SIGNATURE \
    "RootFlags(ALLOW_INPUT_ASSEMBLER_INPUT_LAYOUT), " \
    "CBV(b0), " \
    "CBV(b1)"

// The ground's and the cube's, as Scene.rs says.
static const uint MaterialCount = 2;
static const float Ambient = 0.28f;
static const float Diffuse = 0.72f;

cbuffer ObjectConstants : register(b0)
{
    // Three columns of affine transforms; clip W is always one.
    float4 worldToClip[3];
    float4 normalToView[3];
};
cbuffer ShadingConstants : register(b1)
{
    // Toward the light, in view space, like the normals.
    float4 light;
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
    // Each face is flat, so its vertices all have its color.
    nointerpolation float3 color : COLOR;
};

[RootSignature(ROOT_SIGNATURE)]
PixelInput VSMain(VertexInput input)
{
    PixelInput output;
    const float4 p = float4(input.position, 1.0f);
    output.position = float4(dot(p, worldToClip[0]), dot(p, worldToClip[1]),
        dot(p, worldToClip[2]), 1.0f);
    const float3 normal = normalize(float3(dot(input.normal, normalToView[0].xyz),
        dot(input.normal, normalToView[1].xyz), dot(input.normal, normalToView[2].xyz)));
    const float brightness = Ambient + Diffuse * saturate(dot(normal, light.xyz));
    output.color = saturate(colors[input.materialIndex].rgb * brightness);
    return output;
}

float4 PSMain(PixelInput input) : SV_TARGET
{
    return float4(input.color, 1.0f);
}
