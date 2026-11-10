// The page that every program's window shows: the PerfBar along the very
// top, which the library draws; an app bar under it, as tall as WinUI's
// TitleBar control, with the program's icon, title and its own commands; a
// menu row under that, if the program wants one, for a menu bar whose menus'
// names line up with the page's left edge; notices under them; a status bar
// along the bottom with shortcuts and status; and the page between them,
// where the cards go. Tree::shell adds the bars and the notices, and says
// where the page is:
//
//     Shell shell = tree.shell(ui, AppBar(Icon::Settings, "Settings"), false, base::vec(Status::text("Ready")));
//     // Cards in shell.page.
//
// This defines Tree::shell, which Core/Tree.h declares, and Shell.cpp
// defines Interface::perf_bar, which Core/Interface.h declares.

#pragma once

#include "Base/Sum.h"
#include "UserInterface/Application/Program.h"
#include "UserInterface/Core/Interaction.h"
#include "UserInterface/Core/Interface.h"
#include "UserInterface/Core/Tree.h"
#include "UserInterface/Design/Icons.h"
#include "UserInterface/Design/Tokens.h"
#include "UserInterface/Drawing/Geometry.h"
#include "UserInterface/Drawing/Painter.h"
#include "UserInterface/Drawing/Text.h"
#include "UserInterface/Widgets/InfoBar.h"

#include <cstddef>
#include <format>
#include <optional>
#include <span>
#include <string>
#include <string_view>
#include <utility>
#include <variant>
#include <vector>

namespace user_interface {

// The app bar: the program's icon and title. The rest of the bar, which
// `commands` gives, is for the program's own tabs and commands.
class AppBar : public Widget<> {
public:
    AppBar(Icon icon, std::string title);

    AppBar&& subtitle(std::string subtitle) &&;

    // Where the app bar goes: under the PerfBar, the window's width.
    static Rect rect(const Ui& ui);

    // The rest of the bar, right of the title, for the program's own tabs
    // and commands.
    Rect commands(Ui& ui) const;

    std::string_view kind() const override;

    bool covers() const override;

    void draw(Draw& ctx) const override;

private:
    float title_right(Fonts& fonts) const;

    Icon icon;
    std::string title;
    std::optional<std::string> subtitle_;
};

// A row under the app bar, the window's width and as tall as WinUI's
// MenuBar, for a menu bar and the program's commands. Its `area` is for
// them: from where a menu bar's first menu has its name on the page's left
// edge, in line with the PerfBar's text, the app bar's icon and the cards,
// as Windows 11 lines up a navigation item's icon under the title bar's,
// with its backplate out in the padding; to the page's right edge.
class MenuRow : public Widget<> {
public:
    MenuRow() = default;
    MenuRow(const MenuRow&) = delete;
    MenuRow& operator=(const MenuRow&) = delete;
    MenuRow(MenuRow&&) = default;
    MenuRow& operator=(MenuRow&&) = default;

    static Rect rect(const Ui& ui, float top);

    static Rect area(Rect rect);

    std::string_view kind() const override;

    bool covers() const override;

    void draw(Draw& ctx) const override;
};

// What a status bar shows, from its left, then from its right after a
// spacer.
struct Status {
    // Keys, like "Ctrl+S", as keycaps, and what they do.
    struct Shortcut {
        std::string keys;
        std::string label;
        bool operator==(const Shortcut&) const = default;
    };
    struct Text {
        std::string text;
        bool operator==(const Text&) const = default;
    };
    // A dot in a status's color, and what it says.
    struct Indicator {
        Severity severity;
        std::string text;
        bool operator==(const Indicator&) const = default;
    };
    // What comes after goes along the right.
    struct Spacer {
        bool operator==(const Spacer&) const = default;
    };
    std::variant<Shortcut, Text, Indicator, Spacer> value;

    template <base::AlternativeOf<Status> T> Status(T&& alternative) : value(std::forward<T>(alternative)) {}

    static Status shortcut(std::string keys, std::string label);

    static Status text(std::string text);

    static Status indicator(Severity severity, std::string text);

    bool operator==(const Status&) const = default;

private:
    // The status bar lays its items out and draws them.
    friend class StatusBar;

    float width(Fonts& fonts) const;

    void draw(Painter& painter, Rect rect) const;
};

// The status bar along the bottom.
class StatusBar : public Widget<> {
public:
    explicit StatusBar(std::vector<Status> items);
    StatusBar(const StatusBar&) = delete;
    StatusBar& operator=(const StatusBar&) = delete;
    StatusBar(StatusBar&&) = default;
    StatusBar& operator=(StatusBar&&) = default;

    static Rect rect(const Ui& ui);

    std::string_view kind() const override;

    bool covers() const override;

