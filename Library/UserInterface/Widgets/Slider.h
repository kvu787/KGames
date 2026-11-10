// WinUI's Slider, horizontal: a rail in ControlStrongFill, the part up to the
// value in the accent, and a round thumb whose accent dot is bigger with the
// pointer over it and smaller while it's pressed. A press anywhere on the
// rail moves the thumb there and drags it; the arrows, Page Up, Page Down,
// Home and End move it; and the wheel turns it a step a notch, adding up
// parts of notches. While it's dragged, its value shows above the thumb.
//
// Its value is a double, which the program keeps in any number type. The
// rail's ends can be wider than the values allowed, like a rail from 0 to 1
// whose values must stay inside it, and the value never goes outside what's
// allowed. Only input moves the value: the slider sends the value the input
// asks for, and the program keeps it, so one from a file that's between
// steps or past the range stays until the user changes it.

#pragma once

#include "UserInterface/Core/Interaction.h"
#include "UserInterface/Core/Tree.h"
#include "UserInterface/Design/Controls.h"
#include "UserInterface/Design/Tokens.h"
#include "UserInterface/Drawing/Geometry.h"
#include "UserInterface/Drawing/Painter.h"
#include "UserInterface/Drawing/Text.h"
#include "UserInterface/Input/Devices.h"
#include "UserInterface/Input/Events.h"
#include "UserInterface/Widgets/Widget.h"

#include <algorithm>
#include <cmath>
#include <cstddef>
#include <format>
#include <optional>
#include <string>
#include <string_view>
#include <utility>

