// Draws what goes over the mesh in the view: the cage's edges as lines a few
// pixels wide, its vertices as dots, selected faces as tinted triangles, and
// the floor's grid. Each is an instance of a quad that the vertex shader puts
// on the screen from its points in the world, so lines and dots are as wide in
// pixels however far they are. Depth tests against the mesh, pulled a little
// toward the camera, hide what's behind the mesh, unless the host draws them
// with its pipeline that doesn't test depth, for X-ray.
//
// Colors are sRGB and premultiplied, for the frame's view that stores them as
// they are.
#define ROOT_SIGNATURE \
    "RootConstants(num32BitConstants=4, b0), " \
    "CBV(b1), " \
    "SRV(t0)"

#define KIND_LINE 0
#define KIND_POINT 1
#define KIND_TRIANGLE 2

cbuffer Frame : register(b0)
{
    // The view's size in pixels, and how far toward the camera depth moves.
    float2 viewportSize;
    float depthBias;
    float unused;
};
cbuffer View : register(b1)
{
    // From the world to the clip space of the view, as rows: v * M.
    row_major float4x4 worldToClip;
};

struct Item
{
    float3 a;
    float width;
    float3 b;
    uint color;
    float3 c;
    uint kind;
};

StructuredBuffer<Item> items : register(t0);

struct Varyings
{
    float4 position : SV_POSITION;
    float2 local : LOCAL;
    nointerpolation float halfWidth : HALF_WIDTH;
    nointerpolation uint color : COLOR;
    nointerpolation uint kind : KIND;
};

float4 unpack(uint color)
{
    return float4(color & 0xFF, (color >> 8) & 0xFF, (color >> 16) & 0xFF, color >> 24) / 255.0f;
}

float3 toScreen(float3 world)
{
    const float4 clip = mul(float4(world, 1.0f), worldToClip);
    // Orthographic: w is 1. x and y to pixels, y down; z stays.
    return float3((clip.x * 0.5f + 0.5f) * viewportSize.x, (0.5f - clip.y * 0.5f) * viewportSize.y, clip.z);
}

float4 toClip(float3 screen)
{
    return float4(screen.x / viewportSize.x * 2.0f - 1.0f, 1.0f - screen.y / viewportSize.y * 2.0f, saturate(screen.z - depthBias), 1.0f);
}

[RootSignature(ROOT_SIGNATURE)]
Varyings VSMain(uint vertex : SV_VertexID, uint index : SV_InstanceID)
{
    const Item item = items[index];
    const float2 corner = float2(vertex & 1, vertex >> 1);
    Varyings output;
    output.color = item.color;
    output.kind = item.kind;
    output.halfWidth = item.width * 0.5f;
    output.local = 0.0f;
    if (item.kind == KIND_TRIANGLE)
    {
        const float3 screen = toScreen(vertex == 0 ? item.a : (vertex == 1 ? item.b : item.c));
        output.position = toClip(screen);
        return output;
    }
    const float3 a = toScreen(item.a);
    if (item.kind == KIND_POINT)
    {
        const float reach = output.halfWidth + 1.0f;
        const float2 offset = (corner * 2.0f - 1.0f) * reach;
        output.position = toClip(float3(a.xy + offset, a.z));
        output.local = offset;
        return output;
    }
    // A line: a box around it, a pixel wider than it.
    const float3 b = toScreen(item.b);
    const float2 along = b.xy - a.xy;
    const float segment = length(along);
    const float2 direction = segment > 0.0f ? along / segment : float2(1.0f, 0.0f);
    const float2 normal = float2(-direction.y, direction.x);
    const float reach = output.halfWidth + 1.0f;
    const float3 end = corner.x > 0.5f ? b : a;
    const float across = (corner.y * 2.0f - 1.0f) * reach;
    const float extend = (corner.x * 2.0f - 1.0f) * 1.0f;
    output.position = toClip(float3(end.xy + normal * across + direction * extend, end.z));
    output.local = float2(0.0f, across);
    return output;
}

float4 PSMain(Varyings input) : SV_TARGET
{
    const float4 color = unpack(input.color);
    if (input.kind == KIND_TRIANGLE)
    {
        return color;
    }
    float distance;
    if (input.kind == KIND_POINT)
    {
        distance = length(input.local) - input.halfWidth;
    }
    else
    {
        distance = abs(input.local.y) - input.halfWidth;
    }
    return color * saturate(0.5f - distance);
}
