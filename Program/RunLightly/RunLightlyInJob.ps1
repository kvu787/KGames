#Requires -Version 7

# What RunLightly.ps1 runs, in a PowerShell process of its own, which puts
# itself in a job object whose limits hold the command and every process that
# it starts, which can't escape them, and then runs the command:
#
# - Only on the CPU's efficiency cores, which leaves the performance cores
#   that a game's threads run on to the game. On a CPU whose cores are all
#   alike, on all of its cores but a quarter.
# - At idle priority, so that anything else that wants a core gets it first.
# - With at most a share of the whole CPU's time, 33% unless
#   KGAMES_LIGHT_CPU_PERCENT says otherwise, which also keeps a laptop's
#   power for the game's cores.
# - Killed when this script ends, however that happens, like by a coding
#   agent's time limit, so that no build keeps running unseen. A program
#   that RunInBackground keeps open holds a handle to the job, whose name
#   KGAMES_LIGHT_JOB gives, so it outlives this script with the limits.
#
# And, since jobs can't hold these, it sets them itself:
#
# - Low I/O priority and low memory priority on this process, which the
#   processes it starts inherit, so that the game's reads and writes go
#   first, and the build's pages go first when memory is short.
# - Efficiency mode (EcoQoS) on every process in the job, which a thread
#   here sets a few times a second, as children don't inherit it, so that the
#   efficiency cores run them at efficient clocks and the chip's power budget
#   stays with the game's cores.
# - KGAMES_GPU_PRIORITY=Idle in the environment, which Library/Graphics
#   reads: every process that draws, like a test or a test run of a program,
#   puts its GPU work after other processes', like the game's frames.
#
# The command's process is made here, suspended, put in the job, and only then
# let run: a process that PowerShell starts the usual way doesn't always end
# up in its job (cmd and the rest didn't here, though a child
# PowerShell did), while the processes that a native process in the job
# starts do. Microsoft's documentation says that a job's processes start
# theirs in it unless the job allows breakaway, which this one doesn't, but
# on this computer, what PowerShell or another .NET process in the job starts
# is in no job at all, even when it calls CreateProcessW itself with no flags
# (IsProcessInJob said so on 2026-11-09, inside and outside a coding agent's
# sandbox). So for the ones that slip out, the thread that sets efficiency
# mode also puts every process whose parent is in the job into it, within a
# quarter second.
#
# KGAMES_LIGHT_JOBS in the environment says how many cores the cap is worth,
# like 8 of 24 at 33%. Program\Build\Build.ps1 has MSBuild build that many
# projects at once, and each project's compiler run that many copies of
# itself, instead of one for each core, so that a build's work is steady
# rather than bursts that the cap stops, and the other efficiency cores stay
# free. .NET's and MSBuild's build servers would run outside the job, so
# they're turned off.
#
# It exits with the command's exit code.

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$percent = 33
if ($env:KGAMES_LIGHT_CPU_PERCENT) {
    $percent = [int]$env:KGAMES_LIGHT_CPU_PERCENT
}

Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading;

public static class Lightly {
    [StructLayout(LayoutKind.Sequential)]
    struct ExtendedLimits {
        public long PerProcessUserTimeLimit, PerJobUserTimeLimit;
        public uint LimitFlags;
        public UIntPtr MinimumWorkingSetSize, MaximumWorkingSetSize;
        public uint ActiveProcessLimit;
        public UIntPtr Affinity;
        public uint PriorityClass, SchedulingClass;
        public ulong ReadOperationCount, WriteOperationCount, OtherOperationCount, ReadTransferCount, WriteTransferCount, OtherTransferCount;
        public UIntPtr ProcessMemoryLimit, JobMemoryLimit, PeakProcessMemoryUsed, PeakJobMemoryUsed;
    }

    [StructLayout(LayoutKind.Sequential)]
    struct CpuRateControl {
        public uint ControlFlags, CpuRate;
    }

