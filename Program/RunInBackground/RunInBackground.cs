// Builds one of the GUI programs in Program, runs it on one display without
// taking focus, saves a screenshot of its window, and closes it. This is how
// coding agents run the programs while someone works in other windows (see
// the AGENTS.md at the root of the repository):
//
//   - The program opens with --display, so it opens on that display or not
//     at all.
//   - It opens with --no-activate, so it opens without taking focus from the
//     window that has it, and typing there stays there.
//   - The screenshot comes from PrintWindow, which works while the window is
//     behind others, and doesn't activate it.
//
// Without focus, programs ignore all input. To test input, it runs scripts
// instead: with --script, the program reads its input from a script, saves
// the frames and state that the script asks for, checks its state against
// what the script expects, and exits. A run fails if the program's window
// ever becomes the foreground window, which would take focus from the user.
//
// This is a .NET file-based app, which RunInBackground.ps1 runs with
// dotnet run. It builds into dotnet's own cache, not next to this file.

#:property TargetFramework=net10.0-windows
#:property UseWindowsForms=true
#:property AllowUnsafeBlocks=true
#:property PublishAot=false

using System.ComponentModel;
using System.Diagnostics;
using System.Drawing;
using System.Drawing.Imaging;
using System.Globalization;
using System.Runtime.InteropServices;

const string Usage = """
    Usage: RunInBackground.ps1 <program> [options]

      <program>               The program to run, like GameInputDemo: the name
                              of its folder in Program, and of its exe

      --display <identifier>  The display to open on, as IdentifyDisplays --list
                              shows it. CVVNY2 by default
      --seconds <seconds>     How long the program runs before the screenshot. 3
                              by default
      --screenshot <path>     Where to save the screenshot of the window's
                              contents, as a PNG.
                              target\RunInBackground\<program>.png by default
      --keep-open             Leaves the program running after the screenshot
      --debug-build           Builds with Cargo's dev profile, which turns on the
                              D3D12 debug layer

      --script <path>         Runs a script of input instead, and saves what it
                              captures in target\RunInBackground\<program>\<script>
      --all-scripts           Runs every script in Test\<program>\Scripts
      --timeout <seconds>     How long a script may run. 60 by default
    """;

try {
    Run(args);
    return 0;
} catch (FailureException failure) {
    Console.Error.WriteLine(failure.Message);
    return 1;
}

