#include "UserInterface/Widgets/Navigation.h"

#include "Base/Numbers.h"

#include <algorithm>
#include <cmath>

namespace user_interface {

float navigation_detail::choice_width(Fonts& fonts, const Choice& choice) {
    float icon = choice.icon ? tokens::Icon + controls::ButtonIconGap : 0.0f;
    return std::ceil(icon + fonts.measure(choice.label, tokens::Body).width);
}

std::optional<size_t> navigation_detail::arrows(const Dispatch<void>& ctx, bool focused, size_t selected, size_t count) {
    if (!focused || count == 0) {
        return std::nullopt;
    }
    const Input& input = ctx.input();
    size_t moved = selected;
    if (input.key_pressed(Key::Right) || input.key_pressed(Key::Down)) {
        moved = std::min(moved + 1, count - 1);
    }
    if (input.key_pressed(Key::Left) || input.key_pressed(Key::Up)) {
        moved = base::saturating_sub<size_t>(moved, 1);
    }
    if (input.key_pressed(Key::Home)) {
        moved = 0;
    }
    if (input.key_pressed(Key::End)) {
        moved = count - 1;
    }
    if (moved != selected) {
        return moved;
    }
    return std::nullopt;
}

std::optional<size_t> navigation_detail::items_input(const Interaction& interaction, Dispatch<void>& ctx, std::span<const Choice> items, size_t selected) {
    std::optional<size_t> chosen;
    for (size_t index = 0; index < items.size(); index++) {
        const Choice& item = items[index];
        Part part = Part::Named{item.label};
        const Hit* hit = interaction.hit(part);
        if (hit == nullptr) {
            continue;
        }
        if (hit->pressed && ctx.focusable()) {
            ctx.focus();
        }
        if (hit->activated && selected != index) {
            chosen = index;
        }
    }
    return chosen;
}

} // namespace user_interface
