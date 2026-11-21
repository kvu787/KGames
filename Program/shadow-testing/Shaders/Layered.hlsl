// Layered Gaussians, like several of CSS's box-shadows with no offset over
// each other: the widest layer is the box blurred by the blur, and each one
// after it by half the one before, each at the opacity, over each other. The
// narrow layers make a dark core near the box, and the wide ones a long soft
// tail.

#include "Common.hlsli"
#include "Blur.hlsli"

#define LAYERS params0.x
#define SIGMA params0.y
#define OPACITY params0.z

// The most layers that the slider allows.
#define MOST_LAYERS 5

float shadowOpacity(float2 p) {
    // How much of what's under the layers shows through all of them.
    float through = 1.0;
    float sigma = SIGMA;
    for (int layer = 0; layer < MOST_LAYERS; layer++) {
        if (layer < LAYERS) {
            through *= 1.0 - OPACITY * blurredBox(p, box, max(sigma, 0.25));
        }
        sigma *= 0.5;
    }
    return 1.0 - through;
}

float4 PSMain(Varyings input) : SV_Target {
    return boxOverShadow(input.position.xy, shadowOpacity(input.position.xy));
}
