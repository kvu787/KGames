#include "UserInterface/Core/Tree.h"

#include "UserInterface/Core/Interface.h"
#include "UserInterface/Widgets/Widget.h"

#include <algorithm>
#include <format>

namespace user_interface {

TooltipLine TooltipLine::heading(std::string text) {
    return TooltipLine::Heading{std::move(text)};
}

TooltipLine TooltipLine::text(std::string text) {
    return TooltipLine::Text{std::move(text)};
}

TooltipLine TooltipLine::pair(std::string label, std::string pair_value) {
    return TooltipLine::Pair{std::move(label), std::move(pair_value)};
}

TooltipLine TooltipLine::mono(std::string text) {
    return TooltipLine::Mono{std::move(text)};
}

ControlState Look::state() const {
    return hit.state(enabled);
}

ControlState Look::part_state(const Part& part) const {
    Hit part_hit{
        .hovered = hovered_part == part && enabled,
        .held = held_part == part,
        .held_button = held_button,
    };
    return part_hit.state(enabled);
}

bool Look::hovered(const Part& part) const {
    return hovered_part == part;
}

bool Look::held(const Part& part) const {
    return held_part == part;
}

Rect Parts::rect() const {
    return rect_;
}

Id Parts::id() const {
    return id_;
}

std::string_view Parts::name() const {
    return name_;
}

bool Parts::enabled() const {
    return enabled_;
}

Fonts& Parts::fonts() {
    return ui.fonts_;
}

Size Parts::measure(std::string_view text, TextStyle style) {
    return ui.fonts_.measure(text, style);
}

const Theme& Parts::theme() const {
    return ui.theme_;
}

double Parts::time() const {
    return ui.input_.time;
}

Rect Parts::bounds() const {
    return ui.bounds();
}

bool Parts::popup_open() const {
    return ui.popup_open(id_);
}

bool Parts::popup_open_of(Id id) const {
    return ui.popup_open(id);
}

bool Parts::is_focused() const {
    return ui.focused == id_;
}

const Input& Parts::input() const {
    return ui.input_;
}

void Parts::part(Part part, std::string_view suffix, Rect rect, Sense sense, std::string_view kind, std::optional<std::string> label) {
    std::string full_name = std::format("{}/{}", name_, suffix);
    part_on(layer_, std::move(part), full_name, rect, sense, kind, std::move(label));
}

void Parts::part_on(Layer layer, Part part, std::string_view name, Rect rect, Sense sense, std::string_view kind, std::optional<std::string> label) {
    part_on_enabled(layer, std::move(part), name, rect, sense, kind, std::move(label), enabled_);
}

void Parts::part_on_enabled(Layer layer, Part part, std::string_view name, Rect rect, Sense sense, std::string_view kind, std::optional<std::string> label, bool enabled) {
    Rect target_clip = layer > layer_ ? ui.bounds() : clip;
    bool target_enabled = enabled && enabled_;
    bool focusable = focusable_ && sense.focusable;
    ui.targets_.push_back(Target{
        .id = user_interface::id(name),
        .name = std::string(name),
        .label = std::move(label),
        .rect = rect.intersect(target_clip),
        .layer = layer,
        .enabled = target_enabled,
        .focusable = focusable,
        .sense = sense,
        .kind = kind,
        .listed = true,
        .owner = id_,
        .part = std::move(part),
    });
}

void Parts::cover_on(Layer layer, Rect rect) {
    ui.cover_on(layer, rect, id_);
}

Rect Dispatch<void>::rect() const {
    return rect_;
}

Id Dispatch<void>::id() const {
    return id_;
}

std::string_view Dispatch<void>::name() const {
    return name_;
}

bool Dispatch<void>::enabled() const {
    return enabled_;
}

bool Dispatch<void>::focusable() const {
    return focusable_;
}

const Input& Dispatch<void>::input() const {
    return ui.input_;
}

Modifiers Dispatch<void>::modifiers() const {
    return ui.input_.modifiers();
}

double Dispatch<void>::time() const {
    return ui.input_.time;
}

std::optional<Point> Dispatch<void>::pointer() const {
    return ui.input_.pointer;
}

Fonts& Dispatch<void>::fonts() {
    return ui.fonts_;
}

Size Dispatch<void>::measure(std::string_view text, TextStyle style) {
    return ui.fonts_.measure(text, style);
}

Clipboard& Dispatch<void>::clipboard() {
    return ui.clipboard;
}

bool Dispatch<void>::key_pressed(Key key) const {
    return ui.input_.key_pressed(key) && !(key == Key::Escape && ui.escape_consumed);
}

bool Dispatch<void>::key_pressed_once(Key key) const {
    return ui.input_.key_pressed_once(key) && !(key == Key::Escape && ui.escape_consumed);
}

bool Dispatch<void>::key_down(Key key) const {
    return ui.key_down(key);
}

void Dispatch<void>::open_popup(std::string_view name) {
    ui.open_popup(id_, name);
}

void Dispatch<void>::open_popup_of(Id owner, std::string_view name) {
    ui.open_popup(owner, name);
}

void Dispatch<void>::close_popup() {
    ui.close_popup(id_);
}

void Dispatch<void>::close_popup_of(Id owner) {
    ui.close_popup(owner);
}

void Dispatch<void>::close_popups() {
    ui.popups.clear();
}

bool Dispatch<void>::popup_open() const {
    return ui.popup_open(id_);
}

bool Dispatch<void>::popup_open_of(Id owner) const {
    return ui.popup_open(owner);
}

bool Dispatch<void>::popup_on_top() const {
    return ui.popup_on_top(id_);
}

bool Dispatch<void>::popup_on_top_of(Id owner) const {
    return ui.popup_on_top(owner);
}

bool Dispatch<void>::any_popup() const {
    return !ui.popups.empty();
}

void Dispatch<void>::focus() {
    ui.set_focused(id_);
    ui.focus_visible = false;
}

void Dispatch<void>::focus_id(Id id) {
    ui.set_focused(id);
}

void Dispatch<void>::clear_focus() {
    ui.set_focused(std::nullopt);
}

bool Dispatch<void>::focused_takes_enter() const {
    if (!ui.focused) {
        return false;
    }
    auto target = std::ranges::find_if(ui.targets_, [&](const Target& candidate) { return candidate.id == *ui.focused; });
    if (target == ui.targets_.end()) {
        return false;
    }
    bool keys_here = (ui.popups.empty() || target->layer >= Layer::Popup) && (!ui.modal_shown || target->layer >= Layer::Dialog);
    return keys_here && target->enabled && target->sense.keys != Keys::None;
}

bool Dispatch<void>::is_focused() const {
    return ui.focused == id_;
}

bool Dispatch<void>::is_focused_id(Id id) const {
    return ui.focused == id;
}

bool Dispatch<void>::focus_visible() const {
    return ui.focus_visible;
}

void Dispatch<void>::set_focus_visible(bool visible) {
    ui.focus_visible = visible;
}

void Dispatch<void>::set_cursor(Cursor cursor) {
    ui.cursor_ = cursor;
}

void Dispatch<void>::consume_escape() {
    ui.escape_consumed = true;
}

std::optional<std::string_view> Dispatch<void>::hovered_name() const {
    return ui.hovered_name();
}

std::optional<Id> Dispatch<void>::hovered_id() const {
    return ui.hovered;
}

std::optional<std::string_view> Dispatch<void>::focused_name() const {
    return ui.focused_name();
}

Rect Dispatch<void>::bounds() const {
    return ui.bounds();
}

std::vector<Notice>& Dispatch<void>::notices_mut() {
    return ui.notices_;
}

Painter Draw::painter() {
    return Painter{ui.draw, ui.fonts_, ui.theme_};
}

const Theme& Draw::theme() const {
    return ui.theme_;
}

const Look& Draw::look() const {
    return look_;
}

ControlState Draw::state() const {
    return look_.state();
}

Rect Draw::rect() const {
    return rect_;
}

Id Draw::id() const {
    return id_;
}

std::string_view Draw::name() const {
    return name_;
}

bool Draw::enabled() const {
    return look_.enabled;
}

bool Draw::focused() const {
    return look_.focused;
}

bool Draw::focus_shown() const {
    return look_.focused && look_.focus_visible;
}

bool Draw::focus_visible() const {
    return ui.focus_visible;
}

Layer Draw::layer() const {
    return layer_;
}

double Draw::time() const {
    return ui.input_.time;
}

Rect Draw::bounds() const {
    return ui.bounds();
}

const Input& Draw::input() const {
    return ui.input_;
}

Fonts& Draw::fonts() {
    return ui.fonts_;
}

Size Draw::measure(std::string_view text, TextStyle style) {
    return ui.fonts_.measure(text, style);
}

bool Draw::popup_open() const {
    return ui.popup_open(id_);
}

bool Draw::popup_open_of(Id id) const {
    return ui.popup_open(id);
}

bool Draw::popup_on_top_of(Id id) const {
    return ui.popup_on_top(id);
}

bool Draw::is_focused_id(Id id) const {
    return ui.focused == id;
}

const Target* Draw::hovered_target() const {
    return ui.hovered_target();
}

const Target* Draw::held_target() const {
    return ui.held_target();
}

const Environment& Draw::environment() const {
    return ui.environment_;
}

bool Draw::window_focused() const {
    return ui.input_.focused;
}

FrameStats Draw::stats() const {
    return ui.stats();
}

std::span<const Notice> Draw::notices() const {
    return ui.notices_;
}

void Draw::focus_visual_if(bool focused, Rect rect, float radius) {
    focus_visual_around(focused, rect, radius, controls::FocusVisualMargin);
}

void Draw::focus_visual_around(bool focused, Rect rect, float radius, Insets margin) {
    if (focused && ui.focus_visible) {
        Painter visual = painter();
        const Theme& theme = visual.theme();
        focus_visual(visual, theme, rect, radius, margin);
    }
}

size_t Draw::layer_len() const {
    return ui.draw.layer_len();
}

void Draw::move_to_back(size_t first, size_t middle) {
    ui.draw.move_to_back(first, middle);
}

} // namespace user_interface
