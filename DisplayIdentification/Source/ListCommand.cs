using System.Text.Encodings.Web;
using System.Text.Json;

namespace DisplayIdentification;

internal static class ListCommand {
    private static readonly string[] ValueOptions = [];
    private static readonly string[] FlagOptions = ["--json", "--help"];
    private static readonly JsonSerializerOptions JsonOptions = new() {
        // The output is for reading and scripts, not HTML, so keep characters such as & in device paths readable.
        Encoder = JavaScriptEncoder.UnsafeRelaxedJsonEscaping,
        WriteIndented = true,
    };

    internal static int Run(IReadOnlyList<string> arguments) {
        ParsedArguments parsed = ParsedArguments.Parse(arguments, ValueOptions, FlagOptions, allowRemaining: false);
        if (parsed.HasFlag("--help")) {
            Program.ShowHelp();
            return 0;
        }

        List<Display> displays = Display.GetAll();
        if (parsed.HasFlag("--json")) {
            Console.Out.WriteLine(JsonSerializer.Serialize(displays, JsonOptions));
        } else {
            WriteTable(displays);
        }

        return 0;
    }

    internal static void WriteTable(IReadOnlyList<Display> displays) {
        List<string[]> rows = [["Key", "Name", "Device", "Resolution", "Position", "Primary"]];
        foreach (Display display in displays) {
            rows.Add([display.Key, display.Name, display.ShortDeviceName, $"{display.Width}x{display.Height}", $"{display.X},{display.Y}", display.IsPrimary ? "Yes" : ""]);
        }

        int[] widths = new int[rows[0].Length];
        foreach (string[] row in rows) {
            for (int column = 0; column < row.Length; column++) {
                widths[column] = Math.Max(widths[column], row[column].Length);
            }
        }

        foreach (string[] row in rows) {
            Console.Out.WriteLine(string.Join("  ", row.Select((cell, column) => cell.PadRight(widths[column]))).TrimEnd());
        }
    }
}
