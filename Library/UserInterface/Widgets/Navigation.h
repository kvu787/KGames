// Choosing one of a few places or modes: WinUI's SelectorBar, whose selected
// item has the accent's pill under it, and the Windows Community Toolkit's
// Segmented, whose selected segment is raised like a button. Their items are
// targets named "<name>/<label>", and the control is one focus stop whose
// arrows move the selection, as WinUI's are.

#pragma once

#include "UserInterface/Core/Interaction.h"
#include "UserInterface/Core/Interface.h"
#include "UserInterface/Core/Tree.h"
#include "UserInterface/Design/Controls.h"
#include "UserInterface/Design/Icons.h"
#include "UserInterface/Design/Tokens.h"
#include "UserInterface/Drawing/Geometry.h"
#include "UserInterface/Drawing/Painter.h"
#include "UserInterface/Drawing/Text.h"
#include "UserInterface/Input/Devices.h"
#include "UserInterface/Widgets/Widget.h"

#include <algorithm>
#include <cmath>
#include <concepts>
#include <cstddef>
#include <optional>
#include <ranges>
#include <span>
#include <string>
#include <string_view>
#include <tuple>
#include <utility>
#include <vector>

namespace user_interface {

// An item: its label, and maybe an icon before it. A text is a choice,
// wherever choices are wanted, and so is a pair or a tuple of an icon and a
// text.
struct Choice {
    std::string label;
    std::optional<Icon> icon;

    // Not explicit, on purpose: these are the conversions.
    Choice(std::string label) : label(std::move(label)) {}
    Choice(std::string_view label) : label(label) {}
    Choice(const char* label) : label(label) {}
    Choice(Icon icon, std::string_view label) : label(label), icon(icon) {}
    template <std::convertible_to<Icon> IconOf, std::convertible_to<std::string_view> Label>
    Choice(const std::pair<IconOf, Label>& pair) : label(std::string_view(pair.second)), icon(pair.first) {}
    template <std::convertible_to<Icon> IconOf, std::convertible_to<std::string_view> Label>
    Choice(const std::tuple<IconOf, Label>& pair) : label(std::string_view(std::get<1>(pair))), icon(std::get<0>(pair)) {}
    Choice(std::string label, std::optional<Icon> icon) : label(std::move(label)), icon(icon) {}

