// The round Gaussian shadow: the Gaussian's edge, by each pixel's distance
// from the box grown by the spread, as many UI renderers draw it. Straight out
// from a side it's the Gaussian's, but around the corners it's round, and as
// dark diagonally out from a corner as straight out from a side.

#include "Common.hlsli"
#include "Blur.hlsli"

#define SIGMA params0.x
#define SPREAD params0.y
#define OPACITY params0.z

float shadowOpacity(float2 p) {
    float distance = boxDistance(p, box) - SPREAD;
    return OPACITY * 0.5 * (1.0 - erfApproximation(distance / (SIGMA * sqrt(2.0))));
}

float4 PSMain(Varyings input) : SV_Target {
    return boxOverShadow(input.position.xy, shadowOpacity(input.position.xy));
}
