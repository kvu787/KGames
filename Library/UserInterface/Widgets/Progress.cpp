#include "UserInterface/Widgets/Progress.h"

#include "UserInterface/Design/Theme.h"
#include "UserInterface/Drawing/Geometry.h"
#include "UserInterface/Drawing/Painter.h"

#include <algorithm>
#include <cmath>
#include <numbers>
#include <utility>

namespace user_interface {

namespace {

// A full turn, and what Rust's f32::to_radians multiplies by: each a float
// that's rounded once.
constexpr float Tau = 2.0f * std::numbers::pi_v<float>;
constexpr float RadiansPerDegree = std::numbers::pi_v<float> / 180.0f;

} // namespace

ProgressBar::ProgressBar(std::optional<float> value) : value(value) {}

std::string_view ProgressBar::kind() const {
    return "ProgressBar";
}

void ProgressBar::draw(Draw& ctx) const {
    Rect rect = ctx.rect();
    double time = ctx.time();
    Painter painter = ctx.painter();
    const Theme& theme = painter.theme();
    float middle = rect.center().y;
    Rect bar(rect.x, std::round(middle - controls::ProgressBarHeight / 2.0f), rect.width, controls::ProgressBarHeight);
    if (value) {
        Rect track(rect.x, std::round(middle - controls::ProgressBarTrackHeight / 2.0f), rect.width, controls::ProgressBarTrackHeight);
        painter.rounded(track, controls::ProgressBarTrackRadius, theme.control_strong_stroke);
        float width = rect.width * std::clamp(*value, 0.0f, 1.0f);
        if (width > 0.0f) {
            painter.rounded(Rect(bar.x, bar.y, width, bar.height), controls::ProgressBarRadius, theme.accent);
        }
    } else {
        // Two indicators chase each other across, every 2 seconds:
        // one 0.4 wide from -0.4 to 1.2 over 1.5 seconds, and one 0.6
        // wide from -0.9 to 0.996, from 0.75 seconds to 2.
        float cycle = static_cast<float>(std::fmod(time, 2.0));
        float width = rect.width;
        float first = -0.4f + 1.6f * controls::ProgressBarCurve.ease(std::min(cycle / 1.5f, 1.0f));
        float second = cycle < 0.75f ? -0.9f : -0.9f + 1.896f * controls::ProgressBarCurve.ease((cycle - 0.75f) / 1.25f);
        painter.push_clip(Rect(rect.x, bar.y, rect.width, bar.height));
        for (auto [offset, length] : {std::pair(first, 0.4f), std::pair(second, 0.6f)}) {
            Rect indicator(rect.x + width * offset, bar.y, width * length, bar.height);
            painter.rounded(indicator, controls::ProgressBarRadius, theme.accent);
        }
        painter.pop_clip();
    }
}

ProgressRing::ProgressRing(std::optional<float> value) : value(value) {}

ProgressRing&& ProgressRing::size(float size) && {
    size_ = size;
    return std::move(*this);
}

std::string_view ProgressRing::kind() const {
    return "ProgressRing";
}

void ProgressRing::draw(Draw& ctx) const {
    Point center = ctx.rect().center();
    double time = ctx.time();
    float radius = size_ * controls::ProgressRingRadius;
    float stroke = size_ * controls::ProgressRingStroke;
    Painter painter = ctx.painter();
    const Theme& theme = painter.theme();
    if (value) {
        // Its track is ProgressRingBackgroundThemeBrush, which is
        // transparent; the arc is all that shows.
        painter.arc(center, radius, stroke, 0.0f, Tau * std::clamp(*value, 0.0f, 1.0f), theme.accent);
    } else {
        // Over 2 seconds the ring turns 900 degrees; in the first
        // second its head runs half way round, and in the next its
        // tail catches up.
        float cycle = static_cast<float>(std::fmod(time, 2.0));
        auto [half, t] = cycle < 1.0f ? std::pair(0, cycle) : std::pair(1, cycle - 1.0f);
        float eased = controls::ProgressRingCurve.ease(t);
        float rotation = (450.0f * (static_cast<float>(half) + eased)) * RadiansPerDegree;
        auto [start, end] = half == 0 ? std::pair(0.0f, 0.0001f + 0.4999f * eased) : std::pair(0.5f * eased, 0.5f);
        float tau = Tau;
        painter.arc(center, radius, stroke, rotation + start * tau, std::max(end - start, 0.0001f) * tau, theme.accent);
    }
}

} // namespace user_interface
