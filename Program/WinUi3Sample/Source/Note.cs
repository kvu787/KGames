using System.Globalization;

namespace WinUi3Sample;

public sealed partial class Note : ObservableObject {
    string title;
    string body;
    DateTimeOffset modified;
    bool isPinned;
    string? filePath;

    public Note(string title, string body, DateTimeOffset modified, bool isPinned = false) {
        this.title = title;
        this.body = body;
        this.modified = modified;
        this.isPinned = isPinned;
    }

    public string Title {
        get => this.title;
        set {
            if (this.Set(ref this.title, value)) {
                this.OnPropertyChanged(nameof(this.DisplayTitle));
                this.Touch();
            }
        }
    }

    public string DisplayTitle => string.IsNullOrWhiteSpace(this.title) ? "Untitled note" : this.title;

    public string Body {
        get => this.body;
        set {
            if (this.Set(ref this.body, value)) {
                this.OnPropertyChanged(nameof(this.Preview));
                this.OnPropertyChanged(nameof(this.WordCount));
                // Touch raises Summary's.
                this.Touch();
            }
        }
    }

    // The start of the body, on one line, for the list of notes.
    public string Preview {
        get {
            var flat = string.Join(' ', this.body.Split(['\r', '\n'], StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries));
            return flat.Length == 0 ? "No text" : flat.Length <= 160 ? flat : flat[..160];
        }
    }

    public int WordCount => this.body.Split((char[]?)null, StringSplitOptions.RemoveEmptyEntries).Length;

    public DateTimeOffset Modified => this.modified;

    // Like "Today at 9:41 AM", "Yesterday at 6:02 PM" or "Nov 2, 2026".
    public string ModifiedText {
        get {
            var culture = CultureInfo.CurrentCulture;
            var local = this.modified.ToLocalTime();
            var days = (DateTime.Today - local.Date).Days;
            return days switch {
                0 => $"Today at {local.ToString("t", culture)}",
                1 => $"Yesterday at {local.ToString("t", culture)}",
                < 7 => local.ToString("dddd", culture),
                _ => local.ToString("MMM d, yyyy", culture),
            };
        }
    }

    // Like "Edited today at 9:41 AM  ·  102 words".
    public string Summary {
        get {
            var culture = CultureInfo.CurrentCulture;
            var local = this.modified.ToLocalTime();
            var edited = (DateTime.Today - local.Date).Days switch {
                0 => $"today at {local.ToString("t", culture)}",
                1 => $"yesterday at {local.ToString("t", culture)}",
                < 7 => $"on {local.ToString("dddd", culture)}",
                _ => $"on {local.ToString("MMM d, yyyy", culture)}",
            };
            var words = this.WordCount == 1 ? "1 word" : $"{this.WordCount} words";
            return $"Edited {edited}  ·  {words}";
        }
    }

    public bool IsPinned {
        get => this.isPinned;
        set => this.Set(ref this.isPinned, value);
    }

    // The file that File > Save writes to, once the note was opened from one or saved as one.
    public string? FilePath {
        get => this.filePath;
        set => this.Set(ref this.filePath, value);
    }

    void Touch() {
        this.modified = DateTimeOffset.Now;
        this.OnPropertyChanged(nameof(this.Modified));
        this.OnPropertyChanged(nameof(this.ModifiedText));
        this.OnPropertyChanged(nameof(this.Summary));
    }
}
