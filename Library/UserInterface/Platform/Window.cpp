#include "UserInterface/Platform/Window.h"

#include "Base/Text.h"
#include "UserInterface/Platform/Clock.h"

#include <dwmapi.h>

#include <algorithm>
#include <format>
#include <mutex>

#pragma comment(lib, "dwmapi.lib")
#pragma comment(lib, "user32.lib")

namespace user_interface {

namespace window_detail {

// What the window procedure shares with the window, by GWLP_USERDATA.
//
// The window procedure runs inside the Windows calls that send the window
// messages, while one of the window's methods is running, like
// DispatchMessageW in `pump`, SetWindowPos in `apply_mode` and `show`,
// ShowWindow, SetCapture and ReleaseCapture. It runs inside its own calls
// too, like ReleaseCapture and DefWindowProcW. So both read and change it
// through the pointer, where it is, and neither keeps a copy of any of it
// across a call that can run the other.
struct Shared {
    std::vector<FocusChange> focus_changes;
    bool focused = false;
    bool minimized = false;
    bool close_requested = false;
    bool setup_changed = true;
    // Whether the cursor was on the client area when last asked.
    bool cursor_inside = false;
    HCURSOR cursor = nullptr;

    // The window got focus, or lost it, now.
    void focus_changed(bool gained) {
        std::int64_t time = clock::now();
        focused = gained;
        focus_changes.push_back(FocusChange{.time = time, .gained = gained});
    }
};

} // namespace window_detail

namespace {

using window_detail::Shared;

// RunInBackground finds programs' windows by this class.
constexpr const wchar_t* Class = L"MainWindow";
constexpr const wchar_t* OverlayClass = L"OverlayWindow";

constexpr std::array<Cursor, 10> Cursors{
    Cursor::Arrow,     Cursor::Text,       Cursor::Hand, Cursor::Move, Cursor::ResizeHorizontal, Cursor::ResizeVertical, Cursor::ResizeDiagonal, Cursor::ResizeAntidiagonal,
    Cursor::Crosshair, Cursor::NotAllowed,
};

// Once a process: per-monitor DPI awareness, so that Windows never scales the
// window.
void prepare_process() {
    static std::once_flag prepared;
    std::call_once(prepared, [] { SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2); });
}

// Whether Windows may throttle the process's execution speed, as it does on
// efficiency cores while a process's window is in the background. A window
// that someone uses asks it not to, so that the main loop stays fast. One
// that opens without focus, like a coding agent's test run while someone
// plays a game, asks for efficiency mode instead, so that it runs at
// efficient clocks and the chip's power budget stays with the game.
void set_power_throttling(bool throttle) {
    PROCESS_POWER_THROTTLING_STATE state{
        .Version = PROCESS_POWER_THROTTLING_CURRENT_VERSION,
        .ControlMask = PROCESS_POWER_THROTTLING_EXECUTION_SPEED,
        .StateMask = throttle ? static_cast<ULONG>(PROCESS_POWER_THROTTLING_EXECUTION_SPEED) : 0,
    };
    SetProcessInformation(GetCurrentProcess(), ProcessPowerThrottling, &state, sizeof(PROCESS_POWER_THROTTLING_STATE));
}

std::int32_t low_word(LPARAM value) {
    return static_cast<std::int16_t>(static_cast<std::uint16_t>(value & 0xFFFF));
}

// noexcept, so that an exception stops the program with its reason, as a
// panic in a function that Windows calls does in Rust, instead of unwinding
// into Windows.
LRESULT CALLBACK window_procedure(HWND hwnd, UINT message, WPARAM wparam, LPARAM lparam) noexcept {
    Shared* shared = reinterpret_cast<Shared*>(GetWindowLongPtrW(hwnd, GWLP_USERDATA));
    if (shared == nullptr) {
        return DefWindowProcW(hwnd, message, wparam, lparam);
    }
    switch (message) {
    case WM_SETFOCUS:
        shared->focus_changed(true);
        return 0;
    case WM_KILLFOCUS:
        shared->focus_changed(false);
        ReleaseCapture();
        return 0;
    case WM_SYSCOMMAND:
        // Alt alone would open the window's menu, which takes the keys.
        if ((wparam & 0xFFF0) == SC_KEYMENU) {
            return 0;
        }
        return DefWindowProcW(hwnd, message, wparam, lparam);
    case WM_SETCURSOR:
        if (low_word(lparam) == HTCLIENT) {
            SetCursor(shared->cursor);
            return 1;
        }
        return DefWindowProcW(hwnd, message, wparam, lparam);
    case WM_SIZE:
        shared->minimized = static_cast<std::int32_t>(wparam) == SIZE_MINIMIZED;
        shared->setup_changed = true;
        return 0;
    case WM_MOVE:
    case WM_DISPLAYCHANGE:
    case WM_SETTINGCHANGE:
        shared->setup_changed = true;
        return DefWindowProcW(hwnd, message, wparam, lparam);
    case WM_DPICHANGED:
        // It keeps its size: the check says what's wrong instead.
        shared->setup_changed = true;
        return 0;
    case WM_ERASEBKGND:
        return 1;
    case WM_PAINT:
        ValidateRect(hwnd, nullptr);
        return 0;
    case WM_CLOSE:
        shared->close_requested = true;
        return 0;
    default:
        return DefWindowProcW(hwnd, message, wparam, lparam);
    }
}

void register_class(const wchar_t* name) {
    WNDCLASSEXW window_class{
        .cbSize = sizeof(WNDCLASSEXW),
        .lpfnWndProc = window_procedure,
        .hInstance = GetModuleHandleW(nullptr),
        .lpszClassName = name,
    };
    // ERROR_CLASS_ALREADY_EXISTS: a second window of the class.
    if (RegisterClassExW(&window_class) == 0) {
        base::WindowsError error = base::last_error();
        if (static_cast<std::uint32_t>(error.code()) != 0x80070582) {
            throw error;
        }
    }
}

} // namespace