    [StructLayout(LayoutKind.Sequential)]
    struct MemoryPriority {
        public uint Priority;
    }

    [StructLayout(LayoutKind.Sequential)]
    struct PowerThrottling {
        public uint Version, ControlMask, StateMask;
    }

    [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
    struct StartupInfo {
        public int cb;
        public IntPtr lpReserved, lpDesktop, lpTitle;
        public int dwX, dwY, dwXSize, dwYSize, dwXCountChars, dwYCountChars, dwFillAttribute, dwFlags;
        public short wShowWindow, cbReserved2;
        public IntPtr lpReserved2, hStdInput, hStdOutput, hStdError;
    }

    [StructLayout(LayoutKind.Sequential)]
    struct ProcessInformation {
        public IntPtr hProcess, hThread;
        public int dwProcessId, dwThreadId;
    }

    [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
    struct ProcessEntry {
        public uint dwSize, cntUsage, th32ProcessID;
        public UIntPtr th32DefaultHeapID;
        public uint th32ModuleID, cntThreads, th32ParentProcessID;
        public int pcPriClassBase;
        public uint dwFlags;
        [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 260)]
        public string szExeFile;
    }

    [DllImport("kernel32.dll", SetLastError = true)]
    static extern bool GetSystemCpuSetInformation(IntPtr information, uint length, out uint returned, IntPtr process, uint flags);
    [DllImport("kernel32.dll", SetLastError = true, CharSet = CharSet.Unicode)]
    static extern IntPtr CreateJobObjectW(IntPtr attributes, string name);
    [DllImport("kernel32.dll", SetLastError = true)]
    static extern bool SetInformationJobObject(IntPtr job, int informationClass, ref ExtendedLimits information, int length);
    [DllImport("kernel32.dll", SetLastError = true)]
    static extern bool SetInformationJobObject(IntPtr job, int informationClass, ref CpuRateControl information, int length);
    [DllImport("kernel32.dll", SetLastError = true)]
    static extern bool QueryInformationJobObject(IntPtr job, int informationClass, IntPtr information, int length, out int returned);
    [DllImport("kernel32.dll", SetLastError = true)]
    static extern bool AssignProcessToJobObject(IntPtr job, IntPtr process);
    [DllImport("kernel32.dll", SetLastError = true)]
    static extern bool SetProcessInformation(IntPtr process, int informationClass, ref MemoryPriority information, int length);
    [DllImport("kernel32.dll", SetLastError = true)]
    static extern bool SetProcessInformation(IntPtr process, int informationClass, ref PowerThrottling information, int length);
    // Documented by Process Explorer's use, not by Microsoft: the only way to
    // lower a process's I/O priority to Low. Background mode,
    // PROCESS_MODE_BACKGROUND_BEGIN, is documented, but sets Very Low, which
    // the system can starve for as long as anything else does I/O.
    [DllImport("ntdll.dll")]
    static extern int NtSetInformationProcess(IntPtr process, int informationClass, ref int information, int length);
    [DllImport("kernel32.dll", SetLastError = true)]
    static extern IntPtr OpenProcess(uint access, bool inherit, uint processId);
    [DllImport("kernel32.dll", SetLastError = true)]
    static extern bool IsProcessInJob(IntPtr process, IntPtr job, out bool inJob);
    [DllImport("kernel32.dll", SetLastError = true)]
    static extern IntPtr CreateToolhelp32Snapshot(uint flags, uint processId);
    [DllImport("kernel32.dll", SetLastError = true, CharSet = CharSet.Unicode)]
    static extern bool Process32FirstW(IntPtr snapshot, ref ProcessEntry entry);
    [DllImport("kernel32.dll", SetLastError = true, CharSet = CharSet.Unicode)]
    static extern bool Process32NextW(IntPtr snapshot, ref ProcessEntry entry);
    // The command line may be written to, so it isn't a string.
    [DllImport("kernel32.dll", SetLastError = true, CharSet = CharSet.Unicode)]
    static extern bool CreateProcessW(string application, [In, Out] char[] commandLine, IntPtr processAttributes, IntPtr threadAttributes, bool inheritHandles, uint flags, IntPtr environment, string directory, ref StartupInfo startupInfo, out ProcessInformation information);
    [DllImport("kernel32.dll", SetLastError = true)]
    static extern uint ResumeThread(IntPtr thread);
    [DllImport("kernel32.dll", SetLastError = true)]
    static extern uint WaitForSingleObject(IntPtr handle, uint milliseconds);
    [DllImport("kernel32.dll", SetLastError = true)]
    static extern bool GetExitCodeProcess(IntPtr process, out int exitCode);
    [DllImport("kernel32.dll")]
    static extern bool CloseHandle(IntPtr handle);
    [DllImport("kernel32.dll")]
    static extern IntPtr GetCurrentProcess();

    const int JobObjectBasicProcessIdList = 3;
    const int ERROR_MORE_DATA = 234;
    // SYSTEM_CPU_SET_INFORMATION's only type.
    const int CpuSetInformation = 0;
    const int JobObjectExtendedLimitInformation = 9;
    const int JobObjectCpuRateControlInformation = 15;
    const uint JOB_OBJECT_LIMIT_AFFINITY = 0x10;
    const uint JOB_OBJECT_LIMIT_PRIORITY_CLASS = 0x20;
    const uint JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE = 0x2000;
    const uint IDLE_PRIORITY_CLASS = 0x40;
    const uint JOB_OBJECT_CPU_RATE_CONTROL_ENABLE = 0x1;
    const uint JOB_OBJECT_CPU_RATE_CONTROL_HARD_CAP = 0x4;
    const int ProcessMemoryPriority = 0;
    const int ProcessPowerThrottling = 4;
    const uint MEMORY_PRIORITY_LOW = 2;
    const uint PROCESS_POWER_THROTTLING_CURRENT_VERSION = 1;
    const uint PROCESS_POWER_THROTTLING_EXECUTION_SPEED = 0x1;
    const int ProcessIoPriority = 33;
    const int IoPriorityLow = 1;
    const uint PROCESS_TERMINATE = 0x1;
    const uint PROCESS_SET_QUOTA = 0x100;
    const uint PROCESS_SET_INFORMATION = 0x200;
    const uint PROCESS_QUERY_LIMITED_INFORMATION = 0x1000;
    const uint CREATE_SUSPENDED = 0x4;
    const uint INFINITE = 0xFFFFFFFF;
    const uint TH32CS_SNAPPROCESS = 0x2;
    static readonly IntPtr INVALID_HANDLE_VALUE = new IntPtr(-1);

    // How many logical processors the CPU has in group 0, which
    // EfficientProcessors counts.
    public static int ProcessorCount;

    // The job, which lasts as long as this process does.
    static IntPtr job;

    // The logical processors of the least efficiency class, the efficiency
    // cores, as an affinity mask. If all of them are of one class, all but a
    // quarter of them.
    public static ulong EfficientProcessors() {
        uint length;
        GetSystemCpuSetInformation(IntPtr.Zero, 0, out length, IntPtr.Zero, 0);
        IntPtr buffer = Marshal.AllocHGlobal((int)length);
        try {
            if (!GetSystemCpuSetInformation(buffer, length, out length, IntPtr.Zero, 0)) {
                throw new System.ComponentModel.Win32Exception();
            }
            // SYSTEM_CPU_SET_INFORMATION: its size, its type, then the CPU set's
            // fields, with the logical processor at 14 and the efficiency class
            // at 18. Only group 0's 64 processors, as the job's affinity is one
            // group's, and only CpuSetInformation, the one type there is, as
            // its documentation says to skip types it doesn't know.
            var processors = new System.Collections.Generic.List<(int Logical, int Efficiency)>();
            for (int offset = 0; offset < length;) {
                int size = Marshal.ReadInt32(buffer, offset);
                int type = Marshal.ReadInt32(buffer, offset + 4);
                ushort group = (ushort)Marshal.ReadInt16(buffer, offset + 12);
                if (type == CpuSetInformation && group == 0) {
                    processors.Add((Marshal.ReadByte(buffer, offset + 14), Marshal.ReadByte(buffer, offset + 18)));
                }
                offset += size;
            }
            ProcessorCount = processors.Count;
            int least = int.MaxValue, most = int.MinValue;
            foreach (var processor in processors) {
                least = Math.Min(least, processor.Efficiency);
                most = Math.Max(most, processor.Efficiency);
            }
            ulong mask = 0;
            if (least != most) {
                foreach (var processor in processors) {
                    if (processor.Efficiency == least) {
                        mask |= 1UL << processor.Logical;
                    }
                }
            } else {
                processors.Sort();
                int count = Math.Max(1, processors.Count - processors.Count / 4);
                for (int index = 0; index < count; index++) {
                    mask |= 1UL << processors[index].Logical;
                }
            }
            return mask;
        } finally {
            Marshal.FreeHGlobal(buffer);
        }
    }

    // Puts this process in a job of the name that holds its processes to
    // the processors, at idle priority, and to at most the percent of the
    // whole CPU's time, and that kills them when its last handle closes:
    // this process's, when it ends, unless a process that must outlive it
    // opens one of its own by the name.
    public static void Confine(string name, ulong processors, int percent) {
        job = CreateJobObjectW(IntPtr.Zero, name);
        if (job == IntPtr.Zero) {
            throw new System.ComponentModel.Win32Exception();
        }
        var limits = new ExtendedLimits {
            LimitFlags = JOB_OBJECT_LIMIT_AFFINITY | JOB_OBJECT_LIMIT_PRIORITY_CLASS | JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
            Affinity = new UIntPtr(processors),
            PriorityClass = IDLE_PRIORITY_CLASS,
        };
        if (!SetInformationJobObject(job, JobObjectExtendedLimitInformation, ref limits, Marshal.SizeOf<ExtendedLimits>())) {
            throw new System.ComponentModel.Win32Exception();
        }
        var rate = new CpuRateControl {
            ControlFlags = JOB_OBJECT_CPU_RATE_CONTROL_ENABLE | JOB_OBJECT_CPU_RATE_CONTROL_HARD_CAP,
            CpuRate = (uint)(Math.Clamp(percent, 1, 100) * 100),
        };
        if (!SetInformationJobObject(job, JobObjectCpuRateControlInformation, ref rate, Marshal.SizeOf<CpuRateControl>())) {
            throw new System.ComponentModel.Win32Exception();
        }
        if (!AssignProcessToJobObject(job, GetCurrentProcess())) {
            throw new System.ComponentModel.Win32Exception();
        }
    }

    // Lowers this process's I/O priority and memory priority, which the
    // processes it starts inherit.
    public static void LowerIoAndMemoryPriority() {
        int io = IoPriorityLow;
        int status = NtSetInformationProcess(GetCurrentProcess(), ProcessIoPriority, ref io, sizeof(int));
        if (status != 0) {
            throw new InvalidOperationException("NtSetInformationProcess(ProcessIoPriority) failed with status 0x" + status.ToString("x"));
        }
        var memory = new MemoryPriority { Priority = MEMORY_PRIORITY_LOW };
        if (!SetProcessInformation(GetCurrentProcess(), ProcessMemoryPriority, ref memory, Marshal.SizeOf<MemoryPriority>())) {
            throw new System.ComponentModel.Win32Exception();
        }
    }

    // Starts a thread that, now and every quarter second, puts into the job
    // every process whose parent is in it but that isn't, and puts every
    // process in the job in efficiency mode, as processes don't inherit it.
    // A process that lives less than that is too short to matter.
    public static void WatchJob() {
        AdoptAndThrottle();
        var thread = new Thread(() => {
            while (true) {
                Thread.Sleep(250);
                AdoptAndThrottle();
            }
        }) { IsBackground = true, Name = "RunLightly job watch" };
        thread.Start();
    }

    // The IDs of the job's processes. JOBOBJECT_BASIC_PROCESS_ID_LIST: how
    // many processes the job has, how many IDs fit here, then the IDs. Room
    // for thousands; with more, it fills what fits, and the rest wait for
    // the next time.
    static System.Collections.Generic.HashSet<uint> Members() {
        var members = new System.Collections.Generic.HashSet<uint>();
        int length = 64 * 1024;
        IntPtr buffer = Marshal.AllocHGlobal(length);
        try {
            int returned;
            // With more processes than fit, it fails with ERROR_MORE_DATA,
            // and lists the ones that fit.
            if (!QueryInformationJobObject(job, JobObjectBasicProcessIdList, buffer, length, out returned)
                && Marshal.GetLastPInvokeError() != ERROR_MORE_DATA) {
                return members;
            }
            int count = Math.Min(Marshal.ReadInt32(buffer, 4), (length - 8) / IntPtr.Size);
            for (int index = 0; index < count; index++) {
                members.Add((uint)Marshal.ReadIntPtr(buffer, 8 + index * IntPtr.Size).ToInt64());
            }
        } finally {
            Marshal.FreeHGlobal(buffer);
        }
        return members;
    }

    static void AdoptAndThrottle() {
        var members = Members();
        IntPtr snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if (snapshot != INVALID_HANDLE_VALUE) {
            var entry = new ProcessEntry { dwSize = (uint)Marshal.SizeOf<ProcessEntry>() };
            for (bool more = Process32FirstW(snapshot, ref entry); more; more = Process32NextW(snapshot, ref entry)) {
                if (members.Contains(entry.th32ProcessID) || !members.Contains(entry.th32ParentProcessID)) {
                    continue;
                }
                // The console's host, which csrss starts for a process, and
                // which isn't the process's work.
                if (string.Equals(entry.szExeFile, "conhost.exe", StringComparison.OrdinalIgnoreCase)) {
                    continue;
                }
                // The parent must still be in the job itself: its ID could
                // be another process's by now.
                IntPtr parent = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, entry.th32ParentProcessID);
                if (parent == IntPtr.Zero) {
                    continue;
                }
                bool parentInJob;
                bool known = IsProcessInJob(parent, job, out parentInJob);
                CloseHandle(parent);
                if (!known || !parentInJob) {
                    continue;
                }
                IntPtr process = OpenProcess(PROCESS_SET_QUOTA | PROCESS_TERMINATE | PROCESS_SET_INFORMATION, false, entry.th32ProcessID);
                if (process == IntPtr.Zero) {
                    continue;
                }
                if (AssignProcessToJobObject(job, process)) {
                    members.Add(entry.th32ProcessID);
                }
                CloseHandle(process);
            }
            CloseHandle(snapshot);
        }
        foreach (uint processId in members) {
            IntPtr process = OpenProcess(PROCESS_SET_INFORMATION, false, processId);
            if (process == IntPtr.Zero) {
                // Gone already.
                continue;
            }
            var state = new PowerThrottling {
                Version = PROCESS_POWER_THROTTLING_CURRENT_VERSION,
                ControlMask = PROCESS_POWER_THROTTLING_EXECUTION_SPEED,
                StateMask = PROCESS_POWER_THROTTLING_EXECUTION_SPEED,
            };
            SetProcessInformation(process, ProcessPowerThrottling, ref state, Marshal.SizeOf<PowerThrottling>());
            CloseHandle(process);
        }
    }

