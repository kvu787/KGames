using System.ComponentModel;
using Microsoft.UI;
using Microsoft.UI.Composition.SystemBackdrops;
using Microsoft.UI.Text;
using Microsoft.UI.Windowing;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Automation.Peers;
using Microsoft.UI.Xaml.Automation.Provider;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Input;
using Microsoft.UI.Xaml.Media;
using Microsoft.UI.Xaml.Media.Animation;
using Microsoft.UI.Xaml.Media.Imaging;
using Microsoft.UI.Xaml.Navigation;
using Microsoft.Windows.Storage.Pickers;
using Windows.Graphics;
using Windows.System;

namespace WinUi3Sample;

// The app's one window: its title bar, menu bar, navigation, pages and status bar. The menus and
// their keyboard accelerators act through the methods here, which the pages use too.
public sealed partial class MainWindow : Window {
    static readonly Dictionary<string, Type> PageTypes = new() {
        ["Home"] = typeof(HomePage),
        ["Notes"] = typeof(NotesPage),
        ["Controls"] = typeof(ControlsPage),
        ["Settings"] = typeof(SettingsPage),
    };

    const string DocumentationUri = "https://learn.microsoft.com/windows/apps/winui/winui3/";

    readonly AppState state = App.State;
    // The presenter of a window with a frame, which full screen switches away from and back to.
    readonly OverlappedPresenter overlapped;
    // The text box that the Edit menu works on: the last one that had focus, as opening a menu
    // takes focus away from it.
    TextBox? lastTextBox;
    Note? watchedNote;
    bool dialogOpen;

    public MainWindow() {
        this.InitializeComponent();

        // The app draws the title bar itself, with WinUI 3's TitleBar control, over the Mica
        // backdrop. Windows still draws the minimize, maximize and close buttons.
        this.ExtendsContentIntoTitleBar = true;
        this.SetTitleBar(this.AppTitleBar);
        this.AppWindow.TitleBar.PreferredHeightOption = TitleBarHeightOption.Tall;
        this.AppWindow.SetIcon(Path.Combine(AppContext.BaseDirectory, "Assets", "AppIcon.ico"));
        this.overlapped = (OverlappedPresenter)this.AppWindow.Presenter;
        this.AppWindow.Changed += this.OnAppWindowChanged;

        this.AddAccelerators();
        this.Root.GotFocus += (_, args) => {
            if (args.OriginalSource is TextBox box) {
                this.lastTextBox = box;
            }
        };
        this.Root.ActualThemeChanged += (_, _) => this.UpdateStatusBar();
        this.ContentFrame.Navigated += this.OnNavigated;
        this.state.PropertyChanged += this.OnStateChanged;
        this.state.Notes.CollectionChanged += (_, _) => this.UpdateNotesBadge();

        this.ApplyTheme();
        this.ApplyNavigationStyle();
        this.WatchSelectedNote();
        this.UpdateNotesBadge();
        this.SyncMenus();
        this.UpdateStatusBar();
    }

    // Shows the window maximized, on the display if there's one, like the other KGames programs.
    public void Open(Displays.Display? display, StartupOptions options) {
        this.Navigate(options.Page ?? "Home");
        var workArea = display?.WorkArea ?? this.CurrentWorkArea();
        // The size that the window restores to, in the middle of the work area.
        var width = Math.Min(1600, workArea.Width);
        var height = Math.Min(1000, workArea.Height);
        this.AppWindow.MoveAndResize(new RectInt32(
            workArea.Left + (workArea.Width - width) / 2, workArea.Top + (workArea.Height - height) / 2, width, height));

        if (options.NoActivate) {
            NativeWindow.ShowMaximizedWithoutActivating(Win32Interop.GetWindowFromWindowId(this.AppWindow.Id), workArea);
        } else {
            this.overlapped.Maximize();
            this.Activate();
        }

        if (options.OpenMenu is { } menu) {
            this.Root.Loaded += (_, _) => this.DispatcherQueue.TryEnqueue(Microsoft.UI.Dispatching.DispatcherQueuePriority.Low, () => this.OpenMenu(menu));
        }
    }

    Displays.Rect CurrentWorkArea() {
        var area = DisplayArea.GetFromWindowId(this.AppWindow.Id, DisplayAreaFallback.Primary).WorkArea;
        return new Displays.Rect { Left = area.X, Top = area.Y, Right = area.X + area.Width, Bottom = area.Y + area.Height };
    }

