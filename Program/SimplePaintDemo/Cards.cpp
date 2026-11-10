#include "SimplePaintDemo/Cards.h"

#include "Base/Numbers.h"
#include "Base/Text.h"
#include "Base/Vec.h"
#include "SimplePaint/Material.h"
#include "SimplePaintDemo/Editor.h"
#include "UserInterface/Core/Interaction.h"
#include "UserInterface/Design/Controls.h"
#include "UserInterface/Design/Icons.h"
#include "UserInterface/Design/Theme.h"
#include "UserInterface/Drawing/Geometry.h"
#include "UserInterface/Drawing/Painter.h"
#include "UserInterface/Widgets/Buttons.h"
#include "UserInterface/Widgets/Card.h"
#include "UserInterface/Widgets/InfoBar.h"
#include "UserInterface/Widgets/Shell.h"
#include "UserInterface/Widgets/Slider.h"
#include "UserInterface/Widgets/Text.h"

#include <algorithm>
#include <array>
#include <cmath>
#include <cstdint>
#include <format>
#include <optional>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

namespace simple_paint_demo::cards {

using demo::Demo;
using demo::Message;
using demo::Title;
using editor::Editor;
using editor::Setting;
using editor::Settings;
using simple_paint::GpuMaterial;
using simple_paint::Parameters;
using user_interface::AppBar;
using user_interface::Border;
using user_interface::Button;
using user_interface::Canvas;
using user_interface::Card;
using user_interface::CardArea;
using user_interface::Color;
using user_interface::Dispatch;
using user_interface::Draw;
using user_interface::Environment;
using user_interface::Icon;
using user_interface::Interaction;
using user_interface::Look;
using user_interface::Painter;
using user_interface::Point;
using user_interface::Rect;
using user_interface::Run;
using user_interface::Sense;
using user_interface::Severity;
using user_interface::Shell;
using user_interface::Size;
using user_interface::Slider;
using user_interface::Spans;
using user_interface::Status;
using user_interface::StatusIcon;
using user_interface::Target;
using user_interface::Text;
using user_interface::TextColor;
using user_interface::Theme;
using user_interface::Tree;
using user_interface::Ui;
using user_interface::Widget;
namespace controls = user_interface::controls;
namespace tokens = user_interface::tokens;

namespace {

// Reset and Save, which are as wide as each other.
constexpr float ButtonWidth = 96.0f;
// Each setting's row, like a list's, and the parts of it.
constexpr float RowHeight = controls::ListItemHeight;
constexpr float LabelWidth = 120.0f;
constexpr float ValueWidth = 72.0f;
constexpr float RangeWidth = 128.0f;
constexpr Size SwatchSize = Size(64.0f, 24.0f);
// The GPU constants' names, and their values right of them.
constexpr float NameWidth = 152.0f;

// Defined below what uses them.
void setting_row(const Editor& editor, Tree<Message>& tree, size_t index, Rect row, Rect card);
void save_status(Ui& ui, Tree<Message>& tree, Rect area, const std::optional<std::pair<std::string, bool>>& status);
void base_color(Ui& ui, Tree<Message>& tree, Rect row, const Editor& editor);
double linear(double srgb);

std::vector<Status> status_bar(Ui& ui) {
    const Environment& environment = ui.environment();
    std::string display = std::format("{} {}", environment.display, environment.display_name);
    return base::vec(Status::shortcut("Up/Down", "Select a setting"), Status::shortcut("Left/Right", "Change it"), Status::shortcut("Shift+Left/Right", "By 10 steps"),
                     Status::shortcut("Ctrl+S", "Save to Paint.json"), Status::shortcut("F9", "Theme"), Status::shortcut("F10", "VSync"), Status::shortcut("F11", "Fullscreen"),
                     Status(Status::Spacer{}), Status::text(environment.gpu), Status::text(std::string(base::trim(display))));
}

// A card's content, cut from the top down, which notes whether everything
// fit in it.
struct Content {
    Rect area;
    bool fits = true;

    explicit Content(Rect area) : area(area) {}

    Rect take(float height, float gap) {
        fits &= height <= area.height;
        return area.cut_top(height, gap);
    }

