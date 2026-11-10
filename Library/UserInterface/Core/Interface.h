// The interface's frame machinery. A frame: the input is found on the tree
// the user saw, as the frame before this one drew it, and each widget there
// says what the input did to it, as messages; the program's own messages
// are added; the messages are applied, and those they send in turn, until
// none are left; the program describes the frame's tree; the tree is laid
// out, its targets found, the pointer placed on the topmost of them, and
// the tree drawn. So every widget shows all that the input did, whatever
// order the tree lists them in, on the frame of the input.
//
// Widgets are named, like "Volume" or "Devices/3". A name is what a widget is
// known by from one frame to the next, for what it keeps, like a caret, and
// it's what scripts and tests point at. Every widget that a frame shows is a
// target with its area, and so is each part of one that's a target of its
// own, like a list's row. The pointer is on the topmost target under it, so
// a popup over a card takes the pointer from the card's widgets, and a
// dialog takes it from everything under it.

#pragma once

#include "Base/Panic.h"
#include "Base/VecDeque.h"
#include "UserInterface/Application/Performance.h"
#include "UserInterface/Application/Program.h"
#include "UserInterface/Core/Interaction.h"
#include "UserInterface/Core/Tree.h"
#include "UserInterface/Design/Theme.h"
#include "UserInterface/Drawing/Geometry.h"
#include "UserInterface/Drawing/Painter.h"
#include "UserInterface/Drawing/Text.h"
#include "UserInterface/Input/Devices.h"
#include "UserInterface/Input/Events.h"
#include "UserInterface/Platform/Clipboard.h"
#include "UserInterface/Widgets/Menu.h"
#include "UserInterface/Widgets/ToolTip.h"
#include "Json/Json.h"

#include <algorithm>
#include <array>
#include <chrono>
#include <cstddef>
#include <cstdint>
#include <format>
#include <iterator>
#include <memory>
#include <optional>
#include <span>
#include <string>
#include <string_view>
#include <tuple>
#include <typeindex>
#include <unordered_map>
#include <unordered_set>
#include <utility>
#include <vector>

namespace user_interface {

// What the program runs on, for its bars and its title.
struct Environment {
    // Like "NVIDIA GeForce RTX 5090 Laptop GPU".
    std::string gpu;
    // The display's identifier, like "CVVNY2", and its model.
    std::string display;
    std::string display_name;
    // Whether frames may tear, and whether they wait for the screen, which
    // F10 switches.
    bool tearing = false;
    bool vertical_sync = false;
    // Whether input comes from a script instead of the user.
    bool scripted = false;
};

// The library's own: what a program asks of its window.
enum class Request : std::uint8_t {
    ToggleFullscreen,
    ToggleVerticalSync,
    Close,
};

// The frame rate, as the PerfBar along the top of the window measures it:
// over the last second, refreshed 10 times a second.
struct FrameStats {
    float frames_per_second = 0.0f;
    // Milliseconds.
    float frame_time = 0.0f;
};

// What the parts of the last frame cost the CPU, in milliseconds: the input
// on the tree the user saw, the messages, the view, its layout, and its
// drawing. For benchmarks.
struct FramePhases {
    double input = 0.0;
    double update = 0.0;
    double view = 0.0;
    double layout = 0.0;
    double draw = 0.0;
};

// The library's own: an open popup, like a combo box's list or a menu, by
// the widget that owns it. Popups stack: a submenu opens over its menu.
struct Popup {
    Id owner;
    std::string name;
};

// What only the interface's own code uses, here and in Interface.cpp.
namespace interface_detail {

struct TooltipState {
    std::optional<Id> candidate;
    double since = 0.0;
    Point anchor;
    bool dismissed = false;
};

// The most messages a frame applies, counting those that messages send in
// turn, before it gives up on a chain that never ends.
inline constexpr size_t MostMessages = 10'000;

// The targets that decide how elements look this frame: the one the pointer
// is on, the one with a press, and the one with focus, each as its owner and
// which part of the owner it is.
struct LookSources {
    std::optional<std::pair<Id, Part>> hovered;
    std::optional<std::tuple<Id, Part, MouseButton>> active;
    // Whether the keys reach the focused target: no popup or dialog over it
    // takes them.
    std::optional<std::tuple<Id, Part, bool>> focused;

    // How an element looks this frame, from what the pointer and the
    // keyboard are doing to its targets.
    Look look(Id element, bool enabled, bool focus_visible) const;
};

// What a widget keeps between frames is kept by its id and the type.
struct MemoryKey {
    Id id;
    std::type_index type;

