// The tree that a program's view describes a frame with: elements, each a
// named widget in a rectangle, with the message that the input on it makes,
// and children for the containers, like cards. The program builds a tree
// from its state each frame, and never changes state while it does; the
// interface lays it out, finds what the input did to the tree the user saw,
// and draws the new one.
//
//     Tree<Message> tree;
//     tree.add("Save", rect, Button<Message>("Save").accent().on_press(Message::Save{}));
//     tree.add("Volume", rect, Slider<Message>(volume, 0.0f, 1.0f).on_change(base::make<Message::Volume>));
//
// A widget is a value of a class that derives from Widget: what it is to
// scripts, what its targets are, what the input on them means, and how it
// draws. One that sends no messages derives from Widget<>, and goes in any
// program's tree; one that sends messages of type M derives from Widget<M>.
// The interface's own widgets are in Widgets/; a program may add its own.
//
// This doesn't include Interface.h, which includes this: what's here reaches
// the interface through Tree.cpp. It ends by including Widgets/Menu.h, for
// the items of an element's context menu, so that whoever has a tree has
// them.

#pragma once

#include "Base/Sum.h"
#include "Base/Vec.h"
#include "UserInterface/Core/Interaction.h"
#include "UserInterface/Design/Controls.h"
#include "UserInterface/Design/Theme.h"
#include "UserInterface/Drawing/Geometry.h"
#include "UserInterface/Drawing/Painter.h"
#include "UserInterface/Drawing/Text.h"
#include "UserInterface/Input/Devices.h"
#include "UserInterface/Input/Events.h"
#include "UserInterface/Platform/Clipboard.h"

#include <concepts>
#include <cstddef>
#include <iterator>
#include <memory>
#include <optional>
#include <span>
#include <string>
#include <string_view>
#include <typeindex>
#include <typeinfo>
#include <utility>
#include <variant>
#include <vector>

namespace user_interface {

class Interface;
class Ui;
class Parts;
class Draw;
// Dispatch<> and Widget<>, without messages, are the bases of Dispatch<M> and
// Widget<M>, with them.
template <class M = void> class Dispatch;
template <> class Dispatch<void>;
template <class M = void> class Widget;
template <> class Widget<void>;
template <class M> class Element;
template <class M> class Tree;
// Widgets/Menu.h defines it.
template <class M> struct MenuItem;
// Core/Interface.h and Application/Program.h define these.
struct Environment;
struct FrameStats;
struct Notice;
// The widgets whose headers define the members that Tree declares for them:
// Widgets/Card.h, Widgets/InfoCard.h and Widgets/Shell.h.
class Card;
struct InfoCard;
class AppBar;
struct Status;
struct Shell;

// A line of a tooltip with more than a sentence in it.
struct TooltipLine {
    struct Heading {
        std::string text;
        bool operator==(const Heading&) const = default;
    };
    struct Text {
        std::string text;
        bool operator==(const Text&) const = default;
    };
    // A label and its value, in two columns.
    struct Pair {
        std::string label;
        std::string value;
        bool operator==(const Pair&) const = default;
    };
    struct Mono {
        std::string text;
        bool operator==(const Mono&) const = default;
    };
    struct Separator {
        bool operator==(const Separator&) const = default;
    };
    std::variant<Heading, Text, Pair, Mono, Separator> value;

    template <base::AlternativeOf<TooltipLine> T> TooltipLine(T&& alternative) : value(std::forward<T>(alternative)) {}

    static TooltipLine heading(std::string text);

    static TooltipLine text(std::string text);

    static TooltipLine pair(std::string label, std::string pair_value);

    static TooltipLine mono(std::string text);

    bool operator==(const TooltipLine&) const = default;
};

// What a widget says after the pointer rests on it.
struct Tooltip {
    std::vector<TooltipLine> lines;
    float max_width;

