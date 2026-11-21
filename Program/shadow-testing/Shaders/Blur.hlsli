// What the Gaussian ways share: the error function, and a rectangle with
// square corners blurred by a Gaussian. A rectangle blurs one axis at a time,
// so its blurred coverage is exactly the product of the error function's
// differences across it in x and in y.

// Abramowitz and Stegun's 7.1.26, within 1.5e-7 of erf everywhere.
float erfApproximation(float x) {
    float t = 1.0 / (1.0 + 0.3275911 * abs(x));
    float polynomial = ((((1.061405429 * t - 1.453152027) * t + 1.421413741) * t - 0.284496736) * t + 0.254829592) * t;
    float y = 1.0 - polynomial * exp(-x * x);
    return x < 0 ? -y : y;
}

// How much of the point a rectangle, left, top, right and bottom, covers,
// blurred by a Gaussian of the standard deviation.
float blurredBox(float2 p, float4 rect, float sigma) {
    if (rect.z <= rect.x || rect.w <= rect.y) {
        return 0;
    }
    float scale = 1.0 / (sigma * sqrt(2.0));
    float2 low = (p - rect.xy) * scale;
    float2 high = (p - rect.zw) * scale;
    float across = 0.5 * (erfApproximation(low.x) - erfApproximation(high.x));
    float down = 0.5 * (erfApproximation(low.y) - erfApproximation(high.y));
    return across * down;
}
