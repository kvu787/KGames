// The linear shadow: a box of one color, and around it a shadow of pure
// black, whose opacity goes in a straight line from the start's at the box's
// edge to the end's at the shadow's length from it, by each pixel's distance
// from the box, so that it's round around the box's square corners. Nothing
// past the length.
//
// One quad, the box grown by the length and a pixel, whose pixel shader
// works out each pixel's distance from the box at the pixel's center. Colors
// are sRGB and premultiplied, and blend in sRGB, as the interface's do.

#define ROOT_SIGNATURE "RootConstants(num32BitConstants=16, b0)"

cbuffer Constants : register(b0) {
    float2 viewport;
    float2 unused;
    // The box's left, top, right and bottom, in the window's pixels.
    float4 box;
    // The box's color, premultiplied.
    float4 color;
    float shadowLength;
    float startOpacity;
    float endOpacity;
    float unused2;
};

struct Varyings {
    float4 position : SV_Position;
};

[RootSignature(ROOT_SIGNATURE)]
Varyings VSMain(uint vertex : SV_VertexID) {
    float2 corner = float2(vertex & 1, vertex >> 1);
    float margin = shadowLength + 1.0;
    float2 position = lerp(box.xy - margin, box.zw + margin, corner);
    Varyings output;
    output.position = float4(position / viewport * float2(2, -2) + float2(-1, 1), 0, 1);
    return output;
}

// The signed distance from a point to the box: 0 on its edge, and less than
// 0 inside it.
float boxDistance(float2 p) {
    float2 center = (box.xy + box.zw) * 0.5;
    float2 halfSize = (box.zw - box.xy) * 0.5;
    float2 q = abs(p - center) - halfSize;
    return min(max(q.x, q.y), 0) + length(max(q, 0));
}

// How much of a pixel is on the side of an edge where the distance is less
// than 0: all of it for a pixel whose center is half a pixel in or more.
float coverage(float distance) {
    return saturate(0.5 - distance);
}

float4 PSMain(Varyings input) : SV_Target {
    // A pixel's center, as SV_Position has it.
    float distance = boxDistance(input.position.xy);
    float inside = coverage(distance);
    float opacity = 0;
    if (shadowLength > 0) {
        opacity = lerp(startOpacity, endOpacity, saturate(distance / shadowLength)) * coverage(distance - shadowLength);
    }
    return color * inside + float4(0, 0, 0, opacity) * (1.0 - inside);
}
