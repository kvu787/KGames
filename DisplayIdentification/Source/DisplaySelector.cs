namespace DisplayIdentification;

// Finds the display that the user means. A display can be given as "primary", or by its key, model, serial number, name, device
// name or device path.
internal static class DisplaySelector {
    internal static Display Resolve(string selector, IReadOnlyList<Display> displays) {
        string value = selector.Trim();
        List<Display> matches = string.Equals(value, "primary", StringComparison.OrdinalIgnoreCase)
            ? [.. displays.Where(display => display.IsPrimary)]
            : [.. displays.Where(display => Matches(display, value))];

        if (matches.Count == 0) {
            throw new CommandException($"No connected display matches \"{value}\". The connected displays are:{Environment.NewLine}{Describe(displays)}");
        }

        // Mirrored displays show the same desktop area, so any of them will do.
        return matches.TrueForAll(match => match.HasSameSource(matches[0]))
            ? matches[0]
            : throw new CommandException($"More than one display matches \"{value}\". Use one of these keys instead:{Environment.NewLine}{Describe(matches)}");
    }

    private static bool Matches(Display display, string value) {
        string[] identifiers = [display.Key, display.Model, display.SerialNumber, display.Name, display.DeviceName, display.ShortDeviceName, display.DevicePath];
        return identifiers.Any(identifier => identifier.Length > 0 && string.Equals(identifier, value, StringComparison.OrdinalIgnoreCase));
    }

    private static string Describe(IEnumerable<Display> displays) {
        return string.Join(Environment.NewLine, displays.Select(display => $"  {display.Key} ({display.Name}, {display.ShortDeviceName}{(display.IsPrimary ? ", primary" : "")})"));
    }
}
