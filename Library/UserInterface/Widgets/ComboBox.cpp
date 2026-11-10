#include "UserInterface/Widgets/ComboBox.h"

namespace user_interface::combo_box_detail {

void open_list(Dispatch<void>& ctx, size_t selected, size_t count, std::string_view name) {
    ctx.open_popup(name);
    ComboState& state = ctx.memory<ComboState>();
    state.highlighted = std::min(selected, base::saturating_sub(count, size_t{1}));
    state.first = std::min(base::saturating_sub(selected, controls::ComboBoxPopupMaxItems / 2), base::saturating_sub(count, controls::ComboBoxPopupMaxItems));
}

} // namespace user_interface::combo_box_detail