    // Opens a menu of the menu bar, like File, or one of its submenus, like View/Theme, for
    // screenshots. MenuBarItem has no method that opens its menu, but its automation peer, which
    // screen readers use, does.
    void OpenMenu(string path) {
        var names = path.Split('/');
        var menu = this.AppMenuBar.Items.FirstOrDefault(item => item.Title.Equals(names[0], StringComparison.OrdinalIgnoreCase));
        if (menu == null) {
            Console.Error.WriteLine($"There's no menu {names[0]}");
            return;
        }
        Expand(menu);
        if (names.Length < 2) {
            return;
        }
        var submenu = menu.Items.OfType<MenuFlyoutSubItem>().FirstOrDefault(item => item.Text.Equals(names[1], StringComparison.OrdinalIgnoreCase));
        if (submenu == null) {
            Console.Error.WriteLine($"The {menu.Title} menu has no submenu {names[1]}");
            return;
        }
        // Once the menu has opened, with its items.
        var timer = this.DispatcherQueue.CreateTimer();
        timer.Interval = TimeSpan.FromMilliseconds(500);
        timer.IsRepeating = false;
        timer.Tick += (_, _) => Expand(submenu);
        timer.Start();
    }

    static void Expand(UIElement element) {
        var peer = FrameworkElementAutomationPeer.CreatePeerForElement(element);
        if (peer?.GetPattern(PatternInterface.ExpandCollapse) is IExpandCollapseProvider provider) {
            provider.Expand();
        }
    }

    // Navigation

    public void Navigate(string page) {
        var type = PageTypes[page];
        if (this.ContentFrame.CurrentSourcePageType != type) {
            this.ContentFrame.Navigate(type, null, new EntranceNavigationTransitionInfo());
        }
    }

    // Selects the note on the notes page, and goes there.
    public void ShowNote(Note note, bool focusTitle = false) {
        this.state.SelectedNote = note;
        if (this.ContentFrame.Content is NotesPage page) {
            if (focusTitle) {
                page.FocusTitle();
            }
        } else {
            this.ContentFrame.Navigate(typeof(NotesPage), focusTitle ? NotesPage.FocusTitleParameter : null, new EntranceNavigationTransitionInfo());
        }
    }

    void OnNavigationSelectionChanged(NavigationView sender, NavigationViewSelectionChangedEventArgs args) {
        var page = args.IsSettingsSelected ? "Settings" : (args.SelectedItem as NavigationViewItem)?.Tag as string;
        if (page != null && this.ContentFrame.CurrentSourcePageType != PageTypes[page]) {
            this.ContentFrame.Navigate(PageTypes[page], null, args.RecommendedNavigationTransitionInfo);
        }
    }

    // Keeps the pane's selection on the page, also after the back button.
    void OnNavigated(object sender, NavigationEventArgs args) {
        this.AppTitleBar.IsBackButtonEnabled = this.ContentFrame.CanGoBack;
        var page = PageTypes.First(pair => pair.Value == args.SourcePageType).Key;
        this.Navigation.SelectedItem = page == "Settings"
            ? this.Navigation.SettingsItem
            : this.Navigation.MenuItems.OfType<NavigationViewItem>().First(item => (string)item.Tag == page);
    }

    void OnBackRequested(TitleBar sender, object args) {
        if (this.ContentFrame.CanGoBack) {
            this.ContentFrame.GoBack();
        }
    }

    void OnPaneToggleRequested(TitleBar sender, object args) {
        this.TogglePane();
    }

    void OnPaneOpenedOrClosed(NavigationView sender, object args) {
        this.SyncMenus();
    }

    void TogglePane() {
        if (this.state.NavigationStyle != NavigationStyle.Top) {
            this.Navigation.IsPaneOpen = !this.Navigation.IsPaneOpen;
        }
        this.SyncMenus();
    }

    // Keyboard accelerators

