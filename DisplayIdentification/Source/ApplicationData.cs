namespace DisplayIdentification;

internal static class ApplicationData {
    // %LOCALAPPDATA%\DisplayIdentification
    internal static string DirectoryPath => Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "DisplayIdentification");
}
