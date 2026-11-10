// WinUI's Expander: a header, as tall as ExpanderMinHeight, with a chevron
// button that shows or hides the content under it. The header is a card's
// fill and stroke; the content is the secondary card fill, which
// ExpanderContent draws under the header, with the program's widgets as its
// children.

#pragma once

#include "UserInterface/Core/Interaction.h"
#include "UserInterface/Core/Tree.h"
#include "UserInterface/Design/Controls.h"
#include "UserInterface/Design/Icons.h"
#include "UserInterface/Design/Tokens.h"
#include "UserInterface/Drawing/Geometry.h"
#include "UserInterface/Drawing/Painter.h"
#include "UserInterface/Widgets/Widget.h"

#include <optional>
#include <string>
#include <string_view>
#include <utility>

namespace user_interface {

template <class M> class Expander : public Widget<M> {
public:
    // The header, which the element's rectangle is.
    Expander(bool expanded, std::string header) : expanded(expanded), header(std::move(header)) {}
    Expander(const Expander&) = delete;
    Expander& operator=(const Expander&) = delete;
    Expander(Expander&&) = default;
    Expander& operator=(Expander&&) = default;

    // A caption under the header's text.
    Expander&& caption(std::string caption) && {
        caption_ = std::move(caption);
        return std::move(*this);
    }

    // What a click sends, with whether it's expanded then: `reaction` is
    // called with a bool, and returns the message.
    template <class F> Expander&& on_toggle(F&& reaction) && {
        on_toggle_ = react<bool, M>(std::forward<F>(reaction));
        return std::move(*this);
    }

    std::string_view kind() const override {
        return "Expander";
    }

    std::optional<std::string> label() const override {
        return expanded ? std::string("Expanded") : std::string("Collapsed");
    }

    std::optional<Sense> sense() const override {
        return Sense::Button;
    }

    void interact(const Interaction& interaction, Dispatch<M>& ctx) const override {
        if (interaction.whole().activated && on_toggle_) {
            ctx.send(on_toggle_(!expanded));
        }
    }

    void draw(Draw& ctx) const override {
        Rect rect = ctx.rect();
        ControlState state = ctx.state();
        {
            Painter painter = ctx.painter();
            const Theme& theme = painter.theme();
            // ExpanderChevronBackground: only the chevron's button changes.
            Color chevron_fill = theme.subtle_fill_transparent;
            if (state == ControlState::PointerOver) {
                chevron_fill = theme.subtle_fill_secondary;
            } else if (state == ControlState::Pressed) {
                chevron_fill = theme.subtle_fill_tertiary;
            }
            Color text = state == ControlState::Disabled ? theme.text_disabled : theme.text_primary;
            Color caption_color = state == ControlState::Disabled ? theme.text_disabled : theme.text_secondary;
            Corners corners = expanded ? Corners::top(tokens::RadiusControl) : Corners::all(tokens::RadiusControl);
            painter.shape(rect, corners, theme.card_background, Border(theme.card_stroke, 1.0f));
            Rect button(rect.right() - controls::ExpanderChevronMargin.right - controls::ExpanderChevronButton, rect.center().y - controls::ExpanderChevronButton / 2.0f,
                        controls::ExpanderChevronButton, controls::ExpanderChevronButton);
            painter.rounded(button, tokens::RadiusControl, chevron_fill);
            painter.icon(button.center(), expanded ? Icon::ChevronUp : Icon::ChevronDown, controls::ExpanderChevronGlyph, text);
            float left = rect.x + controls::ExpanderHeaderPaddingLeft;
            Rect room(left, rect.y, button.x - controls::ExpanderChevronMargin.left - left, rect.height);
            if (!caption_) {
                painter.text_fit(room, header, tokens::Body, text, Align::Start);
            } else {
                float top = rect.center().y - (tokens::Body.line_height + tokens::Caption.line_height) / 2.0f;
                painter.text_fit(Rect(room.x, top, room.width, tokens::Body.line_height), header, tokens::Body, text, Align::Start);
                painter.text_fit(Rect(room.x, top + tokens::Body.line_height, room.width, tokens::Caption.line_height), *caption_, tokens::Caption, caption_color, Align::Start);
            }
        }
        bool focused = ctx.focused();
        ctx.focus_visual_if(focused, rect, tokens::RadiusControl);
    }

private:
    bool expanded;
    std::string header;
    std::optional<std::string> caption_;
    Reaction<bool, M> on_toggle_;
};

// The content of an expanded expander, under its header: its fill, with
// the program's widgets as its children, inside `inner`.
class ExpanderContent : public Widget<void> {
public:
    ExpanderContent() = default;
    ExpanderContent(const ExpanderContent&) = delete;
    ExpanderContent& operator=(const ExpanderContent&) = delete;
    ExpanderContent(ExpanderContent&&) = default;
    ExpanderContent& operator=(ExpanderContent&&) = default;

    // The area inside the content's padding.
    static Rect inner(Rect rect);

    std::string_view kind() const override;

    bool covers() const override;

    void draw(Draw& ctx) const override;
};

} // namespace user_interface