Window Window::create(std::string_view title, HMONITOR monitor, WindowMode mode) {
    prepare_process();
    register_class(Class);
    // Where it would be if it were ever restored: half the work area,
    // in its middle.
    RECT work = monitor_info(monitor).rcWork;
    int width = (work.right - work.left) / 2;
    int height = (work.bottom - work.top) / 2;
    std::wstring wide_title = base::widen(title);
    HWND hwnd = CreateWindowExW(WS_EX_APPWINDOW, Class, wide_title.c_str(), WS_OVERLAPPEDWINDOW, work.left + width / 2, work.top + height / 2, width, height, nullptr, nullptr,
                                GetModuleHandleW(nullptr), nullptr);
    if (hwnd == nullptr) {
        throw base::last_error();
    }
    Window window(hwnd);
    window.apply_mode(mode, false);
    return window;
}

Window Window::create_overlay(std::string_view title, RECT rect) {
    prepare_process();
    register_class(OverlayClass);
    std::wstring wide_title = base::widen(title);
    HWND hwnd = CreateWindowExW(WS_EX_TOPMOST | WS_EX_TOOLWINDOW, OverlayClass, wide_title.c_str(), WS_POPUP, rect.left, rect.top, rect.right - rect.left, rect.bottom - rect.top,
                                nullptr, nullptr, GetModuleHandleW(nullptr), nullptr);
    if (hwnd == nullptr) {
        throw base::last_error();
    }
    return Window(hwnd);
}

Window::Window(HWND hwnd) : hwnd_(hwnd), shared_(nullptr), mode_(WindowMode::Maximized), cursors{} {
    const std::array<const wchar_t*, 10> names{
        IDC_ARROW, IDC_IBEAM, IDC_HAND, IDC_SIZEALL, IDC_SIZEWE, IDC_SIZENS, IDC_SIZENWSE, IDC_SIZENESW, IDC_CROSS, IDC_NO,
    };
    for (size_t index = 0; index < names.size(); index++) {
        cursors[index] = LoadCursorW(nullptr, names[index]);
    }
    shared_ = new Shared{.cursor = cursors[0]};
    SetWindowLongPtrW(hwnd, GWLP_USERDATA, reinterpret_cast<LONG_PTR>(shared_));
}

Window::~Window() {
    destroy();
}

Window::Window(Window&& other) noexcept :
    hwnd_(std::exchange(other.hwnd_, nullptr)), shared_(std::exchange(other.shared_, nullptr)), mode_(other.mode_), cursors(other.cursors), dark_title_bar(other.dark_title_bar) {}

Window& Window::operator=(Window&& other) noexcept {
    if (this != &other) {
        destroy();
        hwnd_ = std::exchange(other.hwnd_, nullptr);
        shared_ = std::exchange(other.shared_, nullptr);
        mode_ = other.mode_;
        cursors = other.cursors;
        dark_title_bar = other.dark_title_bar;
    }
    return *this;
}

void Window::destroy() {
    if (hwnd_ == nullptr) {
        return;
    }
    SetWindowLongPtrW(hwnd_, GWLP_USERDATA, 0);
    DestroyWindow(hwnd_);
    // Nothing can read it now: the window procedure finds no pointer,
    // and the window is gone.
    delete shared_;
    hwnd_ = nullptr;
    shared_ = nullptr;
}