    void draw(Draw& ctx) const override;

private:
    std::vector<Status> items;
};

// Where the page's bars are, and the page between them.
struct Shell {
    Rect app_bar;
    // The app bar right of the title, for the program's own commands.
    Rect commands;
    // The menu row's area for a menu bar and commands, if there's a row.
    std::optional<Rect> menu_row;
    std::optional<Rect> status_bar;
    // Between the bars, inside the page's padding, under the notices.
    Rect page;
};

// What only the shell's own code uses, here and in Shell.cpp.
namespace shell_detail {

// A notice under the app bar: an info bar whose close button removes it
// from the interface's notices.
template <class M> class NoticeBar : public Widget<M> {
public:
    NoticeBar(size_t index, InfoBar<M> bar) : index(index), bar(std::move(bar)) {}
    NoticeBar(const NoticeBar&) = delete;
    NoticeBar& operator=(const NoticeBar&) = delete;
    NoticeBar(NoticeBar&&) = default;
    NoticeBar& operator=(NoticeBar&&) = default;

    std::string_view kind() const override {
        return "InfoBar";
    }

    std::optional<std::string> label() const override {
        return bar.label();
    }

    std::optional<Sense> sense() const override {
        return Sense::None;
    }

    void parts(Parts& ctx) const override {
        bar.parts(ctx);
    }

    void interact(const Interaction& interaction, Dispatch<M>& ctx) const override {
        if (interaction.activated(Part::Named{"Close"})) {
            std::vector<Notice>& notices = ctx.notices_mut();
            if (index < notices.size()) {
                notices.erase(notices.begin() + static_cast<std::ptrdiff_t>(index));
            }
        }
    }

    void draw(Draw& ctx) const override {
        bar.draw(ctx);
    }

private:
    size_t index;
    InfoBar<M> bar;
};

template <class M> InfoBar<M> notice_bar(const Notice& notice) {
    return InfoBar<M>(notice.severity, notice.title, notice.message).closable();
}

// What unsupported_setup shows.
class UnsupportedSetup : public Widget<> {
public:
    explicit UnsupportedSetup(std::vector<std::string> problems);
    UnsupportedSetup(const UnsupportedSetup&) = delete;
    UnsupportedSetup& operator=(const UnsupportedSetup&) = delete;
    UnsupportedSetup(UnsupportedSetup&&) = default;
    UnsupportedSetup& operator=(UnsupportedSetup&&) = default;

    std::string_view kind() const override;

    void draw(Draw& ctx) const override;

private:
    std::vector<std::string> problems;
};

} // namespace shell_detail

// Adds the page's bars and the notices: the app bar, a menu row under
// it if `menu_row`, a status bar with the items if there are any, and
// each notice as "Notice/<index>". Returns where they are, and where the
// page is.
template <class M> Shell Tree<M>::shell(Ui& ui, AppBar app_bar, bool menu_row, std::optional<std::vector<Status>> status) {
    Rect bar = AppBar::rect(ui);
    Rect commands = app_bar.commands(ui);
    draw(bar, std::move(app_bar));
    float top = bar.bottom();
    std::optional<Rect> menu_area;
    if (menu_row) {
        Rect rect = MenuRow::rect(ui, top);
        draw(rect, MenuRow());
        top = rect.bottom();
        menu_area = MenuRow::area(rect);
    }
    float bottom = ui.size().height;
    std::optional<Rect> status_rect;
    if (status) {
        Rect rect = StatusBar::rect(ui);
        draw(rect, StatusBar(std::move(*status)));
        bottom = rect.y;
        status_rect = rect;
    }
    float width = ui.size().width;
    Rect page(tokens::PagePadding, top + tokens::PagePadding, width - tokens::PagePadding * 2.0f, bottom - top - tokens::PagePadding * 2.0f);
    std::span<const Notice> notices = ui.notices();
    for (size_t index = 0; index < notices.size(); index++) {
        InfoBar<M> notice = shell_detail::notice_bar<M>(notices[index]);
        float height = notice.height(ui, page.width);
        Rect rect = page.cut_top(height, tokens::SpaceS);
        add(std::format("Notice/{}", index), rect, shell_detail::NoticeBar<M>(index, std::move(notice)));
    }
    if (!notices.empty()) {
        page.cut_top(tokens::SpaceS, 0.0f);
    }
    return Shell{
        .app_bar = bar,
        .commands = commands,
        .menu_row = menu_area,
        .status_bar = status_rect,
        .page = page,
    };
}

// What the window shows instead of the program in a setup that programs
// don't support: what's wrong, and the setups that are.
template <class M> Tree<M> unsupported_setup(std::span<const std::string> problems) {
    Tree<M> tree;
    tree.draw(Rect(0.0f, 0.0f, 0.0f, 0.0f), shell_detail::UnsupportedSetup(std::vector<std::string>(problems.begin(), problems.end())));
    return tree;
}

} // namespace user_interface
