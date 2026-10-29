namespace DisplayIdentification;

// The active display paths and modes from the Connecting and Configuring Displays (CCD) API.
internal sealed unsafe class DisplayConfiguration {
    internal required NativeMethods.DisplayConfigPathInfo[] Paths { get; init; }
    internal required NativeMethods.DisplayConfigModeInfo[] Modes { get; init; }

    internal static DisplayConfiguration QueryActive() {
        while (true) {
            int result = NativeMethods.GetDisplayConfigBufferSizes(NativeMethods.QDC_ONLY_ACTIVE_PATHS, out uint pathCount, out uint modeCount);
            NativeMethods.ThrowIfFailed("GetDisplayConfigBufferSizes", result);
            NativeMethods.DisplayConfigPathInfo[] paths = new NativeMethods.DisplayConfigPathInfo[pathCount];
            NativeMethods.DisplayConfigModeInfo[] modes = new NativeMethods.DisplayConfigModeInfo[modeCount];
            fixed (NativeMethods.DisplayConfigPathInfo* pathPointer = paths) {
                fixed (NativeMethods.DisplayConfigModeInfo* modePointer = modes) {
                    result = NativeMethods.QueryDisplayConfig(NativeMethods.QDC_ONLY_ACTIVE_PATHS, ref pathCount, pathPointer, ref modeCount, modePointer, 0);
                }
            }

            // The configuration can change between the two calls, which makes the buffers too small.
            if (result == NativeMethods.ERROR_INSUFFICIENT_BUFFER) {
                continue;
            }

            NativeMethods.ThrowIfFailed("QueryDisplayConfig", result);
            return new DisplayConfiguration {
                Paths = paths[..(int)pathCount],
                Modes = modes[..(int)modeCount],
            };
        }
    }

    internal static string GetSourceDeviceName(NativeMethods.Luid adapterId, uint sourceId) {
        NativeMethods.DisplayConfigSourceDeviceName request = new() {
            Header = new() {
                Type = NativeMethods.DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME,
                Size = (uint)sizeof(NativeMethods.DisplayConfigSourceDeviceName),
                AdapterId = adapterId,
                Id = sourceId,
            },
        };
        NativeMethods.ThrowIfFailed("DisplayConfigGetDeviceInfo", NativeMethods.DisplayConfigGetDeviceInfo(ref request));
        return new string(request.ViewGdiDeviceName);
    }

    internal static (string FriendlyName, string DevicePath, uint OutputTechnology) GetTargetName(NativeMethods.Luid adapterId, uint targetId) {
        NativeMethods.DisplayConfigTargetDeviceName request = new() {
            Header = new() {
                Type = NativeMethods.DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME,
                Size = (uint)sizeof(NativeMethods.DisplayConfigTargetDeviceName),
                AdapterId = adapterId,
                Id = targetId,
            },
        };
        NativeMethods.ThrowIfFailed("DisplayConfigGetDeviceInfo", NativeMethods.DisplayConfigGetDeviceInfo(ref request));
        return (new string(request.MonitorFriendlyDeviceName), new string(request.MonitorDevicePath), request.OutputTechnology);
    }
}
