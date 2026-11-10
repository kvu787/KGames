#include "UserInterface/Widgets/Expander.h"

namespace user_interface {

Rect ExpanderContent::inner(Rect rect) {
    return rect.inset(controls::ExpanderContentPadding);
}

std::string_view ExpanderContent::kind() const {
    return "ExpanderContent";
}

bool ExpanderContent::covers() const {
    return true;
}

void ExpanderContent::draw(Draw& ctx) const {
    Rect rect = ctx.rect();
    Painter painter = ctx.painter();
    const Theme& theme = painter.theme();
    painter.shape(rect, Corners::bottom(tokens::RadiusControl), theme.card_background_secondary, Border::sides(theme.card_stroke, Insets(1.0f, 0.0f, 1.0f, 1.0f)));
}

} // namespace user_interface
