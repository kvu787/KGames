#Requires -Version 7

# Runs a command lightly, so that it doesn't disturb someone who's using the
# computer, like playing a game: in a job object, whose limits hold the
# command and every process that it starts, which can't escape them:
#
# - Only on the CPU's efficiency cores, which leaves the performance cores
#   that a game's threads run on to the game. On a CPU whose cores are all
#   alike, on all of its cores but a quarter.
# - At idle priority, so that anything else that wants a core gets it first.
# - With at most a share of the whole CPU's time, 33% unless
#   KGAMES_LIGHT_CPU_PERCENT says otherwise, which also keeps a laptop's
#   power for the game's cores.
# - Killed when this script ends, however that happens, like by a coding
#   agent's time limit, so that no build keeps running unseen.
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
# Cargo runs as many jobs as the cap is worth in cores, like 8 of 24 at 33%,
# instead of one for each efficiency core, so that its work is steady rather
# than bursts that the cap stops, and the other efficiency cores stay free.
# Tests run on as many threads. .NET's and MSBuild's build servers would run
# outside the job, so they're turned off.
#
#     Program\RunLightly\RunLightly.ps1 cargo test --workspace
#
# It exits with the command's exit code.

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if ($args.Count -eq 0) {
    Write-Error 'Name a command to run, like: Program\RunLightly\RunLightly.ps1 cargo test --workspace'
}

$percent = 33
if ($env:KGAMES_LIGHT_CPU_PERCENT) {
    $percent = [int]$env:KGAMES_LIGHT_CPU_PERCENT
}

Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
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
    [DllImport("kernel32.dll")]
    static extern bool CloseHandle(IntPtr handle);
    [DllImport("kernel32.dll")]
    static extern IntPtr GetCurrentProcess();

    const int JobObjectBasicProcessIdList = 3;
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
    const uint PROCESS_SET_INFORMATION = 0x200;

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
            // group's.
            var processors = new System.Collections.Generic.List<(int Logical, int Efficiency)>();
            for (int offset = 0; offset < length;) {
                int size = Marshal.ReadInt32(buffer, offset);
                ushort group = (ushort)Marshal.ReadInt16(buffer, offset + 12);
                if (group == 0) {
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

    // Puts this process, and every process that it starts from now on, in a
    // job that holds them to the processors, at idle priority, and to at most
    // the percent of the whole CPU's time, and that kills them when this
    // process ends.
    public static void Confine(ulong processors, int percent) {
        job = CreateJobObjectW(IntPtr.Zero, null);
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

    // Starts a thread that puts every process in the job in efficiency mode,
    // now and every quarter second, as processes don't inherit it. A process
    // that lives less than that is too short to matter.
    public static void ThrottleJob() {
        ThrottleProcesses();
        var thread = new Thread(() => {
            while (true) {
                Thread.Sleep(250);
                ThrottleProcesses();
            }
        }) { IsBackground = true, Name = "RunLightly efficiency mode" };
        thread.Start();
    }

    static void ThrottleProcesses() {
        // JOBOBJECT_BASIC_PROCESS_ID_LIST: how many processes the job has,
        // how many IDs fit here, then the IDs. Room for thousands; with more,
        // it fills what fits, and the rest wait for the next time.
        int length = 64 * 1024;
        IntPtr buffer = Marshal.AllocHGlobal(length);
        try {
            int returned;
            QueryInformationJobObject(job, JobObjectBasicProcessIdList, buffer, length, out returned);
            int count = Marshal.ReadInt32(buffer, 4);
            for (int index = 0; index < count; index++) {
                uint processId = (uint)Marshal.ReadIntPtr(buffer, 8 + index * IntPtr.Size).ToInt64();
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
        } finally {
            Marshal.FreeHGlobal(buffer);
        }
    }
}
'@

$processors = [Lightly]::EfficientProcessors()
[Lightly]::Confine($processors, $percent)
[Lightly]::LowerIoAndMemoryPriority()
[Lightly]::ThrottleJob()

# Build servers would run outside the job, and outlive it.
$env:DOTNET_CLI_USE_MSBUILD_SERVER = '0'
$env:MSBUILDDISABLENODEREUSE = '1'
$env:UseSharedCompilation = 'false'
# As many jobs as the cap is worth in cores, and no more than the cores.
$cores = [System.Numerics.BitOperations]::PopCount($processors)
$share = [Math]::Max(1, [Math]::Round([Lightly]::ProcessorCount * $percent / 100))
$jobs = [Math]::Min($cores, $share)
$env:CARGO_BUILD_JOBS = [string]$jobs
$env:RUST_TEST_THREADS = [string]$jobs
# Library/Graphics puts the GPU work of every process that draws after other
# processes'.
$env:KGAMES_GPU_PRIORITY = 'Idle'

$command = $args[0]
$arguments = @($args | Select-Object -Skip 1)
& $command @arguments
exit $LASTEXITCODE
