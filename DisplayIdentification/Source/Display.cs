using System.Globalization;
using System.Text.Json.Serialization;

namespace DisplayIdentification;

// A monitor on an active display path.
internal sealed record Display {
    // A stable way to select the display: its model, plus its serial number when the monitor reports one. Unlike Windows display
    // numbers such as DISPLAY5, it doesn't change when the computer restarts or the monitor moves to another port.
    public required string Key { get; init; }

    // The EDID manufacturer and product code, such as AUSAA0F. Windows uses the same format in hardware IDs.
    public required string Model { get; init; }

    public required string SerialNumber { get; init; }
    public required string Name { get; init; }

    // The Windows display device name, such as \\.\DISPLAY5.
    public required string DeviceName { get; init; }

    // The monitor device path, which identifies the monitor on a specific port.
    public required string DevicePath { get; init; }

    public required string Connection { get; init; }
    public required int X { get; init; }
    public required int Y { get; init; }
    public required int Width { get; init; }
    public required int Height { get; init; }
    public required bool IsPrimary { get; init; }

    // The display path source, which owns the desktop area. Only valid until Windows restarts.
    [JsonIgnore]
    public NativeMethods.Luid SourceAdapterId { get; init; }

    [JsonIgnore]
    public uint SourceId { get; init; }

    [JsonIgnore]
    public string ShortDeviceName => this.DeviceName.StartsWith(@"\\.\", StringComparison.Ordinal) ? this.DeviceName[4..] : this.DeviceName;

    // Whether both displays show the same desktop area, which is the case for the same display and for mirrored displays.
    internal bool HasSameSource(Display other) {
        return this.SourceAdapterId.Matches(other.SourceAdapterId) && this.SourceId == other.SourceId;
    }

    // Returns the active displays from left to right, then top to bottom.
    internal static List<Display> GetAll() {
        DisplayConfiguration configuration = DisplayConfiguration.QueryActive();
        List<DisplayMonitor> monitors = DisplayMonitor.GetAll();
        List<Display> displays = [];
        foreach (NativeMethods.DisplayConfigPathInfo path in configuration.Paths) {
            if ((path.Flags & NativeMethods.DISPLAYCONFIG_PATH_ACTIVE) == 0) {
                continue;
            }

            string deviceName = DisplayConfiguration.GetSourceDeviceName(path.SourceInfo.AdapterId, path.SourceInfo.Id);
            (string friendlyName, string devicePath, uint outputTechnology) = DisplayConfiguration.GetTargetName(path.TargetInfo.AdapterId, path.TargetInfo.Id);
            Edid? edid = Edid.FromDevicePath(devicePath);
            string model = edid != null ? edid.ManufacturerId + edid.ProductCode.ToString("X4", CultureInfo.InvariantCulture) : ModelFromDevicePath(devicePath);
            string serialNumber = SerialNumberOf(edid);
            DisplayMonitor? monitor = monitors.Find(candidate => string.Equals(candidate.DeviceName, deviceName, StringComparison.OrdinalIgnoreCase));
            NativeMethods.Rect bounds = monitor?.Bounds ?? SourceBounds(path, configuration.Modes);
            displays.Add(new Display {
                Key = serialNumber.Length > 0 ? $"{model}-{serialNumber}" : model,
                Model = model,
                SerialNumber = serialNumber,
                Name = friendlyName.Length > 0 ? friendlyName : edid?.MonitorName ?? "",
                DeviceName = deviceName,
                DevicePath = devicePath,
                Connection = ConnectionName(outputTechnology),
                X = bounds.Left,
                Y = bounds.Top,
                Width = bounds.Width,
                Height = bounds.Height,
                IsPrimary = monitor?.IsPrimary ?? (bounds.Left == 0 && bounds.Top == 0),
                SourceAdapterId = path.SourceInfo.AdapterId,
                SourceId = path.SourceInfo.Id,
            });
        }

        displays.Sort((first, second) => first.X != second.X ? first.X.CompareTo(second.X) : first.Y.CompareTo(second.Y));
        return displays;
    }

    // Prefers the serial number text descriptor, which is what is printed on the monitor's label.
    private static string SerialNumberOf(Edid? edid) {
        return edid switch {
            { SerialText.Length: > 0 } => edid.SerialText,
            { SerialNumber: not 0 } => edid.SerialNumber.ToString(CultureInfo.InvariantCulture),
            _ => "",
        };
    }

    // Without an EDID, uses the hardware ID in the device path, such as AUSAA0F in \\?\DISPLAY#AUSAA0F#...
    private static string ModelFromDevicePath(string devicePath) {
        string[] parts = devicePath.Split('#');
        return parts.Length >= 2 && parts[1].Length > 0 ? parts[1] : "Unknown";
    }

    private static NativeMethods.Rect SourceBounds(NativeMethods.DisplayConfigPathInfo path, NativeMethods.DisplayConfigModeInfo[] modes) {
        uint index = path.SourceInfo.ModeInfoIndex;
        if (index >= modes.Length) {
            return default;
        }

        NativeMethods.DisplayConfigSourceMode mode = modes[index].SourceMode;
        return new NativeMethods.Rect {
            Left = mode.Position.X,
            Top = mode.Position.Y,
            Right = mode.Position.X + (int)mode.Width,
            Bottom = mode.Position.Y + (int)mode.Height,
        };
    }

    // DISPLAYCONFIG_VIDEO_OUTPUT_TECHNOLOGY values.
    private static string ConnectionName(uint outputTechnology) {
        return outputTechnology switch {
            0 => "VGA",
            4 => "DVI",
            5 => "HDMI",
            6 => "LVDS",
            10 => "DisplayPort",
            11 => "Embedded DisplayPort",
            12 => "UDI",
            13 => "Embedded UDI",
            15 => "Miracast",
            16 => "Indirect wired",
            17 => "Indirect virtual",
            18 => "DisplayPort over USB",
            0x80000000 => "Internal",
            _ => "Other",
        };
    }
}
