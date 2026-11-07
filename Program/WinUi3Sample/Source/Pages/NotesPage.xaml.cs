using System.ComponentModel;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Navigation;

namespace WinUi3Sample;

// The notes, in a list beside an editor of the selected one. The menu bar's File and Edit menus
// work on them too.
public sealed partial class NotesPage : Page {
    public const string FocusTitleParameter = "FocusTitle";

    bool focusTitleOnLoad;

    public NotesPage() {
        this.InitializeComponent();
        this.Loaded += (_, _) => {
            if (this.focusTitleOnLoad) {
                this.focusTitleOnLoad = false;
                this.FocusTitle();
            }
        };
    }

    public AppState State => App.State;

    protected override void OnNavigatedTo(NavigationEventArgs e) {
        base.OnNavigatedTo(e);
        this.focusTitleOnLoad = e.Parameter as string == FocusTitleParameter;
        this.State.PropertyChanged += this.OnStateChanged;
        this.State.Notes.CollectionChanged += this.OnNotesChanged;
        this.SelectInList();
        this.UpdateCount();
    }

    protected override void OnNavigatedFrom(NavigationEventArgs e) {
        base.OnNavigatedFrom(e);
        this.State.PropertyChanged -= this.OnStateChanged;
        this.State.Notes.CollectionChanged -= this.OnNotesChanged;
    }

    // Puts the caret in the title, as a new note has none yet.
    public void FocusTitle() {
        // Once the editor shows, if the page had no note before.
        this.DispatcherQueue.TryEnqueue(() => this.TitleBox.Focus(FocusState.Programmatic));
    }

    void OnStateChanged(object? sender, PropertyChangedEventArgs args) {
        if (args.PropertyName == nameof(AppState.SelectedNote)) {
            this.SelectInList();
        }
    }

    void OnNotesChanged(object? sender, System.Collections.Specialized.NotifyCollectionChangedEventArgs args) {
        this.UpdateCount();
    }

    void SelectInList() {
        if (this.NoteList.SelectedItem != this.State.SelectedNote) {
            this.NoteList.SelectedItem = this.State.SelectedNote;
        }
        if (this.State.SelectedNote != null) {
            this.NoteList.ScrollIntoView(this.State.SelectedNote);
        }
    }

    void UpdateCount() {
        var count = this.State.Notes.Count;
        var pinned = this.State.Notes.Count(note => note.IsPinned);
        this.NoteCount.Text = $"{count} {(count == 1 ? "note" : "notes")}, {pinned} pinned";
    }

    void OnSelectionChanged(object sender, SelectionChangedEventArgs args) {
        // A deleted note leaves nothing selected for a moment, until AppState picks the next.
        if (this.NoteList.SelectedItem is Note note && note != this.State.SelectedNote) {
            this.State.SelectedNote = note;
        }
    }

    void OnNewNote(object sender, RoutedEventArgs args) {
        App.Window.NewNote();
    }

    void OnPinClick(object sender, RoutedEventArgs args) {
        App.Window.TogglePin();
        this.UpdateCount();
    }

    void OnDelete(object sender, RoutedEventArgs args) {
        _ = App.Window.DeleteNoteAsync();
    }

    void OnDuplicate(object sender, RoutedEventArgs args) {
        if (this.State.SelectedNote is { } note) {
            this.State.AddNote($"{note.DisplayTitle} (copy)", note.Body);
            this.State.StatusMessage = $"Duplicated {note.DisplayTitle}";
        }
    }

    void OnSortByTitle(object sender, RoutedEventArgs args) {
        this.Sort(this.State.Notes.OrderBy(note => note.DisplayTitle, StringComparer.CurrentCultureIgnoreCase).ToList());
        this.State.StatusMessage = "Sorted the notes by title";
    }

    void OnSortByDate(object sender, RoutedEventArgs args) {
        this.Sort(this.State.Notes.OrderByDescending(note => note.Modified).ToList());
        this.State.StatusMessage = "Sorted the notes by the date they were edited";
    }

    // Moves the notes into the order, which keeps the list's selection.
    void Sort(List<Note> order) {
        for (var index = 0; index < order.Count; index++) {
            var from = this.State.Notes.IndexOf(order[index]);
            if (from != index) {
                this.State.Notes.Move(from, index);
            }
        }
    }
}
