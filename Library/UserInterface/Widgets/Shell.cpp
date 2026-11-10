#include "UserInterface/Widgets/Shell.h"

#include "Base/Numbers.h"
#include "Base/Panic.h"
#include "Base/Text.h"
#include "UserInterface/Application/Performance.h"
#include "UserInterface/Design/Controls.h"
#include "UserInterface/Design/Theme.h"
#include "UserInterface/Widgets/Text.h"

#include <algorithm>
#include <cmath>
#include <ranges>

namespace user_interface {

namespace {

// The PerfBar's text, as runs of labels and values, each given to `take`.
template <class Take> void runs(std::span<const performance::Run> runs, Take& take) {
    for (const performance::Run& run : runs) {
        take(run.text, run.value);
    }
}

// The PerfBar's columns, each a character of Cascadia Mono wide, from the
// bar's padding, and its text's color.
struct PerfBarGrid {
    float advance;
    float y;
    Color color;

    // Draws runs from the first column, cutting short what would go past the
    // end column. `runs` is called with what takes each run, in order: its
    // text, and whether it's a value.
    template <class Runs> void draw(Painter& painter, size_t end, Runs&& runs) const {
        size_t column = 0;
        auto take = [&](std::string_view text, bool) {
            size_t count = base::char_count(text);
            Point at(tokens::PagePadding + static_cast<float>(column) * advance, y);
            if (column + count <= end) {
                painter.text(at, text, tokens::PerfBarText, color);
            } else if (column < end) {
                Fitted fitted = painter.fit(text, tokens::PerfBarText, static_cast<float>(end - column) * advance + 0.5f);
                painter.text(at, fitted.view(), tokens::PerfBarText, color);
            }
            column += count;
        };
        runs(take);
    }
};

} // namespace

AppBar::AppBar(Icon icon, std::string title) : icon(icon), title(std::move(title)) {}

AppBar&& AppBar::subtitle(std::string subtitle) && {
    subtitle_ = std::move(subtitle);
    return std::move(*this);
}

Rect AppBar::rect(const Ui& ui) {
    return Rect(0.0f, ui.content_top(), ui.size().width, tokens::AppBarHeight);
}

Rect AppBar::commands(Ui& ui) const {
    Rect rect = AppBar::rect(ui);
    float right = title_right(ui.fonts());
    float left = right + tokens::SpaceL;
    return Rect(left, rect.y + tokens::SpaceS, rect.right() - tokens::SpaceL - left, rect.height - tokens::SpaceS * 2.0f);
}

float AppBar::title_right(Fonts& fonts) const {
    float x = tokens::SpaceL + tokens::Icon + tokens::SpaceL + fonts.measure(title, tokens::Caption).width;
    if (subtitle_) {
        x += tokens::SpaceS + fonts.measure(*subtitle_, tokens::Caption).width;
    }
    return x;
}

std::string_view AppBar::kind() const {
    return "AppBar";
}

bool AppBar::covers() const {
    return true;
}

void AppBar::draw(Draw& ctx) const {
    Rect rect = ctx.rect();
    Painter painter = ctx.painter();
    const Theme& theme = painter.theme();
    // Opaque, like Mica's solid stand-in, so that it covers a 3D view.
    painter.fill(rect, theme.background);
    Point icon_center(tokens::SpaceL + tokens::Icon / 2.0f, rect.center().y);
    painter.icon(icon_center, icon, tokens::Icon, theme.accent);
    float x = tokens::SpaceL + tokens::Icon + tokens::SpaceL;
    Rect title_rect = painter.text_in(Rect(x, rect.y, rect.width, rect.height), title, tokens::Caption, theme.text_primary, Align::Start);
    if (subtitle_) {
        painter.text_in(Rect(title_rect.right() + tokens::SpaceS, rect.y, rect.width, rect.height), *subtitle_, tokens::Caption, theme.text_secondary, Align::Start);
    }
}

Rect MenuRow::rect(const Ui& ui, float top) {
    return Rect(0.0f, top, ui.size().width, controls::MenuBarHeight);
}

Rect MenuRow::area(Rect rect) {
    float left = tokens::PagePadding - controls::MenuBarItemMargin.left - controls::MenuBarItemPadding.left;
    return Rect(left, rect.y, rect.right() - tokens::PagePadding - left, rect.height);
}

std::string_view MenuRow::kind() const {
    return "MenuRow";
}

bool MenuRow::covers() const {
    return true;
}

void MenuRow::draw(Draw& ctx) const {
    Rect rect = ctx.rect();
    Painter painter = ctx.painter();
    const Theme& theme = painter.theme();
    // Opaque, like the app bar.
    painter.fill(rect, theme.background);
}

Status Status::shortcut(std::string keys, std::string label) {
    return Shortcut{std::move(keys), std::move(label)};
}

Status Status::text(std::string text) {
    return Text{std::move(text)};
}

Status Status::indicator(Severity severity, std::string text) {
    return Indicator{severity, std::move(text)};
}

float Status::width(Fonts& fonts) const {
    return base::match(
        *this,
        [&](const Shortcut& shortcut) {
            float keys = keys_width(fonts, shortcut.keys);
            return keys + tokens::SpaceS + fonts.measure(shortcut.label, tokens::Caption).width;
        },
        [&](const Text& text) { return fonts.measure(text.text, tokens::Caption).width; },
        [&](const Indicator& indicator) { return 8.0f + tokens::SpaceS + fonts.measure(indicator.text, tokens::Caption).width; }, [&](const Spacer&) { return 0.0f; });
}

void Status::draw(Painter& painter, Rect rect) const {
    const Theme& theme = painter.theme();
    base::match(
        *this,
        [&](const Shortcut& shortcut) {
            float width = draw_keys(painter, Point(rect.x, rect.center().y - KeycapHeight / 2.0f), shortcut.keys);
            Rect label_rect(rect.x + width + tokens::SpaceS, rect.y, std::max(rect.width - width - tokens::SpaceS, 0.0f), rect.height);
            painter.text_fit(label_rect, shortcut.label, tokens::Caption, theme.text_secondary, Align::Start);
        },
        [&](const Text& text) { painter.text_fit(rect, text.text, tokens::Caption, theme.text_secondary, Align::Start); },
        [&](const Indicator& indicator) {
            painter.circle(Point(rect.x + 4.0f, rect.center().y), 4.0f, theme.status(indicator.severity));
            Rect label_rect(rect.x + 8.0f + tokens::SpaceS, rect.y, std::max(rect.width - 8.0f - tokens::SpaceS, 0.0f), rect.height);
            painter.text_fit(label_rect, indicator.text, tokens::Caption, theme.text_secondary, Align::Start);
        },
        [&](const Spacer&) {});
}

StatusBar::StatusBar(std::vector<Status> items) : items(std::move(items)) {}

Rect StatusBar::rect(const Ui& ui) {
    Size size = ui.size();
    return Rect(0.0f, size.height - tokens::StatusBarHeight, size.width, tokens::StatusBarHeight);
}

std::string_view StatusBar::kind() const {
    return "StatusBar";
}

bool StatusBar::covers() const {
    return true;
}

void StatusBar::draw(Draw& ctx) const {
    Rect rect = ctx.rect();
    Painter painter = ctx.painter();
    const Theme& theme = painter.theme();
    painter.fill(rect, theme.background);
    painter.fill(rect.top_strip(1.0f), theme.divider);
    size_t spacer = items.size();
    for (size_t index = 0; index < items.size(); index++) {
        if (base::is<Status::Spacer>(items[index])) {
            spacer = index;
            break;
        }
    }
    std::span<const Status> left = std::span<const Status>(items).first(spacer);
    std::span<const Status> from_spacer = std::span<const Status>(items).subspan(spacer);
    std::span<const Status> right = from_spacer.empty() ? from_spacer : from_spacer.subspan(1);
    // The right's items, from the right edge in, so that the left's can
    // be cut short where they'd meet them. They take at most the bar's
    // right half, and the one that would go past its middle is cut
    // short, like a long message.
    float middle = rect.center().x;
    float right_edge = rect.right() - tokens::SpaceL;
    for (const Status& item : right | std::views::reverse) {
        float width = std::min(item.width(painter.fonts()), right_edge - middle);
        if (width <= 0.0f) {
            break;
        }
        item.draw(painter, Rect(right_edge - width, rect.y + 1.0f, width, rect.height - 1.0f));
        right_edge -= width + tokens::SpaceL + tokens::SpaceS;
    }
    float x = rect.x + tokens::SpaceL;
    for (const Status& item : left) {
        float width = std::min(item.width(painter.fonts()), std::max(right_edge - x, 0.0f));
        if (width <= 0.0f) {
            break;
        }
        item.draw(painter, Rect(x, rect.y + 1.0f, width, rect.height - 1.0f));
        x += width + tokens::SpaceL + tokens::SpaceS;
    }
}

void Interface::perf_bar() {
    // The summary's words are made 10 times a second, and read here every
    // frame, not copied.
    if (!performance) {
        return;
    }
    const user_interface::performance::Summary& summary = performance->summary();
    Rect bar(0.0f, 0.0f, size().width, tokens::PerfBarHeight);
    Color divider = theme().divider;
    auto [background, text] = [&]() -> std::pair<Color, Color> {
        switch (theme().kind) {
        case ThemeKind::Light:
            return {Color::hex(0xFFFFFF), Color::hex(0x000000)};
        case ThemeKind::Dark:
            return {Color::hex(0x000000), Color::hex(0xFFFFFF)};
        }
        base::unreachable();
    }();
    // Vertical sync from this frame on, as F10 switches it, not as of the
    // summary, as wide whether it's on or off.
    std::string_view vertical_sync = environment_.vertical_sync ? "on " : "off";
    {
        Painter painter = this->painter();
        // Opaque, even over a 3D view.
        painter.fill(bar, background);
        painter.fill(Rect(0.0f, bar.bottom() - 1.0f, bar.width, 1.0f), divider);
        // The text's ink centered above the divider.
        float above = bar.height - 1.0f;
        PerfBarGrid grid{
            .advance = painter.measure("0", tokens::PerfBarText).width,
            .y = std::round((above - tokens::PerfBarText.line_height) / 2.0f - tokens::MonoInkDrop * tokens::PerfBarText.size),
            .color = text,
        };
        size_t columns = base::saturating_cast<size_t>(std::floor((bar.width - tokens::PagePadding * 2.0f) / grid.advance));
        // The stats from the first column, then vertical sync, then why
        // there's no input latency, if there isn't.
        grid.draw(painter, columns, [&](auto& take) {
            runs(summary.stats, take);
            take(user_interface::performance::Separator, false);
            take("VSync = ", false);
            take(vertical_sync, true);
            runs(summary.note, take);
        });
    }
}

shell_detail::UnsupportedSetup::UnsupportedSetup(std::vector<std::string> problems) : problems(std::move(problems)) {}

std::string_view shell_detail::UnsupportedSetup::kind() const {
    return "UnsupportedSetup";
}

void shell_detail::UnsupportedSetup::draw(Draw& ctx) const {
    Rect bounds = ctx.bounds();
    float width = std::min(720.0f, bounds.width - tokens::SpaceL * 2.0f);
    float height = tokens::SpaceL * 2.0f + 28.0f + tokens::SpaceM + tokens::SpaceS;
    std::vector<float> lines;
    for (const std::string& problem : problems) {
        float line = ctx.fonts().measure_wrapped(problem, tokens::Body, width - tokens::SpaceL * 2.0f - tokens::SpaceM).height;
        lines.push_back(line);
        height += line + tokens::SpaceXs;
    }
    std::string_view advice = "Programs run on a 2560 × 1440 display at 100% scale, with a 48-pixel taskbar at its bottom, maximized or in borderless fullscreen with F11.";
    float advice_height = ctx.fonts().measure_wrapped(advice, tokens::Body, width - tokens::SpaceL * 2.0f).height;
    height += tokens::SpaceS + advice_height;
    Rect rect = bounds.centered(Size(width, height)).round();
    Painter painter = ctx.painter();
    const Theme& theme = painter.theme();
    painter.fill(bounds, theme.background);
    painter.shape(rect, tokens::RadiusControl * 2.0f, theme.critical_background, Border(theme.card_stroke, 1.0f));
    Rect area = rect.inset(tokens::SpaceL);
    Rect title = area.cut_top(28.0f, tokens::SpaceM);
    painter.text_in(title, "This window setup isn't supported", tokens::Subtitle, theme.text_primary, Align::Start);
    for (size_t index = 0; index < problems.size(); index++) {
        Rect row = area.cut_top(lines[index], tokens::SpaceXs);
        painter.circle(Point(row.x + 3.0f, row.y + 10.0f), 2.5f, theme.critical);
        painter.text_wrapped(Rect(row.x + tokens::SpaceM, row.y, row.width - tokens::SpaceM, row.height), problems[index], tokens::Body, theme.text_primary);
    }
    area.cut_top(tokens::SpaceS, 0.0f);
    painter.text_wrapped(area, advice, tokens::Body, theme.text_secondary);
}

} // namespace user_interface