namespace user_interface {

// The rail's ends in a slider's rectangle: where its thumb's center goes
// at its minimum and its maximum.
std::pair<float, float> slider_rail(Rect rect);

// What only the slider's own code uses, here and in Slider.cpp.
namespace slider_detail {

// Turns of the wheel that haven't added up to a step yet.
struct WheelRemainder {
    float turns = 0.0f;
};

// Rounds away the noise of floating-point steps.
double tidy(double value);

} // namespace slider_detail

template <class M> class Slider : public Widget<M> {
public:
    Slider(double value, double minimum, double maximum) : value(value), minimum(minimum), maximum(maximum), allowed_(std::min(minimum, maximum), std::max(minimum, maximum)) {}
    Slider(const Slider&) = delete;
    Slider& operator=(const Slider&) = delete;
    Slider(Slider&&) = default;
    Slider& operator=(Slider&&) = default;

    // What moving it sends, with the value.
    template <class F> Slider&& on_change(F&& reaction) && {
        on_change_ = react<double, M>(std::forward<F>(reaction));
        return std::move(*this);
    }

    // What a press taking hold of it and letting go send, with whether it's
    // held, for a program that shows what's being dragged.
    template <class F> Slider&& on_hold(F&& reaction) && {
        on_hold_ = react<bool, M>(std::forward<F>(reaction));
        return std::move(*this);
    }

    // What the wheel turning over it sends, with its notches, besides
    // moving the value: even a turn that can't move it, like a part of a
    // notch, or at a value past what's allowed.
    template <class F> Slider&& on_wheel(F&& reaction) && {
        on_wheel_ = react<double, M>(std::forward<F>(reaction));
        return std::move(*this);
    }

    // The value moves in steps of this, from the minimum.
    Slider&& step(double step) && {
        step_ = step;
        return std::move(*this);
    }

    // How far Page Up and Page Down move it.
    Slider&& large_step(double step) && {
        large_step_ = step;
        return std::move(*this);
    }

    // The values allowed, inside the rail's ends.
    Slider&& allowed(double lowest, double highest) && {
        allowed_ = {std::min(lowest, highest), std::max(lowest, highest)};
        return std::move(*this);
    }

    // A header above the rail.
    Slider&& header(std::string header) && {
        header_ = std::move(header);
        return std::move(*this);
    }

    // Shows the value at the header's right, with this many decimals.
    Slider&& show_value(size_t decimals) && {
        show_value_ = decimals;
        decimals_ = decimals;
        return std::move(*this);
    }

    // The value's decimals, which show in the header and its tooltip.
    Slider&& decimals(size_t decimals) && {
        decimals_ = decimals;
        return std::move(*this);
    }

    Slider&& suffix(std::string suffix) && {
        suffix_ = std::move(suffix);
        return std::move(*this);
    }

    // How tall a slider is, with a header or without.
    static float height(bool headed) {
        return headed ? tokens::Body.line_height + controls::SliderHeaderGap + controls::SliderHeight : controls::SliderHeight;
    }

    std::string_view kind() const override {
        return "Slider";
    }

    std::optional<std::string> label() const override {
        return format(value);
    }

    std::optional<Sense> sense() const override {
        return Sense::Navigate.takes_wheel();
    }

    void interact(const Interaction& interaction, Dispatch<M>& ctx) const override {
        Hit hit = interaction.whole();
        if (hit.pressed && on_hold_) {
            ctx.send(on_hold_(true));
        }
        Rect rect = ctx.rect();
        auto [left, right] = slider_rail(rect);
        double before = value;
        double new_value = before;
        double span = maximum - minimum;
        double step = step_.value_or(span / 100.0);
        double large_step = large_step_.value_or(span / 10.0);

        if (hit.pressing() && interaction.pointer) {
            Point pointer = *interaction.pointer;
            double fraction = static_cast<double>(std::clamp((pointer.x - left) / std::max(right - left, 1.0f), 0.0f, 1.0f));
            new_value = snap(minimum + span * fraction);
        }
        if (interaction.focused) {
            const Input& input = ctx.input();
            double moved = 0.0;
            if (input.key_pressed(Key::Right) || input.key_pressed(Key::Up)) {
                moved += step;
            }
            if (input.key_pressed(Key::Left) || input.key_pressed(Key::Down)) {
                moved -= step;
            }
            if (input.key_pressed(Key::PageUp)) {
                moved += large_step;
            }
            if (input.key_pressed(Key::PageDown)) {
                moved -= large_step;
            }
            if (moved != 0.0) {
                new_value = snap(new_value + moved * std::copysign(1.0, span));
            }
            if (input.key_pressed(Key::Home)) {
                new_value = allowed_.first;
            }
            if (input.key_pressed(Key::End)) {
                new_value = allowed_.second;
            }
        }
        if (hit.wheel.y != 0.0f) {
            if (on_wheel_) {
                ctx.send(on_wheel_(static_cast<double>(hit.wheel.y)));
            }
            // Parts of notches add up, and a value between steps goes to the
            // next step the way the wheel turns, never back.
            float& remainder = ctx.template memory<slider_detail::WheelRemainder>().turns;
            remainder += hit.wheel.y;
            float notches = std::trunc(remainder);
            remainder -= notches;
            if (notches != 0.0f) {
                double direction = static_cast<double>(notches) * std::copysign(1.0, span);
                double current = new_value;
                if (step_ && *step_ > 0.0) {
                    // A value on a step, give or take rounding, is on it;
                    // between steps, it goes from the one behind it.
                    double index = (new_value - minimum) / *step_;
                    double nearest = std::round(index);
                    double snapped = std::abs(index - nearest) < 1e-6 ? nearest : direction > 0.0 ? std::floor(index) : std::ceil(index);
                    current = minimum + snapped * *step_;
                }
                double moved = std::clamp(slider_detail::tidy(current + direction * step), allowed_.first, allowed_.second);
                // Never back against the wheel, like from a value just past
                // what's allowed.
                if ((moved - new_value) * direction > 0.0) {
                    new_value = moved;
                }
            }
        }
        if (new_value != before && on_change_) {
            ctx.send(on_change_(new_value));
        }
        if (hit.released && on_hold_) {
            ctx.send(on_hold_(false));
        }
    }

    void draw(Draw& ctx) const override {
        Rect rect = ctx.rect();
        Rect track_row = this->track_row(rect);
        auto [left, right] = slider_rail(rect);
        std::string text = format(value);
        ControlState state = ctx.state();
        bool pressed = ctx.look().hit.pressing();
        ControlState visual_state = pressed ? ControlState::Pressed : state;
        float scale = 0.0f;
        switch (visual_state) {
        case ControlState::PointerOver:
            scale = controls::SliderInnerThumbScalePointerOver;
            break;
        case ControlState::Pressed:
            scale = controls::SliderInnerThumbScalePressed;
            break;
        case ControlState::Disabled:
            scale = controls::SliderInnerThumbScaleDisabled;
            break;
        case ControlState::Normal:
            scale = controls::SliderInnerThumbScale;
            break;
        }
        const Theme& theme = ctx.theme();
        StateColors rail(theme.control_strong_fill, theme.control_strong_fill, theme.control_strong_fill, theme.control_strong_fill_disabled);
        StateColors accent(theme.accent, theme.accent_secondary, theme.accent_tertiary, theme.accent_disabled);
        Color header_color = state == ControlState::Disabled ? theme.text_disabled : theme.text_primary;
        Color value_color = state == ControlState::Disabled ? theme.text_disabled : theme.text_secondary;
        float thumb_x = left + (right - left) * fraction(value);
        float center_y = std::round(track_row.center().y);
        bool focused = ctx.focused();
        Painter painter = ctx.painter();
        if (header_) {
            Rect header_row(rect.x, rect.y, rect.width, tokens::Body.line_height);
            painter.text_fit(header_row, *header_, tokens::Body, header_color, Align::Start);
            if (show_value_) {
                painter.text_in(header_row, text, tokens::Body, value_color, Align::End);
            }
        }
        Rect track(track_row.x, center_y - controls::SliderTrackHeight / 2.0f, track_row.width, controls::SliderTrackHeight);
        painter.rounded(track, controls::SliderTrackRadius, rail.get(state));
        // The accent fills from the minimum's end, or from zero when the
        // rail goes through it.
        float origin = minimum < 0.0 && maximum > 0.0 ? left + (right - left) * fraction(0.0) : track.x;
        auto [from, to] = thumb_x >= origin ? std::pair(origin, thumb_x) : std::pair(thumb_x, origin);
        painter.rounded(Rect(from, track.y, to - from, track.height), controls::SliderTrackRadius, accent.get(state));
        // The thumb is 18 pixels in layout, and its disc overhangs that by 2
        // on each side, with ControlElevationBorderBrush around it.
        Rect thumb = Rect::from_center(Point(thumb_x, center_y), Size::square(controls::SliderThumbSize));
        Rect disc = thumb.expand(controls::SliderThumbOverhang);
        painter.shape(disc, controls::SliderThumbRadius, theme.control_solid_fill, border(Elevation::Control, theme, disc));
        float inner = controls::SliderInnerThumbSize * scale;
        painter.circle(thumb.center(), inner / 2.0f, accent.get(visual_state));

        // The value above the thumb while it's dragged, like WinUI's thumb
        // tooltip.
        if (pressed) {
            TextStyle style(tokens::Body.font, tokens::Body.weight_, controls::SliderTooltipFontSize, 20.0f);
            Size size = painter.measure(text, style);
            Size tip(std::ceil(size.width) + controls::SliderTooltipPadding.horizontal(), size.height + controls::SliderTooltipPadding.vertical());
            Rect tip_rect(std::round(thumb_x - tip.width / 2.0f), std::round(disc.y - controls::TooltipOffset / 2.0f - tip.height), tip.width, tip.height);
            Layer layer = painter.layer();
            painter.set_layer(Layer::Tooltip);
            painter.shadow(tip_rect, tokens::RadiusControl, tokens::ElevationTooltip);
            painter.shape(tip_rect, tokens::RadiusControl, theme.acrylic, Border(theme.surface_stroke_flyout, 1.0f));
            painter.text_in(tip_rect.pad(controls::SliderTooltipPadding), text, style, theme.text_primary, Align::Center);
            painter.set_layer(layer);
        }
        ctx.focus_visual_around(focused, track_row, tokens::RadiusControl, Insets(7.0f, 0.0f, 7.0f, 0.0f));
    }

private:
    std::string format(double shown) const {
        return std::format("{:.{}f}{}", shown, decimals_, suffix_);
    }

    float fraction(double at) const {
        double span = maximum - minimum;
        return span == 0.0 ? 0.0f : static_cast<float>(std::clamp((at - minimum) / span, 0.0, 1.0));
    }

    // The value on a step nearest `wanted`, within what's allowed.
    double snap(double wanted) const {
        double snapped = step_ && *step_ > 0.0 ? minimum + std::round((wanted - minimum) / *step_) * *step_ : wanted;
        return std::clamp(slider_detail::tidy(snapped), allowed_.first, allowed_.second);
    }

    Rect track_row(Rect rect) const {
        return rect.bottom_strip(controls::SliderHeight);
    }

    double value;
    double minimum;
    double maximum;
    std::optional<double> step_;
    std::optional<double> large_step_;
    // The values allowed, inside the rail's ends.
    std::pair<double, double> allowed_;
    std::optional<std::string> header_;
    // Decimals of the value shown in the header, and of the label.
    std::optional<size_t> show_value_;
    size_t decimals_ = 2;
    std::string suffix_;
    Reaction<double, M> on_change_;
    Reaction<bool, M> on_hold_;
    Reaction<double, M> on_wheel_;
};

} // namespace user_interface
