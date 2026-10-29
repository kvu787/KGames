using System.Runtime.InteropServices;

namespace DisplayIdentification;

// A display's area of the desktop, as the window manager sees it (an HMONITOR).
internal sealed unsafe record DisplayMonitor(nint Handle, string DeviceName, NativeMethods.Rect Bounds, NativeMethods.Rect WorkArea, bool IsPrimary) {
    internal static List<DisplayMonitor> GetAll() {
        List<nint> handles = [];
        GCHandle handlesHandle = GCHandle.Alloc(handles);
        try {
            _ = NativeMethods.EnumDisplayMonitors(0, 0, &CollectMonitor, GCHandle.ToIntPtr(handlesHandle));
        } finally {
            handlesHandle.Free();
        }

        List<DisplayMonitor> monitors = [];
        foreach (nint handle in handles) {
            DisplayMonitor? monitor = FromHandle(handle);
            if (monitor != null) {
                monitors.Add(monitor);
            }
        }

        return monitors;
    }

    internal static DisplayMonitor? Find(string deviceName) {
        return GetAll().Find(monitor => string.Equals(monitor.DeviceName, deviceName, StringComparison.OrdinalIgnoreCase));
    }

    internal static DisplayMonitor? FromWindow(nint window) {
        return FromHandle(NativeMethods.MonitorFromWindow(window, NativeMethods.MONITOR_DEFAULTTONEAREST));
    }

    private static DisplayMonitor? FromHandle(nint handle) {
        NativeMethods.MonitorInfoEx info = new() {
            Size = (uint)sizeof(NativeMethods.MonitorInfoEx),
        };
        return NativeMethods.GetMonitorInfo(handle, ref info)
            ? new DisplayMonitor(handle, new string(info.Device), info.Monitor, info.Work, (info.Flags & NativeMethods.MONITORINFOF_PRIMARY) != 0)
            : null;
    }

    [UnmanagedCallersOnly]
    private static int CollectMonitor(nint monitor, nint hdc, NativeMethods.Rect* rect, nint data) {
        ((List<nint>)GCHandle.FromIntPtr(data).Target!).Add(monitor);
        return 1;
    }
}
