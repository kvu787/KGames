using System.Runtime.InteropServices;

namespace WinUi3Sample;

// What WinUI 3 has no API for: showing the window maximized without activating it, as
// RunInBackground needs, so that it opens beneath the other windows, and typing in the window that
// has focus stays there. Library/UserInterface's window.rs does the same for the other programs.
static unsafe partial class NativeWindow {
    static nint activationHook;

    // Keeps every window of this thread from being activated from now on, which would take focus
    // from the user's window: the app's window, and WinUI's menus, and the flyouts and popups that
    // can go past the window's edges, which are windows of their own, and which activate when they
    // open. Windows asks a CBT hook before it activates a window, and the hook says no.
    public static void PreventActivation() {
        if (activationHook == 0) {
            activationHook = SetWindowsHookExW(WH_CBT, &OnCbt, 0, GetCurrentThreadId());
            if (activationHook == 0) {
                throw new StartupException($"Couldn't keep the window from taking focus: error {Marshal.GetLastPInvokeError()}");
            }
        }
    }

    [UnmanagedCallersOnly]
    static nint OnCbt(int code, nint wParam, nint lParam) {
        if (code == HCBT_ACTIVATE) {
            return 1;
        }
        return CallNextHookEx(0, code, wParam, lParam);
    }

    public static void ShowMaximizedWithoutActivating(nint window, Displays.Rect workArea) {
        // Clicks on the window never activate it either.
        var extendedStyle = GetWindowLongPtrW(window, GWL_EXSTYLE);
        _ = SetWindowLongPtrW(window, GWL_EXSTYLE, extendedStyle | WS_EX_NOACTIVATE);

        // Maximized as Windows would, but without activating: the work area, with the frame's
        // borders past its edges.
        var dpi = Math.Max(GetDpiForWindow(window), 96u);
        var frame = new Displays.Rect();
        _ = AdjustWindowRectExForDpi(ref frame, WS_OVERLAPPEDWINDOW, false, 0, dpi);
        var border = frame.Right;
        var style = GetWindowLongPtrW(window, GWL_STYLE);
        _ = SetWindowLongPtrW(window, GWL_STYLE, style | WS_MAXIMIZE);
        _ = SetWindowPos(window, 0, workArea.Left - border, workArea.Top - border,
            workArea.Width + 2 * border, workArea.Height + 2 * border,
            SWP_NOACTIVATE | SWP_FRAMECHANGED | SWP_NOZORDER | SWP_NOOWNERZORDER);

        _ = ShowWindow(window, SW_SHOWNA);
        _ = SetWindowPos(window, HWND_BOTTOM, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_NOOWNERZORDER);
    }

    const int WH_CBT = 5;
    const int HCBT_ACTIVATE = 5;
    const int GWL_STYLE = -16;
    const int GWL_EXSTYLE = -20;
    const nint WS_EX_NOACTIVATE = 0x08000000;
    const nint WS_MAXIMIZE = 0x01000000;
    const uint WS_OVERLAPPEDWINDOW = 0x00CF0000;
    const int SW_SHOWNA = 8;
    const nint HWND_BOTTOM = 1;
    const uint SWP_NOSIZE = 0x0001;
    const uint SWP_NOMOVE = 0x0002;
    const uint SWP_NOZORDER = 0x0004;
    const uint SWP_NOACTIVATE = 0x0010;
    const uint SWP_FRAMECHANGED = 0x0020;
    const uint SWP_NOOWNERZORDER = 0x0200;

    [LibraryImport("user32.dll", SetLastError = true)]
    private static partial nint SetWindowsHookExW(int hookType, delegate* unmanaged<int, nint, nint, nint> callback, nint module, uint threadId);
    [LibraryImport("user32.dll")]
    private static partial nint CallNextHookEx(nint hook, int code, nint wParam, nint lParam);
    [LibraryImport("kernel32.dll")]
    private static partial uint GetCurrentThreadId();
    [LibraryImport("user32.dll")]
    private static partial nint GetWindowLongPtrW(nint window, int index);
    [LibraryImport("user32.dll")]
    private static partial nint SetWindowLongPtrW(nint window, int index, nint value);
    [LibraryImport("user32.dll")]
    private static partial uint GetDpiForWindow(nint window);
    [LibraryImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static partial bool AdjustWindowRectExForDpi(ref Displays.Rect rect, uint style, [MarshalAs(UnmanagedType.Bool)] bool menu, uint extendedStyle, uint dpi);
    [LibraryImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static partial bool SetWindowPos(nint window, nint insertAfter, int x, int y, int width, int height, uint flags);
    [LibraryImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static partial bool ShowWindow(nint window, int command);
}
