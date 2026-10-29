using Microsoft.Win32.SafeHandles;

namespace DisplayIdentification;

// Tracks the processes of a launched program: the process that was started, the processes it starts, and processes with given
// image names. Launchers often start the real program and exit, and steam:// URLs have no process of their own.
internal sealed unsafe class ProcessTree : IDisposable {
    private readonly List<TrackedProcess> processes = [];
    private readonly HashSet<string> imageNames;
    private readonly long startTime = DateTime.UtcNow.ToFileTimeUtc();
    private TrackedProcess? root;
    private bool sawNamedProcess;

    internal ProcessTree(IEnumerable<string> imageNames) {
        this.imageNames = new HashSet<string>(imageNames.Select(NormalizeImageName), StringComparer.OrdinalIgnoreCase);
    }

    public void Dispose() {
        foreach (TrackedProcess process in this.processes) {
            process.Handle?.Dispose();
        }
    }

    internal uint AddRoot(SafeProcessHandle handle) {
        _ = NativeMethods.GetProcessTimes(handle, out long creationTime, out _, out _, out _);
        this.root = new TrackedProcess {
            Id = NativeMethods.GetProcessId(handle),
            Handle = handle,
            CreationTime = creationTime,
        };
        this.processes.Add(this.root);
        return this.root.Id;
    }

    internal HashSet<uint> GetRunningProcessIds() {
        return [.. this.processes.Where(process => !process.HasExited).Select(process => process.Id)];
    }

    // Whether the program has exited. Until the startup period is over, a program that is expected to start a named process, or
    // that couldn't be tracked at all, counts as starting.
    internal bool HasExited(bool startupOver) {
        bool isRunning = this.processes.Exists(process => !process.HasExited);
        bool isStarting = !startupOver && (this.imageNames.Count > 0 ? !this.sawNamedProcess : this.root == null);
        return !isRunning && !isStarting;
    }

    internal int? GetExitCode() {
        return this.root is { HasExited: true, Handle: SafeProcessHandle handle } && NativeMethods.GetExitCodeProcess(handle, out uint exitCode)
            ? (int)exitCode
            : null;
    }

    internal void Refresh() {
        List<ProcessEntry> snapshot = TakeSnapshot();
        this.UpdateExited(snapshot);

        // Repeat, so that a child is found in the same refresh as its parent.
        bool added = true;
        while (added) {
            added = false;
            foreach (ProcessEntry entry in snapshot) {
                // A tracked process keeps its ID while its handle is open, so an ID can't belong to two tracked processes.
                if (this.processes.Exists(process => process.Id == entry.Id)) {
                    continue;
                }

                TrackedProcess? process = this.TryTrack(entry);
                if (process != null) {
                    this.processes.Add(process);
                    added = true;
                }
            }
        }
    }

    private void UpdateExited(List<ProcessEntry> snapshot) {
        long now = DateTime.UtcNow.ToFileTimeUtc();
        foreach (TrackedProcess process in this.processes) {
            if (process.HasExited) {
                continue;
            }

            if (process.Handle == null) {
                if (!snapshot.Exists(entry => entry.Id == process.Id)) {
                    process.HasExited = true;
                    process.ExitTime = now;
                }
            } else if (NativeMethods.WaitForSingleObject(process.Handle, 0) == NativeMethods.WAIT_OBJECT_0) {
                process.HasExited = true;
                process.ExitTime = NativeMethods.GetProcessTimes(process.Handle, out _, out long exitTime, out _, out _) ? exitTime : now;
            }
        }
    }

    private TrackedProcess? TryTrack(ProcessEntry entry) {
        bool hasTrackedName = this.imageNames.Contains(entry.ImageName);
        TrackedProcess? parent = this.processes.Find(process => process.Id == entry.ParentId);
        if (!hasTrackedName && parent == null) {
            return null;
        }

        // Protected processes, such as games with anti-cheat, may not allow even this access.
        SafeProcessHandle? handle = NativeMethods.OpenProcess(NativeMethods.PROCESS_QUERY_LIMITED_INFORMATION | NativeMethods.SYNCHRONIZE, false, entry.Id);
        long creationTime = 0;
        if (handle.IsInvalid) {
            handle.Dispose();
            handle = null;
        } else {
            _ = NativeMethods.GetProcessTimes(handle, out creationTime, out _, out _, out _);
        }

        // Process IDs are reused, so the parent ID only counts if the process started while the parent ran.
        bool isChild = parent != null && (creationTime == 0
            ? !parent.HasExited
            : creationTime >= parent.CreationTime && (!parent.HasExited || creationTime <= parent.ExitTime));
        bool isNamed = hasTrackedName && (creationTime == 0 || creationTime >= this.startTime);
        if (!isChild && !isNamed) {
            handle?.Dispose();
            return null;
        }

        this.sawNamedProcess |= isNamed;
        Log.Info($"Tracking process {entry.Id} ({entry.ImageName}).");
        return new TrackedProcess {
            Id = entry.Id,
            Handle = handle,
            CreationTime = creationTime,
        };
    }

    private static List<ProcessEntry> TakeSnapshot() {
        List<ProcessEntry> entries = [];
        using SafeFileHandle snapshot = NativeMethods.CreateToolhelp32Snapshot(NativeMethods.TH32CS_SNAPPROCESS, 0);
        if (snapshot.IsInvalid) {
            return entries;
        }

        NativeMethods.ProcessEntry32 entry = new() {
            Size = (uint)sizeof(NativeMethods.ProcessEntry32),
        };
        for (bool found = NativeMethods.Process32First(snapshot, ref entry); found; found = NativeMethods.Process32Next(snapshot, ref entry)) {
            entries.Add(new ProcessEntry(entry.ProcessId, entry.ParentProcessId, new string(entry.ExeFile)));
        }

        return entries;
    }

    // Accepts names with or without a folder and the .exe extension.
    private static string NormalizeImageName(string name) {
        string fileName = Path.GetFileName(name);
        return Path.HasExtension(fileName) ? fileName : fileName + ".exe";
    }

    private sealed record ProcessEntry(uint Id, uint ParentId, string ImageName);

    private sealed class TrackedProcess {
        public required uint Id { get; init; }
        public required SafeProcessHandle? Handle { get; init; }
        public required long CreationTime { get; init; }
        public bool HasExited { get; set; }
        public long ExitTime { get; set; }
    }
}
