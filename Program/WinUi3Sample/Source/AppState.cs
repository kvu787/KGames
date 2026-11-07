using System.Collections.ObjectModel;
using Microsoft.UI.Xaml;

namespace WinUi3Sample;

public enum Backdrop { Mica, MicaAlt, Acrylic, None }

public enum NavigationStyle { Left, LeftCompact, Top }

// The notes and the settings, which the window and the pages bind to. The window applies the
// settings when they change, whichever page or menu changed them.
public sealed partial class AppState : ObservableObject {
    public const int DefaultZoom = 100;

    ElementTheme theme = ElementTheme.Light;
    Backdrop backdrop = Backdrop.Mica;
    NavigationStyle navigationStyle = NavigationStyle.Left;
    bool showStatusBar = true;
    bool spellCheck = true;
    int zoom = DefaultZoom;
    Note? selectedNote;
    string statusMessage = "Ready";

    public AppState() {
        this.Notes = SampleNotes();
        this.selectedNote = this.Notes[0];
    }

    public ObservableCollection<Note> Notes { get; }

    public Note? SelectedNote {
        get => this.selectedNote;
        set => this.Set(ref this.selectedNote, value);
    }

    // Light by default, like every KGames program. Default follows Windows' app mode.
    public ElementTheme Theme {
        get => this.theme;
        set {
            if (this.Set(ref this.theme, value)) {
                this.OnPropertyChanged(nameof(this.ThemeIndex));
            }
        }
    }

    // For the settings page's combo box: Light, Dark, and Use system setting.
    public int ThemeIndex {
        get => this.theme switch { ElementTheme.Light => 0, ElementTheme.Dark => 1, _ => 2 };
        set => this.Theme = value switch { 0 => ElementTheme.Light, 1 => ElementTheme.Dark, _ => ElementTheme.Default };
    }

    public Backdrop Backdrop {
        get => this.backdrop;
        set {
            if (this.Set(ref this.backdrop, value)) {
                this.OnPropertyChanged(nameof(this.BackdropIndex));
            }
        }
    }

    public int BackdropIndex {
        get => (int)this.backdrop;
        set => this.Backdrop = (Backdrop)Math.Clamp(value, 0, 3);
    }

    public NavigationStyle NavigationStyle {
        get => this.navigationStyle;
        set {
            if (this.Set(ref this.navigationStyle, value)) {
                this.OnPropertyChanged(nameof(this.NavigationStyleIndex));
            }
        }
    }

    public int NavigationStyleIndex {
        get => (int)this.navigationStyle;
        set => this.NavigationStyle = (NavigationStyle)Math.Clamp(value, 0, 2);
    }

    public bool ShowStatusBar {
        get => this.showStatusBar;
        set => this.Set(ref this.showStatusBar, value);
    }

    public bool SpellCheck {
        get => this.spellCheck;
        set => this.Set(ref this.spellCheck, value);
    }

    // The editor's zoom, in percent.
    public int Zoom {
        get => this.zoom;
        set {
            if (this.Set(ref this.zoom, Math.Clamp(value, 50, 300))) {
                this.OnPropertyChanged(nameof(this.ZoomValue));
                this.OnPropertyChanged(nameof(this.EditorFontSize));
            }
        }
    }

    // The zoom for a slider, whose values are doubles.
    public double ZoomValue {
        get => this.zoom;
        set => this.Zoom = (int)Math.Round(value);
    }

    public double EditorFontSize => 15.0 * this.zoom / 100;

    // What the status bar says happened last, like "Saved Groceries.txt".
    public string StatusMessage {
        get => this.statusMessage;
        set => this.Set(ref this.statusMessage, value);
    }

    // Starts a note, from a template of File > New from template.
    public Note AddNote(string title = "", string body = "") {
        var note = new Note(title, body, DateTimeOffset.Now);
        this.Notes.Insert(0, note);
        this.SelectedNote = note;
        return note;
    }

    public void DeleteNote(Note note) {
        var index = this.Notes.IndexOf(note);
        if (index < 0) {
            return;
        }
        this.Notes.RemoveAt(index);
        if (this.SelectedNote == note) {
            this.SelectedNote = this.Notes.Count == 0 ? null : this.Notes[Math.Min(index, this.Notes.Count - 1)];
        }
    }

    static ObservableCollection<Note> SampleNotes() {
        var now = DateTimeOffset.Now;
        return [
            new Note(
                "Welcome to the WinUI 3 sample",
                """
                This is a small notes app, made with WinUI 3 and the Windows App SDK, to show what a WinUI 3 app looks like.

                Try the menu bar along the top. File starts, opens and saves notes. Edit works on the text box that has focus. View switches the theme, the navigation pane, the status bar, the zoom and full screen. Help has the keyboard shortcuts.

                F9 switches between the light and the dark theme, and F11 switches full screen.

                The Controls page has most of WinUI 3's common controls, and the Settings page changes the window's backdrop and the navigation's style.
                """,
                now.AddMinutes(-4),
                isPinned: true),
            new Note(
                "Groceries",
                """
                Oat milk
                Eggs
                Spinach
                Two lemons
                Rice, the big bag
                Coffee beans
                Something for dessert
                """,
                now.AddHours(-3)),
            new Note(
                "Trip to Kyoto",
                """
                Day 1: Arrive at Kansai, train to Kyoto Station, check in near Shijo.
                Day 2: Fushimi Inari early, before the crowds. Tofuku-ji in the afternoon.
                Day 3: Arashiyama bamboo grove, the monkey park, and a boat on the Hozu river.
                Day 4: Day trip to Nara.

                Book the ryokan for the last night.
                """,
                now.AddDays(-1).AddHours(-2),
                isPinned: true),
            new Note(
                "Q4 planning meeting",
                """
                Attendees: Alex, Sam, Priya, Jordan

                Decisions
                - Ship the new onboarding in November.
                - Move the settings redesign to next quarter.

                Action items
                - Sam: draft the release notes.
                - Priya: book the usability study.
                - Jordan: check the crash reports from the last build.
                """,
                now.AddDays(-2)),
            new Note(
                "Book ideas",
                """
                A lighthouse keeper who receives letters from the future.
                A city where it rains only on Tuesdays.
                Two rival bakers who have to share one oven.
                """,
                now.AddDays(-5)),
            new Note(
                "Workout plan",
                """
                Monday: run, 5 km
                Wednesday: legs and core
                Friday: run, intervals
                Saturday: long walk or a hike
                """,
                now.AddDays(-12)),
        ];
    }
}
