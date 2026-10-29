using System.Runtime.InteropServices;

namespace DisplayIdentification;

// Moves the windows of a program to a display as they appear.
internal sealed unsafe class WindowMover {
    // Some programs move their window again right after it appears, for example to a saved position. Moving a window a few times
    // handles that without fighting the user forever.
    private const int MaximumMovesPerWindow = 3;
    private const int MinimumWindowSize = 32;

    private readonly Dictionary<nint, int> moveCounts = [];

    internal void MoveWindows(string targetDeviceName, HashSet<uint> processIds) {
        if (processIds.Count == 0) {
            return;
        }

        DisplayMonitor? target = DisplayMonitor.Find(targetDeviceName);
        if (target == null) {
            return;
        }

        foreach (nint window in GetTopLevelWindows()) {
            _ = NativeMethods.GetWindowThreadProcessId(window, out uint processId);
            if (!processIds.Contains(processId) || !IsProgramWindow(window)) {
                continue;
            }

            DisplayMonitor? current = DisplayMonitor.FromWindow(window);
            int moves = this.moveCounts.GetValueOrDefault(window);
            if (current == null || current.DeviceName == target.DeviceName || moves >= MaximumMovesPerWindow) {
                continue;
            }

            this.moveCounts[window] = moves + 1;
            string description = $"window \"{GetTitle(window)}\" of process {processId}";
            if (Move(window, current, target)) {
                Log.Info($"Moved {description} from {current.DeviceName} to {target.DeviceName}.");
            } else {
                int error = Marshal.GetLastPInvokeError();
                string hint = error == NativeMethods.ERROR_ACCESS_DENIED ? " Only programs that run as administrator can move the windows of programs that run as administrator." : "";
                Log.Warning($"Couldn't move {description}: {Marshal.GetPInvokeErrorMessage(error)}{hint}");
            }
        }
    }

    // Top-level, visible, unowned windows that aren't tool windows: the windows that show in the taskbar.
    private static bool IsProgramWindow(nint window) {
        if (!NativeMethods.IsWindowVisible(window) || NativeMethods.IsIconic(window) || NativeMethods.GetWindow(window, NativeMethods.GW_OWNER) != 0) {
            return false;
        }

        if ((NativeMethods.GetWindowLongPtr(window, NativeMethods.GWL_EXSTYLE) & NativeMethods.WS_EX_TOOLWINDOW) != 0) {
            return false;
        }

        // Cloaked windows exist but aren't shown, such as suspended UWP windows.
        bool isCloaked = NativeMethods.DwmGetWindowAttribute(window, NativeMethods.DWMWA_CLOAKED, out int cloaked, sizeof(int)) == 0 && cloaked != 0;
        return !isCloaked && NativeMethods.GetWindowRect(window, out NativeMethods.Rect rect) && rect.Width >= MinimumWindowSize && rect.Height >= MinimumWindowSize;
    }

    private static bool Move(nint window, DisplayMonitor current, DisplayMonitor target) {
        if (NativeMethods.IsZoomed(window)) {
            // Restore the window before moving it, so that it is maximized on the target display and also restores there.
            _ = NativeMethods.ShowWindow(window, NativeMethods.SW_SHOWNOACTIVATE);
            bool moved = NativeMethods.GetWindowRect(window, out NativeMethods.Rect restored) && MoveKeepingOffset(window, restored, DisplayMonitor.FromWindow(window) ?? current, target);
            _ = NativeMethods.ShowWindow(window, NativeMethods.SW_MAXIMIZE);
            return moved;
        }

        if (!NativeMethods.GetWindowRect(window, out NativeMethods.Rect rect)) {
            return false;
        }

        // A window that covers its whole display is a borderless fullscreen window, so make it cover the target display.
        NativeMethods.Rect bounds = current.Bounds;
        if (rect.Left <= bounds.Left && rect.Top <= bounds.Top && rect.Right >= bounds.Right && rect.Bottom >= bounds.Bottom) {
            NativeMethods.Rect targetBounds = target.Bounds;
            return NativeMethods.SetWindowPos(window, 0, targetBounds.Left, targetBounds.Top, targetBounds.Width, targetBounds.Height, NativeMethods.SWP_NOZORDER | NativeMethods.SWP_NOACTIVATE);
        }

        return MoveKeepingOffset(window, rect, current, target);
    }

    // Keeps the window's position relative to the work area, so that it lands in the same place on the target display. The window
    // is kept inside the target work area.
    private static bool MoveKeepingOffset(nint window, NativeMethods.Rect rect, DisplayMonitor from, DisplayMonitor to) {
        NativeMethods.Rect area = to.WorkArea;
        int width = Math.Min(rect.Width, area.Width);
        int height = Math.Min(rect.Height, area.Height);
        int x = Math.Clamp(area.Left + rect.Left - from.WorkArea.Left, area.Left, area.Right - width);
        int y = Math.Clamp(area.Top + rect.Top - from.WorkArea.Top, area.Top, area.Bottom - height);
        return NativeMethods.SetWindowPos(window, 0, x, y, width, height, NativeMethods.SWP_NOZORDER | NativeMethods.SWP_NOACTIVATE);
    }

    private static string GetTitle(nint window) {
        const int MaximumLength = 256;
        char* title = stackalloc char[MaximumLength];
        int length = NativeMethods.GetWindowText(window, title, MaximumLength);
        return new string(title, 0, length);
    }

    private static List<nint> GetTopLevelWindows() {
        List<nint> windows = [];
        GCHandle windowsHandle = GCHandle.Alloc(windows);
        try {
            _ = NativeMethods.EnumWindows(&CollectWindow, GCHandle.ToIntPtr(windowsHandle));
        } finally {
            windowsHandle.Free();
        }

        return windows;
    }

    [UnmanagedCallersOnly]
    private static int CollectWindow(nint window, nint parameter) {
        ((List<nint>)GCHandle.FromIntPtr(parameter).Target!).Add(window);
        return 1;
    }
}
