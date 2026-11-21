// The smoothstep shadow: pure black around the box, as far as its length,
// round around the box's corners, fading to nothing along an S curve,
// smoothstep, by each pixel's distance from the box, so that it's soft both
// where it leaves the box and where it ends.

#include "Common.hlsli"

#define LENGTH params0.x
#define OPACITY params0.y

float shadowOpacity(float2 p) {
    if (LENGTH <= 0) {
        return 0;
    }
    float distance = boxDistance(p, box);
    float t = saturate(distance / LENGTH);
    return OPACITY * (1.0 - t * t * (3.0 - 2.0 * t)) * coverage(distance - LENGTH);
}

float4 PSMain(Varyings input) : SV_Target {
    return boxOverShadow(input.position.xy, shadowOpacity(input.position.xy));
}
