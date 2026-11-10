// The main loop of every GUI program, built for the lowest input latency:
//
// 1. Handle the window's messages, without waiting.
// 2. Go on handling them until the presenter is ready: the GPU has finished
//    the last frame, and DXGI's queue has room.
// 3. Only then read the input, from GameInput, build the frame's interface
//    with the newest of it, draw the frame, and present it.
//
// So the input that a frame shows is as fresh as it can be when the frame
// reaches the screen. While the window is minimized, the loop waits for
// messages instead, which stops it drawing.
//
// Programs start with vertical sync on: frames present at the screen's
// refreshes, and DXGI's queue has room again only once the last frame has
// gone to the screen, so the loop waits for that, and for messages, instead
// of spinning: a frame couldn't begin any sooner, and the input is still read
// as soon as one can. F10 switches it on and off. Without it, frames present
// as soon as they're drawn, and the loop spins, for the lowest latency.
//
// A program's Main.cpp is little more than its call of `run`:
//
//     int main() {
//         return user_interface::run(user_interface::Options("Platformer", folders::Project).gamepad(),
//                                    [](user_interface::Startup& startup) { return platformer::Platformer::start(startup); });
//     }

#pragma once

#include "Base/Files.h"
#include "Base/Text.h"
#include "Base/Windows.h"
#include "Displays/Displays.h"
#include "GameInput/Installation.h"
#include "Graphics/Graphics.h"
#include "UserInterface/Application/Attempt.h"
#include "UserInterface/Application/Performance.h"
#include "UserInterface/Application/Program.h"
#include "UserInterface/Application/Settings.h"
#include "UserInterface/Core/Interface.h"
#include "UserInterface/Core/Tree.h"
#include "UserInterface/Design/Theme.h"
#include "UserInterface/Drawing/Geometry.h"
#include "UserInterface/Drawing/Renderer.h"
#include "UserInterface/Drawing/Text.h"
#include "UserInterface/Input/Events.h"
#include "UserInterface/Input/Reader.h"
#include "UserInterface/Platform/Clipboard.h"
#include "UserInterface/Platform/Clock.h"
#include "UserInterface/Platform/InputDevices.h"
#include "UserInterface/Platform/Window.h"
#include "UserInterface/Testing/Script.h"
#include "UserInterface/Widgets/InfoBar.h"
#include "UserInterface/Widgets/Shell.h"
#include "Json/Json.h"

#include <array>
#include <chrono>
#include <cstddef>
#include <cstdint>
#include <expected>
#include <filesystem>
#include <format>
#include <memory>
#include <optional>
#include <string>
#include <string_view>
#include <type_traits>
#include <utility>
#include <vector>

