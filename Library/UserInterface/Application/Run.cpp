#include "UserInterface/Application/Run.h"

#include "Base/Panic.h"
#include "Base/Sum.h"

#include <fcntl.h>
#include <io.h>

#include <cstdio>
#include <print>

namespace user_interface::run_detail {

void begin() {
    base::fail_quietly();
    // Text goes out as it's written, as Rust's does: without this, a line
    // that's written to a file or a pipe ends with a carriage return and a
    // line feed, not a line feed alone. A program that a double click
    // started has neither a console nor a file to write to, and no stream
    // to set: what it writes goes nowhere, as in Rust.
    for (FILE* stream : {stdout, stderr}) {
        int descriptor = _fileno(stream);
        if (descriptor >= 0) {
            _setmode(descriptor, _O_BINARY);
        }
    }
}

void report(std::string_view name, std::string_view message) {
    std::println(stderr, "{}: {}", name, message);
}

void report_passed(std::uint64_t frames) {
    std::println("The script passed, in {} frames.", frames);
}

json::Value state_with(json::Value program, const Interface& ui) {
    std::vector<std::pair<std::string, json::Value>> members;
    if (json::Value::Object* object = base::as<json::Value::Object>(program)) {
        members = std::move(object->members);
    } else if (!base::is<json::Value::Null>(program)) {
        members.emplace_back("Program", std::move(program));
    }
    members.emplace_back("Interface", ui.state());
    return json::Value::Object{std::move(members)};
}

double seconds_since(std::chrono::steady_clock::time_point start) {
    return std::chrono::duration<double>(std::chrono::steady_clock::now() - start).count();
}

std::expected<void, std::string> save_capture(const std::filesystem::path& folder, std::string_view name, const graphics::Image& image, const json::Value& state) {
    std::filesystem::path png = folder / base::path(std::format("{}.png", name));
    if (std::expected<void, std::string> written = attempt(std::format("Couldn't write {}", base::display(png)), [&] { graphics::write_png(png, image); }); !written) {
        return written;
    }
    std::filesystem::path path = folder / base::path(std::format("{}.json", name));
    if (std::expected<void, std::string> written = base::write(path, state.to_json() + "\n"); !written) {
        return std::unexpected(std::format("Couldn't write {}: {}", base::display(path), written.error()));
    }
    return {};
}

} // namespace user_interface::run_detail
