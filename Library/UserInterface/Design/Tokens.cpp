#include "UserInterface/Design/Tokens.h"

// Controls.h has no .cpp of its own, so it's compiled here.
#include "UserInterface/Design/Controls.h"

#include <algorithm>

namespace user_interface::tokens {

float Curve::ease(float t) const {
    float clamped = std::clamp(t, 0.0f, 1.0f);
    auto bezier = [](float a, float b, float s) {
        float u = 1.0f - s;
        return 3.0f * u * u * s * a + 3.0f * u * s * s * b + s * s * s;
    };
    // x(s) = t by bisection, which never diverges, then y(s).
    float low = 0.0f;
    float high = 1.0f;
    for (int step = 0; step < 24; step++) {
        float middle = (low + high) / 2.0f;
        if (bezier(x1, x2, middle) < clamped) {
            low = middle;
        } else {
            high = middle;
        }
    }
    return bezier(y1, y2, (low + high) / 2.0f);
}

float Shadow::sigma() const {
    return std::max(blur / 3.0f, 0.5f);
}

} // namespace user_interface::tokens
