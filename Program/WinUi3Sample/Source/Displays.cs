using System.Runtime.InteropServices;
using System.Text;

namespace WinUi3Sample;

// Finds a display by the identifier that Library/Displays gives it, like CVVNY2, so that --display
// names the same displays as it does for the other KGames programs. An identifier is the top 30 bits
// of an FNV-1a hash of the monitor's device path, in Crockford's Base32. IdentifyDisplays' README
// explains them.
public static unsafe partial class Displays {
    public sealed record Display(string Identifier, string DevicePath, Rect Area, Rect WorkArea, bool Primary);

    [StructLayout(LayoutKind.Sequential)]
    public struct Rect {
        public int Left, Top, Right, Bottom;
        public readonly int Width => this.Right - this.Left;
        public readonly int Height => this.Bottom - this.Top;
    }

    // Crockford's Base32, which has no I, L, O or U.
    const string Alphabet = "0123456789ABCDEFGHJKMNPQRSTVWXYZ";
    const int IdentifierLength = 6;

    // The display with this identifier, ignoring case, or Primary for the primary display.
    public static Display Find(string identifier) {
        var displays = Enumerate();
        identifier = identifier.Trim();
        var found = identifier.Equals("Primary", StringComparison.OrdinalIgnoreCase)
            ? displays.FirstOrDefault(display => display.Primary) ?? displays.FirstOrDefault()
            : displays.FirstOrDefault(display => display.Identifier.Equals(identifier, StringComparison.OrdinalIgnoreCase));
        return found ?? throw new StartupException(
            $"Display {identifier} isn't connected. The connected displays are {string.Join(", ", displays.Select(display => display.Identifier))}");
    }

    public static List<Display> Enumerate() {
        // Without QueryDisplayConfig's monitors, identifiers come from GDI device names.
        Dictionary<string, List<string>> paths;
        try {
            paths = DevicePaths();
        } catch (InvalidOperationException) {
            paths = [];
        }
        var displays = new List<Display>();
        foreach (var monitor in Monitors()) {
            var info = new MONITORINFOEXW { cbSize = (uint)sizeof(MONITORINFOEXW) };
            if (!GetMonitorInfoW(monitor, &info)) {
                continue;
            }
            var gdiName = new string(info.szDevice);
            var primary = (info.dwFlags & MONITORINFOF_PRIMARY) != 0;
            // A monitor that clones another shows the same part of the desktop, so both are on
            // one HMONITOR.
            var devicePaths = paths.TryGetValue(gdiName, out var found) ? found : [gdiName];
            foreach (var path in devicePaths) {
                displays.Add(new Display(Identifier(path), path, info.rcMonitor, info.rcWork, primary));
            }
        }
        return displays;
    }

    // FNV-1a over the lowercase device path, as Library/Displays makes identifiers.
    static string Identifier(string devicePath) {
        var hash = 0xCBF2_9CE4_8422_2325UL;
        foreach (var character in Encoding.UTF8.GetBytes(devicePath)) {
            var lower = character is >= (byte)'A' and <= (byte)'Z' ? (byte)(character + 32) : character;
            hash ^= lower;
            hash *= 0x0100_0000_01B3UL;
        }
        var identifier = new StringBuilder(IdentifierLength);
        for (var index = 1; index <= IdentifierLength; index++) {
            identifier.Append(Alphabet[(int)(hash >> (64 - 5 * index)) & 31]);
        }
        return identifier.ToString();
    }

    static List<nint> Monitors() {
        var monitors = new List<nint>();
        var handle = GCHandle.Alloc(monitors);
        try {
            _ = EnumDisplayMonitors(0, null, &AddMonitor, GCHandle.ToIntPtr(handle));
        } finally {
            handle.Free();
        }
        return monitors;
    }

    [UnmanagedCallersOnly]
    static int AddMonitor(nint monitor, nint dc, Rect* area, nint monitors) {
        ((List<nint>)GCHandle.FromIntPtr(monitors).Target!).Add(monitor);
        return 1;
    }

