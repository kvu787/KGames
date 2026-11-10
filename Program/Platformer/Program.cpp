#include "Platformer/Program.h"

#include "Base/Files.h"
#include "Base/Vec.h"
#include "Platformer/Cards.h"
#include "Platformer/Level.h"
#include "Platformer/Platformer.h"
#include "Platformer/Progress.h"
#include "UserInterface/Widgets/InfoBar.h"

#include <expected>
#include <filesystem>

namespace platformer::program {

using controls::Controls;
using game::Game;
using level::Level;
using progress::Progress;
using user_interface::Context;
using user_interface::Severity;
using user_interface::Startup;
using user_interface::Tree;
using user_interface::Ui;

Platformer::Platformer(Game game) : game(std::move(game)) {}

Platformer Platformer::start(Startup& startup) {
    std::vector<Level> levels = level::levels();
    std::filesystem::path save_path = startup.data_folder / base::path(progress::File);
    std::expected<Progress, std::string> progress = Progress::load(save_path, levels.size());
    if (!progress) {
        startup.notify(Severity::Warning, progress::File, progress.error());
        progress = Progress(levels.size());
    }
    return Platformer(Game(std::move(levels), std::move(*progress), std::move(save_path)));
}

std::string Platformer::title() const {
    return std::string(Name);
}

std::vector<Message> Platformer::input(const Ui& ui) const {
    return base::vec(Message(Message::Tick{Controls::read(ui)}));
}

void Platformer::update(Message message, Context<Message>& ui) {
    base::match(
        message, [&](const Message::Tick& tick) { game.update(tick.controls, ui.time()); }, [&](const Message::GoLevel& go_level) { game.go_to_level(go_level.level); },
        [&](const Message::Restart&) { game.restart(); });
}

Tree<Message> Platformer::view(Ui& ui) const {
    return cards::view(game, ui);
}

json::Value Platformer::state() const {
    return game.state();
}

} // namespace platformer::program