    bool operator==(const MemoryKey&) const = default;
};

struct MemoryKeyHash {
    size_t operator()(const MemoryKey& key) const noexcept;
};

// The id of a cover over the area, which is no widget: a hash of where it
// is, which no name hashes to.
Id cover_id(Rect rect);

// The focus's owner among the targets, with its part and whether the keys
// reach it under the open popups and dialogs.
std::optional<std::tuple<Id, Part, bool>> owner_of_focus(std::span<const Target> targets, std::span<const Popup> popups, bool modal_shown, std::optional<Id> focused);

// An element of the tree the user saw, with its enabling, whether it takes
// focus, and its layer.
template <class M> struct Collected {
    const Element<M>* element;
    bool enabled;
    bool focusable;
    Layer layer;
};

// Every element with its enabling, whether it takes focus, and its layer,
// in drawing order.
template <class M> void collect(const std::vector<Element<M>>& elements, bool enabled, Layer layer, std::vector<Collected<M>>& out) {
    for (const Element<M>& element : elements) {
        bool element_enabled = enabled && element.enabled_;
        // The element's own flag, as layout takes it.
        bool focusable = element.focusable_;
        Layer element_layer = std::max(layer, element.widget->layer());
        out.push_back(Collected<M>{&element, element_enabled, focusable, element_layer});
        collect(element.children_, element_enabled, element_layer, out);
    }
}

} // namespace interface_detail

class Interface {
public:
    // The library's own, as far as `focus_leavers`.
    Theme theme_;
    Fonts fonts_;
    DrawList draw;
    Input input_;
    // The targets of the frame that's laid out, which are the frame the
    // user saw, that the next frame's input is on, until that frame's
    // layout.
    std::vector<Target> targets_;
    std::optional<Id> hovered;
    std::optional<Id> focused;
    bool focus_visible = false;
    std::vector<Popup> popups;
    // Whether the frame the user saw had a dialog, which then takes all
    // input.
    bool modal_shown = false;
    bool escape_consumed = false;
    // The tooltip that this frame showed: its widget's name, and its area.
    std::optional<std::pair<std::string, Rect>> shown_tooltip;
    Clipboard clipboard;
    Cursor cursor_ = Cursor::Arrow;
    Environment environment_;
    std::vector<Request> requests;
    // The notices under the app bar.
    std::vector<Notice> notices_;
    // What the PerfBar along the top of the window shows, which every
    // program's window has, and overlays don't.
    std::optional<performance::Performance> performance;
    // How many messages the last frame applied, and whether it hit the cap.
    size_t messages_applied = 0;
    // Widgets that lost keyboard focus and haven't been told, so a number
    // box commits its draft whoever took the focus, and in the same frame.
    std::vector<Id> focus_leavers;

    Interface(Fonts fonts, ThemeKind theme, Clipboard clipboard);
    Interface(const Interface&) = delete;
    Interface& operator=(const Interface&) = delete;
    Interface(Interface&&) = default;
    Interface& operator=(Interface&&) = default;

    // The library's own: runs a frame of the program: the input on the tree
    // the user saw, the messages, the new tree, and its drawing. `shown` is
    // that tree, which this leaves as the new one. `sent` are messages from
    // outside, like a test's, applied before the input's.
    template <IsProgram P>
    void frame(P& program, std::optional<Tree<typename P::Message>>& shown, std::span<const Event> events, std::vector<typename P::Message> sent, double time, Size size,
               bool fullscreen);

    // What the parts of the last frame cost the CPU.
    FramePhases phases() const;

    // The library's own: runs a frame of a tree that isn't a program's, like
    // an overlay's or the unsupported setup's: the interface's own keys, and
    // drawing.
    template <class M> void frame_of(const Tree<M>& tree, std::span<const Event> events, double time, Size size, bool fullscreen);

    // The library's own: moves keyboard focus, remembering who lost it:
    // dispatch tells the loser, in the same frame, so a draft commits
    // whoever took the focus.
    void set_focused(std::optional<Id> to);

    // The library's own: the target that the pointer is on, and the one
    // with a press, or null.
    const Target* hovered_target() const;

    const Target* held_target() const;

    // The theme, whose roles color everything: the one this frame draws in,
    // as the frame's toggles leave it.
    const Theme& theme() const;

    // Switches between the light and dark themes, as F9 does, for this
    // frame's view on.
    void toggle_theme();

    // The window's client area.
    Size size() const;

    // The part of the client area that the program has: all of it, under the
    // PerfBar along its top.
    Rect bounds() const;

    // Where the program's part of the window starts: under the PerfBar,
    // tokens::PerfBarHeight pixels down, or at the top in an overlay.
    float content_top() const;

    bool fullscreen() const;

    // Whether frames wait for the screen's refresh.
    bool vertical_sync() const;

    // Seconds since the program started, on the frames' clock, which runs
    // at a fixed rate for scripts and tests.
    double time() const;