HWND Window::hwnd() const {
    return hwnd_;
}

window_detail::Shared& Window::shared() const {
    // Freed only by `destroy`.
    return *shared_;
}

void Window::show(bool activate) {
    if (activate) {
        set_power_throttling(false);
        ShowWindow(hwnd_, SW_SHOW);
        return;
    }
    // A window that mustn't take focus, like a test run's while
    // someone works or plays a game, goes beneath the other windows,
    // where it never covers theirs or takes their pointer, and a click
    // on it never activates it. Screenshots of it work all the same.
    set_power_throttling(true);
    LONG_PTR style = GetWindowLongPtrW(hwnd_, GWL_EXSTYLE);
    SetWindowLongPtrW(hwnd_, GWL_EXSTYLE, style | WS_EX_NOACTIVATE);
    ShowWindow(hwnd_, SW_SHOWNA);
    UINT flags = SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_NOOWNERZORDER;
    SetWindowPos(hwnd_, HWND_BOTTOM, 0, 0, 0, 0, flags);
}

WindowMode Window::mode() const {
    return mode_;
}

void Window::set_mode(WindowMode mode) {
    if (mode != mode_) {
        apply_mode(mode, true);
    }
}

void Window::apply_mode(WindowMode mode, bool visible) {
    mode_ = mode;
    MONITORINFO info = monitor_info(MonitorFromWindow(hwnd_, MONITOR_DEFAULTTONEAREST));
    DWORD shown = visible ? WS_VISIBLE : 0;
    UINT flags = SWP_NOACTIVATE | SWP_FRAMECHANGED | SWP_NOZORDER | SWP_NOOWNERZORDER;
    DWORD style = 0;
    RECT area{};
    switch (mode) {
    case WindowMode::BorderlessFullscreen:
        style = WS_POPUP | shown;
        area = info.rcMonitor;
        break;
    case WindowMode::Maximized: {
        // Maximized as Windows would, but without activating, which
        // Windows has no documented way to do: ShowWindow's
        // SW_SHOWMAXIMIZED activates the window. So the style says
        // maximized, though the documentation doesn't list
        // WS_MAXIMIZE among the styles that may change after the
        // window is made, and the window takes the work area, with
        // the frame's borders past its edges, which leaves the
        // title bar at its top.
        UINT dpi = std::max<UINT>(GetDpiForWindow(hwnd_), USER_DEFAULT_SCREEN_DPI);
        RECT frame{};
        AdjustWindowRectExForDpi(&frame, WS_OVERLAPPEDWINDOW, FALSE, WS_EX_APPWINDOW, dpi);
        LONG border = frame.right;
        RECT work = info.rcWork;
        area = RECT{
            .left = work.left - border,
            .top = work.top - border,
            .right = work.right + border,
            .bottom = work.bottom + border,
        };
        style = WS_OVERLAPPEDWINDOW | WS_MAXIMIZE | shown;
        break;
    }
    }
    SetWindowLongPtrW(hwnd_, GWL_STYLE, static_cast<LONG_PTR>(style));
    SetWindowPos(hwnd_, nullptr, area.left, area.top, area.right - area.left, area.bottom - area.top, flags);
    shared().setup_changed = true;
}

void Window::set_dark_title_bar(bool dark) {
    if (dark_title_bar == dark) {
        return;
    }
    dark_title_bar = dark;
    BOOL value = dark;
    DwmSetWindowAttribute(hwnd_, DWMWA_USE_IMMERSIVE_DARK_MODE, &value, sizeof(BOOL));
}

bool Window::pump() {
    MSG message{};
    while (PeekMessageW(&message, nullptr, 0, 0, PM_REMOVE)) {
        if (message.message == WM_QUIT) {
            return false;
        }
        // No TranslateMessage: typed text comes from GameInput's keys,
        // and translating the messages too would use up dead keys.
        DispatchMessageW(&message);
    }
    return !shared().close_requested;
}

void Window::wait() const {
    WaitMessage();
}

void Window::wait_for(std::uint32_t milliseconds) const {
    MsgWaitForMultipleObjects(0, nullptr, FALSE, milliseconds, QS_ALLINPUT);
}

std::vector<FocusChange> Window::take_focus_changes() {
    return std::exchange(shared().focus_changes, {});
}

