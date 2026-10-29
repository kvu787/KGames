namespace DisplayIdentification;

// An error the user can fix, such as a wrong argument. Reported without a stack trace.
internal sealed class CommandException(string message) : Exception(message);