static void Run(string[] arguments) {
    string? program = null;
    var display = "CVVNY2";
    var seconds = 3.0;
    string? screenshot = null;
    var keepOpen = false;
    var debugBuild = false;
    var scripts = new List<string>();
    var allScripts = false;
    var timeout = 60.0;
    for (var i = 0; i < arguments.Length; i++) {
        switch (arguments[i]) {
        case "--display":
            display = Value(arguments, ref i);
            break;
        case "--seconds":
            if (!double.TryParse(Value(arguments, ref i), NumberStyles.Float, CultureInfo.InvariantCulture, out seconds) || seconds < 0) {
                throw new FailureException($"--seconds needs a number of seconds after it, not {arguments[i]}");
            }
            break;
        case "--screenshot":
            screenshot = Value(arguments, ref i);
            break;
        case "--keep-open":
            keepOpen = true;
            break;
        case "--debug-build":
            debugBuild = true;
            break;
        case "--script":
            // A path given on the command line is relative to the current directory.
            scripts.Add(Path.GetFullPath(Value(arguments, ref i)));
            break;
        case "--all-scripts":
            allScripts = true;
            break;
        case "--timeout":
            if (!double.TryParse(Value(arguments, ref i), NumberStyles.Float, CultureInfo.InvariantCulture, out timeout) || timeout <= 0) {
                throw new FailureException($"--timeout needs a number of seconds after it, not {arguments[i]}");
            }
            break;
        case "--help":
            Console.WriteLine(Usage);
            return;
        default:
            if (program != null || arguments[i].StartsWith('-')) {
                throw new FailureException($"Unknown argument {arguments[i]}\n\n{Usage}");
            }
            program = arguments[i];
            break;
        }
    }
    if (program == null) {
        throw new FailureException($"Name the program to run\n\n{Usage}");
    }

    // This file is in Program\RunInBackground. The repository's Cargo
    // workspace builds every program into the target folder at its root.
    var here = (string)AppContext.GetData("EntryPointFileDirectoryPath")!;
    var repository = Path.GetFullPath(Path.Combine(here, "..", ".."));
    var target = Path.Combine(repository, "target");
    var output = Path.Combine(target, "RunInBackground");
    Directory.CreateDirectory(output);
    // A path given on the command line is relative to the current directory.
    screenshot = Path.GetFullPath(screenshot ?? Path.Combine(output, $"{program}.png"));

    if (allScripts) {
        var folder = Path.Combine(repository, "Test", program, "Scripts");
        var found = Directory.Exists(folder) ? Directory.GetFiles(folder, "*.json") : [];
        if (found.Length == 0) {
            throw new FailureException($"{program} has no scripts in {folder}");
        }
        scripts.AddRange(found.Order(StringComparer.OrdinalIgnoreCase));
    }

    var build = new List<string> { "build", "--manifest-path", Path.Combine(repository, "Cargo.toml"), "--bin", program };
    if (!debugBuild) {
        build.Add("--release");
    }
    using (var cargo = Process.Start("cargo", build)) {
        cargo.WaitForExit();
        if (cargo.ExitCode != 0) {
            throw new FailureException($"cargo build failed with exit code {cargo.ExitCode}");
        }
    }
    var exe = Path.Combine(target, debugBuild ? "debug" : "release", $"{program}.exe");

    if (scripts.Count > 0) {
        RunScripts(program, exe, display, scripts, Path.Combine(output, program), timeout);
        return;
    }

    // The program has no console, but writes why it couldn't start to standard error.
    var outputLog = Path.Combine(output, $"{program}Output.log");
    var errorLog = Path.Combine(output, $"{program}Errors.log");
    using var process = ProgramProcess.Start($"\"{exe}\" --display \"{display}\" --no-activate", outputLog, errorLog);
    // From the start, as a window could take focus as soon as it opens.
    using var watch = new FocusWatch(process.Id);

    var window = IntPtr.Zero;
    var waiting = Stopwatch.StartNew();
    while (window == IntPtr.Zero) {
        if (process.HasExited) {
            throw new FailureException($"{program} exited with code {process.ExitCode} before opening its window. {ReadLog(errorLog)}");
        }
        if (waiting.Elapsed > TimeSpan.FromSeconds(30)) {
            process.Kill();
            throw new FailureException($"{program} didn't open its window within 30 seconds, so it was stopped");
        }
        Thread.Sleep(100);
        window = ProgramWindow.Find(process.Id);
    }

    Thread.Sleep(TimeSpan.FromSeconds(seconds));
    if (watch.TookFocus) {
        process.Kill();
        throw new FailureException($"{program}'s window took focus, so it was stopped");
    }
    if (process.HasExited) {
        throw new FailureException($"{program} exited with code {process.ExitCode}. {ReadLog(errorLog)}");
    }
    var (width, height) = ProgramWindow.ClientSize(window);
    if (width <= 0 || height <= 0) {
        throw new FailureException($"{program}'s window has no contents to capture");
    }
    // GDI leaves alpha at 0, so the bitmap has none.
    using (var bitmap = new Bitmap(width, height, PixelFormat.Format24bppRgb)) {
        bool printed;
        using (var graphics = Graphics.FromImage(bitmap)) {
            var dc = graphics.GetHdc();
            printed = ProgramWindow.Print(window, dc);
            graphics.ReleaseHdc(dc);
        }
        if (!printed) {
            throw new FailureException($"PrintWindow couldn't capture {program}'s window");
        }
        bitmap.Save(screenshot, ImageFormat.Png);
    }
    Console.WriteLine($"Title: {ProgramWindow.Title(window)}");
    Console.WriteLine($"Screenshot: {screenshot}");

    if (keepOpen) {
        Console.WriteLine($"{program} is still running, with process ID {process.Id}");
    } else {
        ProgramWindow.Close(window);
        if (!process.WaitForExit(10000)) {
            process.Kill();
            throw new FailureException($"{program} didn't close within 10 seconds, so it was stopped");
        }
        if (process.ExitCode != 0) {
            throw new FailureException($"{program} exited with code {process.ExitCode}. {ReadLog(errorLog)}");
        }
        if (watch.TookFocus) {
            throw new FailureException($"{program}'s window took focus");
        }
    }
}

