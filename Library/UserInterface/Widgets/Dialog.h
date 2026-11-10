// WinUI's ContentDialog: a box in the middle of the window, over smoke that
// keeps the page from input, with a title, a message or content of the
// program's own, and up to three buttons, of which the default is the
// accent's. Esc is its close button, and Enter its default one.
//
// The program's view adds it while it's open, in the area that `area`
// gives, with its own content as children in `area.content`, and the
// dialog sends which button was chosen:
//
//     Dialog<Message> dialog = Dialog<Message>("Quit?", "Unsaved work is lost.").primary("Quit").on_result(base::make<Message::Quit>);
//     DialogArea area = dialog.area(ui);
//     tree.add("Quit", area.rect, std::move(dialog));

#pragma once

#include "UserInterface/Core/Interaction.h"
#include "UserInterface/Core/Interface.h"
#include "UserInterface/Core/Tree.h"
#include "UserInterface/Design/Controls.h"
#include "UserInterface/Design/Tokens.h"
#include "UserInterface/Drawing/Geometry.h"
#include "UserInterface/Drawing/Painter.h"
#include "UserInterface/Drawing/Text.h"
#include "UserInterface/Input/Devices.h"
#include "UserInterface/Widgets/Buttons.h"
#include "UserInterface/Widgets/Widget.h"

#include <algorithm>
#include <cmath>
#include <cstddef>
#include <cstdint>
#include <optional>
#include <string>
#include <string_view>
#include <tuple>
#include <utility>
#include <vector>

