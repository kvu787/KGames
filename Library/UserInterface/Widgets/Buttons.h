// WinUI's buttons: Button, in its standard, accent and subtle styles, with
// text, an icon, or both, and a chevron as a DropDownButton has, with a menu
// that opens below it; ToggleButton, which is the accent's colors while it's
// on; and HyperlinkButton. Their colors change at once between visual
// states, without WinUI's BrushTransition.
//
// This doesn't include Core/Interface.h, which includes this, by way of
// Application/Program.h and Widgets/InfoBar.h: a program that calls
// Button::width has the Ui from there.

#pragma once

#include "UserInterface/Core/Interaction.h"
#include "UserInterface/Core/Tree.h"
#include "UserInterface/Design/Controls.h"
#include "UserInterface/Design/Icons.h"
#include "UserInterface/Design/Theme.h"
#include "UserInterface/Design/Tokens.h"
#include "UserInterface/Drawing/Geometry.h"
#include "UserInterface/Drawing/Painter.h"
#include "UserInterface/Drawing/Text.h"
#include "UserInterface/Widgets/Menu.h"
#include "UserInterface/Widgets/Widget.h"

#include <cmath>
#include <cstdint>
#include <format>
#include <optional>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

namespace user_interface {

enum class ButtonStyle : std::uint8_t {
    Standard,
    Accent,
    // Transparent until the pointer is over it, like a toolbar's.
    Subtle,
};

// What only the buttons' own code uses, here and in Buttons.cpp.
namespace buttons_detail {

// The fonts that a Ui measures with, for Button::width, which is here,
// where the Ui isn't defined yet.
Fonts& fonts_of(Ui& ui);

} // namespace buttons_detail

// The library's own: draws a button's face in the rectangle: its background,
// border, icon, text and chevron.
void draw_button(Draw& ctx, Rect rect, std::string_view text, std::optional<Icon> icon, ButtonStyle style, bool checked, bool chevron, ControlState state);

// The library's own: a button inside a text field, like its delete button:
// transparent until the pointer is over it, with its glyph in the secondary
// color, as TextControlButton's brushes are.
void draw_text_control_button(Draw& ctx, const Part& part, Rect rect, Icon icon, float glyph);

template <class M> class ToggleButton;

template <class M> class Button : public Widget<M> {
public:
    explicit Button(std::string text) : text(std::move(text)) {}
    Button(const Button&) = delete;
    Button& operator=(const Button&) = delete;
    Button(Button&&) = default;
    Button& operator=(Button&&) = default;

    // A square button with only an icon, which wants a tooltip to say what
    // it does.
    static Button icon_only(Icon icon) {
        return Button("").icon(icon);
    }

    Button&& icon(Icon icon) && {
        icon_ = icon;
        return std::move(*this);
    }

    Button&& accent() && {
        style_ = ButtonStyle::Accent;
        return std::move(*this);
    }

    Button&& subtle() && {
        style_ = ButtonStyle::Subtle;
        return std::move(*this);
    }

    Button&& style(ButtonStyle style) && {
        style_ = style;
        return std::move(*this);
    }

    Button&& chevron() && {
        chevron_ = true;
        return std::move(*this);
    }

    // What a click sends.
    Button&& on_press(M message) && {
        on_press_ = std::move(message);
        return std::move(*this);
    }

    // A menu that opens below it, with a chevron after its text, as a
    // DropDownButton.
    Button&& menu(std::vector<MenuItem<M>> items) && {
        menu_ = std::move(items);
        chevron_ = true;
        return std::move(*this);
    }

    // How wide the button is for its text, icon and chevron: ButtonPadding
    // around them.
    float width(Ui& ui) const {
        float width = controls::ButtonPadding.horizontal();
        if (icon_) {
            width += text.empty() ? controls::ButtonIconOnlyWidth - controls::ButtonPadding.horizontal() : tokens::Icon + controls::ButtonIconGap;
        }
        if (!text.empty()) {
            width += std::ceil(buttons_detail::fonts_of(ui).measure(text, tokens::Body).width);
        }
        if (chevron_) {
            width += controls::DropDownChevronGap + controls::DropDownChevronSize;
        }
        return width;
    }

    std::string_view kind() const override {
        return menu_ ? "DropDownButton" : "Button";
    }

    std::optional<std::string> label() const override {
        if (text.empty()) {
            return std::nullopt;
        }
        return text;
    }

    std::optional<Sense> sense() const override {
        return Sense::Button;
    }

    void parts(Parts& ctx) const override {
        if (menu_) {
            std::string name = menu_name(ctx.name());
            Id owner = ctx.id();
            parts_menu(ctx, name, owner, *menu_);
        }
    }

    void interact(const Interaction& interaction, Dispatch<M>& ctx) const override {
        Hit hit = interaction.whole();
        if (!menu_) {
            if (hit.activated && on_press_) {
                ctx.send(*on_press_);
            }
        } else {
            std::string name = menu_name(ctx.name());
            if (hit.activated) {
                if (ctx.popup_open()) {
                    ctx.close_popup();
                } else {
                    Rect rect = ctx.rect();
                    open_menu(ctx, ctx.id(), name, Point(rect.x, rect.bottom() + 4.0f));
                }
            }
            if (ctx.popup_open()) {
                interact_menu(interaction, ctx, name, ctx.id(), *menu_, 0);
            }
        }
    }

    void draw(Draw& ctx) const override {
        ControlState state = ctx.state();
        Rect rect = ctx.rect();
        draw_button(ctx, rect, text, icon_, style_, checked, chevron_, state);
        bool focused = ctx.focused();
        ctx.focus_visual_if(focused, rect, tokens::RadiusControl);
        if (menu_ && ctx.popup_open()) {
            std::string name = menu_name(ctx.name());
            Id owner = ctx.id();
            draw_menu(ctx, name, owner, *menu_, 0);
        }
    }

private:
    // A toggle button draws its button's text, icon, style and chevron.
    friend class ToggleButton<M>;

    static std::string menu_name(std::string_view name) {
        return std::format("{}/Menu", name);
    }

    std::string text;
    std::optional<Icon> icon_;
    ButtonStyle style_ = ButtonStyle::Standard;
    // A chevron after the text, as a DropDownButton has.
    bool chevron_ = false;
    // A ToggleButton's state: on, so the accent's colors.
    bool checked = false;
    std::optional<M> on_press_;
    // A menu that a press opens below the button, named "<name>/Menu".
    std::optional<std::vector<MenuItem<M>>> menu_;
};

// A button that's on or off, in the accent's colors while it's on.
template <class M> class ToggleButton : public Widget<M> {
public:
    ToggleButton(bool on, Button<M> button) : button(std::move(button)), on(on) {}
    ToggleButton(const ToggleButton&) = delete;
    ToggleButton& operator=(const ToggleButton&) = delete;
    ToggleButton(ToggleButton&&) = default;
    ToggleButton& operator=(ToggleButton&&) = default;

    // What switching it sends, with whether it's on then: `reaction` is
    // called with a bool, and returns the message.
    template <class F> ToggleButton&& on_toggle(F&& reaction) && {
        on_toggle_ = react<bool, M>(std::forward<F>(reaction));
        return std::move(*this);
    }

    std::string_view kind() const override {
        return "ToggleButton";
    }

    std::optional<std::string> label() const override {
        return on ? std::string("On") : std::string("Off");
    }

    std::optional<Sense> sense() const override {
        return Sense::Button;
    }

    void interact(const Interaction& interaction, Dispatch<M>& ctx) const override {
        if (interaction.whole().activated && on_toggle_) {
            ctx.send(on_toggle_(!on));
        }
    }

    void draw(Draw& ctx) const override {
        ControlState state = ctx.state();
        Rect rect = ctx.rect();
        draw_button(ctx, rect, button.text, button.icon_, button.style_, on, button.chevron_, state);
        bool focused = ctx.focused();
        ctx.focus_visual_if(focused, rect, tokens::RadiusControl);
    }

private:
    Button<M> button;
    bool on;
    Reaction<bool, M> on_toggle_;
};

// A link that's a button: accent-colored text, which takes a subtle fill
// with the pointer over it.
template <class M> class HyperlinkButton : public Widget<M> {
public:
    explicit HyperlinkButton(std::string text) : text(std::move(text)) {}
    HyperlinkButton(const HyperlinkButton&) = delete;
    HyperlinkButton& operator=(const HyperlinkButton&) = delete;
    HyperlinkButton(HyperlinkButton&&) = default;
    HyperlinkButton& operator=(HyperlinkButton&&) = default;

    HyperlinkButton&& on_press(M message) && {
        on_press_ = std::move(message);
        return std::move(*this);
    }

    std::string_view kind() const override {
        return "HyperlinkButton";
    }

    std::optional<std::string> label() const override {
        return text;
    }

    std::optional<Sense> sense() const override {
        return Sense::Button;
    }

    void interact(const Interaction& interaction, Dispatch<M>& ctx) const override {
        Hit hit = interaction.whole();
        if (hit.hovered && interaction.enabled) {
            ctx.set_cursor(Cursor::Hand);
        }
        if (hit.activated && on_press_) {
            ctx.send(*on_press_);
        }
    }

    void draw(Draw& ctx) const override {
        ControlState state = ctx.state();
        Rect rect = ctx.rect();
        const Theme& theme = ctx.theme();
        Color background = StateColors(theme.subtle_fill_transparent, theme.subtle_fill_secondary, theme.subtle_fill_tertiary, theme.subtle_fill_disabled).get(state);
        Color foreground = StateColors(theme.accent_text, theme.accent_text_secondary, theme.accent_text_tertiary, theme.accent_text_disabled).get(state);
        Painter painter = ctx.painter();
        painter.shape(rect, tokens::RadiusControl, background, std::nullopt);
        painter.text_fit(rect.pad(controls::HyperlinkPadding), text, tokens::Body, foreground, Align::Center);
        bool focused = ctx.focused();
        ctx.focus_visual_if(focused, rect, tokens::RadiusControl);
    }

private:
    std::string text;
    std::optional<M> on_press_;
};

} // namespace user_interface
