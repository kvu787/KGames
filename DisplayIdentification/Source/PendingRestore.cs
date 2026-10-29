using System.ComponentModel;
using System.Diagnostics;
using System.Text.Json;

namespace DisplayIdentification;

// Remembers a temporary primary display change on disk while a launch runs, so that it can be undone if the launch is interrupted.
internal sealed record PendingRestore {
    private static readonly JsonSerializerOptions JsonOptions = new() {
        WriteIndented = true,
    };

    public required string OriginalKey { get; init; }
    public required string OriginalDevicePath { get; init; }
    public required string TemporaryDevicePath { get; init; }
    public required int ProcessId { get; init; }
    public required DateTime ProcessStartTime { get; init; }

    private static string FilePath => Path.Combine(ApplicationData.DirectoryPath, "PendingRestore.json");

    internal static PendingRestore Create(Display original, Display temporary) {
        using Process process = Process.GetCurrentProcess();
        return new PendingRestore {
            OriginalKey = original.Key,
            OriginalDevicePath = original.DevicePath,
            TemporaryDevicePath = temporary.DevicePath,
            ProcessId = process.Id,
            ProcessStartTime = process.StartTime.ToUniversalTime(),
        };
    }

    internal static PendingRestore? Load() {
        try {
            return JsonSerializer.Deserialize<PendingRestore>(File.ReadAllText(FilePath));
        } catch (Exception exception) when (exception is FileNotFoundException or DirectoryNotFoundException or JsonException) {
            return null;
        }
    }

    internal static void Delete() {
        File.Delete(FilePath);
    }

    // Undoes the change of an interrupted launch, unless the primary display was changed again since then.
    internal static void RecoverInterruptedLaunch() {
        PendingRestore? pending = Load();
        if (pending == null || pending.IsOwnerRunning()) {
            return;
        }

        List<Display> displays = Display.GetAll();
        Display? primary = displays.Find(display => display.IsPrimary);
        Display? original = pending.FindOriginal(displays);
        if (primary != null && original != null && string.Equals(primary.DevicePath, pending.TemporaryDevicePath, StringComparison.OrdinalIgnoreCase)) {
            Log.Info($"Making {original.Key} the primary display again, since an earlier launch was interrupted.");
            PrimaryDisplay.Set(original, persist: false);
        }

        Delete();
    }

    internal void Save() {
        _ = Directory.CreateDirectory(ApplicationData.DirectoryPath);
        File.WriteAllText(FilePath, JsonSerializer.Serialize(this, JsonOptions));
    }

    internal Display? FindOriginal(IReadOnlyList<Display> displays) {
        return displays.FirstOrDefault(display => string.Equals(display.DevicePath, this.OriginalDevicePath, StringComparison.OrdinalIgnoreCase))
            ?? displays.FirstOrDefault(display => string.Equals(display.Key, this.OriginalKey, StringComparison.OrdinalIgnoreCase));
    }

    internal bool IsOwnerRunning() {
        try {
            using Process process = Process.GetProcessById(this.ProcessId);
            return process.StartTime.ToUniversalTime() == this.ProcessStartTime;
        } catch (Exception exception) when (exception is ArgumentException or InvalidOperationException) {
            // The process has exited.
            return false;
        } catch (Win32Exception) {
            // The process can't be inspected, so it isn't certain that it is a different process.
            return true;
        }
    }
}