    // A paragraph of secondary text, wrapped to the content's width.
    void paragraph(Ui& ui, Tree<Message>& tree, std::string_view text) {
        float height = ui.paragraph_height(area.width, text, tokens::Body);
        Rect rect = take(height, 0.0f);
        tree.draw(rect, Text::paragraph(std::string(text)).secondary());
    }
};

// The preview: a card whose view the demo's render fills with the sphere,
// around the card's color. The demo notes where the view is and the card's
// color here, for render.
void preview(const Demo& demo, Ui& ui, Tree<Message>& tree, Rect rect) {
    Card card = Card("Preview").subtitle("A UV sphere, orthographic, 4x multisampled. Any view of a sphere shows every facing once.").with_view();
    CardArea area = card.area(ui, rect);
    demo.view_ = area.content;
    demo.background = ui.theme().card_opaque();
    tree.draw(rect, std::move(card));
}

// A slider for each setting, in rows like a list's, of which the arrow keys
// move the selected one, the base color, and the buttons that save and reset
// the material.
bool settings(const Demo& demo, Ui& ui, Tree<Message>& tree, Rect rect) {
    const Editor& editor = demo.editor;
    bool fits = true;
    tree.card(ui, std::nullopt, rect, Card("Settings"), [&](Ui& ui, Tree<Message>& card, CardArea area) {
        // The buttons at the header's right, and what the last save did,
        // left of them.
        Rect header = area.header;
        Button<Message> save = Button<Message>("Save").icon(Icon::Save).accent().on_press(Message::Save{});
        Button<Message> reset = Button<Message>("Reset").icon(Icon::Refresh).on_press(Message::Reset{});
        Rect save_rect = header.cut_right(std::max(save.width(ui), ButtonWidth), tokens::SpaceS);
        Rect reset_rect = header.cut_right(std::max(reset.width(ui), ButtonWidth), tokens::SpaceL);
        card.add("Reset", reset_rect, std::move(reset)).set_tooltip("Goes back to SimplePaint's default material");
        card.add("Save", save_rect, std::move(save)).set_tooltip("Saves the material to Paint.json (Ctrl+S)");
        save_status(ui, card, header, editor.status);

        // The sliders don't take keyboard focus, as the arrow keys move the
        // selected one.
        card.set_focusable(false);
        Content content(area.content);
        for (size_t index = 0; index < Settings.size(); index++) {
            Rect row = content.take(RowHeight, 0.0f);
            setting_row(editor, card, index, row, area.rect);
        }
        card.set_focusable(true);

        content.take(0.0f, tokens::SpaceM);
        base_color(ui, card, content.take(tokens::ControlHeight, tokens::SpaceM), editor);
        content.paragraph(ui, card,
                          "Drag a slider, or scroll over it for its smallest step. Values step by 0.001, and Rotation by 0.1 degrees, "
                          "and stop at the last step that SimplePaint accepts, so Material::compile always takes them.");
        fits = content.fits;
    });
    return fits;
}

// A setting's row face, like a list's item: a press on it selects the
// setting, its fill follows the pointer on it or on its slider, and the
// selected one has the accent's pill at its left edge.
class SettingRow : public Widget<Message> {
public:
    SettingRow(size_t index, bool selected, std::string_view name) : index(index), selected(selected), name(name) {}

    std::string_view kind() const override {
        return "ListViewItem";
    }

    std::optional<std::string> label() const override {
        return std::string(name);
    }

    std::optional<Sense> sense() const override {
        return Sense::Click;
    }

    void interact(const Interaction& interaction, Dispatch<Message>& ctx) const override {
        if (interaction.whole().pressed) {
            ctx.send(Message::Select{index});
        }
    }

