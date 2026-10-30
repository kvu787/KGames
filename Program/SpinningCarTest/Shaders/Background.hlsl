// Draws the baked background, a 32:9 image of the scene's backdrop, ground
// and cube, over the whole window with one triangle. It's center-cropped to
// the window's aspect, so the camera's vertical scale stays the same.
//
// The root signature is embedded in the compiled vertex shader, and the Rust
// side creates it straight from those bytes.
#define ROOT_SIGNATURE \
    "RootFlags(DENY_HULL_SHADER_ROOT_ACCESS | DENY_DOMAIN_SHADER_ROOT_ACCESS | " \
        "DENY_GEOMETRY_SHADER_ROOT_ACCESS), " \
    "DescriptorTable(SRV(t0), visibility = SHADER_VISIBILITY_PIXEL), " \
    "RootConstants(num32BitConstants = 1, b0, visibility = SHADER_VISIBILITY_VERTEX), " \
    "StaticSampler(s0, filter = FILTER_MIN_MAG_MIP_LINEAR, " \
        "addressU = TEXTURE_ADDRESS_CLAMP, addressV = TEXTURE_ADDRESS_CLAMP, " \
        "addressW = TEXTURE_ADDRESS_CLAMP, visibility = SHADER_VISIBILITY_PIXEL)"

cbuffer BackgroundConstants : register(b0)
{
    float horizontalUvScale;
};

Texture2D<float4> backgroundTexture : register(t0);
SamplerState backgroundSampler : register(s0);

struct PixelInput
{
    float4 position : SV_POSITION;
    float2 textureCoordinate : TEXCOORD0;
};

[RootSignature(ROOT_SIGNATURE)]
PixelInput VSMain(const uint vertexId : SV_VERTEXID)
{
    const float2 corner = float2((vertexId << 1u) & 2u, vertexId & 2u);

    PixelInput output;
    output.position = float4(corner.x * 2.0f - 1.0f, 1.0f - corner.y * 2.0f, 0.0f, 1.0f);
    output.textureCoordinate = float2(
        0.5f + (corner.x - 0.5f) * horizontalUvScale,
        corner.y);
    return output;
}

float4 PSMain(const PixelInput input) : SV_TARGET
{
    if (any(input.textureCoordinate < 0.0f) || any(input.textureCoordinate > 1.0f))
    {
        // The baked image's top-left texel is the scene's neutral clear color.
        // Reuse it as a side matte when a window is wider than the 32:9 bake.
        return backgroundTexture.Load(int3(0, 0, 0));
    }
    return backgroundTexture.Sample(backgroundSampler, input.textureCoordinate);
}
