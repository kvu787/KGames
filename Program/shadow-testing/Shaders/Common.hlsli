// What every way of drawing a shadow shares: the constants, the vertex
// shader, which draws one quad over the area that the way's shadow can reach,
// and the box over its shadow. Each way's .hlsl includes this, then works out
// its shadow's opacity at a point, and its PSMain puts the box over it.
//
// Colors are sRGB and premultiplied, and blend in sRGB, as the interface's
// do. The box's edges are on whole pixels, so every pixel is all box or none
// of it.

#define ROOT_SIGNATURE "RootConstants(num32BitConstants=24, b0)"

cbuffer Constants : register(b0) {
    float2 viewport;
    float2 unused;
    // The box's left, top, right and bottom, in the window's pixels.
    float4 box;
    // The box's color, premultiplied.
    float4 color;
    // Where the quad goes: left, top, right and bottom.
    float4 area;
    // The way's settings, which each way names for itself.
    float4 params0;
    float4 params1;
};

struct Varyings {
    float4 position : SV_Position;
};

[RootSignature(ROOT_SIGNATURE)]
Varyings VSMain(uint vertex : SV_VertexID) {
    float2 corner = float2(vertex & 1, vertex >> 1);
    float2 position = lerp(area.xy, area.zw, corner);
    Varyings output;
    output.position = float4(position / viewport * float2(2, -2) + float2(-1, 1), 0, 1);
    return output;
}

// The signed distance from a point to a rectangle, left, top, right and
// bottom: 0 on its edge, and less than 0 inside it.
float boxDistance(float2 p, float4 rect) {
    float2 center = (rect.xy + rect.zw) * 0.5;
    float2 halfSize = (rect.zw - rect.xy) * 0.5;
    float2 q = abs(p - center) - halfSize;
    return min(max(q.x, q.y), 0) + length(max(q, 0));
}

// How much of a pixel is on the side of an edge where the distance is less
// than 0: all of it for a pixel whose center is half a pixel in or more.
float coverage(float distance) {
    return saturate(0.5 - distance);
}

// The box over a shadow of the opacity, at a pixel's center.
float4 boxOverShadow(float2 p, float shadow) {
    float inside = coverage(boxDistance(p, box));
    return color * inside + float4(0, 0, 0, shadow) * (1.0 - inside);
}