namespace user_interface {

// What `run` and `state` are made of, here and in Run.cpp.
namespace run_detail {

// What a program's start gives: the program, or a std::expected of it, with
// why it couldn't start in place of it.
template <class Created> struct Start {
    using Program = Created;
};

template <class P> struct Start<std::expected<P, std::string>> {
    using Program = P;
};

// The program that `create` makes, given a Startup&.
template <class Create> using ProgramOf = typename Start<std::remove_cvref_t<std::invoke_result_t<Create, Startup&>>>::Program;

// Why a start gave no program, or null if it gave one.
template <class P> const std::string* failure(const P&) {
    return nullptr;
}

template <class P> const std::string* failure(const std::expected<P, std::string>& created) {
    return created ? nullptr : &created.error();
}

// The program that a start gave.
template <class P> P& program_of(P& created) {
    return created;
}

template <class P> P& program_of(std::expected<P, std::string>& created) {
    return *created;
}

// What every program does before anything else: it stops a failure from
// opening a dialog, with base::fail_quietly, and has what it writes to
// standard output and standard error go out as it's written, with each line
// ending in a line feed alone, as the Rust's does.
void begin();

// Says why the program ended, on standard error, after the program's name.
void report(std::string_view name, std::string_view message);

// Says that the script passed, on standard output.
void report_passed(std::uint64_t frames);

// A program's state with the interface's.
json::Value state_with(json::Value program, const Interface& ui);

// Seconds since the start.
double seconds_since(std::chrono::steady_clock::time_point start);

std::expected<void, std::string> save_capture(const std::filesystem::path& folder, std::string_view name, const graphics::Image& image, const json::Value& state);

} // namespace run_detail

// The program's state with the interface's, as scripts save and check it.
template <IsProgram P> json::Value state(const P& program, const Interface& ui) {
    return run_detail::state_with(program.state(), ui);
}

namespace run_detail {

template <class Create> std::expected<void, std::string> run_program(const Options& options, Create&& create) {
    using P = ProgramOf<Create>;
    std::expected<Arguments, std::string> parsed = Arguments::parse(base::arguments(), options.arguments_);
    if (!parsed) {
        return std::unexpected(std::move(parsed.error()));
    }
    const Arguments& arguments = *parsed;
    std::filesystem::path folder = base::path(options.folder);
    bool scripted = arguments.script.has_value();
    std::optional<Script> script;
    if (arguments.script) {
        std::expected<Script, std::string> loaded = Script::load(*arguments.script);
        if (!loaded) {
            return std::unexpected(std::move(loaded.error()));
        }
        script = std::move(*loaded);
    }
    std::filesystem::path data_folder = arguments.output ? *arguments.output : folder;
    if (arguments.output) {
        if (std::expected<void, std::string> made = base::create_dir_all(*arguments.output); !made) {
            return std::unexpected(std::format("Couldn't make the folder {}: {}", base::display(*arguments.output), made.error()));
        }
    }

    // The settings that every program has, then the program's own.
    Settings settings = Settings::load(folder / "Settings.json");
    std::string display_setting = settings.string("Display", "Primary");
    std::array<std::string_view, 2> modes{name(WindowMode::Maximized), name(WindowMode::BorderlessFullscreen)};
    std::string mode_name = settings.choice("WindowMode", name(WindowMode::Maximized), modes);
    WindowMode mode = mode_name == name(WindowMode::Maximized) ? WindowMode::Maximized : WindowMode::BorderlessFullscreen;
    bool allow_tearing = settings.boolean("AllowTearing", true);
    bool vertical_sync_setting = settings.boolean("VerticalSync", true);
    bool vertical_sync = arguments.vsync.value_or(vertical_sync_setting);
    Startup startup(std::move(settings), data_folder, scripted);
    startup.arguments = arguments.own;

    // All input comes from GameInput, except in a scripted run, which the
    // user's input never reaches, and with --no-input. Every program needs
    // GameInput 4.0.306 installed, though, and doesn't start without it, even
    // when it doesn't read it (see Library/GameInput).
    std::shared_ptr<Devices> devices;
    if (scripted || arguments.no_input) {
        if (std::expected<std::filesystem::path, std::string> runtime = game_input::installation::installed_runtime(); !runtime) {
            return std::unexpected(std::move(runtime.error()));
        }
    } else {
        std::expected<Devices, std::string> started = Devices::create();
        if (!started) {
            return std::unexpected(std::move(started.error()));
        }
        devices = std::make_shared<Devices>(std::move(*started));
    }
    startup.devices = devices;

    // The display: --display's, which must be connected, or else the
    // setting's, or else the primary one.
    std::expected<std::vector<displays::Display>, std::string> connected = attempt("Couldn't find the displays", [] { return displays::enumerate(); });
    if (!connected) {
        return std::unexpected(std::move(connected.error()));
    }
    const displays::Display* primary = displays::primary(*connected);
    if (primary == nullptr) {
        return std::unexpected("There's no primary display.");
    }
    const displays::Display* display = primary;
    if (arguments.display) {
        if (!base::eq_ignore_ascii_case(*arguments.display, "Primary")) {
            display = displays::find(*connected, *arguments.display);
            if (display == nullptr) {
                std::string identifiers;
                for (size_t index = 0; index < connected->size(); index++) {
                    identifiers += index == 0 ? "" : ", ";
                    identifiers += (*connected)[index].identifier;
                }
                return std::unexpected(std::format("The display {} isn't connected. These are: {}.", *arguments.display, identifiers));
            }
        }
    } else if (!base::eq_ignore_ascii_case(display_setting, "Primary")) {
        display = displays::find(*connected, display_setting);
        if (display == nullptr) {
            startup.notify(Severity::Warning, "Settings.json", std::format("The display {} isn't connected, so the window is on the primary display.", display_setting));
            display = primary;
        }
    }

    std::expected<Fonts, std::string> fonts = attempt("Couldn't start DirectWrite", [] { return Fonts(); });
    if (!fonts) {
        return std::unexpected(std::move(fonts.error()));
    }
    // Windows' clipboard, once there's a window to own what it writes, or,
    // in a scripted run, the interface's own.
    Interface ui(std::move(*fonts), ThemeKind::Light, Clipboard::Own{});
    // The tree the last frame showed, which the next frame's input is on.
    std::optional<Tree<typename P::Message>> shown;
    // The PerfBar along the top of the window, with the input
    // latency to the screen, which a scripted run has no input for, nor a
    // run without input, which then also doesn't start the ETW session that
    // every process's presents would go through.
    if (scripted) {
        ui.performance = performance::Performance::without_latency("none in a scripted run");
    } else if (arguments.no_input) {
        ui.performance = performance::Performance::without_latency("none without input");
    } else {
        ui.performance = performance::Performance::measure(options.name);
    }
    // The program, or a std::expected of it, where it's made, so that it's
    // never moved.
    auto created = std::forward<Create>(create)(startup);
    if (const std::string* error = run_detail::failure(created)) {
        return std::unexpected(*error);
    }
    P& program = run_detail::program_of(created);
    ui.notices_ = startup.take_notices();
    if (!scripted) {
        if (std::expected<void, std::string> written = startup.settings.write_if_missing(); !written) {
            ui.notices_.push_back(Notice{
                .severity = Severity::Warning,
                .title = "Settings.json",
                .message = std::format("Couldn't write it: {}.", written.error()),
            });
        }
    }

    std::expected<Window, std::string> opened = attempt("Couldn't open the window", [&] { return Window::create(program.title(), display->monitor, mode); });
    if (!opened) {
        return std::unexpected(std::move(opened.error()));
    }
    Window& window = *opened;
    if (!scripted) {
        ui.clipboard = Clipboard::Windows{window.hwnd()};
    }
    window.set_dark_title_bar(ui.theme().kind == ThemeKind::Dark);
    std::expected<graphics::Gpu, std::string> gpu_started =
        attempt("Couldn't start Direct3D 12", [&] { return graphics::Gpu(graphics::AdapterChoice::Monitor{window.monitor()}); });
    if (!gpu_started) {
        return std::unexpected(std::move(gpu_started.error()));
    }
    graphics::Gpu& gpu = *gpu_started;
    std::pair<std::uint32_t, std::uint32_t> client = window.client_size();
    // The renderer before the presenter: what's made last goes first, and
    // the presenter waits for the GPU to finish the last frame as it goes,
    // before the renderer's buffers and the program's resources go.
    std::expected<Renderer, std::string> renderer_made = attempt("Couldn't make the renderer", [&] { return Renderer(gpu); });
    if (!renderer_made) {
        return std::unexpected(std::move(renderer_made.error()));
    }
    Renderer& renderer = *renderer_made;
    std::expected<graphics::Presenter, std::string> presenter_made =
        attempt("Couldn't make the swap chain", [&] { return graphics::Presenter(gpu, window.hwnd(), client.first, client.second, allow_tearing); });
    if (!presenter_made) {
        return std::unexpected(std::move(presenter_made.error()));
    }
    graphics::Presenter& presenter = *presenter_made;
    presenter.set_vsync(vertical_sync);
    if (std::expected<void, std::string> prepared = attempt("Couldn't prepare to draw", [&] { program.prepare(gpu); }); !prepared) {
        return prepared;
    }
    ui.environment_ = Environment{
        .gpu = gpu.name,
        .display = display->identifier,
        .display_name = display->name,
        .tearing = presenter.tearing(),
        .vertical_sync = vertical_sync,
        .scripted = scripted,
    };
    std::string title = program.title();
    window.show(!arguments.no_activate);

    InputReader reader(options.gamepad_);
    std::optional<ScriptRunner> runner;
    if (script) {
        runner.emplace(std::move(*script));
    }
    std::chrono::steady_clock::time_point start = std::chrono::steady_clock::now();
    std::uint64_t frame = 0;
    std::vector<std::string> problems;
    while (true) {
        if (!window.pump()) {
            // The window was asked to close. A program with unsaved work may
            // say not yet, and close itself later with Context::close.
            if (program.may_close()) {
                break;
            }
            window.cancel_close();
        }
        if (window.take_setup_changed()) {
            problems = window.setup_problems();
        }
        const std::pair<std::uint32_t, std::uint32_t> window_size = window.client_size();
        const std::uint32_t width = window_size.first;
        const std::uint32_t height = window_size.second;
        if ((window.minimized() || width == 0 || height == 0) && !runner) {
            // A minimized window doesn't draw, but the program still hears
            // what happened, like the window losing focus as it minimized,
            // so that it can stop what it's doing, like a gamepad's motors.
            std::vector<FocusChange> focus_changes = window.take_focus_changes();
            if (devices) {
                devices->read();
            }
            FrameInput input = reader.frame(devices.get(), window, focus_changes);
            if (!focus_changes.empty() && problems.empty() && !runner) {
                std::pair<std::uint32_t, std::uint32_t> last_size = presenter.size();
                bool fullscreen = window.mode() == WindowMode::BorderlessFullscreen;
                ui.frame(program, shown, input.events, {}, seconds_since(start), Size(static_cast<float>(last_size.first), static_cast<float>(last_size.second)), fullscreen);
                bool closing = false;
                for (Request request : std::exchange(ui.requests, {})) {
                    if (request == Request::Close) {
                        closing = true;
                        break;
                    }
                }
                if (closing) {
                    break;
                }
            }
            window.wait();
            continue;
        }
        if (window_size != presenter.size()) {
            if (std::expected<void, std::string> resized = attempt("Couldn't resize the swap chain", [&] { presenter.resize(gpu, width, height); }); !resized) {
                return resized;
            }
        }
        if (!presenter.poll_ready(gpu)) {
            // With vertical sync, the loop waits for the GPU and DXGI instead
            // of spinning, which ends as soon as a frame can begin, or a
            // message comes. Without it, the loop spins, so that the frame
            // begins the moment that it can, without a thread's wake-up.
            if (presenter.vsync()) {
                if (std::expected<void, std::string> waited = attempt("Couldn't wait for the GPU", [&] { presenter.wait_ready(gpu, 100); }); !waited) {
                    return waited;
                }
            }
            continue;
        }

        // The input, read as late as it can be. A script's frames count from
        // 0, and the user's input never reaches a scripted run.
        std::uint64_t script_number = frame;
        frame += 1;
        std::vector<FocusChange> focus_changes = window.take_focus_changes();
        std::int64_t read = clock::now();
        std::optional<std::int64_t> input_time;
        std::vector<Event> events;
        std::optional<ScriptFrame> script_frame;
        double time = 0.0;
        if (runner) {
            std::expected<ScriptFrame, std::string> scripted_frame = runner->frame(script_number, ui);
            if (!scripted_frame) {
                return std::unexpected(std::move(scripted_frame.error()));
            }
            events = scripted_frame->events;
            for (const json::Value& step : scripted_frame->program_steps) {
                if (std::expected<void, std::string> done = program.script(step); !done) {
                    return std::unexpected(std::format("Frame {}: Program step: {}", script_number, done.error()));
                }
            }
            script_frame = std::move(*scripted_frame);
            time = static_cast<double>(script_number) * runner->frame_time();
        } else {
            if (devices) {
                devices->read();
            }
            FrameInput input = reader.frame(devices.get(), window, focus_changes);
            events = std::move(input.events);
            input_time = input.input_time;
            time = seconds_since(start);
        }
        bool fullscreen = window.mode() == WindowMode::BorderlessFullscreen;
        Size size(static_cast<float>(width), static_cast<float>(height));
        if (problems.empty()) {
            ui.frame(program, shown, events, {}, time, size, fullscreen);
        } else {
            // The error instead of the program, until the setup is right.
            shown = std::nullopt;
            ui.frame_of(unsupported_setup<typename P::Message>(problems), events, time, size, fullscreen);
        }
        bool closing = false;
        for (Request request : std::exchange(ui.requests, {})) {
            switch (request) {
            case Request::ToggleFullscreen:
                window.set_mode(other(window.mode()));
                break;
            case Request::ToggleVerticalSync:
                presenter.set_vsync(!presenter.vsync());
                ui.environment_.vertical_sync = presenter.vsync();
                ui.environment_.tearing = presenter.tearing();
                break;
            case Request::Close:
                closing = true;
                break;
            }
            if (closing) {
                break;
            }
        }
        if (closing) {
            break;
        }
        window.set_cursor(ui.cursor());
        window.set_dark_title_bar(ui.theme().kind == ThemeKind::Dark);
        std::string new_title = program.title();
        if (new_title != title) {
            window.set_title(new_title);
            title = std::move(new_title);
        }

        // The frame: the theme's background, the program's 3D, and the
        // interface over them.
        std::expected<graphics::Frame, std::string> begun = attempt("Couldn't begin a frame", [&] { return presenter.begin(); });
        if (!begun) {
            return std::unexpected(std::move(begun.error()));
        }
        graphics::Frame& target = *begun;
        Color background = ui.theme().background;
        target.clear({background.r, background.g, background.b, 1.0f});
        if (problems.empty()) {
            if (std::expected<void, std::string> drawn = attempt("Couldn't draw the frame", [&] { program.render(gpu, target); }); !drawn) {
                return drawn;
            }
        }
        if (std::expected<void, std::string> drawn = attempt("Couldn't draw the interface", [&] { renderer.draw(gpu, target, ui.draw, ui.fonts_); }); !drawn) {
            return drawn;
        }
        std::vector<std::string> captures = script_frame ? script_frame->captures : std::vector<std::string>();
        std::optional<graphics::Readback> readback;
        if (!captures.empty()) {
            std::expected<graphics::Readback, std::string> recorded = attempt("Couldn't capture the frame", [&] { return graphics::Readback::record(gpu.device.Get(), target); });
            if (!recorded) {
                return std::unexpected(std::move(recorded.error()));
            }
            readback = std::move(*recorded);
        }
        std::expected<std::uint64_t, std::string> fence = attempt("Couldn't present", [&] { return presenter.present(gpu, std::move(target)); });
        if (!fence) {
            return std::unexpected(std::move(fence.error()));
        }
        if (ui.performance) {
            ui.performance->presented(performance::Frame{
                .input = input_time,
                .read = read,
                .present = presenter.present_time(),
            });
        }
        gpu.report_messages();

        if (runner && script_frame) {
            json::Value state = user_interface::state(program, ui);
            if (readback) {
                std::expected<graphics::Image, std::string> image = attempt("Couldn't read the frame back", [&] { return readback->read(gpu, *fence); });
                if (!image) {
                    return std::unexpected(std::move(image.error()));
                }
                for (const std::string& capture : captures) {
                    if (std::expected<void, std::string> saved = save_capture(data_folder, capture, *image, state); !saved) {
                        return saved;
                    }
                }
            }
            for (const json::Value& expectation : script_frame->expectations) {
                runner->check(script_number, expectation, state);
            }
            if (script_number >= runner->last_frame()) {
                break;
            }
        }
    }

    // The presenter waits for the GPU to finish the last frame as it goes,
    // on every way out of here. This also says so when the wait fails.
    if (std::expected<void, std::string> flushed = attempt("Couldn't wait for the GPU", [&] { gpu.flush(); }); !flushed) {
        return flushed;
    }
    if (runner) {
        if (!runner->failures.empty()) {
            std::string failures;
            for (size_t index = 0; index < runner->failures.size(); index++) {
                failures += index == 0 ? "" : "\n";
                failures += runner->failures[index];
            }
            return std::unexpected(std::format("The script's expectations weren't met:\n{}", failures));
        }
        report_passed(frame);
    }
    return {};
}

} // namespace run_detail

// Runs the program that `create` makes until its window closes or its script
// ends. If it can't start, or a script's expectation isn't met, it says why
// on standard error and exits with a failure: this returns what `main`
// returns, 0 or 1.
//
// `create` is called once, with a Startup&, and returns the program, or a
// std::expected<P, std::string> with why it couldn't start.
template <class Create>
    requires IsProgram<run_detail::ProgramOf<Create>>
int run(Options options, Create&& create) {
    run_detail::begin();
    std::expected<void, std::string> result = run_detail::run_program(options, std::forward<Create>(create));
    if (!result) {
        run_detail::report(options.name, result.error());
        return 1;
    }
    return 0;
}

} // namespace user_interface