    bool operator==(const Tooltip&) const = default;
};

// What a widget is: what it is to scripts, what its targets are, what the
// input on them means, and how it draws. Each is called once a frame, for
// each element that has the widget: `parts` and `draw` for the frame's
// tree, and `interact` for the tree the user saw as the input came, with
// what the input did to it.
//
// This is a widget that sends no messages, which a tree of any message type
// takes, and the base of those that send them, Widget<M>.
template <> class Widget<void> {
public:
    virtual ~Widget() = default;

    // Like "Button" or "Slider", as WinUI names its controls. The interface
    // keeps it for as long as it keeps the frame's targets, so it's a
    // literal, or text that lasts as long as the program.
    virtual std::string_view kind() const = 0;

    // What it shows, like a button's text, for scripts and tests.
    virtual std::optional<std::string> label() const {
        return std::nullopt;
    }

    // What its whole area senses, or nothing for a widget that the pointer
    // goes through, like text.
    virtual std::optional<Sense> sense() const {
        return std::nullopt;
    }

    // Whether an unnamed element of it still stops the pointer, like a card
    // over a 3D view.
    virtual bool covers() const {
        return false;
    }

    // The layer it and its children draw on: a dialog's, or the base.
    virtual Layer layer() const {
        return Layer::Base;
    }

    // Registers its parts that are targets of their own, like a list's rows,
    // with `ctx.part`.
    virtual void parts(Parts&) const {}

    // Whether `interact` is called every frame, even when nothing happened
    // to its targets, for a widget that hears keys without focus, like a
    // dialog.
    virtual bool always_interacts() const {
        return false;
    }

    // Whether the wheel that nothing under the pointer takes scrolls this
    // widget, like a scroll area or a list. The deepest such widget under
    // the pointer gets it.
    virtual bool takes_free_wheel() const {
        return false;
    }

    // Does what the input on it means: changes what it keeps between
    // frames. A Widget<M> has the one that sends messages instead, which
    // takes a Dispatch<M>.
    virtual void interact(const Interaction&, Dispatch<void>&) const {}

    // The tooltip of one of its parts, like a list's row.
    virtual std::optional<Tooltip> part_tooltip(const Part&) const {
        return std::nullopt;
    }

    // Draws it.
    virtual void draw(Draw&) const {}

    // Where its children are clipped to, if they are, and moved by, like a
    // scroll area's content.
    virtual std::optional<Rect> children_clip(Parts&) const {
        return std::nullopt;
    }

protected:
    Widget() = default;
    Widget(const Widget&) = default;
    Widget& operator=(const Widget&) = default;
    Widget(Widget&&) = default;
    Widget& operator=(Widget&&) = default;
};

// A widget that sends messages of type M.
template <class M> class Widget : public Widget<void> {
public:
    // What its tree's messages are, which is how Tree::add knows that it
    // goes in a Tree<M> and in no other.
    using Message = M;

    // Does what the input on it means: sends messages, and changes what it
    // keeps between frames.
    virtual void interact(const Interaction&, Dispatch<M>&) const {}

private:
    // The interface calls the one above for a Widget<M>, and never this.
    void interact(const Interaction&, Dispatch<void>&) const final {}
};

// A widget that a Tree<M> takes: one that sends no messages, or one that
// sends M's.
template <class W, class M>
concept WidgetOf = std::derived_from<W, Widget<void>> && std::move_constructible<W> && (!requires { typename W::Message; } || std::same_as<typename W::Message, M>);

// How an element looks this frame: what the input is doing to it.
struct Look {
    bool enabled = false;
    // The whole element's hit.
    Hit hit;
    // The part the pointer is on, and the part with a press.
    std::optional<Part> hovered_part;
    std::optional<Part> held_part;
    std::optional<MouseButton> held_button;
    // It has keyboard focus, and the keyboard put it there, so the focus
    // visual shows; and which of its parts has it.
    bool focused = false;
    std::optional<Part> focused_part;
    bool focus_visible = false;

    ControlState state() const;

    // The state of one of its parts.
    ControlState part_state(const Part& part) const;

    bool hovered(const Part& part) const;