namespace user_interface {

enum class DialogResult : std::uint8_t {
    Primary,
    Secondary,
    Close,
};

// How a failed check in a test shows a result.
std::string describe_value(DialogResult result);

// Where a dialog is, and where its content goes.
struct DialogArea {
    Rect rect;
    Rect content;
};

// What only the dialog's own code uses, here and in Dialog.cpp.
namespace dialog_detail {

// The part of the dialog that its button is, named "<dialog>/Primary"
// and so on.
std::string_view part_name(DialogResult result);

// A dialog's layout, from its texts.
struct Metrics {
    Rect rect;
    float inner;
    float title_height;
    float message_height;
    float top_height;
    Rect content;
};

float gap(float height);

} // namespace dialog_detail

template <class M> class Dialog : public Widget<M> {
public:
    Dialog(std::string title, std::string message) : title(std::move(title)), message(std::move(message)) {}
    Dialog(const Dialog&) = delete;
    Dialog& operator=(const Dialog&) = delete;
    Dialog(Dialog&&) = default;
    Dialog& operator=(Dialog&&) = default;

    // The primary button, which is the default, in the accent's colors.
    Dialog&& primary(std::string text) && {
        primary_ = std::move(text);
        if (!default_) {
            default_ = DialogResult::Primary;
        }
        return std::move(*this);
    }

    Dialog&& secondary(std::string text) && {
        secondary_ = std::move(text);
        return std::move(*this);
    }

    Dialog&& close(std::string text) && {
        close_ = std::move(text);
        return std::move(*this);
    }

    Dialog&& default_button(DialogResult button) && {
        default_ = button;
        return std::move(*this);
    }

    // Its width, between ContentDialogMinWidth and ContentDialogMaxWidth.
    Dialog&& width(float width) && {
        width_ = width;
        return std::move(*this);
    }

    // Room under the message for content of the program's own, which goes
    // in the dialog's children, in `area().content`.
    Dialog&& content_height(float height) && {
        content_height_ = height;
        return std::move(*this);
    }

    // What choosing a button sends: `reaction` is called with a
    // DialogResult, and returns the message.
    template <class F> Dialog&& on_result(F&& reaction) && {
        on_result_ = react<DialogResult, M>(std::forward<F>(reaction));
        return std::move(*this);
    }

    // Where the dialog goes in the window, and where its content goes.
    DialogArea area(Ui& ui) const {
        Rect bounds = ui.bounds();
        dialog_detail::Metrics metrics = this->metrics(ui.fonts(), bounds);
        return DialogArea{
            .rect = metrics.rect,
            .content = metrics.content,
        };
    }

    std::string_view kind() const override {
        return "ContentDialog";
    }

    std::optional<std::string> label() const override {
        return title;
    }

    std::optional<Sense> sense() const override {
        return Sense::None;
    }

    Layer layer() const override {
        return Layer::Dialog;
    }

    bool always_interacts() const override {
        return true;
    }

    void parts(Parts& ctx) const override {
        // The smoke covers the page and takes its input.
        Rect bounds = ctx.bounds();
        ctx.cover_on(Layer::Dialog, bounds);
        for (const auto& [rect, result, text] : buttons(ctx.rect())) {
            std::string_view name = dialog_detail::part_name(result);
            ctx.part(Part::Named{std::string(name)}, name, rect, Sense::Button, "Button", std::string(text));
        }
    }

    void interact(const Interaction& interaction, Dispatch<M>& ctx) const override {
        std::optional<DialogResult> chosen;
        for (const auto& button : buttons(ctx.rect())) {
            DialogResult result = std::get<DialogResult>(button);
            if (interaction.activated(Part::Named{std::string(dialog_detail::part_name(result))})) {
                chosen = result;
            }
        }
        if (ctx.key_pressed_once(Key::Escape)) {
            chosen = DialogResult::Close;
            ctx.consume_escape();
        }
        // Enter is the default button's, unless something that takes Enter
        // has focus - a text box submitting, a combo box opening, a focused
        // button clicking - or a popup is open, which takes it.
        if (!chosen && default_ && !ctx.any_popup() && ctx.key_pressed_once(Key::Enter) && !ctx.focused_takes_enter()) {
            chosen = default_;
        }
        if (chosen && on_result_) {
            ctx.send(on_result_(*chosen));
        }
    }

    void draw(Draw& ctx) const override {
        Rect rect = ctx.rect();
        Rect bounds = ctx.bounds();
        dialog_detail::Metrics metrics = this->metrics(ctx.fonts(), bounds);
        const Look& look = ctx.look();
        {
            Painter painter = ctx.painter();
            const Theme& theme = painter.theme();
            painter.fill(bounds, theme.smoke);
            painter.shadow(rect, tokens::RadiusOverlay, tokens::ElevationDialog);
            painter.shape(rect, tokens::RadiusOverlay, theme.background, Border(theme.surface_stroke, 1.0f));
            Rect top(rect.x + 1.0f, rect.y + 1.0f, rect.width - 2.0f, metrics.top_height - 1.0f);
            painter.shape(top, Corners::top(tokens::RadiusOverlay - 1.0f), theme.layer_alt, std::nullopt);
            painter.fill(Rect(rect.x + 1.0f, rect.y + metrics.top_height - 1.0f, rect.width - 2.0f, 1.0f), theme.card_stroke);
            Rect area = rect.pad(Insets::all(controls::DialogPadding));
            Rect title_area = area.cut_top(metrics.title_height, dialog_detail::gap(metrics.message_height + content_height_));
            painter.text_wrapped(title_area, title, tokens::Subtitle, theme.text_primary);
            if (metrics.message_height > 0.0f) {
                Rect message_area = area.cut_top(metrics.message_height, dialog_detail::gap(content_height_));
                painter.text_wrapped(message_area, message, tokens::Body, theme.text_primary);
            }
        }
        for (const auto& [button_rect, result, text] : buttons(rect)) {
            Part part = Part::Named{std::string(dialog_detail::part_name(result))};
            ButtonStyle style = default_ == result ? ButtonStyle::Accent : ButtonStyle::Standard;
            ControlState state = look.part_state(part);
            draw_button(ctx, button_rect, text, std::nullopt, style, false, false, state);
            bool focused = look.focused && look.focused_part == part;
            ctx.focus_visual_if(focused, button_rect, tokens::RadiusControl);
        }
    }

private:
    dialog_detail::Metrics metrics(Fonts& fonts, Rect bounds) const {
        using dialog_detail::gap;
        float width = std::clamp(width_.value_or(controls::DialogMaxWidth), controls::DialogMinWidth, controls::DialogMaxWidth);
        float inner = width - controls::DialogPadding * 2.0f;
        float title_height = fonts.measure_wrapped(title, tokens::Subtitle, inner).height;
        float message_height = message.empty() ? 0.0f : fonts.measure_wrapped(message, tokens::Body, inner).height;
        float content_height = content_height_;
        float top_height = controls::DialogPadding * 2.0f + title_height + gap(message_height) + message_height + gap(content_height) + content_height;
        float commands_height = controls::DialogPadding * 2.0f + tokens::ControlHeight;
        float height = std::max(top_height + commands_height, controls::DialogMinHeight);
        Rect rect = bounds.centered(Size(width, height)).round();
        float content_top = rect.y + controls::DialogPadding + title_height + gap(message_height + content_height) + message_height + gap(content_height);
        return dialog_detail::Metrics{
            .rect = rect,
            .inner = inner,
            .title_height = title_height,
            .message_height = message_height,
            .top_height = top_height,
            .content = Rect(rect.x + controls::DialogPadding, content_top, inner, content_height),
        };
    }

    // The buttons, which share the command area's width, the last at its
    // right: each one's rectangle, which button it is, and its text.
    std::vector<std::tuple<Rect, DialogResult, std::string_view>> buttons(Rect rect) const {
        std::vector<std::pair<std::string_view, DialogResult>> buttons;
        if (primary_) {
            buttons.emplace_back(*primary_, DialogResult::Primary);
        }
        if (secondary_) {
            buttons.emplace_back(*secondary_, DialogResult::Secondary);
        }
        if (!close_.empty()) {
            buttons.emplace_back(close_, DialogResult::Close);
        }
        float inner = rect.width - controls::DialogPadding * 2.0f;
        Rect row(rect.x + controls::DialogPadding, rect.bottom() - controls::DialogPadding - tokens::ControlHeight, inner, tokens::ControlHeight);
        float count = static_cast<float>(std::max<size_t>(buttons.size(), 1));
        float button_width = std::floor((row.width - controls::DialogButtonGap * (count - 1.0f)) / count);
        std::vector<std::tuple<Rect, DialogResult, std::string_view>> placed;
        placed.reserve(buttons.size());
        for (size_t index = 0; index < buttons.size(); index++) {
            const auto& [text, result] = buttons[index];
            float x = index + 1 == buttons.size() ? row.right() - button_width : row.x + (button_width + controls::DialogButtonGap) * static_cast<float>(index);
            placed.emplace_back(Rect(x, row.y, button_width, row.height), result, text);
        }
        return placed;
    }

    std::string title;
    std::string message;
    std::optional<std::string> primary_;
    std::optional<std::string> secondary_;
    std::string close_ = "Close";
    std::optional<DialogResult> default_;
    std::optional<float> width_;
    // How tall the program's own content under the message is.
    float content_height_ = 0.0f;
    Reaction<DialogResult, M> on_result_;
};

} // namespace user_interface
