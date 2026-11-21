// The offset shadow: a copy of the box in black, moved by the offset, with
// hard edges, as flat designs have it.

#include "Common.hlsli"

#define OFFSET params0.xy
#define OPACITY params0.z

float shadowOpacity(float2 p) {
    return OPACITY * coverage(boxDistance(p, box + OFFSET.xyxy));
}

float4 PSMain(Varyings input) : SV_Target {
    return boxOverShadow(input.position.xy, shadowOpacity(input.position.xy));
}