    // The root's accelerators work wherever focus is in the window, and while the menus are
    // closed. The menus show the same keys with KeyboardAcceleratorTextOverride. The text boxes
    // handle Ctrl+Z, X, C, V and A themselves.
    void AddAccelerators() {
        const VirtualKeyModifiers none = VirtualKeyModifiers.None;
        const VirtualKeyModifiers control = VirtualKeyModifiers.Control;
        const VirtualKeyModifiers shift = VirtualKeyModifiers.Shift;
        // The keys beside Backspace, which have no names in VirtualKey: VK_OEM_PLUS and VK_OEM_MINUS.
        const VirtualKey plus = (VirtualKey)0xBB;
        const VirtualKey minus = (VirtualKey)0xBD;

        this.AddAccelerator(VirtualKey.N, control, this.NewNote);
        this.AddAccelerator(VirtualKey.O, control, () => _ = this.OpenAsync());
        this.AddAccelerator(VirtualKey.S, control, () => _ = this.SaveAsync(choosePath: false));
        this.AddAccelerator(VirtualKey.S, control | shift, () => _ = this.SaveAsync(choosePath: true));
        this.AddAccelerator(VirtualKey.F, control, this.FocusSearch);
        this.AddAccelerator(VirtualKey.D, control, this.TogglePin);
        this.AddAccelerator(plus, control, this.ZoomIn);
        this.AddAccelerator(VirtualKey.Add, control, this.ZoomIn);
        this.AddAccelerator(minus, control, this.ZoomOut);
        this.AddAccelerator(VirtualKey.Subtract, control, this.ZoomOut);
        this.AddAccelerator(VirtualKey.Number0, control, this.ZoomReset);
        this.AddAccelerator(VirtualKey.NumberPad0, control, this.ZoomReset);
        this.AddAccelerator(VirtualKey.Number1, control, () => this.Navigate("Home"));
        this.AddAccelerator(VirtualKey.Number2, control, () => this.Navigate("Notes"));
        this.AddAccelerator(VirtualKey.Number3, control, () => this.Navigate("Controls"));
        this.AddAccelerator(VirtualKey.Number4, control, () => this.Navigate("Settings"));
        this.AddAccelerator(VirtualKey.F1, none, () => _ = this.ShowShortcutsAsync());
        this.AddAccelerator(VirtualKey.F9, none, this.SwitchTheme);
        this.AddAccelerator(VirtualKey.F11, none, this.ToggleFullScreen);
    }

    void AddAccelerator(VirtualKey key, VirtualKeyModifiers modifiers, Action action) {
        var accelerator = new KeyboardAccelerator { Key = key, Modifiers = modifiers };
        accelerator.Invoked += (_, args) => {
            args.Handled = true;
            action();
        };
        this.Root.KeyboardAccelerators.Add(accelerator);
    }

    // File

    public void NewNote() {
        var note = this.state.AddNote();
        this.state.StatusMessage = "Started a new note";
        this.ShowNote(note, focusTitle: true);
    }

    void OnNewNote(object sender, RoutedEventArgs args) {
        this.NewNote();
    }

    void OnNewFromTemplate(object sender, RoutedEventArgs args) {
        var (title, body) = (string)((FrameworkElement)sender).Tag switch {
            "Meeting" => ("Meeting notes", "Attendees:\n\nAgenda\n- \n\nDecisions\n- \n\nAction items\n- "),
            "ToDo" => ("To-do list", "[ ] \n[ ] \n[ ] "),
            _ => ($"Journal, {DateTime.Now:MMMM d}", "What happened today:\n\n\nWhat went well:\n\n\nWhat to try tomorrow:\n"),
        };
        var note = this.state.AddNote(title, body);
        this.state.StatusMessage = $"Started {title.ToLowerInvariant()} from a template";
        this.ShowNote(note);
    }

    // Opens a text file as a new note, with the Windows App SDK's file picker, which takes the
    // window's ID instead of needing a package or InitializeWithWindow.
    public async Task OpenAsync() {
        var picker = new FileOpenPicker(this.AppWindow.Id) { SuggestedStartLocation = PickerLocationId.DocumentsLibrary };
        picker.FileTypeFilter.Add(".txt");
        picker.FileTypeFilter.Add(".md");
        var result = await picker.PickSingleFileAsync();
        if (result == null) {
            return;
        }
        try {
            var text = await File.ReadAllTextAsync(result.Path);
            var note = this.state.AddNote(Path.GetFileNameWithoutExtension(result.Path), text);
            note.FilePath = result.Path;
            this.state.StatusMessage = $"Opened {Path.GetFileName(result.Path)}";
            this.ShowNote(note);
        } catch (IOException exception) {
            this.state.StatusMessage = $"Couldn't open {Path.GetFileName(result.Path)}: {exception.Message}";
        }
    }