    // The argument as a command line's word, quoted as CommandLineToArgvW
    // and C runtimes read it.
    public static string Quote(string argument) {
        if (argument.Length > 0 && argument.IndexOfAny(new[] { ' ', '\t', '\n', '\v', '"' }) < 0) {
            return argument;
        }
        var quoted = new StringBuilder("\"");
        int backslashes = 0;
        foreach (char character in argument) {
            if (character == '\\') {
                backslashes++;
            } else if (character == '"') {
                quoted.Append('\\', backslashes * 2 + 1).Append('"');
                backslashes = 0;
            } else {
                quoted.Append('\\', backslashes).Append(character);
                backslashes = 0;
            }
        }
        return quoted.Append('\\', backslashes * 2).Append('"').ToString();
    }

    // Runs the command line in the job, with this process's console and
    // standard handles, waits for it, and returns its exit code. The
    // process starts suspended, joins the job, and only then runs, so that
    // it and every process it starts are in the job from their first
    // instruction.
    public static int Run(string commandLine) {
        var startupInfo = new StartupInfo { cb = Marshal.SizeOf<StartupInfo>() };
        ProcessInformation information;
        if (!CreateProcessW(null, (commandLine + '\0').ToCharArray(), IntPtr.Zero, IntPtr.Zero, true, CREATE_SUSPENDED, IntPtr.Zero, null, ref startupInfo, out information)) {
            throw new System.ComponentModel.Win32Exception();
        }
        try {
            if (!AssignProcessToJobObject(job, information.hProcess)) {
                throw new System.ComponentModel.Win32Exception();
            }
            if (ResumeThread(information.hThread) == uint.MaxValue) {
                throw new System.ComponentModel.Win32Exception();
            }
            WaitForSingleObject(information.hProcess, INFINITE);
            int exitCode;
            if (!GetExitCodeProcess(information.hProcess, out exitCode)) {
                throw new System.ComponentModel.Win32Exception();
            }
            return exitCode;
        } finally {
            CloseHandle(information.hThread);
            CloseHandle(information.hProcess);
        }
    }
}
'@

