// A program's Win32 window. Its input comes from GameInput
// (Platform/InputDevices.h and Input/Reader.h), not from its messages,
// which it handles only to be a window: its focus, its cursor, its size and
// its close button.
//
// - It's maximized or borderless fullscreen, the two supported setups, and
//   switches between them without taking focus.
// - It says when it gets and loses focus, on the clock that input is timed
//   on, as input without focus is ignored. The time is when it handles the
//   message, which Windows sends it once the focus has moved, so it can be
//   a little after.
// - Drags keep going past its edges, as it captures the mouse while a button
//   is held.
// - It says when its setup may have changed: when it moves or changes size,
//   or the displays, their scale or the taskbar change.
// - It asks for a dark title bar in the dark theme, as Windows 11's apps
//   have, with DWMWA_USE_IMMERSIVE_DARK_MODE, which Microsoft's documentation
//   describes as drawing the frame dark while Windows' own dark mode is on.

#pragma once

#include "Base/Windows.h"
#include "UserInterface/Application/Settings.h"
#include "UserInterface/Core/Interaction.h"
#include "UserInterface/Drawing/Geometry.h"

#include <array>
#include <cstdint>
#include <optional>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

namespace user_interface {

// The one supported display and its taskbar.
inline constexpr std::int32_t DisplayWidth = 2560;
inline constexpr std::int32_t DisplayHeight = 1440;
inline constexpr std::int32_t TaskbarHeight = 48;
inline constexpr std::int32_t MaximizedClientHeight = 1369;

// The window got focus, or lost it, at a time on the counter.
struct FocusChange {
    std::int64_t time = 0;
    bool gained = false;

    bool operator==(const FocusChange&) const = default;
};

namespace window_detail {

// What the window procedure shares with the window, which Window.cpp
// defines.
struct Shared;

} // namespace window_detail

class Window {
public:
    // A program's window on the monitor, hidden until shown. Throws a
    // base::WindowsError if Windows can't make it.
    static Window create(std::string_view title, HMONITOR monitor, WindowMode mode);

    // A window without a frame that floats over the others, like one of
    // IdentifyDisplays' boxes, in screen pixels. Throws a base::WindowsError
    // if Windows can't make it.
    static Window create_overlay(std::string_view title, RECT rect);

    ~Window();
    Window(const Window&) = delete;
    Window& operator=(const Window&) = delete;
    Window(Window&& other) noexcept;
    Window& operator=(Window&& other) noexcept;

    HWND hwnd() const;

    // Shows the window. Without `activate`, it opens beneath the others,
    // without taking focus from the window that has it.
    void show(bool activate);

    WindowMode mode() const;

    // Switches between maximized and borderless fullscreen, on the monitor
    // that the window is on, without taking focus.
    void set_mode(WindowMode mode);

    // Dark or light, as the theme is, like the title bars of Windows 11's
    // apps. DWM draws the dark one; its documentation promises it while
    // Windows' own dark mode is on.
    void set_dark_title_bar(bool dark);

    // Handles the window's messages without waiting. False once it should
    // close.
    bool pump();

    // Waits for a message, like while the window is minimized.
    void wait() const;

    // Waits for a message, or for the milliseconds to pass.
    void wait_for(std::uint32_t milliseconds) const;

    // The focus changes since the last call, oldest first.
    std::vector<FocusChange> take_focus_changes();

    // Where Windows' cursor is, in the client area's pixels, and whether it's
    // on the client area, with no other window over it there.
    std::pair<Point, bool> cursor();

    // Whether Windows' cursor is on the client area.
    bool cursor_over();

    // Captures the mouse, so that a drag keeps going past the window's edges,
    // or lets it go.
    void set_capture(bool capture);

    bool focused() const;

    bool minimized() const;

    // Whether it was asked to close, like by its close button or Alt+F4.
    bool close_requested() const;

    // Keeps the window open after all, when the program says it may not
    // close yet.
    void cancel_close();

    // The client area's width and height.
    std::pair<std::uint32_t, std::uint32_t> client_size() const;

    HMONITOR monitor() const;

    void set_title(std::string_view title) const;

    void set_cursor(Cursor cursor);

    // Whether the setup may have changed since the last call.
    bool take_setup_changed();

    // What's wrong with the setup, if anything. Programs support only a
    // 2560 × 1440 display at 100% scale with a 48-pixel taskbar at its
    // bottom, and the window maximized or borderless fullscreen.
    std::vector<std::string> setup_problems() const;

private:
    // The window's object for a window that Windows made.
    explicit Window(HWND hwnd);

    window_detail::Shared& shared() const;

    void apply_mode(WindowMode mode, bool visible);

    // Destroys the window, if this still has one, and then frees what its
    // procedure shared with it.
    void destroy();

    HWND hwnd_;
    // On the heap, so that it stays where GWLP_USERDATA's copy of the
    // pointer says it is when the Window moves. The window procedure goes on
    // using it until the destructor frees it, after the window is destroyed.
    window_detail::Shared* shared_;
    WindowMode mode_;
    std::array<HCURSOR, 10> cursors;
    std::optional<bool> dark_title_bar;
};

// The library's own: what Windows says about the monitor, like its area and
// its work area.
MONITORINFO monitor_info(HMONITOR monitor);

} // namespace user_interface
