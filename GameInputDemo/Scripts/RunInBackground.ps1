#Requires -Version 7

# Builds GameInputDemo, runs it on one display without taking focus, saves a
# screenshot of its window, and closes it. This is how coding agents run the
# demo while someone works in other windows (see AGENTS.md):
#
#   - The demo opens with --display, so it opens on that display or not at all.
#   - It opens with --no-activate, so it opens behind the window that has
#     focus, and typing there stays there.
#   - The screenshot comes from PrintWindow, which works while the window is
#     behind others, and doesn't activate it.
#
# Without focus, GameInput gives the demo no keyboard or mouse input, but
# gamepads still work.

param(
    # The identifier of the display to open on, as IdentifyDisplays --list shows it.
    [string] $Display = 'CVVNY2',
    # How long the demo runs before the screenshot.
    [double] $Seconds = 3,
    # Where to save the screenshot of the window's contents, as a PNG.
    [string] $Screenshot = (Join-Path $PSScriptRoot '..\target\Screenshot.png'),
    # Leaves the demo running after the screenshot.
    [switch] $KeepOpen,
    # Builds with Cargo's dev profile, which turns on the D3D12 debug layer.
    [switch] $DebugBuild
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

Add-Type -AssemblyName System.Drawing
Add-Type -TypeDefinition @'
using System;
using System.ComponentModel;
using System.Diagnostics;
using System.IO;
using System.Runtime.InteropServices;
using System.Text;

// Starts the demo with its standard output and standard error going to files.
// CreateProcess passes on every inheritable handle unless it's given a list of
// them, and Start-Process gives none. Then the demo would get a copy of the pipe
// that this script's output goes to, and whatever reads that output would wait
// until the demo closed.
public static class DemoProcess {
    [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
    struct STARTUPINFOEX {
        public int cb;
        public string lpReserved, lpDesktop, lpTitle;
        public int dwX, dwY, dwXSize, dwYSize, dwXCountChars, dwYCountChars, dwFillAttribute, dwFlags;
        public short wShowWindow, cbReserved2;
        public IntPtr lpReserved2, hStdInput, hStdOutput, hStdError, lpAttributeList;
    }

    [StructLayout(LayoutKind.Sequential)]
    struct PROCESS_INFORMATION {
        public IntPtr hProcess, hThread;
        public int dwProcessId, dwThreadId;
    }

    [DllImport("kernel32.dll", SetLastError = true)]
    static extern bool InitializeProcThreadAttributeList(IntPtr list, int count, int flags, ref IntPtr size);
    [DllImport("kernel32.dll", SetLastError = true)]
    static extern bool UpdateProcThreadAttribute(IntPtr list, uint flags, IntPtr attribute, IntPtr value, IntPtr size, IntPtr previousValue, IntPtr returnSize);
    [DllImport("kernel32.dll")]
    static extern void DeleteProcThreadAttributeList(IntPtr list);
    [DllImport("kernel32.dll", SetLastError = true)]
    static extern bool SetHandleInformation(IntPtr handle, uint mask, uint flags);
    [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    static extern bool CreateProcessW(string application, StringBuilder commandLine, IntPtr processAttributes, IntPtr threadAttributes, bool inheritHandles, uint flags, IntPtr environment, string directory, ref STARTUPINFOEX startupInfo, out PROCESS_INFORMATION information);
    [DllImport("kernel32.dll")]
    static extern bool CloseHandle(IntPtr handle);

    const uint HANDLE_FLAG_INHERIT = 1;
    const int STARTF_USESTDHANDLES = 0x100;
    const uint EXTENDED_STARTUPINFO_PRESENT = 0x80000;
    static readonly IntPtr PROC_THREAD_ATTRIBUTE_HANDLE_LIST = new IntPtr(0x20002);

    public static Process Start(string commandLine, string outputPath, string errorPath) {
        using (var output = Open(outputPath))
        using (var error = Open(errorPath)) {
            var handles = new[] { output.SafeFileHandle.DangerousGetHandle(), error.SafeFileHandle.DangerousGetHandle() };
            IntPtr size = IntPtr.Zero;
            InitializeProcThreadAttributeList(IntPtr.Zero, 1, 0, ref size);
            IntPtr list = Marshal.AllocHGlobal(size);
            IntPtr handleList = Marshal.AllocHGlobal(IntPtr.Size * handles.Length);
            bool initialized = false;
            try {
                if (!InitializeProcThreadAttributeList(list, 1, 0, ref size)) {
                    throw new Win32Exception();
                }
                initialized = true;
                Marshal.Copy(handles, 0, handleList, handles.Length);
                if (!UpdateProcThreadAttribute(list, 0, PROC_THREAD_ATTRIBUTE_HANDLE_LIST, handleList, new IntPtr(IntPtr.Size * handles.Length), IntPtr.Zero, IntPtr.Zero)) {
                    throw new Win32Exception();
                }
                var startupInfo = new STARTUPINFOEX {
                    cb = Marshal.SizeOf(typeof(STARTUPINFOEX)),
                    dwFlags = STARTF_USESTDHANDLES,
                    hStdOutput = handles[0],
                    hStdError = handles[1],
                    lpAttributeList = list,
                };
                PROCESS_INFORMATION information;
                if (!CreateProcessW(null, new StringBuilder(commandLine), IntPtr.Zero, IntPtr.Zero, true, EXTENDED_STARTUPINFO_PRESENT, IntPtr.Zero, null, ref startupInfo, out information)) {
                    throw new Win32Exception();
                }
                var process = Process.GetProcessById(information.dwProcessId);
                // The Process opens its own handle, which keeps the exit code
                // readable after the demo exits.
                _ = process.Handle;
                CloseHandle(information.hThread);
                CloseHandle(information.hProcess);
                return process;
            } finally {
                if (initialized) {
                    DeleteProcThreadAttributeList(list);
                }
                Marshal.FreeHGlobal(handleList);
                Marshal.FreeHGlobal(list);
            }
        }
    }

    // A file the demo writes to, which this script can read while it does.
    static FileStream Open(string path) {
        var stream = new FileStream(path, FileMode.Create, FileAccess.Write, FileShare.ReadWrite | FileShare.Delete);
        if (!SetHandleInformation(stream.SafeFileHandle.DangerousGetHandle(), HANDLE_FLAG_INHERIT, HANDLE_FLAG_INHERIT)) {
            throw new Win32Exception();
        }
        return stream;
    }
}

public static class DemoWindow {
    [StructLayout(LayoutKind.Sequential)]
    struct RECT { public int Left, Top, Right, Bottom; }

    delegate bool EnumWindowsProc(IntPtr window, IntPtr parameter);

    [DllImport("user32.dll")]
    static extern bool EnumWindows(EnumWindowsProc callback, IntPtr parameter);
    [DllImport("user32.dll")]
    static extern uint GetWindowThreadProcessId(IntPtr window, out uint processId);
    [DllImport("user32.dll")]
    static extern bool IsWindowVisible(IntPtr window);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    static extern int GetClassNameW(IntPtr window, StringBuilder name, int capacity);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    static extern int GetWindowTextW(IntPtr window, StringBuilder text, int capacity);
    [DllImport("user32.dll")]
    static extern bool GetClientRect(IntPtr window, out RECT rect);
    [DllImport("user32.dll")]
    static extern bool PrintWindow(IntPtr window, IntPtr dc, uint flags);
    [DllImport("user32.dll")]
    static extern bool PostMessageW(IntPtr window, uint message, IntPtr wParam, IntPtr lParam);

    const uint PW_CLIENTONLY = 1;
    // Asks the desktop compositor for the window's contents, which is the
    // only way to get what Direct3D draws.
    const uint PW_RENDERFULLCONTENT = 2;
    const uint WM_CLOSE = 0x0010;

    // The demo's window: a visible top-level window of the process, of the
    // class that Main.rs registers. The process has other windows.
    public static IntPtr Find(int processId) {
        IntPtr found = IntPtr.Zero;
        EnumWindows((window, parameter) => {
            uint owner;
            GetWindowThreadProcessId(window, out owner);
            if (owner != processId || !IsWindowVisible(window)) {
                return true;
            }
            var name = new StringBuilder(256);
            GetClassNameW(window, name, name.Capacity);
            if (name.ToString() != "MainWindow") {
                return true;
            }
            found = window;
            return false;
        }, IntPtr.Zero);
        return found;
    }

    public static string Title(IntPtr window) {
        var text = new StringBuilder(512);
        GetWindowTextW(window, text, text.Capacity);
        return text.ToString();
    }

    // The size of the window's contents.
    public static int[] ClientSize(IntPtr window) {
        RECT client;
        GetClientRect(window, out client);
        return new[] { client.Right, client.Bottom };
    }

    // Draws the window's contents into a device context.
    public static bool Print(IntPtr window, IntPtr dc) {
        return PrintWindow(window, dc, PW_CLIENTONLY | PW_RENDERFULLCONTENT);
    }

    // Asks the window to close, as its close button would, without activating it.
    public static void Close(IntPtr window) {
        PostMessageW(window, WM_CLOSE, IntPtr.Zero, IntPtr.Zero);
    }
}
'@

$project = Split-Path $PSScriptRoot -Parent
$manifest = Join-Path $project 'Cargo.toml'
if ($DebugBuild) {
    cargo build --manifest-path $manifest --bin GameInputDemo
    $exe = Join-Path $project 'target\debug\GameInputDemo.exe'
} else {
    cargo build --manifest-path $manifest --bin GameInputDemo --release
    $exe = Join-Path $project 'target\release\GameInputDemo.exe'
}
if ($LASTEXITCODE -ne 0) {
    throw "cargo build failed with exit code $LASTEXITCODE"
}

# The demo has no console, but writes why it couldn't start to standard error.
$outputLog = Join-Path $project 'target\RunInBackgroundOutput.log'
$errorLog = Join-Path $project 'target\RunInBackgroundErrors.log'
$process = [DemoProcess]::Start("`"$exe`" --display `"$Display`" --no-activate", $outputLog, $errorLog)

function Get-ErrorLog {
    if (Test-Path $errorLog) {
        (Get-Content -Raw $errorLog) ?? ''
    } else {
        ''
    }
}

$window = [IntPtr]::Zero
$deadline = (Get-Date).AddSeconds(30)
while ($window -eq [IntPtr]::Zero) {
    if ($process.HasExited) {
        throw "GameInputDemo exited with code $($process.ExitCode) before opening its window. $(Get-ErrorLog)"
    }
    if ((Get-Date) -gt $deadline) {
        $process.Kill()
        throw 'GameInputDemo didn''t open its window within 30 seconds, so it was stopped'
    }
    Start-Sleep -Milliseconds 100
    $window = [DemoWindow]::Find($process.Id)
}

Start-Sleep -Seconds $Seconds
if ($process.HasExited) {
    throw "GameInputDemo exited with code $($process.ExitCode). $(Get-ErrorLog)"
}
$width, $height = [DemoWindow]::ClientSize($window)
if ($width -le 0 -or $height -le 0) {
    throw 'GameInputDemo''s window has no contents to capture'
}
# GDI leaves alpha at 0, so the bitmap has none.
$bitmap = [System.Drawing.Bitmap]::new($width, $height, [System.Drawing.Imaging.PixelFormat]::Format24bppRgb)
try {
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    $dc = $graphics.GetHdc()
    $printed = [DemoWindow]::Print($window, $dc)
    $graphics.ReleaseHdc($dc)
    $graphics.Dispose()
    if (-not $printed) {
        throw 'PrintWindow couldn''t capture GameInputDemo''s window'
    }
    $Screenshot = [System.IO.Path]::GetFullPath($Screenshot)
    $bitmap.Save($Screenshot, [System.Drawing.Imaging.ImageFormat]::Png)
} finally {
    $bitmap.Dispose()
}
Write-Output "Title: $([DemoWindow]::Title($window))"
Write-Output "Screenshot: $Screenshot"

if ($KeepOpen) {
    Write-Output "GameInputDemo is still running, with process ID $($process.Id)"
} else {
    [DemoWindow]::Close($window)
    if (-not $process.WaitForExit(10000)) {
        $process.Kill()
        throw 'GameInputDemo didn''t close within 10 seconds, so it was stopped'
    }
    if ($process.ExitCode -ne 0) {
        throw "GameInputDemo exited with code $($process.ExitCode). $(Get-ErrorLog)"
    }
}
