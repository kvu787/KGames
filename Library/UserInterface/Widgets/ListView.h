// WinUI's ListView: rows that scroll, of which one may be selected. Only the
// rows in sight are shown, so a list may be long. Each row is a target named
// "<name>/<index>", with its label, and what's in it is the list's children,
// which the program lays out in the rows that `rows` gives, inside
// ListViewItem's padding:
//
//     ListView<Message> list = ListView<Message>(count, controls::ListItemHeight).selection(selected).on_select(base::make<Message::Select>);
//     ListRows rows = list.rows(ui, "Devices", rect);
//     tree.container("Devices", rect, std::move(list), [&](Tree<Message>& list) {
//         for (size_t index : rows.indices()) {
//             list.draw(rows.content(index), Text::label(names[index]));
//         }
//     });
//
// A row's backplate is inset from it by 4 and 2, with a subtle fill under the
// pointer, and the selected row has the accent's pill at its left. With
// focus, the arrows, Home, End, Page Up and Page Down move the selection,
// and Enter activates the selected row, as a double-click does.

#pragma once

#include "Base/Numbers.h"
#include "Base/Sum.h"
#include "UserInterface/Core/Interaction.h"
#include "UserInterface/Core/Interface.h"
#include "UserInterface/Core/Tree.h"
#include "UserInterface/Design/Controls.h"
#include "UserInterface/Drawing/Geometry.h"
#include "UserInterface/Drawing/Painter.h"
#include "UserInterface/Input/Devices.h"
#include "UserInterface/Input/Events.h"
#include "UserInterface/Widgets/Scrolling.h"
#include "UserInterface/Widgets/Widget.h"

#include <algorithm>
#include <cmath>
#include <cstddef>
#include <format>
#include <functional>
#include <optional>
#include <ranges>
#include <string>
#include <string_view>
#include <utility>

namespace user_interface {

// Rows that a notch of the wheel scrolls.
inline constexpr float WheelRows = 3.0f;

template <class M> class ListView;

// The rows of a list in sight, and where each is.
class ListRows {
public:
    size_t first;
    // One past the last.
    size_t last;
    float offset;
    // The rows' area: the list's, less its bar.
    Rect area;

    std::ranges::iota_view<size_t, size_t> indices() const;

    // A row's whole area, as scrolled.
    Rect row(size_t index) const;

    // A row's area for what's in it, inside ListViewItem's padding.
    Rect content(size_t index) const;

private:
    // A list makes its rows, and reads how tall they are.
    template <class M> friend class ListView;

    ListRows(size_t first, size_t last, float offset, Rect area, Rect list, float row_height);

