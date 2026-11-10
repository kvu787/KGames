#include "UserInterface/Widgets/Slider.h"

namespace user_interface {

std::pair<float, float> slider_rail(Rect rect) {
    Rect track = rect.bottom_strip(controls::SliderHeight);
    return {track.x + controls::SliderThumbSize / 2.0f, track.right() - controls::SliderThumbSize / 2.0f};
}

namespace slider_detail {

double tidy(double value) {
    return std::round(value * 1e9) / 1e9;
}

} // namespace slider_detail

} // namespace user_interface
