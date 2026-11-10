// WinUI's ScrollBar, and a region whose content scrolls behind it, like a
// ScrollViewer. The bar's thumb is a 2-pixel line while the pointer is away,
// and is 6 pixels, over an acrylic track, once the pointer has rested on the
// bar; the wheel scrolls the region under the pointer.
//
// How far something is scrolled is its own, kept between frames, which the
// program reads with Ui::memory to lay out what's in it:
//
//     Rect content = ScrollArea::content(ui, "Log", viewport, height);
//     tree.container("Log", viewport, ScrollArea(height), [&](Tree<Message>& area) {
//         area.draw(Rect(content.x, content.y + 40.0f, content.width, 20.0f), Text::label("A line"));
//     });

#pragma once

#include "UserInterface/Core/Interaction.h"
#include "UserInterface/Core/Tree.h"
#include "UserInterface/Drawing/Geometry.h"
#include "UserInterface/Widgets/Widget.h"

#include <optional>
#include <string_view>
#include <utility>

namespace user_interface {

// Pixels that a notch of the wheel scrolls: three lines of 48.
inline constexpr float WheelScroll = 144.0f;

// How far something that scrolls is scrolled, which it keeps between frames
// under its name.
struct Scrolled {
    float offset = 0.0f;

    bool operator==(const Scrolled&) const = default;
};

// The library's own, as the three functions after it are: the part that a
// widget's bar is, named "<widget>/ScrollBar".
Part bar_part();

// Registers a widget's bar along its viewport's right edge, if its content
// is taller than the viewport.
void scroll_bar_part(Parts& ctx, Rect viewport, float content);

// What the input on a bar does: a press on the thumb drags it, and a press
// on the track pages toward it. Returns the offset as the input leaves it.
float bar_input(Dispatch<void>& ctx, Id state_id, const Hit& hit, std::optional<Point> pointer, Rect bar, float content, float viewport, float offset);

// Draws a bar: expanded after the pointer has rested on it, and collapsed
// a while after the pointer left. `over` is whether the pointer is on it or
// holds it this frame.
void bar_draw(Draw& ctx, Id state_id, bool over, Rect bar, float content, float viewport, float offset);

// A vertical scroll bar on its own, for content `content` tall, seen
// through a viewport `viewport` tall, scrolled down by `offset`.
template <class M> class ScrollBar : public Widget<M> {
public:
    ScrollBar(float offset, float content, float viewport) : offset(offset), content(content), viewport(viewport) {}
    ScrollBar(const ScrollBar&) = delete;
    ScrollBar& operator=(const ScrollBar&) = delete;
    ScrollBar(ScrollBar&&) = default;
    ScrollBar& operator=(ScrollBar&&) = default;

    // What moving it sends, with the offset: `reaction` is called with a
    // float, and returns the message.
    template <class F> ScrollBar&& on_scroll(F&& reaction) && {
        on_scroll_ = react<float, M>(std::forward<F>(reaction));
        return std::move(*this);
    }

    std::string_view kind() const override {
        return "ScrollBar";
    }

    std::optional<Sense> sense() const override {
        return Sense::Click;
    }

    void interact(const Interaction& interaction, Dispatch<M>& ctx) const override {
        Hit hit = interaction.whole();
        Rect rect = ctx.rect();
        Id state_id = ctx.id();
        float scrolled = bar_input(ctx, state_id, hit, interaction.pointer, rect, content, viewport, offset);
        if (scrolled != offset && on_scroll_) {
            ctx.send(on_scroll_(scrolled));
        }
    }

    void draw(Draw& ctx) const override {
        Rect rect = ctx.rect();
        Id state_id = ctx.id();
        bool over = ctx.look().hit.hovered || ctx.look().hit.held;
        bar_draw(ctx, state_id, over, rect, content, viewport, offset);
    }

private:
    float offset;
    float content;
    float viewport;
    Reaction<float, M> on_scroll_;
};

// A region whose content scrolls: its children are clipped to it and go
// where `content` says, moved up by how far it's scrolled, and its bar is
// along its right edge, named "<name>/ScrollBar". The wheel over it scrolls
// it, unless a widget under the pointer takes the wheel itself, like a
// slider, or a menu or a dialog is over it.
class ScrollArea : public Widget<void> {
public:
    explicit ScrollArea(float content_height);
    ScrollArea(const ScrollArea&) = delete;
    ScrollArea& operator=(const ScrollArea&) = delete;
    ScrollArea(ScrollArea&&) = default;
    ScrollArea& operator=(ScrollArea&&) = default;

    // Where the content of the scroll area with the name goes: the
    // viewport's width less the bar's, as tall as the content, moved up by
    // how far it's scrolled.
    static Rect content(Ui& ui, std::string_view name, Rect viewport, float height);

    std::string_view kind() const override;

    bool covers() const override;

    void parts(Parts& ctx) const override;

    void interact(const Interaction& interaction, Dispatch<void>& ctx) const override;

    void draw(Draw& ctx) const override;

    bool takes_free_wheel() const override;

    std::optional<Rect> children_clip(Parts& ctx) const override;

private:
    float content_height;
};

} // namespace user_interface
