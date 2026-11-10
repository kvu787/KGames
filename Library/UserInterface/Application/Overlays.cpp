#include "UserInterface/Application/Overlays.h"

#include "GameInput/GameInput.h"
#include "Graphics/Graphics.h"
#include "UserInterface/Application/Attempt.h"
#include "UserInterface/Drawing/Geometry.h"
#include "UserInterface/Drawing/Renderer.h"
#include "UserInterface/Drawing/Text.h"
#include "UserInterface/Input/Devices.h"
#include "UserInterface/Platform/Clipboard.h"
#include "UserInterface/Platform/InputDevices.h"
#include "UserInterface/Platform/Window.h"

#include <algorithm>
#include <chrono>
#include <cstdint>
#include <format>
#include <optional>
#include <span>
#include <utility>
#include <vector>

namespace user_interface {

namespace {

// An overlay that's shown. Its members are destroyed last to first, and are
// in this order for that: the window goes first, and the presenter, which
// waits for the GPU to finish its last frame as it goes, before the renderer
// and its buffers.
struct Shown {
    std::uint32_t frames;
    Interface ui;
    Renderer renderer;
    graphics::Presenter presenter;
    Window window;
};

// Frames that an overlay draws before the loop waits: the second makes sure
// the first reached the screen.
constexpr std::uint32_t Frames = 2;

// How long the loop waits for messages between checks of the input, once
// the overlays have drawn.
constexpr std::uint32_t WaitMilliseconds = 4;

// Whether the device's button or key went down, and remembers that it's down.
bool went_down(std::vector<std::pair<std::optional<size_t>, bool>>& devices, std::optional<size_t> device, bool down) {
    for (auto& [known, was_down] : devices) {
        if (known == device) {
            return !std::exchange(was_down, down) && down;
        }
    }
    devices.emplace_back(device, down);
    return false;
}

// Whether a left click or Esc came since the last check. Each device's first
// reading only says what's held already.
struct Presses {
    std::vector<std::pair<std::optional<size_t>, bool>> mice;
    std::vector<std::pair<std::optional<size_t>, bool>> keyboards;

    // Whether the left button went down while the cursor was on an overlay,
    // or Esc went down while one had focus.
    bool closing(const Devices& devices, std::span<Shown> shown) {
        bool clicked = false;
        for (const Reading& reading : devices.readings(game_input::GameInputKindMouse)) {
            bool down = (reading.mouse.buttons & game_input::GameInputMouseLeftButton) != 0;
            clicked |= went_down(mice, reading.device, down);
        }
        bool escaped = false;
        for (const Reading& reading : devices.readings(game_input::GameInputKindKeyboard)) {
            bool down = std::ranges::any_of(reading.keys, [](const game_input::GameInputKeyState& key) { return key.virtualKey == Key::Escape.code; });
            escaped |= went_down(keyboards, reading.device, down);
        }
        return (clicked && std::ranges::any_of(shown, [](Shown& overlay) { return overlay.window.cursor_over(); })) ||
               (escaped && std::ranges::any_of(shown, [](const Shown& overlay) { return overlay.window.focused(); }));
    }
};

} // namespace

std::expected<void, std::string> overlays_detail::run_overlays(std::span<const Overlay> overlays, ThemeKind theme, const ViewFunction& view) {
    std::expected<graphics::Gpu, std::string> gpu_started = attempt("Couldn't start Direct3D 12", [] { return graphics::Gpu(graphics::AdapterChoice::HighPerformance{}); });
    if (!gpu_started) {
        return std::unexpected(std::move(gpu_started.error()));
    }
    graphics::Gpu& gpu = *gpu_started;
    std::expected<Devices, std::string> devices = Devices::create();
    if (!devices) {
        return std::unexpected(std::format("Couldn't start GameInput, which reads the click or Esc that closes the overlays: {}", devices.error()));
    }
    Presses presses;
    std::vector<Shown> shown;
    for (const Overlay& overlay : overlays) {
        std::expected<Window, std::string> window = attempt("Couldn't open a window", [&] { return Window::create_overlay(overlay.title, overlay.rect); });
        if (!window) {
            return std::unexpected(std::move(window.error()));
        }
        std::pair<std::uint32_t, std::uint32_t> size = window->client_size();
        std::expected<graphics::Presenter, std::string> presenter =
            attempt("Couldn't make a swap chain", [&] { return graphics::Presenter(gpu, window->hwnd(), size.first, size.second, false); });
        if (!presenter) {
            return std::unexpected(std::move(presenter.error()));
        }
        std::expected<Renderer, std::string> renderer = attempt("Couldn't make a renderer", [&] { return Renderer(gpu); });
        if (!renderer) {
            return std::unexpected(std::move(renderer.error()));
        }
        std::expected<Fonts, std::string> fonts = attempt("Couldn't start DirectWrite", [] { return Fonts(); });
        if (!fonts) {
            return std::unexpected(std::move(fonts.error()));
        }
        shown.push_back(Shown{
            .frames = 0,
            .ui = Interface(std::move(*fonts), theme, Clipboard::Own{}),
            .renderer = std::move(*renderer),
            .presenter = std::move(*presenter),
            .window = std::move(*window),
        });
    }
    for (Shown& overlay : shown) {
        overlay.window.show(true);
    }
    std::chrono::steady_clock::time_point start = std::chrono::steady_clock::now();
    while (true) {
        for (Shown& overlay : shown) {
            overlay.window.pump();
        }
        devices->read();
        if (std::ranges::any_of(shown, [](const Shown& overlay) { return overlay.window.close_requested(); }) || presses.closing(*devices, shown)) {
            // The swap chains go as this returns, before the GPU does, so the
            // GPU must be done with their last frames first.
            return attempt("Couldn't wait for the GPU", [&] { gpu.flush(); });
        }
        bool waiting = false;
        for (size_t index = 0; index < shown.size(); index++) {
            Shown& overlay = shown[index];
            if (overlay.frames >= Frames) {
                continue;
            }
            if (!overlay.presenter.poll_ready(gpu)) {
                waiting = true;
                continue;
            }
            std::pair<std::uint32_t, std::uint32_t> size = overlay.presenter.size();
            Ui ui{overlay.ui};
            Tree<std::monostate> tree = view(ui, index);
            double time = std::chrono::duration<double>(std::chrono::steady_clock::now() - start).count();
            overlay.ui.frame_of(tree, {}, time, Size(static_cast<float>(size.first), static_cast<float>(size.second)), false);
            std::expected<graphics::Frame, std::string> frame = attempt("Couldn't begin a frame", [&] { return overlay.presenter.begin(); });
            if (!frame) {
                return std::unexpected(std::move(frame.error()));
            }
            Color background = overlay.ui.theme().background;
            frame->clear({background.r, background.g, background.b, 1.0f});
            if (std::expected<void, std::string> drawn = attempt("Couldn't draw", [&] { overlay.renderer.draw(gpu, *frame, overlay.ui.draw, overlay.ui.fonts_); }); !drawn) {
                return drawn;
            }
            if (std::expected<void, std::string> presented = attempt("Couldn't present", [&] { overlay.presenter.present(gpu, std::move(*frame)); }); !presented) {
                return presented;
            }
            gpu.report_messages();
            overlay.frames += 1;
            waiting |= overlay.frames < Frames;
        }
        if (!waiting && !shown.empty()) {
            shown.front().window.wait_for(WaitMilliseconds);
        }
    }
}

} // namespace user_interface