    bool held(const Part& part) const;
};

// What the interface keeps for widgets between frames, which the `memory`
// of Parts, Dispatch and Draw reach from here, where the Interface isn't
// defined yet. Only they and the Interface use it.
namespace interface_detail {

// A value of any type that the interface keeps, in an allocation of its own,
// so that a reference to it stays good as more are kept.
struct Kept {
    virtual ~Kept() = default;
};

template <class T> struct KeptOf final : Kept {
    T value{};
};

// Where the interface keeps the value of the type for the id, which is empty
// until there's one. In Interface.cpp.
std::unique_ptr<Kept>& kept(Interface& ui, Id id, std::type_index type);

// The value of the type that the interface keeps for the id: T's default,
// the first time.
template <class T> T& memory(Interface& ui, Id id) {
    std::unique_ptr<Kept>& place = kept(ui, id, std::type_index(typeid(T)));
    if (place == nullptr) {
        place = std::make_unique<KeptOf<T>>();
    }
    return static_cast<KeptOf<T>&>(*place).value;
}

} // namespace interface_detail

// What a widget's `parts` works with: its element, the fonts to measure
// with, what it keeps between frames, and the interface's state.
class Parts {
public:
    // The library's own.
    Interface& ui;
    Id id_;
    std::string_view name_;
    Rect rect_;
    bool enabled_;
    bool focusable_;
    Layer layer_;
    Rect clip;

    Rect rect() const;

    Id id() const;

    std::string_view name() const;

    bool enabled() const;

    Fonts& fonts();

    Size measure(std::string_view text, TextStyle style);

    const Theme& theme() const;

    double time() const;

    Rect bounds() const;

    // What the widget keeps between frames, by its id: a T, which starts as
    // T's default.
    template <class T> T& memory() {
        return interface_detail::memory<T>(ui, id_);
    }

    template <class T> T& memory_of(Id id) {
        return interface_detail::memory<T>(ui, id);
    }

    bool popup_open() const;

    bool popup_open_of(Id id) const;

    bool is_focused() const;

    const Input& input() const;

    // Registers a part of the widget as a target, named "<widget>/<suffix>".
    // `kind` is a literal, like a widget's.
    void part(Part part, std::string_view suffix, Rect rect, Sense sense, std::string_view kind, std::optional<std::string> label);

    // Registers a part on a layer, like a popup's items, with its full name.
    void part_on(Layer layer, Part part, std::string_view name, Rect rect, Sense sense, std::string_view kind, std::optional<std::string> label);

    // Like part_on, for a part that's disabled on its own, like a menu item.
    void part_on_enabled(Layer layer, Part part, std::string_view name, Rect rect, Sense sense, std::string_view kind, std::optional<std::string> label, bool enabled);