    std::uint64_t frame_number() const;

    // The frame rate, as the PerfBar measures it, so that a program's own
    // card agrees with the PerfBar. Zero without it.
    FrameStats stats() const;

    const Environment& environment() const;

    const Input& input() const;

    const Gamepad& gamepad() const;

    bool tooltips_enabled() const;

    // Whether the key was pressed this frame, counting a held key's repeats,
    // unless the widget with keyboard focus takes it, as a text field takes
    // letters, or a dialog is open over the page.
    bool key_pressed(Key key) const;

    bool key_pressed_once(Key key) const;

    bool key_down(Key key) const;

    // The library's own.
    bool is_captured(Key key) const;

    // The library's own.
    Painter painter();

    Size measure(std::string_view text, TextStyle style);

    Fonts& fonts();

    Cursor cursor() const;

    // The library's own: covers what's under the area, like a card covers
    // the 3D view under it, without being a target that scripts see.
    void cover_on(Layer layer, Rect rect, Id owner);

    // How many instances the frame drew: each shape, glyph, line, arc,
    // polygon and shadow is one, which the renderer draws in one call. For
    // benchmarks.
    size_t instance_count() const;

    // The library's own: opens the popup that the widget with the id owns,
    // like a combo box's list. A popup whose owner is in an open popup opens
    // over it, like a submenu; any other closes the open ones first.
    void open_popup(Id owner, std::string_view name);

    // The library's own: closes the popup that the widget owns, and those
    // over it.
    void close_popup(Id owner);

    // The library's own.
    bool popup_open(Id owner) const;

    // The library's own: whether the widget's popup is the top one, which
    // takes the keys.
    bool popup_on_top(Id owner) const;

    bool any_popup() const;

    // Whether a dialog is open, which takes the input from the page.
    bool modal() const;

    // The library's own: state that a widget keeps from frame to frame, like
    // a text field's caret, by the widget's id: a T, which starts as T's
    // default. The reference stays good, whatever else is kept.
    template <class T> T& memory(Id id) {
        return interface_detail::memory<T>(*this, id);
    }

    // The notices under the app bar.
    std::span<const Notice> notices() const;

    // The targets of the last frame that's laid out.
    std::span<const Target> targets() const;

    // The last frame's listed target with the name: the first of them, as
    // the state lists it, if the name is used twice. Null if there's none.
    const Target* target(std::string_view name) const;

    // The names of the widgets that the input is on, as the interface's
    // state has them.
    std::optional<std::string_view> hovered_name() const;

    std::optional<std::string_view> pressed_name() const;

    std::optional<std::string_view> focused_name() const;

    std::optional<std::string_view> popup_name() const;

    std::optional<std::string_view> tooltip_name() const;

    // What scripts save and check of the interface: what the input is on,
    // and where each widget is.
    json::Value state() const;

    // The library's own, which Widgets/Shell.cpp defines: draws the PerfBar
    // along the top of the window: the frames a second, the input latency's
    // mean from GameInput's readings to the screen, how frames reach the
    // screen and whether they wait for vertical sync, in sections that |s
    // separate, one after another from the left, set in Cascadia Mono, in
    // fields whose widths never change, so that every label and value stays
    // in the same columns. It's pure black and white: black on white in the
    // light theme, and white on black in the dark one. The frame draws it
    // every frame; programs don't.
    void perf_bar();

private:
    friend std::unique_ptr<interface_detail::Kept>& interface_detail::kept(Interface& ui, Id id, std::type_index type);

    // Starts on the frame's input: where the pointer is, by the targets the
    // user saw, and what the keys do before any widget hears them.
    void begin_input();

    // Finds what the input did to each target the user saw, and tells each
    // widget of the tree what the input did to it, which it turns into
    // messages.
    template <class M> void dispatch(const Tree<M>& tree, std::vector<M>& messages);

    // What the input did to a target this frame. A press acts on the frame
    // that the button goes down: a left press activates the target, and a
    // right press asks for its context menu, whatever the pointer does
    // before the button comes up.
    Hit hit(const Target& target);

    // Moves keyboard focus as the frame's input asks, after every target's
    // hit is found and before any widget hears the input, so that each
    // widget knows whether it has focus as the frame ends.
    void move_focus();

    // Ends the frame's input: what no widget took is let go of, and the
    // interface's own keys.
    void end_input();

    // Lays the frame's tree out: finds every target, and which layer each
    // is on, and places the pointer on the topmost of them, for the look.
    template <class M> void layout(const Tree<M>& tree);

    template <class M> void layout_elements(const std::vector<Element<M>>& elements, bool enabled, Layer layer, Rect clip);

    // Draws the frame's tree, each element after what it's over, and the
    // tooltip and the PerfBar over all of it.
    template <class M> void draw_tree(const Tree<M>& tree);

