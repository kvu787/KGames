#include "UserInterface/Application/Program.h"

#include "Base/Files.h"
#include "Base/Text.h"

#include <algorithm>
#include <iterator>
#include <utility>

namespace user_interface {

Options::Options(std::string_view name, std::string_view folder) : name(name), folder(folder) {}

Options&& Options::gamepad() && {
    gamepad_ = true;
    return std::move(*this);
}

Options&& Options::arguments(std::span<const std::string_view> arguments) && {
    arguments_ = arguments;
    return std::move(*this);
}

std::string describe_value(const Notice& notice) {
    return std::format("Notice {{ severity: {}, title: {}, message: {} }}", name(notice.severity), base::debug(notice.title), base::debug(notice.message));
}

Startup::Startup(Settings settings, const std::filesystem::path& data_folder, bool scripted) : settings(std::move(settings)), data_folder(data_folder), scripted(scripted) {}

Startup Startup::for_tests(std::string_view settings_text, const std::filesystem::path& folder) {
    return Startup(Settings::from_text(settings_text), folder, true);
}

void Startup::notify(Severity severity, std::string_view title, std::string_view message) {
    notices.push_back(Notice{
        .severity = severity,
        .title = std::string(title),
        .message = std::string(message),
    });
}

std::vector<Notice> Startup::take_notices() {
    std::span<const std::string> problems = settings.problems();
    std::vector<Notice> taken;
    taken.reserve(problems.size());
    for (const std::string& problem : problems) {
        taken.push_back(Notice{
            .severity = Severity::Warning,
            .title = "Settings.json",
            .message = problem,
        });
    }
    taken.insert(taken.end(), std::make_move_iterator(notices.begin()), std::make_move_iterator(notices.end()));
    notices.clear();
    return taken;
}

std::expected<Arguments, std::string> Arguments::parse(std::vector<std::string> arguments, std::span<const std::string_view> allowed) {
    Arguments parsed;
    size_t next = 0;
    // The argument after this one, taken as its value.
    auto value = [&](std::string_view name) -> std::expected<std::string, std::string> {
        if (next == arguments.size()) {
            return std::unexpected(std::format("{} needs a value after it.", name));
        }
        std::string taken = std::move(arguments[next]);
        next += 1;
        return taken;
    };
    while (next < arguments.size()) {
        std::string argument = std::move(arguments[next]);
        next += 1;
        if (argument == "--display") {
            std::expected<std::string, std::string> given = value("--display");
            if (!given) {
                return std::unexpected(std::move(given.error()));
            }
            parsed.display = std::move(*given);
        } else if (argument == "--no-activate") {
            parsed.no_activate = true;
        } else if (argument == "--vsync") {
            parsed.vsync = true;
        } else if (argument == "--no-vsync") {
            parsed.vsync = false;
        } else if (argument == "--no-input") {
            parsed.no_input = true;
        } else if (argument == "--script") {
            std::expected<std::string, std::string> given = value("--script");
            if (!given) {
                return std::unexpected(std::move(given.error()));
            }
            parsed.script = base::path(*given);
        } else if (argument == "--output") {
            std::expected<std::string, std::string> given = value("--output");
            if (!given) {
                return std::unexpected(std::move(given.error()));
            }
            parsed.output = base::path(*given);
        } else if (std::ranges::find(allowed, std::string_view(argument)) != allowed.end()) {
            parsed.own.push_back(std::move(argument));
        } else {
            return std::unexpected(std::format(
                "Unknown argument {}. Every program takes --display <identifier>, --no-activate, --no-input, --vsync or --no-vsync, and --script <path> with --output <folder>.",
                argument));
        }
    }
    if (parsed.script.has_value() != parsed.output.has_value()) {
        return std::unexpected(std::string("--script and --output go together."));
    }
    return parsed;
}

} // namespace user_interface
