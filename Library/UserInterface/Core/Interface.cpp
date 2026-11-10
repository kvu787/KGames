#include "UserInterface/Core/Interface.h"

#include "Base/Panic.h"
#include "UserInterface/Design/Tokens.h"

#include <algorithm>
#include <bit>
#include <functional>

namespace user_interface {

namespace interface_detail {

std::unique_ptr<Kept>& kept(Interface& ui, Id id, std::type_index type) {
    return ui.memory_[MemoryKey{id, type}];
}

size_t MemoryKeyHash::operator()(const MemoryKey& key) const noexcept {
    // The id is a hash already, which the type's hash is mixed into.
    size_t hash = static_cast<size_t>(key.id);
    return hash ^ (key.type.hash_code() + 0x9e3779b97f4a7c15 + (hash << 6) + (hash >> 2));
}

Look LookSources::look(Id element, bool enabled, bool focus_visible) const {
    Look made;
    made.enabled = enabled;
    if (hovered && hovered->first == element && enabled) {
        const Part& part = hovered->second;
        made.hovered_part = part;
        if (base::is<Part::Whole>(part)) {
            made.hit.hovered = true;
        }
    }
    if (active && std::get<0>(*active) == element) {
        const Part& part = std::get<1>(*active);
        MouseButton button = std::get<2>(*active);
        made.held_part = part;
        made.held_button = button;
        if (base::is<Part::Whole>(part)) {
            made.hit.held = true;
            made.hit.held_button = button;
        }
    }
    if (focused && std::get<0>(*focused) == element) {
        made.focused = std::get<2>(*focused);
        made.focused_part = std::get<1>(*focused);
        made.focus_visible = focus_visible;
    }
    return made;
}

Id cover_id(Rect rect) {
    std::uint64_t hash = 0x84222325cbf29ce4;
    for (float value : {rect.x, rect.y, rect.width, rect.height}) {
        hash ^= static_cast<std::uint64_t>(std::bit_cast<std::uint32_t>(value));
        hash *= 0x0100000001b3;
    }
    return hash;
}

std::optional<std::tuple<Id, Part, bool>> owner_of_focus(std::span<const Target> targets, std::span<const Popup> popups, bool modal_shown, std::optional<Id> focused) {
    if (!focused) {
        return std::nullopt;
    }
    for (const Target& target : targets) {
        if (target.id == *focused) {
            bool keys_here = (popups.empty() || target.layer >= Layer::Popup) && (!modal_shown || target.layer >= Layer::Dialog);
            return std::tuple<Id, Part, bool>(target.owner, target.part, keys_here);
        }
    }
    return std::nullopt;
}

} // namespace interface_detail

Interface::Interface(Fonts fonts, ThemeKind theme, Clipboard clipboard) : theme_(Theme(theme)), fonts_(std::move(fonts)), clipboard(std::move(clipboard)) {}

FramePhases Interface::phases() const {
    return phases_;
}

void Interface::begin_input() {
    cursor_ = Cursor::Arrow;
    press_consumed = {false, false, false};
    escape_consumed = false;
    requests.clear();
    if (input_.focus_lost) {
        popups.clear();
    }

    // The pointer is on the topmost target under it, or only on the
    // target with a press while there is one. A dialog leaves nothing
    // under it to be on.
    std::optional<Id> top = topmost(targets_, modal_shown);
    hovered = under_pointer(top);

    // A press outside the open popups closes them, and does nothing else.
    if (!popups.empty()) {
        for (MouseButton button : AllMouseButtons) {
            if (!input_.button_pressed(button)) {
                continue;
            }
            bool inside = false;
            if (top) {
                Id under = *top;
                inside =
                    std::ranges::any_of(popups, [&](const Popup& popup) { return popup.owner == under; }) ||
                    std::ranges::any_of(targets_, [&](const Target& target) { return target.id == under && target.layer == Layer::Popup; }) ||
                    std::ranges::any_of(targets_, [&](const Target& target) {
                        return target.id == under && std::ranges::any_of(popups, [&](const Popup& popup) { return popup.owner == target.owner; }) && target.layer == Layer::Popup;
                    });
            }
            if (!inside) {
                popups.clear();
                press_consumed[index(button)] = true;
            }
        }
    }

    // A press or the wheel puts the tooltip away until the pointer moves
    // to another widget.
    if (std::ranges::any_of(AllMouseButtons, [&](MouseButton button) { return input_.button_pressed(button); }) || input_.wheel != Point::Zero) {
        tooltip.dismissed = true;
    }

    captured = Keys::None;
    if (focused) {
        auto target = std::ranges::find_if(targets_, [&](const Target& candidate) { return candidate.id == *focused; });
        if (target != targets_.end()) {
            captured = target->sense.keys;
        }
    }
    if (!popups.empty() && captured != Keys::Text) {
        captured = Keys::Navigate;
    }

    // F9 switches the theme from this frame, which draws in the new one.
    if (input_.key_pressed_once(Key::F9) && !is_captured(Key::F9)) {
        toggle_theme();
    }

    // Esc closes the top popup, or else takes keyboard focus away. A
    // dialog hears it itself.
    if (input_.key_pressed(Key::Escape)) {
        if (!popups.empty()) {
            popups.pop_back();
            escape_consumed = true;
        } else if (focused && !modal_shown) {
            set_focused(std::nullopt);
            escape_consumed = true;
        }
    }
}

Hit Interface::hit(const Target& target) {
    Id target_id = target.id;
    Sense sense = target.sense;
    Hit made;
    made.hovered = target.enabled && hovered == target_id;
    if (!target.enabled) {
        if (active && active->first == target_id) {
            active.reset();
        }
        return made;
    }
    for (MouseButton button : AllMouseButtons) {
        if (!(made.hovered && sense.takes(button) && input_.button_pressed(button) && !press_consumed[index(button)])) {
            continue;
        }
        active = std::pair<Id, MouseButton>(target_id, button);
        press_consumed[index(button)] = true;
        switch (button) {
        case MouseButton::Left:
            made.pressed = true;
            made.activated = true;
            made.double_clicked = input_.clicks(MouseButton::Left) >= 2;
            // A target that doesn't take focus leaves it where it
            // was, like a text field's clear button leaves it in the
            // field.
            if (sense.focusable && target.focusable) {
                set_focused(target_id);
                focus_visible = false;
            }
            break;
        case MouseButton::Right:
            made.secondary_pressed = true;
            break;
        case MouseButton::Middle:
            made.middle_pressed = true;
            break;
        }
    }
    // A widget that doesn't take the right button still hears a right
    // press, for a context menu, unless the press closed the popups.
    if (made.hovered && !sense.right && input_.button_pressed(MouseButton::Right) && !press_consumed[index(MouseButton::Right)]) {
        made.secondary_pressed = true;
    }
    if (active && active->first == target_id) {
        MouseButton button = active->second;
        made.held = true;
        made.held_button = button;
        if (button == MouseButton::Left && (input_.button_released(button) || input_.focus_lost)) {
            made.released = true;
        }
    }
    if (made.hovered) {
        made.wheel = input_.wheel;
    }
    if (focused == target_id && sense.keys == Keys::Activate && (input_.key_pressed_once(Key::Space) || input_.key_pressed_once(Key::Enter))) {
        bool keys_here = (popups.empty() || target.layer >= Layer::Popup) && (!modal_shown || target.layer >= Layer::Dialog);
        if (keys_here) {
            made.activated = true;
        }
    }
    return made;
}

void Interface::move_focus() {
    // Tab and Shift+Tab move keyboard focus through the targets that take
    // it, in the order the frame showed them, in a dialog if one is open.
    if (input_.key_pressed(Key::Tab) && popups.empty() && !input_.modifiers().control) {
        Layer layer = modal_shown ? Layer::Dialog : Layer::Base;
        std::vector<Id> order;
        for (const Target& target : targets_) {
            if (target.focusable && target.sense.focusable && target.enabled && target.layer == layer) {
                order.push_back(target.id);
            }
        }
        if (!order.empty()) {
            bool backward = input_.modifiers().shift;
            std::optional<size_t> at;
            if (focused) {
                auto found = std::ranges::find(order, *focused);
                if (found != order.end()) {
                    at = static_cast<size_t>(found - order.begin());
                }
            }
            size_t next;
            if (at && backward) {
                next = (*at + order.size() - 1) % order.size();
            } else if (at) {
                next = (*at + 1) % order.size();
            } else if (backward) {
                next = order.size() - 1;
            } else {
                next = 0;
            }
            set_focused(order[next]);
            focus_visible = true;
        }
    }
    // A left press that no widget took takes focus away.
    if (input_.button_pressed(MouseButton::Left) && !press_consumed[0] && !active) {
        set_focused(std::nullopt);
    }
}

void Interface::set_focused(std::optional<Id> to) {
    if (focused != to) {
        if (focused && std::ranges::find(focus_leavers, *focused) == focus_leavers.end()) {
            focus_leavers.push_back(*focused);
        }
        focused = to;
    }
}

void Interface::end_input() {
    if (active && !input_.button_down(active->second)) {
        active.reset();
    }
    if (input_.key_pressed_once(Key::F10) && !is_captured(Key::F10)) {
        // The frame presents with the new setting, so the PerfBar says
        // it from this very frame; the loop re-reads the truth after.
        environment_.vertical_sync = !environment_.vertical_sync;
        requests.push_back(Request::ToggleVerticalSync);
    }
    if (input_.key_pressed_once(Key::F11) && !is_captured(Key::F11)) {
        requests.push_back(Request::ToggleFullscreen);
    }
}

interface_detail::LookSources Interface::look_sources() const {
    auto find = [&](std::optional<Id> wanted) -> const Target* {
        if (!wanted) {
            return nullptr;
        }
        auto target = std::ranges::find_if(targets_, [&](const Target& candidate) { return candidate.id == *wanted; });
        return target != targets_.end() ? &*target : nullptr;
    };
    auto keys_here = [&](const Target& target) {
        return (popups.empty() || target.layer >= Layer::Popup) && (!modal_ || target.layer >= Layer::Dialog);
    };
    interface_detail::LookSources sources;
    if (const Target* target = find(hovered)) {
        sources.hovered = std::pair<Id, Part>(target->owner, target->part);
    }
    if (active) {
        if (const Target* target = find(active->first)) {
            sources.active = std::tuple<Id, Part, MouseButton>(target->owner, target->part, active->second);
        }
    }
    if (const Target* target = find(focused)) {
        sources.focused = std::tuple<Id, Part, bool>(target->owner, target->part, keys_here(*target));
    }
    return sources;
}

std::optional<Id> Interface::topmost(std::span<const Target> targets, bool modal) const {
    if (!input_.pointer) {
        return std::nullopt;
    }
    Point point = *input_.pointer;
    Layer lowest = modal ? Layer::Dialog : Layer::Base;
    const Target* best = nullptr;
    for (const Target& target : targets) {
        if (!(target.layer >= lowest && target.rect.contains(point))) {
            continue;
        }
        if (best != nullptr && best->layer > target.layer) {
            continue;
        }
        best = &target;
    }
    if (best == nullptr) {
        return std::nullopt;
    }
    return best->id;
}

std::optional<Id> Interface::under_pointer(std::optional<Id> topmost) const {
    if (active) {
        if (topmost == active->first) {
            return active->first;
        }
        return std::nullopt;
    }
    return topmost;
}

bool Interface::hovered_takes_wheel(std::span<const Target> targets) const {
    if (!hovered) {
        return false;
    }
    for (auto target = targets.rbegin(); target != targets.rend(); ++target) {
        if (target->id == *hovered && target->enabled) {
            switch (target->sense.wheel) {
            case Wheel::Taken:
                return true;
            case Wheel::WhileFocused:
                return hovered == focused;
            case Wheel::Ignored:
                return false;
            }
            return false;
        }
    }
    return false;
}

const Target* Interface::hovered_target() const {
    if (!hovered) {
        return nullptr;
    }
    for (auto target = targets_.rbegin(); target != targets_.rend(); ++target) {
        if (target->id == *hovered) {
            return &*target;
        }
    }
    return nullptr;
}

const Target* Interface::held_target() const {
    if (!active) {
        return nullptr;
    }
    for (auto target = targets_.rbegin(); target != targets_.rend(); ++target) {
        if (target->id == active->first) {
            return &*target;
        }
    }
    return nullptr;
}

const Theme& Interface::theme() const {
    return next_theme ? *next_theme : theme_;
}

void Interface::toggle_theme() {
    ThemeKind kind = other(theme().kind);
    next_theme = Theme(kind);
}

Size Interface::size() const {
    return size_;
}

Rect Interface::bounds() const {
    float top = content_top();
    return Rect(0.0f, top, size_.width, std::max(size_.height - top, 0.0f));
}

float Interface::content_top() const {
    return performance ? tokens::PerfBarHeight : 0.0f;
}

bool Interface::fullscreen() const {
    return fullscreen_;
}

bool Interface::vertical_sync() const {
    return environment_.vertical_sync;
}

double Interface::time() const {
    return input_.time;
}

std::uint64_t Interface::frame_number() const {
    return frame_;
}

FrameStats Interface::stats() const {
    if (performance) {
        const performance::Summary& summary = performance->summary();
        return FrameStats{
            .frames_per_second = static_cast<float>(summary.frames_per_second),
            .frame_time = static_cast<float>(summary.frame_time),
        };
    }
    return FrameStats{};
}

const Environment& Interface::environment() const {
    return environment_;
}

const Input& Interface::input() const {
    return input_;
}

const Gamepad& Interface::gamepad() const {
    return input_.gamepad;
}

bool Interface::tooltips_enabled() const {
    return tooltips_enabled_;
}

bool Interface::key_pressed(Key key) const {
    return input_.key_pressed(key) && !is_captured(key) && !modal_shown;
}

bool Interface::key_pressed_once(Key key) const {
    return input_.key_pressed_once(key) && !is_captured(key) && !modal_shown;
}

bool Interface::key_down(Key key) const {
    return input_.key_down(key) && !(captured == Keys::Text && !key.is_modifier()) && !modal_shown;
}

bool Interface::is_captured(Key key) const {
    if (key == Key::Escape && escape_consumed) {
        return true;
    }
    switch (captured) {
    case Keys::None:
        return false;
    case Keys::Activate:
        return key == Key::Space || key == Key::Enter;
    case Keys::Navigate:
        return key.is_navigation() || key == Key::Space || key == Key::Enter;
    case Keys::Text: {
        bool control = input_.modifiers().control;
        bool editing = key == Key::A || key == Key::C || key == Key::X || key == Key::V || key == Key::Z || key == Key::Y || key == Key::Left || key == Key::Right ||
                       key == Key::Backspace || key == Key::Delete || key == Key::Home || key == Key::End;
        return !key.is_function() && key != Key::Escape && key != Key::Tab && (!control || editing);
    }
    }
    base::unreachable();
}

Painter Interface::painter() {
    return Painter{draw, fonts_, theme_};
}

Size Interface::measure(std::string_view text, TextStyle style) {
    return fonts_.measure(text, style);
}

Fonts& Interface::fonts() {
    return fonts_;
}

Cursor Interface::cursor() const {
    return cursor_;
}

void Interface::cover_on(Layer layer, Rect rect, Id owner) {
    targets_.push_back(Target{
        .id = interface_detail::cover_id(rect),
        .name = std::string(),
        .label = std::nullopt,
        .rect = rect,
        .layer = layer,
        .enabled = true,
        .focusable = false,
        .sense = Sense::None,
        .kind = "Surface",
        .listed = false,
        .owner = owner,
        .part = Part::Named{"Cover"},
    });
}

size_t Interface::instance_count() const {
    return draw.instance_count();
}

void Interface::open_popup(Id owner, std::string_view name) {
    bool in_popup = std::ranges::any_of(targets_, [&](const Target& target) { return target.id == owner && target.layer == Layer::Popup; });
    if (!in_popup) {
        popups.clear();
    } else if (auto open = std::ranges::find_if(popups, [&](const Popup& popup) { return popup.owner == owner; }); open != popups.end()) {
        popups.erase(open, popups.end());
    }
    popups.push_back(Popup{
        .owner = owner,
        .name = std::string(name),
    });
}

void Interface::close_popup(Id owner) {
    auto open = std::ranges::find_if(popups, [&](const Popup& popup) { return popup.owner == owner; });
    if (open != popups.end()) {
        popups.erase(open, popups.end());
    }
}

bool Interface::popup_open(Id owner) const {
    return std::ranges::any_of(popups, [&](const Popup& popup) { return popup.owner == owner; });
}

bool Interface::popup_on_top(Id owner) const {
    return !popups.empty() && popups.back().owner == owner;
}

bool Interface::any_popup() const {
    return !popups.empty();
}

bool Interface::modal() const {
    return modal_;
}

std::optional<Point> Interface::tooltip_due() const {
    bool due = tooltips_enabled_ && !active && popups.empty() && hovered.has_value() && tooltip.candidate == hovered && !tooltip.dismissed &&
               input_.time - tooltip.since >= tokens::TooltipDelay;
    if (due) {
        return tooltip.anchor;
    }
    return std::nullopt;
}

std::span<const Notice> Interface::notices() const {
    return notices_;
}

std::span<const Target> Interface::targets() const {
    return targets_;
}

const Target* Interface::target(std::string_view name) const {
    Id wanted = id(name);
    auto found = std::ranges::find_if(targets_, [&](const Target& candidate) { return candidate.id == wanted && candidate.listed; });
    return found != targets_.end() ? &*found : nullptr;
}

std::optional<std::string_view> Interface::listed_name(std::optional<Id> id) const {
    if (!id) {
        return std::nullopt;
    }
    auto found = std::ranges::find_if(targets_, [&](const Target& candidate) { return candidate.id == *id && candidate.listed; });
    if (found == targets_.end()) {
        return std::nullopt;
    }
    return std::string_view(found->name);
}

std::optional<std::string_view> Interface::hovered_name() const {
    return listed_name(hovered);
}

std::optional<std::string_view> Interface::pressed_name() const {
    return listed_name(active ? std::optional<Id>(active->first) : std::nullopt);
}

std::optional<std::string_view> Interface::focused_name() const {
    return listed_name(focused);
}

std::optional<std::string_view> Interface::popup_name() const {
    if (popups.empty()) {
        return std::nullopt;
    }
    return std::string_view(popups.back().name);
}

std::optional<std::string_view> Interface::tooltip_name() const {
    if (!shown_tooltip) {
        return std::nullopt;
    }
    return std::string_view(shown_tooltip->first);
}

json::Value Interface::state() const {
    using json::Value;
    auto area = [](Rect rect) {
        return Value(std::vector<double>{static_cast<double>(rect.x), static_cast<double>(rect.y), static_cast<double>(rect.width), static_cast<double>(rect.height)});
    };
    auto name = [](std::optional<std::string_view> found) {
        return found ? Value(*found) : Value();
    };
    std::vector<std::pair<std::string, Value>> listed;
    for (const Target& target : targets_) {
        if (!target.listed) {
            continue;
        }
        if (std::ranges::any_of(listed, [&](const std::pair<std::string, Value>& member) { return member.first == target.name; })) {
            continue;
        }
        std::vector<std::pair<std::string, Value>> members;
        members.reserve(3);
        members.emplace_back("Area", area(target.rect));
        members.emplace_back("Kind", Value(target.kind));
        members.emplace_back("Enabled", Value(target.enabled));
        if (target.label) {
            members.emplace_back("Label", Value(std::string_view(*target.label)));
        }
        listed.emplace_back(target.name, Value(Value::Object{std::move(members)}));
    }
    Value tooltip_shown;
    if (shown_tooltip) {
        std::vector<std::pair<std::string, Value>> members;
        members.reserve(2);
        members.emplace_back("Name", Value(std::string_view(shown_tooltip->first)));
        members.emplace_back("Area", area(shown_tooltip->second));
        tooltip_shown = Value(Value::Object{std::move(members)});
    }
    Value pointer;
    if (input_.pointer) {
        pointer = Value(std::vector<double>{static_cast<double>(input_.pointer->x), static_cast<double>(input_.pointer->y)});
    }
    std::vector<std::pair<std::string, Value>> members;
    members.reserve(13);
    members.emplace_back("Theme", Value(user_interface::name(theme_.kind)));
    members.emplace_back("Size", Value(std::vector<double>{static_cast<double>(size_.width), static_cast<double>(size_.height)}));
    members.emplace_back("Fullscreen", Value(fullscreen_));
    members.emplace_back("VerticalSync", Value(environment_.vertical_sync));
    members.emplace_back("WindowFocused", Value(input_.focused));
    members.emplace_back("Pointer", std::move(pointer));
    members.emplace_back("Hovered", name(hovered_name()));
    members.emplace_back("Pressed", name(pressed_name()));
    members.emplace_back("Focused", name(focused_name()));
    members.emplace_back("Popup", name(popup_name()));
    members.emplace_back("Dialog", Value(modal_));
    members.emplace_back("Tooltip", std::move(tooltip_shown));
    members.emplace_back("Targets", Value(Value::Object{std::move(listed)}));
    return Value(Value::Object{std::move(members)});
}

const Theme& Ui::theme() const {
    return inner.theme();
}

Size Ui::size() const {
    return inner.size();
}

Rect Ui::bounds() const {
    return inner.bounds();
}

float Ui::content_top() const {
    return inner.content_top();
}

bool Ui::fullscreen() const {
    return inner.fullscreen();
}

bool Ui::vertical_sync() const {
    return inner.environment_.vertical_sync;
}

double Ui::time() const {
    return inner.input_.time;
}

float Ui::delta() const {
    return inner.input_.delta;
}

std::uint64_t Ui::frame_number() const {
    return inner.frame_number();
}

FrameStats Ui::stats() const {
    return inner.stats();
}

const Environment& Ui::environment() const {
    return inner.environment_;
}

const Input& Ui::input() const {
    return inner.input_;
}

bool Ui::window_focused() const {
    return inner.input_.focused;
}

const Gamepad& Ui::gamepad() const {
    return inner.input_.gamepad;
}

std::optional<Point> Ui::pointer() const {
    return inner.input_.pointer;
}

bool Ui::key_pressed(Key key) const {
    return inner.key_pressed(key);
}

bool Ui::key_pressed_once(Key key) const {
    return inner.key_pressed_once(key);
}

bool Ui::shortcut(Modifiers modifiers, Key key) const {
    return inner.input_.modifiers() == modifiers && key_pressed(key);
}

bool Ui::shortcut_pressed(Shortcut shortcut) const {
    return this->shortcut(shortcut.modifiers, shortcut.key_);
}

bool Ui::key_down(Key key) const {
    return inner.key_down(key);
}

Size Ui::measure(std::string_view text, TextStyle style) {
    return inner.fonts_.measure(text, style);
}

Size Ui::measure_wrapped(std::string_view text, TextStyle style, float width) {
    return inner.fonts_.measure_wrapped(text, style, width);
}

float Ui::paragraph_height(float width, std::string_view text, TextStyle style) {
    return inner.fonts_.measure_wrapped(text, style, width).height;
}

Fonts& Ui::fonts() {
    return inner.fonts_;
}

std::span<const Notice> Ui::notices() const {
    return inner.notices_;
}

bool Ui::modal() const {
    return inner.modal();
}

bool Ui::any_popup() const {
    return !inner.popups.empty();
}

std::optional<std::string_view> Ui::hovered_name() const {
    return inner.hovered_name();
}

std::optional<std::string_view> Ui::pressed_name() const {
    return inner.pressed_name();
}

std::optional<std::string_view> Ui::focused_name() const {
    return inner.focused_name();
}

std::optional<std::string_view> Ui::popup_name() const {
    return inner.popup_name();
}

std::optional<std::string_view> Ui::tooltip_name() const {
    return inner.tooltip_name();
}

bool Ui::is_focused(std::string_view name) const {
    return inner.focused == id(name);
}

bool Ui::tooltips_enabled() const {
    return inner.tooltips_enabled();
}

} // namespace user_interface
