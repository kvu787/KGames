namespace DisplayIdentification;

internal static class SetPrimaryCommand {
    private static readonly string[] ValueOptions = ["--display"];
    private static readonly string[] FlagOptions = ["--check", "--help"];

    internal static int Run(IReadOnlyList<string> arguments) {
        ParsedArguments parsed = ParsedArguments.Parse(arguments, ValueOptions, FlagOptions, allowRemaining: false);
        if (parsed.HasFlag("--help")) {
            Program.ShowHelp();
            return 0;
        }

        string selector = parsed.GetValue("--display") ?? throw new CommandException("Choose a display with --display.");
        Display display = DisplaySelector.Resolve(selector, Display.GetAll());
        if (display.IsPrimary) {
            Log.Info($"{display.Key} is already the primary display.");
            return 0;
        }

        if (parsed.HasFlag("--check")) {
            PrimaryDisplay.Validate(display);
            Log.Info($"Windows can make {display.Key} the primary display. Nothing was changed.");
            return 0;
        }

        PrimaryDisplay.Set(display, persist: true);
        Log.Info($"{display.Key} is now the primary display.");
        return 0;
    }
}
