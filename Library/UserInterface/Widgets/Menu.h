// WinUI's menus: MenuFlyout, which opens from a button, from a right-click as
// a context menu, or from a MenuBar's items; its items with their icons,
// checks and keyboard accelerators; its separators; and its submenus, which
// open beside their item, each raised 8 more than the menu it's from. A menu
// shows whole on the frame that opens it, and goes on the frame that closes
// it.
//
// Items are targets named "<menu>/<label>", and each carries the message
// that choosing it sends. The arrows move a highlight, Right opens a submenu
// and Left closes it, Enter chooses, and Esc closes the top menu. The
// functions here are what every owner of a menu shares: a button, a menu
// bar's item, and an element with a context menu.
//
// An item is never copied, so a menu's items are a vector made with vec,
// from Base/Vec.h, which moves them in, where braces would copy them:
//
//     std::vector<MenuItem<Message>> recent = base::vec(MenuItem<Message>("First"), MenuItem<Message>("Second"));
//     MenuBar<Message> bar(base::vec(MenuBar<Message>::Menu("File", base::vec(MenuItem<Message>("Open"), MenuItem<Message>::submenu("Recent", std::move(recent))))));

#pragma once

#include "UserInterface/Core/Interaction.h"
#include "UserInterface/Core/Tree.h"
#include "UserInterface/Design/Controls.h"
#include "UserInterface/Design/Icons.h"
#include "UserInterface/Design/Tokens.h"
#include "UserInterface/Drawing/Geometry.h"
#include "UserInterface/Drawing/Painter.h"
#include "UserInterface/Drawing/Text.h"
#include "UserInterface/Input/Devices.h"
#include "UserInterface/Input/Events.h"

#include <algorithm>
#include <cmath>
#include <cstddef>
#include <cstdint>
#include <format>
#include <optional>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

