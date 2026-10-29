namespace DisplayIdentification;

internal enum LaunchMethod {
    // Move the program's windows to the display.
    Move,

    // Also make the display the primary display while the program runs.
    Primary,
}

internal sealed record LaunchOptions {
    public required string Display { get; init; }
    public required LaunchMethod Method { get; init; }
    public required TimeSpan WatchDuration { get; init; }
    public required TimeSpan? RestoreDelay { get; init; }
    public required IReadOnlyList<string> ProcessNames { get; init; }
    public required string? WorkingDirectory { get; init; }

    // A program, shortcut, document or URL.
    public required string FileName { get; init; }

    public required IReadOnlyList<string> Arguments { get; init; }
}
