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
#
# .NET's and MSBuild's build servers would run outside the job, so they're
# turned off, and Cargo runs a job for each of the cores it has.
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

public static class Lightly {
    [StructLayout(LayoutKind.Sequential)]
    struct BasicLimits {
        public long PerProcessUserTimeLimit, PerJobUserTimeLimit;
        public uint LimitFlags;
        public UIntPtr MinimumWorkingSetSize, MaximumWorkingSetSize;
        public uint ActiveProcessLimit;
        public UIntPtr Affinity;
        public uint PriorityClass, SchedulingClass;
    }

    [StructLayout(LayoutKind.Sequential)]
    struct CpuRateControl {
        public uint ControlFlags, CpuRate;
    }

    [DllImport("kernel32.dll", SetLastError = true)]
    static extern bool GetSystemCpuSetInformation(IntPtr information, uint length, out uint returned, IntPtr process, uint flags);
    [DllImport("kernel32.dll", SetLastError = true, CharSet = CharSet.Unicode)]
    static extern IntPtr CreateJobObjectW(IntPtr attributes, string name);
    [DllImport("kernel32.dll", SetLastError = true)]
    static extern bool SetInformationJobObject(IntPtr job, int informationClass, ref BasicLimits information, int length);
    [DllImport("kernel32.dll", SetLastError = true)]
    static extern bool SetInformationJobObject(IntPtr job, int informationClass, ref CpuRateControl information, int length);
    [DllImport("kernel32.dll", SetLastError = true)]
    static extern bool AssignProcessToJobObject(IntPtr job, IntPtr process);
    [DllImport("kernel32.dll")]
    static extern IntPtr GetCurrentProcess();

    const int JobObjectBasicLimitInformation = 2;
    const int JobObjectCpuRateControlInformation = 15;
    const uint JOB_OBJECT_LIMIT_AFFINITY = 0x10;
    const uint JOB_OBJECT_LIMIT_PRIORITY_CLASS = 0x20;
    const uint IDLE_PRIORITY_CLASS = 0x40;
    const uint JOB_OBJECT_CPU_RATE_CONTROL_ENABLE = 0x1;
    const uint JOB_OBJECT_CPU_RATE_CONTROL_HARD_CAP = 0x4;

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
    // the percent of the whole CPU's time. The job lasts as long as they do.
    public static void Confine(ulong processors, int percent) {
        IntPtr job = CreateJobObjectW(IntPtr.Zero, null);
        if (job == IntPtr.Zero) {
            throw new System.ComponentModel.Win32Exception();
        }
        var limits = new BasicLimits {
            LimitFlags = JOB_OBJECT_LIMIT_AFFINITY | JOB_OBJECT_LIMIT_PRIORITY_CLASS,
            Affinity = new UIntPtr(processors),
            PriorityClass = IDLE_PRIORITY_CLASS,
        };
        if (!SetInformationJobObject(job, JobObjectBasicLimitInformation, ref limits, Marshal.SizeOf<BasicLimits>())) {
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
}
'@

$processors = [Lightly]::EfficientProcessors()
[Lightly]::Confine($processors, $percent)

# Build servers would run outside the job, and outlive it.
$env:DOTNET_CLI_USE_MSBUILD_SERVER = '0'
$env:MSBUILDDISABLENODEREUSE = '1'
$env:UseSharedCompilation = 'false'
$env:CARGO_BUILD_JOBS = [string][System.Numerics.BitOperations]::PopCount($processors)

$command = $args[0]
$arguments = @($args | Select-Object -Skip 1)
& $command @arguments
exit $LASTEXITCODE