namespace user_interface {

template <class M> struct MenuItem {
    // The library's own.
    std::string label;
    std::optional<Icon> icon_;
    // Its keyboard accelerator's text, like "Ctrl+S".
    std::optional<std::string> accelerator_;
    bool enabled_ = true;
    // A ToggleMenuFlyoutItem's check.
    std::optional<bool> checked_;
    // What choosing it sends.
    std::optional<M> message;
    std::optional<std::vector<MenuItem<M>>> submenu_;
    bool separator_ = false;

    explicit MenuItem(std::string label) : label(std::move(label)) {}
    MenuItem(const MenuItem&) = delete;
    MenuItem& operator=(const MenuItem&) = delete;
    MenuItem(MenuItem&&) = default;
    MenuItem& operator=(MenuItem&&) = default;

    static MenuItem submenu(std::string text, std::vector<MenuItem<M>> items) {
        MenuItem item(std::move(text));
        item.submenu_ = std::move(items);
        return item;
    }

    static MenuItem separator() {
        MenuItem item{std::string()};
        item.separator_ = true;
        return item;
    }

    MenuItem&& icon(Icon icon) && {
        icon_ = icon;
        return std::move(*this);
    }

    MenuItem&& accelerator(std::string text) && {
        accelerator_ = std::move(text);
        return std::move(*this);
    }

    MenuItem&& enabled(bool enabled) && {
        enabled_ = enabled;
        return std::move(*this);
    }

    MenuItem&& checked(bool checked) && {
        checked_ = checked;
        return std::move(*this);
    }

    // What choosing it sends.
    MenuItem&& on_choose(M chosen) && {
        message = std::move(chosen);
        return std::move(*this);
    }

    // The library's own.
    bool is_enabled() const {
        return enabled_ && !separator_;
    }

    float height() const {
        return separator_ ? controls::MenuSeparatorHeight : controls::MenuItemHeight + controls::MenuItemMargin.vertical();
    }
};

// What only the menus' own code uses, here and in Menu.cpp.
namespace menu_detail {

// Where an open menu is, and which item the keyboard highlights.
struct MenuState {
    Point origin;
    std::optional<size_t> highlighted;
    // The submenu open from one of its items.
    std::optional<size_t> open_submenu;
};

// A menu's panel and columns, from its items and where it opens.
struct MenuLayout {
    Rect panel;
    bool has_icons;
    bool has_checks;
};

template <class M> MenuLayout layout(Fonts& fonts, Rect bounds, Point origin, const std::vector<MenuItem<M>>& items) {
    bool has_icons = std::ranges::any_of(items, [](const MenuItem<M>& item) { return item.icon_.has_value(); });
    bool has_checks = std::ranges::any_of(items, [](const MenuItem<M>& item) { return item.checked_.has_value(); });
    float indent = controls::MenuItemPlaceholder * static_cast<float>(static_cast<std::uint32_t>(has_icons) + static_cast<std::uint32_t>(has_checks));
    float widest = controls::MenuFlyoutMinWidth;
    for (const MenuItem<M>& item : items) {
        float text;
        if (item.separator_) {
            text = 0.0f;
        } else if (item.submenu_) {
            text = fonts.measure(item.label, tokens::Body).width + controls::MenuItemAcceleratorGap + controls::MenuItemChevronSize;
        } else {
            text = fonts.measure(item.label, tokens::Body).width +
                   (item.accelerator_ ? controls::MenuItemAcceleratorGap + fonts.measure(*item.accelerator_, tokens::Caption).width : 0.0f);
        }
        widest = std::max(widest, controls::MenuItemMargin.horizontal() + controls::MenuItemPadding.horizontal() + indent + text);
    }
    float width = std::ceil(widest);
    // From -0, as Rust's sum of floats is.
    float heights = -0.0f;
    for (const MenuItem<M>& item : items) {
        heights += item.height();
    }
    float height = controls::MenuFlyoutPadding.vertical() + heights;
    // It opens down from its origin, or up if there's no room below.
    float y = origin.y;
    if (y + height > bounds.bottom() - tokens::SpaceS) {
        y = std::max(origin.y - height, bounds.y + tokens::SpaceS);
    }
    float x = std::clamp(origin.x, bounds.x + tokens::SpaceS, std::max(bounds.right() - width - tokens::SpaceS, bounds.x));
    return MenuLayout{
        .panel = Rect(std::round(x), std::round(y), width, height),
        .has_icons = has_icons,
        .has_checks = has_checks,
    };
}

// Each item's slot in the panel, from the top.
template <class M> std::vector<Rect> slots(Rect panel, const std::vector<MenuItem<M>>& items) {
    float top = panel.y + controls::MenuFlyoutPadding.top;
    std::vector<Rect> made;
    made.reserve(items.size());
    for (const MenuItem<M>& item : items) {
        Rect slot(panel.x, top, panel.width, item.height());
        top += item.height();
        made.push_back(slot);
    }
    return made;
}

template <class M> std::string item_name(std::string_view menu, const MenuItem<M>& item) {
    return std::format("{}/{}", menu, item.label);
}

std::string child_name(std::string_view item_name);

// A context menu on an element: named "<element>/ContextMenu", opened at
// the pointer by a right press, on the frame the button goes down.
std::string context_menu_name(std::string_view element);

// The fonts that a Ui measures with, for MenuBar::width, which is here,
// where the Ui isn't defined yet.
Fonts& fonts_of(Ui& ui);

} // namespace menu_detail

// The library's own, as the rest of these functions are: opens the menu
// named `name` that `owner` owns, at the point.
void open_menu(Dispatch<void>& ctx, Id owner, std::string_view name, Point origin);

// Registers an open menu's targets: its panel, its items, and its open
// submenu's, recursively.
template <class M> void parts_menu(Parts& ctx, std::string_view name, Id owner, const std::vector<MenuItem<M>>& items) {
    if (!ctx.popup_open_of(owner)) {
        return;
    }
    menu_detail::MenuState state = ctx.memory_of<menu_detail::MenuState>(id(name));
    Rect bounds = ctx.bounds();
    menu_detail::MenuLayout layout = menu_detail::layout(ctx.fonts(), bounds, state.origin, items);
    ctx.part_on(Layer::Popup, Part::Named{std::string(name)}, name, layout.panel, Sense::None, "MenuFlyout", std::nullopt);
    std::vector<Rect> slots = menu_detail::slots(layout.panel, items);
    for (size_t index = 0; index < items.size(); index++) {
        const MenuItem<M>& item = items[index];
        if (item.separator_) {
            continue;
        }
        Rect rect = slots[index].pad(controls::MenuItemMargin);
        std::string item_name = menu_detail::item_name(name, item);
        ctx.part_on_enabled(Layer::Popup, Part::Named{item_name}, item_name, rect, Sense::Click, "MenuFlyoutItem", item.label, item.enabled_);
    }
    if (state.open_submenu && *state.open_submenu < items.size() && items[*state.open_submenu].submenu_) {
        const MenuItem<M>& item = items[*state.open_submenu];
        std::string item_name = menu_detail::item_name(name, item);
        parts_menu(ctx, menu_detail::child_name(item_name), id(item_name), *item.submenu_);
    }
}

// Does what the input on an open menu means: the keys move its highlight
// and choose, the pointer opens submenus, and a press chooses, which sends
// the item's message and closes every menu.
template <class M>
void interact_menu(const Interaction& interaction, Dispatch<M>& ctx, std::string_view name, Id owner, const std::vector<MenuItem<M>>& items, std::uint32_t level) {
    using menu_detail::MenuState;
    if (!ctx.popup_open_of(owner)) {
        return;
    }
    Id menu_id = id(name);
    MenuState state = std::exchange(ctx.template memory_of<MenuState>(menu_id), MenuState{});
    bool on_top = ctx.popup_on_top_of(owner);
    std::vector<size_t> selectable;
    for (size_t index = 0; index < items.size(); index++) {
        if (items[index].is_enabled()) {
            selectable.push_back(index);
        }
    }
    // The item the keyboard chose, if any: its message, which an item may
    // not have, in which case choosing it only closes the menus.
    std::optional<std::optional<M>> chosen;
    std::optional<size_t> open_child = state.open_submenu;
    if (on_top && !selectable.empty()) {
        std::optional<size_t> position;
        if (state.highlighted) {
            auto found = std::ranges::find(selectable, *state.highlighted);
            if (found != selectable.end()) {
                position = static_cast<size_t>(found - selectable.begin());
            }
        }
        bool down = ctx.input().key_pressed(Key::Down);
        bool up = ctx.input().key_pressed(Key::Up);
        if (down) {
            state.highlighted = selectable[position ? (*position + 1) % selectable.size() : 0];
        }
        if (up) {
            state.highlighted = selectable[position ? (*position + selectable.size() - 1) % selectable.size() : selectable.size() - 1];
        }
        if (down || up) {
            ctx.set_focus_visible(true);
        }
        if (state.highlighted) {
            size_t highlighted = *state.highlighted;
            const MenuItem<M>& item = items[highlighted];
            bool opens = item.submenu_.has_value();
            const Input& input = ctx.input();
            if ((input.key_pressed_once(Key::Enter) || input.key_pressed_once(Key::Space)) && !opens) {
                chosen.emplace(item.message);
            } else if ((input.key_pressed(Key::Right) || input.key_pressed_once(Key::Enter)) && opens) {
                open_child = highlighted;
            }
        }
        if (level > 0 && ctx.input().key_pressed(Key::Left)) {
            ctx.close_popup_of(owner);
        }
    }
    bool close_all = false;
    for (size_t index = 0; index < items.size(); index++) {
        const MenuItem<M>& item = items[index];
        if (item.separator_) {
            continue;
        }
        Part part = Part::Named{menu_detail::item_name(name, item)};
        const Hit* hit = interaction.hit(part);
        if (hit == nullptr) {
            continue;
        }
        if (hit->hovered) {
            state.highlighted = index;
            if (item.submenu_) {
                open_child = index;
            } else if (open_child) {
                open_child = std::nullopt;
            }
        }
        if (hit->activated && item.is_enabled()) {
            if (item.submenu_) {
                open_child = index;
            } else {
                if (item.message) {
                    ctx.send(*item.message);
                }
                close_all = true;
            }
        }
    }
    if (chosen) {
        if (*chosen) {
            ctx.send(std::move(**chosen));
        }
        close_all = true;
    }
    state.open_submenu = open_child;
    ctx.template memory_of<MenuState>(menu_id) = state;

    // The open submenu, beside its item, and what's chosen from it.
    if (open_child && *open_child < items.size() && items[*open_child].submenu_) {
        size_t index = *open_child;
        const MenuItem<M>& item = items[index];
        std::string item_name = menu_detail::item_name(name, item);
        Id item_id = id(item_name);
        std::string child = menu_detail::child_name(item_name);
        if (!ctx.popup_open_of(item_id)) {
            Rect bounds = ctx.bounds();
            Point origin = ctx.template memory_of<MenuState>(menu_id).origin;
            menu_detail::MenuLayout layout = menu_detail::layout(ctx.fonts(), bounds, origin, items);
            Rect slot = menu_detail::slots(layout.panel, items)[index];
            Rect rect = slot.pad(controls::MenuItemMargin);
            ctx.open_popup_of(item_id, child);
            MenuState& child_state = ctx.template memory_of<MenuState>(id(child));
            child_state.origin = Point(rect.right() + controls::MenuItemMargin.right, rect.y - controls::MenuFlyoutPadding.top - controls::MenuItemMargin.top);
            child_state.highlighted = std::nullopt;
            child_state.open_submenu = std::nullopt;
        }
        interact_menu(interaction, ctx, child, item_id, *item.submenu_, level + 1);
    } else {
        // Any submenu that was open from this menu closes.
        for (const MenuItem<M>& item : items) {
            if (item.submenu_) {
                ctx.close_popup_of(id(menu_detail::item_name(name, item)));
            }
        }
    }
    if (close_all) {
        ctx.close_popups();
    }
}

// Draws an open menu, raised for its level of nesting, and its open
// submenu.
template <class M> void draw_menu(Draw& ctx, std::string_view name, Id owner, const std::vector<MenuItem<M>>& items, std::uint32_t level) {
    if (!ctx.popup_open_of(owner)) {
        return;
    }
    menu_detail::MenuState state = ctx.memory_of<menu_detail::MenuState>(id(name));
    Rect bounds = ctx.bounds();
    menu_detail::MenuLayout layout = menu_detail::layout(ctx.fonts(), bounds, state.origin, items);
    Rect panel = layout.panel;
    bool focus_visible = ctx.focus_visible();
    const Look& look = ctx.look();
    const Theme& theme = ctx.theme();
    Painter painter = ctx.painter();
    Layer layer_before = painter.layer();
    painter.set_layer(Layer::Popup);
    painter.push_clip(bounds);
    painter.shadow(panel, tokens::RadiusOverlay,
                   tokens::elevation(tokens::ElevationFlyout.z + (tokens::ElevationSubmenu.z - tokens::ElevationFlyout.z) * static_cast<float>(level)));
    painter.shape(panel, tokens::RadiusOverlay, theme.acrylic, Border(theme.surface_stroke_flyout, 1.0f));
    std::vector<Rect> slots = menu_detail::slots(panel, items);
    for (size_t index = 0; index < items.size(); index++) {
        const MenuItem<M>& item = items[index];
        Rect slot = slots[index];
        if (item.separator_) {
            Rect line(panel.x + 1.0f, slot.y + 1.0f, panel.width - 2.0f, 1.0f);
            painter.fill(line, theme.divider);
            continue;
        }
        Rect rect = slot.pad(controls::MenuItemMargin);
        Part part = Part::Named{menu_detail::item_name(name, item)};
        ControlState state_now = item.enabled_ ? look.part_state(part) : ControlState::Disabled;
        bool submenu_open = state.open_submenu == index;
        bool highlighted = state.highlighted == index && focus_visible && !look.hovered(part);
        Color background = theme.subtle_fill_transparent;
        if (state_now == ControlState::Pressed) {
            background = theme.subtle_fill_tertiary;
        } else if (state_now == ControlState::PointerOver) {
            background = theme.subtle_fill_secondary;
        } else if (submenu_open || highlighted) {
            background = theme.subtle_fill_secondary;
        }
        Color foreground = state_now == ControlState::Disabled ? theme.text_disabled : theme.text_primary;
        Color secondary = theme.text_secondary;
        if (state_now == ControlState::Disabled) {
            secondary = theme.text_disabled;
        } else if (state_now == ControlState::Pressed) {
            secondary = theme.text_tertiary;
        }
        Rect content = rect.pad(controls::MenuItemPadding);
        painter.rounded(rect, tokens::RadiusControl, background);
        float x = content.x;
        if (layout.has_checks) {
            if (item.checked_ == true) {
                painter.icon(Point(x + controls::MenuItemCheckGlyph / 2.0f, rect.center().y), Icon::CheckMark, controls::MenuItemCheckGlyph, foreground);
            }
            x += controls::MenuItemPlaceholder;
        }
        if (layout.has_icons) {
            if (item.icon_) {
                painter.icon(Point(x + controls::MenuItemIconSize / 2.0f, rect.center().y), *item.icon_, controls::MenuItemIconSize, foreground);
            }
            x += controls::MenuItemPlaceholder;
        }
        painter.text_in(Rect(x, rect.y, content.right() - x, rect.height), item.label, tokens::Body, foreground, Align::Start);
        if (item.submenu_) {
            Point center(content.right() - controls::MenuItemChevronSize / 2.0f, rect.center().y);
            painter.icon(center, Icon::ChevronRight, controls::MenuItemChevronSize, secondary);
        } else if (item.accelerator_) {
            painter.text_in(content, *item.accelerator_, tokens::Caption, secondary, Align::End);
        }
    }
    painter.pop_clip();
    painter.set_layer(layer_before);
    if (state.open_submenu && *state.open_submenu < items.size() && items[*state.open_submenu].submenu_) {
        const MenuItem<M>& item = items[*state.open_submenu];
        std::string item_name = menu_detail::item_name(name, item);
        draw_menu(ctx, menu_detail::child_name(item_name), id(item_name), *item.submenu_, level + 1);
    }
}

template <class M> void parts_context_menu(Parts& ctx, const std::vector<MenuItem<M>>& items) {
    std::string name = menu_detail::context_menu_name(ctx.name());
    Id owner = ctx.id();
    parts_menu(ctx, name, owner, items);
}

template <class M> void interact_context_menu(const Interaction& interaction, Dispatch<M>& ctx, const std::vector<MenuItem<M>>& items) {
    std::string name = menu_detail::context_menu_name(ctx.name());
    Id owner = ctx.id();
    Hit hit = interaction.whole();
    if (hit.secondary_pressed) {
        if (std::optional<Point> pointer = ctx.pointer()) {
            open_menu(ctx, owner, name, *pointer);
        }
    }
    interact_menu(interaction, ctx, name, owner, items, 0);
}

template <class M> void draw_context_menu(Draw& ctx, const std::vector<MenuItem<M>>& items) {
    std::string name = menu_detail::context_menu_name(ctx.name());
    Id owner = ctx.id();
    draw_menu(ctx, name, owner, items, 0);
}

// WinUI's MenuBar: menus' names along a bar, each opening its menu below it.
// While one is open, the pointer moving onto another opens that one. Its
// items are targets named "<name>/<title>", and their menus
// "<name>/<title>/Menu".
template <class M> class MenuBar : public Widget<M> {
public:
    // A menu's title, and its items.
    using Menu = std::pair<std::string, std::vector<MenuItem<M>>>;

    explicit MenuBar(std::vector<Menu> menus) : menus(std::move(menus)) {}
    MenuBar(const MenuBar&) = delete;
    MenuBar& operator=(const MenuBar&) = delete;
    MenuBar(MenuBar&&) = default;
    MenuBar& operator=(MenuBar&&) = default;

    // The width its items take.
    float width(Ui& ui) const {
        Fonts& fonts = menu_detail::fonts_of(ui);
        // From -0, as Rust's sum of floats is.
        float total = -0.0f;
        for (const Menu& menu : menus) {
            total += std::ceil(fonts.measure(menu.first, tokens::Body).width) + controls::MenuBarItemPadding.horizontal() + controls::MenuBarItemMargin.horizontal();
        }
        return total;
    }

    std::string_view kind() const override {
        return "MenuBar";
    }

    std::optional<Sense> sense() const override {
        return Sense::None;
    }

    void parts(Parts& ctx) const override {
        Rect rect = ctx.rect();
        std::vector<Rect> rects = items(ctx.fonts(), rect);
        for (size_t index = 0; index < menus.size(); index++) {
            const auto& [title, menu_items] = menus[index];
            Rect item = rects[index];
            std::string name = item_name(ctx.name(), title);
            ctx.part(Part::Named{name}, title, item, Sense::Button, "MenuBarItem", title);
            std::string menu_name = menu_detail::child_name(name);
            parts_menu(ctx, menu_name, id(name), menu_items);
        }
    }

    void interact(const Interaction& interaction, Dispatch<M>& ctx) const override {
        Rect rect = ctx.rect();
        std::vector<Rect> rects = items(ctx.fonts(), rect);
        bool any_open = std::ranges::any_of(menus, [&](const Menu& menu) { return ctx.popup_open_of(id(item_name(ctx.name(), menu.first))); });
        for (size_t index = 0; index < menus.size(); index++) {
            const auto& [title, menu_items] = menus[index];
            Rect item = rects[index];
            std::string name = item_name(ctx.name(), title);
            Id item_id = id(name);
            std::string menu_name = menu_detail::child_name(name);
            Part part = Part::Named{name};
            const Hit* found = interaction.hit(part);
            Hit hit = found != nullptr ? *found : Hit{};
            bool open = ctx.popup_open_of(item_id);
            bool activated = hit.activated;
            if (any_open && !open && hit.hovered) {
                // Moving across the bar while a menu is open opens this one.
                activated = true;
            }
            if (interaction.focused_part == part && interaction.focused && ctx.input().key_pressed(Key::Down) && !open) {
                activated = true;
            }
            if (activated) {
                if (open) {
                    ctx.close_popup_of(item_id);
                } else {
                    open_menu(ctx, item_id, menu_name, Point(item.x, item.bottom() + 4.0f));
                }
            }
            interact_menu(interaction, ctx, menu_name, item_id, menu_items, 0);
        }
    }

    void draw(Draw& ctx) const override {
        Rect rect = ctx.rect();
        std::vector<Rect> rects = items(ctx.fonts(), rect);
        const Look& look = ctx.look();
        for (size_t index = 0; index < menus.size(); index++) {
            const auto& [title, menu_items] = menus[index];
            Rect item = rects[index];
            std::string name = item_name(ctx.name(), title);
            Id item_id = id(name);
            Part part = Part::Named{name};
            bool open = ctx.popup_open_of(item_id);
            ControlState state = look.part_state(part);
            const Theme& theme = ctx.theme();
            Color background = theme.subtle_fill_transparent;
            if (state == ControlState::Pressed) {
                background = theme.subtle_fill_tertiary;
            } else if (open) {
                background = theme.subtle_fill_tertiary;
            } else if (state == ControlState::PointerOver) {
                background = theme.subtle_fill_secondary;
            }
            Color foreground = state == ControlState::Disabled ? theme.text_disabled : theme.text_primary;
            Painter painter = ctx.painter();
            painter.rounded(item, tokens::RadiusControl, background);
            painter.text_in(item, title, tokens::Body, foreground, Align::Center);
            bool focused = look.focused && look.focused_part == part;
            ctx.focus_visual_if(focused, item, tokens::RadiusControl);
            draw_menu(ctx, menu_detail::child_name(name), item_id, menu_items, 0);
        }
    }

private:
    // Each item's rectangle, from the bar's left.
    std::vector<Rect> items(Fonts& fonts, Rect rect) const {
        float x = rect.x;
        std::vector<Rect> made;
        made.reserve(menus.size());
        for (const Menu& menu : menus) {
            float width = std::ceil(fonts.measure(menu.first, tokens::Body).width) + controls::MenuBarItemPadding.horizontal();
            Rect slot(x, rect.y, width + controls::MenuBarItemMargin.horizontal(), rect.height);
            x += slot.width;
            made.push_back(slot.pad(controls::MenuBarItemMargin));
        }
        return made;
    }

    static std::string item_name(std::string_view name, std::string_view title) {
        return std::format("{}/{}", name, title);
    }

    std::vector<Menu> menus;
};

} // namespace user_interface
