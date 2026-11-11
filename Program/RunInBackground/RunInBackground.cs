// Builds one of the GUI programs in Program, runs it on one display without
// taking focus, saves a screenshot of its window, and closes it. This is how
// coding agents run the programs while someone works in other windows (see
// the AGENTS.md at the root of the repository):
//
//   - The program opens with --display, so it opens on that display or not
//     at all.
//   - It opens with --no-activate, so it opens without taking focus from the
//     window that has it, and typing there stays there.
//   - It opens with --vsync, whatever its settings say, so it draws at most
//     one frame a refresh, and waits between them instead of spinning, which
//     keeps it light on the GPU and the CPU, for someone who's using the
//     computer, like playing a game. --no-vsync opens it with --no-vsync
//     instead, like for measuring frame times.
//   - It opens with --no-input, so it doesn't start GameInput, or the helper
//     process that GameInput starts to read keyboards and mice. Without
//     focus, it would ignore all input anyway.
//   - It opens with --theme Light and --window-mode MaximizedWindowed,
//     whatever its settings say, so that its scripts and screenshots start
//     the same way whatever someone set for themselves. --arguments can say
//     otherwise.
//   - The screenshot comes from PrintWindow, which works while the window is
//     behind others, and doesn't activate it.
//   - It runs in a job that ends it if this program stops partway.
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
                              of its folder in Program

      --display <identifier>  The display to open on, as IdentifyDisplays --list
                              shows it. CVVNY2 by default
      --seconds <seconds>     How long the program runs before the screenshot. 3
                              by default
      --screenshot <path>     Where to save the screenshot of the window's
                              contents, as a PNG.
                              target\RunInBackground\<program>.png by default
      --keep-open             Leaves the program running after the screenshot
      --debug-build           Builds with Cargo's dev profile, which turns on the
                              D3D12 debug layer, or a WinUI 3 app's Debug
                              configuration
      --no-vsync              Runs without vertical sync, drawing frames as fast
                              as it can. Runs are vertically synced by default

      --script <path>         Runs a script of input instead, and saves what it
                              captures in target\RunInBackground\<program>\<script>
      --all-scripts           Runs every script in Test\<program>\Scripts
      --timeout <seconds>     How long a script may run. 60 by default

      --arguments <text>      More arguments for the program, like a WinUI 3
                              program's "--page Settings --theme Dark"
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
    var verticalSync = true;
    var scripts = new List<string>();
    var allScripts = false;
    var timeout = 60.0;
    var extraArguments = "";
    for (var i = 0; i < arguments.Length; i++) {
        switch (arguments[i]) {
        case "--arguments":
            extraArguments = " " + Value(arguments, ref i);
            break;
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
        case "--no-vsync":
            verticalSync = false;
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

    // A program with a .csproj, like WinUi3Sample, is a WinUI 3 app, which dotnet builds, and
    // which isn't built on Library/UserInterface: it has no scripts, GameInput or vertical sync
    // to turn off, and its window is WinUI's.
    var project = Path.Combine(repository, "Program", program, $"{program}.csproj");
    var winUi = File.Exists(project);
    if (winUi && scripts.Count > 0) {
        throw new FailureException($"{program} is a WinUI 3 app, which doesn't run scripts");
    }

    string exe;
    if (winUi) {
        var configuration = debugBuild ? "Debug" : "Release";
        var dotnetExit = ProgramProcess.RunInLightJob(["dotnet", "build", project, "--configuration", configuration, "--nologo", "--verbosity", "minimal"]);
        if (dotnetExit != 0) {
            throw new FailureException($"dotnet build failed with exit code {dotnetExit}");
        }
        // Where the program's Directory.Build.props puts the build.
        exe = Path.Combine(target, "DotNet", "bin", program, $"{configuration.ToLowerInvariant()}_win-x64", $"{program}.exe");
    } else {
        // A Rust program's exe has its package's name, like game_input_demo for GameInputDemo.
        var manifest = Path.Combine(repository, "Program", program, "Cargo.toml");
        if (!File.Exists(manifest)) {
            throw new FailureException($"Program\\{program} has no Cargo.toml or .csproj");
        }
        var package = PackageName(manifest);
        var build = new List<string> { "cargo", "build", "--manifest-path", Path.Combine(repository, "Cargo.toml"), "--bin", package };
        if (!debugBuild) {
            build.Add("--release");
        }
        var cargoExit = ProgramProcess.RunInLightJob(build);
        if (cargoExit != 0) {
            throw new FailureException($"cargo build failed with exit code {cargoExit}");
        }
        exe = Path.Combine(target, debugBuild ? "debug" : "release", $"{package}.exe");
    }
    var sync = verticalSync ? " --vsync" : " --no-vsync";
    // Light and maximized windowed, whatever the program's settings say, so that every run starts
    // the same way. --arguments come after these, so they can say otherwise.
    const string LightAndMaximizedWindowed = " --theme Light --window-mode MaximizedWindowed";

    if (scripts.Count > 0) {
        RunScripts(program, exe, display, sync + LightAndMaximizedWindowed + extraArguments, scripts, Path.Combine(output, program), timeout);
        return;
    }

    // The program has no console, but writes why it couldn't start to standard error.
    var outputLog = Path.Combine(output, $"{program}Output.log");
    var errorLog = Path.Combine(output, $"{program}Errors.log");
    var options = winUi ? "" : $" --no-input{sync}{LightAndMaximizedWindowed}";
    using var process = ProgramProcess.Start($"\"{exe}\" --display \"{display}\" --no-activate{options}{extraArguments}", outputLog, errorLog, keepOpen);
    // From the start, as a window could take focus as soon as it opens.
    using var watch = new FocusWatch(process);

    var window = IntPtr.Zero;
    var waiting = Stopwatch.StartNew();
    while (window == IntPtr.Zero) {
        if (watch.TookFocus) {
            throw new FailureException($"{program}'s window took focus, so it was stopped");
        }
        if (process.HasExited) {
            throw new FailureException($"{program} exited with code {process.ExitCode} before opening its window. {ReadLog(errorLog)}");
        }
        if (waiting.Elapsed > TimeSpan.FromSeconds(30)) {
            process.Kill();
            throw new FailureException($"{program} didn't open its window within 30 seconds, so it was stopped");
        }
        Thread.Sleep(100);
        window = ProgramWindow.Find(process.Id, winUi ? ProgramWindow.WinUiClass : ProgramWindow.LibraryClass);
    }

    Thread.Sleep(TimeSpan.FromSeconds(seconds));
    if (watch.TookFocus) {
        // The watch stopped it already.
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
        // WinUI 3's menus, flyouts and tooltips are windows of their own, over the program's.
        if (winUi) {
            using var graphics = Graphics.FromImage(bitmap);
            var origin = ProgramWindow.ClientOrigin(window);
            foreach (var popup in ProgramWindow.Popups(process.Id, window)) {
                var area = ProgramWindow.Area(popup);
                if (area.Width <= 0 || area.Height <= 0) {
                    continue;
                }
                using var image = new Bitmap(area.Width, area.Height, PixelFormat.Format24bppRgb);
                using (var popupGraphics = Graphics.FromImage(image)) {
                    var dc = popupGraphics.GetHdc();
                    _ = ProgramWindow.PrintWhole(popup, dc);
                    popupGraphics.ReleaseHdc(dc);
                }
                using var layer = WithoutBlackMargin(image);
                graphics.DrawImageUnscaled(layer, area.Left - origin.X, area.Top - origin.Y);
            }
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
    }
    // Once the watch has handled every change of focus that Windows reported.
    watch.Dispose();
    if (watch.TookFocus) {
        throw new FailureException($"{program}'s window took focus");
    }
}

// Runs each script, one at a time, and says which passed. A script fails if
// the program exits with an error, which includes expectations it didn't
// meet, if it runs past the timeout, or if its window takes focus.
static void RunScripts(string program, string exe, string display, string options, List<string> scripts, string output, double timeout) {
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
        using (var process = ProgramProcess.Start($"\"{exe}\" --display \"{display}\" --no-activate --no-input{options} --script \"{script}\" --output \"{folder}\"", outputLog, errorLog, false)) {
            var watch = new FocusWatch(process);
            using (watch) {
                if (!process.WaitForExit(TimeSpan.FromSeconds(timeout))) {
                    process.Kill();
                    process.WaitForExit();
                    failure = $"it didn't finish within {timeout} seconds, so it was stopped";
                }
            }
            // Once the watch has handled every change of focus that Windows reported.
            if (watch.TookFocus) {
                failure = "its window took focus";
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

// The name in a Cargo.toml's [package] table, which Cargo gives its program's exe.
static string PackageName(string manifest) {
    var package = false;
    foreach (var line in File.ReadLines(manifest)) {
        var text = line.Trim();
        if (text.StartsWith('[')) {
            package = text == "[package]";
            continue;
        }
        var parts = text.Split('=', 2);
        if (package && parts.Length == 2 && parts[0].Trim() == "name") {
            return parts[1].Trim().Trim('"');
        }
    }
    throw new FailureException($"{manifest} names no package");
}

// A popup's picture, with the black that PrintWindow gives the transparent
// margin around it, where its shadow goes, made transparent: the pure black
// that reaches the picture's edges, which the popup's own black doesn't.
static Bitmap WithoutBlackMargin(Bitmap image) {
    var (width, height) = (image.Width, image.Height);
    var bounds = new Rectangle(0, 0, width, height);
    var pixels = new int[width * height];
    // Read as 32 bits a pixel, with an opaque alpha.
    var source = image.LockBits(bounds, ImageLockMode.ReadOnly, PixelFormat.Format32bppArgb);
    for (var y = 0; y < height; y++) {
        Marshal.Copy(source.Scan0 + y * source.Stride, pixels, y * width, width);
    }
    image.UnlockBits(source);

    var margin = new bool[pixels.Length];
    var queue = new Queue<int>();
    void Visit(int index) {
        if (!margin[index] && (pixels[index] & 0xFFFFFF) == 0) {
            margin[index] = true;
            queue.Enqueue(index);
        }
    }
    for (var x = 0; x < width; x++) {
        Visit(x);
        Visit((height - 1) * width + x);
    }
    for (var y = 0; y < height; y++) {
        Visit(y * width);
        Visit(y * width + width - 1);
    }
    while (queue.TryDequeue(out var index)) {
        var (x, y) = (index % width, index / width);
        if (x > 0) {
            Visit(index - 1);
        }
        if (x < width - 1) {
            Visit(index + 1);
        }
        if (y > 0) {
            Visit(index - width);
        }
        if (y < height - 1) {
            Visit(index + width);
        }
    }
    for (var index = 0; index < pixels.Length; index++) {
        if (margin[index]) {
            pixels[index] = 0;
        }
    }

    var result = new Bitmap(width, height, PixelFormat.Format32bppArgb);
    var target = result.LockBits(bounds, ImageLockMode.WriteOnly, PixelFormat.Format32bppArgb);
    for (var y = 0; y < height; y++) {
        Marshal.Copy(pixels, y * width, target.Scan0 + y * target.Stride, width);
    }
    result.UnlockBits(target);
    return result;
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
    [LibraryImport("user32.dll")]
    private static partial void PostQuitMessage(int exitCode);

    const uint EVENT_SYSTEM_FOREGROUND = 3;
    const uint WINEVENT_OUTOFCONTEXT = 0;
    const uint PM_NOREMOVE = 0;
    // WM_APP, which the watch's thread takes as a request to stop.
    const uint WM_STOP = 0x8000;

    // Windows calls OnForeground on the thread that watches, while it waits
    // for messages.
    [ThreadStatic]
    static FocusWatch? watching;

    readonly uint processId;
    // The program, which the watch stops as soon as it takes focus, so that
    // it has it for as short a time as can be.
    readonly Process? program;
    readonly Thread thread;
    readonly ManualResetEventSlim started = new();
    uint threadId;
    bool hooked;
    // The hook for the process's own windows.
    IntPtr ownHook;
    volatile bool tookFocus;
    bool disposed;

    public FocusWatch(Process program) {
        this.processId = (uint)program.Id;
        this.program = program;
        this.thread = new Thread(this.Watch) { IsBackground = true };
        this.thread.Start();
        this.started.Wait();
        if (!this.hooked) {
            this.thread.Join();
            throw new FailureException("Couldn't watch which window has focus");
        }
    }

    // Whether a window of the process had focus so far. After Dispose, it
    // includes every change that Windows reported before that.
    public bool TookFocus => this.tookFocus;

    void Watch() {
        watching = this;
        // Makes the thread's message queue, so that Dispose can post to it.
        _ = PeekMessageW(out _, IntPtr.Zero, 0, 0, PM_NOREMOVE);
        this.threadId = GetCurrentThreadId();
        // One hook reports every change, and Check looks up whose window it
        // is. That lookup fails for a window that's gone by then, like when
        // the program exits right after taking focus, so the other hook
        // reports only changes to the process's own windows, which need no
        // lookup.
        var hook = SetWinEventHook(EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_FOREGROUND, IntPtr.Zero, &OnForeground, 0, 0, WINEVENT_OUTOFCONTEXT);
        this.ownHook = SetWinEventHook(EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_FOREGROUND, IntPtr.Zero, &OnForeground, this.processId, 0, WINEVENT_OUTOFCONTEXT);
        this.hooked = hook != IntPtr.Zero && this.ownHook != IntPtr.Zero;
        this.started.Set();
        if (!this.hooked) {
            _ = UnhookWinEvent(hook);
            _ = UnhookWinEvent(this.ownHook);
            return;
        }
        // The window with focus before the first change.
        this.Check(IntPtr.Zero, ProgramWindow.Foreground());
        // Windows calls OnForeground from within GetMessageW, until Dispose
        // asks the thread to stop. Then PostQuitMessage, whose WM_QUIT
        // GetMessageW returns only once nothing else is waiting.
        while (GetMessageW(out var message, IntPtr.Zero, 0, 0) > 0) {
            if (message.message == WM_STOP) {
                PostQuitMessage(0);
            }
        }
        this.Check(IntPtr.Zero, ProgramWindow.Foreground());
        _ = UnhookWinEvent(hook);
        _ = UnhookWinEvent(this.ownHook);
    }

    [UnmanagedCallersOnly]
    static void OnForeground(IntPtr hook, uint winEvent, IntPtr window, int objectId, int childId, uint eventThread, uint time) {
        watching?.Check(hook, window);
    }

    void Check(IntPtr hook, IntPtr window) {
        if (hook == this.ownHook || ProgramWindow.Owner(window) == this.processId) {
            this.tookFocus = true;
            try {
                this.program?.Kill();
            } catch (Exception exception) when (exception is InvalidOperationException or Win32Exception) {
                // It had exited already.
            }
        }
    }

    // Stops watching, once the watch has handled every change of focus that
    // Windows reported until now. Windows calls OnForeground for those before
    // GetMessageW returns the message that this posts after them, and the
    // watch then quits with PostQuitMessage, as WM_QUIT's documentation asks,
    // instead of being posted WM_QUIT.
    public void Dispose() {
        if (this.disposed) {
            return;
        }
        this.disposed = true;
        _ = PostThreadMessageW(this.threadId, WM_STOP, IntPtr.Zero, IntPtr.Zero);
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
//
// Unless it's to keep running, the program starts in a job that ends it when
// this program exits, however that happens. Otherwise a run that's stopped
// partway, like by Ctrl+C or a coding agent's time limit, would leave it
// running on the display, drawing as fast as it can, until someone noticed.
//
// Run through RunLightly, this program is in its job, whose name
// KGAMES_LIGHT_JOB gives, and the processes it starts should be too: the
// build, and the program. Windows doesn't always put them there itself (see
// RunLightlyInJob.ps1), and rustup's cargo, once it runs, is in a job of its
// own that it can't be taken out of, so they're made with that job in their
// list of jobs, the program's before the job that ends it, which nests in
// it. A program that's to keep running also gets a handle to RunLightly's
// job, which it holds until it exits: the job ends its processes when its
// last handle closes, as RunLightly's does when it ends, so the program
// keeps running, with the job's limits, until it's stopped.
static partial class ProgramProcess {
    [StructLayout(LayoutKind.Sequential)]
    struct JOBOBJECT_EXTENDED_LIMIT_INFORMATION {
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
    private static partial bool CreateProcessW(string? application, [In, Out] char[] commandLine, IntPtr processAttributes, IntPtr threadAttributes, [MarshalAs(UnmanagedType.Bool)] bool inheritHandles, uint flags, IntPtr environment, string? directory, ref STARTUPINFOEX startupInfo, out PROCESS_INFORMATION information);
    [LibraryImport("kernel32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static partial bool CloseHandle(IntPtr handle);
    [LibraryImport("kernel32.dll", SetLastError = true)]
    private static partial IntPtr CreateJobObjectW(IntPtr attributes, IntPtr name);
    [LibraryImport("kernel32.dll", SetLastError = true, StringMarshalling = StringMarshalling.Utf16)]
    private static partial IntPtr OpenJobObjectW(uint access, [MarshalAs(UnmanagedType.Bool)] bool inherit, string name);
    [LibraryImport("kernel32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static partial bool SetInformationJobObject(IntPtr job, int informationClass, ref JOBOBJECT_EXTENDED_LIMIT_INFORMATION information, int size);
    [LibraryImport("kernel32.dll", SetLastError = true)]
    private static partial uint WaitForSingleObject(IntPtr handle, uint milliseconds);
    [LibraryImport("kernel32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static partial bool GetExitCodeProcess(IntPtr process, out int exitCode);

    const uint HANDLE_FLAG_INHERIT = 1;
    const int STARTF_USESTDHANDLES = 0x100;
    const uint EXTENDED_STARTUPINFO_PRESENT = 0x80000;
    // A program with a console, which none of the GUI programs have, gets
    // none, instead of a console window that could take focus.
    const uint CREATE_NO_WINDOW = 0x8000000;
    const int JobObjectExtendedLimitInformation = 9;
    const uint JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE = 0x2000;
    const uint JOB_OBJECT_ASSIGN_PROCESS = 0x1;
    const uint JOB_OBJECT_QUERY = 0x4;
    static readonly IntPtr PROC_THREAD_ATTRIBUTE_HANDLE_LIST = new(0x20002);
    static readonly IntPtr PROC_THREAD_ATTRIBUTE_JOB_LIST = new(0x2000D);

    // Its handle closes only when this program exits.
    static readonly Lazy<IntPtr> job = new(CreateJob);

    // RunLightly's job, if this program runs in one, with the access to put
    // processes in it, or zero. Its caller closes the handle as soon as it has
    // made its process: the job ends its processes when its last handle
    // closes, so one that this program kept would keep them all running past
    // RunLightly, if RunLightly were stopped.
    static IntPtr OpenLightJob() {
        if (Environment.GetEnvironmentVariable("KGAMES_LIGHT_JOB") is not { Length: > 0 } name) {
            return IntPtr.Zero;
        }
        var job = OpenJobObjectW(JOB_OBJECT_ASSIGN_PROCESS | JOB_OBJECT_QUERY, false, name);
        if (job == IntPtr.Zero) {
            throw new Win32Exception();
        }
        return job;
    }

    // Runs a command with this program's console and standard handles, in
    // RunLightly's job if this program is in one, and returns its exit code.
    // The command's first word is a program, which Windows finds as it does
    // for any command line.
    public static int RunInLightJob(List<string> words) {
        var commandLine = string.Join(' ', words.Select(Quote));
        var lightJob = OpenLightJob();
        var jobs = lightJob != IntPtr.Zero ? new[] { lightJob } : [];
        var size = IntPtr.Zero;
        _ = InitializeProcThreadAttributeList(IntPtr.Zero, 1, 0, ref size);
        var list = Marshal.AllocHGlobal(size);
        var jobList = Marshal.AllocHGlobal(IntPtr.Size * Math.Max(jobs.Length, 1));
        var initialized = false;
        try {
            if (!InitializeProcThreadAttributeList(list, 1, 0, ref size)) {
                throw new Win32Exception();
            }
            initialized = true;
            if (jobs.Length > 0) {
                Marshal.Copy(jobs, 0, jobList, jobs.Length);
                if (!UpdateProcThreadAttribute(list, 0, PROC_THREAD_ATTRIBUTE_JOB_LIST, jobList, new IntPtr(IntPtr.Size * jobs.Length), IntPtr.Zero, IntPtr.Zero)) {
                    throw new Win32Exception();
                }
            }
            var startupInfo = new STARTUPINFOEX {
                cb = Marshal.SizeOf<STARTUPINFOEX>(),
                lpAttributeList = list,
            };
            if (!CreateProcessW(null, (commandLine + '\0').ToCharArray(), IntPtr.Zero, IntPtr.Zero, true, EXTENDED_STARTUPINFO_PRESENT, IntPtr.Zero, null, ref startupInfo, out var information)) {
                throw new Win32Exception();
            }
            if (lightJob != IntPtr.Zero) {
                _ = CloseHandle(lightJob);
                lightJob = IntPtr.Zero;
            }
            _ = CloseHandle(information.hThread);
            _ = WaitForSingleObject(information.hProcess, uint.MaxValue);
            if (!GetExitCodeProcess(information.hProcess, out var exitCode)) {
                throw new Win32Exception();
            }
            _ = CloseHandle(information.hProcess);
            return exitCode;
        } finally {
            if (lightJob != IntPtr.Zero) {
                _ = CloseHandle(lightJob);
            }
            if (initialized) {
                DeleteProcThreadAttributeList(list);
            }
            Marshal.FreeHGlobal(jobList);
            Marshal.FreeHGlobal(list);
        }
    }

    // The word as a command line's, quoted as CommandLineToArgvW and C
    // runtimes read it.
    static string Quote(string word) {
        if (word.Length > 0 && word.IndexOfAny([' ', '\t', '\n', '\v', '"']) < 0) {
            return word;
        }
        var quoted = new System.Text.StringBuilder("\"");
        var backslashes = 0;
        foreach (var character in word) {
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

    static IntPtr CreateJob() {
        var job = CreateJobObjectW(IntPtr.Zero, IntPtr.Zero);
        if (job == IntPtr.Zero) {
            throw new Win32Exception();
        }
        var limits = new JOBOBJECT_EXTENDED_LIMIT_INFORMATION { LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE };
        if (!SetInformationJobObject(job, JobObjectExtendedLimitInformation, ref limits, Marshal.SizeOf<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>())) {
            throw new Win32Exception();
        }
        return job;
    }

    public static Process Start(string commandLine, string outputPath, string errorPath, bool keepRunning) {
        using var output = Open(outputPath);
        using var error = Open(errorPath);
        var handles = new List<IntPtr> { output.SafeFileHandle.DangerousGetHandle(), error.SafeFileHandle.DangerousGetHandle() };
        var jobs = new List<IntPtr>();
        var inherited = IntPtr.Zero;
        var lightJob = OpenLightJob();
        if (lightJob != IntPtr.Zero) {
            jobs.Add(lightJob);
            if (keepRunning) {
                // An inheritable handle of its own, so that the program gets
                // a copy.
                inherited = OpenJobObjectW(JOB_OBJECT_QUERY, true, Environment.GetEnvironmentVariable("KGAMES_LIGHT_JOB")!);
                if (inherited == IntPtr.Zero) {
                    throw new Win32Exception();
                }
                handles.Add(inherited);
            }
        }
        if (!keepRunning) {
            jobs.Add(job.Value);
        }
        var attributes = jobs.Count > 0 ? 2 : 1;
        var size = IntPtr.Zero;
        // This call only gets the size, and fails because the list is missing.
        _ = InitializeProcThreadAttributeList(IntPtr.Zero, attributes, 0, ref size);
        var list = Marshal.AllocHGlobal(size);
        var handleList = Marshal.AllocHGlobal(IntPtr.Size * handles.Count);
        var jobList = Marshal.AllocHGlobal(IntPtr.Size * Math.Max(jobs.Count, 1));
        var initialized = false;
        try {
            if (!InitializeProcThreadAttributeList(list, attributes, 0, ref size)) {
                throw new Win32Exception();
            }
            initialized = true;
            Marshal.Copy(handles.ToArray(), 0, handleList, handles.Count);
            if (!UpdateProcThreadAttribute(list, 0, PROC_THREAD_ATTRIBUTE_HANDLE_LIST, handleList, new IntPtr(IntPtr.Size * handles.Count), IntPtr.Zero, IntPtr.Zero)) {
                throw new Win32Exception();
            }
            if (jobs.Count > 0) {
                Marshal.Copy(jobs.ToArray(), 0, jobList, jobs.Count);
                if (!UpdateProcThreadAttribute(list, 0, PROC_THREAD_ATTRIBUTE_JOB_LIST, jobList, new IntPtr(IntPtr.Size * jobs.Count), IntPtr.Zero, IntPtr.Zero)) {
                    throw new Win32Exception();
                }
            }
            var startupInfo = new STARTUPINFOEX {
                cb = Marshal.SizeOf<STARTUPINFOEX>(),
                dwFlags = STARTF_USESTDHANDLES,
                hStdOutput = handles[0],
                hStdError = handles[1],
                lpAttributeList = list,
            };
            if (!CreateProcessW(null, (commandLine + '\0').ToCharArray(), IntPtr.Zero, IntPtr.Zero, true, EXTENDED_STARTUPINFO_PRESENT | CREATE_NO_WINDOW, IntPtr.Zero, null, ref startupInfo, out var information)) {
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
            if (lightJob != IntPtr.Zero) {
                _ = CloseHandle(lightJob);
            }
            if (inherited != IntPtr.Zero) {
                // The program has its own copy.
                _ = CloseHandle(inherited);
            }
            if (initialized) {
                DeleteProcThreadAttributeList(list);
            }
            Marshal.FreeHGlobal(jobList);
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
    [LibraryImport("user32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static partial bool GetWindowRect(IntPtr window, out RECT rect);
    [LibraryImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static partial bool ClientToScreen(IntPtr window, ref POINT point);
    [LibraryImport("dwmapi.dll")]
    private static partial int DwmGetWindowAttribute(IntPtr window, uint attribute, out int value, int size);

    [StructLayout(LayoutKind.Sequential)]
    struct POINT { public int X, Y; }

    const uint PW_CLIENTONLY = 1;
    // Asks the desktop compositor for the window's contents, so that
    // PrintWindow gets what Direct3D draws, which it doesn't without it.
    // WinUser.h defines it, but PrintWindow's documentation doesn't list it.
    // Windows.Graphics.Capture is the documented way to capture what a window
    // shows.
    const uint PW_RENDERFULLCONTENT = 2;
    const uint WM_CLOSE = 0x0010;
    const uint DWMWA_CLOAKED = 14;

    // The class of the windows that Library/UserInterface opens.
    public const string LibraryClass = "MainWindow";
    // The class of a WinUI 3 app's windows.
    public const string WinUiClass = "WinUIDesktopWin32WindowClass";

    // The program's window: a visible top-level window of the process, of the
    // class that its UI library registers. The process has other windows.
    public static IntPtr Find(int processId, string className) {
        var window = IntPtr.Zero;
        while ((window = FindWindowExW(IntPtr.Zero, window, className, null)) != IntPtr.Zero) {
            _ = GetWindowThreadProcessId(window, out var owner);
            if (owner == processId && IsWindowVisible(window)) {
                return window;
            }
        }
        return IntPtr.Zero;
    }

    // The process's other top-level windows that show, from the bottom to the
    // top, like a WinUI 3 app's open menus.
    public static List<IntPtr> Popups(int processId, IntPtr window) {
        var popups = new List<IntPtr>();
        var other = IntPtr.Zero;
        // FindWindowExW goes through the top-level windows from the top down.
        while ((other = FindWindowExW(IntPtr.Zero, other, null, null)) != IntPtr.Zero) {
            if (other == window || Owner(other) != processId || !IsWindowVisible(other)) {
                continue;
            }
            // A cloaked window is visible to Windows, but the compositor hides it.
            if (DwmGetWindowAttribute(other, DWMWA_CLOAKED, out var cloaked, sizeof(int)) == 0 && cloaked != 0) {
                continue;
            }
            popups.Add(other);
        }
        popups.Reverse();
        return popups;
    }

    // Where the window's contents start on the screen.
    public static (int X, int Y) ClientOrigin(IntPtr window) {
        var point = new POINT();
        _ = ClientToScreen(window, ref point);
        return (point.X, point.Y);
    }

    // The whole window's area on the screen.
    public static Rectangle Area(IntPtr window) {
        if (!GetWindowRect(window, out var rect)) {
            return Rectangle.Empty;
        }
        return Rectangle.FromLTRB(rect.Left, rect.Top, rect.Right, rect.Bottom);
    }

    // Draws the whole window, with its frame if it has one, into a device context.
    public static bool PrintWhole(IntPtr window, IntPtr dc) {
        return PrintWindow(window, dc, PW_RENDERFULLCONTENT);
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