    // Covers the area, so that the pointer stops at it, without a target
    // that scripts see.
    void cover_on(Layer layer, Rect rect);
};

// What a widget's `interact` works with: the frame's input, what it keeps
// between frames, and the interface's state. This is all of it but where its
// messages go, which Dispatch<M> adds; a widget that sends none gets this.
template <> class Dispatch<void> {
public:
    // The library's own.
    Interface& ui;
    Id id_;
    std::string_view name_;
    Rect rect_;
    bool enabled_;
    bool focusable_;

    Rect rect() const;

    Id id() const;

    std::string_view name() const;

    bool enabled() const;

    bool focusable() const;

    const Input& input() const;

    Modifiers modifiers() const;

    double time() const;

    std::optional<Point> pointer() const;

    Fonts& fonts();

    Size measure(std::string_view text, TextStyle style);

    Clipboard& clipboard();

    // What the widget keeps between frames, by its id: a T, which starts as
    // T's default.
    template <class T> T& memory() {
        return interface_detail::memory<T>(ui, id_);
    }

    template <class T> T& memory_of(Id id) {
        return interface_detail::memory<T>(ui, id);
    }

    // Whether the key was pressed this frame, with the widget's own keys
    // counted: what the interface doesn't take, like Esc for a popup.
    bool key_pressed(Key key) const;

    bool key_pressed_once(Key key) const;

    // Whether the key is down, unless the widget with keyboard focus takes
    // it, as a text field takes letters, or a dialog is open.
    bool key_down(Key key) const;

    // Opens the widget's popup, named `name`, like a combo box's list. A
    // popup whose owner is in an open popup opens over it, like a submenu;
    // any other closes the open ones first.
    void open_popup(std::string_view name);

    void open_popup_of(Id owner, std::string_view name);

    void close_popup();

    void close_popup_of(Id owner);

    void close_popups();

    bool popup_open() const;

    bool popup_open_of(Id owner) const;

    bool popup_on_top() const;

    bool popup_on_top_of(Id owner) const;

    bool any_popup() const;

    // Takes keyboard focus for the widget, or for another by its id, as a
    // press on one of its parts does.
    void focus();

    void focus_id(Id id);

    void clear_focus();

    // Whether the widget with keyboard focus takes Enter, like a text box,
    // a list or a button, so a dialog's default button leaves it alone. A
    // focused widget the keys can't reach, like one under the dialog, takes
    // nothing.
    bool focused_takes_enter() const;

    bool is_focused() const;

    bool is_focused_id(Id id) const;

    bool focus_visible() const;

    void set_focus_visible(bool visible);

    void set_cursor(Cursor cursor);

    // Takes this frame's Esc, so that nothing else hears it.
    void consume_escape();

    // The name of the target under the pointer, as of the frame the user
    // saw.
    std::optional<std::string_view> hovered_name() const;

    std::optional<Id> hovered_id() const;

    // The name of the target with keyboard focus, as this frame's input
    // leaves it.
    std::optional<std::string_view> focused_name() const;

    Rect bounds() const;

    // The library's own: the notices under the app bar, for the bar that
    // shows them.
    std::vector<Notice>& notices_mut();
};

// What the `interact` of a widget that sends messages works with: all that
// Dispatch<> has, and where its messages go.
template <class M> class Dispatch : public Dispatch<void> {
public:
    // The library's own.
    std::vector<M>& messages;

    void send(M message) {
        messages.push_back(std::move(message));
    }
};

// What a widget's `draw` works with: the painter, the theme, how the
// element looks, and what it keeps between frames.
class Draw {
public:
    // The library's own.
    Interface& ui;
    Id id_;
    std::string_view name_;
    Rect rect_;
    Look look_;
    Layer layer_;

    Painter painter();

    const Theme& theme() const;

    const Look& look() const;

    ControlState state() const;

    Rect rect() const;

    Id id() const;

    std::string_view name() const;

    bool enabled() const;

    bool focused() const;

    // Whether the focus visual shows: it has focus, and the keyboard moved
    // focus there.
    bool focus_shown() const;

    bool focus_visible() const;

    Layer layer() const;

    double time() const;

    Rect bounds() const;

    const Input& input() const;

    Fonts& fonts();

    Size measure(std::string_view text, TextStyle style);

    // What the widget keeps between frames, by its id: a T, which starts as
    // T's default.
    template <class T> T& memory() {
        return interface_detail::memory<T>(ui, id_);
    }

    template <class T> T& memory_of(Id id) {
        return interface_detail::memory<T>(ui, id);
    }

    bool popup_open() const;

    bool popup_open_of(Id id) const;

    bool popup_on_top_of(Id id) const;

    bool is_focused_id(Id id) const;

    // The part of another widget that the pointer is on, by that widget's
    // id, for widgets built of others. Null if the pointer is on none.
    const Target* hovered_target() const;

    const Target* held_target() const;

    const Environment& environment() const;

    bool window_focused() const;

    FrameStats stats() const;

    std::span<const Notice> notices() const;

    // Draws the focus visual around the rectangle, if the widget has focus
    // and the keyboard moved it there.
    void focus_visual_if(bool focused, Rect rect, float radius);

    void focus_visual_around(bool focused, Rect rect, float radius, Insets margin);

    // Where the painter's instances are in its layer, to mark a place in
    // it, for drawing something behind what was drawn after.
    size_t layer_len() const;

