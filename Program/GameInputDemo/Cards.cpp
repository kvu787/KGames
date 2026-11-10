#include "GameInputDemo/Cards.h"

#include "Base/Files.h"
#include "Base/Sum.h"
#include "Base/Text.h"
#include "Base/Vec.h"
#include "GameInput/GameInput.h"
#include "GameInputDemo/Devices.h"
#include "GameInputDemo/GameInputDemo.h"
#include "UserInterface/Design/Controls.h"
#include "UserInterface/Design/Icons.h"
#include "UserInterface/Design/Theme.h"
#include "UserInterface/Design/Tokens.h"
#include "UserInterface/Drawing/Geometry.h"
#include "UserInterface/Drawing/Painter.h"
#include "UserInterface/Widgets/Card.h"
#include "UserInterface/Widgets/ComboBox.h"
#include "UserInterface/Widgets/InfoBar.h"
#include "UserInterface/Widgets/ListView.h"
#include "UserInterface/Widgets/Scrolling.h"
#include "UserInterface/Widgets/Shell.h"
#include "UserInterface/Widgets/Text.h"

#include <algorithm>
#include <array>
#include <cmath>
#include <cstddef>
#include <filesystem>
#include <format>
#include <optional>
#include <span>
#include <utility>
#include <vector>

namespace game_input_demo {

using namespace game_input;
namespace controls = user_interface::controls;
namespace tokens = user_interface::tokens;
using user_interface::Align;
using user_interface::AppBar;
using user_interface::Badge;
using user_interface::Border;
using user_interface::Canvas;
using user_interface::Card;
using user_interface::CardArea;
using user_interface::Color;
using user_interface::ComboBox;
using user_interface::Divider;
using user_interface::Environment;
using user_interface::Field;
using user_interface::FrameStats;
using user_interface::Icon;
using user_interface::IconView;
using user_interface::Insets;
using user_interface::KeycapHeight;
using user_interface::Keycaps;
using user_interface::ListRows;
using user_interface::ListView;
using user_interface::Painter;
using user_interface::Point;
using user_interface::Rect;
using user_interface::ScrollArea;
using user_interface::Severity;
using user_interface::Shell;
using user_interface::Size;
using user_interface::Span;
using user_interface::Status;
using user_interface::StatusIcon;
using user_interface::Text;
using user_interface::TextColor;
using user_interface::TextStyle;
using user_interface::Theme;
using user_interface::Tooltip;
using user_interface::TooltipLine;
using user_interface::Ui;

namespace {

// The Latest input event card's details, and the Keyboard card's keys, which
// scroll if they don't fit.
constexpr std::string_view LatestDetails = "Latest";
constexpr std::string_view KeyRows = "Keys";

constexpr float TriggerBar = 48.0f;
constexpr float GraphGap = tokens::SpaceXxl;
constexpr float MousePad = 200.0f;
// A chip for a gamepad's or a mouse's button.
constexpr float ChipHeight = 24.0f;
constexpr float ChipMinWidth = 32.0f;
// A row of the Keyboard card: a keycap with the key's name, then its codes.
constexpr float KeyRow = 28.0f;
constexpr float KeyNameColumn = 112.0f;
// Where the values of the Latest input event card's and the Mouse card's
// fields start.
constexpr float LatestLabel = 200.0f;
constexpr float MouseLabel = 104.0f;
// The combo box is as wide as its widest choice, within these.
constexpr float GamepadComboBoxMinWidth = 200.0f;
constexpr float GamepadComboBoxMaxWidth = 420.0f;
// The combo box's choice that listens to no gamepad, as Listen names it.
constexpr std::string_view NoGamepad = "None";
// Taller than any card's details, to measure them in.
constexpr float Unbounded = 100'000.0f;

// What the colors mean: the left and right sticks and triggers, and each
// source of input, from the theme's colors for telling series apart.
struct Palette {
    Color left;
    Color right;
    Color keyboard;
    Color mouse;
    Color device;
    Color demo;

    static Palette of(const Theme& theme) {
        return Palette{
            .left = theme.data.blue,
            .right = theme.data.orange,
            .keyboard = theme.data.purple,
            .mouse = theme.data.teal,
            .device = theme.data.green,
            .demo = theme.text_secondary,
        };
    }

