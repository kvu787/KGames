// WinUI's NumberBox: a text field for a number, with its spin buttons inline.
// What's typed takes effect with Enter or when the field loses focus, inside
// its range and on its steps; the spin buttons, the arrows and Page Up and
// Page Down, and the wheel while it has focus, step it. The value is the
// program's; what's typed, until it takes effect, is the box's own.

#pragma once

#include "Base/Text.h"
#include "UserInterface/Core/Interaction.h"
#include "UserInterface/Core/Tree.h"
#include "UserInterface/Design/Controls.h"
#include "UserInterface/Design/Icons.h"
#include "UserInterface/Drawing/Geometry.h"
#include "UserInterface/Input/Devices.h"
#include "UserInterface/Input/Events.h"
#include "UserInterface/Widgets/Buttons.h"
#include "UserInterface/Widgets/TextBox.h"
#include "UserInterface/Widgets/Widget.h"

#include <algorithm>
#include <cmath>
#include <cstddef>
#include <format>
#include <limits>
#include <optional>
#include <string>
#include <string_view>
#include <utility>

namespace user_interface {

// What only the number box's own code uses, here and in NumberBox.cpp.
namespace number_box_detail {

// What's typed, while the field has focus.
struct Draft {
    std::string text;
    bool editing = false;
};

bool numeric(char32_t character);

} // namespace number_box_detail

template <class M> class NumberBox : public Widget<M> {
public:
    explicit NumberBox(double value) : value(value) {}
    NumberBox(const NumberBox&) = delete;
    NumberBox& operator=(const NumberBox&) = delete;
    NumberBox(NumberBox&&) = default;
    NumberBox& operator=(NumberBox&&) = default;

    // What a change sends, with the value.
    template <class F> NumberBox&& on_change(F&& reaction) && {
        on_change_ = react<double, M>(std::forward<F>(reaction));
        return std::move(*this);
    }

    NumberBox&& range(double lowest, double highest) && {
        minimum = std::min(lowest, highest);
        maximum = std::max(lowest, highest);
        return std::move(*this);
    }

    NumberBox&& step(double step) && {
        step_ = step;
        large_step_ = step * 10.0;
        return std::move(*this);
    }

    NumberBox&& large_step(double step) && {
        large_step_ = step;
        return std::move(*this);
    }

    NumberBox&& decimals(size_t decimals) && {
        decimals_ = decimals;
        return std::move(*this);
    }

    NumberBox&& placeholder(std::string placeholder) && {
        placeholder_ = std::move(placeholder);
        return std::move(*this);
    }

    std::string_view kind() const override {
        return "NumberBox";
    }

    std::optional<std::string> label() const override {
        return format(value);
    }

    std::optional<Sense> sense() const override {
        return Sense::Text.takes_wheel_while_focused();
    }

    void parts(Parts& ctx) const override {
        bool focused = ctx.is_focused();
        std::string text = shown_text(ctx.memory<number_box_detail::Draft>(), focused);
        field_parts(ctx, text, settings());
        auto [up, down] = spin_buttons(ctx.rect());
        ctx.part(Part::Named{"Up"}, "Up", up, Sense::Click, "Button", std::nullopt);
        ctx.part(Part::Named{"Down"}, "Down", down, Sense::Click, "Button", std::nullopt);
    }

    void interact(const Interaction& interaction, Dispatch<M>& ctx) const override {
        bool focused = interaction.focused;
        number_box_detail::Draft draft = std::exchange(ctx.template memory<number_box_detail::Draft>(), number_box_detail::Draft{});
        double current = value;
        // What's typed stays until it's taken or left, even as the field
        // loses focus, when it's taken - never thrown away.
        if (!draft.editing) {
            draft.text = format(current);
        }
        double new_value = current;
        Edited edited = field_interact(draft.text, settings(), interaction, ctx);
        if (edited.text) {
            draft.text = std::move(*edited.text);
            draft.editing = true;
        }
        bool commit = edited.submitted || (draft.editing && !ctx.is_focused());
        if (commit) {
            if (std::optional<double> typed = base::parse_f64(base::trim(draft.text))) {
                new_value = clamp(*typed);
            }
            draft.editing = false;
            draft.text = format(new_value);
        }
        double steps = 0.0;
        if (interaction.activated(Part::Named{"Up"})) {
            steps += 1.0;
        }
        if (interaction.activated(Part::Named{"Down"})) {
            steps -= 1.0;
        }
        if (focused) {
            const Input& input = ctx.input();
            if (input.key_pressed(Key::Up)) {
                steps += 1.0;
            }
            if (input.key_pressed(Key::Down)) {
                steps -= 1.0;
            }
            if (input.key_pressed(Key::PageUp)) {
                steps += large_step_ / step_;
            }
            if (input.key_pressed(Key::PageDown)) {
                steps -= large_step_ / step_;
            }
        }
        // The wheel steps it while it has focus and the pointer is on it, or
        // on one of its buttons, as WinUI's does; elsewhere, the wheel is for
        // what's under the pointer.
        if (focused && interaction.hovered_part() != nullptr) {
            steps += static_cast<double>(std::round(interaction.wheel().y));
        }
        if (steps != 0.0) {
            if (draft.editing) {
                if (std::optional<double> typed = base::parse_f64(base::trim(draft.text))) {
                    new_value = *typed;
                }
            }
            new_value = clamp(new_value + steps * step_);
            draft.editing = false;
            draft.text = format(new_value);
        }
        ctx.template memory<number_box_detail::Draft>() = std::move(draft);
        if (new_value != current && on_change_) {
            ctx.send(on_change_(new_value));
        }
    }

    void draw(Draw& ctx) const override {
        bool focused = ctx.focused();
        std::string text = shown_text(ctx.memory<number_box_detail::Draft>(), focused);
        field_draw(ctx, text, settings());
        auto [up, down] = spin_buttons(ctx.rect());
        draw_text_control_button(ctx, Part::Named{"Up"}, up, Icon::ChevronUp, controls::NumberBoxSpinGlyph);
        draw_text_control_button(ctx, Part::Named{"Down"}, down, Icon::ChevronDown, controls::NumberBoxSpinGlyph);
    }

private:
    std::string format(double shown) const {
        return std::format("{:.{}f}", shown, decimals_);
    }

    double clamp(double wanted) const {
        return std::clamp(wanted, minimum, maximum);
    }

    FieldSettings settings() const {
        return FieldSettings{
            .placeholder = placeholder_,
            .accept = number_box_detail::numeric,
            .trailing = controls::NumberBoxSpinColumn,
        };
    }

    // The text the field shows: what's typed, or the value.
    std::string shown_text(const number_box_detail::Draft& draft, bool focused) const {
        return focused && draft.editing ? draft.text : format(value);
    }

    // The spin buttons, inside the field's right end.
    static std::pair<Rect, Rect> spin_buttons(Rect rect) {
        Rect column(rect.right() - controls::NumberBoxSpinColumn, rect.y, controls::NumberBoxSpinColumn, rect.height);
        auto [up, down] = column.columns<2>(0.0f);
        return {up.pad(Insets(4.0f, 4.0f, 0.0f, 4.0f)), down.pad(controls::TextBoxButtonMargin)};
    }

    double value;
    double minimum = std::numeric_limits<double>::lowest();
    double maximum = std::numeric_limits<double>::max();
    double step_ = 1.0;
    double large_step_ = 10.0;
    size_t decimals_ = 0;
    std::string placeholder_;
    Reaction<double, M> on_change_;
};

} // namespace user_interface
