using System.Reflection;

namespace WinUi3Sample;

public static class Versions {
    // The Windows App SDK's version, from the project's package reference.
    public static string WindowsAppSdk { get; } =
        typeof(Versions).Assembly.GetCustomAttributes<AssemblyMetadataAttribute>()
            .FirstOrDefault(attribute => attribute.Key == "WindowsAppSdkVersion")?.Value ?? "unknown";

    // Like "Windows App SDK 2.6.5, .NET 10.0.14, Windows 10.0.26300".
    public static string Summary =>
        $"Windows App SDK {WindowsAppSdk}, .NET {Environment.Version}, Windows {Environment.OSVersion.Version.ToString(3)}";
}
