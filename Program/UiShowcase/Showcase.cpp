// What the showcase's widgets show, what its shortcuts and menus do, its
// log, and its state for scripts. See Showcase.h.

#include "UiShowcase/Showcase.h"

#include "Base/Numbers.h"
#include "Base/Text.h"
#include "Base/Windows.h"
#include "UiShowcase/Cards.h"

#include <cmath>
#include <format>
#include <utility>

using user_interface::Context;
using user_interface::DialogResult;
using user_interface::Key;
using user_interface::Modifiers;
using user_interface::Startup;
using user_interface::ThemeKind;
using user_interface::Tree;
using user_interface::Ui;

namespace ui_showcase::showcase {

namespace {

constexpr size_t LogLength = 200;

// The text in lower case, of every alphabet, as Windows maps it:
// str::to_lowercase.
std::string to_lowercase(std::string_view text) {
    std::wstring wide = base::widen(text);
    if (wide.empty()) {
        return {};
    }
    std::wstring lower(wide.size(), L'\0');
    int length =
        LCMapStringEx(LOCALE_NAME_INVARIANT, LCMAP_LOWERCASE, wide.data(), static_cast<int>(wide.size()), lower.data(), static_cast<int>(lower.size()), nullptr, nullptr, 0);
    if (length <= 0) {
        return base::to_ascii_lowercase(text);
    }
    lower.resize(static_cast<size_t>(length));
    return base::narrow(lower);
}

} // namespace

// The list's 120 items.
std::vector<std::string> items() {
    static constexpr std::array<std::string_view, 12> Words{"Aurora", "Basalt", "Cobalt", "Dune", "Ember", "Fjord", "Glacier", "Harbor", "Iris", "Juniper", "Kelp", "Lagoon"};
    std::vector<std::string> made;
    made.reserve(120);
    for (size_t index = 0; index < 120; index++) {
        made.push_back(std::format("{} {}", Words[index % Words.size()], index / Words.size() + 1));
    }
    return made;
}

Showcase::Showcase(Startup&) {}

void Showcase::log(std::string line) {
    log_.push_front(std::move(line));
    log_.truncate(LogLength);
}

// The items that the search matches, by their index in the list.
std::vector<size_t> Showcase::matches() const {
    std::string wanted = to_lowercase(base::trim(search));
    std::vector<size_t> matched;
    for (size_t index = 0; index < items.size(); index++) {
        if (to_lowercase(items[index]).contains(wanted)) {
            matched.push_back(index);
        }
    }
    return matched;
}

void Showcase::reset(double time) {
    base::VecDeque<std::string> kept_log = std::exchange(log_, {});
    std::array<std::optional<std::string>, 3> kept_seen = std::exchange(seen, {});
    ThemeKind kept_theme = seen_theme;
    *this = Showcase();
    log_ = std::move(kept_log);
    seen = std::move(kept_seen);
    seen_theme = kept_theme;
    fill_started = time;
    log("Reset every widget");
}

void Showcase::toggle_theme(Context<Message>& ui) {
    ui.toggle_theme();
    // Logged here, on the frame that switches it: the theme is the one
    // the frame draws in, as the toggle leaves it.
    ThemeKind kind = ui.theme().kind;
    seen_theme = kind;
    log(std::format("{} theme", user_interface::name(kind)));
}

void Showcase::toggle_tooltips() {
    tooltips = !tooltips;
    log(std::format("Tooltips {}", tooltips ? "on" : "off"));
}

// What a menu's command does, by its label.
void Showcase::command(Context<Message>& ui, std::string_view label) {
    if (label == "Reset") {
        reset(ui.time());
    } else if (label == "Dark theme") {
        toggle_theme(ui);
    } else if (label == "Tooltips") {
        toggle_tooltips();
    } else if (label == "VSync") {
        ui.toggle_vertical_sync();
    } else if (label == "Fullscreen") {
        ui.toggle_fullscreen();
    } else if (label == "Ask a question") {
        dialog_open = true;
    } else if (label == "Pinned") {
        pinned = !pinned;
    }
    // After the command, so Reset's fresh state keeps the choice too.
    menu_choice = std::string(label);
    log(std::format("Chose {}", label));
}

std::string Showcase::title() const {
    return "UiShowcase";
}

// The shortcuts, and what the interface is on, which the log tells of
// when it changes.
std::vector<Message> Showcase::input(const Ui& ui) const {
    std::vector<Message> messages;
    if (ui.key_pressed(Key::F1)) {
        messages.push_back(Message::ToggleTooltips{});
    }
    if (ui.shortcut(Modifiers::Control, Key::R)) {
        messages.push_back(Message::Reset{});
    }
    bool theme_changed = ui.theme().kind != seen_theme;
    std::array<std::optional<std::string_view>, 3> names{ui.focused_name(), ui.pressed_name(), ui.popup_name()};
    bool names_changed = false;
    for (size_t index = 0; index < names.size(); index++) {
        names_changed |= names[index] != seen[index];
    }
    if (theme_changed || names_changed) {
        Message::Watch watch;
        for (size_t index = 0; index < names.size(); index++) {
            if (names[index]) {
                watch.names[index] = std::string(*names[index]);
            }
        }
        messages.push_back(std::move(watch));
    }
    return messages;
}

void Showcase::update(Message message, Context<Message>& ui) {
    base::match(
        message,
        [&](const Message::Press& press) {
            presses[press.index] += 1;
            if (press.index < 3) {
                log(std::format("Pressed {}", Buttons[press.index]));
            }
        },
        [&](const Message::Link&) {
            link_clicks += 1;
            log("Followed the link");
        },
        [&](const Message::Pin& pin) {
            pinned = pin.pinned;
            log(pin.pinned ? "Pinned" : "Unpinned");
        },
        [&](const Message::Add&) { count += 1; }, [&](const Message::Remove&) { count -= 1; }, [&](const Message::Command& chosen) { command(ui, chosen.label); },
        [&](const Message::ToggleTheme&) { toggle_theme(ui); }, [&](const Message::ToggleTooltips&) { toggle_tooltips(); },
        [&](const Message::Tooltips& switched) { tooltips = switched.on; },
        [&](const Message::OptionsEnabled& switched) {
            options_enabled = switched.enabled;
            log(std::format("Options {}", switched.enabled ? "enabled" : "disabled"));
        },
        [&](const Message::Option& option) { options[option.index] = option.on; },
        [&](const Message::Filling& switched) {
            filling = switched.filling;
            fill_started = ui.time();
        },
        [&](const Message::Size& picked) { size = picked.size; }, [&](const Message::Alignment& picked) { alignment = picked.alignment; },
        [&](const Message::Tool& picked) {
            tool = picked.tool;
            log(std::format("Tool: {}", Tools[picked.tool]));
        },
        [&](const Message::Volume& moved) { volume = static_cast<float>(moved.volume); }, [&](const Message::Opacity& moved) { opacity = moved.opacity; },
        [&](const Message::Angle& moved) { angle = static_cast<float>(moved.angle); },
        [&](const Message::Role& picked) {
            role = picked.role;
            log(std::format("Role: {}", cards::Roles[picked.role]));
        },
        [&](Message::Name& typed) { name = std::move(typed.name); },
        [&](const Message::Greet&) {
            std::string_view trimmed = base::trim(name);
            greeting = trimmed.empty() ? std::string("Hello, whoever you are!") : std::format("Hello, {}!", trimmed);
        },
        [&](const Message::CloseGreeting&) { greeting = std::nullopt; }, [&](Message::Search& typed) { search = std::move(typed.search); },
        [&](const Message::Quantity& typed) { quantity = base::saturating_cast<std::int32_t>(typed.quantity); }, [&](const Message::Scale& typed) { scale = typed.scale; },
        [&](const Message::Color& picked) {
            color = picked.color;
            log(std::format("Color {}", picked.color.to_hex()));
        },
        [&](const Message::Swatch& swatch) { color = user_interface::Color::hex(cards::Palette[swatch.index]); }, [&](const Message::Tab& picked) { tab = picked.tab; },
        [&](const Message::Select& select) { selected = select.item < items.size() ? std::optional(select.item) : std::nullopt; },
        [&](const Message::Activate& activate) {
            if (activate.item < items.size()) {
                opened = activate.item;
                log(std::format("Opened {}", items[activate.item]));
            }
        },
        [&](const Message::CloseCaution&) {
            caution_shown = false;
            log("Closed the caution");
        },
        [&](const Message::BringBackCaution&) { caution_shown = true; }, [&](const Message::Expand& toggled) { expanded = toggled.expanded; },
        [&](const Message::Ask&) { dialog_open = true; },
        [&](const Message::Answer& answered) {
            std::string_view answer;
            switch (answered.result) {
            case DialogResult::Primary:
                answer = "Save";
                break;
            case DialogResult::Secondary:
                answer = "Don't save";
                break;
            case DialogResult::Close:
                answer = "Cancel";
                break;
            }
            dialog_result = std::string(answer);
            dialog_open = false;
            log(std::format("Answered {}", answer));
        },
        [&](const Message::Reset&) { reset(ui.time()); },
        [&](Message::Watch& watch) {
            // The theme as it stands NOW, not as input() saw it: a
            // toggle earlier in this frame's queue already logged and
            // marked itself, so a stale snapshot would double-log.
            ThemeKind theme = ui.theme().kind;
            if (theme != seen_theme) {
                seen_theme = theme;
                log(std::format("{} theme", user_interface::name(theme)));
            }
            static constexpr std::array<std::string_view, 3> Verbs{"Focus on", "Press on", "Opened"};
            for (size_t index = 0; index < Verbs.size(); index++) {
                if (watch.names[index] != seen[index]) {
                    if (const std::optional<std::string>& now = watch.names[index]) {
                        log(std::format("{} {}", Verbs[index], *now));
                    }
                    seen[index] = watch.names[index];
                }
            }
        });
}

Tree<Message> Showcase::view(Ui& ui) const {
    return cards::view(*this, ui);
}

json::Value Showcase::state() const {
    auto name_of = [](std::span<const std::string_view> names, size_t index) {
        return json::Value(names[index]);
    };
    auto optional = [](std::optional<size_t> index) {
        return index ? json::Value(*index) : json::Value();
    };
    auto text = [](const std::optional<std::string>& value) {
        return value ? json::Value(*value) : json::Value();
    };
    json::Value::Object pressed;
    for (size_t index = 0; index < Buttons.size(); index++) {
        pressed.members.emplace_back(std::string(Buttons[index]), json::Value(presses[index]));
    }
    std::vector<std::string> newest;
    for (const std::string& line : log_) {
        if (newest.size() == 10) {
            break;
        }
        newest.push_back(line);
    }
    return json::object({
        {"Presses", json::Value(std::move(pressed))},
        {"Count", json::Value(static_cast<std::int64_t>(count))},
        {"Pinned", json::Value(pinned)},
        {"LinkClicks", json::Value(link_clicks)},
        {"OptionsEnabled", json::Value(options_enabled)},
        {"Options", json::Value(options)},
        {"Tooltips", json::Value(tooltips)},
        {"Filling", json::Value(filling)},
        {"Size", name_of(Sizes, size)},
        {"Alignment", name_of(Alignments, alignment)},
        {"Tool", name_of(Tools, tool)},
        {"Volume", json::Value(static_cast<double>(volume))},
        {"Opacity", json::Value(std::round(opacity * 1000.0) / 1000.0)},
        {"Angle", json::Value(std::round(static_cast<double>(angle) * 10.0) / 10.0)},
        {"Role", json::Value(cards::Roles[role])},
        {"Name", json::Value(name)},
        {"Search", json::Value(search)},
        {"Greeting", text(greeting)},
        {"Quantity", json::Value(static_cast<std::int64_t>(quantity))},
        {"Scale", json::Value(std::round(scale * 100.0) / 100.0)},
        {"Color", json::Value(color.to_hex())},
        {"Tab", name_of(Tabs, tab)},
        {"Selected", optional(selected)},
        {"Opened", optional(opened)},
        {"Matches", json::Value(matches().size())},
        {"CautionShown", json::Value(caution_shown)},
        {"Expanded", json::Value(expanded)},
        {"DialogOpen", json::Value(dialog_open)},
        {"DialogResult", text(dialog_result)},
        {"MenuChoice", text(menu_choice)},
        {"Log", json::Value(std::move(newest))},
    });
}

} // namespace ui_showcase::showcase
