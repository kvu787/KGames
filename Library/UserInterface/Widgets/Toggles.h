// WinUI's controls that are on or off: CheckBox, RadioButton, and
// ToggleSwitch. Each is a target as wide as its rectangle, so its label takes
// clicks too, and its focus visual goes around the label, as XAML's do.

#pragma once

#include "UserInterface/Core/Interaction.h"
#include "UserInterface/Core/Interface.h"
#include "UserInterface/Core/Tree.h"
#include "UserInterface/Design/Controls.h"
#include "UserInterface/Design/Icons.h"
#include "UserInterface/Design/Theme.h"
#include "UserInterface/Design/Tokens.h"
#include "UserInterface/Drawing/Geometry.h"
#include "UserInterface/Drawing/Painter.h"
#include "UserInterface/Widgets/Widget.h"

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

// What only the toggles' own code uses, here and in Toggles.cpp.
namespace toggles_detail {

// Draws a radio button in the rectangle, in the state, checked or not.
void draw_radio(Painter& painter, const Theme& theme, Rect rect, ControlState state, bool checked, std::string_view label);

} // namespace toggles_detail

// A box that's checked or not, with a label after it.
template <class M> class CheckBox : public Widget<M> {
public:
    CheckBox(bool checked, std::string label) : checked(checked), label_(std::move(label)) {}
    CheckBox(const CheckBox&) = delete;
    CheckBox& operator=(const CheckBox&) = delete;
    CheckBox(CheckBox&&) = default;
    CheckBox& operator=(CheckBox&&) = default;

    // What a click sends, with whether it's checked then: `reaction` is
    // called with a bool, and returns the message.
    template <class F> CheckBox&& on_change(F&& reaction) && {
        on_change_ = react<bool, M>(std::forward<F>(reaction));
        return std::move(*this);
    }

    static float width(Ui& ui, std::string_view label) {
        return controls::CheckBoxSize + (label.empty() ? 0.0f : controls::CheckBoxPadding.left + std::ceil(ui.measure(label, tokens::Body).width));
    }

    std::string_view kind() const override {
        return "CheckBox";
    }

    std::optional<std::string> label() const override {
        return checked ? std::string("Checked") : std::string("Unchecked");
    }

    std::optional<Sense> sense() const override {
        return Sense::Button;
    }

    void interact(const Interaction& interaction, Dispatch<M>& ctx) const override {
        if (interaction.whole().activated && on_change_) {
            ctx.send(on_change_(!checked));
        }
    }

    void draw(Draw& ctx) const override {
        Rect rect = ctx.rect();
        ControlState state = ctx.state();
        const Theme& theme = ctx.theme();
        // CheckBoxCheckBackgroundFill, CheckBoxCheckBackgroundStroke and
        // CheckBoxCheckGlyphForeground.
        auto [fill, stroke] =
            checked ? std::pair(StateColors(theme.accent, theme.accent_secondary, theme.accent_tertiary, theme.accent_disabled),
                                StateColors(theme.accent, theme.accent_secondary, theme.accent_tertiary, theme.control_strong_stroke_disabled))
                    : std::pair(StateColors(theme.control_alt_fill_secondary, theme.control_alt_fill_tertiary, theme.control_alt_fill_quarternary, theme.control_alt_fill_disabled),
                                StateColors(theme.control_strong_stroke, theme.control_strong_stroke, theme.control_strong_stroke_disabled, theme.control_strong_stroke_disabled));
        StateColors glyph(theme.text_on_accent, theme.text_on_accent, theme.text_on_accent_secondary, theme.text_on_accent_disabled);
        Color text = state == ControlState::Disabled ? theme.text_disabled : theme.text_primary;
        Rect box_rect(rect.x, std::round(rect.center().y - controls::CheckBoxSize / 2.0f), controls::CheckBoxSize, controls::CheckBoxSize);
        Painter painter = ctx.painter();
        // The stroke is inside the box, as a Rectangle's is.
        painter.shape(box_rect, tokens::RadiusControl, fill.get(state), Border(stroke.get(state), 1.0f));
        if (checked) {
            painter.icon(box_rect.center() + Point(0.0f, 1.0f), Icon::CheckMark, controls::CheckBoxGlyphSize, glyph.get(state));
        }
        if (!label_.empty()) {
            float left = box_rect.right() + controls::CheckBoxPadding.left;
            painter.text_fit(Rect(left, rect.y, rect.right() - left, rect.height), label_, tokens::Body, text, Align::Start);
        }
        bool focused = ctx.focused();
        ctx.focus_visual_around(focused, rect, tokens::RadiusControl, controls::FocusVisualMarginWide);
    }

private:
    bool checked;
    std::string label_;
    Reaction<bool, M> on_change_;
};