    void draw(Draw& ctx) const override {
        Rect face = ctx.rect();
        const Look& look = ctx.look();
        // The pointer on the slider, or holding it, is on the row too.
        const Target* hovered_target = ctx.hovered_target();
        const Target* held_target = ctx.held_target();
        bool slider_hovered = hovered_target != nullptr && hovered_target->name == name;
        bool slider_held = held_target != nullptr && held_target->name == name;
        bool hovered = look.hit.hovered || slider_hovered;
        bool pressed = look.hit.pressing() || slider_held;
        const Theme& theme = ctx.theme();
        // ListViewItemBackground in each of the row's states.
        Color fill = theme.subtle_fill_transparent;
        if ((!selected && pressed) || (selected && !pressed && hovered)) {
            fill = theme.subtle_fill_tertiary;
        } else if ((!selected && hovered) || selected) {
            fill = theme.subtle_fill_secondary;
        }
        Color accent = theme.accent;
        Painter painter = ctx.painter();
        painter.rounded(face, controls::ListItemRadius, fill);
        // The pill that marks the selected row, at its face's left edge, as
        // a list's.
        if (selected) {
            float y = std::round(face.center().y - controls::SelectionPillLength / 2.0f);
            Rect pill(face.x, y, controls::SelectionPillThickness, controls::SelectionPillLength);
            painter.rounded(pill, controls::SelectionPillThickness / 2.0f, accent);
        }
    }

private:
    size_t index;
    bool selected;
    // The setting's name, which its slider's target has.
    std::string_view name;
};

// A setting's row: its face, its name, its slider, its value, and its range.
void setting_row(const Editor& editor, Tree<Message>& tree, size_t index, Rect row, Rect card) {
    const Setting& setting = Settings[index];
    // The row's face, from near the card's edges, like a list's row.
    Rect face = Rect(card.x, row.y, card.width, row.height).pad(controls::ListItemBackplateMargin);
    tree.add(std::format("Setting/{}", setting.name), face, SettingRow(index, index == editor.selected, setting.name));

    Rect parts = row;
    Rect label = parts.cut_left(LabelWidth, tokens::SpaceM);
    Rect range = parts.cut_right(RangeWidth, tokens::SpaceXxl);
    Rect value_rect = parts.cut_right(ValueWidth, tokens::SpaceL);
    Rect track = parts.centered(Size(parts.width, Slider<Message>::height(false)));

    double value = setting.value(editor.parameters);
    Slider<Message> slider = setting.slider<Message>(value)
                                 .on_change([index](double moved) { return Message::Set{index, moved}; })
                                 .on_hold([index](bool held) { return Message::Drag{index, held}; })
                                 .on_wheel([index](double) { return Message::Select{index}; });
    tree.add(setting.name, track, std::move(slider));

    tree.draw(label, Text::label(std::string(setting.name)));
    tree.draw(value_rect, Text::label(setting.format(value)).style(tokens::Mono).end());
    tree.draw(range, Text::label(setting.range()).secondary());
}

// What the last save did, at the right of the area: a success or an error,
// with InfoBar's status icon, and the whole message as a tooltip if it's cut
// short.
void save_status(Ui& ui, Tree<Message>& tree, Rect area, const std::optional<std::pair<std::string, bool>>& status) {
    if (!status) {
        return;
    }
    const std::string& message = status->first;
    bool failed = status->second;
    Severity severity = failed ? Severity::Error : Severity::Success;
    float room = std::max(area.width - tokens::Icon - tokens::SpaceS, 0.0f);
    float width = std::min(ui.measure(message, tokens::Body).width, room);
    Rect text = area.right_strip(width);
    tree.add("SaveStatus", text, Text::label(message).hoverable()).set_tooltip(message);
    Rect icon = Rect::from_center(Point(text.x - tokens::SpaceS - tokens::Icon / 2.0f, area.center().y), Size::square(tokens::Icon));
    tree.draw(icon, StatusIcon(severity, tokens::Icon));
}

// The base color, as the sliders make it: a swatch, in hex, in linear, and
// the tone that it appears at.
void base_color(Ui& ui, Tree<Message>& tree, Rect row, const Editor& editor) {
    const Parameters& parameters = editor.parameters;
    std::array<double, 3> srgb = parameters.base_color_srgb;
    std::uint8_t red = base::saturating_cast<std::uint8_t>(std::round(srgb[0] * 255.0));
    std::uint8_t green = base::saturating_cast<std::uint8_t>(std::round(srgb[1] * 255.0));
    std::uint8_t blue = base::saturating_cast<std::uint8_t>(std::round(srgb[2] * 255.0));
    double r = linear(srgb[0]);
    double g = linear(srgb[1]);
    double b = linear(srgb[2]);
    std::string hex = std::format("#{:02X}{:02X}{:02X}", red, green, blue);
    std::string in_linear = std::format("{:.4f}, {:.4f}, {:.4f}", r, g, b);
    std::string tone = std::format("{:.3f}", 1.0 - parameters.brightness);

    Rect parts = row;
    tree.draw(parts.cut_left(LabelWidth, tokens::SpaceM), Text::label("Base color"));
    // Like a palette's swatch, which this one isn't, as it can't be chosen.
    Rect column = parts.cut_left(SwatchSize.width, tokens::SpaceM);
    Rect swatch(column.x, row.center().y - SwatchSize.height / 2.0f, SwatchSize.width, SwatchSize.height);
    Color color = Color::from_bytes({red, green, blue, 255});
    tree.draw(swatch, Canvas([color](Painter& painter, Rect rect) {
                  Border border(painter.theme().control_stroke_secondary, 1.0f);
                  painter.shape(rect, tokens::RadiusControl, color, border);
              }));
    std::array<Spans, 3> groups{
        Spans(base::vec(Run(std::move(hex), tokens::Mono, TextColor::Primary{}))),
        Spans(base::vec(Run("Linear ", tokens::Body, TextColor::Secondary{}), Run(std::move(in_linear), tokens::Mono, TextColor::Primary{}))),
        Spans(base::vec(Run("Appears at tone 1 - Brightness = ", tokens::Body, TextColor::Secondary{}), Run(std::move(tone), tokens::Mono, TextColor::Primary{}))),
    };
    float top = std::round(row.center().y - tokens::Body.line_height / 2.0f);
    float x = parts.x;
    for (Spans& spans : groups) {
        float width = spans.width(ui.fonts());
        tree.draw(Rect(x, top, width, tokens::Body.line_height), std::move(spans));
        x += width + tokens::SpaceXxl;
    }
}

// A number as Rust's {:.6e} writes it, like 1.234568e-2, which the GPU
// constants have always shown: C++'s own puts a sign and at least two digits
// in the exponent, like 1.234568e-02.
std::string exponential(float value) {
    if (!std::isfinite(value)) {
        return base::display(value);
    }
    std::string text = std::format("{:.6e}", value);
    size_t at = text.find('e');
    std::string_view exponent = std::string_view(text).substr(at + 1);
    bool negative = exponent.starts_with('-');
    exponent.remove_prefix(1);
    while (exponent.size() > 1 && exponent.front() == '0') {
        exponent.remove_prefix(1);
    }
    return std::format("{}e{}{}", std::string_view(text).substr(0, at), negative ? "-" : "", exponent);
}

// What Material::compile made for the GPU, in columns of Cascadia Mono.
bool constants(const Editor& editor, Ui& ui, Tree<Message>& tree, Rect rect) {
    Card card = Card("GPU constants").subtitle("GpuMaterial: 80 bytes at b1, five float4 registers");
    const GpuMaterial& m = editor.material;
    // To the 6 decimals shown, where adding 0 turns -0, like the sine of
    // 180 degrees in floats, -0.0000000874, into 0.
    auto shown = [](float value) {
        return std::round(static_cast<double>(value) * 1e6) / 1e6 + 0.0;
    };
    double cosine = shown(m.warp[0]);
    double sine = shown(m.warp[1]);
    double k = shown(m.warp[2]);
    double flag = shown(m.warp[3]);
    std::vector<std::pair<std::string_view, std::string>> rows;
    rows.emplace_back("warp", std::format("cos {:+.6f}   sin {:+.6f}   k {:.6f}   shifted {}", cosine, sine, k, base::display(flag)));
    for (const auto& [name, values] : std::array<std::pair<std::string_view, std::array<float, 4>>, 4>{{
             {"numerator_dark", m.numerator_dark},
             {"numerator_light", m.numerator_light},
             {"complement_dark", m.complement_dark},
             {"complement_light", m.complement_light},
         }}) {
        rows.emplace_back(name, std::format("R {}   G {}   B {}", exponential(values[0]), exponential(values[1]), exponential(values[2])));
    }
    bool fits = true;
    tree.card(ui, std::nullopt, rect, std::move(card), [&](Ui& ui, Tree<Message>& card, CardArea area) {
        Content content(area.content);
        for (auto& [name, value] : rows) {
            Rect row = content.take(tokens::Mono.line_height, 0.0f);
            card.draw(row, Text::label(std::string(name)).style(tokens::Mono).secondary());
            card.draw(Rect(row.x + NameWidth, row.y, std::max(row.width - NameWidth, 0.0f), row.height), Text::label(std::move(value)).style(tokens::Mono));
        }
        content.take(0.0f, tokens::SpaceS);
        content.paragraph(ui, card,
                          "Per pixel, with facing weights P and Q: color = numerator / (numerator + complement), where numerator = "
                          "numerator_dark * Q + numerator_light * P, and likewise for complement.");
        fits = content.fits;
    });
    return fits;
}

double linear(double srgb) {
    if (srgb <= 0.04045) {
        return srgb / 12.92;
    }
    return std::pow((srgb + 0.055) / 1.055, 2.4);
}

} // namespace

Tree<Message> view(const Demo& demo, Ui& ui) {
    Tree<Message> tree;
    std::string_view subtitle = "A SimplePaint material on a UV sphere, with an editor for its settings";
    std::vector<Status> status = status_bar(ui);
    Shell shell = tree.shell(ui, AppBar(Icon::Color, std::string(Title)).subtitle(std::string(subtitle)), false, std::move(status));
    Rect page = shell.page;
    std::vector<std::string_view>& overflow = demo.overflow;
    overflow.clear();
    Rect preview_rect = page.cut_left(PreviewWidth, tokens::CardGap);
    preview(demo, ui, tree, preview_rect);
    Rect column = page;
    bool settings_fit = settings(demo, ui, tree, column.cut_top(SettingsHeight, tokens::CardGap));
    bool constants_fit = constants(demo.editor, ui, tree, column);
    std::array<std::pair<std::string_view, bool>, 2> fits{{
        {"Settings", settings_fit},
        {"GPU constants", constants_fit},
    }};
    for (const auto& [name, fit] : fits) {
        if (!fit) {
            overflow.push_back(name);
        }
    }
    return tree;
}

} // namespace simple_paint_demo::cards
