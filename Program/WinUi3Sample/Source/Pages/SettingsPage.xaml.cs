using Microsoft.UI.Xaml.Controls;

namespace WinUi3Sample;

// The settings, bound two ways to AppState, which the window applies as they change.
public sealed partial class SettingsPage : Page {
    public SettingsPage() {
        this.InitializeComponent();
        this.VersionText.Text = Versions.Summary;
    }

    public AppState State => App.State;

    public static string Percent(int value) => $"{value}%";
}
