// The linear shadow: pure black all around the box, as far as its length,
// whose opacity goes in a straight line from the start's at the box's edge to
// the end's at the length, by each pixel's distance from the box, so that
// it's round around the box's square corners. Nothing past the length.

#include "Common.hlsli"

#define LENGTH params0.x
#define START params0.y
#define END params0.z

float shadowOpacity(float2 p) {
    if (LENGTH <= 0) {
        return 0;
    }
    float distance = boxDistance(p, box);
    return lerp(START, END, saturate(distance / LENGTH)) * coverage(distance - LENGTH);
}

float4 PSMain(Varyings input) : SV_Target {
    return boxOverShadow(input.position.xy, shadowOpacity(input.position.xy));
}
