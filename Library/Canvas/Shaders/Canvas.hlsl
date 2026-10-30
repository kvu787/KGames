// Draws the Canvas's triangles: pixel positions, a texture coordinate into the
// font atlas, and a color. Shapes sample the atlas's solid white cell, so text
// and shapes go through the same path.
//
// The root signature is embedded in the compiled vertex shader, and the Rust
// side creates it straight from those bytes.
#define ROOT_SIGNATURE \
    "RootFlags(ALLOW_INPUT_ASSEMBLER_INPUT_LAYOUT), " \
    "RootConstants(num32BitConstants = 2, b0, visibility = SHADER_VISIBILITY_VERTEX), " \
    "DescriptorTable(SRV(t0), visibility = SHADER_VISIBILITY_PIXEL), " \
    "StaticSampler(s0, filter = FILTER_MIN_MAG_MIP_POINT, " \
        "addressU = TEXTURE_ADDRESS_CLAMP, addressV = TEXTURE_ADDRESS_CLAMP, " \
        "visibility = SHADER_VISIBILITY_PIXEL)"

cbuffer Constants : register(b0)
{
    // 2 / width and 2 / height of the render target, in pixels.
    float2 pixelToClip;
};

Texture2D<float> atlas : register(t0);
SamplerState atlasSampler : register(s0);

struct VertexInput
{
    float2 position : POSITION;
    float2 uv : TEXCOORD;
    float4 color : COLOR;
};

struct VertexOutput
{
    float4 position : SV_Position;
    float2 uv : TEXCOORD;
    float4 color : COLOR;
};

[RootSignature(ROOT_SIGNATURE)]
VertexOutput VSMain(VertexInput input)
{
    VertexOutput output;
    // Pixels have y pointing down from the top left. Clip space has y pointing up from the center.
    float2 clip = input.position * pixelToClip - 1.0;
    output.position = float4(clip.x, -clip.y, 0.0, 1.0);
    output.uv = input.uv;
    output.color = input.color;
    return output;
}

float4 PSMain(VertexOutput input) : SV_Target
{
    float coverage = atlas.Sample(atlasSampler, input.uv);
    return float4(input.color.rgb, input.color.a * coverage);
}