// Runs each script, one at a time, and says which passed. A script fails if
// the program exits with an error, which includes expectations it didn't
// meet, if it runs past the timeout, or if its window takes focus.
static void RunScripts(string program, string exe, string display, List<string> scripts, string output, double timeout) {
    var failed = new List<string>();
    foreach (var script in scripts) {
        var name = Path.GetFileNameWithoutExtension(script);
        var folder = Path.Combine(output, name);
        if (Directory.Exists(folder)) {
            Directory.Delete(folder, true);
        }
        Directory.CreateDirectory(folder);
        var outputLog = Path.Combine(folder, "Output.log");
        var errorLog = Path.Combine(folder, "Errors.log");
        string? failure = null;
        using (var process = ProgramProcess.Start($"\"{exe}\" --display \"{display}\" --no-activate --script \"{script}\" --output \"{folder}\"", outputLog, errorLog)) {
            using (var watch = new FocusWatch(process.Id)) {
                if (!process.WaitForExit(TimeSpan.FromSeconds(timeout))) {
                    process.Kill();
                    process.WaitForExit();
                    failure = $"it didn't finish within {timeout} seconds, so it was stopped";
                }
                if (watch.TookFocus) {
                    failure = "its window took focus";
                }
            }
            if (failure == null && process.ExitCode != 0) {
                failure = $"{program} exited with code {process.ExitCode}:\n{ReadLog(errorLog).TrimEnd()}";
            }
        }
        var captures = Directory.GetFiles(folder, "*.png").Order(StringComparer.OrdinalIgnoreCase).ToList();
        if (failure == null) {
            Console.WriteLine($"Passed: {name}");
        } else {
            Console.WriteLine($"Failed: {name}: {failure}");
            failed.Add(name);
        }
        foreach (var capture in captures) {
            Console.WriteLine($"  {capture}");
        }
    }
    var summary = $"{scripts.Count - failed.Count} of {scripts.Count} scripts passed";
    if (failed.Count > 0) {
        throw new FailureException($"{summary}. Failed: {string.Join(", ", failed)}");
    }
    Console.WriteLine(summary);
}

// The argument after the option at index, which it moves past.
static string Value(string[] arguments, ref int index) {
    if (index + 1 >= arguments.Length) {
        throw new FailureException($"{arguments[index]} needs a value after it");
    }
    return arguments[++index];
}

// A log the program may still be writing to.
static string ReadLog(string path) {
    try {
        using var stream = new FileStream(path, FileMode.Open, FileAccess.Read, FileShare.ReadWrite | FileShare.Delete);
        using var reader = new StreamReader(stream);
        return reader.ReadToEnd();
    } catch (FileNotFoundException) {
        return "";
    }
}

// Watches, until it's disposed, whether a window of the process ever becomes
// the foreground window, which would mean that it took focus from the user.
// Windows reports every change of the foreground window to the watch's
// thread, so even one that lasts a moment counts, unlike with checking now
// and then.
sealed unsafe partial class FocusWatch : IDisposable {
    [StructLayout(LayoutKind.Sequential)]
    struct MSG {
        public IntPtr hwnd;
        public uint message;
        public IntPtr wParam, lParam;
        public uint time;
        public int x, y;
        public uint lPrivate;
    }

