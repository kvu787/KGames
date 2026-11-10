// The showcase's page. See Cards.h.

#include "UiShowcase/Cards.h"

#include "Base/Numbers.h"
#include "Base/Text.h"
#include "Base/Vec.h"

#include <algorithm>
#include <cmath>
#include <format>
#include <limits>
#include <numbers>
#include <optional>
#include <ranges>
#include <string>
#include <tuple>
#include <utility>
#include <vector>

using user_interface::Align;
using user_interface::AppBar;
using user_interface::Badge;
using user_interface::Border;
using user_interface::Brush;
using user_interface::Button;
using user_interface::Canvas;
using user_interface::Card;
using user_interface::CardArea;
using user_interface::CheckBox;
using user_interface::Color;
using user_interface::ColorField;
using user_interface::ColorPicker;
using user_interface::ComboBox;
using user_interface::Corners;
using user_interface::Dialog;
using user_interface::DialogArea;
using user_interface::Environment;
using user_interface::Expander;
using user_interface::ExpanderContent;
using user_interface::Field;
using user_interface::Form;
using user_interface::FrameStats;
using user_interface::Gamepad;
using user_interface::HyperlinkButton;
using user_interface::Icon;
using user_interface::IconView;
using user_interface::InfoBar;
using user_interface::Insets;
using user_interface::Keycaps;
using user_interface::Keys;
using user_interface::ListRows;
using user_interface::ListView;
using user_interface::MenuBar;
using user_interface::MenuItem;
using user_interface::NumberBox;
using user_interface::Painter;
using user_interface::Point;
using user_interface::ProgressBar;
using user_interface::ProgressRing;
using user_interface::RadioGroup;
using user_interface::Rect;
using user_interface::Run;
using user_interface::Segmented;
using user_interface::SelectorBar;
using user_interface::Sense;
using user_interface::Severity;
using user_interface::Shell;
using user_interface::Size;
using user_interface::Slider;
using user_interface::Spans;
using user_interface::Status;
using user_interface::Text;
using user_interface::TextBox;
using user_interface::TextColor;
using user_interface::TextStyle;
using user_interface::Theme;
using user_interface::ThemeKind;
using user_interface::ToggleButton;
using user_interface::ToggleSwitch;
using user_interface::TooltipLine;
using user_interface::Tree;
using user_interface::Ui;
using user_interface::View;
namespace tokens = user_interface::tokens;

