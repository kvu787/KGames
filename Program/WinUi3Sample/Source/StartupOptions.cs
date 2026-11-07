using Microsoft.UI.Xaml;

namespace WinUi3Sample;

// The command line's options. RunInBackground opens the app with --display and --no-activate, and
// --page, --theme and --open-menu let it take screenshots of the rest.
public sealed record StartupOptions(string? Display, bool NoActivate, string? Page, ElementTheme? Theme, string? OpenMenu) {
    public const string Usage = """
        Usage: WinUi3Sample.exe [options]

          --display <identifier>  The display to open on, as IdentifyDisplays --list shows it,
                                  or Primary
          --no-activate           Opens beneath the other windows, without taking focus
          --page <page>           The page to open on: Home, Notes, Controls or Settings
          --theme <theme>         Light, Dark or System. Light by default
          --open-menu <menu>      Opens a menu of the menu bar, like File, once the window shows
        """;

    public static readonly string[] Pages = ["Home", "Notes", "Controls", "Settings"];

    public static StartupOptions Parse(IEnumerable<string> arguments) {
        var options = new StartupOptions(null, false, null, null, null);
        using var argument = arguments.GetEnumerator();
        while (argument.MoveNext()) {
            switch (argument.Current) {
            case "--display":
                options = options with { Display = Value(argument, "--display") };
                break;
            case "--no-activate":
                options = options with { NoActivate = true };
                break;
            case "--page":
                var page = Value(argument, "--page");
                var match = Pages.FirstOrDefault(name => name.Equals(page, StringComparison.OrdinalIgnoreCase))
                    ?? throw new StartupException($"There's no page {page}. The pages are {string.Join(", ", Pages)}");
                options = options with { Page = match };
                break;
            case "--theme":
                var theme = Value(argument, "--theme");
                options = options with {
                    Theme = theme.ToLowerInvariant() switch {
                        "light" => ElementTheme.Light,
                        "dark" => ElementTheme.Dark,
                        "system" => ElementTheme.Default,
                        _ => throw new StartupException($"--theme takes Light, Dark or System, not {theme}"),
                    },
                };
                break;
            case "--open-menu":
                options = options with { OpenMenu = Value(argument, "--open-menu") };
                break;
            default:
                throw new StartupException($"Unknown argument {argument.Current}\n\n{Usage}");
            }
        }
        return options;
    }

    static string Value(IEnumerator<string> argument, string option) {
        if (!argument.MoveNext()) {
            throw new StartupException($"{option} needs a value after it");
        }
        return argument.Current;
    }
}

// Why the app couldn't start, to show without a stack trace.
public sealed class StartupException(string message) : Exception(message);