    [LibraryImport("user32.dll")]
    private static partial IntPtr SetWinEventHook(uint eventMin, uint eventMax, IntPtr module, delegate* unmanaged<IntPtr, uint, IntPtr, int, int, uint, uint, void> callback, uint processId, uint threadId, uint flags);
    [LibraryImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static partial bool UnhookWinEvent(IntPtr hook);
    [LibraryImport("user32.dll")]
    private static partial int GetMessageW(out MSG message, IntPtr window, uint filterMin, uint filterMax);
    [LibraryImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static partial bool PeekMessageW(out MSG message, IntPtr window, uint filterMin, uint filterMax, uint remove);
    [LibraryImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static partial bool PostThreadMessageW(uint threadId, uint message, IntPtr wParam, IntPtr lParam);
    [LibraryImport("kernel32.dll")]
    private static partial uint GetCurrentThreadId();

    const uint EVENT_SYSTEM_FOREGROUND = 3;
    const uint WINEVENT_OUTOFCONTEXT = 0;
    const uint PM_NOREMOVE = 0;
    const uint WM_QUIT = 0x0012;

    // Windows calls OnForeground on the thread that watches, while it waits
    // for messages.
    [ThreadStatic]
    static FocusWatch? watching;

    readonly uint processId;
    readonly Thread thread;
    readonly ManualResetEventSlim started = new();
    uint threadId;
    bool hooked;
    volatile bool tookFocus;

    public FocusWatch(int processId) {
        this.processId = (uint)processId;
        this.thread = new Thread(this.Watch) { IsBackground = true };
        this.thread.Start();
        this.started.Wait();
        if (!this.hooked) {
            this.thread.Join();
            throw new FailureException("Couldn't watch which window has focus");
        }
    }

    public bool TookFocus => this.tookFocus;

    void Watch() {
        watching = this;
        // Makes the thread's message queue, so that Dispose can post to it.
        _ = PeekMessageW(out _, IntPtr.Zero, 0, 0, PM_NOREMOVE);
        this.threadId = GetCurrentThreadId();
        var hook = SetWinEventHook(EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_FOREGROUND, IntPtr.Zero, &OnForeground, 0, 0, WINEVENT_OUTOFCONTEXT);
        this.hooked = hook != IntPtr.Zero;
        this.started.Set();
        if (!this.hooked) {
            return;
        }
        // The window with focus before the first change.
        this.Check(ProgramWindow.Foreground());
        // Windows calls OnForeground from within GetMessageW, until Dispose
        // posts WM_QUIT.
        while (GetMessageW(out _, IntPtr.Zero, 0, 0) > 0) {
        }
        this.Check(ProgramWindow.Foreground());
        _ = UnhookWinEvent(hook);
    }

    [UnmanagedCallersOnly]
    static void OnForeground(IntPtr hook, uint winEvent, IntPtr window, int objectId, int childId, uint eventThread, uint time) {
        watching?.Check(window);
    }

    void Check(IntPtr window) {
        if (ProgramWindow.Owner(window) == this.processId) {
            this.tookFocus = true;
        }
    }

    public void Dispose() {
        _ = PostThreadMessageW(this.threadId, WM_QUIT, IntPtr.Zero, IntPtr.Zero);
        this.thread.Join();
    }
}

// An error to show without a stack trace.
sealed class FailureException(string message) : Exception(message);

// Starts the program with its standard output and standard error going to files.
// CreateProcess passes on every inheritable handle unless it's given a list of
// them, and Process.Start gives none. Then the program would get a copy of the pipe
// that this program's output goes to, and whatever reads that output would wait
// until the program closed.
static partial class ProgramProcess {
    [StructLayout(LayoutKind.Sequential)]
    struct STARTUPINFOEX {
        public int cb;
        public IntPtr lpReserved, lpDesktop, lpTitle;
        public int dwX, dwY, dwXSize, dwYSize, dwXCountChars, dwYCountChars, dwFillAttribute, dwFlags;
        public short wShowWindow, cbReserved2;
        public IntPtr lpReserved2, hStdInput, hStdOutput, hStdError, lpAttributeList;
    }

    [StructLayout(LayoutKind.Sequential)]
    struct PROCESS_INFORMATION {
        public IntPtr hProcess, hThread;
        public int dwProcessId, dwThreadId;
    }

    [LibraryImport("kernel32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static partial bool InitializeProcThreadAttributeList(IntPtr list, int count, int flags, ref IntPtr size);
    [LibraryImport("kernel32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static partial bool UpdateProcThreadAttribute(IntPtr list, uint flags, IntPtr attribute, IntPtr value, IntPtr size, IntPtr previousValue, IntPtr returnSize);
    [LibraryImport("kernel32.dll")]
    private static partial void DeleteProcThreadAttributeList(IntPtr list);
    [LibraryImport("kernel32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static partial bool SetHandleInformation(IntPtr handle, uint mask, uint flags);
    // CreateProcessW may write to the command line, so it isn't a string.
    [LibraryImport("kernel32.dll", SetLastError = true, StringMarshalling = StringMarshalling.Utf16)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static partial bool CreateProcessW(string? application, char[] commandLine, IntPtr processAttributes, IntPtr threadAttributes, [MarshalAs(UnmanagedType.Bool)] bool inheritHandles, uint flags, IntPtr environment, string? directory, ref STARTUPINFOEX startupInfo, out PROCESS_INFORMATION information);
    [LibraryImport("kernel32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static partial bool CloseHandle(IntPtr handle);

    const uint HANDLE_FLAG_INHERIT = 1;
    const int STARTF_USESTDHANDLES = 0x100;
    const uint EXTENDED_STARTUPINFO_PRESENT = 0x80000;
    static readonly IntPtr PROC_THREAD_ATTRIBUTE_HANDLE_LIST = new(0x20002);

    public static Process Start(string commandLine, string outputPath, string errorPath) {
        using var output = Open(outputPath);
        using var error = Open(errorPath);
        var handles = new[] { output.SafeFileHandle.DangerousGetHandle(), error.SafeFileHandle.DangerousGetHandle() };
        var size = IntPtr.Zero;
        // This call only gets the size, and fails because the list is missing.
        _ = InitializeProcThreadAttributeList(IntPtr.Zero, 1, 0, ref size);
        var list = Marshal.AllocHGlobal(size);
        var handleList = Marshal.AllocHGlobal(IntPtr.Size * handles.Length);
        var initialized = false;
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
                cb = Marshal.SizeOf<STARTUPINFOEX>(),
                dwFlags = STARTF_USESTDHANDLES,
                hStdOutput = handles[0],
                hStdError = handles[1],
                lpAttributeList = list,
            };
            if (!CreateProcessW(null, (commandLine + '\0').ToCharArray(), IntPtr.Zero, IntPtr.Zero, true, EXTENDED_STARTUPINFO_PRESENT, IntPtr.Zero, null, ref startupInfo, out var information)) {
                throw new Win32Exception();
            }
            var process = Process.GetProcessById(information.dwProcessId);
            // The Process opens its own handle, which keeps the exit code
            // readable after the program exits.
            _ = process.Handle;
            _ = CloseHandle(information.hThread);
            _ = CloseHandle(information.hProcess);
            return process;
        } finally {
            if (initialized) {
                DeleteProcThreadAttributeList(list);
            }
            Marshal.FreeHGlobal(handleList);
            Marshal.FreeHGlobal(list);
        }
    }

    // A file the program writes to, which this program can read while it does.
    static FileStream Open(string path) {
        var stream = new FileStream(path, FileMode.Create, FileAccess.Write, FileShare.ReadWrite | FileShare.Delete);
        if (!SetHandleInformation(stream.SafeFileHandle.DangerousGetHandle(), HANDLE_FLAG_INHERIT, HANDLE_FLAG_INHERIT)) {
            throw new Win32Exception();
        }
        return stream;
    }
}

static partial class ProgramWindow {
    [StructLayout(LayoutKind.Sequential)]
    struct RECT { public int Left, Top, Right, Bottom; }

    [LibraryImport("user32.dll", StringMarshalling = StringMarshalling.Utf16)]
    private static partial IntPtr FindWindowExW(IntPtr parent, IntPtr after, string? className, string? windowName);
    [LibraryImport("user32.dll")]
    private static partial uint GetWindowThreadProcessId(IntPtr window, out uint processId);
    [LibraryImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static partial bool IsWindowVisible(IntPtr window);
    [LibraryImport("user32.dll", StringMarshalling = StringMarshalling.Utf16)]
    private static partial int GetWindowTextW(IntPtr window, [Out] char[] text, int capacity);
    [LibraryImport("user32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static partial bool GetClientRect(IntPtr window, out RECT rect);
    [LibraryImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static partial bool PrintWindow(IntPtr window, IntPtr dc, uint flags);
    [LibraryImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static partial bool PostMessageW(IntPtr window, uint message, IntPtr wParam, IntPtr lParam);
    [LibraryImport("user32.dll")]
    private static partial IntPtr GetForegroundWindow();

    const uint PW_CLIENTONLY = 1;
    // Asks the desktop compositor for the window's contents, which is the
    // only way to get what Direct3D draws.
    const uint PW_RENDERFULLCONTENT = 2;
    const uint WM_CLOSE = 0x0010;

    // The program's window: a visible top-level window of the process, of the
    // class that Library/Application registers. The process has other windows.
    public static IntPtr Find(int processId) {
        var window = IntPtr.Zero;
        while ((window = FindWindowExW(IntPtr.Zero, window, "MainWindow", null)) != IntPtr.Zero) {
            _ = GetWindowThreadProcessId(window, out var owner);
            if (owner == processId && IsWindowVisible(window)) {
                return window;
            }
        }
        return IntPtr.Zero;
    }

    // The window that has focus, or none.
    public static IntPtr Foreground() {
        return GetForegroundWindow();
    }

    // The ID of the process that owns a window, or 0 for none.
    public static uint Owner(IntPtr window) {
        if (window == IntPtr.Zero || GetWindowThreadProcessId(window, out var process) == 0) {
            return 0;
        }
        return process;
    }

    public static string Title(IntPtr window) {
        var text = new char[512];
        var length = GetWindowTextW(window, text, text.Length);
        return new string(text, 0, length);
    }

    // The size of the window's contents.
    public static (int Width, int Height) ClientSize(IntPtr window) {
        if (!GetClientRect(window, out var client)) {
            throw new Win32Exception();
        }
        return (client.Right, client.Bottom);
    }

    // Draws the window's contents into a device context.
    public static bool Print(IntPtr window, IntPtr dc) {
        return PrintWindow(window, dc, PW_CLIENTONLY | PW_RENDERFULLCONTENT);
    }

    // Asks the window to close, as its close button would, without activating it.
    public static void Close(IntPtr window) {
        _ = PostMessageW(window, WM_CLOSE, IntPtr.Zero, IntPtr.Zero);
    }
}