std::pair<Point, bool> Window::cursor() {
    POINT point{};
    GetCursorPos(&point);
    bool over = WindowFromPoint(point) == hwnd_;
    ScreenToClient(hwnd_, &point);
    auto [width, height] = client_size();
    bool inside = over && point.x >= 0 && point.y >= 0 && static_cast<std::uint32_t>(point.x) < width && static_cast<std::uint32_t>(point.y) < height;
    shared().cursor_inside = inside;
    return {Point(static_cast<float>(point.x), static_cast<float>(point.y)), inside};
}

bool Window::cursor_over() {
    return cursor().second;
}

void Window::set_capture(bool capture) {
    bool captured = GetCapture() == hwnd_;
    if (capture && !captured) {
        SetCapture(hwnd_);
    } else if (!capture && captured) {
        ReleaseCapture();
    }
}

bool Window::focused() const {
    return shared().focused;
}

bool Window::minimized() const {
    return shared().minimized;
}

bool Window::close_requested() const {
    return shared().close_requested;
}

void Window::cancel_close() {
    shared().close_requested = false;
}

std::pair<std::uint32_t, std::uint32_t> Window::client_size() const {
    RECT rect{};
    GetClientRect(hwnd_, &rect);
    return {static_cast<std::uint32_t>(std::max<LONG>(rect.right - rect.left, 0)), static_cast<std::uint32_t>(std::max<LONG>(rect.bottom - rect.top, 0))};
}

HMONITOR Window::monitor() const {
    return MonitorFromWindow(hwnd_, MONITOR_DEFAULTTONEAREST);
}

void Window::set_title(std::string_view title) const {
    std::wstring wide_title = base::widen(title);
    SetWindowTextW(hwnd_, wide_title.c_str());
}

void Window::set_cursor(Cursor cursor) {
    auto known = std::ranges::find(Cursors, cursor);
    size_t index = known == Cursors.end() ? 0 : static_cast<size_t>(known - Cursors.begin());
    HCURSOR handle = cursors[index];
    Shared& state = shared();
    if (state.cursor != handle) {
        state.cursor = handle;
        // The pointer may be still, so set it now, not at its next move.
        if (state.focused && state.cursor_inside) {
            SetCursor(handle);
        }
    }
}

bool Window::take_setup_changed() {
    return std::exchange(shared().setup_changed, false);
}

std::vector<std::string> Window::setup_problems() const {
    std::vector<std::string> problems;
    MONITORINFO info = monitor_info(monitor());
    RECT display = info.rcMonitor;
    LONG width = display.right - display.left;
    LONG height = display.bottom - display.top;
    if (width != DisplayWidth || height != DisplayHeight) {
        problems.push_back(std::format("The display is {} × {} pixels. It must be {} × {}.", width, height, DisplayWidth, DisplayHeight));
    }
    UINT dpi = GetDpiForWindow(hwnd_);
    if (dpi != USER_DEFAULT_SCREEN_DPI) {
        problems.push_back(std::format("The display's scale is {}%. It must be 100%.", dpi * 100 / USER_DEFAULT_SCREEN_DPI));
    }
    auto [client_width, client_height] = client_size();
    std::int32_t signed_width = static_cast<std::int32_t>(client_width);
    std::int32_t signed_height = static_cast<std::int32_t>(client_height);
    switch (mode_) {
    case WindowMode::Maximized: {
        RECT work = info.rcWork;
        bool taskbar_right = work.left == display.left && work.top == display.top && work.right == display.right && display.bottom - work.bottom == TaskbarHeight;
        if (!taskbar_right) {
            problems.push_back(std::format("The taskbar must be {} pixels tall, at the bottom of the display.", TaskbarHeight));
        }
        if (!IsZoomed(hwnd_)) {
            problems.push_back("The window must be maximized, or borderless fullscreen with F11.");
        } else if (signed_width != DisplayWidth || signed_height != MaximizedClientHeight) {
            problems.push_back(
                std::format("The window's client area is {} × {}. Maximized, it must be {} × {}.", client_width, client_height, DisplayWidth, MaximizedClientHeight));
        }
        break;
    }
    case WindowMode::BorderlessFullscreen:
        if (signed_width != DisplayWidth || signed_height != DisplayHeight) {
            problems.push_back(
                std::format("The window's client area is {} × {}. In borderless fullscreen, it must be {} × {}.", client_width, client_height, DisplayWidth, DisplayHeight));
        }
        break;
    }
    return problems;
}

MONITORINFO monitor_info(HMONITOR monitor) {
    MONITORINFO info{.cbSize = sizeof(MONITORINFO)};
    GetMonitorInfoW(monitor, &info);
    return info;
}

} // namespace user_interface
