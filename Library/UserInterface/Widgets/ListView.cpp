#include "UserInterface/Widgets/ListView.h"

namespace user_interface {

ListRows::ListRows(size_t first, size_t last, float offset, Rect area, Rect list, float row_height) :
    first(first), last(last), offset(offset), area(area), list(list), row_height(row_height) {}

std::ranges::iota_view<size_t, size_t> ListRows::indices() const {
    return std::views::iota(first, last);
}

Rect ListRows::row(size_t index) const {
    return Rect(area.x, list.y + static_cast<float>(index) * row_height - offset, area.width, row_height);
}

Rect ListRows::content(size_t index) const {
    return row(index).pad(controls::ListItemPadding);
}

} // namespace user_interface
