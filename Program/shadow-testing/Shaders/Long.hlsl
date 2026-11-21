// The long shadow, as flat designs have it: the box dragged along a
// direction as far as the length. A point's opacity goes in a straight line
// from the start's to the end's by how far along the direction the box had to
// go to cover it. Its edges run along the direction, at any angle, so each
// pixel is the mean of 16 samples in a 4 x 4 grid.

#include "Common.hlsli"

#define LENGTH params0.x
#define DIRECTION params0.yz
#define START params0.w
#define END params1.x

// The opacity at a point: from the least distance along the direction, from
// 0 to the length, at which the box, moved that far, has the point in it.
float sampleOpacity(float2 p) {
    float enter = 0;
    float leave = LENGTH;
    for (int axis = 0; axis < 2; axis++) {
        float position = p[axis];
        float direction = DIRECTION[axis];
        float low = box[axis];
        float high = box[axis + 2];
        if (abs(direction) < 1e-6) {
            if (position < low || position > high) {
                return 0;
            }
        } else {
            float first = (position - high) / direction;
            float second = (position - low) / direction;
            enter = max(enter, min(first, second));
            leave = min(leave, max(first, second));
        }
    }
    if (enter > leave) {
        return 0;
    }
    return lerp(START, END, enter / LENGTH);
}

float shadowOpacity(float2 p) {
    if (LENGTH <= 0) {
        return 0;
    }
    float sum = 0;
    for (int row = 0; row < 4; row++) {
        for (int column = 0; column < 4; column++) {
            sum += sampleOpacity(p + (float2(column, row) + 0.5) / 4.0 - 0.5);
        }
    }
    return sum / 16.0;
}

float4 PSMain(Varyings input) : SV_Target {
    return boxOverShadow(input.position.xy, shadowOpacity(input.position.xy));
}
