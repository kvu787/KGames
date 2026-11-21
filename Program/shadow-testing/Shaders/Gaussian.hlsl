// The Gaussian shadow, like CSS's box-shadow with no offset: a copy of the
// box, grown by the spread on every side, blurred by a Gaussian. The way that
// blends in linear light draws with this shader too, through the frame's sRGB
// view.

#include "Common.hlsli"
#include "Blur.hlsli"

#define SIGMA params0.x
#define SPREAD params0.y
#define OPACITY params0.z

float shadowOpacity(float2 p) {
    return OPACITY * blurredBox(p, float4(box.xy - SPREAD, box.zw + SPREAD), SIGMA);
}

float4 PSMain(Varyings input) : SV_Target {
    return boxOverShadow(input.position.xy, shadowOpacity(input.position.xy));
}
