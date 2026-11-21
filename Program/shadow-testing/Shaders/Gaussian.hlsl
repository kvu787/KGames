// The Gaussian shadow, like CSS's box-shadow: a copy of the box, grown by
// the spread and moved by the offset, blurred by a Gaussian. A rectangle with
// square corners blurs one axis at a time, so its blurred coverage is
// exactly the product of the error function's differences across it in x and
// in y.

#include "Common.hlsli"

#define SIGMA params0.x
#define OFFSET params0.yz
#define SPREAD params0.w
#define OPACITY params1.x

// Abramowitz and Stegun's 7.1.26, within 1.5e-7 of erf everywhere.
float erfApproximation(float x) {
    float t = 1.0 / (1.0 + 0.3275911 * abs(x));
    float polynomial = ((((1.061405429 * t - 1.453152027) * t + 1.421413741) * t - 0.284496736) * t + 0.254829592) * t;
    float y = 1.0 - polynomial * exp(-x * x);
    return x < 0 ? -y : y;
}

float shadowOpacity(float2 p) {
    float4 rect = float4(box.xy + OFFSET - SPREAD, box.zw + OFFSET + SPREAD);
    if (rect.z <= rect.x || rect.w <= rect.y) {
        return 0;
    }
    float scale = 1.0 / (SIGMA * sqrt(2.0));
    float2 low = (p - rect.xy) * scale;
    float2 high = (p - rect.zw) * scale;
    float across = 0.5 * (erfApproximation(low.x) - erfApproximation(high.x));
    float down = 0.5 * (erfApproximation(low.y) - erfApproximation(high.y));
    return OPACITY * across * down;
}

float4 PSMain(Varyings input) : SV_Target {
    return boxOverShadow(input.position.xy, shadowOpacity(input.position.xy));
}
