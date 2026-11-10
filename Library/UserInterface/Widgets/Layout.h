// Laying a card's content out in fixed pixels, top to bottom: a Form gives
// rows for controls with their headers above them, as XAML's Header puts
// them, or labels beside them, and headings, captions and gaps between them,
// adding the text to the tree as it goes. Its `bottom` is where the content
// ends, for a card that sizes itself.
//
// And a 3D view's area, which takes every mouse button and the wheel without
// taking keyboard focus, so that the program's keys keep working while it's
// used, and sends what the pointer does in it.

#pragma once

#include "UserInterface/Core/Interaction.h"
#include "UserInterface/Core/Interface.h"
#include "UserInterface/Core/Tree.h"
#include "UserInterface/Design/Controls.h"
#include "UserInterface/Design/Tokens.h"
#include "UserInterface/Drawing/Geometry.h"
#include "UserInterface/Input/Devices.h"
#include "UserInterface/Widgets/Text.h"
#include "UserInterface/Widgets/Widget.h"

#include <algorithm>
#include <array>
#include <cstddef>
#include <optional>
#include <string>
#include <string_view>
#include <utility>

namespace user_interface {

class Form {
public:
    // Headers above the controls.
    explicit Form(Rect area);
    Form(const Form&) = delete;
    Form& operator=(const Form&) = delete;
    Form(Form&&) = default;
    Form& operator=(Form&&) = default;

    // Labels beside the controls, in a column `label_width` wide.
    static Form beside(Rect area, float label_width);

    // The gap between rows.
    Form&& gap(float gap) &&;

    // A rectangle `height` tall for anything, and the gap after it.
    Rect take(float height);

    // A control's row with its label, which goes in the tree: above the
    // control, or beside it. Returns the control's rectangle, `height` tall.
    template <class M> Rect row(Tree<M>& tree, std::string_view label, float height) {
        if (label_width) {
            float width = *label_width;
            Rect row = take(std::max(height, tokens::ControlHeight));
            tree.draw(Rect(row.x, row.y, width - tokens::SpaceS, std::min(tokens::ControlHeight, row.height)), Text::label(std::string(label)));
            return Rect(row.x + width, row.y, row.width - width, height);
        }
        if (!label.empty()) {
            Rect header = area.cut_top(tokens::Body.line_height, controls::TextBoxHeaderGap);
            tree.draw(header, Text::label(std::string(label)));
        }
        return take(height);
    }

    // A control's row of the standard height.
    template <class M> Rect control(Tree<M>& tree, std::string_view label) {
        return row(tree, label, tokens::ControlHeight);
    }

    // A heading in Body Strong, with a divider under it.
    template <class M> void heading(Tree<M>& tree, std::string_view text) {
        Rect row = area.cut_top(tokens::BodyStrong.line_height + tokens::SpaceXs + 1.0f, gap_);
        tree.draw(row.top_strip(tokens::BodyStrong.line_height), Text::heading(std::string(text)));
        tree.draw(row.bottom_strip(1.0f), Divider());
        used = row.bottom();
    }

    // A caption, wrapped, in the secondary color.
    template <class M> void caption(Ui& ui, Tree<M>& tree, std::string_view text) {
        float height = ui.paragraph_height(area.width, text, tokens::Caption);
        Rect rect = take(height);
        tree.draw(rect, Text::caption(std::string(text)).wrap());
    }

    // Body text, wrapped.
    template <class M> void paragraph(Ui& ui, Tree<M>& tree, std::string_view text) {
        float height = ui.paragraph_height(area.width, text, tokens::Body);
        Rect rect = take(height);
        tree.draw(rect, Text::paragraph(std::string(text)));
    }

    // N columns of equal width, `height` tall, like a row of buttons.
    template <size_t N> std::array<Rect, N> columns(float height) {
        return take(height).columns<N>(tokens::SpaceS);
    }

    // N controls side by side, each with its header above it, as `row` puts
    // one, in columns of equal width with a gap of a card's padding between
    // them. Returns the controls' rectangles, `height` tall. N comes first,
    // for a list of labels in braces: headed_columns<2>(tree, {"Width", "Height"}, height).
    template <size_t N, class M> std::array<Rect, N> headed_columns(Tree<M>& tree, const std::array<std::string_view, N>& labels, float height) {
        bool headed = std::ranges::any_of(labels, [](std::string_view label) { return !label.empty(); });
        float header_height = headed ? tokens::Body.line_height + controls::TextBoxHeaderGap : 0.0f;
        Rect headed_area = take(header_height + height);
        std::array<Rect, N> columns = headed_area.columns<N>(tokens::CardPadding);
        for (size_t index = 0; index < N; index++) {
            const Rect& column = columns[index];
            if (!labels[index].empty()) {
                tree.draw(Rect(column.x, column.y, column.width, tokens::Body.line_height), Text::label(std::string(labels[index])));
            }
        }
        for (Rect& column : columns) {
            column = Rect(column.x, column.y + header_height, column.width, height);
        }
        return columns;
    }