    void move_to_back(size_t first, size_t middle);
};

// The widgets of a frame, in the order they're drawn and that Tab goes
// through them. A tree is built, used and dropped, and never copied.
template <class M> class Tree {
public:
    // The library's own.
    std::vector<Element<M>> elements;
    // The library's own: whether tooltips show this frame.
    bool tooltips = true;

    Tree() = default;
    Tree(const Tree&) = delete;
    Tree& operator=(const Tree&) = delete;
    Tree(Tree&&) = default;
    Tree& operator=(Tree&&) = default;

    // Adds a widget under the name, in the rectangle, and returns it to
    // say more about, like its tooltip. The element is the tree's: the
    // reference is good until the tree gets another.
    template <WidgetOf<M> W> Element<M>& add(std::string_view name, Rect rect, W widget) {
        elements.push_back(Element<M>(name, rect, std::move(widget)).focusable(focusable));
        return elements.back();
    }

    // Adds a widget that's no target, like text or a drawing, in the
    // rectangle.
    template <WidgetOf<M> W> Element<M>& draw(Rect rect, W widget) {
        elements.push_back(Element<M>::unnamed(rect, std::move(widget)).focusable(focusable));
        return elements.back();
    }

    Element<M>& push(Element<M> element) {
        elements.push_back(std::move(element));
        return elements.back();
    }

    // Adds a container, like a card, with the children that `build` adds to
    // the tree it's given, a Tree<M>&, which inherit its enabling. Without a
    // name, it's no target.
    template <WidgetOf<M> W, class Build> Element<M>& container(std::optional<std::string_view> name, Rect rect, W widget, Build&& build) {
        Tree<M> children;
        children.focusable = focusable;
        children.tooltips = tooltips;
        std::forward<Build>(build)(children);
        Element<M> element = name ? Element<M>(*name, rect, std::move(widget)) : Element<M>::unnamed(rect, std::move(widget));
        element.children_ = std::move(children.elements);
        return push(std::move(element).focusable(focusable));
    }

    // Whether what's added from now on takes keyboard focus. A program that
    // keeps focus off everything, like a game, opts its text fields back in
    // with Element::focusable.
    void set_focusable(bool takes_focus) {
        focusable = takes_focus;
    }

    // Whether tooltips show this frame.
    void set_tooltips(bool shown) {
        tooltips = shown;
    }

    bool is_empty() const {
        return elements.empty();
    }

    size_t len() const {
        return elements.size();
    }

    void append(Tree<M> other) {
        elements.insert(elements.end(), std::make_move_iterator(other.elements.begin()), std::make_move_iterator(other.elements.end()));
    }

    // Every element, with its children after it, as they're drawn. `visit`
    // is called with each, a const Element<M>&.
    template <class Visit> void walk(Visit&& visit) const {
        walk_through(elements, visit);
    }

    // The element with the name, if the tree has one, or null.
    const Element<M>* find(std::string_view name) const {
        Id wanted = id(name);
        const Element<M>* found = nullptr;
        walk([&](const Element<M>& element) {
            if (found == nullptr && element.id == wanted) {
                found = &element;
            }
        });
        return found;
    }

    // What the widgets' own headers add, each of which defines its own.
    //
    // Widgets/Card.h: a card in the rectangle, with the children that
    // `build` adds, given where the card's parts are: it's called with a
    // Ui&, the card's Tree<M>& and its CardArea. A named card is a target of
    // kind "Card".
    template <class Build> Element<M>& card(Ui& ui, std::optional<std::string_view> name, Rect rect, Card card, Build&& build);

    // Widgets/Card.h: a card at the rectangle's top, as tall as its content:
    // `build` lays the content out from the top of the area it's given,
    // which may be as tall as the rectangle, and returns where the content
    // ends, a float. Returns where the card is.
    template <class Build> Rect sized_card(Ui& ui, std::optional<std::string_view> name, Rect rect, Card card, Build&& build);

    // Widgets/InfoCard.h: adds the card at the bottom left of the window, as
    // tall as what it says.
    void info_card(Ui& ui, const InfoCard& contents);

    // Widgets/Shell.h: adds the page's bars and the notices: the app bar, a
    // menu row under it if `menu_row`, a status bar with the items if there
    // are any, and each notice as "Notice/<index>". Returns where they are,
    // and where the page is.
    Shell shell(Ui& ui, AppBar app_bar, bool menu_row, std::optional<std::vector<Status>> status);

private:
    template <class Visit> static void walk_through(const std::vector<Element<M>>& list, Visit& visit) {
        for (const Element<M>& element : list) {
            visit(element);
            walk_through(element.children_, visit);
        }
    }

    // What elements added from now on take keyboard focus by, unless they
    // say otherwise.
    bool focusable = true;
};

// A widget in a rectangle, with its name, and what it says about itself.
template <class M> class Element {
public:
    // The library's own.
    std::optional<Id> id;
    std::optional<std::string> name_;
    Rect rect_;
    bool enabled_ = true;
    bool focusable_ = true;
    // What it shows, for scripts, if the widget doesn't say.
    std::optional<std::string> label_;
    std::optional<Tooltip> tooltip_;
    // A menu that a right-click on it opens.
    std::optional<std::vector<MenuItem<M>>> context_menu_;
    std::unique_ptr<Widget<void>> widget;
    // Whether the widget is a Widget<M>, whose `interact` takes a
    // Dispatch<M>, and not only a Widget<>.
    bool sends = false;
    std::vector<Element<M>> children_;

    template <WidgetOf<M> W>
    Element(std::string_view name, Rect rect, W widget) :
        id(user_interface::id(name)), name_(std::string(name)), rect_(rect), widget(std::make_unique<W>(std::move(widget))), sends(std::derived_from<W, Widget<M>>) {}
    Element(const Element&) = delete;
    Element& operator=(const Element&) = delete;
    Element(Element&&) = default;
    Element& operator=(Element&&) = default;

    // An element that's no target: text, a drawing, or a container that only
    // covers what's under it.
    template <WidgetOf<M> W> static Element unnamed(Rect rect, W content) {
        Element element("", rect, std::move(content));
        element.id = std::nullopt;
        element.name_ = std::nullopt;
        return element;
    }

    Element&& enabled(bool enabled) && {
        enabled_ = enabled;
        return std::move(*this);
    }

    Element&& focusable(bool focusable) && {
        focusable_ = focusable;
        return std::move(*this);
    }

    Element&& label(std::string label) && {
        label_ = std::move(label);
        return std::move(*this);
    }

    Element&& tooltip(std::string text) && {
        set_tooltip(std::move(text));
        return std::move(*this);
    }

    Element&& tooltip_lines(std::vector<TooltipLine> lines) && {
        set_tooltip_lines(std::move(lines));
        return std::move(*this);
    }

    // A tooltip with more in it than WinUI's 320 pixels hold well, up to
    // `max_width` wide.
    Element&& tooltip_wide(std::vector<TooltipLine> lines, float max_width) && {
        set_tooltip_wide(std::move(lines), max_width);
        return std::move(*this);
    }

    // A menu that opens at the pointer when it's right-clicked.
    Element&& context_menu(std::vector<MenuItem<M>> items) && {
        context_menu_ = std::move(items);
        return std::move(*this);
    }

    Element&& children(Tree<M> children) && {
        children_ = std::move(children.elements);
        return std::move(*this);
    }

    // The same, on an element that's in a tree.
    Element& set_enabled(bool enabled) {
        enabled_ = enabled;
        return *this;
    }

    Element& set_focusable(bool focusable) {
        focusable_ = focusable;
        return *this;
    }

    Element& set_label(std::string label) {
        label_ = std::move(label);
        return *this;
    }

    Element& set_tooltip(std::string text) {
        tooltip_ = Tooltip{.lines = base::vec(TooltipLine(TooltipLine::Text{std::move(text)})), .max_width = controls::TooltipMaxWidth};
        return *this;
    }

    Element& set_tooltip_lines(std::vector<TooltipLine> lines) {
        tooltip_ = Tooltip{.lines = std::move(lines), .max_width = controls::TooltipMaxWidth};
        return *this;
    }

    Element& set_tooltip_wide(std::vector<TooltipLine> lines, float max_width) {
        tooltip_ = Tooltip{.lines = std::move(lines), .max_width = max_width};
        return *this;
    }

    Element& set_context_menu(std::vector<MenuItem<M>> items) {
        context_menu_ = std::move(items);
        return *this;
    }

    Rect rect() const {
        return rect_;
    }

    std::optional<std::string_view> name() const {
        if (name_) {
            return std::string_view(*name_);
        }
        return std::nullopt;
    }

    bool is_enabled() const {
        return enabled_;
    }
};

} // namespace user_interface

// The items of an element's context menu, which need everything above.
#include "UserInterface/Widgets/Menu.h"
