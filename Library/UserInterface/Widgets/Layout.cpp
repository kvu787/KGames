#include "UserInterface/Widgets/Layout.h"

namespace user_interface {

Form::Form(Rect area) : area(area), label_width(std::nullopt), gap_(tokens::FormRowGap), used(area.y), fits_(true) {}

Form Form::beside(Rect area, float label_width) {
    Form form(area);
    form.label_width = label_width;
    return form;
}

Form&& Form::gap(float gap) && {
    gap_ = gap;
    return std::move(*this);
}

Rect Form::take(float height) {
    fits_ &= height <= area.height;
    Rect rect = area.cut_top(height, gap_);
    used = rect.bottom();
    return rect;
}

void Form::space(float height) {
    area.cut_top(height, 0.0f);
}

Rect Form::rest() const {
    return area;
}

float Form::bottom() const {
    return used;
}

bool Form::fits() const {
    return fits_;
}

float Form::width() const {
    return area.width;
}

std::string_view Cover::kind() const {
    return "Surface";
}

bool Cover::covers() const {
    return true;
}

std::string_view Group::kind() const {
    return "Group";
}

bool ViewInput::pressing() const {
    return held && held_button == MouseButton::Left;
}

bool ViewInput::holding(MouseButton button) const {
    return held && held_button == button;
}

} // namespace user_interface