// One of a group of choices, checked while the group's value is its own.
template <class M> class RadioButton : public Widget<M> {
public:
    RadioButton(bool checked, std::string label) : checked(checked), label_(std::move(label)) {}
    RadioButton(const RadioButton&) = delete;
    RadioButton& operator=(const RadioButton&) = delete;
    RadioButton(RadioButton&&) = default;
    RadioButton& operator=(RadioButton&&) = default;

    // What a click sends, which should check it.
    RadioButton&& on_check(M message) && {
        on_check_ = std::move(message);
        return std::move(*this);
    }

    static float width(Ui& ui, std::string_view label) {
        return controls::RadioButtonSize + (label.empty() ? 0.0f : controls::RadioButtonPadding.left + std::ceil(ui.measure(label, tokens::Body).width));
    }

    std::string_view kind() const override {
        return "RadioButton";
    }

    std::optional<std::string> label() const override {
        return checked ? std::string("Checked") : std::string("Unchecked");
    }

    std::optional<Sense> sense() const override {
        return Sense::Button;
    }

    void interact(const Interaction& interaction, Dispatch<M>& ctx) const override {
        if (interaction.whole().activated && !checked && on_check_) {
            ctx.send(*on_check_);
        }
    }

    void draw(Draw& ctx) const override {
        Rect rect = ctx.rect();
        ControlState state = ctx.state();
        const Theme& theme = ctx.theme();
        Painter painter = ctx.painter();
        toggles_detail::draw_radio(painter, theme, rect, state, checked, label_);
        bool focused = ctx.focused();
        ctx.focus_visual_around(focused, rect, tokens::RadiusControl, controls::FocusVisualMarginWide);
    }

private:
    bool checked;
    std::string label_;
    std::optional<M> on_check_;
};

// A column of radio buttons that pick one of a group's items, by index.
// Each item is a row of ControlHeight named "{name}/{item}", so a group
// "Size" with "Small" in it has a target "Size/Small".
template <class M> class RadioGroup : public Widget<M> {
public:
    // The items are any range of texts, like an array of string views or a
    // vector of strings, or a list of them in double braces:
    // RadioGroup<Message>(size, {{"Small", "Large"}}).
    template <class Items = std::span<const std::string_view>>
        requires std::ranges::input_range<Items> && std::convertible_to<std::ranges::range_reference_t<Items>, std::string_view>
    RadioGroup(size_t selected, Items&& items) : selected(selected) {
        if constexpr (std::ranges::sized_range<Items>) {
            this->items.reserve(std::ranges::size(items));
        }
        for (auto&& item : items) {
            this->items.push_back(std::string(std::string_view(item)));
        }
    }
    RadioGroup(const RadioGroup&) = delete;
    RadioGroup& operator=(const RadioGroup&) = delete;
    RadioGroup(RadioGroup&&) = default;
    RadioGroup& operator=(RadioGroup&&) = default;

    // What a click on an item sends, with the item's index: `reaction` is
    // called with a size_t, and returns the message.
    template <class F> RadioGroup&& on_select(F&& reaction) && {
        on_select_ = react<size_t, M>(std::forward<F>(reaction));
        return std::move(*this);
    }

    // The group's height: a ControlHeight row an item.
    static float height(size_t count) {
        return static_cast<float>(count) * tokens::ControlHeight;
    }

    std::string_view kind() const override {
        return "RadioButtons";
    }

    std::optional<std::string> label() const override {
        if (selected < items.size()) {
            return items[selected];
        }
        return std::nullopt;
    }

    std::optional<Sense> sense() const override {
        return Sense::None;
    }

    void parts(Parts& ctx) const override {
        Rect rect = ctx.rect();
        for (size_t index = 0; index < items.size(); index++) {
            bool checked = index == selected;
            std::optional<std::string> label = checked ? std::string("Checked") : std::string("Unchecked");
            ctx.part(Part::Index{index}, items[index], row(rect, index), Sense::Button, "RadioButton", std::move(label));
        }
    }

    void interact(const Interaction& interaction, Dispatch<M>& ctx) const override {
        const Part* activated = interaction.activated_part();
        const Part::Index* part = activated != nullptr ? base::as<Part::Index>(*activated) : nullptr;
        if (part != nullptr && part->index != selected && on_select_) {
            ctx.send(on_select_(part->index));
        }
    }

    void draw(Draw& ctx) const override {
        Rect rect = ctx.rect();
        const Look& look = ctx.look();
        const Theme& theme = ctx.theme();
        for (size_t index = 0; index < items.size(); index++) {
            Part part = Part::Index{index};
            Rect row = RadioGroup::row(rect, index);
            Painter painter = ctx.painter();
            toggles_detail::draw_radio(painter, theme, row, look.part_state(part), index == selected, items[index]);
            bool focused = look.focused && look.focused_part == part;
            ctx.focus_visual_around(focused, row, tokens::RadiusControl, controls::FocusVisualMarginWide);
        }
    }

private:
    static Rect row(Rect rect, size_t index) {
        return Rect(rect.x, rect.y + tokens::ControlHeight * static_cast<float>(index), rect.width, tokens::ControlHeight);
    }

    size_t selected;
    std::vector<std::string> items;
    Reaction<size_t, M> on_select_;
};