namespace ui_showcase::cards {

using showcase::Alignments;
using showcase::Buttons;
using showcase::Message;
using showcase::Options;
using showcase::Showcase;
using showcase::Sizes;
using showcase::Tabs;
using showcase::Tools;

namespace {

constexpr std::array<std::string_view, 5> RoleUses{
    "What's on, chosen, or the one command that matters most", "What went well", "What needs care", "What went wrong", "What's worth knowing",
};

// The cards' heights; the bottom card of each column takes the rest.
constexpr float ButtonsHeight = 260.0f;
constexpr float TogglesHeight = 344.0f;
constexpr float SlidersHeight = 492.0f;
constexpr float ChoicesHeight = 320.0f;
constexpr float InfoHeight = 324.0f;
constexpr float ColorHeight = 492.0f;

// A menu item that sends its label as a command.
MenuItem<Message> command(std::string_view label) {
    return MenuItem<Message>(std::string(label)).on_choose(Message::Command{std::string(label)});
}

std::vector<MenuBar<Message>::Menu> menus() {
    std::vector<MenuItem<Message>> file =
        base::vec(command("New").icon(Icon::Document).accelerator("Ctrl+N"), command("Open").icon(Icon::OpenFile).accelerator("Ctrl+O"),
                  command("Save").icon(Icon::Save).accelerator("Ctrl+S"), MenuItem<Message>::separator(), command("Reset").icon(Icon::Refresh).accelerator("Ctrl+R"));
    std::vector<MenuItem<Message>> theme = base::vec(command("Dark theme").accelerator("F9"), command("Tooltips").accelerator("F1"));
    std::vector<MenuItem<Message>> view =
        base::vec(command("VSync").accelerator("F10"), command("Fullscreen").icon(Icon::FullScreen).accelerator("F11"),
                  MenuItem<Message>::submenu("Theme", std::move(theme)).icon(Icon::Color), MenuItem<Message>::separator(), MenuItem<Message>("Disabled").enabled(false));
    std::vector<MenuItem<Message>> help = base::vec(command("Ask a question").icon(Icon::Help));
    return base::vec(MenuBar<Message>::Menu("File", std::move(file)), MenuBar<Message>::Menu("View", std::move(view)), MenuBar<Message>::Menu("Help", std::move(help)));
}

std::vector<MenuItem<Message>> context_menu() {
    return base::vec(command("Cut").icon(Icon::Cut).accelerator("Ctrl+X"), command("Copy").icon(Icon::Copy).accelerator("Ctrl+C"),
                     command("Paste").icon(Icon::Paste).accelerator("Ctrl+V"), MenuItem<Message>::separator(), command("Pinned"));
}

void buttons(const Showcase& showcase, Ui& ui, Tree<Message>& tree, Rect rect) {
    tree.card(ui, std::nullopt, rect, Card("Buttons"), [&](Ui&, Tree<Message>& card, CardArea card_area) {
        Rect area = card_area.content;
        auto [standard, accent, subtle] = area.cut_top(tokens::ControlHeight, tokens::SpaceS).columns<3>(tokens::SpaceS);
        card.add(Buttons[0], standard, Button<Message>("Standard").on_press(Message::Press{0}));
        card.add(Buttons[1], accent, Button<Message>("Accent").accent().on_press(Message::Press{1}));
        card.add(Buttons[2], subtle, Button<Message>("Subtle").subtle().on_press(Message::Press{2}));
        auto [disabled, disabled_accent, link] = area.cut_top(tokens::ControlHeight, tokens::SpaceS).columns<3>(tokens::SpaceS);
        card.add("Disabled", disabled, Button<Message>("Disabled")).set_enabled(false);
        card.add("DisabledAccent", disabled_accent, Button<Message>("Disabled").accent()).set_enabled(false);
        card.add("Link", link, HyperlinkButton<Message>("Learn more").on_press(Message::Link{}));
        auto [save, pin, more] = area.cut_top(tokens::ControlHeight, tokens::SpaceS).columns<3>(tokens::SpaceS);
        card.add("Save", save, Button<Message>("Save").icon(Icon::Save).on_press(Message::Press{3})).set_tooltip("Saves nothing, but counts");
        card.add("Pin", pin, ToggleButton<Message>(showcase.pinned, Button<Message>("Pin").icon(Icon::Pin)).on_toggle(base::make<Message::Pin>));
        card.add("More", more, Button<Message>("More").menu(context_menu()));
        Rect row = area.cut_top(tokens::ControlHeight, tokens::SpaceS);
        Rect minus = row.cut_left(tokens::ControlHeight, tokens::SpaceS);
        Rect count = row.cut_left(48.0f, tokens::SpaceS);
        Rect plus = row.cut_left(tokens::ControlHeight, tokens::SpaceS);
        card.add("Remove", minus, Button<Message>::icon_only(Icon::Remove).on_press(Message::Remove{})).set_tooltip("One less");
        card.draw(count, Text::label(std::to_string(showcase.count)).center());
        card.add("Add", plus, Button<Message>::icon_only(Icon::Add).on_press(Message::Add{})).set_tooltip("One more");
        std::string presses =
            std::format("Pressed: Standard {}, Accent {}, Subtle {}, Save {}", showcase.presses[0], showcase.presses[1], showcase.presses[2], showcase.presses[3]);
        card.draw(area.cut_top(tokens::Caption.line_height, 0.0f), Text::caption(std::move(presses)));
    });
}

void toggles(const Showcase& showcase, Ui& ui, Tree<Message>& tree, Rect rect) {
    bool dark = ui.theme().kind == ThemeKind::Dark;
    tree.card(ui, std::nullopt, rect, Card("Checkboxes and switches"), [&](Ui&, Tree<Message>& card, CardArea card_area) {
        Rect area = card_area.content;
        card.add("OptionsEnabled", area.cut_top(tokens::ControlHeight, 0.0f),
                 CheckBox<Message>(showcase.options_enabled, "Enable the options under this").on_change(base::make<Message::OptionsEnabled>));
        for (size_t index = 0; index < Options.size(); index++) {
            std::string_view option = Options[index];
            Rect row = area.cut_top(tokens::ControlHeight, 0.0f);
            card.add(std::format("Option/{}", option), Rect(row.x + 28.0f, row.y, row.width - 28.0f, row.height),
                     CheckBox<Message>(showcase.options[index], std::string(option)).on_change([index](bool on) { return Message::Option{index, on}; }))
                .set_enabled(showcase.options_enabled);
        }
        area.cut_top(tokens::SpaceS, 0.0f);
        card.add("Tooltips", area.cut_top(tokens::ControlHeight, 0.0f), ToggleSwitch<Message>(showcase.tooltips, "Tooltips (F1)").on_change(base::make<Message::Tooltips>));
        card.add("DarkTheme", area.cut_top(tokens::ControlHeight, 0.0f), ToggleSwitch<Message>(dark, "Dark theme (F9)").on_change([](bool) { return Message::ToggleTheme{}; }));
        card.add("Filling", area.cut_top(tokens::ControlHeight, 0.0f), ToggleSwitch<Message>(showcase.filling, "Fill the progress bar").on_change(base::make<Message::Filling>));
    });
}

void choices(const Showcase& showcase, Ui& ui, Tree<Message>& tree, Rect rect) {
    Color accent = ui.theme().accent;
    Color fill = ui.theme().control_alt_fill_secondary;
    tree.card(ui, std::nullopt, rect, Card("Radio buttons and segments"), [&](Ui&, Tree<Message>& card, CardArea card_area) {
        Rect area = card_area.content;
        Rect radios = area.cut_top(RadioGroup<Message>::height(Sizes.size()), tokens::SpaceM);
        Rect preview = radios.cut_right(96.0f, 0.0f);
        card.add("Size", radios, RadioGroup<Message>(showcase.size, Sizes).on_select(base::make<Message::Size>));
        float side = 24.0f + 16.0f * static_cast<float>(showcase.size);
        card.draw(preview, Canvas([side, accent](Painter& painter, Rect rect) { painter.rounded(rect.centered(Size::square(side)), tokens::RadiusControl, accent); }));
        card.add("Alignment", area.cut_top(tokens::ControlHeight, tokens::SpaceS), Segmented<Message>(showcase.alignment, Alignments).on_select(base::make<Message::Alignment>));
        Rect line = area.cut_top(tokens::ControlHeight, tokens::SpaceM);
        Align align = std::array{Align::Start, Align::Center, Align::End}[showcase.alignment];
        card.draw(line, Canvas([fill](Painter& painter, Rect rect) { painter.rounded(rect, tokens::RadiusControl, fill); }));
        card.draw(line.inset(tokens::SpaceM), Text::label("Aligned text").align(align));
        std::vector<std::pair<Icon, std::string_view>> tools;
        for (auto [icon, tool] : std::views::zip(std::array{Icon::Brush, Icon::Fill, Icon::Erase, Icon::Eyedropper}, Tools)) {
            tools.emplace_back(icon, tool);
        }
        card.add("Tool", area.cut_top(tokens::ControlHeight, tokens::SpaceS), Segmented<Message>(showcase.tool, tools).on_select(base::make<Message::Tool>));
        card.draw(area.cut_top(tokens::Caption.line_height, 0.0f), Text::caption(std::format("{}: segments with icons, like a tool's modes", Tools[showcase.tool])));
    });
}

void sliders(const Showcase& showcase, Ui& ui, Tree<Message>& tree, Rect rect) {
    double time = ui.time();
    tree.card(ui, std::nullopt, rect, Card("Sliders and progress"), [&](Ui&, Tree<Message>& card, CardArea area) {
        Form form(area.content);
        float height = Slider<Message>::height(true);
        card.add("Volume", form.take(height), Slider<Message>(showcase.volume, 0.0, 100.0).step(1.0).header("Volume").show_value(0).on_change(base::make<Message::Volume>));
        card.add("Opacity", form.take(height),
                 Slider<Message>(showcase.opacity, 0.0, 1.0).step(0.001).large_step(0.1).header("Opacity").show_value(3).on_change(base::make<Message::Opacity>));
        card.add("Angle", form.take(height),
                 Slider<Message>(showcase.angle, 0.0, 360.0).step(0.1).large_step(15.0).header("Angle").show_value(1).suffix("°").on_change(base::make<Message::Angle>));
        card.add("DisabledSlider", form.take(height), Slider<Message>(30.0, 0.0, 100.0).header("Disabled").show_value(0)).set_enabled(false);
        float fill = showcase.filling ? static_cast<float>(std::fmod(time - showcase.fill_started, 4.0) / 4.0) : 0.78f;
        std::array<std::pair<std::string_view, std::optional<float>>, 3> bars{{
            {"At the volume", showcase.volume / 100.0f},
            {"Fills every 4 seconds", fill},
            {"Of unknown length", std::nullopt},
        }};
        for (auto [label, value] : bars) {
            Rect row = form.take(tokens::Body.line_height + tokens::SpaceXs + 4.0f);
            card.draw(row.top_strip(tokens::Body.line_height), Text::label(std::string(label)).secondary());
            if (value) {
                card.draw(row.top_strip(tokens::Body.line_height), Text::label(std::format("{:.0f}%", *value * 100.0f)).secondary().end());
            }
            card.draw(row.bottom_strip(4.0f), ProgressBar(value));
        }
        Rect rings = form.take(40.0f);
        card.draw(Rect(rings.x, rings.y, 40.0f, rings.height), ProgressRing(showcase.volume / 100.0f));
        card.draw(Rect(rings.x + 48.0f, rings.y, 40.0f, rings.height), ProgressRing(std::nullopt));
        card.draw(Rect(rings.x + 104.0f, rings.y, rings.width - 104.0f, rings.height), Text::caption("Progress rings, at the volume, and spinning"));
    });
}

void fields(const Showcase& showcase, Ui& ui, Tree<Message>& tree, Rect rect) {
    const Theme& theme = ui.theme();
    Color role_color = std::array{theme.accent, theme.success, theme.caution, theme.critical, theme.attention}[showcase.role];
    tree.card(ui, std::nullopt, rect, Card("Combo boxes and fields"), [&](Ui& ui, Tree<Message>& card, CardArea area) {
        Form form(area.content);
        Rect row = form.control(card, "A color of the theme");
        Rect swatch = row.cut_right(tokens::ControlHeight, tokens::SpaceS);
        card.add("Role", row, ComboBox<Message>(showcase.role, Roles).on_select(base::make<Message::Role>));
        card.draw(swatch, Canvas([role_color](Painter& painter, Rect rect) { painter.rounded(rect, tokens::RadiusControl, role_color); }));
        form.caption(ui, card, RoleUses[showcase.role]);
        row = form.control(card, "Your name");
        Rect greet = row.cut_right(96.0f, tokens::SpaceS);
        card.add("Name", row, TextBox<Message>(showcase.name).placeholder("Type a name").max_length(40).on_change(base::make<Message::Name>).on_submit(Message::Greet{}));
        card.add("Greet", greet, Button<Message>("Greet").accent().on_press(Message::Greet{}));
        row = form.control(card, "Search the list");
        card.add("Search", row, TextBox<Message>(showcase.search).placeholder("Search").button(Icon::Search).on_change(base::make<Message::Search>));
        auto [quantity, scale] = form.take(tokens::Body.line_height + 8.0f + tokens::ControlHeight).columns<2>(tokens::SpaceS);
        card.draw(quantity.top_strip(tokens::Body.line_height), Text::label("Quantity"));
        card.add("Quantity", quantity.bottom_strip(tokens::ControlHeight),
                 NumberBox<Message>(static_cast<double>(showcase.quantity)).range(0.0, 99.0).on_change(base::make<Message::Quantity>));
        card.draw(scale.top_strip(tokens::Body.line_height), Text::label("Scale"));
        card.add("Scale", scale.bottom_strip(tokens::ControlHeight),
                 NumberBox<Message>(showcase.scale).range(0.25, 4.0).step(0.25).decimals(2).on_change(base::make<Message::Scale>));
        if (const std::optional<std::string>& greeting = showcase.greeting) {
            card.add("Greeting", form.take(48.0f), InfoBar<Message>(Severity::Success, "", *greeting).on_close(Message::CloseGreeting{}));
        }
    });
}

void menus_card(const Showcase& showcase, Ui& ui, Tree<Message>& tree, Rect rect) {
    const Theme& theme = ui.theme();
    Color fill = theme.control_alt_fill_secondary;
    Color stroke = theme.control_strong_stroke;
    tree.card(ui, std::nullopt, rect, Card("Menus and dialogs"), [&](Ui&, Tree<Message>& card, CardArea card_area) {
        Form form(card_area.content);
        auto [ask, choice] = form.columns<2>(tokens::ControlHeight);
        card.add("Ask", ask, Button<Message>("Ask a question").icon(Icon::Help).on_press(Message::Ask{}));
        std::string result = showcase.dialog_result.value_or("Not asked yet");
        card.draw(choice, Text::caption(std::format("Answer: {}", result)));
        Rect area = form.take(64.0f);
        card.draw(area, Canvas([fill, stroke](Painter& painter, Rect rect) { painter.shape(rect, tokens::RadiusControl, fill, Border(stroke.fade(0.5f), 1.0f)); }));
        card.draw(area, Text::label("Right-click here for a context menu").secondary().center());
        card.add("ContextArea", area, View<Message>().sense(Sense(true, true, false, false, Keys::None)).kind("Area")).set_context_menu(context_menu());
        std::string chosen = showcase.menu_choice.value_or("Nothing yet");
        card.draw(form.take(tokens::Caption.line_height), Text::caption(std::format("Chose from a menu: {}", chosen)));
        Rect header = form.take(48.0f);
        card.add("Expander", header, Expander<Message>(showcase.expanded, "An expander").caption("Shows and hides what's under it").on_toggle(base::make<Message::Expand>));
        if (showcase.expanded) {
            Rect rest = form.rest();
            Rect content(header.x, header.bottom(), header.width, rest.bottom() - header.bottom());
            card.container(std::nullopt, content, ExpanderContent(), [&](Tree<Message>& expander) {
                expander.draw(ExpanderContent::inner(content), Text::paragraph("Expander content sits on the secondary card fill, under its header, as WinUI's does.").secondary());
            });
        }
    });
}

void dialog(const Showcase& showcase, Ui& ui, Tree<Message>& tree) {
    if (!showcase.dialog_open) {
        return;
    }
    Dialog<Message> dialog = Dialog<Message>("Save your work?", "This is a content dialog: it covers the page with smoke until you choose.")
                                 .primary("Save")
                                 .secondary("Don't save")
                                 .close("Cancel")
                                 .on_result(base::make<Message::Answer>);
    DialogArea area = dialog.area(ui);
    tree.add("Question", area.rect, std::move(dialog));
}

void type_ramp(Tree<Message>& tree, Rect area) {
    std::array<std::tuple<std::string_view, TextStyle, std::string_view>, 11> styles{{
        {"Caption · 12 on 16", tokens::Caption, "The quick brown fox jumps"},
        {"Body · 14 on 20", tokens::Body, "The quick brown fox jumps"},
        {"Body strong · 14 on 20", tokens::BodyStrong, "The quick brown fox jumps"},
        {"Body large · 18 on 24", tokens::BodyLarge, "The quick brown fox jumps"},
        {"Subtitle · 20 on 28", tokens::Subtitle, "The quick brown fox jumps"},
        {"Title · 28 on 36", tokens::Title, "The quick brown fox jumps"},
        {"Title large · 40 on 52", tokens::TitleLarge, "Fluent"},
        {"Display · 68 on 92", tokens::Display, "Fluent"},
        {"Mono · 13 on 20", tokens::Mono, "The quick brown fox jumps"},
        {"Mono strong · 13 on 20", tokens::Mono.weight(user_interface::Weight::Semibold), "The quick brown fox jumps"},
        {"Mono caption · 12 on 16", tokens::MonoCaption, "The quick brown fox jumps"},
    }};
    for (auto [label, style, sample] : styles) {
        Rect row = area.cut_top(style.line_height, tokens::SpaceS);
        tree.draw(row.left_strip(148.0f), Text::caption(std::string(label)));
        tree.draw(Rect(row.x + 148.0f, row.y, row.width - 148.0f, row.height), Text::label(std::string(sample)).style(style));
    }
    tree.draw(Rect(area.x, area.y + tokens::SpaceS, area.width, tokens::Body.line_height),
              Spans(base::vec(Run("Spans: ", tokens::Body, TextColor::Secondary{}), Run("several styles ", tokens::BodyStrong, TextColor::Primary{}),
                              Run("and colors ", tokens::Body, TextColor::Accent{}), Run("on a line", tokens::Caption, TextColor::Secondary{}))));
}

void icons(Tree<Message>& tree, Rect area) {
    size_t columns = 4;
    Size cell(area.width / static_cast<float>(columns), 40.0f);
    for (size_t index = 0; index < user_interface::AllIcons.size(); index++) {
        Icon icon = user_interface::AllIcons[index];
        float x = area.x + cell.width * static_cast<float>(index % columns);
        float y = area.y + cell.height * static_cast<float>(index / columns);
        if (y + cell.height > area.bottom()) {
            break;
        }
        tree.draw(Rect(x, y, 32.0f, cell.height), IconView(icon, tokens::IconLarge));
        tree.draw(Rect(x + 36.0f, y, cell.width - 40.0f, cell.height), Text::caption(std::string(user_interface::name(icon))));
    }
}

void theme_colors(Ui& ui, Tree<Message>& tree, Rect area) {
    std::vector<std::pair<std::string_view, Color>> roles = ui.theme().roles();
    float row_height = 26.0f;
    Color stroke = ui.theme().control_stroke_secondary;
    auto [left, right] = area.columns<2>(tokens::SpaceL);
    size_t per_column = base::saturating_cast<size_t>(std::floor(area.height / row_height));
    for (size_t index = 0; index < roles.size(); index++) {
        auto [name, color] = roles[index];
        Rect column = index < per_column ? left : right;
        float y = column.y + row_height * static_cast<float>(index % per_column);
        if (index >= per_column * 2) {
            break;
        }
        Rect swatch(column.x, y + 3.0f, 20.0f, 20.0f);
        tree.draw(swatch, Canvas([color, stroke](Painter& painter, Rect rect) { painter.shape(rect, tokens::RadiusControl, color, Border(stroke, 1.0f)); }));
        tree.draw(Rect(column.x + 28.0f, y, column.width - 120.0f, row_height), Text::caption(std::string(name)));
        tree.draw(Rect(column.x, y, column.width, row_height), Text::caption(color.to_hex()).end());
    }
}

void shapes(Ui& ui, Tree<Message>& tree, Rect area) {
    tree.draw(area, Canvas([theme = ui.theme()](Painter& painter, Rect area) {
                  auto [elevations, rest] = area.rows<2>(tokens::SpaceL);
                  std::array<Rect, 3> tiles = elevations.top_strip(120.0f).inset(tokens::SpaceL).columns<3>(tokens::SpaceL * 2.0f);
                  std::array<std::string_view, 3> names{"Tooltip · 16", "Flyout · 32", "Dialog · 128"};
                  for (auto [tile, elevation, name] : std::views::zip(tiles, std::array{tokens::ElevationTooltip, tokens::ElevationFlyout, tokens::ElevationDialog}, names)) {
                      painter.shadow(tile, tokens::RadiusOverlay, elevation);
                      painter.shape(tile, tokens::RadiusOverlay, theme.acrylic, Border(theme.surface_stroke_flyout, 1.0f));
                      painter.text_in(tile, name, tokens::Caption, theme.text_secondary, Align::Center);
                  }
                  float y = elevations.y + 160.0f;
                  std::array<Color, 8> data = theme.data.all();
                  for (size_t index = 0; index < data.size(); index++) {
                      float x = elevations.x + 24.0f + static_cast<float>(index) * 52.0f;
                      painter.circle(Point(x, y), 16.0f, data[index]);
                  }
                  y += 52.0f;
                  painter.line(Point(elevations.x + 8.0f, y), Point(elevations.right() - 8.0f, y + 24.0f), 2.0f, theme.accent);
                  painter.polyline({{Point(elevations.x + 8.0f, y + 56.0f), Point(elevations.x + 120.0f, y + 36.0f), Point(elevations.x + 240.0f, y + 72.0f),
                                     Point(elevations.x + 360.0f, y + 40.0f)}},
                                   2.0f, theme.data.orange);
                  Point center(rest.x + 48.0f, rest.y + 48.0f);
                  painter.ring(center, 40.0f, 4.0f, theme.control_strong_stroke);
                  painter.arc(center, 38.0f, 4.0f, 0.0f, 4.0f, theme.accent);
                  painter.polygon({{Point(rest.x + 140.0f, rest.y + 88.0f), Point(rest.x + 180.0f, rest.y + 8.0f), Point(rest.x + 220.0f, rest.y + 88.0f)}}, theme.data.green);
                  std::vector<Point> hexagon;
                  for (int index = 0; index < 6; index++) {
                      float angle = 2.0f * std::numbers::pi_v<float> * static_cast<float>(index) / 6.0f;
                      hexagon.push_back(Point(rest.x + 300.0f + 40.0f * std::cos(angle), rest.y + 48.0f + 40.0f * std::sin(angle)));
                  }
                  painter.polygon(hexagon, theme.data.purple);
                  Rect gradient(rest.x, rest.y + 120.0f, rest.width, 32.0f);
                  painter.shape(gradient, tokens::RadiusControl, Brush::horizontal(gradient, theme.accent, theme.data.pink), std::nullopt);
                  Rect vertical(rest.x, rest.y + 168.0f, 120.0f, 80.0f);
                  painter.shape(vertical, Corners::top(16.0f), Brush::vertical(vertical, theme.data.teal, theme.data.blue), std::nullopt);
                  Rect bordered(rest.x + 140.0f, rest.y + 168.0f, 120.0f, 80.0f);
                  painter.shape(bordered, tokens::RadiusControl, theme.control_fill, user_interface::border(user_interface::Elevation::Control, theme, bordered));
                  painter.text_in(bordered, "Elevation border", tokens::Caption, theme.text_secondary, Align::Center);
              }));
}

void tabs(const Showcase& showcase, Ui& ui, Tree<Message>& tree, Rect rect) {
    tree.card(ui, std::nullopt, rect, Card(""), [&](Ui& ui, Tree<Message>& card, CardArea card_area) {
        Rect area = card_area.content;
        card.add("Tabs", area.cut_top(48.0f, tokens::SpaceS), SelectorBar<Message>(showcase.tab, Tabs).on_select(base::make<Message::Tab>));
        switch (showcase.tab) {
        case 0:
            type_ramp(card, area);
            break;
        case 1:
            icons(card, area);
            break;
        case 2:
            theme_colors(ui, card, area);
            break;
        default:
            shapes(ui, card, area);
            break;
        }
    });
}

void info_bars(const Showcase& showcase, Ui& ui, Tree<Message>& tree, Rect rect) {
    tree.card(ui, std::nullopt, rect, Card("Info bars"), [&](Ui& ui, Tree<Message>& card, CardArea card_area) {
        Rect area = card_area.content;
        using Bar = std::pair<std::string_view, InfoBar<Message>>;
        std::array<Bar, 4> bars{
            Bar("Informational", InfoBar<Message>(Severity::Informational, "Informational", "Something to know.")),
            Bar("Success", InfoBar<Message>(Severity::Success, "Success", "It went well.")),
            Bar("Caution", InfoBar<Message>(Severity::Warning, "Caution", "Close me.").on_close(Message::CloseCaution{})),
            Bar("Error", InfoBar<Message>(Severity::Error, "Error", "It went wrong.")),
        };
        for (auto& [name, bar] : bars) {
            if (name == "Caution" && !showcase.caution_shown) {
                Rect row = area.cut_top(tokens::ControlHeight, tokens::SpaceS);
                card.add("BringBack", row.left_strip(160.0f), Button<Message>("Bring it back").on_press(Message::BringBackCaution{}));
                continue;
            }
            float height = bar.height(ui, area.width);
            card.add(name, area.cut_top(height, tokens::SpaceS), std::move(bar));
        }
    });
}

void colors(const Showcase& showcase, Ui& ui, Tree<Message>& tree, Rect rect) {
    tree.card(ui, std::nullopt, rect, Card("Color"), [&](Ui&, Tree<Message>& card, CardArea card_area) {
        Rect area = card_area.content;
        float picker_width = std::min(256.0f, area.width);
        Rect picker = area.cut_top(ColorPicker<Message>::height(picker_width), tokens::SpaceM).left_strip(picker_width);
        card.add("Picker", picker, ColorPicker<Message>(showcase.color).on_change(base::make<Message::Color>));
        Rect row = area.cut_top(tokens::ControlHeight, tokens::SpaceM);
        Rect field = row.cut_left(160.0f, tokens::SpaceM);
        card.add("Hex", field, ColorField<Message>(showcase.color).on_change(base::make<Message::Color>));
        // Swatches 24 pixels square and 8 apart, in the middle of the row's
        // height. The one picked is the one that's the color, if one is.
        std::array<Color, 8> colors;
        std::ranges::transform(Palette, colors.begin(), Color::hex);
        std::optional<size_t> picked;
        if (auto found = std::ranges::find(colors, showcase.color); found != colors.end()) {
            picked = static_cast<size_t>(found - colors.begin());
        }
        Rect swatches = row.pad(Insets(2.0f, 4.0f, 0.0f, 4.0f));
        card.add("Palette", swatches, user_interface::Palette<Message>(picked, colors).size(24.0f).gap(tokens::SpaceS).on_pick(base::make<Message::Swatch>));
    });
}

void list(const Showcase& showcase, Ui& ui, Tree<Message>& tree, Rect rect) {
    std::vector<size_t> matches = showcase.matches();
    std::string subtitle = std::format("{} of {} · Double-click to open", matches.size(), showcase.items.size());
    std::vector<std::string> names;
    names.reserve(matches.size());
    for (size_t item : matches) {
        names.push_back(showcase.items[item]);
    }
    std::optional<size_t> selected;
    if (showcase.selected) {
        if (auto found = std::ranges::find(matches, *showcase.selected); found != matches.end()) {
            selected = static_cast<size_t>(found - matches.begin());
        }
    }
    // The messages carry the item each row showed, the gesture's own fact,
    // not a row number for apply time to re-resolve.
    ListView<Message> list = ListView<Message>(matches.size(), 40.0f)
                                 .selection(selected)
                                 .on_select([picks = matches](size_t row) { return Message::Select{row < picks.size() ? picks[row] : std::numeric_limits<size_t>::max()}; })
                                 .on_activate([opens = matches](size_t row) { return Message::Activate{row < opens.size() ? opens[row] : std::numeric_limits<size_t>::max()}; })
                                 .labels([labels = names](size_t row) { return labels[row]; });
    tree.card(ui, std::nullopt, rect, Card("List").subtitle(std::move(subtitle)), [&](Ui& ui, Tree<Message>& card, CardArea area) {
        ListRows rows = list.rows(ui, "Items", area.content);
        card.container("Items", area.content, std::move(list), [&](Tree<Message>& items) {
            for (size_t row : rows.indices()) {
                Rect content = rows.content(row);
                items.draw(Rect(content.x, content.y, tokens::Icon, content.height), IconView(Icon::Document, tokens::Icon));
                items.draw(Rect(content.x + tokens::Icon + tokens::SpaceM, content.y, content.width - 80.0f, content.height), Text::label(names[row]));
                items.draw(content, Text::caption(std::format("#{}", matches[row] + 1)).tertiary().end());
            }
        });
    });
}

void interface_(const Showcase& showcase, Ui& ui, Tree<Message>& tree, Rect rect) {
    std::optional<Point> at = ui.pointer();
    std::string pointer = at ? std::format("{:.0f}, {:.0f}", at->x, at->y) : std::string("Not in the window");
    std::array<std::pair<std::string_view, std::string>, 8> rows{{
        {"Window focus", ui.window_focused() ? "Yes" : "No"},
        {"Pointer", std::move(pointer)},
        {"Under the pointer", std::string(ui.hovered_name().value_or("None"))},
        {"Pressed", std::string(ui.pressed_name().value_or("None"))},
        {"Keyboard focus", std::string(ui.focused_name().value_or("None"))},
        {"Open popup", std::string(ui.popup_name().value_or("None"))},
        {"Tooltip", std::string(ui.tooltip_name().value_or("None"))},
        {"Dialog", ui.modal() ? "Open" : "None"},
    }};
    std::vector<std::string> held_names;
    for (user_interface::Key key : ui.input().keys_held()) {
        held_names.push_back(key.name());
    }
    std::string held = held_names.empty() ? std::string("None") : base::join(held_names, ", ");
    const Theme& theme = ui.theme();
    Gamepad gamepad = ui.gamepad();
    std::string_view theme_name = user_interface::name(theme.kind);
    tree.card(ui, std::nullopt, rect, Card("Interface"), [&](Ui& ui, Tree<Message>& card, CardArea card_area) {
        Rect area = card_area.content;
        for (auto& [label, value] : rows) {
            card.draw(area.cut_top(24.0f, 0.0f), Field(std::string(label), 160.0f, std::move(value)));
        }
        card.draw(area.cut_top(24.0f, tokens::SpaceS), Field("Keys held", 160.0f, std::move(held)));
        card.draw(area.cut_top(tokens::BodyStrong.line_height, tokens::SpaceS), Text::heading("Gamepad"));
        Rect sticks = area.cut_top(96.0f, tokens::SpaceS);
        for (Point stick : {gamepad.left_stick, gamepad.right_stick}) {
            Rect box_rect = sticks.cut_left(96.0f, tokens::SpaceM);
            Color ring = theme.control_strong_stroke;
            Color divider = theme.divider;
            Color accent = theme.accent;
            card.draw(box_rect, Canvas([ring, divider, accent, stick](Painter& painter, Rect rect) {
                          Point center = rect.center();
                          painter.ring(center, 44.0f, 1.0f, ring);
                          painter.line(Point(center.x - 44.0f, center.y), Point(center.x + 44.0f, center.y), 1.0f, divider);
                          painter.line(Point(center.x, center.y - 44.0f), Point(center.x, center.y + 44.0f), 1.0f, divider);
                          painter.circle(Point(center.x + stick.x * 40.0f, center.y - stick.y * 40.0f), 5.0f, accent);
                      }));
        }
        for (float trigger : {gamepad.left_trigger, gamepad.right_trigger}) {
            Rect bar = sticks.cut_left(12.0f, tokens::SpaceS);
            Color track = theme.control_alt_fill_tertiary;
            Color accent = theme.accent;
            card.draw(bar, Canvas([track, accent, trigger](Painter& painter, Rect bar) {
                          painter.rounded(bar, 6.0f, track);
                          float height = bar.height * std::clamp(trigger, 0.0f, 1.0f);
                          painter.rounded(Rect(bar.x, bar.bottom() - height, bar.width, height), 6.0f, accent);
                      }));
        }
        std::vector<std::string_view> buttons = gamepad.buttons.names();
        std::string pressed = !gamepad.connected ? std::string("No gamepad") : buttons.empty() ? std::string("No buttons held") : base::join(buttons, ", ");
        card.draw(area.cut_top(tokens::Caption.line_height, tokens::SpaceL), Text::caption(std::move(pressed)));
        Rect badges = area.cut_top(24.0f, tokens::SpaceL);
        float width = Keycaps::width(ui.fonts(), "Ctrl+Shift+S");
        card.draw(Rect(badges.x, badges.center().y - 10.0f, width, 20.0f), Keycaps("Ctrl+Shift+S"));
        badges.cut_left(width + tokens::SpaceM, 0.0f);
        Size size = Badge::size(ui.fonts(), "3");
        card.draw(Rect(badges.x, badges.y, 24.0f, badges.height), Badge("3"));
        badges.cut_left(size.width + tokens::SpaceS + 8.0f, 0.0f);
        card.draw(Rect(badges.x, badges.y, 12.0f, badges.height), Badge::dot());
        card.draw(Rect(badges.x + 20.0f, badges.y, badges.width - 20.0f, badges.height), Text::caption("Keycaps, badges and a dot"));
        card.add("Details", area.cut_top(tokens::ControlHeight, tokens::SpaceL).left_strip(200.0f), Button<Message>("Details").icon(Icon::Info))
            .set_tooltip_lines(base::vec(TooltipLine::heading("Details"), TooltipLine::pair("Theme", std::string(theme_name)), TooltipLine(TooltipLine::Separator{}),
                                         TooltipLine::text("Tooltips can have lines, columns and separators.")));
        card.draw(area.cut_top(tokens::BodyStrong.line_height, tokens::SpaceS), Text::heading("Log"));
        size_t lines = std::min(base::saturating_cast<size_t>(std::floor(area.height / 20.0f)), showcase.log_.size());
        size_t shown = 0;
        for (const std::string& line : showcase.log_) {
            if (shown == lines) {
                break;
            }
            shown += 1;
            Rect row = area.cut_top(20.0f, 0.0f);
            card.draw(row, Text::label(line).style(tokens::Mono).secondary());
        }
    });
}

} // namespace

Tree<Message> view(const Showcase& showcase, Ui& ui) {
    Tree<Message> tree;
    tree.set_tooltips(showcase.tooltips);
    bool dark = ui.theme().kind == ThemeKind::Dark;
    std::string subtitle = std::format("Fluent 2 · WinUI 3 · {} theme", user_interface::name(ui.theme().kind));
    FrameStats stats = ui.stats();
    std::string fps = std::format("{:.0f} frames a second · {:.2f} ms a frame", stats.frames_per_second, stats.frame_time);
    const Environment& environment = ui.environment();
    std::string display = std::format("{} {}", environment.display, environment.display_name);
    std::vector<Status> status = base::vec(Status::shortcut("F1", "Tooltips"), Status::shortcut("Tab", "Next widget"), Status::shortcut("Ctrl+R", "Reset"),
                                           Status::shortcut("F9", "Theme"), Status::shortcut("F10", "VSync"), Status::shortcut("F11", "Fullscreen"), Status(Status::Spacer{}),
                                           Status::indicator(Severity::Success, std::move(fps)), Status::text(environment.gpu), Status::text(std::move(display)));
    Shell shell = tree.shell(ui, AppBar(Icon::Library, "UiShowcase").subtitle(std::move(subtitle)), true, std::move(status));
    Rect row = shell.menu_row.value();
    MenuBar<Message> menu(menus());
    float menu_width = menu.width(ui);
    tree.add("Menu", row.left_strip(menu_width), std::move(menu));
    Rect theme_button = Rect(row.right() - 32.0f, row.y, 32.0f, row.height).centered(Size::square(32.0f));
    tree.add("Theme", theme_button, Button<Message>::icon_only(dark ? Icon::Light : Icon::Moon).subtle().on_press(Message::ToggleTheme{})).set_tooltip("Switch the theme (F9)");
    Rect page = shell.page;
    auto [first, second, third, fourth, fifth] = page.columns<5>(tokens::CardGap);

    Rect column = first;
    buttons(showcase, ui, tree, column.cut_top(ButtonsHeight, tokens::CardGap));
    toggles(showcase, ui, tree, column.cut_top(TogglesHeight, tokens::CardGap));
    fields(showcase, ui, tree, column);

    column = second;
    sliders(showcase, ui, tree, column.cut_top(SlidersHeight, tokens::CardGap));
    choices(showcase, ui, tree, column.cut_top(ChoicesHeight, tokens::CardGap));
    menus_card(showcase, ui, tree, column);

    tabs(showcase, ui, tree, third);

    column = fourth;
    info_bars(showcase, ui, tree, column.cut_top(InfoHeight, tokens::CardGap));
    colors(showcase, ui, tree, column.cut_top(ColorHeight, tokens::CardGap));
    list(showcase, ui, tree, column);

    interface_(showcase, ui, tree, fifth);
    dialog(showcase, ui, tree);
    return tree;
}

} // namespace ui_showcase::cards
