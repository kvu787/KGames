// What a GUI program is to the library: a Program with a message type, which
// says what the input on its widgets means; `view`, which describes a
// frame's interface from the program's state and changes nothing; `update`,
// which applies a message; and `input`, which turns the frame's keys, time
// and devices into messages. It may draw 3D under the interface, and tells
// scripts its state. `run` runs one with the Options that say how.
//
// A frame: the interface finds what the input did to the widgets the user
// saw, as messages; the program's `input` adds its own; `update` applies
// them all, and any they send in turn, until none are left; then `view`
// describes the frame, which is laid out and drawn. So every widget shows
// what every message did, on the frame of the input.
//
// This includes neither Core/Tree.h nor Core/Interface.h, which includes
// this: a program's own code includes Core/Interface.h, for the Ui, the
// Context and the Tree that its functions take and return.

#pragma once

#include "UserInterface/Application/Settings.h"
#include "UserInterface/Widgets/InfoBar.h"
#include "Json/Json.h"

#include <concepts>
#include <expected>
#include <filesystem>
#include <format>
#include <memory>
#include <optional>
#include <span>
#include <string>
#include <string_view>
#include <vector>

namespace graphics {

class Gpu;
struct Frame;

} // namespace graphics

namespace user_interface {

class Devices;
class Ui;
template <class M> class Context;
template <class M> class Tree;

// A program derives from Program<M>, where M is its message type: what the
// input on its widgets means, and what else it does. M is copied freely.
// What's here is what a program may leave out. A program defines the rest
// itself:
//
//     // The window's title.
//     std::string title() const;
//
//     // Applies a message: changes the program's state, and may send more
//     // messages, or ask the window to close, or switch the theme.
//     void update(M message, Context<M>& ui);
//
//     // Describes the frame's interface from the program's state, which it
//     // doesn't change. Once a frame, after the messages are applied.
//     Tree<M> view(Ui& ui) const;
//
// Nothing here is virtual: what runs a program, like `run` and Headless, is
// a template of the program's own class, and calls what it defines, which
// hides what's here.
template <class M> class Program {
public:
    using Message = M;

    // The frame's messages from what isn't a widget: the keys that no widget
    // took, the time since the last frame, the gamepad, and the devices.
    // Once a frame, before the messages are applied.
    std::vector<M> input(const Ui&) const {
        return {};
    }

    // Creates what the program draws with on the GPU, once, before the first
    // frame. A failure throws a base::WindowsError.
    void prepare(graphics::Gpu&) {}

    // Draws the program's 3D into the frame, after it's cleared to the
    // theme's background and before the interface draws over it. A failure
    // throws a base::WindowsError.
    void render(graphics::Gpu&, const graphics::Frame&) {}

    // Whether the window may close now that it's asked to, like by its close
    // button. A program with unsaved work can say no, ask with a dialog, and
    // close with Context::close once the user says so.
    bool may_close() {
        return true;
    }

    // What scripts save and check, besides the interface's, as a JSON object.
    json::Value state() const {
        return json::Value::Object{};
    }

    // Does a script's Program step, like connecting a made-up device.
    std::expected<void, std::string> script(const json::Value& step) {
        return std::unexpected(std::format("This program has no Program steps, but got {}.", step.to_json()));
    }
};

// A program: a class that derives from Program of its message type.
template <class P>
concept IsProgram = requires { typename P::Message; } && std::derived_from<P, Program<typename P::Message>>;

// How the library runs a program. Every program starts in the light theme,
// maximized unless Settings.json says otherwise, and with vertical sync on
// unless Settings.json or the command line says otherwise, which F9, F11 and
// F10 switch.
struct Options {
    // The program's name, like "SimplePaintDemo".
    std::string_view name;
    // The program's folder, which has its Settings.json: folders::Project.
    std::string_view folder;
    // Whether the program reads the gamepad: its newest reading goes to
    // Ui::gamepad, and its readings count as input for input latency.
    bool gamepad_ = false;
    // Arguments of the program's own, which it reads from Startup.
    std::span<const std::string_view> arguments_;

    // The name and the folder last as long as the program, like literals.
    Options(std::string_view name, std::string_view folder);

    Options&& gamepad() &&;

    // The arguments last as long as the program, like a constant array's.
    Options&& arguments(std::span<const std::string_view> arguments) &&;
};

// Something that a notice under the app bar tells of, like a file that's
// wrong.
struct Notice {
    Severity severity;
    std::string title;
    std::string message;

    bool operator==(const Notice&) const = default;
};

// How a failed check in a test shows a notice.
std::string describe_value(const Notice& notice);

// What a program starts with.
struct Startup {
    // Settings.json, which the program reads its own settings from.
    Settings settings;
    // Where the program reads and writes its files: its folder, or a
    // script's output folder, so that scripts never change its files.
    std::filesystem::path data_folder;
    // Whether input comes from a script.
    bool scripted;
    // The program's own arguments, of those that Options allows.
    std::vector<std::string> arguments;
    // The input devices that the interface reads, for programs that show
    // them or run their motors, like GameInputDemo. Each frame's readings are
    // there when the program's messages are applied. Null in a scripted run,
    // and with --no-input.
    std::shared_ptr<Devices> devices;
    // The library's own.
    std::vector<Notice> notices;

    Startup(Settings settings, const std::filesystem::path& data_folder, bool scripted);

    // A start for tests: settings from the text, and files in the folder.
    static Startup for_tests(std::string_view settings_text, const std::filesystem::path& folder);

    // Tells of something under the app bar, like a file that's wrong.
    void notify(Severity severity, std::string_view title, std::string_view message);

    // The library's own: the notices, with one for each problem with the
    // settings first.
    std::vector<Notice> take_notices();
};

// The command line that every program takes.
struct Arguments {
    // The display to open on, by its identifier, or "Primary".
    std::optional<std::string> display;
    // Opens without taking focus.
    bool no_activate = false;
    // Whether frames wait for vertical sync, with --vsync or --no-vsync,
    // whatever the VerticalSync setting says, like for test runs.
    std::optional<bool> vsync;
    // Doesn't start GameInput, for test runs, which never have focus, so
    // that nothing of theirs reads the devices of someone using the computer.
    bool no_input = false;
    // A script to run instead of the user's input, and the folder for what
    // it saves.
    std::optional<std::filesystem::path> script;
    std::optional<std::filesystem::path> output;
    // The program's own, of those it allows.
    std::vector<std::string> own;

    // The arguments, after the program's own name, as base::arguments gives
    // them, of which `allowed` are the program's own. An argument that's
    // wrong is an error that says what every program takes.
    static std::expected<Arguments, std::string> parse(std::vector<std::string> arguments, std::span<const std::string_view> allowed);
};

} // namespace user_interface
