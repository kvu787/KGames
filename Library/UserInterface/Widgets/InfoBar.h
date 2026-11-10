// WinUI's InfoBar: a message of one of four severities, in its background
// color, with its status icon, its title in Body Strong and its message in
// Body after it on the same line, or under it if they don't fit, and a close
// button if it's closable, named "<name>/Close".
//
// This doesn't include Core/Interface.h, which includes this, by way of
// Application/Program.h, for the severity of a notice: a program that calls
// InfoBar::height has the Ui from there.

#pragma once

#include "Base/Panic.h"
#include "UserInterface/Core/Interaction.h"
#include "UserInterface/Core/Tree.h"
#include "UserInterface/Design/Controls.h"
#include "UserInterface/Design/Icons.h"
#include "UserInterface/Design/Theme.h"
#include "UserInterface/Design/Tokens.h"
#include "UserInterface/Drawing/Geometry.h"
#include "UserInterface/Drawing/Painter.h"
#include "UserInterface/Drawing/Text.h"
#include "UserInterface/Widgets/Buttons.h"
#include "UserInterface/Widgets/Text.h"

#include <algorithm>
#include <cstdint>
#include <format>
#include <optional>
#include <string>
#include <string_view>
#include <utility>

namespace user_interface {

enum class Severity : std::uint8_t {
    Informational,
    Success,
    Warning,
    Error,
};

constexpr std::string_view name(Severity severity) {
    switch (severity) {
    case Severity::Informational:
        return "Informational";
    case Severity::Success:
        return "Success";
    case Severity::Warning:
        return "Warning";
    case Severity::Error:
        return "Error";
    }
    base::unreachable();
}

// How a failed check in a test shows a severity.
inline std::string describe_value(Severity severity) {
    return std::string(name(severity));
}

// What only the info bar's own code uses, here and in InfoBar.cpp.
namespace info_bar_detail {

inline constexpr std::string_view Close = "Close";

// The fonts that a Ui measures with, for InfoBar::height, which is here,
// where the Ui isn't defined yet.
Fonts& fonts_of(Ui& ui);

// Without the whitespace at its end: str::trim_end.
std::string_view trim_end(std::string_view text);

} // namespace info_bar_detail

template <class M> class InfoBar : public Widget<M> {
public:
    InfoBar(Severity severity, std::string title, std::string message) : severity(severity), title(std::move(title)), message(std::move(message)) {}
    InfoBar(const InfoBar&) = delete;
    InfoBar& operator=(const InfoBar&) = delete;
    InfoBar(InfoBar&&) = default;
    InfoBar& operator=(InfoBar&&) = default;

    InfoBar&& closable() && {
        closable_ = true;
        return std::move(*this);
    }

    // What its close button sends, which it has then.
    InfoBar&& on_close(M close_message) && {
        closable_ = true;
        on_close_ = std::move(close_message);
        return std::move(*this);
    }

    // How tall the bar is in the width.
    float height(Ui& ui, float width) const {
        return height_with(info_bar_detail::fonts_of(ui), width);
    }

    std::string_view kind() const override {
        return "InfoBar";
    }

    std::optional<std::string> label() const override {
        std::string text = std::format("{}: {} {}", user_interface::name(severity), title, message);
        return std::string(info_bar_detail::trim_end(text));
    }

    std::optional<Sense> sense() const override {
        return Sense::None;
    }

    void parts(Parts& ctx) const override {
        if (closable_) {
            ctx.part(Part::Named{std::string(info_bar_detail::Close)}, info_bar_detail::Close, close_rect(ctx.rect()), Sense::Button, "Button", std::nullopt);
        }
    }

    void interact(const Interaction& interaction, Dispatch<M>& ctx) const override {
        if (interaction.activated(Part::Named{std::string(info_bar_detail::Close)}) && on_close_) {
            ctx.send(*on_close_);
        }
    }

    void draw(Draw& ctx) const override {
        Rect rect = ctx.rect();
        bool one_line = layout(ctx.fonts(), rect.width).first;
        const Look& look = ctx.look();
        {
            Painter painter = ctx.painter();
            const Theme& theme = painter.theme();
            Color background = [&] {
                switch (severity) {
                case Severity::Informational:
                    return theme.attention_background;
                case Severity::Success:
                    return theme.success_background;
                case Severity::Warning:
                    return theme.caution_background;
                case Severity::Error:
                    return theme.critical_background;
                }
                base::unreachable();
            }();
            painter.shape(rect, tokens::RadiusControl, background, Border(theme.card_stroke, 1.0f));
            float left = rect.x + text_left();
            float room = std::max(rect.right() - text_right() - left, 0.0f);
            float title_top = rect.y + controls::InfoBarTextTop;
            if (one_line) {
                Rect line(left, title_top, room, tokens::Body.line_height);
                Rect title_rect = painter.text_fit(line, title, tokens::BodyStrong, theme.text_primary, Align::Start).first;
                float after = title.empty() ? title_rect.x : title_rect.right() + controls::InfoBarMessageGap;
                painter.text_fit(Rect(after, line.y, std::max(line.right() - after, 0.0f), line.height), message, tokens::Body, theme.text_primary, Align::Start);
            } else {
                Rect title_rect(left, title_top, room, tokens::BodyStrong.line_height);
                painter.text_fit(title_rect, title, tokens::BodyStrong, theme.text_primary, Align::Start);
                float message_top = title_rect.bottom() + controls::InfoBarWrappedMessageGap;
                painter.text_wrapped(Rect(left, message_top, room, rect.bottom() - message_top), message, tokens::Body, theme.text_primary);
            }
            // The icon's top is 16 from the bar's, as is a one-line bar's
            // middle.
            Point icon_center(rect.x + controls::InfoBarPaddingLeft + controls::InfoBarIconSize / 2.0f,
                              rect.y + controls::InfoBarIconMargin.top + controls::InfoBarIconSize / 2.0f);
            status_icon(painter, icon_center, severity, controls::InfoBarIconSize);
        }
        if (closable_) {
            Part part = Part::Named{std::string(info_bar_detail::Close)};
            Rect close = close_rect(rect);
            ControlState state = look.part_state(part);
            draw_button(ctx, close, "", Icon::Dismiss, ButtonStyle::Subtle, false, false, state);
            bool focused = look.focused && look.focused_part == part;
            ctx.focus_visual_if(focused, close, tokens::RadiusControl);
        }
    }

private:
    float height_with(Fonts& fonts, float width) const {
        auto [one_line, message_height] = layout(fonts, width);
        if (one_line) {
            return controls::InfoBarMinHeight;
        }
        return std::max(controls::InfoBarTextTop + tokens::BodyStrong.line_height + controls::InfoBarWrappedMessageGap + message_height + controls::InfoBarWrappedBottom,
                        controls::InfoBarMinHeight);
    }

    float text_left() const {
        return controls::InfoBarPaddingLeft + controls::InfoBarIconSize + controls::InfoBarIconMargin.right;
    }

    float text_right() const {
        return controls::InfoBarPanelMarginRight +
               (closable_ ? controls::InfoBarCloseButtonSize + controls::InfoBarCloseButtonMargin * 2.0f - controls::InfoBarPanelMarginRight : 0.0f);
    }

    // Whether the title and message fit on one line, and if not, the
    // message's height, wrapped.
    std::pair<bool, float> layout(Fonts& fonts, float width) const {
        float room = std::max(width - text_left() - text_right(), 1.0f);
        float title_width = fonts.measure(title, tokens::BodyStrong).width;
        float message_width = fonts.measure(message, tokens::Body).width;
        if (title_width + controls::InfoBarMessageGap + message_width <= room) {
            return {true, tokens::Body.line_height};
        }
        return {false, fonts.measure_wrapped(message, tokens::Body, room).height};
    }

    static Rect close_rect(Rect rect) {
        float size = controls::InfoBarCloseButtonSize;
        return Rect(rect.right() - controls::InfoBarCloseButtonMargin - size, rect.y + controls::InfoBarCloseButtonMargin, size, size);
    }

    Severity severity;
    std::string title;
    std::string message;
    bool closable_ = false;
    std::optional<M> on_close_;
};

} // namespace user_interface
