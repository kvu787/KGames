using System.Windows.Forms;

namespace DisplayIdentification;

internal static class Program {
    private const string HelpText = """
        DisplayIdentification: identifies displays by keys that stay the same across restarts and ports.

        Usage:
          DisplayIdentification list [--json]
          DisplayIdentification identify [--seconds <seconds>]
          DisplayIdentification set-primary --display <display> [--check]
          DisplayIdentification restore

        Commands:
          list          List the active displays and their keys.
          identify      Show each display's key on the display itself. Default: 15 seconds.
          set-primary   Make a display the primary display. With --check, only check that
                        Windows accepts the change.
          restore       Undo a temporary primary display change that an interrupted launch
                        left behind.

        A <display> can be any of:
          primary                The current primary display.
          AUSAA0F-TCLMSV002189   A key from the list command. Recommended: it stays the same
                                 across restarts, ports and Windows display numbers.
          TCLMSV002189           A serial number.
          PA278QGV               A name or model, if only one display has it.
          DISPLAY5               A Windows display device name. These change over time.

        """;

    [STAThread]
    private static int Main(string[] arguments) {
        // Work in physical pixels on every display.
        _ = Application.SetHighDpiMode(HighDpiMode.PerMonitorV2);
        Application.EnableVisualStyles();
        try {
            return Run(arguments);
        } catch (CommandException exception) {
            ReportError(exception.Message);
            return 1;
        } catch (Exception exception) {
            ReportError(exception.ToString());
            return 1;
        }
    }

    internal static void ShowHelp() {
        if (Log.HasConsole) {
            Console.Out.Write(HelpText);
        } else {
            _ = MessageBox.Show(HelpText, "DisplayIdentification", MessageBoxButtons.OK, MessageBoxIcon.Information);
        }
    }

    private static int Run(string[] arguments) {
        if (arguments.Length == 0 || arguments[0] is "help" or "--help" or "-h" or "/?") {
            ShowHelp();
            return 0;
        }

        string[] commandArguments = arguments[1..];
        return arguments[0].ToLowerInvariant() switch {
            "list" => ListCommand.Run(commandArguments),
            "identify" => IdentifyCommand.Run(commandArguments),
            "set-primary" => SetPrimaryCommand.Run(commandArguments),
            "restore" => RestoreCommand.Run(commandArguments),
            _ => throw new CommandException($"Unknown command \"{arguments[0]}\". Run \"DisplayIdentification help\" to see the commands."),
        };
    }

    // Started from a shortcut or Steam there is no console, so errors also show in a message box.
    private static void ReportError(string message) {
        Log.Error(message);
        if (!Log.HasConsole) {
            _ = MessageBox.Show(message, "DisplayIdentification", MessageBoxButtons.OK, MessageBoxIcon.Error);
        }
    }
}
