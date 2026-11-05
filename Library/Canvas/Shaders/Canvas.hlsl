// Draws the Canvas's instances: one quad each, from four vertices made from
// the vertex's index, around a box, a segment or a glyph. The pixel shader
// works out how much of each pixel the shape covers from its outline, so that
// edges and corners are antialiased exactly, and draws a glyph's coverage
// from the fonts' atlas. Colors come out premultiplied by their alpha.
//
// The root signature is embedded in the compiled vertex shader, and the Rust
// side creates it straight from those bytes.
#define ROOT_SIGNATURE \
    "RootFlags(ALLOW_INPUT_ASSEMBLER_INPUT_LAYOUT), " \
    "RootConstants(num32BitConstants = 2, b0, visibility = SHADER_VISIBILITY_VERTEX), " \
    "DescriptorTable(SRV(t0), visibility = SHADER_VISIBILITY_PIXEL)"

#define BOX 0
#define SEGMENT 1
#define GLYPH 2

cbuffer Constants : register(b0)
{
    // 2 / width and 2 / height of the render target, in pixels.
    float2 pixelToClip;
};

Texture2D<float> atlas : register(t0);

struct Instance
{
    float4 bounds : BOUNDS;
    float4 clip : CLIP;
    float4 data : DATA;
    float4 fill : FILL;
    float4 stroke : STROKE;
    uint kind : KIND;
};

struct VertexOutput
{
    float4 position : SV_Position;
    // A glyph's texel in the atlas.
    float2 texel : TEXCOORD;
    nointerpolation float4 bounds : BOUNDS;
    nointerpolation float4 clip : CLIP;
    nointerpolation float4 data : DATA;
    nointerpolation float4 fill : FILL;
    nointerpolation float4 stroke : STROKE;
    nointerpolation uint kind : KIND;
};

[RootSignature(ROOT_SIGNATURE)]
VertexOutput VSMain(Instance instance, uint vertex : SV_VertexID)
{
    // A triangle strip: top left, top right, bottom left, bottom right.
    float2 corner = float2(vertex & 1, vertex >> 1);
    float2 position;
    float2 texel = 0;
    if (instance.kind == SEGMENT)
    {
        // A quad along the segment, a pixel past its round ends and sides.
        float2 from = instance.bounds.xy;
        float2 to = instance.bounds.zw;
        float2 along = to - from;
        float len = length(along);
        float2 direction = len > 0.0001 ? along / len : float2(1, 0);
        float2 normal = float2(-direction.y, direction.x);
        float reach = instance.data.x * 0.5 + 1.0;
        position = lerp(from - direction * reach, to + direction * reach, corner.x)
            + normal * reach * (corner.y * 2.0 - 1.0);
    }
    else if (instance.kind == GLYPH)
    {
        position = lerp(instance.bounds.xy, instance.bounds.zw, corner);
        texel = lerp(instance.data.xy, instance.data.zw, corner);
    }
    else
    {
        // A box, with room for its antialiased edge, and for its blur.
        float reach = 1.0 + instance.data.z * 3.0;
        position = lerp(instance.bounds.xy - reach, instance.bounds.zw + reach, corner);
    }

    VertexOutput output;
    // Pixels have y pointing down from the top left. Clip space has y
    // pointing up from the center.
    float2 clip = position * pixelToClip - 1.0;
    output.position = float4(clip.x, -clip.y, 0.0, 1.0);
    output.texel = texel;
    output.bounds = instance.bounds;
    output.clip = instance.clip;
    output.data = instance.data;
    output.fill = instance.fill;
    output.stroke = instance.stroke;
    output.kind = instance.kind;
    return output;
}

// How far a point is outside a box centered on the origin, with corners of
// `radius`. Negative inside.
float roundedBoxDistance(float2 p, float2 halfSize, float radius)
{
    float2 q = abs(p) - halfSize + radius;
    return length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - radius;
}

// The soft shadow of a box with rounded corners, after Evan Wallace's
// closed form: the Gaussian is integrated exactly across, and with four
// samples down.
float gaussian(float x, float sigma)
{
    return exp(-(x * x) / (2.0 * sigma * sigma)) / (2.5066283 * sigma);
}

float2 erf(float2 x)
{
    float2 s = sign(x);
    float2 a = abs(x);
    x = 1.0 + (0.278393 + (0.230389 + 0.078108 * (a * a)) * a) * a;
    x *= x;
    return s - s / (x * x);
}

float shadowAcross(float x, float y, float sigma, float corner, float2 halfSize)
{
    float delta = min(halfSize.y - corner - abs(y), 0.0);
    float curved = halfSize.x - corner + sqrt(max(0.0, corner * corner - delta * delta));
    float2 integral = 0.5 + 0.5 * erf((x + float2(-curved, curved)) * (0.70710678 / sigma));
    return integral.y - integral.x;
}

float roundedBoxShadow(float2 p, float2 halfSize, float sigma, float corner)
{
    float low = p.y - halfSize.y;
    float high = p.y + halfSize.y;
    float start = clamp(-3.0 * sigma, low, high);
    float end = clamp(3.0 * sigma, low, high);
    float step = (end - start) / 4.0;
    float y = start + step * 0.5;
    float value = 0.0;
    for (int i = 0; i < 4; i++)
    {
        value += shadowAcross(p.x, p.y - y, sigma, corner, halfSize) * gaussian(y, sigma) * step;
        y += step;
    }
    return value;
}

float4 premultiply(float4 color)
{
    return float4(color.rgb * color.a, color.a);
}

float4 PSMain(VertexOutput input) : SV_Target
{
    // The pixel's center.
    float2 p = input.position.xy;
    if (p.x < input.clip.x || p.y < input.clip.y || p.x >= input.clip.z || p.y >= input.clip.w)
    {
        discard;
    }

    if (input.kind == GLYPH)
    {
        float coverage = atlas.Load(int3(floor(input.texel), 0));
        // A little more contrast, as Windows gives grayscale text, so that
        // thin strokes don't look washed out.
        coverage = coverage * 1.5 / (1.0 + 0.5 * coverage);
        return premultiply(input.fill) * coverage;
    }

    if (input.kind == SEGMENT)
    {
        float2 from = input.bounds.xy;
        float2 to = input.bounds.zw;
        float2 along = to - from;
        float t = saturate(dot(p - from, along) / max(dot(along, along), 0.0001));
        float distance = length(p - from - along * t) - input.data.x * 0.5;
        return premultiply(input.fill) * saturate(0.5 - distance);
    }

    float2 center = (input.bounds.xy + input.bounds.zw) * 0.5;
    float2 halfSize = (input.bounds.zw - input.bounds.xy) * 0.5;
    float radius = min(input.data.x, min(halfSize.x, halfSize.y));
    float blur = input.data.z;
    if (blur > 0.0)
    {
        return premultiply(input.fill) * roundedBoxShadow(p - center, halfSize, blur, radius);
    }
    float distance = roundedBoxDistance(p - center, halfSize, radius);
    float outside = saturate(0.5 - distance);
    float width = input.data.y;
    if (width <= 0.0)
    {
        return premultiply(input.fill) * outside;
    }
    // The outline is inside the edge, and the fill inside the outline.
    float inside = saturate(0.5 - (distance + width));
    return premultiply(input.stroke) * (outside - inside) + premultiply(input.fill) * inside;
}