// A switch that's on or off, with a label after it. Its knob is at the
// track's end for its value, bigger with the pointer over it, and stretched
// toward the other end while it's pressed.
template <class M> class ToggleSwitch : public Widget<M> {
public:
    ToggleSwitch(bool on, std::string label) : on(on), label_(std::move(label)) {}
    ToggleSwitch(const ToggleSwitch&) = delete;
    ToggleSwitch& operator=(const ToggleSwitch&) = delete;
    ToggleSwitch(ToggleSwitch&&) = default;
    ToggleSwitch& operator=(ToggleSwitch&&) = default;

    // What a click sends, with whether it's on then: `reaction` is called
    // with a bool, and returns the message.
    template <class F> ToggleSwitch&& on_change(F&& reaction) && {
        on_change_ = react<bool, M>(std::forward<F>(reaction));
        return std::move(*this);
    }

    static float width(Ui& ui, std::string_view label) {
        return controls::ToggleSwitchWidth + (label.empty() ? 0.0f : controls::ToggleSwitchContentGap + std::ceil(ui.measure(label, tokens::Body).width));
    }

    std::string_view kind() const override {
        return "ToggleSwitch";
    }

    std::optional<std::string> label() const override {
        return on ? std::string("On") : std::string("Off");
    }

    std::optional<Sense> sense() const override {
        return Sense::Button;
    }

    void interact(const Interaction& interaction, Dispatch<M>& ctx) const override {
        if (interaction.whole().activated && on_change_) {
            ctx.send(on_change_(!on));
        }
    }

    void draw(Draw& ctx) const override {
        Rect rect = ctx.rect();
        ControlState state = ctx.state();
        const Theme& theme = ctx.theme();
        // ToggleSwitchFillOn/Off, ToggleSwitchStrokeOff, and the knob's fill.
        // The track that's on has no stroke, and neither has the knob.
        auto [fill, stroke, knob] =
            on ? std::tuple(StateColors(theme.accent, theme.accent_secondary, theme.accent_tertiary, theme.accent_disabled), std::optional<StateColors>(),
                            StateColors(theme.text_on_accent, theme.text_on_accent, theme.text_on_accent, theme.text_on_accent_disabled))
               : std::tuple(StateColors(theme.control_alt_fill_secondary, theme.control_alt_fill_tertiary, theme.control_alt_fill_quarternary, theme.control_alt_fill_disabled),
                            std::optional(StateColors(theme.control_strong_stroke, theme.control_strong_stroke, theme.control_strong_stroke, theme.control_strong_stroke_disabled)),
                            StateColors(theme.text_secondary, theme.text_secondary, theme.text_secondary, theme.text_disabled));
        Color text = state == ControlState::Disabled ? theme.text_disabled : theme.text_primary;
        float knob_width = controls::ToggleSwitchKnob;
        float knob_height = controls::ToggleSwitchKnob;
        switch (state) {
        case ControlState::PointerOver:
            knob_width = controls::ToggleSwitchKnobPointerOver;
            knob_height = controls::ToggleSwitchKnobPointerOver;
            break;
        case ControlState::Pressed:
            knob_width = controls::ToggleSwitchKnobPressedWidth;
            knob_height = controls::ToggleSwitchKnobPressedHeight;
            break;
        case ControlState::Disabled:
        case ControlState::Normal:
            break;
        }
        Rect track(rect.x, std::round(rect.center().y - controls::ToggleSwitchHeight / 2.0f), controls::ToggleSwitchWidth, controls::ToggleSwitchHeight);
        Painter painter = ctx.painter();
        painter.shape(track, controls::ToggleSwitchHeight / 2.0f, fill.get(state), stroke ? std::optional(Border(stroke->get(state), 1.0f)) : std::nullopt);
        // The knob is centered in a 20-pixel square at the track's end; a
        // stretched one keeps 3 pixels from that end.
        float square = on ? track.x + controls::ToggleSwitchKnobTravel : track.x;
        float center_x = square + controls::ToggleSwitchHeight / 2.0f;
        if (state == ControlState::Pressed) {
            center_x = on ? square + controls::ToggleSwitchHeight - controls::ToggleSwitchKnobPressedMargin - knob_width / 2.0f
                          : square + controls::ToggleSwitchKnobPressedMargin + knob_width / 2.0f;
        }
        Rect knob_rect = Rect::from_center(Point(center_x, track.center().y), Size(knob_width, knob_height));
        painter.rounded(knob_rect, knob_height / 2.0f, knob.get(state));
        if (!label_.empty()) {
            float left = track.right() + controls::ToggleSwitchContentGap;
            painter.text_fit(Rect(left, rect.y, rect.right() - left, rect.height), label_, tokens::Body, text, Align::Start);
        }
        bool focused = ctx.focused();
        ctx.focus_visual_around(focused, rect, tokens::RadiusControl, controls::FocusVisualMarginWide);
    }

private:
    bool on;
    std::string label_;
    Reaction<bool, M> on_change_;
};

} // namespace user_interface