    template <class M>
    void draw_elements(const std::vector<Element<M>>& elements, bool enabled, Layer layer, Rect clip, const interface_detail::LookSources& sources,
                       const std::optional<std::tuple<Id, Part, std::string, Point>>& due, std::optional<std::tuple<Tooltip, std::string, Point>>& tooltip_found);

    // What every element's look this frame comes from: the targets that the
    // pointer is on, that have a press, and that have focus, found once.
    interface_detail::LookSources look_sources() const;

    // The topmost of the targets under the pointer, of the dialog's layer or
    // above while a dialog is open, which leaves nothing under it to be on.
    std::optional<Id> topmost(std::span<const Target> targets, bool modal) const;

    // The target that the pointer is on, with the topmost target under it:
    // that one, or only the target with a press while there is one.
    std::optional<Id> under_pointer(std::optional<Id> topmost) const;

    // Whether the target under the pointer takes the wheel itself, so that
    // what it's in shouldn't scroll.
    bool hovered_takes_wheel(std::span<const Target> targets) const;

    // Where the pointer rested on the hovered target, if its tooltip should
    // show now.
    std::optional<Point> tooltip_due() const;

    std::optional<std::string_view> listed_name(std::optional<Id> id) const;

    // The theme that toggle_theme asked for this frame, which the frame's
    // view is drawn in: what `theme_` says until then.
    std::optional<Theme> next_theme;
    Size size_;
    bool fullscreen_ = false;
    // The target with a press, and the button.
    std::optional<std::pair<Id, MouseButton>> active;
    // The keys that the target with focus takes.
    Keys captured = Keys::None;
    // Whether the frame laid out has a dialog.
    bool modal_ = false;
    // Presses that only closed a popup, which no widget may take.
    std::array<bool, 3> press_consumed{};
    interface_detail::TooltipState tooltip;
    bool tooltips_enabled_ = true;
    std::unordered_map<interface_detail::MemoryKey, std::unique_ptr<interface_detail::Kept>, interface_detail::MemoryKeyHash> memory_;
    std::uint64_t frame_ = 0;
    FramePhases phases_;
};

// What a program's `view` and `input` see of the interface: the frame as it
// stands, which they read and don't change, and the fonts to measure text
// with.
class Ui {
public:
    // The library's own.
    Interface& inner;

    const Theme& theme() const;

    Size size() const;

    Rect bounds() const;

    float content_top() const;

    bool fullscreen() const;

    bool vertical_sync() const;

    double time() const;

    // Seconds since the last frame.
    float delta() const;

    std::uint64_t frame_number() const;

    FrameStats stats() const;

    const Environment& environment() const;

    // All of the frame's input, for programs that read it themselves, like
    // the keys held for driving.
    const Input& input() const;

    // Whether the window has focus. Without it there's no input at all.
    bool window_focused() const;

    const Gamepad& gamepad() const;

    std::optional<Point> pointer() const;

    // Whether the key was pressed this frame, counting a held key's repeats,
    // unless the widget with keyboard focus takes it, as a text field takes
    // letters, or a dialog is open over the page.
    bool key_pressed(Key key) const;

    bool key_pressed_once(Key key) const;

    // Whether the key was pressed with exactly these modifiers, like Ctrl+S.
    bool shortcut(Modifiers modifiers, Key key) const;

    bool shortcut_pressed(Shortcut shortcut) const;

    bool key_down(Key key) const;

    Size measure(std::string_view text, TextStyle style);

    Size measure_wrapped(std::string_view text, TextStyle style, float width);

    // How tall the text is, wrapped at the width.
    float paragraph_height(float width, std::string_view text, TextStyle style);

    Fonts& fonts();

    std::span<const Notice> notices() const;

    // Whether a dialog is open, which takes the input from the page.
    bool modal() const;

    bool any_popup() const;

    // The names of the widgets that the input is on.
    std::optional<std::string_view> hovered_name() const;

    std::optional<std::string_view> pressed_name() const;

    std::optional<std::string_view> focused_name() const;

    std::optional<std::string_view> popup_name() const;

    std::optional<std::string_view> tooltip_name() const;

    bool is_focused(std::string_view name) const;

    bool tooltips_enabled() const;