    void space(float height);

    // What's left.
    Rect rest() const;

    // Where the content ends.
    float bottom() const;

    // Whether everything taken fit in the area, for content that may not.
    bool fits() const;

    float width() const;

private:
    Rect area;
    // Labels go beside controls, in a column this wide, or above them.
    std::optional<float> label_width;
    float gap_;
    float used;
    // Whether everything taken so far fit in the area.
    bool fits_;
};

// A surface that the pointer stops at, like a card's, without a widget:
// what's under it isn't under the pointer.
class Cover : public Widget<> {
public:
    Cover() = default;
    Cover(const Cover&) = delete;
    Cover& operator=(const Cover&) = delete;
    Cover(Cover&&) = default;
    Cover& operator=(Cover&&) = default;

    std::string_view kind() const override;

    bool covers() const override;
};

// An element that only groups its children, like to disable a run of them
// at once, with no target and nothing drawn of its own.
class Group : public Widget<> {
public:
    Group() = default;
    Group(const Group&) = delete;
    Group& operator=(const Group&) = delete;
    Group(Group&&) = default;
    Group& operator=(Group&&) = default;

    std::string_view kind() const override;
};

// What the pointer did in a 3D view this frame.
struct ViewInput {
    // The pointer is on the view, and where it is, while it's on the view or
    // holds it.
    bool hovered = false;
    std::optional<Point> pointer;
    // Each button's press on the view this frame, which is what a click in
    // it acts on, and whether the left one was the second of a
    // double-click.
    bool pressed = false;
    bool secondary_pressed = false;
    bool middle_pressed = false;
    bool double_clicked = false;
    // The view is held, by which button, and how far the pointer moved
    // while it was.
    bool held = false;
    std::optional<MouseButton> held_button;
    Point drag_delta;
    // The left press ended this frame, which ends a drag.
    bool released = false;
    // Notches of the wheel turned over it.
    Point wheel;
    Modifiers modifiers;

    // Whether the left button holds it down.
    bool pressing() const;

    bool holding(MouseButton button) const;

    bool operator==(const ViewInput&) const = default;
};

// A 3D view's area. It takes presses of every mouse button and the wheel,
// but never keyboard focus, and a press in it takes focus from whatever had
// it, so that the program's keys work while it's used.
template <class M> class View : public Widget<M> {
public:
    View() = default;
    View(const View&) = delete;
    View& operator=(const View&) = delete;
    View(View&&) = default;
    View& operator=(View&&) = default;

    // What it takes from the pointer, instead of every button and the
    // wheel, like an area that only takes clicks.
    View&& sense(Sense sense) && {
        sense_ = sense;
        return std::move(*this);
    }

    // What it is to scripts, instead of "View": a literal, or text that
    // lasts as long as the program.
    View&& kind(std::string_view kind) && {
        kind_ = kind;
        return std::move(*this);
    }

    // Sends its input every frame, even one where nothing happened to it,
    // like for clearing what was under the pointer once it leaves.
    View&& every_frame() && {
        every_frame_ = true;
        return std::move(*this);
    }

    // The pointer's look over the view.
    View&& cursor(Cursor cursor) && {
        cursor_ = cursor;
        return std::move(*this);
    }

    // What the pointer doing anything in the view sends, each frame that it
    // does: `reaction` is called with a ViewInput, and returns the message.
    template <class F> View&& on_input(F&& reaction) && {
        on_input_ = react<ViewInput, M>(std::forward<F>(reaction));
        return std::move(*this);
    }

    std::string_view kind() const override {
        return kind_;
    }

    std::optional<Sense> sense() const override {
        return sense_;
    }

    bool always_interacts() const override {
        return every_frame_;
    }

    void interact(const Interaction& interaction, Dispatch<M>& ctx) const override {
        Hit hit = interaction.whole();
        if (hit.pressed || hit.secondary_pressed || hit.middle_pressed) {
            ctx.clear_focus();
        }
        if (cursor_ && (hit.hovered || hit.held)) {
            ctx.set_cursor(*cursor_);
        }
        if (on_input_) {
            ViewInput input{
                .hovered = hit.hovered,
                .pointer = interaction.pointer,
                .pressed = hit.pressed,
                .secondary_pressed = hit.secondary_pressed,
                .middle_pressed = hit.middle_pressed,
                .double_clicked = hit.double_clicked,
                .held = hit.held,
                .held_button = hit.held_button,
                .drag_delta = interaction.drag_delta,
                .released = hit.released,
                .wheel = hit.wheel,
                .modifiers = ctx.modifiers(),
            };
            ctx.send(on_input_(input));
        }
    }

private:
    std::optional<Cursor> cursor_;
    Sense sense_ = Sense::View;
    std::string_view kind_ = "View";
    bool every_frame_ = false;
    Reaction<ViewInput, M> on_input_;
};

} // namespace user_interface
