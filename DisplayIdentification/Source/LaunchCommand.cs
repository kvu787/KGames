using System.Diagnostics;
using System.Runtime.InteropServices;
using Microsoft.Win32.SafeHandles;

namespace DisplayIdentification;

// Starts a program on a display, and waits until it exits.
internal static class LaunchCommand {
    private const int DefaultWatchSeconds = 30;
    private static readonly string[] ValueOptions = ["--display", "--method", "--watch-seconds", "--restore-delay", "--process-name", "--working-directory"];
    private static readonly string[] FlagOptions = ["--help"];

    // Time for Windows and the displays to settle after a primary display change, before the program asks where to open.
    private static readonly TimeSpan SettleTime = TimeSpan.FromSeconds(1);
    private static readonly TimeSpan WatchInterval = TimeSpan.FromMilliseconds(200);
    private static readonly TimeSpan WaitInterval = TimeSpan.FromSeconds(1);

    internal static int Run(IReadOnlyList<string> arguments) {
        ParsedArguments parsed = ParsedArguments.Parse(arguments, ValueOptions, FlagOptions, allowRemaining: true);
        if (parsed.HasFlag("--help")) {
            Program.ShowHelp();
            return 0;
        }

        LaunchOptions options = CreateOptions(parsed);
        Log.OpenFile(Path.Combine(ApplicationData.DirectoryPath, "LastLaunch.log"));
        Log.Info($"Launching {string.Join(' ', [Launcher.QuoteArgument(options.FileName), .. options.Arguments.Select(Launcher.QuoteArgument)])}");
        PendingRestore.RecoverInterruptedLaunch();

        List<Display> displays = Display.GetAll();
        Display target = DisplaySelector.Resolve(options.Display, displays);
        Log.Info($"Target display: {target.Key} ({target.Name}, {target.ShortDeviceName}).");

        using TemporaryPrimaryDisplay? temporaryPrimary = options.Method == LaunchMethod.Primary && !target.IsPrimary ? MakePrimary(target, displays) : null;
        return Launch(options, target, temporaryPrimary);
    }

    private static LaunchOptions CreateOptions(ParsedArguments parsed) {
        string display = parsed.GetValue("--display")
            ?? throw new CommandException("Choose a display with --display. Run \"DisplayIdentification list\" to see the displays.");
        if (parsed.Remaining.Count == 0) {
            throw new CommandException("Give the program to launch after the options.");
        }

        LaunchMethod method = parsed.GetValue("--method")?.ToLowerInvariant() switch {
            null or "move" => LaunchMethod.Move,
            "primary" => LaunchMethod.Primary,
            string other => throw new CommandException($"Unknown method \"{other}\". Use move or primary."),
        };
        int? restoreDelay = parsed.GetSeconds("--restore-delay");
        if (restoreDelay != null && method != LaunchMethod.Primary) {
            throw new CommandException("The option --restore-delay only applies to --method primary.");
        }

        string? workingDirectory = parsed.GetValue("--working-directory");
        return new LaunchOptions {
            Display = display,
            Method = method,
            WatchDuration = TimeSpan.FromSeconds(parsed.GetSeconds("--watch-seconds") ?? DefaultWatchSeconds),
            RestoreDelay = restoreDelay != null ? TimeSpan.FromSeconds(restoreDelay.Value) : null,
            ProcessNames = parsed.GetValues("--process-name"),
            WorkingDirectory = workingDirectory != null ? Path.GetFullPath(workingDirectory) : null,
            FileName = parsed.Remaining[0],
            Arguments = parsed.Remaining[1..],
        };
    }

    private static TemporaryPrimaryDisplay MakePrimary(Display target, List<Display> displays) {
        Display original = displays.Find(display => display.IsPrimary) ?? throw new CommandException("Windows didn't report a primary display.");
        Log.Info($"Making {target.Key} the primary display while the program runs, instead of {original.Key}.");
        TemporaryPrimaryDisplay temporaryPrimary = new(original, target);
        Thread.Sleep(SettleTime);
        return temporaryPrimary;
    }

    private static int Launch(LaunchOptions options, Display target, TemporaryPrimaryDisplay? temporaryPrimary) {
        using ManualResetEventSlim stopRequested = new();
        using PosixSignalRegistration interrupt = PosixSignalRegistration.Create(PosixSignal.SIGINT, context => {
            context.Cancel = true;
            stopRequested.Set();
        });

        // When the console window closes, or the user signs out, the process ends right after the handler, so restore right away.
        using PosixSignalRegistration close = PosixSignalRegistration.Create(PosixSignal.SIGHUP, context => temporaryPrimary?.Restore());
        using PosixSignalRegistration terminate = PosixSignalRegistration.Create(PosixSignal.SIGTERM, context => temporaryPrimary?.Restore());

        using ProcessTree processes = new(options.ProcessNames);
        DisplayMonitor? monitor = DisplayMonitor.Find(target.DeviceName);
        SafeProcessHandle? process = Launcher.Start(options.FileName, options.Arguments, options.WorkingDirectory ?? DefaultWorkingDirectory(options.FileName), monitor?.Handle ?? 0);
        if (process != null) {
            Log.Info($"Started process {processes.AddRoot(process)}.");
        } else if (options.ProcessNames.Count == 0) {
            Log.Warning("Windows didn't report a process for the program, so its windows can't be found. Name its process with --process-name.");
        }

        WindowMover mover = new();
        Stopwatch clock = Stopwatch.StartNew();
        while (true) {
            processes.Refresh();
            bool watching = clock.Elapsed < options.WatchDuration;
            if (watching) {
                mover.MoveWindows(target.DeviceName, processes.GetRunningProcessIds());
            }

            if (clock.Elapsed >= options.RestoreDelay) {
                temporaryPrimary?.Restore();
            }

            if (processes.HasExited(startupOver: !watching)) {
                break;
            }

            if (stopRequested.Wait(watching ? WatchInterval : WaitInterval)) {
                Log.Info("Stopped waiting for the program.");
                return 1;
            }
        }

        int? exitCode = processes.GetExitCode();
        Log.Info(exitCode != null ? $"The program exited with code {exitCode}." : "The program exited.");
        return exitCode ?? 0;
    }

    // Games often expect to start in their own folder, as they do from their own shortcuts.
    private static string? DefaultWorkingDirectory(string fileName) {
        return File.Exists(fileName) && string.Equals(Path.GetExtension(fileName), ".exe", StringComparison.OrdinalIgnoreCase)
            ? Path.GetDirectoryName(Path.GetFullPath(fileName))
            : null;
    }
}