    // What a widget keeps between frames, read for laying the frame out,
    // like how far a scroll area is scrolled: a copy of it.
    template <class T> T memory(std::string_view name) {
        return inner.memory<T>(id(name));
    }
};

// What a program's `update` works with: the interface's state, and where
// the messages it sends go.
template <class M> class Context {
public:
    // The library's own.
    Interface& inner;
    base::VecDeque<M>& queue;

    // Sends a message, which is applied after those before it, this frame.
    void send(M message) {
        queue.push_back(std::move(message));
    }

    const Theme& theme() const {
        return inner.theme();
    }

    // Switches between the light and dark themes, as F9 does, from this
    // frame's view on.
    void toggle_theme() {
        inner.toggle_theme();
    }

    void toggle_fullscreen() {
        inner.requests.push_back(Request::ToggleFullscreen);
    }

    // Switches vertical sync, as F10 does, from the next frame on.
    void toggle_vertical_sync() {
        inner.environment_.vertical_sync = !inner.environment_.vertical_sync;
        inner.requests.push_back(Request::ToggleVerticalSync);
    }

    // Closes the window, whatever Program::may_close says.
    void close() {
        inner.requests.push_back(Request::Close);
    }

    double time() const {
        return inner.input_.time;
    }

    float delta() const {
        return inner.input_.delta;
    }

    bool fullscreen() const {
        return inner.fullscreen();
    }

    bool vertical_sync() const {
        return inner.environment_.vertical_sync;
    }

    const Environment& environment() const {
        return inner.environment_;
    }

    const Input& input() const {
        return inner.input_;
    }

    bool window_focused() const {
        return inner.input_.focused;
    }

    Size size() const {
        return inner.size();
    }

    Rect bounds() const {
        return inner.bounds();
    }

    // Shows a notice under the app bar.
    void notify(Notice notice) {
        inner.notices_.push_back(std::move(notice));
    }

    std::span<const Notice> notices() const {
        return inner.notices_;
    }

    void remove_notice(size_t index) {
        if (index < inner.notices_.size()) {
            inner.notices_.erase(inner.notices_.begin() + static_cast<std::ptrdiff_t>(index));
        }
    }

    // Gives keyboard focus to the widget with the name.
    void focus(std::string_view name) {
        inner.set_focused(id(name));
    }

    void clear_focus() {
        inner.set_focused(std::nullopt);
    }

    bool is_focused(std::string_view name) const {
        return inner.focused == id(name);
    }

    std::optional<std::string_view> hovered_name() const {
        return inner.hovered_name();
    }

    std::optional<std::string_view> pressed_name() const {
        return inner.pressed_name();
    }

    std::optional<std::string_view> focused_name() const {
        return inner.focused_name();
    }

    std::optional<std::string_view> popup_name() const {
        return inner.popup_name();
    }

    std::optional<std::string_view> tooltip_name() const {
        return inner.tooltip_name();
    }

    // Whether the frame the user saw had a dialog, which took its input.
    bool modal() const {
        return inner.modal_shown;
    }

    void close_popups() {
        inner.popups.clear();
    }
};

namespace interface_detail {

// Tells the element's widget what the input did to it: with where its
// messages go, if it's a widget that sends them.
template <class M> void interact(const Element<M>& element, const Interaction& interaction, Dispatch<M>& ctx) {
    if (element.sends) {
        static_cast<const Widget<M>&>(*element.widget).interact(interaction, ctx);
    } else {
        element.widget->interact(interaction, ctx);
    }
}

// The element's name, or none.
template <class M> std::string_view name_of(const Element<M>& element) {
    return element.name_ ? std::string_view(*element.name_) : std::string_view();
}

} // namespace interface_detail

template <IsProgram P>
void Interface::frame(P& program, std::optional<Tree<typename P::Message>>& shown, std::span<const Event> events, std::vector<typename P::Message> sent, double time, Size size,
                      bool fullscreen) {
    using Message = typename P::Message;
    frame_ += 1;
    size_ = size;
    fullscreen_ = fullscreen;
    input_.begin(time, events);
    fonts_.begin_frame();
    if (performance) {
        performance->frame_started(time);
        performance->update(time);
    }

    // The input, on the frame the user saw.
    std::chrono::steady_clock::time_point started = std::chrono::steady_clock::now();
    begin_input();
    std::vector<Message> messages = std::move(sent);
    if (shown) {
        dispatch(*shown, messages);
    }
    end_input();
    {
        Ui ui{*this};
        std::vector<Message> own = program.input(std::as_const(ui));
        messages.insert(messages.end(), std::make_move_iterator(own.begin()), std::make_move_iterator(own.end()));
    }
    FramePhases spent;
    std::chrono::steady_clock::time_point mark = started;
    auto lap = [&](double& phase) {
        std::chrono::steady_clock::time_point now = std::chrono::steady_clock::now();
        phase = std::chrono::duration<double>(now - mark).count() * 1000.0;
        mark = now;
    };
    lap(spent.input);

    // The messages, and those they send, until none are left. A chain
    // that doesn't settle is a bug, and stops the program rather than
    // hiding or dropping what's queued behind it.
    base::VecDeque<Message> queue(std::move(messages));
    size_t applied = 0;
    while (std::optional<Message> message = queue.pop_front()) {
        if (!(applied < interface_detail::MostMessages)) {
            base::panic(std::format("the frame stopped after {} messages: a message sends messages without end", interface_detail::MostMessages));
        }
        applied += 1;
        Context<Message> context{*this, queue};
        program.update(std::move(*message), context);
    }
    messages_applied = applied;
    lap(spent.update);

    // The frame's tree, laid out and drawn.
    if (next_theme) {
        theme_ = *next_theme;
        next_theme.reset();
    }
    Ui ui{*this};
    Tree<Message> tree = program.view(ui);
    tooltips_enabled_ = tree.tooltips;
    lap(spent.view);
    layout(tree);
    lap(spent.layout);
    draw_tree(tree);
    lap(spent.draw);
    phases_ = spent;
    shown = std::move(tree);
}

template <class M> void Interface::frame_of(const Tree<M>& tree, std::span<const Event> events, double time, Size size, bool fullscreen) {
    frame_ += 1;
    size_ = size;
    fullscreen_ = fullscreen;
    input_.begin(time, events);
    fonts_.begin_frame();
    begin_input();
    end_input();
    if (next_theme) {
        theme_ = *next_theme;
        next_theme.reset();
    }
    layout(tree);
    draw_tree(tree);
}

template <class M> void Interface::dispatch(const Tree<M>& tree, std::vector<M>& messages) {
    // The targets are out of the interface while each one's hit is found,
    // as they are in Rust, and back before any widget hears of it.
    std::vector<Target> shown = std::exchange(targets_, {});
    std::unordered_map<Id, Interaction> interactions;
    for (const Target& target : shown) {
        Hit target_hit = hit(target);
        bool anything = target_hit.hovered || target_hit.pressed || target_hit.held || target_hit.released || target_hit.activated || target_hit.secondary_pressed ||
                        target_hit.middle_pressed || target_hit.wheel != Point::Zero;
        if (!anything) {
            continue;
        }
        Interaction& interaction = interactions[target.owner];
        interaction.hits.emplace_back(target.part, target_hit);
    }
    // The wheel over a widget whose target under the pointer doesn't
    // take it, for what scrolls.
    bool wheel_free = input_.wheel != Point::Zero && !hovered_takes_wheel(shown);
    std::optional<Layer> hovered_layer;
    if (hovered) {
        for (auto target = shown.rbegin(); target != shown.rend(); ++target) {
            if (target->id == *hovered) {
                hovered_layer = target->layer;
                break;
            }
        }
    }
    targets_ = std::move(shown);
    move_focus();

    // The widget with keyboard focus hears the keys, if no popup or
    // dialog over it takes them, re-found whenever a widget in the loop
    // moves the focus; and every widget that lost focus hears that, like
    // a number box that takes what was typed, however it lost it.
    std::optional<Id> known_focus = focused;
    std::optional<std::tuple<Id, Part, bool>> focused_owner = interface_detail::owner_of_focus(targets_, popups, modal_shown, known_focus);

    std::optional<Point> pointer = input_.pointer;
    Point pointer_delta = input_.pointer_delta;
    std::vector<interface_detail::Collected<M>> elements;
    interface_detail::collect(tree.elements, true, Layer::Base, elements);
    // The wheel that nothing under the pointer takes goes to the deepest
    // widget there that scrolls, and only to it, so one notch never
    // scrolls a list and what it's in together.
    std::optional<Id> wheel_target;
    if (wheel_free) {
        for (auto collected = elements.rbegin(); collected != elements.rend(); ++collected) {
            const Element<M>& element = *collected->element;
            if (element.id && element.widget->takes_free_wheel() && pointer && element.rect_.contains(*pointer) && (!hovered_layer || *hovered_layer <= collected->layer)) {
                wheel_target = element.id;
                break;
            }
        }
    }
    for (const interface_detail::Collected<M>& collected : elements) {
        const Element<M>& element = *collected.element;
        if (!element.id) {
            continue;
        }
        Id element_id = *element.id;
        if (focused != known_focus) {
            known_focus = focused;
            focused_owner = interface_detail::owner_of_focus(targets_, popups, modal_shown, known_focus);
        }
        Interaction interaction;
        if (auto found = interactions.find(element_id); found != interactions.end()) {
            interaction = std::move(found->second);
            interactions.erase(found);
        }
        bool has_focus = false;
        std::optional<Part> focused_part;
        if (focused_owner && std::get<0>(*focused_owner) == element_id) {
            has_focus = std::get<2>(*focused_owner);
            focused_part = std::get<1>(*focused_owner);
        }
        bool wheel_over = wheel_target == element_id;
        bool leaving = std::ranges::find(focus_leavers, element_id) != focus_leavers.end();
        // A widget with an open popup, like a right-clicked context
        // menu's, hears the keys that steer it even while the pointer is
        // elsewhere.
        bool has_popup = std::ranges::any_of(popups, [&](const Popup& popup) { return popup.owner == element_id; });
        if (interaction.hits.empty() && !has_focus && !wheel_over && !leaving && !has_popup && !element.widget->always_interacts()) {
            continue;
        }
        interaction.enabled = collected.enabled;
        interaction.focused = has_focus;
        bool on_it = std::ranges::any_of(interaction.hits, [](const std::pair<Part, Hit>& entry) { return entry.second.hovered || entry.second.held; });
        interaction.pointer = on_it ? pointer : std::nullopt;
        interaction.drag_delta = interaction.any_held() ? pointer_delta : Point::Zero;
        if (wheel_over) {
            interaction.wheel_over = input_.wheel;
        }
        interaction.focused_part = std::move(focused_part);
        Dispatch<M> ctx{Dispatch<void>{*this, element_id, interface_detail::name_of(element), element.rect_, collected.enabled, collected.focusable}, messages};
        interface_detail::interact(element, interaction, ctx);
        if (element.context_menu_) {
            interact_context_menu(interaction, ctx, *element.context_menu_);
        }
        if (leaving) {
            std::erase(focus_leavers, element_id);
        }
    }

    // A widget whose focus was taken by one later in the loop was passed
    // before it lost it: tell it now, so its draft still commits this
    // frame. A told widget may move focus again, so go around, a little.
    int rounds = 0;
    while (!focus_leavers.empty() && rounds < 4) {
        rounds += 1;
        std::vector<Id> pending = std::exchange(focus_leavers, {});
        for (const interface_detail::Collected<M>& collected : elements) {
            const Element<M>& element = *collected.element;
            if (!element.id) {
                continue;
            }
            Id element_id = *element.id;
            if (std::ranges::find(pending, element_id) == pending.end()) {
                continue;
            }
            Interaction interaction;
            interaction.enabled = collected.enabled;
            Dispatch<M> ctx{Dispatch<void>{*this, element_id, interface_detail::name_of(element), element.rect_, collected.enabled, collected.focusable}, messages};
            interface_detail::interact(element, interaction, ctx);
        }
    }
    focus_leavers.clear();
}

template <class M> void Interface::layout(const Tree<M>& tree) {
    targets_.clear();
    modal_ = false;
    Rect area = bounds();
    // The PerfBar along the top covers what's under it.
    if (performance) {
        cover_on(Layer::Base, Rect(0.0f, 0.0f, size_.width, content_top()), id("PerfBar"));
    }
    layout_elements(tree.elements, true, Layer::Base, area);

    // A target that's gone, disabled, or no longer focusable can't keep
    // focus, a press, or a popup.
    if (focused && !std::ranges::any_of(targets_, [&](const Target& target) { return target.id == *focused && target.enabled && target.focusable; })) {
        set_focused(std::nullopt);
    }
    if (active && !std::ranges::any_of(targets_, [&](const Target& target) { return target.id == active->first; })) {
        active.reset();
    }
    if (!popups.empty()) {
        // A popup stays only while its owner still exists AND still lays
        // out popup content, so a widget that changes kind under a kept
        // name can't leave a stale popup capturing the keys. The owner
        // may be a part, like a menu bar's menu, whose content the
        // part's element lays out.
        std::unordered_set<Id> popup_owners;
        for (const Target& target : targets_) {
            if (target.layer >= Layer::Popup) {
                popup_owners.insert(target.owner);
            }
        }
        std::erase_if(popups, [&](const Popup& popup) {
            return !std::ranges::any_of(targets_, [&](const Target& target) { return target.id == popup.owner && popup_owners.contains(target.owner); });
        });
    }

    // The pointer, on the frame as it's drawn, for the look of what's
    // under it.
    std::optional<Id> top = topmost(targets_, modal_);
    hovered = under_pointer(top);
    modal_shown = modal_;

    // Tooltips show once the pointer rests on a widget.
    if (hovered != tooltip.candidate) {
        tooltip = interface_detail::TooltipState{
            .candidate = hovered,
            .since = input_.time,
            .anchor = input_.pointer.value_or(Point()),
            .dismissed = false,
        };
    }
}

template <class M> void Interface::layout_elements(const std::vector<Element<M>>& elements, bool enabled, Layer layer, Rect clip) {
    for (const Element<M>& element : elements) {
        bool element_enabled = enabled && element.enabled_;
        // Each element's own flag, not the ancestors', so that a text
        // field opts back in inside a card that keeps focus off.
        bool focusable = element.focusable_;
        Layer element_layer = std::max(layer, element.widget->layer());
        if (element_layer == Layer::Dialog) {
            modal_ = true;
        }
        Rect element_clip = element.widget->layer() > Layer::Base ? bounds() : clip;
        Id element_id = element.id ? *element.id : interface_detail::cover_id(element.rect_);
        std::optional<Sense> sense = element.widget->sense();
        if (element.id && sense) {
            std::optional<std::string> label = element.label_ ? element.label_ : element.widget->label();
            targets_.push_back(Target{
                .id = *element.id,
                .name = element.name_ ? *element.name_ : std::string(),
                .label = std::move(label),
                .rect = element.rect_.intersect(element_clip),
                .layer = element_layer,
                .enabled = element_enabled,
                .focusable = focusable && sense->focusable,
                .sense = *sense,
                .kind = element.widget->kind(),
                .listed = true,
                .owner = *element.id,
                .part = Part::Whole{},
            });
        } else if (element.widget->covers()) {
            cover_on(element_layer, element.rect_.intersect(element_clip), element_id);
        }
        Parts parts{*this, element_id, interface_detail::name_of(element), element.rect_, element_enabled, focusable, element_layer, element_clip};
        element.widget->parts(parts);
        if (element.context_menu_) {
            parts_context_menu(parts, *element.context_menu_);
        }
        std::optional<Rect> own = element.widget->children_clip(parts);
        Rect children_clip = own ? own->intersect(element_clip) : element_clip;
        layout_elements(element.children_, element_enabled, element_layer, children_clip);
    }
}

template <class M> void Interface::draw_tree(const Tree<M>& tree) {
    draw.clear(size_);
    shown_tooltip.reset();
    Rect area = bounds();
    // The tooltip due, if one is: the widget it's of, and which part.
    std::optional<std::tuple<Id, Part, std::string, Point>> due;
    if (std::optional<Point> anchor = tooltip_due()) {
        for (const Target& target : targets_) {
            if (hovered == target.id) {
                due.emplace(target.owner, target.part, target.name, *anchor);
                break;
            }
        }
    }
    std::optional<std::tuple<Tooltip, std::string, Point>> tooltip_found;
    interface_detail::LookSources sources = look_sources();
    draw_elements(tree.elements, true, Layer::Base, area, sources, due, tooltip_found);
    if (tooltip_found) {
        draw_tooltip(*this, std::get<1>(*tooltip_found), std::get<0>(*tooltip_found), std::get<2>(*tooltip_found));
    }
    if (performance) {
        perf_bar();
    }
}

template <class M>
void Interface::draw_elements(const std::vector<Element<M>>& elements, bool enabled, Layer layer, Rect clip, const interface_detail::LookSources& sources,
                              const std::optional<std::tuple<Id, Part, std::string, Point>>& due, std::optional<std::tuple<Tooltip, std::string, Point>>& tooltip_found) {
    for (const Element<M>& element : elements) {
        bool element_enabled = enabled && element.enabled_;
        Layer element_layer = std::max(layer, element.widget->layer());
        Rect element_clip = element.widget->layer() > Layer::Base ? bounds() : clip;
        Id element_id = element.id ? *element.id : interface_detail::cover_id(element.rect_);
        Look look = sources.look(element_id, element_enabled, focus_visible);
        draw.set_layer(element_layer);
        draw.push_clip(element_clip, true);
        Draw ctx{*this, element_id, interface_detail::name_of(element), element.rect_, std::move(look), element_layer};
        element.widget->draw(ctx);
        if (element.context_menu_ && popup_open(element_id)) {
            Draw menu_ctx{*this, element_id, interface_detail::name_of(element), element.rect_, Look{}, element_layer};
            draw_context_menu(menu_ctx, *element.context_menu_);
        }
        if (due && std::get<0>(*due) == element_id && !tooltip_found) {
            const Part& part = std::get<1>(*due);
            std::optional<Tooltip> own;
            if (base::is<Part::Whole>(part)) {
                own = element.tooltip_;
            } else {
                own = element.widget->part_tooltip(part);
                if (!own) {
                    own = element.tooltip_;
                }
            }
            if (own) {
                tooltip_found.emplace(std::move(*own), std::get<2>(*due), std::get<3>(*due));
            }
        }
        draw.pop_clip();
        Parts parts{*this, element_id, interface_detail::name_of(element), element.rect_, element_enabled, true, element_layer, element_clip};
        std::optional<Rect> own_clip = element.widget->children_clip(parts);
        Rect children_clip = own_clip ? own_clip->intersect(element_clip) : element_clip;
        draw_elements(element.children_, element_enabled, element_layer, children_clip, sources, due, tooltip_found);
    }
}

} // namespace user_interface