    Rect list;
    float row_height;
};

template <class M> class ListView : public Widget<M> {
public:
    // `count` rows, `row_height` tall.
    ListView(size_t count, float row_height) : count(count), row_height(row_height) {}
    ListView(const ListView&) = delete;
    ListView& operator=(const ListView&) = delete;
    ListView(ListView&&) = default;
    ListView& operator=(ListView&&) = default;

    // Rows can be selected, one at a time, and this one is.
    ListView&& selection(std::optional<size_t> row) && {
        selectable = true;
        selected = row;
        return std::move(*this);
    }

    // What selecting a row sends, with its index: `reaction` is called with
    // a size_t, and returns the message, as the two after it are.
    template <class F> ListView&& on_select(F&& reaction) && {
        on_select_ = react<size_t, M>(std::forward<F>(reaction));
        return std::move(*this);
    }

    // What a double-click on a row, or Enter on the selected one, sends.
    template <class F> ListView&& on_activate(F&& reaction) && {
        on_activate_ = react<size_t, M>(std::forward<F>(reaction));
        return std::move(*this);
    }

    // What a right-click on a row sends.
    template <class F> ListView&& on_context(F&& reaction) && {
        on_context_ = react<size_t, M>(std::forward<F>(reaction));
        return std::move(*this);
    }

    // Each row's label, for scripts and tests: `of` is called with a row's
    // index, and returns a std::string.
    template <class F> ListView&& labels(F&& of) && {
        label_of = std::function<std::string(size_t)>(std::forward<F>(of));
        return std::move(*this);
    }

    // Each row's tooltip, if it has one: `of` is called with a row's index,
    // and returns a std::optional<Tooltip>.
    template <class F> ListView&& tooltips(F&& of) && {
        tooltip_of = std::function<std::optional<Tooltip>(size_t)>(std::forward<F>(of));
        return std::move(*this);
    }

    // The rows in sight of the list with the name in the rectangle, as it's
    // scrolled, for laying out what's in them.
    ListRows rows(Ui& ui, std::string_view name, Rect rect) const {
        float offset = ui.memory<Scrolled>(name).offset;
        return geometry(rect, offset);
    }

    std::string_view kind() const override {
        return "ListView";
    }

    std::optional<std::string> label() const override {
        if (selected && label_of) {
            return label_of(*selected);
        }
        return std::nullopt;
    }

    std::optional<Sense> sense() const override {
        return selectable ? Sense::Navigate : Sense::None;
    }

    void parts(Parts& ctx) const override {
        Rect rect = ctx.rect();
        float offset = ctx.memory<Scrolled>().offset;
        ListRows rows = geometry(rect, offset);
        for (size_t index : rows.indices()) {
            std::optional<std::string> label = label_of ? std::optional(label_of(index)) : std::nullopt;
            ctx.part(Part::Index{index}, std::to_string(index), rows.row(index).intersect(rect), Sense::Click, "ListViewItem", std::move(label));
        }
        scroll_bar_part(ctx, rect, content_height());
    }

    std::optional<Tooltip> part_tooltip(const Part& part) const override {
        std::optional<size_t> index = part.index();
        if (!index || !tooltip_of) {
            return std::nullopt;
        }
        return tooltip_of(*index);
    }

    void interact(const Interaction& interaction, Dispatch<M>& ctx) const override {
        Rect rect = ctx.rect();
        float offset = ctx.template memory<Scrolled>().offset;
        ListRows rows = geometry(rect, offset);
        offset = rows.offset;
        float height = rows.row_height;
        std::optional<size_t> now_selected = selected;
        std::optional<size_t> before = now_selected;
        std::optional<size_t> activated;
        std::optional<size_t> context;

        if (interaction.focused && selectable && count > 0) {
            size_t page = std::max<size_t>(base::saturating_cast<size_t>(std::floor(rect.height / height)), 1);
            const Input& input = ctx.input();
            size_t last = count - 1;
            size_t current = now_selected.value_or(0);
            std::optional<size_t> moved;
            if (input.key_pressed(Key::Down)) {
                moved = !now_selected ? size_t{0} : std::min(current + 1, last);
            }
            if (input.key_pressed(Key::Up)) {
                moved = base::saturating_sub<size_t>(current, 1);
            }
            if (input.key_pressed(Key::PageDown)) {
                moved = std::min(current + page, last);
            }
            if (input.key_pressed(Key::PageUp)) {
                moved = base::saturating_sub(current, page);
            }
            if (input.key_pressed(Key::Home)) {
                moved = size_t{0};
            }
            if (input.key_pressed(Key::End)) {
                moved = last;
            }
            if (input.key_pressed_once(Key::Enter) && now_selected) {
                activated = *now_selected;
            }
            if (moved) {
                size_t row = *moved;
                now_selected = row;
                // The selected row scrolls into sight.
                float top = static_cast<float>(row) * height;
                if (top < offset) {
                    offset = top;
                } else if (top + height > offset + rect.height) {
                    offset = top + height - rect.height;
                }
            }
        }
        offset -= interaction.wheel_over.y * WheelRows * height;
        for (const auto& [part, hit] : interaction.hits) {
            const Part::Index* index = base::as<Part::Index>(part);
            if (index == nullptr) {
                continue;
            }
            if (hit.pressed && selectable) {
                now_selected = index->index;
                // A press on a row gives the list focus, as its one stop.
                if (ctx.focusable()) {
                    ctx.focus();
                }
            }
            if (hit.double_clicked) {
                activated = index->index;
            }
            if (hit.secondary_pressed) {
                context = index->index;
            }
        }
        if (const Hit* hit = interaction.hit(bar_part())) {
            Id state_id = id(std::format("{}/ScrollBar", ctx.name()));
            offset = bar_input(ctx, state_id, *hit, interaction.pointer, rect.right_strip(controls::ScrollBarSize), content_height(), rect.height, offset);
        }
        float range = std::max(content_height() - rect.height, 0.0f);
        ctx.template memory<Scrolled>().offset = std::clamp(offset, 0.0f, range);

        if (now_selected != before && now_selected && on_select_) {
            ctx.send(on_select_(*now_selected));
        }
        if (activated && on_activate_) {
            ctx.send(on_activate_(*activated));
        }
        if (context && on_context_) {
            ctx.send(on_context_(*context));
        }
    }

    void draw(Draw& ctx) const override {
        Rect rect = ctx.rect();
        const Look& look = ctx.look();
        float offset = ctx.memory<Scrolled>().offset;
        ListRows rows = geometry(rect, offset);
        bool enabled = ctx.enabled();
        std::optional<Rect> focused_plate;
        {
            Painter painter = ctx.painter();
            const Theme& theme = painter.theme();
            painter.push_clip(rect);
            for (size_t index : rows.indices()) {
                Part part = Part::Index{index};
                ControlState state = look.part_state(part);
                bool is_selected = selected == index;
                bool pressed = state == ControlState::Pressed;
                bool hovered = state == ControlState::PointerOver || pressed;
                // ListViewItemBackground.
                Color fill = theme.subtle_fill_secondary;
                if (!is_selected && pressed) {
                    fill = theme.subtle_fill_tertiary;
                } else if (!is_selected && hovered) {
                    fill = theme.subtle_fill_secondary;
                } else if (!is_selected) {
                    fill = theme.subtle_fill_transparent;
                } else if (pressed) {
                    fill = theme.subtle_fill_secondary;
                } else if (hovered) {
                    fill = theme.subtle_fill_tertiary;
                }
                Color indicator = enabled ? theme.accent : theme.accent_disabled;
                float pill = pressed ? controls::SelectionPillPressedLength : controls::SelectionPillLength;
                Rect plate = rows.row(index).pad(controls::ListItemBackplateMargin);
                painter.rounded(plate, controls::ListItemRadius, fill);
                if (is_selected) {
                    Rect bar(plate.x, plate.center().y - pill / 2.0f, controls::SelectionPillThickness, pill);
                    painter.rounded(bar, controls::SelectionPillThickness / 2.0f, indicator);
                    if (look.focused) {
                        focused_plate = plate;
                    }
                }
            }
            painter.pop_clip();
        }
        if (focused_plate) {
            ctx.focus_visual_around(true, *focused_plate, controls::ListItemRadius, Insets::all(-1.0f));
        }
        Part part = bar_part();
        bool over = look.hovered(part) || look.held(part);
        Id state_id = id(std::format("{}/ScrollBar", ctx.name()));
        bar_draw(ctx, state_id, over, rect.right_strip(controls::ScrollBarSize), content_height(), rect.height, offset);
    }

    bool takes_free_wheel() const override {
        return true;
    }

    std::optional<Rect> children_clip(Parts& ctx) const override {
        return ctx.rect();
    }

private:
    float content_height() const {
        return static_cast<float>(count) * std::max(row_height, 1.0f);
    }

    ListRows geometry(Rect rect, float offset) const {
        float height = std::max(row_height, 1.0f);
        float range = std::max(content_height() - rect.height, 0.0f);
        offset = std::clamp(offset, 0.0f, range);
        float bar_width = range > 0.0f ? controls::ScrollBarSize : 0.0f;
        Rect area(rect.x, rect.y, rect.width - bar_width, rect.height);
        size_t first = std::min(base::saturating_cast<size_t>(std::floor(offset / height)), count);
        size_t last = std::max(std::min(base::saturating_cast<size_t>(std::ceil((offset + rect.height) / height)), count), first);
        return ListRows(first, last, offset, area, rect, height);
    }

    size_t count;
    float row_height;
    bool selectable = false;
    std::optional<size_t> selected;
    Reaction<size_t, M> on_select_;
    Reaction<size_t, M> on_activate_;
    Reaction<size_t, M> on_context_;
    std::function<std::string(size_t)> label_of;
    std::function<std::optional<Tooltip>(size_t)> tooltip_of;
};

} // namespace user_interface
