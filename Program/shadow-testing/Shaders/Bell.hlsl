// The bell shadow: pure black around the box, round around its corners,
// whose opacity falls along a bell curve, a Gaussian of each pixel's
// distance from the box, whose standard deviation is the width: flat where
// it leaves the box, then fading faster and faster. The quad ends where it's
// too faint to change a pixel.

#include "Common.hlsli"

#define WIDTH params0.x
#define OPACITY params0.y

float shadowOpacity(float2 p) {
    float distance = max(boxDistance(p, box), 0);
    return OPACITY * exp(-distance * distance / (2.0 * WIDTH * WIDTH));
}

float4 PSMain(Varyings input) : SV_Target {
    return boxOverShadow(input.position.xy, shadowOpacity(input.position.xy));
}
