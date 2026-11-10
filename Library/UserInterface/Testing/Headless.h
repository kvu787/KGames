// Runs a program's frames without a window, for tests: input that the test
// makes up, on a fixed clock, through the same interface as a real run, so
// that presses land on the cards' real layout. DirectWrite lays text out
// without a GPU. A test that wants to see a frame draws it offscreen with
// `render`.
//
// Each input method runs one frame with the input, as if the user did it at
// 60 frames a second, and returns the harness, so that a test reads like
// what it does:
//
//     Headless app(std::move(program));
//     app.click("Save").key(Key::F11);
//     CHECK(app.fullscreen());
//
// The frame shows all that the input did: a click's press opens a popup
// whose items the next input can point at, and a message's every effect is
// in the frame that applies it.

#pragma once

#include "Base/Numbers.h"
#include "Base/Panic.h"
#include "Base/Text.h"
#include "Graphics/Graphics.h"
#include "UserInterface/Application/Performance.h"
#include "UserInterface/Application/Program.h"
#include "UserInterface/Application/Run.h"
#include "UserInterface/Core/Interaction.h"
#include "UserInterface/Core/Interface.h"
#include "UserInterface/Core/Tree.h"
#include "UserInterface/Design/Theme.h"
#include "UserInterface/Design/Tokens.h"
#include "UserInterface/Drawing/Geometry.h"
#include "UserInterface/Drawing/Renderer.h"
#include "UserInterface/Drawing/Text.h"
#include "UserInterface/Input/Devices.h"
#include "UserInterface/Input/Events.h"
#include "UserInterface/Platform/Clipboard.h"
#include "UserInterface/Testing/Script.h"
#include "Json/Json.h"

#include <algorithm>
#include <array>
#include <cmath>
#include <cstddef>
#include <cstdint>
#include <format>
#include <optional>
#include <ranges>
#include <span>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

