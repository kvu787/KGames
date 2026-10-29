using System.Runtime.InteropServices;

namespace DisplayIdentification;

// Connecting and Configuring Displays (CCD) API and monitor API.
internal static unsafe partial class NativeMethods {
    internal const uint QDC_ONLY_ACTIVE_PATHS = 0x00000002;
    internal const uint SDC_USE_SUPPLIED_DISPLAY_CONFIG = 0x00000020;
    internal const uint SDC_VALIDATE = 0x00000040;
    internal const uint SDC_APPLY = 0x00000080;
    internal const uint SDC_SAVE_TO_DATABASE = 0x00000200;
    internal const uint SDC_ALLOW_CHANGES = 0x00000400;
    internal const uint DISPLAYCONFIG_PATH_ACTIVE = 0x00000001;
    internal const uint DISPLAYCONFIG_MODE_INFO_TYPE_SOURCE = 1;
    internal const uint DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME = 1;
    internal const uint DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME = 2;

    internal const uint MONITOR_DEFAULTTONEAREST = 0x00000002;
    internal const uint MONITORINFOF_PRIMARY = 0x00000001;

    internal struct DisplayConfigRational {
        public uint Numerator;
        public uint Denominator;
    }

    internal struct DisplayConfigPathSourceInfo {
        public Luid AdapterId;
        public uint Id;
        public uint ModeInfoIndex;
        public uint StatusFlags;
    }

    internal struct DisplayConfigPathTargetInfo {
        public Luid AdapterId;
        public uint Id;
        public uint ModeInfoIndex;
        public uint OutputTechnology;
        public uint Rotation;
        public uint Scaling;
        public DisplayConfigRational RefreshRate;
        public uint ScanLineOrdering;
        public int TargetAvailable;
        public uint StatusFlags;
    }

    internal struct DisplayConfigPathInfo {
        public DisplayConfigPathSourceInfo SourceInfo;
        public DisplayConfigPathTargetInfo TargetInfo;
        public uint Flags;
    }

    internal struct DisplayConfigSourceMode {
        public uint Width;
        public uint Height;
        public uint PixelFormat;
        public PointL Position;
    }

    // DISPLAYCONFIG_MODE_INFO. Only the source mode member of its union is declared, but the struct keeps the full native size so
    // that target and desktop image modes pass through unchanged.
    [StructLayout(LayoutKind.Explicit, Size = 64)]
    internal struct DisplayConfigModeInfo {
        [FieldOffset(0)]
        public uint InfoType;

        [FieldOffset(4)]
        public uint Id;

        [FieldOffset(8)]
        public Luid AdapterId;

        [FieldOffset(16)]
        public DisplayConfigSourceMode SourceMode;
    }

    internal struct DisplayConfigDeviceInfoHeader {
        public uint Type;
        public uint Size;
        public Luid AdapterId;
        public uint Id;
    }

    internal struct DisplayConfigSourceDeviceName {
        public DisplayConfigDeviceInfoHeader Header;
        public fixed char ViewGdiDeviceName[32];
    }

    internal struct DisplayConfigTargetDeviceName {
        public DisplayConfigDeviceInfoHeader Header;
        public uint Flags;
        public uint OutputTechnology;
        public ushort EdidManufactureId;
        public ushort EdidProductCodeId;
        public uint ConnectorInstance;
        public fixed char MonitorFriendlyDeviceName[64];
        public fixed char MonitorDevicePath[128];
    }

    internal struct MonitorInfoEx {
        public uint Size;
        public Rect Monitor;
        public Rect Work;
        public uint Flags;
        public fixed char Device[32];
    }

    [LibraryImport("user32.dll")]
    internal static partial int GetDisplayConfigBufferSizes(uint flags, out uint numPathArrayElements, out uint numModeInfoArrayElements);

    [LibraryImport("user32.dll")]
    internal static partial int QueryDisplayConfig(uint flags, ref uint numPathArrayElements, DisplayConfigPathInfo* pathArray, ref uint numModeInfoArrayElements, DisplayConfigModeInfo* modeInfoArray, nint currentTopologyId);

    [LibraryImport("user32.dll")]
    internal static partial int SetDisplayConfig(uint numPathArrayElements, DisplayConfigPathInfo* pathArray, uint numModeInfoArrayElements, DisplayConfigModeInfo* modeInfoArray, uint flags);

    [LibraryImport("user32.dll", EntryPoint = "DisplayConfigGetDeviceInfo")]
    internal static partial int DisplayConfigGetDeviceInfo(ref DisplayConfigSourceDeviceName requestPacket);

    [LibraryImport("user32.dll", EntryPoint = "DisplayConfigGetDeviceInfo")]
    internal static partial int DisplayConfigGetDeviceInfo(ref DisplayConfigTargetDeviceName requestPacket);

    [LibraryImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    internal static partial bool EnumDisplayMonitors(nint hdc, nint clip, delegate* unmanaged<nint, nint, Rect*, nint, int> callback, nint data);

    [LibraryImport("user32.dll", EntryPoint = "GetMonitorInfoW")]
    [return: MarshalAs(UnmanagedType.Bool)]
    internal static partial bool GetMonitorInfo(nint monitor, ref MonitorInfoEx info);

    [LibraryImport("user32.dll")]
    internal static partial nint MonitorFromWindow(nint window, uint flags);
}
