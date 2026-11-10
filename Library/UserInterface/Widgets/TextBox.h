// WinUI's TextBox, one line: a field that takes typing while it has focus,
// with a caret that blinks, a selection that the pointer and Shift make,
// Ctrl's word moves, copy, cut and paste, a delete button while it has text
// and focus, and a button of its own, like AutoSuggestBox's query button.
//
// At rest its border is TextControlElevationBorderBrush, whose bottom edge is
// the strong stroke; with focus its fill is ControlFillColorInputActive and
// its bottom border is 2 pixels of the accent.
//
// The text is the program's: the field sends the text as the typing leaves
// it, and the program keeps it. The caret, the selection and the scroll are
// the field's own, kept between frames. The editing, the buttons and the
// drawing are shared with the widgets built of a field, NumberBox and
// ColorField.

#pragma once

#include "UserInterface/Core/Interaction.h"
#include "UserInterface/Core/Tree.h"
#include "UserInterface/Design/Icons.h"
#include "UserInterface/Widgets/Widget.h"

#include <cstddef>
#include <optional>
#include <string>
#include <string_view>
#include <utility>

namespace user_interface {

// The library's own: how a field is set up: what the text box and the widgets
// built of it share.
struct FieldSettings {
    std::string placeholder;
    std::optional<Icon> button;
    std::optional<size_t> max_length;
    bool clear_button = true;
    // Null for none.
    bool (*accept)(char32_t) = nullptr;
    // Room at its right end for controls of another widget's, like a number
    // box's spin buttons, and at its left, like a color field's swatch.
    float trailing = 0.0f;
    float leading = 0.0f;
};

// The library's own: what the frame's input did to a field.
struct Edited {
    // The text as the typing leaves it, if it changed.
    std::optional<std::string> text;
    bool submitted = false;
    bool button_clicked = false;
};

// The library's own: registers a field's buttons as parts: its clear button
// while it has text and focus, and its own button.
void field_parts(Parts& ctx, std::string_view text, const FieldSettings& settings);

// The library's own: does what the input did to a field with the text, and
// says how the text came out.
Edited field_interact(std::string_view given, const FieldSettings& settings, const Interaction& interaction, Dispatch<void>& ctx);

// The library's own: draws a field with the text, as it looks.
void field_draw(Draw& ctx, std::string_view text, const FieldSettings& settings);

template <class M> class TextBox : public Widget<M> {
public:
    explicit TextBox(std::string text) : text(std::move(text)) {}
    TextBox(const TextBox&) = delete;
    TextBox& operator=(const TextBox&) = delete;
    TextBox(TextBox&&) = default;
    TextBox& operator=(TextBox&&) = default;

    // What typing sends, with the text as it is then.
    template <class F> TextBox&& on_change(F&& reaction) && {
        on_change_ = react<std::string, M>(std::forward<F>(reaction));
        return std::move(*this);
    }

    // What Enter in the field sends.
    TextBox&& on_submit(M message) && {
        on_submit_ = std::move(message);
        return std::move(*this);
    }

    // What a click on its own button sends.
    TextBox&& on_button(M message) && {
        on_button_ = std::move(message);
        return std::move(*this);
    }

    // What the field says while it's empty, in the secondary color.
    TextBox&& placeholder(std::string placeholder) && {
        settings.placeholder = std::move(placeholder);
        return std::move(*this);
    }

    // A button of its own at its right, named "<name>/Button", like a
    // search box's.
    TextBox&& button(Icon icon) && {
        settings.button = icon;
        return std::move(*this);
    }

    // At most this many characters.
    TextBox&& max_length(size_t length) && {
        settings.max_length = length;
        return std::move(*this);
    }

    TextBox&& without_clear_button() && {
        settings.clear_button = false;
        return std::move(*this);
    }

    // Only characters that `accept` says yes to are typed or pasted.
    TextBox&& accept(bool (*accept)(char32_t)) && {
        settings.accept = accept;
        return std::move(*this);
    }

    std::string_view kind() const override {
        return "TextBox";
    }

    std::optional<std::string> label() const override {
        return text;
    }

    std::optional<Sense> sense() const override {
        return Sense::Text;
    }

    void parts(Parts& ctx) const override {
        field_parts(ctx, text, settings);
    }

    void interact(const Interaction& interaction, Dispatch<M>& ctx) const override {
        Edited edited = field_interact(text, settings, interaction, ctx);
        if (edited.text && on_change_) {
            ctx.send(on_change_(std::move(*edited.text)));
        }
        if (edited.submitted && on_submit_) {
            ctx.send(*on_submit_);
        }
        if (edited.button_clicked && on_button_) {
            ctx.send(*on_button_);
        }
    }

    void draw(Draw& ctx) const override {
        field_draw(ctx, text, settings);
    }

private:
    std::string text;
    FieldSettings settings;
    Reaction<std::string, M> on_change_;
    std::optional<M> on_submit_;
    std::optional<M> on_button_;
};

} // namespace user_interface
