// The curved shadow: like the linear one, pure black around the box, as far
// as its length, round around the box's corners, but fading to nothing along
// a curve: its opacity times what's left of the length, as a fraction, to
// the power. A power of 1 is a straight line; more fades faster near the box,
// and has no crease where the shadow ends.

#include "Common.hlsli"

#define LENGTH params0.x
#define OPACITY params0.y
#define POWER params0.z

float shadowOpacity(float2 p) {
    if (LENGTH <= 0) {
        return 0;
    }
    float distance = boxDistance(p, box);
    return OPACITY * pow(1.0 - saturate(distance / LENGTH), POWER) * coverage(distance - LENGTH);
}

float4 PSMain(Varyings input) : SV_Target {
    return boxOverShadow(input.position.xy, shadowOpacity(input.position.xy));
}
