// The B-spline shadow: like the Gaussian, a copy of the box grown by the
// spread, but blurred by a cubic B-spline, which is four box blurs over each
// other, with the same standard deviation. It looks nearly the same, but it
// ends exactly, two knots, about 3.46 deviations, out from the grown box,
// and needs no exponential.

#include "Common.hlsli"

#define SIGMA params0.x
#define SPREAD params0.y
#define OPACITY params0.z

// How much of a cubic B-spline whose knots are 1 apart lies left of t: its
// cumulative distribution, from 0 at -2 to 1 at 2.
float splineBelow(float t) {
    // How much lies between 0 and |t|; the spline is the same either side.
    float a = abs(t);
    float between;
    if (a < 1.0) {
        between = (4.0 * a - 2.0 * a * a * a + 0.75 * a * a * a * a) / 6.0;
    } else if (a < 2.0) {
        float rest = 2.0 - a;
        between = 0.5 - rest * rest * rest * rest / 24.0;
    } else {
        between = 0.5;
    }
    return t < 0 ? 0.5 - between : 0.5 + between;
}

float shadowOpacity(float2 p) {
    float4 rect = float4(box.xy - SPREAD, box.zw + SPREAD);
    if (rect.z <= rect.x || rect.w <= rect.y) {
        return 0;
    }
    // A cubic B-spline's variance is a third of its knot spacing squared.
    float knot = SIGMA * sqrt(3.0);
    float2 low = (p - rect.xy) / knot;
    float2 high = (p - rect.zw) / knot;
    float across = splineBelow(low.x) - splineBelow(high.x);
    float down = splineBelow(low.y) - splineBelow(high.y);
    return OPACITY * across * down;
}

float4 PSMain(Varyings input) : SV_Target {
    return boxOverShadow(input.position.xy, shadowOpacity(input.position.xy));
}
