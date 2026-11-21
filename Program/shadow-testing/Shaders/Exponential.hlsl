// The exponential shadow: pure black around the box, round around its
// corners, whose opacity falls by a factor of e every decay length out from
// the box: steeply at first, with a long faint tail. The quad ends where the
// tail is too faint to change a pixel.

#include "Common.hlsli"

#define DECAY params0.x
#define OPACITY params0.y

float shadowOpacity(float2 p) {
    float distance = max(boxDistance(p, box), 0);
    return OPACITY * exp(-distance / DECAY);
}

float4 PSMain(Varyings input) : SV_Target {
    return boxOverShadow(input.position.xy, shadowOpacity(input.position.xy));
}
