namespace DisplayIdentification;

// Writes messages to the console, if there is one, and to a log file, if one is open.
internal static class Log {
    private static readonly Lock sync = new();
    private static StreamWriter? file;

    // Whether messages can reach the user through a console or redirected output.
    internal static bool HasConsole => NativeMethods.GetConsoleWindow() != 0 || Console.IsOutputRedirected || Console.IsErrorRedirected;

    internal static void OpenFile(string path) {
        lock (sync) {
            try {
                _ = Directory.CreateDirectory(Path.GetDirectoryName(path)!);
                FileStream stream = new(path, FileMode.Create, FileAccess.Write, FileShare.ReadWrite);
                file = new StreamWriter(stream) {
                    AutoFlush = true,
                };
            } catch (IOException exception) {
                Console.Error.WriteLine($"Warning: Could not open the log file {path}: {exception.Message}");
            }
        }
    }

    internal static void Info(string message) {
        Write(Console.Out, "", message);
    }

    internal static void Warning(string message) {
        Write(Console.Error, "Warning: ", message);
    }

    internal static void Error(string message) {
        Write(Console.Error, "Error: ", message);
    }

    private static void Write(TextWriter console, string prefix, string message) {
        lock (sync) {
            console.WriteLine(prefix + message);
            file?.WriteLine($"{DateTime.Now:yyyy-MM-dd HH:mm:ss.fff} {prefix}{message}");
        }
    }
}
