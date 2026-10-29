using System.ComponentModel;
using System.Runtime.InteropServices;
using System.Text;
using Microsoft.Win32.SafeHandles;

namespace DisplayIdentification;

// Starts programs the way Explorer does, so that shortcuts, URLs such as steam://, documents and programs that need
// administrator rights all work.
internal static unsafe class Launcher {
    // The monitor is a hint for where the program's first window opens. Windows applies it to windows that open at the default
    // position, and programs can read it with GetStartupInfo. Returns the process, if Windows reports one.
    internal static SafeProcessHandle? Start(string fileName, IReadOnlyList<string> arguments, string? workingDirectory, nint monitor) {
        string parameters = string.Join(' ', arguments.Select(QuoteArgument));
        nint process;
        fixed (char* file = fileName) {
            fixed (char* parameterText = parameters) {
                fixed (char* directory = workingDirectory) {
                    NativeMethods.ShellExecuteInfo info = new() {
                        Size = (uint)sizeof(NativeMethods.ShellExecuteInfo),
                        Mask = NativeMethods.SEE_MASK_NOCLOSEPROCESS | NativeMethods.SEE_MASK_NOASYNC | NativeMethods.SEE_MASK_FLAG_NO_UI | (monitor != 0 ? NativeMethods.SEE_MASK_HMONITOR : 0),
                        File = file,
                        Parameters = parameters.Length > 0 ? parameterText : null,
                        Directory = directory,
                        Show = NativeMethods.SW_SHOWNORMAL,
                        Monitor = monitor,
                    };
                    if (!NativeMethods.ShellExecuteEx(ref info)) {
                        throw new CommandException($"Couldn't start \"{fileName}\": {new Win32Exception(Marshal.GetLastPInvokeError()).Message}");
                    }

                    process = info.Process;
                }
            }
        }

        return process != 0 ? new SafeProcessHandle(process, ownsHandle: true) : null;
    }

    // Quotes an argument so that programs that split their command line the usual way (CommandLineToArgvW and the C runtime)
    // get it back unchanged.
    internal static string QuoteArgument(string argument) {
        if (argument.Length > 0 && argument.IndexOfAny([' ', '\t', '\n', '\v', '"']) < 0) {
            return argument;
        }

        StringBuilder quoted = new("\"");
        int backslashes = 0;
        foreach (char character in argument) {
            if (character == '\\') {
                backslashes++;
                continue;
            }

            // Backslashes are only special in front of a quote.
            int escapedBackslashes = character == '"' ? (backslashes * 2) + 1 : backslashes;
            _ = quoted.Append('\\', escapedBackslashes).Append(character);
            backslashes = 0;
        }

        // Double the backslashes in front of the closing quote.
        return quoted.Append('\\', backslashes * 2).Append('"').ToString();
    }
}
