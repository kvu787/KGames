// WinUI's ComboBox: a button that shows the chosen item and a chevron, whose
// list opens over it as a popup, with the chosen item where the combo box's
// text is, as WinUI places it. Items are named "<name>/<index>", labeled with
// their text, and the combo box's own label is the chosen item.
//
// Closed, Up and Down choose the item before or after, and Enter, Space or
// Alt+Down open the list. Open, Up and Down move the highlight, Enter chooses
// it, and Esc or a press outside closes the list.

#pragma once

#include "Base/Numbers.h"
#include "Base/Sum.h"
#include "UserInterface/Core/Interaction.h"
#include "UserInterface/Core/Interface.h"
#include "UserInterface/Core/Tree.h"
#include "UserInterface/Design/Controls.h"
#include "UserInterface/Design/Icons.h"
#include "UserInterface/Design/Tokens.h"
#include "UserInterface/Drawing/Geometry.h"
#include "UserInterface/Drawing/Painter.h"
#include "UserInterface/Input/Devices.h"
#include "UserInterface/Input/Events.h"
#include "UserInterface/Widgets/Widget.h"

#include <algorithm>
#include <cmath>
#include <concepts>
#include <cstddef>
#include <format>
#include <optional>
#include <ranges>
#include <span>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

namespace user_interface {

// What only the combo box's own code uses, here and in ComboBox.cpp.
namespace combo_box_detail {

// Which item the keys highlight while the list is open, and which is at its
// top.
struct ComboState {
    size_t highlighted = 0;
    size_t first = 0;
};

// Opens the list, as the popup named after the combo box.
void open_list(Dispatch<void>& ctx, size_t selected, size_t count, std::string_view name);

// Texts to make a combo box's items of: a vector of strings, an array of
// string views, a span of either, and the like.
template <class Items>
concept Texts =
    std::ranges::sized_range<const Items> && std::ranges::input_range<const Items> && std::convertible_to<std::ranges::range_reference_t<const Items>, std::string_view>;

} // namespace combo_box_detail

template <class M> class ComboBox : public Widget<M> {
public:
    // With the texts of its items, of which it keeps copies.
    template <combo_box_detail::Texts Items> ComboBox(size_t selected, const Items& items) : items(owned(items)), selected(selected) {}
    // With its items listed in place, in double braces, like {{"One", "Two"}}.
    ComboBox(size_t selected, std::span<const std::string_view> items) : items(owned(items)), selected(selected) {}
    ComboBox(const ComboBox&) = delete;
    ComboBox& operator=(const ComboBox&) = delete;
    ComboBox(ComboBox&&) = default;
    ComboBox& operator=(ComboBox&&) = default;

    // What choosing an item sends, with its index.
    template <class F> ComboBox&& on_select(F&& reaction) && {
        on_select_ = react<size_t, M>(std::forward<F>(reaction));
        return std::move(*this);
    }

    // What it shows while no item is selected, like with no items.
    ComboBox&& placeholder(std::string placeholder) && {
        placeholder_ = std::move(placeholder);
        return std::move(*this);
    }

    // How wide a combo box is for its widest item: ComboBoxPadding around
    // it, and the chevron's column.
    template <combo_box_detail::Texts Items> static float width(Ui& ui, const Items& texts) {
        float widest = 0.0f;
        for (const auto& text : texts) {
            widest = std::max(widest, ui.measure(std::string_view(text), tokens::Body).width);
        }
        widest = std::ceil(widest);
        return std::max(controls::ComboBoxPadding.left + widest + controls::ComboBoxChevronColumn, controls::ComboBoxMinWidth);
    }

    // The same, for items listed in place, in double braces.
    static float width(Ui& ui, std::span<const std::string_view> texts) {
        return width<std::span<const std::string_view>>(ui, texts);
    }

    std::string_view kind() const override {
        return "ComboBox";
    }

    std::optional<std::string> label() const override {
        return std::string(chosen().value_or(placeholder_));
    }

    std::optional<Sense> sense() const override {
        return Sense::Navigate;
    }

    void parts(Parts& ctx) const override {
        if (!ctx.popup_open() || items.empty()) {
            return;
        }
        combo_box_detail::ComboState state = ctx.memory<combo_box_detail::ComboState>();
        auto [panel, first_slot] = list(ctx.bounds(), ctx.rect(), state.first);
        std::string list_name = std::format("{}/List", ctx.name());
        ctx.part_on(Layer::Popup, Part::Named{"List"}, list_name, panel, Sense::None, "ComboBoxList", std::nullopt);
        for (size_t row = 0; row < shown_count(); row++) {
            size_t index = state.first + row;
            if (index >= items.size()) {
                break;
            }
            const std::string& item = items[index];
            Rect slot(first_slot.x, first_slot.y + static_cast<float>(row) * controls::ComboBoxItemHeight, first_slot.width, first_slot.height);
            Rect item_rect = slot.pad(controls::ComboBoxItemMargin);
            std::string name = std::format("{}/{}", ctx.name(), index);
            ctx.part_on(Layer::Popup, Part::Index{index}, name, item_rect, Sense::Click.takes_wheel(), "ComboBoxItem", item);
        }
    }

    void interact(const Interaction& interaction, Dispatch<M>& ctx) const override {
        size_t count = items.size();
        Hit hit = interaction.whole();
        std::string_view name = ctx.name();
        bool open = ctx.popup_open();
        // The list can't show nothing, like after its last item went while
        // it was open.
        if (open && count == 0) {
            ctx.close_popup();
            open = false;
        }
        std::optional<size_t> chosen;
        if (hit.activated && count > 0) {
            if (open) {
                ctx.close_popup();
                open = false;
            } else {
                combo_box_detail::open_list(ctx, selected, count, name);
                open = true;
            }
        }
        // Whether this very press opened the list, so it doesn't also
        // choose from it below.
        bool opened_now = false;
        if (interaction.focused && !open && count > 0) {
            const Input& input = ctx.input();
            bool alt_down = input.modifiers().alt && input.key_pressed(Key::Down);
            if (alt_down || input.key_pressed_once(Key::Enter) || input.key_pressed_once(Key::Space)) {
                combo_box_detail::open_list(ctx, selected, count, name);
                open = true;
                opened_now = true;
            } else if (input.key_pressed(Key::Down) && selected + 1 < count) {
                chosen = selected + 1;
            } else if (input.key_pressed(Key::Up) && selected > 0) {
                chosen = selected - 1;
            }
        }
        if (open) {
            combo_box_detail::ComboState state = ctx.template memory<combo_box_detail::ComboState>();
            size_t shown = shown_count();
            state.highlighted = std::min(state.highlighted, count - 1);
            state.first = std::min(state.first, base::saturating_sub(count, shown));
            // The keys move the highlight while the list is the top popup.
            if (ctx.popup_on_top()) {
                const Input& input = ctx.input();
                if (input.key_pressed(Key::Down) && state.highlighted + 1 < count) {
                    state.highlighted += 1;
                }
                if (input.key_pressed(Key::Up) && state.highlighted > 0) {
                    state.highlighted -= 1;
                }
                if (input.key_pressed(Key::Home)) {
                    state.highlighted = 0;
                }
                if (input.key_pressed(Key::End)) {
                    state.highlighted = count - 1;
                }
                bool moved = input.key_pressed(Key::Down) || input.key_pressed(Key::Up) || input.key_pressed(Key::Home) || input.key_pressed(Key::End);
                if (!opened_now && (input.key_pressed_once(Key::Enter) || input.key_pressed_once(Key::Space))) {
                    chosen = state.highlighted;
                }
                if (moved) {
                    ctx.set_focus_visible(true);
                }
            }
            // The wheel over the list scrolls it, and the pointer moves the
            // highlight only as it moves, so that the keys can move it while
            // the pointer rests on the list.
            float wheel = interaction.wheel().y;
            if (wheel != 0.0f) {
                float scrolled = static_cast<float>(state.first) - std::round(wheel);
                state.first = base::saturating_cast<size_t>(std::clamp(scrolled, 0.0f, static_cast<float>(base::saturating_sub(count, shown))));
            }
            if (state.highlighted < state.first) {
                state.first = state.highlighted;
            } else if (state.highlighted >= state.first + shown) {
                state.first = state.highlighted + 1 - shown;
            }
            for (const auto& [part, item_hit] : interaction.hits) {
                const Part::Index* indexed = base::as<Part::Index>(part);
                if (indexed == nullptr) {
                    continue;
                }
                if (item_hit.hovered && ctx.input().pointer_delta != Point::Zero) {
                    state.highlighted = indexed->index;
                }
                if (item_hit.activated) {
                    chosen = indexed->index;
                }
            }
            ctx.template memory<combo_box_detail::ComboState>() = state;
            if (chosen) {
                ctx.close_popup();
            }
        }
        if (chosen && *chosen != selected && on_select_) {
            ctx.send(on_select_(*chosen));
        }
    }

    void draw(Draw& ctx) const override {
        Rect rect = ctx.rect();
        const Look& look = ctx.look();
        ControlState state = look.state();
        bool placeholder = !chosen();
        std::string text(chosen().value_or(placeholder_));
        bool keyboard_focus = look.focused && look.focus_visible;
        const Theme& theme = ctx.theme();
        StateColors background(theme.control_fill, theme.control_fill_secondary, theme.control_fill_tertiary, theme.control_fill_disabled);
        // ComboBoxForeground and ComboBoxPlaceHolderForeground.
        Color foreground = (placeholder ? StateColors(theme.text_secondary, theme.text_secondary, theme.text_tertiary, theme.text_disabled)
                                        : StateColors(theme.text_primary, theme.text_primary, theme.text_secondary, theme.text_disabled))
                               .get(state);
        Color chevron = state == ControlState::Disabled ? theme.text_disabled : theme.text_secondary;
        {
            Painter painter = ctx.painter();
            // A combo box draws its own focus cue instead of the system's: a
            // ring 4 pixels outside it, and the pill at its left.
            if (keyboard_focus) {
                Rect ring = rect.expand(controls::ComboBoxFocusMargin);
                painter.shape(ring, controls::ComboBoxFocusRadius, theme.control_fill, Border(theme.focus_outer, controls::ComboBoxFocusThickness));
            }
            Border border =
                state == ControlState::Normal || state == ControlState::PointerOver ? user_interface::border(Elevation::Control, theme, rect) : Border(theme.control_stroke, 1.0f);
            painter.shape(rect, tokens::RadiusControl, background.get(state), border);
            if (keyboard_focus) {
                Rect pill(rect.x, rect.center().y - controls::SelectionPillLength / 2.0f, controls::SelectionPillThickness, controls::SelectionPillLength);
                painter.rounded(pill, controls::SelectionPillThickness / 2.0f, theme.accent);
            }
            Rect text_rect(rect.x + controls::ComboBoxPadding.left, rect.y, rect.width - controls::ComboBoxPadding.left - controls::ComboBoxChevronColumn, rect.height);
            painter.text_fit(text_rect, text, tokens::Body, foreground, Align::Start);
            painter.icon(Point(rect.right() - controls::ComboBoxChevronMargin - controls::ComboBoxChevronSize / 2.0f, rect.center().y), Icon::ChevronDown,
                         controls::ComboBoxChevronSize, chevron);
        }

        if (!ctx.popup_open() || items.empty()) {
            return;
        }
        combo_box_detail::ComboState combo_state = ctx.memory<combo_box_detail::ComboState>();
        auto [panel, first_slot] = list(ctx.bounds(), rect, combo_state.first);
        bool focus_visible = ctx.focus_visible();
        Rect bounds = ctx.bounds();
        Painter painter = ctx.painter();
        Layer layer = painter.layer();
        painter.set_layer(Layer::Popup);
        painter.push_clip(bounds);
        painter.shadow(panel, tokens::RadiusOverlay, tokens::ElevationFlyout);
        painter.shape(panel, tokens::RadiusOverlay, theme.acrylic, Border(theme.surface_stroke_flyout, 1.0f));
        painter.push_clip(panel);
        for (size_t row = 0; row < shown_count(); row++) {
            size_t index = combo_state.first + row;
            if (index >= items.size()) {
                break;
            }
            const std::string& item = items[index];
            Rect slot(first_slot.x, first_slot.y + static_cast<float>(row) * controls::ComboBoxItemHeight, first_slot.width, first_slot.height);
            Rect item_rect = slot.pad(controls::ComboBoxItemMargin);
            Part part = Part::Index{index};
            ControlState item_state = look.part_state(part);
            // ComboBoxItemBackground, selected or not.
            Color fill;
            if (index != selected) {
                fill = item_state == ControlState::Pressed       ? theme.subtle_fill_tertiary
                       : item_state == ControlState::PointerOver ? theme.subtle_fill_secondary
                                                                 : theme.subtle_fill_transparent;
            } else {
                fill = item_state == ControlState::PointerOver ? theme.subtle_fill_tertiary : theme.subtle_fill_secondary;
            }
            float pill_length = item_state == ControlState::Pressed ? controls::SelectionPillPressedLength : controls::SelectionPillLength;
            Color text_color = item_state == ControlState::Pressed ? theme.text_secondary : theme.text_primary;
            painter.rounded(item_rect, controls::ComboBoxItemRadius, fill);
            if (index == selected) {
                Rect pill(item_rect.x, item_rect.center().y - pill_length / 2.0f, controls::SelectionPillThickness, pill_length);
                painter.rounded(pill, controls::SelectionPillThickness / 2.0f, theme.accent);
            }
            Rect text_rect(item_rect.x + controls::ComboBoxItemPadding.left, item_rect.y, item_rect.width - controls::ComboBoxItemPadding.horizontal(), item_rect.height);
            painter.text_fit(text_rect, item, tokens::Body, text_color, Align::Start);
            if (index == combo_state.highlighted && focus_visible && !look.hovered(part)) {
                focus_visual(painter, theme, item_rect, controls::ComboBoxItemRadius, controls::FocusVisualMargin);
            }
        }
        painter.pop_clip();
        painter.pop_clip();
        painter.set_layer(layer);
    }

private:
    // Copies of the texts, as the combo box's items.
    template <class Items> static std::vector<std::string> owned(const Items& texts) {
        std::vector<std::string> made;
        made.reserve(std::ranges::size(texts));
        for (const auto& text : texts) {
            made.emplace_back(std::string_view(text));
        }
        return made;
    }

    std::optional<std::string_view> chosen() const {
        if (selected < items.size()) {
            return std::string_view(items[selected]);
        }
        return std::nullopt;
    }

    // The list's panel and its first item's slot, with the chosen item over
    // the combo box's text and the rest around it, as WinUI opens it, inside
    // the window.
    std::pair<Rect, Rect> list(Rect bounds, Rect rect, size_t first) const {
        size_t count = items.size();
        size_t shown = std::min(count, controls::ComboBoxPopupMaxItems);
        float width = std::max(rect.width, controls::ComboBoxPopupMinWidth);
        float height = controls::ComboBoxPopupPadding.vertical() + static_cast<float>(shown) * controls::ComboBoxItemHeight;
        float offset = static_cast<float>(std::min(base::saturating_sub(selected, first), base::saturating_sub(shown, size_t{1})));
        float y = rect.y - controls::ComboBoxPopupPadding.top - offset * controls::ComboBoxItemHeight;
        y = std::clamp(y, bounds.y, std::max(bounds.bottom() - height, bounds.y));
        Rect panel(rect.x, std::round(y), width, height);
        Rect first_slot(panel.x, panel.y + controls::ComboBoxPopupPadding.top, panel.width, controls::ComboBoxItemHeight);
        return {panel, first_slot};
    }

    size_t shown_count() const {
        return std::min(items.size(), controls::ComboBoxPopupMaxItems);
    }

    std::vector<std::string> items;
    size_t selected;
    std::string placeholder_;
    Reaction<size_t, M> on_select_;
};

} // namespace user_interface
