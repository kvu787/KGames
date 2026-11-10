#include "SimplePaintDemo/Demo.h"

#include "SimplePaintDemo/Cards.h"
#include "UserInterface/Input/Devices.h"
#include "UserInterface/Input/Events.h"
#include "UserInterface/Widgets/InfoBar.h"

namespace simple_paint_demo::demo {

using editor::Settings;
using simple_paint::Parameters;
using user_interface::Context;
using user_interface::Key;
using user_interface::Modifiers;
using user_interface::Severity;
using user_interface::Startup;
using user_interface::Tree;
using user_interface::Ui;

Demo::Demo(const Parameters& parameters, std::filesystem::path paint_path) : editor(parameters), paint_path(std::move(paint_path)) {}

Demo Demo::start(Startup& startup) {
    std::filesystem::path path = startup.data_folder / "Paint.json";
    Parameters parameters;
    if (!startup.scripted) {
        std::vector<std::string> notices;
        parameters = editor::load(path, notices);
        for (const std::string& notice : notices) {
            startup.notify(Severity::Warning, "Paint.json", notice);
        }
    }
    return Demo(parameters, std::move(path));
}

std::string Demo::title() const {
    return std::string(Title);
}

std::vector<Message> Demo::input(const Ui& ui) const {
    Modifiers modifiers = ui.input().modifiers();
    std::vector<Message> messages;
    for (auto [key, repeat] : ui.input().keys_pressed()) {
        if (ui.key_pressed(key)) {
            if (std::optional<editor::Command> command = editor::command(key, repeat, modifiers)) {
                messages.push_back(Message::Command{*command});
            }
        }
    }
    return messages;
}

void Demo::update(Message message, Context<Message>&) {
    base::match(
        message, [&](const Message::Command& key) { editor.apply(key.command, paint_path); }, [&](const Message::Reset&) { editor.reset(); },
        [&](const Message::Save&) { editor.save(paint_path); }, [&](const Message::Select& select) { editor.selected = select.index; },
        [&](const Message::Set& set) {
            editor.selected = set.index;
            editor.set(set.index, Settings[set.index].nearest(set.value));
        },
        [&](const Message::Drag& drag) {
            if (drag.held) {
                editor.selected = drag.index;
                editor.dragging = drag.index;
            } else {
                editor.dragging = std::nullopt;
            }
        });
}

Tree<Message> Demo::view(Ui& ui) const {
    return cards::view(*this, ui);
}

void Demo::prepare(graphics::Gpu& gpu) {
    preview.emplace(gpu);
}

void Demo::render(graphics::Gpu& gpu, const graphics::Frame& frame) {
    if (preview && view_) {
        preview->render(gpu, frame, editor.material, background, *view_);
    }
}

json::Value Demo::state() const {
    return editor.state();
}

} // namespace simple_paint_demo::demo