    void OnOpen(object sender, RoutedEventArgs args) {
        _ = this.OpenAsync();
    }

    // Saves the note to its file, or asks where first.
    async Task SaveAsync(bool choosePath) {
        if (this.state.SelectedNote is not { } note) {
            this.state.StatusMessage = "There's no note to save";
            return;
        }
        var path = note.FilePath;
        if (choosePath || path == null) {
            var name = string.Join("-", note.DisplayTitle.Split(Path.GetInvalidFileNameChars()));
            var picker = new FileSavePicker(this.AppWindow.Id) {
                SuggestedFileName = name,
                SuggestedStartLocation = PickerLocationId.DocumentsLibrary,
            };
            picker.FileTypeChoices.Add("Text document", [".txt"]);
            picker.FileTypeChoices.Add("Markdown", [".md"]);
            var result = await picker.PickSaveFileAsync();
            if (result == null) {
                return;
            }
            path = result.Path;
        }
        try {
            await File.WriteAllTextAsync(path, note.Body.Replace("\r", "\r\n"));
            note.FilePath = path;
            this.state.StatusMessage = $"Saved {Path.GetFileName(path)}";
        } catch (IOException exception) {
            this.state.StatusMessage = $"Couldn't save {Path.GetFileName(path)}: {exception.Message}";
        }
    }

    void OnSave(object sender, RoutedEventArgs args) {
        _ = this.SaveAsync(choosePath: false);
    }

    void OnSaveAs(object sender, RoutedEventArgs args) {
        _ = this.SaveAsync(choosePath: true);
    }

    void OnExit(object sender, RoutedEventArgs args) {
        this.Close();
    }

    // Edit

    // The text box that Edit's commands work on, if it's still on the page.
    TextBox? EditTarget() {
        if (this.lastTextBox is { IsLoaded: true } box) {
            return box;
        }
        this.state.StatusMessage = "Click in a text box first";
        return null;
    }

    void Edit(Action<TextBox> action) {
        if (this.EditTarget() is { } box) {
            action(box);
            box.Focus(FocusState.Programmatic);
        }
    }

    void OnUndo(object sender, RoutedEventArgs args) {
        this.Edit(box => box.Undo());
    }

    void OnRedo(object sender, RoutedEventArgs args) {
        this.Edit(box => box.Redo());
    }

    void OnCut(object sender, RoutedEventArgs args) {
        this.Edit(box => box.CutSelectionToClipboard());
    }

    void OnCopy(object sender, RoutedEventArgs args) {
        this.Edit(box => box.CopySelectionToClipboard());
    }

    void OnPaste(object sender, RoutedEventArgs args) {
        this.Edit(box => box.PasteFromClipboard());
    }

    void OnSelectAll(object sender, RoutedEventArgs args) {
        this.Edit(box => box.SelectAll());
    }

    void FocusSearch() {
        this.SearchBox.Focus(FocusState.Keyboard);
    }

    void OnFind(object sender, RoutedEventArgs args) {
        this.FocusSearch();
    }

    public void TogglePin() {
        if (this.state.SelectedNote is { } note) {
            note.IsPinned = !note.IsPinned;
            this.state.StatusMessage = note.IsPinned ? $"Pinned {note.DisplayTitle}" : $"Unpinned {note.DisplayTitle}";
        }
        this.SyncMenus();
    }

    void OnTogglePin(object sender, RoutedEventArgs args) {
        this.TogglePin();
    }

    public async Task DeleteNoteAsync() {
        if (this.state.SelectedNote is not { } note) {
            return;
        }
        var dialog = new ContentDialog {
            Title = "Delete this note?",
            Content = $"{note.DisplayTitle} will be deleted. This can't be undone.",
            PrimaryButtonText = "Delete",
            CloseButtonText = "Cancel",
            DefaultButton = ContentDialogButton.Close,
        };
        if (await this.ShowDialogAsync(dialog) == ContentDialogResult.Primary) {
            this.state.DeleteNote(note);
            this.state.StatusMessage = $"Deleted {note.DisplayTitle}";
        }
    }

    void OnDeleteNote(object sender, RoutedEventArgs args) {
        _ = this.DeleteNoteAsync();
    }

    // Search

