namespace DisplayIdentification;

// Undoes the temporary primary display change of a launch that was interrupted, for example by a crash.
internal static class RestoreCommand {
    private static readonly string[] ValueOptions = [];
    private static readonly string[] FlagOptions = ["--help"];

    internal static int Run(IReadOnlyList<string> arguments) {
        ParsedArguments parsed = ParsedArguments.Parse(arguments, ValueOptions, FlagOptions, allowRemaining: false);
        if (parsed.HasFlag("--help")) {
            Program.ShowHelp();
            return 0;
        }

        PendingRestore? pending = PendingRestore.Load();
        if (pending == null) {
            Log.Info("There is no primary display change to undo.");
            return 0;
        }

        Display original = pending.FindOriginal(Display.GetAll())
            ?? throw new CommandException($"The original primary display {pending.OriginalKey} isn't connected.");
        if (!original.IsPrimary) {
            PrimaryDisplay.Set(original, persist: false);
        }

        PendingRestore.Delete();
        Log.Info($"{original.Key} is the primary display again.");
        return 0;
    }
}
