#include "UserInterface/Widgets/Scrolling.h"

#include "UserInterface/Core/Interface.h"
#include "UserInterface/Design/Controls.h"
#include "UserInterface/Design/Theme.h"
#include "UserInterface/Drawing/Painter.h"

#include <algorithm>
#include <format>
#include <string>

namespace user_interface {

namespace {

// The part that a widget's bar is, named "<widget>/ScrollBar".
constexpr std::string_view Bar = "ScrollBar";

Id bar_id(std::string_view widget) {
    return id(std::format("{}/{}", widget, Bar));
}

// When the pointer came onto the bar or left it, for its delays, and where
// in the thumb a drag took it.
struct BarState {
    std::optional<double> over_since;
    std::optional<double> away_since;
    bool expanded = false;
    float grab = 0.0f;
};

// A bar's track and thumb, for content `content` tall seen through
// `viewport`.
struct BarGeometry {
    Rect track;
    float thumb_length;
    float travel;
    float range;

    BarGeometry(Rect bar, float content, float viewport) :
        track(bar.inset(controls::ScrollViewerBarMargin)),
        thumb_length(content > 0.0f ? std::clamp(track.height * viewport / content, std::min(controls::ScrollBarMinThumb, track.height), track.height) : track.height),
        travel(std::max(track.height - thumb_length, 0.0f)),
        range(std::max(content - viewport, 0.0f)) {}

    float thumb_y(float offset) const {
        return track.y + (range > 0.0f ? travel * offset / range : 0.0f);
    }
};

} // namespace

Part bar_part() {
    return Part::Named{std::string(Bar)};
}

void scroll_bar_part(Parts& ctx, Rect viewport, float content) {
    if (content > viewport.height) {
        ctx.part(bar_part(), Bar, viewport.right_strip(controls::ScrollBarSize), Sense::Click, "ScrollBar", std::nullopt);
    }
}

float bar_input(Dispatch<void>& ctx, Id state_id, const Hit& hit, std::optional<Point> pointer, Rect bar, float content, float viewport, float offset) {
    BarGeometry geometry(bar, content, viewport);
    BarState& state = ctx.memory_of<BarState>(state_id);
    if (hit.pressed && pointer) {
        float top = geometry.thumb_y(offset);
        if (pointer->y >= top && pointer->y < top + geometry.thumb_length) {
            state.grab = pointer->y - top;
        } else {
            float page = std::max(viewport, 1.0f);
            offset = pointer->y < top ? offset - page : offset + page;
            state.grab = geometry.thumb_length / 2.0f;
        }
    } else if (hit.held && pointer) {
        float top = pointer->y - state.grab;
        offset = geometry.travel > 0.0f ? (top - geometry.track.y) / geometry.travel * geometry.range : 0.0f;
    }
    return std::clamp(offset, 0.0f, geometry.range);
}

void bar_draw(Draw& ctx, Id state_id, bool over, Rect bar, float content, float viewport, float offset) {
    BarGeometry geometry(bar, content, viewport);
    if (geometry.range <= 0.0f) {
        return;
    }
    double now = ctx.time();
    bool enabled = ctx.enabled();
    BarState& state = ctx.memory_of<BarState>(state_id);
    if (over) {
        state.away_since = std::nullopt;
        if (!state.over_since) {
            state.over_since = now;
        }
        double since = *state.over_since;
        if (now - since >= controls::ScrollBarExpandDelay) {
            state.expanded = true;
        }
    } else {
        state.over_since = std::nullopt;
        if (!state.away_since) {
            state.away_since = now;
        }
        double since = *state.away_since;
        if (now - since >= controls::ScrollBarContractDelay) {
            state.expanded = false;
        }
    }
    bool expanded = state.expanded;
    // A collapsed thumb sits toward the outer edge.
    auto [width, center_x] = expanded ? std::pair(controls::ScrollBarThumbExpanded, geometry.track.center().x)
                                      : std::pair(controls::ScrollBarThumbCollapsed, geometry.track.center().x + controls::ScrollBarThumbOffset);
    Painter painter = ctx.painter();
    const Theme& theme = painter.theme();
    if (expanded) {
        painter.rounded(geometry.track, controls::ScrollBarRadius * 2.0f, theme.acrylic);
    }
    Rect thumb(center_x - width / 2.0f, geometry.thumb_y(std::clamp(offset, 0.0f, geometry.range)), width, geometry.thumb_length);
    Color color = enabled ? theme.control_strong_fill : theme.control_strong_fill_disabled;
    painter.rounded(thumb, std::min(controls::ScrollBarRadius, width / 2.0f), color);
}

ScrollArea::ScrollArea(float content_height) : content_height(content_height) {}

Rect ScrollArea::content(Ui& ui, std::string_view name, Rect viewport, float height) {
    float range = std::max(height - viewport.height, 0.0f);
    float offset = std::clamp(ui.memory<Scrolled>(name).offset, 0.0f, range);
    float bar = range > 0.0f ? controls::ScrollBarSize : 0.0f;
    return Rect(viewport.x, viewport.y - offset, viewport.width - bar, height);
}

std::string_view ScrollArea::kind() const {
    return "ScrollViewer";
}

bool ScrollArea::covers() const {
    return true;
}

void ScrollArea::parts(Parts& ctx) const {
    Rect rect = ctx.rect();
    scroll_bar_part(ctx, rect, content_height);
}

void ScrollArea::interact(const Interaction& interaction, Dispatch<void>& ctx) const {
    Rect rect = ctx.rect();
    float range = std::max(content_height - rect.height, 0.0f);
    float offset = ctx.memory<Scrolled>().offset;
    offset -= interaction.wheel_over.y * WheelScroll;
    if (const Hit* hit = interaction.hit(bar_part())) {
        Id state_id = bar_id(ctx.name());
        offset = bar_input(ctx, state_id, *hit, interaction.pointer, rect.right_strip(controls::ScrollBarSize), content_height, rect.height, offset);
    }
    ctx.memory<Scrolled>().offset = std::clamp(offset, 0.0f, range);
}

void ScrollArea::draw(Draw& ctx) const {
    Rect rect = ctx.rect();
    float offset = ctx.memory<Scrolled>().offset;
    Part part = bar_part();
    bool over = ctx.look().hovered(part) || ctx.look().held(part);
    Id state_id = bar_id(ctx.name());
    bar_draw(ctx, state_id, over, rect.right_strip(controls::ScrollBarSize), content_height, rect.height, offset);
}

bool ScrollArea::takes_free_wheel() const {
    return true;
}

std::optional<Rect> ScrollArea::children_clip(Parts& ctx) const {
    return ctx.rect();
}

} // namespace user_interface
