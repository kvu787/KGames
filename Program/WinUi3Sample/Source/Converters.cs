using Microsoft.UI.Xaml;

namespace WinUi3Sample;

// Functions for x:Bind, which calls them again when their arguments change.
public static class Converters {
    public static bool IsNotNull(object? value) => value != null;

    public static Visibility VisibleIfNotNull(object? value) => value != null ? Visibility.Visible : Visibility.Collapsed;

    public static Visibility VisibleIfNull(object? value) => value == null ? Visibility.Visible : Visibility.Collapsed;
}