    // The device path of each monitor, by the GDI device name of the part of the desktop that it
    // shows, like \\.\DISPLAY1, which GetMonitorInfoW also reports.
    static Dictionary<string, List<string>> DevicePaths() {
        DISPLAYCONFIG_PATH_INFO[] paths;
        while (true) {
            Check(GetDisplayConfigBufferSizes(QDC_ONLY_ACTIVE_PATHS, out var pathCount, out var modeCount));
            paths = new DISPLAYCONFIG_PATH_INFO[pathCount];
            // Modes aren't read, so they're just bytes of the right size.
            var modes = new byte[modeCount * DisplayConfigModeInfoSize];
            int error;
            fixed (DISPLAYCONFIG_PATH_INFO* pathPointer = paths)
            fixed (byte* modePointer = modes) {
                error = QueryDisplayConfig(QDC_ONLY_ACTIVE_PATHS, ref pathCount, pathPointer, ref modeCount, modePointer, 0);
            }
            // A display came on between the two calls, so ask again.
            if (error == ERROR_INSUFFICIENT_BUFFER) {
                continue;
            }
            Check(error);
            Array.Resize(ref paths, (int)pathCount);
            break;
        }

        var devicePaths = new Dictionary<string, List<string>>(StringComparer.OrdinalIgnoreCase);
        foreach (var path in paths) {
            var source = new DISPLAYCONFIG_SOURCE_DEVICE_NAME {
                header = new DISPLAYCONFIG_DEVICE_INFO_HEADER {
                    type = DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME,
                    size = (uint)sizeof(DISPLAYCONFIG_SOURCE_DEVICE_NAME),
                    adapterId = path.sourceInfo.adapterId,
                    id = path.sourceInfo.id,
                },
            };
            if (DisplayConfigGetDeviceInfo(&source.header) != 0) {
                continue;
            }
            var target = new DISPLAYCONFIG_TARGET_DEVICE_NAME {
                header = new DISPLAYCONFIG_DEVICE_INFO_HEADER {
                    type = DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME,
                    size = (uint)sizeof(DISPLAYCONFIG_TARGET_DEVICE_NAME),
                    adapterId = path.targetInfo.adapterId,
                    id = path.targetInfo.id,
                },
            };
            if (DisplayConfigGetDeviceInfo(&target.header) != 0) {
                continue;
            }
            var gdiName = new string(source.viewGdiDeviceName);
            var devicePath = new string(target.monitorDevicePath);
            if (!devicePaths.TryGetValue(gdiName, out var list)) {
                devicePaths[gdiName] = list = [];
            }
            list.Add(devicePath.Length > 0 ? devicePath : gdiName);
        }
        return devicePaths;
    }

    static void Check(int error) {
        if (error != 0) {
            throw new InvalidOperationException($"QueryDisplayConfig failed with error {error}");
        }
    }

    const uint QDC_ONLY_ACTIVE_PATHS = 2;
    const int ERROR_INSUFFICIENT_BUFFER = 122;
    const uint DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME = 1;
    const uint DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME = 2;
    const uint MONITORINFOF_PRIMARY = 1;
    const int DisplayConfigModeInfoSize = 64;

    [StructLayout(LayoutKind.Sequential)]
    struct LUID {
        public uint LowPart;
        public int HighPart;
    }

    [StructLayout(LayoutKind.Sequential)]
    struct DISPLAYCONFIG_PATH_SOURCE_INFO {
        public LUID adapterId;
        public uint id, modeInfoIdx, statusFlags;
    }

    [StructLayout(LayoutKind.Sequential)]
    struct DISPLAYCONFIG_PATH_TARGET_INFO {
        public LUID adapterId;
        public uint id, modeInfoIdx, outputTechnology, rotation, scaling;
        public uint refreshRateNumerator, refreshRateDenominator;
        public uint scanLineOrdering;
        public int targetAvailable;
        public uint statusFlags;
    }

    [StructLayout(LayoutKind.Sequential)]
    struct DISPLAYCONFIG_PATH_INFO {
        public DISPLAYCONFIG_PATH_SOURCE_INFO sourceInfo;
        public DISPLAYCONFIG_PATH_TARGET_INFO targetInfo;
        public uint flags;
    }

    [StructLayout(LayoutKind.Sequential)]
    struct DISPLAYCONFIG_DEVICE_INFO_HEADER {
        public uint type, size;
        public LUID adapterId;
        public uint id;
    }

    [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
    struct DISPLAYCONFIG_SOURCE_DEVICE_NAME {
        public DISPLAYCONFIG_DEVICE_INFO_HEADER header;
        public fixed char viewGdiDeviceName[32];
    }

    [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
    struct DISPLAYCONFIG_TARGET_DEVICE_NAME {
        public DISPLAYCONFIG_DEVICE_INFO_HEADER header;
        public uint flags, outputTechnology;
        public ushort edidManufactureId, edidProductCodeId;
        public uint connectorInstance;
        public fixed char monitorFriendlyDeviceName[64];
        public fixed char monitorDevicePath[128];
    }

    [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
    struct MONITORINFOEXW {
        public uint cbSize;
        public Rect rcMonitor, rcWork;
        public uint dwFlags;
        public fixed char szDevice[32];
    }

    [LibraryImport("user32.dll")]
    private static partial int GetDisplayConfigBufferSizes(uint flags, out uint pathCount, out uint modeCount);
    [LibraryImport("user32.dll")]
    private static partial int QueryDisplayConfig(uint flags, ref uint pathCount, DISPLAYCONFIG_PATH_INFO* paths, ref uint modeCount, byte* modes, nint topology);
    [LibraryImport("user32.dll")]
    private static partial int DisplayConfigGetDeviceInfo(DISPLAYCONFIG_DEVICE_INFO_HEADER* header);
    [LibraryImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static partial bool EnumDisplayMonitors(nint dc, Rect* clip, delegate* unmanaged<nint, nint, Rect*, nint, int> callback, nint data);
    [LibraryImport("user32.dll", EntryPoint = "GetMonitorInfoW")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static partial bool GetMonitorInfoW(nint monitor, MONITORINFOEXW* info);
}
