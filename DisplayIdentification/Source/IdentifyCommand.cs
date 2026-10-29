using System.Windows.Forms;

namespace DisplayIdentification;

// Shows a card on each display with its key, so that it is clear which key belongs to which display.
internal static class IdentifyCommand {
    private const int DefaultSeconds = 15;
    private static readonly string[] ValueOptions = ["--seconds"];
    private static readonly string[] FlagOptions = ["--help"];

    internal static int Run(IReadOnlyList<string> arguments) {
        ParsedArguments parsed = ParsedArguments.Parse(arguments, ValueOptions, FlagOptions, allowRemaining: false);
        if (parsed.HasFlag("--help")) {
            Program.ShowHelp();
            return 0;
        }

        int seconds = parsed.GetSeconds("--seconds") ?? DefaultSeconds;
        if (seconds < 1) {
            throw new CommandException("The option --seconds needs at least 1 second.");
        }

        List<Display> displays = Display.GetAll();
        ListCommand.WriteTable(displays);

        // Mirrored displays share a desktop area, so they share a card.
        List<IdentifyForm> forms = [];
        foreach (IGrouping<string, Display> group in displays.GroupBy(display => display.DeviceName, StringComparer.OrdinalIgnoreCase)) {
            DisplayMonitor? monitor = DisplayMonitor.Find(group.Key);
            if (monitor != null) {
                forms.Add(new IdentifyForm([.. group], monitor.Bounds));
            }
        }

        if (forms.Count > 0) {
            using IdentifyContext context = new(forms, TimeSpan.FromSeconds(seconds));
            Application.Run(context);
        }

        return 0;
    }
}