    IEnumerable<Note> Matches(string text) {
        return this.state.Notes.Where(note =>
            note.Title.Contains(text, StringComparison.CurrentCultureIgnoreCase) || note.Body.Contains(text, StringComparison.CurrentCultureIgnoreCase));
    }

    void OnSearchTextChanged(AutoSuggestBox sender, AutoSuggestBoxTextChangedEventArgs args) {
        if (args.Reason == AutoSuggestionBoxTextChangeReason.UserInput) {
            var text = sender.Text.Trim();
            sender.ItemsSource = text.Length == 0 ? null : this.Matches(text).ToList();
        }
    }

    void OnSearchSuggestionChosen(AutoSuggestBox sender, AutoSuggestBoxSuggestionChosenEventArgs args) {
        sender.Text = ((Note)args.SelectedItem).DisplayTitle;
    }

    void OnSearchQuerySubmitted(AutoSuggestBox sender, AutoSuggestBoxQuerySubmittedEventArgs args) {
        var note = args.ChosenSuggestion as Note ?? this.Matches(args.QueryText.Trim()).FirstOrDefault();
        if (note == null) {
            this.state.StatusMessage = $"No notes have {args.QueryText.Trim()} in them";
            return;
        }
        this.ShowNote(note);
    }

    // View

    void ApplyTheme() {
        this.Root.RequestedTheme = this.state.Theme;
        // The caption buttons, which Windows draws, follow the app's theme too.
        this.AppWindow.TitleBar.PreferredTheme = this.state.Theme switch {
            ElementTheme.Light => TitleBarTheme.Light,
            ElementTheme.Dark => TitleBarTheme.Dark,
            _ => TitleBarTheme.UseDefaultAppMode,
        };
    }

    void SwitchTheme() {
        this.state.Theme = this.Root.ActualTheme == ElementTheme.Dark ? ElementTheme.Light : ElementTheme.Dark;
    }

    void OnSwitchTheme(object sender, RoutedEventArgs args) {
        this.SwitchTheme();
    }

    void OnThemeChosen(object sender, RoutedEventArgs args) {
        this.state.Theme = (string)((FrameworkElement)sender).Tag switch {
            "Light" => ElementTheme.Light,
            "Dark" => ElementTheme.Dark,
            _ => ElementTheme.Default,
        };
    }

    void ApplyBackdrop() {
        this.SystemBackdrop = this.state.Backdrop switch {
            Backdrop.Mica => new MicaBackdrop(),
            Backdrop.MicaAlt => new MicaBackdrop { Kind = MicaKind.BaseAlt },
            Backdrop.Acrylic => new DesktopAcrylicBackdrop(),
            _ => null,
        };
        this.SolidBackground.Visibility = this.state.Backdrop == Backdrop.None ? Visibility.Visible : Visibility.Collapsed;
    }

    void ApplyNavigationStyle() {
        this.Navigation.PaneDisplayMode = this.state.NavigationStyle switch {
            NavigationStyle.LeftCompact => NavigationViewPaneDisplayMode.LeftCompact,
            NavigationStyle.Top => NavigationViewPaneDisplayMode.Top,
            _ => NavigationViewPaneDisplayMode.Left,
        };
        this.AppTitleBar.IsPaneToggleButtonVisible = this.state.NavigationStyle != NavigationStyle.Top;
    }

    void OnTogglePane(object sender, RoutedEventArgs args) {
        this.TogglePane();
    }

    void OnToggleStatusBar(object sender, RoutedEventArgs args) {
        this.state.ShowStatusBar = !this.state.ShowStatusBar;
    }

    void ZoomIn() {
        this.state.Zoom += 10;
    }

    void ZoomOut() {
        this.state.Zoom -= 10;
    }

    void ZoomReset() {
        this.state.Zoom = AppState.DefaultZoom;
    }

    void OnZoomIn(object sender, RoutedEventArgs args) {
        this.ZoomIn();
    }

    void OnZoomOut(object sender, RoutedEventArgs args) {
        this.ZoomOut();
    }

    void OnZoomReset(object sender, RoutedEventArgs args) {
        this.ZoomReset();
    }

    void OnGoTo(object sender, RoutedEventArgs args) {
        this.Navigate((string)((FrameworkElement)sender).Tag);
    }

