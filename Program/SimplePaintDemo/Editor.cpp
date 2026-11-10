#include "SimplePaintDemo/Editor.h"

#include "Base/Files.h"
#include "Base/Panic.h"
#include "Base/Text.h"
#include "PaintSettings/PaintSettings.h"

#include <algorithm>
#include <cmath>
#include <expected>
#include <format>

namespace simple_paint_demo::editor {

using json::Value;
using simple_paint::GpuMaterial;
using simple_paint::Material;
using simple_paint::Parameters;
using user_interface::Key;
using user_interface::Modifiers;

namespace {

// The constants of the material that the parameters make, or a panic with
// the message if SimplePaint rejects them.
GpuMaterial compile(const Parameters& parameters, std::string_view message) {
    std::expected<Material, simple_paint::Error> material = Material::compile(parameters);
    if (!material) {
        base::panic(std::format("{}: Error({})", message, base::debug(material.error().message)));
    }
    return material->constants();
}

} // namespace

double Setting::value(const Parameters& parameters) const {
    return get(parameters);
}

double Setting::nearest(double value) const {
    double stepped = std::round(value * steps) / steps;
    return std::clamp(stepped, minimum, maximum);
}

double Setting::nudge(double value, double count) const {
    double next = nearest(value + count / steps);
    return (next - value) * count > 0.0 ? next : value;
}

std::string Setting::format(double value) const {
    return std::format("{:.{}f}{}", value, decimals, unit);
}

std::string Setting::range() const {
    return std::format("{:.{}f} to {}", minimum, decimals, format(maximum));
}

std::string describe_value(const Command& command) {
    return base::match(
        command, [](const Command::Select& select) { return std::format("Select({})", select.offset); },
        [](const Command::Nudge& nudge) { return std::format("Nudge({})", base::debug(nudge.steps)); }, [](const Command::Save&) { return std::string("Save"); });
}

std::optional<Command> command(Key key, bool repeat, Modifiers modifiers) {
    if (modifiers == Modifiers::Control) {
        return key == Key::S && !repeat ? std::optional<Command>(Command::Save{}) : std::nullopt;
    }
    if (modifiers.control || modifiers.alt) {
        return std::nullopt;
    }
    double steps = modifiers.shift ? 10.0 : 1.0;
    if (key == Key::Up) {
        return Command::Select{-1};
    }
    if (key == Key::Down) {
        return Command::Select{1};
    }
    if (key == Key::Left) {
        return Command::Nudge{-steps};
    }
    if (key == Key::Right) {
        return Command::Nudge{steps};
    }
    return std::nullopt;
}

Editor::Editor(const Parameters& parameters) : parameters(parameters), material(compile(parameters, "the editor starts with a valid material")) {}

void Editor::set(size_t setting, double value) {
    if (value == Settings[setting].value(parameters)) {
        return;
    }
    Settings[setting].set(parameters, value);
    // Settings only take valid values, and set leaves the others alone.
    material = compile(parameters, "settings only make valid materials");
}

void Editor::nudge(size_t setting, double steps) {
    double current = Settings[setting].value(parameters);
    set(setting, Settings[setting].nudge(current, steps));
}

void Editor::apply(Command command, const std::filesystem::path& paint_path) {
    base::match(
        command,
        [&](const Command::Select& select) {
            selected = static_cast<size_t>(std::clamp(static_cast<std::int32_t>(selected) + select.offset, 0, static_cast<std::int32_t>(Settings.size()) - 1));
        },
        [&](const Command::Nudge& moved) { nudge(selected, moved.steps); }, [&](const Command::Save&) { save(paint_path); });
}

void Editor::reset() {
    Editor fresh{Parameters{}};
    fresh.selected = selected;
    *this = std::move(fresh);
}

Value Editor::state() const {
    const GpuMaterial& m = material;
    return json::object({
        {"Parameters", paint_settings::to_value(parameters)},
        {"Selected", Settings[selected].name},
        {"Dragging", dragging.transform([](size_t index) { return Settings[index].name; })},
        {"Saved", status.transform([](const std::pair<std::string, bool>& saved) { return !saved.second; })},
        {"Constants", json::object({
                          {"Warp", m.warp},
                          {"NumeratorDark", m.numerator_dark},
                          {"NumeratorLight", m.numerator_light},
                          {"ComplementDark", m.complement_dark},
                          {"ComplementLight", m.complement_light},
                      })},
    });
}

void Editor::save(const std::filesystem::path& path) {
    std::string text = paint_settings::format(parameters, 0) + "\n";
    std::expected<void, std::string> written = base::write(path, std::string_view(text));
    if (written) {
        status = std::pair(std::format("Saved to {}", base::display(path)), false);
    } else {
        status = std::pair(std::format("Couldn't save to {}: {}", base::display(path), written.error()), true);
    }
}

Parameters load(const std::filesystem::path& path, std::vector<std::string>& notices) {
    Parameters defaults;
    std::expected<std::string, std::string> read = base::read_to_string(path);
    if (!read) {
        if (base::is_not_found(read.error())) {
            return defaults;
        }
        notices.push_back(std::format("Couldn't read {}, so the default material is used: {}", base::display(path), read.error()));
        return defaults;
    }
    std::string_view text = *read;
    // A byte order mark, U+FEFF, before the text.
    if (text.starts_with("\uFEFF")) {
        text.remove_prefix(3);
    }
    std::expected<Parameters, std::string> result = [&]() -> std::expected<Parameters, std::string> {
        std::expected<Value, json::Error> value = json::parse(text);
        if (!value) {
            const json::Error& error = value.error();
            return std::unexpected(std::format("line {}, column {}: {}", error.line, error.column, error.message));
        }
        return paint_settings::parse(*value, defaults);
    }();
    if (!result) {
        notices.push_back(std::format("{}: {}. The default material is used.", base::display(path), result.error()));
        return defaults;
    }
    return *result;
}

} // namespace simple_paint_demo::editor
