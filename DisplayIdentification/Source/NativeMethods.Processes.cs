using System.Runtime.InteropServices;
using Microsoft.Win32.SafeHandles;

namespace DisplayIdentification;

// Process API.
internal static unsafe partial class NativeMethods {
    internal const uint SEE_MASK_NOCLOSEPROCESS = 0x00000040;
    internal const uint SEE_MASK_NOASYNC = 0x00000100;
    internal const uint SEE_MASK_FLAG_NO_UI = 0x00000400;
    internal const uint SEE_MASK_HMONITOR = 0x00200000;

    internal const uint TH32CS_SNAPPROCESS = 0x00000002;
    internal const uint PROCESS_QUERY_LIMITED_INFORMATION = 0x00001000;
    internal const uint SYNCHRONIZE = 0x00100000;
    internal const uint WAIT_OBJECT_0 = 0;

    internal struct ShellExecuteInfo {
        public uint Size;
        public uint Mask;
        public nint Window;
        public char* Verb;
        public char* File;
        public char* Parameters;
        public char* Directory;
        public int Show;
        public nint InstanceApplication;
        public nint IdList;
        public char* Class;
        public nint ClassKey;
        public uint HotKey;

        // A union of hIcon and hMonitor.
        public nint Monitor;

        public nint Process;
    }

    internal struct ProcessEntry32 {
        public uint Size;
        public uint Usage;
        public uint ProcessId;
        public nuint DefaultHeapId;
        public uint ModuleId;
        public uint Threads;
        public uint ParentProcessId;
        public int PriorityClassBase;
        public uint Flags;
        public fixed char ExeFile[260];
    }

    [LibraryImport("shell32.dll", EntryPoint = "ShellExecuteExW", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    internal static partial bool ShellExecuteEx(ref ShellExecuteInfo info);

    [LibraryImport("kernel32.dll", SetLastError = true)]
    internal static partial SafeFileHandle CreateToolhelp32Snapshot(uint flags, uint processId);

    [LibraryImport("kernel32.dll", EntryPoint = "Process32FirstW", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    internal static partial bool Process32First(SafeFileHandle snapshot, ref ProcessEntry32 entry);

    [LibraryImport("kernel32.dll", EntryPoint = "Process32NextW", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    internal static partial bool Process32Next(SafeFileHandle snapshot, ref ProcessEntry32 entry);

    [LibraryImport("kernel32.dll", SetLastError = true)]
    internal static partial SafeProcessHandle OpenProcess(uint access, [MarshalAs(UnmanagedType.Bool)] bool inheritHandle, uint processId);

    [LibraryImport("kernel32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    internal static partial bool GetProcessTimes(SafeProcessHandle process, out long creationTime, out long exitTime, out long kernelTime, out long userTime);

    [LibraryImport("kernel32.dll")]
    internal static partial uint GetProcessId(SafeProcessHandle process);

    [LibraryImport("kernel32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    internal static partial bool GetExitCodeProcess(SafeProcessHandle process, out uint exitCode);

    [LibraryImport("kernel32.dll")]
    internal static partial uint WaitForSingleObject(SafeProcessHandle handle, uint milliseconds);
}