    bool operator==(const Choice&) const = default;
};

// What only these controls' own code uses, here and in Navigation.cpp.
namespace navigation_detail {

float choice_width(Fonts& fonts, const Choice& choice);

// The arrows move the selection of a control with focus.
std::optional<size_t> arrows(const Dispatch<void>& ctx, bool focused, size_t selected, size_t count);

// The items' input. A press on one gives the control focus, as its one
// stop, unless widgets can't take focus here, and selects it.
std::optional<size_t> items_input(const Interaction& interaction, Dispatch<void>& ctx, std::span<const Choice> items, size_t selected);

// What makes a control's items: any range of what makes a choice.
template <class Items>
concept Choices = std::ranges::input_range<Items> && std::constructible_from<Choice, std::ranges::range_reference_t<Items>>;

} // namespace navigation_detail

template <class M> class SelectorBar : public Widget<M> {
public:
    // The items are any range of what makes a choice, like an array of
    // string views, or a vector of pairs of an icon and a text, or a list of
    // texts in double braces: SelectorBar<Message>(tab, {{"First", "Second"}}).
    template <navigation_detail::Choices Items = std::span<const std::string_view>> SelectorBar(size_t selected, Items&& items) : selected(selected) {
        if constexpr (std::ranges::sized_range<Items>) {
            this->items.reserve(std::ranges::size(items));
        }
        for (auto&& item : items) {
            this->items.push_back(Choice(item));
        }
    }
    SelectorBar(const SelectorBar&) = delete;
    SelectorBar& operator=(const SelectorBar&) = delete;
    SelectorBar(SelectorBar&&) = default;
    SelectorBar& operator=(SelectorBar&&) = default;

    // What selecting an item sends, with its index: `reaction` is called
    // with a size_t, and returns the message.
    template <class F> SelectorBar&& on_select(F&& reaction) && {
        on_select_ = react<size_t, M>(std::forward<F>(reaction));
        return std::move(*this);
    }

    // The width that its items take.
    template <navigation_detail::Choices Items = std::span<const std::string_view>> static float width(Ui& ui, Items&& items) {
        // From -0, as Rust's sum of floats is.
        float total = -0.0f;
        for (auto&& item : items) {
            total += navigation_detail::choice_width(ui.fonts(), Choice(item)) + controls::SelectorBarItemPadding.horizontal() + 2.0f;
        }
        return total;
    }

    std::string_view kind() const override {
        return "SelectorBar";
    }

    std::optional<std::string> label() const override {
        if (selected < items.size()) {
            return items[selected].label;
        }
        return std::nullopt;
    }

    std::optional<Sense> sense() const override {
        return Sense::Navigate;
    }

    void parts(Parts& ctx) const override {
        Rect rect = ctx.rect();
        std::vector<Rect> rects = item_rects(ctx.fonts(), rect);
        for (size_t index = 0; index < items.size(); index++) {
            const Choice& item = items[index];
            ctx.part(Part::Named{item.label}, item.label, rects[index], Sense::Click, "SelectorBarItem", item.label);
        }
    }

    void interact(const Interaction& interaction, Dispatch<M>& ctx) const override {
        std::optional<size_t> chosen = navigation_detail::arrows(ctx, interaction.focused, selected, items.size());
        if (std::optional<size_t> index = navigation_detail::items_input(interaction, ctx, items, selected)) {
            chosen = index;
        }
        if (chosen && on_select_) {
            ctx.send(on_select_(*chosen));
        }
    }

    void draw(Draw& ctx) const override {
        Rect rect = ctx.rect();
        std::vector<Rect> rects = item_rects(ctx.fonts(), rect);
        const Look& look = ctx.look();
        for (size_t index = 0; index < items.size(); index++) {
            const Choice& item = items[index];
            Rect item_rect = rects[index];
            Part part = Part::Named{item.label};
            bool is_selected = selected == index;
            ControlState state = look.part_state(part);
            {
                Painter painter = ctx.painter();
                const Theme& theme = painter.theme();
                // SelectorBarItem's text: secondary with the pointer over it,
                // and tertiary pressed, unless it's selected.
                Color color = theme.text_primary;
                if (state == ControlState::Disabled) {
                    color = theme.text_disabled;
                } else if (!is_selected && state == ControlState::Pressed) {
                    color = theme.text_tertiary;
                } else if (state == ControlState::PointerOver || (is_selected && state == ControlState::Pressed)) {
                    color = theme.text_secondary;
                }
                Color pill_color = state == ControlState::Disabled ? theme.accent_disabled : theme.accent;
                Rect content = item_rect.pad(controls::SelectorBarItemPadding);
                Rect text_line(content.x, content.y, content.width, tokens::Body.line_height);
                float text_x = content.x;
                if (item.icon) {
                    painter.icon(Point(text_x + tokens::Icon / 2.0f, text_line.center().y), *item.icon, tokens::Icon * 0.8f, color);
                    text_x += tokens::Icon + controls::ButtonIconGap;
                }
                painter.text_in(Rect(text_x, text_line.y, content.right() - text_x, text_line.height), item.label, tokens::Body, color, Align::Start);
                if (is_selected) {
                    Rect pill(item_rect.center().x - controls::SelectorBarPillWidth / 2.0f, text_line.bottom() + 3.0f, controls::SelectorBarPillWidth,
                              controls::SelectorBarPillHeight);
                    painter.rounded(pill, 1.0f, pill_color);
                }
            }
            if (is_selected) {
                ctx.focus_visual_around(look.focused, item_rect, tokens::RadiusControl, Insets::all(2.0f));
            }
        }
    }

private:
    // Each item's rectangle, from the bar's left.
    std::vector<Rect> item_rects(Fonts& fonts, Rect rect) const {
        Rect items_area = rect.pad(controls::SelectorBarPadding);
        float x = items_area.x;
        std::vector<Rect> rects;
        rects.reserve(items.size());
        for (const Choice& item : items) {
            float width = navigation_detail::choice_width(fonts, item) + controls::SelectorBarItemPadding.horizontal() + 2.0f;
            Rect item_rect(x, items_area.y, width, items_area.height);
            x += width;
            rects.push_back(item_rect);
        }
        return rects;
    }

    size_t selected;
    std::vector<Choice> items;
    Reaction<size_t, M> on_select_;
};

template <class M> class Segmented : public Widget<M> {
public:
    // The items are any range of what makes a choice, as a SelectorBar's
    // are: Segmented<Message>(alignment, {{"Left", "Center", "Right"}}).
    template <navigation_detail::Choices Items = std::span<const std::string_view>> Segmented(size_t selected, Items&& items) : selected(selected) {
        if constexpr (std::ranges::sized_range<Items>) {
            this->items.reserve(std::ranges::size(items));
        }
        for (auto&& item : items) {
            this->items.push_back(Choice(item));
        }
    }
    Segmented(const Segmented&) = delete;
    Segmented& operator=(const Segmented&) = delete;
    Segmented(Segmented&&) = default;
    Segmented& operator=(Segmented&&) = default;

    // What selecting a segment sends, with its index: `reaction` is called
    // with a size_t, and returns the message.
    template <class F> Segmented&& on_select(F&& reaction) && {
        on_select_ = react<size_t, M>(std::forward<F>(reaction));
        return std::move(*this);
    }

    std::string_view kind() const override {
        return "Segmented";
    }

    std::optional<std::string> label() const override {
        if (selected < items.size()) {
            return items[selected].label;
        }
        return std::nullopt;
    }

    std::optional<Sense> sense() const override {
        return Sense::Navigate;
    }

    void parts(Parts& ctx) const override {
        Rect rect = ctx.rect();
        for (size_t index = 0; index < items.size(); index++) {
            const Choice& item = items[index];
            ctx.part(Part::Named{item.label}, item.label, segment(rect, index), Sense::Click, "SegmentedItem", item.label);
        }
    }

    void interact(const Interaction& interaction, Dispatch<M>& ctx) const override {
        std::optional<size_t> chosen = navigation_detail::arrows(ctx, interaction.focused, selected, items.size());
        if (std::optional<size_t> index = navigation_detail::items_input(interaction, ctx, items, selected)) {
            chosen = index;
        }
        if (chosen && on_select_) {
            ctx.send(on_select_(*chosen));
        }
    }

    void draw(Draw& ctx) const override {
        Rect rect = ctx.rect();
        const Look& look = ctx.look();
        bool enabled = ctx.enabled();
        {
            Painter painter = ctx.painter();
            const Theme& theme = painter.theme();
            painter.shape(rect, tokens::RadiusControl, theme.control_alt_fill_secondary, Border(theme.control_stroke, 1.0f));
            Rect raised = segment(rect, selected);
            Color fill = enabled ? theme.control_fill : theme.control_fill_disabled;
            painter.shape(raised, tokens::RadiusControl - 1.0f, fill, border(Elevation::Control, theme, raised));
        }
        for (size_t index = 0; index < items.size(); index++) {
            const Choice& item = items[index];
            Part part = Part::Named{item.label};
            Rect segment = this->segment(rect, index);
            bool is_selected = selected == index;
            ControlState state = look.part_state(part);
            {
                Painter painter = ctx.painter();
                const Theme& theme = painter.theme();
                Color hover = theme.subtle_fill_transparent;
                if (!is_selected && state == ControlState::PointerOver) {
                    hover = theme.subtle_fill_secondary;
                } else if (!is_selected && state == ControlState::Pressed) {
                    hover = theme.subtle_fill_tertiary;
                }
                Color color = theme.text_primary;
                if (state == ControlState::Disabled) {
                    color = theme.text_disabled;
                } else if (state == ControlState::Pressed) {
                    color = theme.text_secondary;
                }
                Color accent = theme.accent;
                painter.rounded(segment.inset(1.0f), tokens::RadiusControl - 1.0f, hover);
                float width = (item.icon ? tokens::Icon + (item.label.empty() ? 0.0f : controls::ButtonIconGap) : 0.0f) +
                              (item.label.empty() ? 0.0f : painter.measure(item.label, tokens::Body).width);
                float x = std::round(std::max(segment.center().x - width / 2.0f, segment.x + 4.0f));
                if (item.icon) {
                    painter.icon(Point(x + tokens::Icon / 2.0f, segment.center().y), *item.icon, tokens::Icon, color);
                    x += tokens::Icon + controls::ButtonIconGap;
                }
                if (!item.label.empty()) {
                    painter.text_fit(Rect(x, segment.y, segment.right() - x - 4.0f, segment.height), item.label, tokens::Body, color, Align::Start);
                }
                if (is_selected) {
                    Rect pill(segment.center().x - 8.0f, segment.bottom() - 4.0f, 16.0f, 3.0f);
                    painter.rounded(pill, 1.5f, accent);
                }
            }
            if (is_selected) {
                ctx.focus_visual_if(look.focused, segment, tokens::RadiusControl);
            }
        }
    }

private:
    Rect segment(Rect rect, size_t index) const {
        size_t count = std::max<size_t>(items.size(), 1);
        Rect inner = rect.inset(2.0f);
        float segment_width = inner.width / static_cast<float>(count);
        return Rect(inner.x + segment_width * static_cast<float>(index), inner.y, segment_width, inner.height).round();
    }

    size_t selected;
    std::vector<Choice> items;
    Reaction<size_t, M> on_select_;
};

} // namespace user_interface