$processors = [Lightly]::EfficientProcessors()
$jobName = "KGames RunLightly $([Guid]::NewGuid().ToString('N'))"
[Lightly]::Confine($jobName, $processors, $percent)
$env:KGAMES_LIGHT_JOB = $jobName
[Lightly]::LowerIoAndMemoryPriority()
[Lightly]::WatchJob()

# Build servers would run outside the job, and outlive it.
$env:DOTNET_CLI_USE_MSBUILD_SERVER = '0'
$env:MSBUILDDISABLENODEREUSE = '1'
$env:UseSharedCompilation = 'false'
# As many jobs as the cap is worth in cores, and no more than the cores.
$cores = [System.Numerics.BitOperations]::PopCount($processors)
$share = [Math]::Max(1, [Math]::Round([Lightly]::ProcessorCount * $percent / 100))
$jobs = [Math]::Min($cores, $share)
$env:KGAMES_LIGHT_JOBS = [string]$jobs
# Library/Graphics puts the GPU work of every process that draws after other
# processes'.
$env:KGAMES_GPU_PRIORITY = 'Idle'

# The command: a program, by its path, or else anything PowerShell runs, like
# a script or a cmdlet, in a PowerShell of its own.
$command = $args[0]
$arguments = @($args | Select-Object -Skip 1)
$found = Get-Command $command -ErrorAction SilentlyContinue | Select-Object -First 1
if ($found -and $found.CommandType -eq 'Application') {
    $words = @($found.Source) + $arguments
} else {
    $quoted = @($command) + $arguments | ForEach-Object { "'" + $_.Replace("'", "''") + "'" }
    $words = @((Get-Process -Id $PID).Path, '-NoProfile', '-Command', ('& ' + ($quoted -join ' ') + '; exit $LASTEXITCODE'))
}
$line = ($words | ForEach-Object { [Lightly]::Quote($_) }) -join ' '
exit [Lightly]::Run($line)