namespace user_interface {

// What Headless is made of that isn't the program's, in Headless.cpp.
namespace headless_detail {

// DirectWrite's fonts, or a panic if it couldn't start.
Fonts fonts();

// The area of the last frame's widget with the name, or a panic that says
// what the frame did show.
Rect target(const Interface& ui, std::string_view name);

} // namespace headless_detail

template <IsProgram P> class Headless {
public:
    using Message = typename P::Message;

    P program;
    Interface ui;

    // The program in a maximized window that has focus, in the light theme.
    explicit Headless(P program) : Headless(std::move(program), ThemeKind::Light) {}

    static Headless with_theme(P program, ThemeKind theme) {
        return Headless(std::move(program), theme);
    }

    // The program that `create` makes from the start, whose notices show
    // under its app bar. `create` is called with a Startup&, and returns the
    // program, or a std::expected<P, std::string> with why it couldn't
    // start, which is a panic here.
    template <class Create> static Headless start(Startup startup, Create&& create) {
        auto created = std::forward<Create>(create)(startup);
        if (const std::string* error = run_detail::failure(created)) {
            base::panic(std::format("the program couldn't start: {}", base::debug(*error)));
        }
        Headless headless(std::move(run_detail::program_of(created)));
        headless.ui.notices_ = startup.take_notices();
        headless.run_frame();
        return headless;
    }

    // A program in a window of the size, like an overlay, which has no
    // PerfBar.
    static Headless with_size(P program, ThemeKind theme, Size size) {
        Headless headless(std::move(program), theme);
        headless.custom_size = size;
        headless.ui.performance = std::nullopt;
        headless.run_frame();
        return headless;
    }

    Headless(const Headless&) = delete;
    Headless& operator=(const Headless&) = delete;
    Headless(Headless&&) = default;
    Headless& operator=(Headless&&) = default;

    // Replaces the interface's own clipboard, like with one that's
    // unavailable, as Windows' is while another program keeps it open.
    Headless& set_clipboard(Clipboard clipboard) {
        ui.clipboard = std::move(clipboard);
        return *this;
    }

    std::optional<std::string> clipboard_text() const {
        return ui.clipboard.text();
    }

    Size size() const {
        if (custom_size) {
            return *custom_size;
        }
        return Size(tokens::WindowWidth, fullscreen_ ? tokens::FullscreenHeight : tokens::MaximizedHeight);
    }

    bool fullscreen() const {
        return fullscreen_;
    }

    Headless& set_fullscreen(bool fullscreen) {
        fullscreen_ = fullscreen;
        return run_frame();
    }

    // Whether frames would wait for vertical sync, which a test's frames
    // start with, as a real run's do, and F10 switches.
    bool vertical_sync() const {
        return ui.vertical_sync();
    }

    // Whether the program closed its window with Context::close.
    bool closed() const {
        return closed_;
    }

    std::uint64_t frame_number() const {
        return frame_;
    }

    double time() const {
        return static_cast<double>(frame_) * DefaultFrameTime;
    }

    // How many messages the last frame applied, counting those that
    // messages sent in turn.
    size_t messages_applied() const {
        return ui.messages_applied;
    }

    // Runs a frame with whatever input is pending.
    Headless& frame() {
        return run_frame();
    }

    Headless& frames(size_t count) {
        for (size_t index = 0; index < count; index++) {
            run_frame();
        }
        return *this;
    }

    // Runs frames for the seconds.
    Headless& wait(double seconds) {
        return frames(base::saturating_cast<size_t>(std::ceil(seconds / DefaultFrameTime)));
    }

    // Sends a message to the program, as a widget would, and runs the frame
    // that applies it.
    Headless& send_message(Message message) {
        sent.push_back(std::move(message));
        return run_frame();
    }

    // Queues a message for the next frame without running it, so that
    // several go in one frame.
    Headless& queue_message(Message message) {
        sent.push_back(std::move(message));
        return *this;
    }

    // The area of the widget with the name, in the last frame.
    Rect target(std::string_view name) const {
        return headless_detail::target(ui, name);
    }

    bool has_target(std::string_view name) const {
        return ui.target(name) != nullptr;
    }

    std::optional<std::string> label(std::string_view name) const {
        const Target* found = ui.target(name);
        return found == nullptr ? std::nullopt : found->label;
    }

    bool is_enabled(std::string_view name) const {
        const Target* found = ui.target(name);
        return found != nullptr && found->enabled;
    }

    // What scripts save: the program's state and the interface's.
    json::Value state() const {
        return user_interface::state(program, ui);
    }

    Point pointer() const {
        return pointer_;
    }

    // Moves the pointer to the middle of the widget.
    Headless& move_to(std::string_view name) {
        Point point = target(name).center();
        return move_point(point);
    }

    Headless& move_point(Point point) {
        pointer_ = point;
        pending.push_back(Event::PointerMoved{point});
        return run_frame();
    }

    // Presses the left button on the widget, and holds it.
    Headless& press(std::string_view name) {
        Point point = target(name).center();
        return press_point(point);
    }

    Headless& press_point(Point point) {
        return press_button(point, MouseButton::Left);
    }

    Headless& press_button(Point point, MouseButton button) {
        pointer_ = point;
        pending.push_back(Event::PointerMoved{point});
        pending.push_back(Event::ButtonPressed{button});
        return run_frame();
    }

    Headless& release() {
        return release_button(MouseButton::Left);
    }

    Headless& release_button(MouseButton button) {
        pending.push_back(Event::ButtonReleased{button});
        return run_frame();
    }

    // A press on the widget, and its release the next frame.
    Headless& click(std::string_view name) {
        return press(name).release();
    }

    Headless& click_point(Point point) {
        return press_point(point).release();
    }

    // A right press and its release, which asks for a context menu.
    Headless& right_click(std::string_view name) {
        Point point = target(name).center();
        return press_button(point, MouseButton::Right).release_button(MouseButton::Right);
    }

    Headless& double_click(std::string_view name) {
        Point point = target(name).center();
        pointer_ = point;
        pending.push_back(Event::PointerMoved{point});
        pending.push_back(Event::ButtonPressed{MouseButton::Left});
        pending.push_back(Event::ButtonReleased{MouseButton::Left});
        run_frame();
        pending.push_back(Event::ButtonPressed{MouseButton::Left});
        pending.push_back(Event::ButtonReleased{MouseButton::Left});
        return run_frame();
    }

    // Presses at one point, moves to the other over `steps` frames, and lets
    // go there.
    Headless& drag(Point from, Point to, size_t steps) {
        return drag_button(from, to, steps, MouseButton::Left);
    }

    Headless& drag_button(Point from, Point to, size_t steps, MouseButton button) {
        press_button(from, button);
        size_t count = std::max<size_t>(steps, 1);
        for (size_t step = 1; step <= count; step++) {
            move_point(from.lerp(to, static_cast<float>(step) / static_cast<float>(count)));
        }
        return release_button(button);
    }

    // Turns the wheel over the widget, by notches, up for more than 0.
    Headless& scroll(std::string_view name, float notches) {
        Point point = target(name).center();
        return scroll_point(point, notches);
    }

    Headless& scroll_point(Point point, float notches) {
        pointer_ = point;
        pending.push_back(Event::PointerMoved{point});
        pending.push_back(Event::Wheel{Point(0.0f, notches)});
        return run_frame();
    }

    // Presses the key, and lets go of it the next frame.
    Headless& key(Key key) {
        return key_with(Modifiers::None, key);
    }

    // Presses the key with the modifiers, like Ctrl+S, and lets go of them.
    Headless& key_with(Modifiers modifiers, Key key) {
        std::vector<Key> keys;
        for (auto [held, modifier] : {std::pair(modifiers.control, Key::Control), std::pair(modifiers.shift, Key::Shift), std::pair(modifiers.alt, Key::Alt)}) {
            if (held) {
                keys.push_back(modifier);
            }
        }
        keys.push_back(key);
        for (Key pressed : keys) {
            pending.push_back(Event::KeyPressed{.key = pressed, .repeat = false});
        }
        run_frame();
        for (Key released : keys | std::views::reverse) {
            pending.push_back(Event::KeyReleased{released});
        }
        return run_frame();
    }

    // Presses the key and holds it, until release_key.
    Headless& hold(Key key) {
        pending.push_back(Event::KeyPressed{.key = key, .repeat = false});
        return run_frame();
    }

    Headless& release_key(Key key) {
        pending.push_back(Event::KeyReleased{key});
        return run_frame();
    }

    // A held key's repeat, as Windows sends while it stays down.
    Headless& repeat(Key key) {
        pending.push_back(Event::KeyPressed{.key = key, .repeat = true});
        return run_frame();
    }

    // Sends the events as they are, all in one frame.
    Headless& send(std::span<const Event> events) {
        pending.insert(pending.end(), events.begin(), events.end());
        return run_frame();
    }

    // Queues the events for the next frame without running it, unlike
    // send, so that a test can look at the very frame that reads them,
    // which frame runs.
    Headless& queue(std::span<const Event> events) {
        pending.insert(pending.end(), events.begin(), events.end());
        return *this;
    }

    // Types the text, all in one frame.
    Headless& type_text(std::string_view text) {
        for (char32_t character : base::chars(text)) {
            pending.push_back(Event::Text{character});
        }
        return run_frame();
    }

    Headless& set_gamepad(Gamepad reading) {
        gamepad = reading;
        return run_frame();
    }

    // Gives the window focus, or takes it away.
    Headless& focus(bool focused) {
        if (focused) {
            pending.push_back(Event::FocusGained{});
        } else {
            pending.push_back(Event::FocusLost{});
        }
        return run_frame();
    }

    // Draws the last frame offscreen, as the window would, with the
    // program's 3D. The first call prepares the program. Throws a
    // base::WindowsError if Direct3D can't.
    graphics::Image render(graphics::Gpu& gpu) {
        Size size = this->size();
        Color background = ui.theme().background;
        std::array<float, 4> clear{background.r, background.g, background.b, 1.0f};
        graphics::Offscreen target(gpu, base::saturating_cast<std::uint32_t>(size.width), base::saturating_cast<std::uint32_t>(size.height), clear);
        graphics::Frame frame = target.begin();
        frame.clear(clear);
        record(gpu, frame);
        return target.finish(gpu, std::move(frame));
    }

    // Records the last frame's drawing into a frame of the caller's, as
    // the window would draw it over the theme's background: the program's
    // 3D, then the interface. For drawing a frame into a target that's kept
    // from frame to frame, like a benchmark's. The first call prepares the
    // program. Throws a base::WindowsError if Direct3D can't.
    void record(graphics::Gpu& gpu, const graphics::Frame& frame) {
        if (!renderer) {
            program.prepare(gpu);
            renderer.emplace(gpu);
        }
        program.render(gpu, frame);
        renderer->draw(gpu, frame, ui.draw, ui.fonts_);
    }

private:
    Headless(P program, ThemeKind theme) : program(std::move(program)), ui(headless_detail::fonts(), theme, Clipboard::Own{}) {
        pending.push_back(Event::FocusGained{});
        ui.environment_.scripted = true;
        // Frames would wait for vertical sync, as a real run's start out.
        ui.environment_.vertical_sync = true;
        ui.performance = performance::Performance::without_latency("none in a test");
        run_frame();
    }

    Headless& run_frame() {
        frame_ += 1;
        std::vector<Event> events = std::exchange(pending, {});
        events.push_back(Event::Gamepad{gamepad});
        std::vector<Message> messages = std::exchange(sent, {});
        Size size = this->size();
        double time = this->time();
        ui.frame(program, shown, events, std::move(messages), time, size, fullscreen_);
        for (Request request : std::exchange(ui.requests, {})) {
            switch (request) {
            case Request::ToggleFullscreen:
                fullscreen_ = !fullscreen_;
                break;
            // The environment already switched when the request was
            // made, so the frame that asked says the new setting; a
            // window would switch its presenter here.
            case Request::ToggleVerticalSync:
                break;
            case Request::Close:
                closed_ = true;
                break;
            }
        }
        return *this;
    }

    // The tree the last frame showed, which the next frame's input is on.
    std::optional<Tree<Message>> shown;
    std::uint64_t frame_ = 0;
    bool fullscreen_ = false;
    std::vector<Event> pending;
    // Messages a test sends, applied in the next frame before the input's.
    std::vector<Message> sent;
    Point pointer_ = Point::Zero;
    Gamepad gamepad;
    std::optional<Renderer> renderer;
    // A window of a size other than the supported setups', like an overlay.
    std::optional<Size> custom_size;
    bool closed_ = false;
};

} // namespace user_interface
