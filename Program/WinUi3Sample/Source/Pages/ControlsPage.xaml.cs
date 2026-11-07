using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Controls.Primitives;
using Microsoft.UI.Xaml.Media;

namespace WinUi3Sample;

// WinUI 3's common controls, as they come.
public sealed partial class ControlsPage : Page {
    static readonly string[] Fruits = [
        "Apple", "Apricot", "Banana", "Blackberry", "Blueberry", "Cherry", "Grape", "Grapefruit",
        "Kiwi", "Lemon", "Lime", "Mango", "Melon", "Orange", "Peach", "Pear", "Pineapple", "Plum",
        "Raspberry", "Strawberry", "Watermelon",
    ];

    int clicks;
    int repeats;

    public ControlsPage() {
        this.InitializeComponent();
        this.Breadcrumbs.ItemsSource = new[] { "Home", "Documents", "Notes", "Trip to Kyoto" };
    }

    void OnCountedClick(object sender, RoutedEventArgs args) {
        this.clicks++;
        this.ClickCount.Text = this.clicks == 1 ? "Pressed once" : $"Pressed {this.clicks} times";
    }

    void OnSplitClick(SplitButton sender, SplitButtonClickEventArgs args) {
        this.OnCountedClick(sender, new RoutedEventArgs());
    }

    void OnRepeat(object sender, RoutedEventArgs args) {
        this.repeats++;
        ((RepeatButton)sender).Content = $"Repeated {this.repeats} times";
    }

    void OnFruitTextChanged(AutoSuggestBox sender, AutoSuggestBoxTextChangedEventArgs args) {
        if (args.Reason == AutoSuggestionBoxTextChangeReason.UserInput) {
            var text = sender.Text.Trim();
            sender.ItemsSource = text.Length == 0
                ? null
                : Fruits.Where(fruit => fruit.Contains(text, StringComparison.CurrentCultureIgnoreCase)).ToList();
        }
    }

    async void OnShowDialog(object sender, RoutedEventArgs args) {
        var result = await App.Window.ShowDialogAsync(new ContentDialog {
            Title = "Save your changes?",
            Content = "A content dialog asks something before the app goes on, over a smoke layer that dims the window.",
            PrimaryButtonText = "Save",
            SecondaryButtonText = "Don't save",
            CloseButtonText = "Cancel",
            DefaultButton = ContentDialogButton.Primary,
        });
        App.State.StatusMessage = result switch {
            ContentDialogResult.Primary => "The dialog said Save",
            ContentDialogResult.Secondary => "The dialog said Don't save",
            _ => "The dialog was canceled",
        };
    }

    void OnShowTip(object sender, RoutedEventArgs args) {
        this.Tip.IsOpen = true;
    }

    void OnAddTab(TabView sender, object args) {
        var number = sender.TabItems.Count + 1;
        var tab = new TabViewItem {
            Header = $"Tab {number}",
            IconSource = new FontIconSource { Glyph = "" },
            Content = new TextBlock { Text = $"Tab number {number}.", Margin = new Thickness(16) },
        };
        sender.TabItems.Add(tab);
        sender.SelectedItem = tab;
    }

    void OnCloseTab(TabView sender, TabViewTabCloseRequestedEventArgs args) {
        sender.TabItems.Remove(args.Tab);
    }

    void OnColorChanged(ColorPicker sender, ColorChangedEventArgs args) {
        this.ColorSwatch.Fill = new SolidColorBrush(args.NewColor);
    }
}
