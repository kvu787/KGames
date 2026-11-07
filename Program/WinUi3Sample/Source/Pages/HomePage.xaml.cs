using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Navigation;

namespace WinUi3Sample;

public sealed partial class HomePage : Page {
    public HomePage() {
        this.InitializeComponent();
    }

    protected override void OnNavigatedTo(NavigationEventArgs e) {
        base.OnNavigatedTo(e);
        var hour = DateTime.Now.Hour;
        this.Greeting.Text = hour < 12 ? "Good morning" : hour < 18 ? "Good afternoon" : "Good evening";
        var pinned = App.State.Notes.Where(note => note.IsPinned).ToList();
        this.PinnedNotes.ItemsSource = pinned;
        this.NoPinnedNotes.Visibility = pinned.Count == 0 ? Visibility.Visible : Visibility.Collapsed;
        this.RecentNotes.ItemsSource = App.State.Notes.OrderByDescending(note => note.Modified).Take(5).ToList();
    }

    void OnNoteClick(object sender, ItemClickEventArgs args) {
        App.Window.ShowNote((Note)args.ClickedItem);
    }

    void OnNewNote(object sender, RoutedEventArgs args) {
        App.Window.NewNote();
    }

    void OnOpen(object sender, RoutedEventArgs args) {
        _ = App.Window.OpenAsync();
    }

    void OnShowControls(object sender, RoutedEventArgs args) {
        App.Window.Navigate("Controls");
    }

    void OnShowSettings(object sender, RoutedEventArgs args) {
        App.Window.Navigate("Settings");
    }
}
