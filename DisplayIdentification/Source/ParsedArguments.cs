using System.Globalization;

namespace DisplayIdentification;

// Command-line options in the forms "--name value", "--name=value" and "--flag".
internal sealed class ParsedArguments {
    private readonly Dictionary<string, List<string>> values = new(StringComparer.Ordinal);
    private readonly HashSet<string> flags = new(StringComparer.Ordinal);

    // The arguments after the options, when allowed.
    internal List<string> Remaining { get; } = [];

    // With allowRemaining, the options end at "--" or at the first argument that isn't an option.
    internal static ParsedArguments Parse(IReadOnlyList<string> arguments, string[] valueOptions, string[] flagOptions, bool allowRemaining) {
        ParsedArguments parsed = new();
        int index = 0;
        while (index < arguments.Count) {
            string argument = arguments[index];
            if (allowRemaining && argument == "--") {
                index++;
                break;
            }

            if (!argument.StartsWith("--", StringComparison.Ordinal)) {
                if (allowRemaining) {
                    break;
                }

                throw new CommandException($"Unexpected argument \"{argument}\".");
            }

            index++;
            int equalsIndex = argument.IndexOf('=');
            string name = equalsIndex >= 0 ? argument[..equalsIndex] : argument;
            if (flagOptions.Contains(name)) {
                if (equalsIndex >= 0) {
                    throw new CommandException($"The option {name} doesn't take a value.");
                }

                _ = parsed.flags.Add(name);
            } else if (valueOptions.Contains(name)) {
                string value;
                if (equalsIndex >= 0) {
                    value = argument[(equalsIndex + 1)..];
                } else if (index < arguments.Count) {
                    value = arguments[index];
                    index++;
                } else {
                    throw new CommandException($"The option {name} needs a value.");
                }

                if (!parsed.values.TryGetValue(name, out List<string>? list)) {
                    list = [];
                    parsed.values[name] = list;
                }

                list.Add(value);
            } else {
                throw new CommandException($"Unknown option \"{name}\".");
            }
        }

        for (; index < arguments.Count; index++) {
            parsed.Remaining.Add(arguments[index]);
        }

        return parsed;
    }

    internal bool HasFlag(string name) {
        return this.flags.Contains(name);
    }

    // The last value given for the option.
    internal string? GetValue(string name) {
        return this.values.TryGetValue(name, out List<string>? list) ? list[^1] : null;
    }

    internal List<string> GetValues(string name) {
        return this.values.TryGetValue(name, out List<string>? list) ? list : [];
    }

    internal int? GetSeconds(string name) {
        string? value = this.GetValue(name);
        return value switch {
            null => null,
            _ when int.TryParse(value, NumberStyles.None, CultureInfo.InvariantCulture, out int seconds) => seconds,
            _ => throw new CommandException($"The option {name} needs a whole number of seconds, not \"{value}\"."),
        };
    }
}