    Color source(Source source) const {
        switch (source) {
        case Source::Gamepad:
            return left;
        case Source::Keyboard:
            return keyboard;
        case Source::Mouse:
            return mouse;
        case Source::Device:
            return device;
        case Source::Demo:
            return demo;
        }
        base::unreachable();
    }
};

// After the app bar's title: whether the window has focus, which the demo
// needs to read input, and the frame rate.
void app_bar_status(Ui& ui, user_interface::Tree<Message>& tree, Rect bar) {
    FrameStats stats = ui.stats();
    std::string rate = std::format("{:.0f} frames a second · {:.2f} ms a frame", stats.frames_per_second, stats.frame_time);
    float rate_width = std::ceil(ui.measure(rate, tokens::Caption).width);
    tree.draw(bar.cut_right(rate_width, tokens::SpaceXxl), Text::caption(std::move(rate)));

    std::string_view status = "Not focused: input is ignored";
    Severity severity = Severity::Warning;
    if (ui.window_focused()) {
        status = "Focused: reading input";
        severity = Severity::Success;
    }
    float width = tokens::Icon + tokens::SpaceS + std::ceil(ui.measure(status, tokens::Caption).width);
    Rect area = bar.cut_right(width, tokens::SpaceXxl);
    Rect icon = area.cut_left(tokens::Icon, tokens::SpaceS);
    tree.draw(icon, StatusIcon(severity, tokens::Icon));
    tree.draw(area, Text::label(std::string(status)).style(tokens::Caption));
}

// The GameInput runtime that the demo loaded, and where from.
std::string runtime(const Input& input) {
    const Runtime* loaded = input.runtime();
    if (loaded == nullptr) {
        return "Runtime: none, the input comes from a script";
    }
    std::string version = std::format("{}.{}.{}.{}", loaded->version[0], loaded->version[1], loaded->version[2], loaded->version[3]);
    std::string name = base::display(loaded->path.filename());
    // A path has a folder that it's in unless it's empty, or only a drive
    // or its root.
    std::string location = loaded->path.has_relative_path() ? std::format(" in {}", base::display(loaded->path.parent_path())) : std::string();
    return std::format("Runtime: {} {}{}", name, version, location);
}

// The status bar: the shortcuts, the GameInput runtime, how many readings
// the demo handled, and the display that the window is on. A runtime deep
// in folders is cut short where it meets the items on the right.
std::vector<Status> status_items(const Ui& ui, const std::expected<Input, std::string>& input) {
    const Environment& environment = ui.environment();
    std::string display = environment.display.empty() ? std::string() : std::format("{} {}", environment.display, environment.display_name);
    std::string loaded = "GameInput isn't available";
    std::string readings;
    if (input) {
        loaded = runtime(*input);
        readings = std::format("Readings handled: {}", input->reading_count);
    }
    std::vector<Status> items = base::vec(Status::shortcut("F1", "Rumble test"), Status::shortcut("F2", "Clear the log"), Status::shortcut("F9", "Theme"),
                                          Status::shortcut("F10", "VSync"), Status::shortcut("F11", "Fullscreen"), Status::text(std::move(loaded)), Status(Status::Spacer{}));
    for (std::string* text : {&readings, &display}) {
        if (!text->empty()) {
            items.push_back(Status::text(std::move(*text)));
        }
    }
    return items;
}

// A dot in a series' color, and its name.
void legend(Painter& painter, const Theme& theme, Rect area, std::string_view text, Color color) {
    painter.circle(Point(area.x + 4.0f, area.center().y), 4.0f, color);
    painter.text_fit(area.pad(Insets(tokens::SpaceM, 0.0f, 0.0f, 0.0f)), text, tokens::BodyStrong, theme.text_primary, Align::Start);
}

// A stick's position, from -1 to 1 across and up, in a square with rings at
// its edge, half way, and the dead zone, if it has one.
void stick_graph(Painter& painter, const Theme& theme, Rect area, std::array<float, 2> value, float dead_zone, Color color) {
    painter.shape(area, tokens::RadiusControl, theme.control_alt_fill_secondary, Border(theme.control_stroke, 1.0f));
    Point center(std::floor(area.center().x), std::floor(area.center().y));
    float radius = area.width / 2.0f - tokens::SpaceS;
    // The mouse pad has no dead zone, and a ring of radius 0 would be a dot.
    if (dead_zone > 0.0f) {
        painter.circle(center, radius * dead_zone, theme.caution_background);
        painter.ring(center, radius * dead_zone, 1.0f, theme.caution.with_alpha(0.7f));
    }
    painter.fill(Rect(area.x + 1.0f, center.y, area.width - 2.0f, 1.0f), theme.divider);
    painter.fill(Rect(center.x, area.y + 1.0f, 1.0f, area.height - 2.0f), theme.divider);
    painter.ring(center, radius * 0.5f, 1.0f, theme.control_stroke_secondary);
    painter.ring(center, radius, 1.0f, theme.control_strong_stroke);

    // Stick y is up, and screen y down.
    Point position(center.x + value[0] * radius, center.y - value[1] * radius);
    painter.line(center, position, 2.0f, color.with_alpha(0.5f));
    painter.circle(position, 8.0f, theme.control_solid_fill);
    painter.circle(position, 6.0f, color);
}

// A trigger's value, as a bar that fills its track from the bottom, with a
// mark at each quarter.
void trigger_bar(Painter& painter, const Theme& theme, Rect area, float value, Color color) {
    painter.shape(area, tokens::RadiusControl, theme.control_alt_fill_secondary, Border(theme.control_stroke, 1.0f));
    for (int quarter = 1; quarter < 4; quarter++) {
        float y = std::round(area.y + area.height * static_cast<float>(quarter) / 4.0f);
        painter.fill(Rect(area.x + 1.0f, y, area.width * 0.25f, 1.0f), theme.control_stroke_secondary);
    }
    Rect track = area.inset(2.0f);
    float filled = std::round(track.height * std::clamp(value, 0.0f, 1.0f));
    if (filled > 0.0f) {
        painter.rounded(Rect(track.x, track.bottom() - filled, track.width, filled), tokens::RadiusControl - 2.0f, color);
    }
}

// A button's chip, which is in the accent color while it's held.
struct Chip {
    std::string_view label;
    bool held;
    // Whether the device has the button.
    bool present;
};

// Where chips go in the area: left to right, in rows that wrap at its width.
std::vector<Rect> chip_rects(Ui& ui, Rect area, std::span<const std::string_view> labels) {
    float x = area.x;
    float y = area.y;
    std::vector<Rect> rects;
    rects.reserve(labels.size());
    for (std::string_view label : labels) {
        float width = std::ceil(std::max(ui.measure(label, tokens::Caption).width + tokens::SpaceS * 2.0f, ChipMinWidth));
        if (x > area.x && x + width > area.right()) {
            x = area.x;
            y += ChipHeight + tokens::SpaceXs;
        }
        rects.push_back(Rect(x, y, width, ChipHeight));
        x += width + tokens::SpaceXs;
    }
    return rects;
}

// How tall chips with these labels are at a width.
float chips_height(Ui& ui, float width, std::span<const std::string_view> labels) {
    std::vector<Rect> rects = chip_rects(ui, Rect(0.0f, 0.0f, width, 0.0f), labels);
    return rects.empty() ? 0.0f : rects.back().bottom();
}

// Draws the chips, and returns how tall they are.
float draw_chips(Ui& ui, user_interface::Tree<Message>& tree, Rect area, std::vector<Chip> chips) {
    std::vector<std::string_view> labels;
    labels.reserve(chips.size());
    for (const Chip& chip : chips) {
        labels.push_back(chip.label);
    }
    std::vector<Rect> rects = chip_rects(ui, area, labels);
    float height = rects.empty() ? 0.0f : rects.back().bottom() - area.y;
    std::vector<std::pair<Rect, Chip>> placed;
    placed.reserve(chips.size());
    for (size_t index = 0; index < chips.size(); index++) {
        placed.emplace_back(rects[index], chips[index]);
    }
    tree.draw(area, Canvas([placed = std::move(placed)](Painter& painter, Rect) {
                  const Theme& theme = painter.theme();
                  for (const auto& [rect, chip] : placed) {
                      Color fill = Color::Transparent;
                      Color stroke = theme.control_stroke;
                      Color text = theme.text_disabled;
                      if (chip.held) {
                          fill = theme.accent;
                          stroke = theme.control_stroke_on_accent;
                          text = theme.text_on_accent;
                      } else if (chip.present) {
                          fill = theme.control_alt_fill_secondary;
                          stroke = theme.control_stroke;
                          text = theme.text_secondary;
                      }
                      painter.shape(rect, tokens::RadiusControl, fill, Border(stroke, 1.0f));
                      painter.text_in(rect, chip.label, tokens::Caption, text, Align::Center);
                  }
              }));
    return height;
}

// How tall the Gamepad card is, and the Latest input event card beside it:
// as tall as the Gamepad card's contents.
float gamepad_height(Ui& ui) {
    std::vector<std::string_view> labels;
    labels.reserve(GamepadButtons.size());
    for (const auto& [button, label] : GamepadButtons) {
        labels.push_back(label);
    }
    float chips = chips_height(ui, GamepadColumn - tokens::CardPadding * 2.0f, labels);
    return tokens::CardPadding + tokens::CardHeaderHeight + tokens::CardHeaderGap + tokens::Body.line_height + tokens::SpaceS + StickGraph + tokens::SpaceS +
           tokens::Mono.line_height * 2.0f + tokens::SpaceL + chips + tokens::SpaceL + tokens::Caption.line_height + tokens::CardPadding;
}

// The combo box in the Gamepad card's header, which shows which gamepad the
// demo listens to, and lists each connected gamepad, a picked one while it's
// disconnected, and none, to pick another. After it, a note on what the card
// shows, if it isn't the gamepad.
void gamepad_combo_box(Ui& ui, user_interface::Tree<Message>& tree, const Input& input, Rect header) {
    const Listen::Gamepad* listened = base::as<Listen::Gamepad>(input.listen);
    bool connected = listened != nullptr ? std::ranges::contains(input.gamepads(), listened->device.id, &Device::id) : true;
    std::vector<std::optional<Device>> choices;
    for (const Device& device : input.gamepads()) {
        choices.emplace_back(device);
    }
    if (listened != nullptr && !connected) {
        choices.emplace_back(listened->device);
    }
    choices.emplace_back(std::nullopt);
    std::vector<std::string_view> labels;
    labels.reserve(choices.size());
    for (const std::optional<Device>& choice : choices) {
        labels.push_back(choice ? std::string_view(choice->label) : NoGamepad);
    }
    std::optional<size_t> chosen = listened != nullptr ? std::optional(listened->device.id) : std::nullopt;
    size_t selected = 0;
    for (size_t index = 0; index < choices.size(); index++) {
        std::optional<size_t> id = choices[index] ? std::optional(choices[index]->id) : std::nullopt;
        if (id == chosen) {
            selected = index;
            break;
        }
    }
    float width = std::min(std::clamp(ComboBox<Message>::width(ui, labels), GamepadComboBoxMinWidth, GamepadComboBoxMaxWidth), header.width);
    Rect rect(header.x, header.y, width, tokens::ControlHeight);
    ComboBox<Message> combo = ComboBox<Message>(selected, labels).on_select([choices = std::move(choices)](size_t index) {
        if (const std::optional<Device>& device = choices[index]) {
            return Message(Message::Listen{Listen::Gamepad{*device}});
        } else {
            return Message(Message::Listen{Listen::NoGamepad{}});
        }
    });
    tree.add(GamepadComboBox, rect, std::move(combo));

    // Until a choice is picked, the demo switches to a gamepad as soon as one
    // connects.
    std::string_view note;
    bool caution = false;
    if (listened != nullptr && !connected && input.listen_picked) {
        note = "Disconnected: waiting for it to reconnect";
        caution = true;
    } else if (listened != nullptr && !connected) {
        note = "Disconnected: waiting for a gamepad to connect";
        caution = true;
    } else if (listened != nullptr) {
        note = "";
        caution = false;
    } else if (!input.listen_picked) {
        note = "No gamepad connected";
        caution = true;
    } else {
        note = "Gamepad input is ignored";
        caution = false;
    }
    Rect area(rect.right() + tokens::SpaceM, header.y, std::max(header.right() - rect.right() - tokens::SpaceM, 0.0f), header.height);
    if (caution) {
        Rect icon = area.cut_left(tokens::Icon, tokens::SpaceS);
        tree.draw(icon, StatusIcon(Severity::Warning, tokens::Icon));
    }
    Text text = caution ? Text::label(std::string(note)) : Text::label(std::string(note)).secondary();
    tree.draw(area, std::move(text));
}

// The Gamepad card: a graph for each stick, a bar for each trigger, the raw
// values, and a chip for each button, which lights up while it's held.
void gamepad(Ui& ui, user_interface::Tree<Message>& tree, const Input& input, Rect rect) {
    const GamepadView& view = input.gamepad;
    GameInputGamepadState state = view.state;
    float rumble_dead_zone = input.rumble_dead_zone;
    std::string_view rumble = input.rumble_enabled ? "on" : "off";
    std::string status_line = std::format("Readings: {}/s   ·   Rumble test: {} (F1)   ·   Motors: {}", view.rate.per_second, rumble, rumble_motor_names(view.rumble_motors));
    std::vector<Chip> chips;
    chips.reserve(GamepadButtons.size());
    for (const auto& [button, label] : GamepadButtons) {
        chips.push_back(Chip{
            .label = label,
            .held = (state.buttons & button) != 0,
            .present = !view.layout || (*view.layout & button) != 0,
        });
    }

    tree.card(ui, std::nullopt, rect, Card("Gamepad"), [&](Ui& ui, user_interface::Tree<Message>& card, CardArea area) {
        gamepad_combo_box(ui, card, input, area.header);
        Rect content = area.content;
        Rect captions = content.cut_top(tokens::Body.line_height, tokens::SpaceS);
        Rect plots = content.cut_top(StickGraph, tokens::SpaceS);
        Rect values = content.cut_top(tokens::Mono.line_height * 2.0f, tokens::SpaceL);
        Rect status = content.cut_bottom(tokens::Caption.line_height, tokens::SpaceL);
        Rect left = plots.cut_left(StickGraph, GraphGap);
        Rect right = plots.cut_left(StickGraph, GraphGap);
        Rect left_trigger = plots.cut_left(TriggerBar, GraphGap);
        Rect right_trigger = plots.cut_left(TriggerBar, GraphGap);
        std::array<std::array<float, 2>, 2> sticks{{
            {state.leftThumbstickX, state.leftThumbstickY},
            {state.rightThumbstickX, state.rightThumbstickY},
        }};
        card.draw(area.content, Canvas([left, right, left_trigger, right_trigger, captions, values, sticks, state, rumble_dead_zone](Painter& painter, Rect) {
                      const Theme& theme = painter.theme();
                      Palette colors = Palette::of(theme);
                      struct Series {
                          Rect column;
                          std::string_view caption;
                          Color color;
                      };
                      for (const Series& series : {
                               Series{left, "Left stick", colors.left},
                               Series{right, "Right stick", colors.right},
                               Series{left_trigger, "LT", colors.left},
                               Series{right_trigger, "RT", colors.right},
                           }) {
                          Rect caption(series.column.x, captions.y, series.column.width + GraphGap, captions.height);
                          legend(painter, theme, caption, series.caption, series.color);
                      }
                      stick_graph(painter, theme, left, sticks[0], rumble_dead_zone, colors.left);
                      stick_graph(painter, theme, right, sticks[1], rumble_dead_zone, colors.right);
                      trigger_bar(painter, theme, left_trigger, state.leftTrigger, colors.left);
                      trigger_bar(painter, theme, right_trigger, state.rightTrigger, colors.right);

                      // The raw values, in Cascadia Mono, so that their digits
                      // stay put.
                      for (const auto& [column, stick] : {std::pair(left, sticks[0]), std::pair(right, sticks[1])}) {
                          float x = stick[0];
                          float y = stick[1];
                          painter.text(Point(column.x, values.y), std::format("x {:+.3f}  y {:+.3f}", x, y), tokens::Mono, theme.text_primary);
                          float distance = std::hypot(x, y);
                          // Radians as degrees, with the factor that Rust's to_degrees has.
                          float degrees = std::atan2(y, x) * 57.2957795130823208767981548141051703f;
                          std::string polar = std::format("r {:.3f}  {:+.0f}°", distance, degrees);
                          painter.text(Point(column.x, values.y + tokens::Mono.line_height), polar, tokens::Mono, theme.text_secondary);
                      }
                      for (const auto& [column, value] : {std::pair(left_trigger, state.leftTrigger), std::pair(right_trigger, state.rightTrigger)}) {
                          painter.text(Point(column.x, values.y), std::format("{:.3f}", value), tokens::Mono, theme.text_primary);
                      }
                  }));
        // A chip for each button. Those that the gamepad says it lacks are
        // faint.
        draw_chips(ui, card, content, std::move(chips));
        card.draw(status, Text::caption(std::move(status_line)));
    });
}

Icon source_icon(Source source) {
    switch (source) {
    case Source::Gamepad:
        return Icon::Gamepad;
    case Source::Keyboard:
        return Icon::Keyboard;
    case Source::Mouse:
        return Icon::Mouse;
    case Source::Device:
    case Source::Demo:
        return Icon::Info;
    }
    base::unreachable();
}

// Lays the event's details out from the top of the area, into the tree if
// one is given, so that they can be measured first. Returns how tall they
// are.
float latest_details(Ui& ui, const LatestEvent& event, Rect area, user_interface::Tree<Message>* tree) {
    Rect rest = area;

    Rect heading = rest.cut_top(tokens::Body.line_height, tokens::SpaceS);
    if (tree != nullptr) {
        Rect icon = heading.cut_left(tokens::Icon, tokens::SpaceS);
        Color color = Palette::of(ui.theme()).source(event.source);
        tree->draw(icon, IconView(source_icon(event.source), tokens::Icon).color(TextColor::Custom{color}));
        tree->draw(heading, Text::heading(std::format("{} reading #{}", name(event.source), event.number)));
    }
    std::string timestamp = std::format("{} µs", event.timestamp);
    std::optional<std::string> interval = event.interval ? std::optional(std::format("{:.3f} ms", *event.interval)) : std::nullopt;
    std::vector<std::pair<std::string_view, std::string_view>> fields{
        {"Device", event.device},
        {"Reading has", event.kinds},
        {"Timestamp", timestamp},
    };
    if (interval) {
        fields.emplace_back("Since its previous reading", *interval);
    }
    for (const auto& [label, value] : fields) {
        Rect row = rest.cut_top(tokens::Body.line_height, tokens::SpaceXs);
        if (tree != nullptr) {
            tree->draw(row, Field(std::string(label), LatestLabel, std::string(value)));
        }
    }
    if (!interval) {
        std::string_view text = "No earlier reading from this device, so this is compared with an idle state.";
        float height = ui.paragraph_height(rest.width, text, tokens::Body);
        Rect row = rest.cut_top(height, tokens::SpaceXs);
        if (tree != nullptr) {
            tree->draw(row, Text::paragraph(std::string(text)).secondary());
        }
    }

    Rect changed_divider = rest.cut_top(tokens::SpaceL, tokens::SpaceXs);
    Rect changed_title = rest.cut_top(tokens::Body.line_height, tokens::SpaceXs);
    if (tree != nullptr) {
        tree->draw(changed_divider.bottom_strip(1.0f), Divider());
        tree->draw(changed_title, Text::heading("Changed"));
    }
    for (const std::string& change : event.changes) {
        // A bullet, and the change after it, which may wrap.
        Rect text = rest.pad(Insets(tokens::SpaceXs + tokens::SpaceM, 0.0f, 0.0f, 0.0f));
        float height = ui.paragraph_height(text.width, change, tokens::Body);
        Rect row = rest.cut_top(height, tokens::SpaceXs);
        if (tree != nullptr) {
            tree->draw(Rect(row.x + tokens::SpaceXs, row.y, tokens::SpaceM, tokens::Body.line_height), Text::label("•").secondary());
            tree->draw(Rect(text.x, row.y, text.width, height), Text::paragraph(change));
        }
    }

    Rect state_divider = rest.cut_top(tokens::SpaceL, tokens::SpaceXs);
    Rect state_title = rest.cut_top(tokens::Body.line_height, tokens::SpaceXs);
    if (tree != nullptr) {
        tree->draw(state_divider.bottom_strip(1.0f), Divider());
        tree->draw(state_title, Text::heading("State now"));
    }
    for (const std::string& line : event.state) {
        Rect row = rest.cut_top(tokens::Mono.line_height, 0.0f);
        if (tree != nullptr) {
            tree->draw(row, Text::label(line).style(tokens::Mono));
        }
    }
    return rest.y - area.y;
}

// The Latest input event card: the newest reading that changed something,
// described in full. Details that don't fit scroll.
void latest(Ui& ui, user_interface::Tree<Message>& tree, const Input& input, Rect rect) {
    Card titled = input.latest ? Card("Latest input event").subtitle(std::format("at {:.6f} s", input.latest->time)) : Card("Latest input event");
    tree.card(ui, std::nullopt, rect, std::move(titled), [&](Ui& ui, user_interface::Tree<Message>& card, CardArea card_area) {
        Rect area = card_area.content;
        if (!input.latest) {
            card.draw(area, Text::paragraph("Waiting for input. Press a key, move the mouse, or use a gamepad.").secondary());
            return;
        }
        const LatestEvent& event = *input.latest;
        float height = latest_details(ui, event, Rect(area.x, area.y, area.width, Unbounded), nullptr);
        if (height > area.height) {
            float width = area.width - controls::ScrollBarSize;
            height = latest_details(ui, event, Rect(area.x, area.y, width, Unbounded), nullptr);
        }
        Rect content = ScrollArea::content(ui, LatestDetails, area, height);
        card.container(LatestDetails, area, ScrollArea(height), [&](user_interface::Tree<Message>& scroll) { latest_details(ui, event, content, &scroll); });
    });
}

// The Keyboard card: the keys held on the keyboard that last changed them,
// each with its codes, and the character it types. Keys that don't fit
// scroll.
void keyboard(Ui& ui, user_interface::Tree<Message>& tree, const Input& input, Rect rect) {
    const KeyboardView& view = input.keyboard;
    tree.card(ui, std::nullopt, rect, Card("Keyboard").subtitle(view.label), [&](Ui& ui, user_interface::Tree<Message>& card, CardArea card_area) {
        Rect area = card_area.content;
        Rect status = area.cut_bottom(tokens::Caption.line_height, tokens::SpaceM);
        card.draw(status, Text::caption(std::format("Readings: {}/s", view.rate.per_second)));

        Rect heading = area.cut_top(tokens::Body.line_height, tokens::SpaceS);
        float title_width = ui.measure("Keys down", tokens::BodyStrong).width;
        card.draw(heading, Text::heading("Keys down"));
        if (view.keys.empty()) {
            card.draw(area.top_strip(tokens::Body.line_height), Text::label("None").tertiary());
            return;
        }
        // The badge is centered on its point, and a count of one digit is
        // round.
        Point center(heading.x + title_width + tokens::SpaceS + controls::InfoBadgeHeight / 2.0f, heading.center().y);
        card.draw(Rect::from_center(center, Size::square(controls::InfoBadgeHeight * 2.0f)), Badge(std::format("{}", view.keys.size())));
        float height = static_cast<float>(view.descriptions.size()) * KeyRow;
        Rect rows = ScrollArea::content(ui, KeyRows, area, height);
        card.container(KeyRows, area, ScrollArea(height), [&](user_interface::Tree<Message>& scroll) {
            for (const auto& [key, details] : view.descriptions) {
                Rect row = rows.cut_top(KeyRow, 0.0f);
                float cap_top = row.y + std::round((KeyRow - KeycapHeight) / 2.0f);
                scroll.draw(Rect(row.x, cap_top, KeyNameColumn, KeycapHeight), Keycaps(key));
                scroll.draw(row.pad(Insets(KeyNameColumn, 0.0f, 0.0f, 0.0f)), Text::label(details).style(tokens::Mono));
            }
        });
    });
}

// The Mouse card: the buttons held on the mouse that last changed, its
// motion and wheel, and a pad that moves with its raw motion, which has no
// pointer acceleration.
void mouse(Ui& ui, user_interface::Tree<Message>& tree, const Input& input, Rect rect) {
    const MouseView& view = input.mouse;
    GameInputMouseState state = view.state;
    std::array<float, 2> pad_value{view.pad[0], -view.pad[1]};
    tree.card(ui, std::nullopt, rect, Card("Mouse").subtitle(view.label), [&](Ui& ui, user_interface::Tree<Message>& card, CardArea card_area) {
        Rect area = card_area.content;
        Rect status = area.cut_bottom(tokens::Caption.line_height, tokens::SpaceM);
        std::string readings = std::format("Readings: {}/s", view.rate.per_second);
        if (view.sample_rate) {
            readings += std::format("   ·   Device reports {} Hz", *view.sample_rate);
        }
        card.draw(status, Text::caption(std::move(readings)));

        Rect pad_column = area.cut_right(MousePad, tokens::SpaceL);
        Rect caption = pad_column.cut_top(tokens::Body.line_height, tokens::SpaceS);
        Rect pad = pad_column.top_strip(MousePad);
        card.draw(Rect(caption.x, caption.y, caption.width, pad.bottom() - caption.y), Canvas([caption, pad, pad_value](Painter& painter, Rect) {
                      const Theme& theme = painter.theme();
                      Palette colors = Palette::of(theme);
                      legend(painter, theme, caption, "Raw motion", colors.mouse);
                      stick_graph(painter, theme, pad, pad_value, 0.0f, colors.mouse);
                  }));

        Rect heading = area.cut_top(tokens::Body.line_height, tokens::SpaceS);
        card.draw(heading, Text::heading("Buttons"));
        std::vector<Chip> chips;
        chips.reserve(MouseButtons.size());
        for (const auto& [button, label] : MouseButtons) {
            chips.push_back(Chip{
                .label = label,
                .held = (state.buttons & button) != 0,
                .present = true,
            });
        }
        float height = draw_chips(ui, card, area, std::move(chips));
        area.cut_top(height, tokens::SpaceL);
        std::vector<std::pair<std::string_view, std::string>> fields;
        fields.emplace_back("Last motion", std::format("({:+}, {:+})", view.delta[0], view.delta[1]));
        fields.emplace_back("Total motion", std::format("({}, {})", state.positionX, state.positionY));
        fields.emplace_back("Wheel total", std::format("({}, {})", state.wheelX, state.wheelY));
        if ((state.positions & GameInputMouseAbsolutePosition) != 0) {
            fields.emplace_back("Absolute", std::format("({}, {})", state.absolutePositionX, state.absolutePositionY));
        }
        for (auto& [label, value] : fields) {
            card.draw(area.cut_top(tokens::Body.line_height, tokens::SpaceXs), Field(std::string(label), MouseLabel, std::move(value)));
        }
    });
}

Icon device_icon(const Device& device) {
    if (device.is_gamepad()) {
        return Icon::Gamepad;
    } else if ((device.supported_input & GameInputKindKeyboard) != 0) {
        return Icon::Keyboard;
    } else {
        return Icon::Mouse;
    }
}

// Everything that GameInput and Windows say about a device, in a tooltip on
// its row, in three parts: what GameInput says, Windows' node for it, and
// what that's connected through. Long values, like the device's path, wrap.
Tooltip device_tooltip(const Device& device) {
    std::vector<TooltipLine> lines = base::vec(TooltipLine::heading(device.label));
    for (Section& section : device.details()) {
        lines.push_back(TooltipLine::Separator{});
        lines.push_back(TooltipLine::heading(std::move(section.title)));
        for (auto& [label, value] : section.rows) {
            lines.push_back(TooltipLine::Pair{std::move(label), std::move(value)});
        }
    }
    return Tooltip{
        .lines = std::move(lines),
        .max_width = DeviceTooltipWidth,
    };
}

// A device's row in the list: an icon for what it is, its label, and its
// family and kinds of input, in its content area.
void device_row(Ui& ui, user_interface::Tree<Message>& tree, const Input& input, const Device& device, Rect row, Rect content) {
    const Theme& theme = ui.theme();
    bool listened = input.gamepad.device == device.id;
    TextStyle style = tokens::Body;
    Color color = theme.text_primary;
    Color icon_color = theme.text_secondary;
    if (listened) {
        style = tokens::BodyStrong;
        color = theme.accent_text;
        icon_color = theme.accent_text;
    } else if (!input.listens_to(device)) {
        style = tokens::Body;
        color = theme.text_tertiary;
        icon_color = theme.text_tertiary;
    }
    std::string details = std::format("{}: {}", device.family_name(), device.kinds());
    if (device.xinput_index) {
        details += std::format(", XInput #{}", *device.xinput_index);
    }
    if (device.rumble_motors != 0) {
        details += ", rumble";
    }

    if (listened) {
        // The list's selection pill, on its row's backplate, though the
        // combo box picks the gamepad, and not the list.
        Rect plate = row.pad(controls::ListItemBackplateMargin);
        Rect pill(plate.x, plate.center().y - controls::SelectionPillLength / 2.0f, controls::SelectionPillThickness, controls::SelectionPillLength);
        Color accent = theme.accent;
        tree.draw(pill, Canvas([accent](Painter& painter, Rect area) { painter.rounded(area, controls::SelectionPillThickness / 2.0f, accent); }));
    }
    tree.draw(Rect(content.x, content.y, tokens::Icon, content.height), IconView(device_icon(device), tokens::Icon).color(TextColor::Custom{icon_color}));
    float details_width = std::ceil(ui.measure(details, tokens::Caption).width);
    Rect label_area = content.pad(Insets(tokens::Icon + tokens::SpaceM, 0.0f, details_width + tokens::SpaceL, 0.0f));
    tree.draw(label_area, Text::label(device.label).style(style).color(TextColor::Custom{color}));
    tree.draw(content, Text::caption(std::move(details)).end());
}

// The Devices card: a list with a row for each connected device, which
// shows a tooltip of everything known about it while the pointer rests on
// it, and scrolls if they don't all fit. The gamepad that the Gamepad card
// shows is in the accent color, with the pill of a selection, and the
// gamepads that the demo ignores are faint.
void devices(Ui& ui, user_interface::Tree<Message>& tree, const Input& input, Rect rect) {
    std::string count = std::format("{} connected", input.devices.size());
    tree.card(ui, std::nullopt, rect, Card("Devices").subtitle(std::move(count)), [&](Ui& ui, user_interface::Tree<Message>& card, CardArea area) {
        if (input.devices.empty()) {
            card.draw(area.content.top_strip(tokens::Body.line_height), Text::label("None yet").tertiary());
            return;
        }
        std::vector<std::string> labels;
        labels.reserve(input.devices.size());
        for (const Device& device : input.devices) {
            labels.push_back(device.label);
        }
        std::vector<Device> tooltip_devices = input.devices;
        ListView<Message> list =
            ListView<Message>(input.devices.size(), controls::ListItemHeight)
                .labels([labels = std::move(labels)](size_t index) { return labels[index]; })
                .tooltips([tooltip_devices = std::move(tooltip_devices)](size_t index) -> std::optional<Tooltip> { return device_tooltip(tooltip_devices[index]); });
        ListRows rows = list.rows(ui, DeviceRows, area.content);
        card.container(DeviceRows, area.content, std::move(list), [&](user_interface::Tree<Message>& items) {
            for (size_t index : rows.indices()) {
                device_row(ui, items, input, input.devices[index], rows.row(index), rows.content(index));
            }
        });
    });
}

// The Event log card: everything above as it happens, newest first, in
// Cascadia Mono, so that its columns line up: when, where from, and what.
void log(Ui& ui, user_interface::Tree<Message>& tree, const Input& input, Rect rect) {
    // An entry's line, where its source and its text start in it, and its
    // source.
    struct Entry {
        std::string line;
        size_t source_at;
        size_t text_at;
        Source source;
    };
    std::vector<Entry> entries;
    entries.reserve(input.log_.size());
    for (const LogEntry& entry : input.log_) {
        entries.push_back(Entry{entry.line, entry.source_at, entry.text_at, entry.source});
    }
    tree.card(ui, std::nullopt, rect, Card("Event log").subtitle("Newest first   ·   a run of motion from one device shares a line"),
              [&](Ui&, user_interface::Tree<Message>& card, CardArea card_area) {
                  Rect area = card_area.content;
                  if (entries.empty()) {
                      card.draw(area.top_strip(tokens::Body.line_height), Text::label("Nothing yet").tertiary());
                      return;
                  }
                  card.draw(area, Canvas([area, entries = std::move(entries)](Painter& painter, Rect) {
                                const Theme& theme = painter.theme();
                                Palette colors = Palette::of(theme);
                                Rect rest = area;
                                for (const Entry& entry : entries) {
                                    if (rest.height < tokens::Mono.line_height) {
                                        break;
                                    }
                                    Rect row = rest.cut_top(tokens::Mono.line_height, 0.0f);
                                    // The time and the source, then what happened, cut short
                                    // to fit.
                                    std::string_view line = entry.line;
                                    float lead = painter.measure(line.substr(0, entry.text_at), tokens::Mono).width;
                                    std::string text = painter.fit(line.substr(entry.text_at), tokens::Mono, std::max(row.width - lead, 0.0f)).into_owned();
                                    std::array<Span, 3> spans{
                                        Span(line.substr(0, entry.source_at), tokens::Mono, theme.text_secondary),
                                        Span(line.substr(entry.source_at, entry.text_at - entry.source_at), tokens::Mono, colors.source(entry.source)),
                                        Span(text, tokens::Mono, theme.text_primary),
                                    };
                                    painter.spans(row.origin(), spans);
                                }
                            }));
              });
}

} // namespace

user_interface::Tree<Message> view(Ui& ui, const std::expected<Input, std::string>& input) {
    user_interface::Tree<Message> tree;
    // No widget takes keyboard focus, so that the keys stay the user's, to
    // try the Keyboard card with. The combo box's list still takes the
    // arrow keys while it's open.
    tree.set_focusable(false);
    std::vector<Status> status = status_items(ui, input);
    Shell shell = tree.shell(ui, AppBar(Icon::Gamepad, std::string(Name)).subtitle("Gamepads, keyboards and mice, as GameInput reads them"), false, std::move(status));
    app_bar_status(ui, tree, shell.commands);
    // Without GameInput, a notice says why, and there's nothing to show.
    if (!input) {
        return tree;
    }
    Rect rest = shell.page;
    Rect first = rest.cut_left(GamepadColumn, tokens::CardGap);
    Rect second = rest.cut_left(LatestColumn, tokens::CardGap);
    float height = gamepad_height(ui);
    gamepad(ui, tree, *input, first.cut_top(height, tokens::CardGap));
    auto [keyboard_area, mouse_area] = first.columns<2>(tokens::CardGap);
    keyboard(ui, tree, *input, keyboard_area);
    mouse(ui, tree, *input, mouse_area);
    latest(ui, tree, *input, second.cut_top(height, tokens::CardGap));
    devices(ui, tree, *input, second);
    log(ui, tree, *input, rest);
    return tree;
}

} // namespace game_input_demo