    // Switches between maximized and full screen. Full screen has no title bar.
    void ToggleFullScreen() {
        if (this.AppWindow.Presenter.Kind == AppWindowPresenterKind.FullScreen) {
            this.AppWindow.SetPresenter(this.overlapped);
        } else {
            this.AppWindow.SetPresenter(AppWindowPresenterKind.FullScreen);
        }
    }

    void OnToggleFullScreen(object sender, RoutedEventArgs args) {
        this.ToggleFullScreen();
    }

    void OnAppWindowChanged(AppWindow sender, AppWindowChangedEventArgs args) {
        if (args.DidPresenterChange) {
            var fullScreen = sender.Presenter.Kind == AppWindowPresenterKind.FullScreen;
            this.AppTitleBar.Visibility = fullScreen ? Visibility.Collapsed : Visibility.Visible;
            this.SyncMenus();
        }
    }

    // Help

    async Task ShowShortcutsAsync() {
        (string[] Keys, string Action)[] shortcuts = [
            (["Ctrl", "N"], "Start a new note"),
            (["Ctrl", "O"], "Open a text file as a note"),
            (["Ctrl", "S"], "Save the note"),
            (["Ctrl", "Shift", "S"], "Save the note as a new file"),
            (["Ctrl", "F"], "Search the notes"),
            (["Ctrl", "D"], "Pin or unpin the note"),
            (["Ctrl", "+"], "Zoom in"),
            (["Ctrl", "-"], "Zoom out"),
            (["Ctrl", "0"], "Restore the default zoom"),
            (["Ctrl", "1"], "Go to Home, and 2, 3 and 4 to Notes, Controls and Settings"),
            (["Alt"], "Show the menu bar's access keys"),
            (["F1"], "Show the keyboard shortcuts"),
            (["F9"], "Switch between the light and the dark theme"),
            (["F11"], "Switch full screen"),
        ];
        var grid = new Grid { ColumnSpacing = 24, RowSpacing = 10 };
        grid.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });
        grid.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
        var keycap = (Style)Application.Current.Resources["KeycapStyle"];
        for (var row = 0; row < shortcuts.Length; row++) {
            grid.RowDefinitions.Add(new RowDefinition { Height = GridLength.Auto });
            var keys = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 4 };
            foreach (var key in shortcuts[row].Keys) {
                keys.Children.Add(new Border {
                    Style = keycap,
                    Child = new TextBlock { Text = key, HorizontalAlignment = HorizontalAlignment.Center, Style = (Style)Application.Current.Resources["CaptionTextBlockStyle"] },
                });
            }
            var action = new TextBlock { Text = shortcuts[row].Action, VerticalAlignment = VerticalAlignment.Center, TextWrapping = TextWrapping.Wrap };
            Grid.SetRow(keys, row);
            Grid.SetRow(action, row);
            Grid.SetColumn(action, 1);
            grid.Children.Add(keys);
            grid.Children.Add(action);
        }
        await this.ShowDialogAsync(new ContentDialog {
            Title = "Keyboard shortcuts",
            Content = new ScrollViewer { Content = grid },
            CloseButtonText = "Close",
            DefaultButton = ContentDialogButton.Close,
        });
    }

    void OnShowShortcuts(object sender, RoutedEventArgs args) {
        _ = this.ShowShortcutsAsync();
    }

    void OnOpenDocumentation(object sender, RoutedEventArgs args) {
        _ = Launcher.LaunchUriAsync(new Uri(DocumentationUri));
    }

    async Task ShowAboutAsync() {
        var header = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 16 };
        header.Children.Add(new Image { Source = new BitmapImage(new Uri("ms-appx:///Assets/AppIcon.png")), Width = 64, Height = 64 });
        var names = new StackPanel { VerticalAlignment = VerticalAlignment.Center };
        names.Children.Add(new TextBlock { Text = "WinUI 3 Sample", Style = (Style)Application.Current.Resources["SubtitleTextBlockStyle"] });
        names.Children.Add(new TextBlock { Text = "Version 1.0", Style = (Style)Application.Current.Resources["SecondaryTextStyle"] });
        header.Children.Add(names);

        var content = new StackPanel { Spacing = 12 };
        content.Children.Add(header);
        content.Children.Add(new TextBlock {
            Text = "A small notes app that shows what a WinUI 3 app looks like: Mica, a title bar of its own, a menu bar, navigation, and WinUI 3's controls in their Fluent styles.",
            TextWrapping = TextWrapping.Wrap,
        });
        content.Children.Add(new TextBlock {
            Text = Versions.Summary,
            TextWrapping = TextWrapping.Wrap,
            Style = (Style)Application.Current.Resources["SecondaryTextStyle"],
        });
        content.Children.Add(new HyperlinkButton { Content = "WinUI 3 documentation", NavigateUri = new Uri(DocumentationUri), Padding = new Thickness(0) });
        await this.ShowDialogAsync(new ContentDialog {
            Title = "About",
            Content = content,
            CloseButtonText = "OK",
            DefaultButton = ContentDialogButton.Close,
        });
    }

    void OnAbout(object sender, RoutedEventArgs args) {
        _ = this.ShowAboutAsync();
    }

    // Shows a dialog in the window, in its theme. One at a time, as WinUI allows.
    public async Task<ContentDialogResult> ShowDialogAsync(ContentDialog dialog) {
        if (this.dialogOpen) {
            return ContentDialogResult.None;
        }
        this.dialogOpen = true;
        try {
            dialog.XamlRoot = this.Root.XamlRoot;
            dialog.RequestedTheme = this.Root.ActualTheme;
            dialog.Style = (Style)Application.Current.Resources["DefaultContentDialogStyle"];
            return await dialog.ShowAsync();
        } finally {
            this.dialogOpen = false;
        }
    }

    // State

    void OnStateChanged(object? sender, PropertyChangedEventArgs args) {
        switch (args.PropertyName) {
        case nameof(AppState.Theme):
            this.ApplyTheme();
            break;
        case nameof(AppState.Backdrop):
            this.ApplyBackdrop();
            break;
        case nameof(AppState.NavigationStyle):
            this.ApplyNavigationStyle();
            break;
        case nameof(AppState.SelectedNote):
            this.WatchSelectedNote();
            break;
        }
        this.SyncMenus();
        this.UpdateStatusBar();
    }

    // Follows the selected note's words for the status bar, and whether it's pinned for the menu.
    void WatchSelectedNote() {
        if (this.watchedNote != null) {
            this.watchedNote.PropertyChanged -= this.OnSelectedNoteChanged;
        }
        this.watchedNote = this.state.SelectedNote;
        if (this.watchedNote != null) {
            this.watchedNote.PropertyChanged += this.OnSelectedNoteChanged;
        }
    }

    void OnSelectedNoteChanged(object? sender, PropertyChangedEventArgs args) {
        if (args.PropertyName is nameof(Note.WordCount) or nameof(Note.IsPinned)) {
            this.SyncMenus();
            this.UpdateStatusBar();
        }
    }

    // Checks the menus' toggles and radio items by what they show.
    void SyncMenus() {
        this.LightThemeItem.IsChecked = this.state.Theme == ElementTheme.Light;
        this.DarkThemeItem.IsChecked = this.state.Theme == ElementTheme.Dark;
        this.SystemThemeItem.IsChecked = this.state.Theme == ElementTheme.Default;
        var hasPane = this.state.NavigationStyle != NavigationStyle.Top;
        this.PaneMenuItem.IsEnabled = hasPane;
        this.PaneMenuItem.IsChecked = hasPane && this.Navigation.IsPaneOpen;
        this.StatusBarMenuItem.IsChecked = this.state.ShowStatusBar;
        this.FullScreenMenuItem.IsChecked = this.AppWindow.Presenter.Kind == AppWindowPresenterKind.FullScreen;
        this.PinMenuItem.IsEnabled = this.state.SelectedNote != null;
        this.PinMenuItem.IsChecked = this.state.SelectedNote?.IsPinned == true;
    }

    void UpdateStatusBar() {
        this.StatusBar.Visibility = this.state.ShowStatusBar ? Visibility.Visible : Visibility.Collapsed;
        this.StatusMessage.Text = this.state.StatusMessage;
        this.StatusWords.Text = this.state.SelectedNote is { } note
            ? note.WordCount == 1 ? "1 word" : $"{note.WordCount} words"
            : "No note";
        this.StatusZoom.Text = $"{this.state.Zoom}%";
        this.StatusTheme.Text = this.Root.ActualTheme == ElementTheme.Dark ? "Dark theme" : "Light theme";
    }

    void UpdateNotesBadge() {
        this.NotesBadge.Value = this.state.Notes.Count;
    }
}
